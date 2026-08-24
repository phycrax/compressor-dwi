//! Dual Wire Interface (DWI) driver for Embraco CF10B inverters.
#![no_std]
#![warn(missing_docs)]

use compressor_dwi::{Command, Driver};
pub use compressor_dwi::{Config, Error};
use embedded_hal_async::delay::DelayNs;
use embedded_io_async::{Read, Write};

/// Operation status of the inverter.
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

/// Fault flags reported by the inverter.
///
/// Whether a fault stopped the compressor is carried by [`Status::running`],
/// not by these flags: `overload` while running is the protection engaging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Faults {
    /// `0x01` start fail.
    pub start_fail: bool,
    /// `0x02` overload, or overload protection when still running.
    pub overload: bool,
    /// `0x04` under speed (1550 RPM or lower).
    pub under_speed: bool,
    /// `0x08` wrong rotor position.
    pub wrong_rotor_position: bool,
    /// `0x10` overvoltage.
    pub overvoltage: bool,
    /// `0x20` over-temperature (above 105 °C).
    pub over_temperature: bool,
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
            overload: bits & 0x02 != 0,
            under_speed: bits & 0x04 != 0,
            wrong_rotor_position: bits & 0x08 != 0,
            overvoltage: bits & 0x10 != 0,
            over_temperature: bits & 0x20 != 0,
            speed_out_of_range: bits & 0x80 != 0,
        })
    }
}

/// Embraco CF10B inverter driver.
pub struct Inverter<TX, RX, D> {
    inner: Driver<TX, RX, D>,
}

impl<TX: Write, RX: Read, D: DelayNs> Inverter<TX, RX, D> {
    /// Create a driver.
    pub const fn new(tx: TX, rx: RX, delay: D, cfg: Config) -> Self {
        Self {
            inner: Driver::new(tx, rx, delay, cfg),
        }
    }

    /// Release the serial halves and the delay provider.
    pub fn release(self) -> (TX, RX, D) {
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

    /// Read the power consumption (W).
    pub async fn read_power(&mut self) -> Result<u16, Error> {
        self.inner.transact(&Command::new(0x3C, 0x3982, 0x82)).await
    }

    /// Read the inverter temperature in 0.1 °C.
    pub async fn read_temperature(&mut self) -> Result<i16, Error> {
        Ok(self.inner.transact(&Command::new(0x3C, 0x3988, 0x88)).await? as i16)
    }

    /// Read the number of starting trials.
    pub async fn read_starting_trials(&mut self) -> Result<u16, Error> {
        self.inner.transact(&Command::new(0x3C, 0x3981, 0x81)).await
    }

    /// Read the DC bus voltage (V).
    pub async fn read_bus_voltage(&mut self) -> Result<u16, Error> {
        self.inner.transact(&Command::new(0x3C, 0x3984, 0x84)).await
    }

    /// Read the active power limitation (W).
    pub async fn read_power_limitation(&mut self) -> Result<u16, Error> {
        self.inner.transact(&Command::new(0x3C, 0x398A, 0x8A)).await
    }

    /// Choose whether a serial set speed overrides the thermostat set point.
    ///
    /// Returns the state the inverter reports, which need not match what was requested.
    /// Reverts to `false` after 4 hours without serial communication.
    pub async fn set_speed_overwrite(&mut self, enable: bool) -> Result<bool, Error> {
        let data = 0x9300 | enable as u16;
        Ok(self.inner.transact(&Command::new(0x69, data, 0xC3)).await? & 0x00FF != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type FaultCase = (u16, fn(&Faults) -> bool);

    #[test]
    fn request_frames_match() {
        let cases: [(Command, [u8; 5]); 10] = [
            (Command::new(0xC3, 2000, 0x83), [0xA5, 0xC3, 0xD0, 0x07, 0xC1]),
            (Command::new(0x3C, 0x3980, 0x80), [0xA5, 0x3C, 0x80, 0x39, 0x66]),
            (Command::new(0x3C, 0x3981, 0x81), [0xA5, 0x3C, 0x81, 0x39, 0x65]),
            (Command::new(0x3C, 0x3982, 0x82), [0xA5, 0x3C, 0x82, 0x39, 0x64]),
            (Command::new(0x3C, 0x3983, 0x83), [0xA5, 0x3C, 0x83, 0x39, 0x63]),
            (Command::new(0x3C, 0x3984, 0x84), [0xA5, 0x3C, 0x84, 0x39, 0x62]),
            (Command::new(0x3C, 0x3988, 0x88), [0xA5, 0x3C, 0x88, 0x39, 0x5E]),
            (Command::new(0x3C, 0x398A, 0x8A), [0xA5, 0x3C, 0x8A, 0x39, 0x5C]),
            (Command::new(0x69, 0x9300, 0xC3), [0xA5, 0x69, 0x00, 0x93, 0x5F]),
            (Command::new(0x69, 0x9301, 0xC3), [0xA5, 0x69, 0x01, 0x93, 0x5E]),
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

        // Notes 1 and 2: the same bits appear with the compressor running.
        let overload_protection = Status::from_bits(0x0002);
        assert!(overload_protection.running);
        assert!(overload_protection.faults.unwrap().overload);

        let running_out_of_range = Status::from_bits(0x0080);
        assert!(running_out_of_range.running);
        assert!(running_out_of_range.faults.unwrap().speed_out_of_range);

        let stopped: [FaultCase; 7] = [
            (0xFF01, |f| f.start_fail),
            (0xFF02, |f| f.overload),
            (0xFF04, |f| f.under_speed),
            (0xFF08, |f| f.wrong_rotor_position),
            (0xFF10, |f| f.overvoltage),
            (0xFF20, |f| f.over_temperature),
            (0xFF80, |f| f.speed_out_of_range),
        ];
        for (raw, is_set) in stopped {
            let s = Status::from_bits(raw);
            assert!(!s.running, "{raw:#06X} should read as stopped");
            assert!(is_set(&s.faults.unwrap()), "wrong fault for {raw:#06X}");
        }
    }
}
