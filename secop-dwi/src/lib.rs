//! Dual Wire Interface (DWI) driver for Secop CCD compressors (NLV/SLVE).
#![no_std]
#![warn(missing_docs)]

pub use compressor_dwi::Error;
use compressor_dwi::{Command, Driver};
use embedded_io_async::{Read, ReadReady, Write};

/// Operation status of the compressor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    /// Whether the compressor is running.
    pub running: bool,
    /// Faults, or `None` if no fault bit is set.
    pub faults: Option<Faults>,
}

impl Status {
    const fn from_bits(raw: u16) -> Self {
        Self {
            running: raw & 0xFF00 == 0,
            faults: Faults::from_bits((raw & 0x00FF) as u8),
        }
    }
}

/// Fault flags reported by the compressor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Faults {
    /// `0x01` start fail.
    pub start_fail: bool,
    /// `0x04` under speed.
    pub under_speed: bool,
    /// `0x08` wrong rotor position.
    pub wrong_rotor_position: bool,
    /// `0x10` motor cable fail.
    pub motor_cable_fail: bool,
    /// `0x20` over-temperature (PCB).
    pub over_temperature: bool,
    /// `0x40` serial fail.
    pub serial_fail: bool,
    /// `0x80` set speed out of range.
    pub speed_out_of_range: bool,
}

impl Faults {
    const fn from_bits(bits: u8) -> Option<Self> {
        if bits == 0 {
            return None;
        }

        Some(Self {
            start_fail: bits & 0x01 != 0,
            under_speed: bits & 0x04 != 0,
            wrong_rotor_position: bits & 0x08 != 0,
            motor_cable_fail: bits & 0x10 != 0,
            over_temperature: bits & 0x20 != 0,
            serial_fail: bits & 0x40 != 0,
            speed_out_of_range: bits & 0x80 != 0,
        })
    }
}

/// Secop CCD compressor driver.
pub struct Compressor<TX, RX> {
    inner: Driver<TX, RX>,
}

impl<TX: Write, RX: Read + ReadReady> Compressor<TX, RX> {
    /// Create a driver.
    pub const fn new(tx: TX, rx: RX) -> Self {
        Self {
            inner: Driver::new(tx, rx),
        }
    }

    /// Give back the serial halves.
    pub fn release(self) -> (TX, RX) {
        self.inner.release()
    }

    /// Set the speed (RPM), and read back the resulting status.
    pub async fn set_speed(&mut self, rpm: u16) -> Result<Status, Error> {
        Ok(Status::from_bits(
            self.inner.transact(&Command::new(0xC3, rpm, 0x83)).await?,
        ))
    }

    /// Read the currently set speed (RPM).
    pub async fn read_set_speed(&mut self) -> Result<u16, Error> {
        self.inner.transact(&Command::new(0x3C, 0x3980, 0x80)).await
    }

    /// Read the operation-status word.
    pub async fn read_status(&mut self) -> Result<Status, Error> {
        Ok(Status::from_bits(
            self.inner.transact(&Command::new(0x3C, 0x3983, 0x83)).await?,
        ))
    }

    /// Read the measured compressor speed (RPM).
    pub async fn read_actual_speed(&mut self) -> Result<u16, Error> {
        self.inner.transact(&Command::new(0x3C, 0x3986, 0x86)).await
    }

    /// Read the power consumption (W).
    pub async fn read_power(&mut self) -> Result<u16, Error> {
        self.inner.transact(&Command::new(0x3C, 0x3982, 0x82)).await
    }

    /// Read the compressor temperature in 0.1 °C.
    pub async fn read_temperature(&mut self) -> Result<i16, Error> {
        Ok(self.inner.transact(&Command::new(0x3C, 0x3988, 0x88)).await? as i16)
    }

    /// Read the input voltage (V).
    pub async fn read_input_voltage(&mut self) -> Result<u16, Error> {
        self.inner.transact(&Command::new(0x3B, 0x0101, 0x01)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type FaultCase = (u16, fn(&Faults) -> bool);

    #[test]
    fn request_frames_match() {
        let cases: [(Command, [u8; 5]); 12] = [
            (Command::new(0xC3, 3000, 0x83), [0xA5, 0xC3, 0xB8, 0x0B, 0xD5]),
            (Command::new(0x3C, 0x3980, 0x80), [0xA5, 0x3C, 0x80, 0x39, 0x66]),
            (Command::new(0x3C, 0x3982, 0x82), [0xA5, 0x3C, 0x82, 0x39, 0x64]),
            (Command::new(0x3C, 0x3983, 0x83), [0xA5, 0x3C, 0x83, 0x39, 0x63]),
            (Command::new(0x3C, 0x3986, 0x86), [0xA5, 0x3C, 0x86, 0x39, 0x60]),
            (Command::new(0x3C, 0x3988, 0x88), [0xA5, 0x3C, 0x88, 0x39, 0x5E]),
            (Command::new(0x3B, 0x0100, 0x00), [0xA5, 0x3B, 0x00, 0x01, 0x1F]),
            (Command::new(0x3B, 0x0101, 0x01), [0xA5, 0x3B, 0x01, 0x01, 0x1E]),
            (Command::new(0x3B, 0x0102, 0x02), [0xA5, 0x3B, 0x02, 0x01, 0x1D]),
            (Command::new(0x3B, 0x0103, 0x03), [0xA5, 0x3B, 0x03, 0x01, 0x1C]),
            (Command::new(0x3E, 0x000E, 0x3E), [0xA5, 0x3E, 0x0E, 0x00, 0x0F]),
            (Command::new(0x3F, 0x0001, 0x3F), [0xA5, 0x3F, 0x01, 0x00, 0x1B]),
        ];
        for (cmd, expected) in cases {
            assert_eq!(cmd.encode(), expected, "frame mismatch for {cmd:?}");
        }
    }

    #[test]
    fn status_words_match() {
        assert_eq!(
            Status::from_bits(0x0000),
            Status {
                running: true,
                faults: None,
            }
        );
        assert_eq!(
            Status::from_bits(0xFF00),
            Status {
                running: false,
                faults: None,
            }
        );

        let running_hot = Status::from_bits(0x0020);
        assert!(running_hot.running);
        assert!(running_hot.faults.unwrap().over_temperature);

        let running_out_of_range = Status::from_bits(0x0080);
        assert!(running_out_of_range.running);
        assert!(running_out_of_range.faults.unwrap().speed_out_of_range);

        let stopped: [FaultCase; 6] = [
            (0xFF01, |f| f.start_fail),
            (0xFF04, |f| f.under_speed),
            (0xFF08, |f| f.wrong_rotor_position),
            (0xFF10, |f| f.motor_cable_fail),
            (0xFF20, |f| f.over_temperature),
            (0xFF40, |f| f.serial_fail),
        ];
        for (raw, is_set) in stopped {
            let s = Status::from_bits(raw);
            assert!(!s.running, "{raw:#06X} should read as stopped");
            assert!(is_set(&s.faults.unwrap()), "wrong fault for {raw:#06X}");
        }

        // 0xFF02 is documented as "not used".
        assert_eq!(Status::from_bits(0xFF02).faults, Some(Faults::default()));
    }
}
