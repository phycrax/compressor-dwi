//! Dual Wire Interface (DWI) driver for Embraco CF10B compressors.
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
    /// [`Request::SetSpeed`] and [`Request::ReadStatus`] answers with this.
    pub const fn from_bits(raw: u16) -> Self {
        Self {
            running: raw & 0xFF00 == 0,
            faults: Faults::from_bits((raw & 0x00FF) as u8),
        }
    }
}

/// Fault flags reported by the compressor.
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

/// Request to the compressor with an u16 response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Request {
    /// Set the speed (RPM), answers with the status, see [`Status::from_bits`].
    SetSpeed(u16),
    /// Read the currently set speed (RPM).
    ReadSetSpeed,
    /// Read the operation status, see [`Status::from_bits`].
    ReadStatus,
    /// Read the power consumption (W).
    ReadPower,
    /// Read the compressor temperature in 0.1 °C, as an `i16`.
    ReadTemperature,
    /// Read the number of starting trials.
    ReadStartingTrials,
    /// Read the DC bus voltage (V).
    ReadBusVoltage,
    /// Read the active power limitation (W).
    ReadPowerLimitation,
    /// Choose whether a serial set speed overrides the thermostat set point.
    SetSpeedOverwrite(bool),
}

impl Request {
    const fn command(self) -> Command {
        match self {
            Self::SetSpeed(rpm) => Command::new(0xC3, rpm, 0x83),
            Self::ReadSetSpeed => Command::new(0x3C, 0x3980, 0x80),
            Self::ReadStatus => Command::new(0x3C, 0x3983, 0x83),
            Self::ReadPower => Command::new(0x3C, 0x3982, 0x82),
            Self::ReadTemperature => Command::new(0x3C, 0x3988, 0x88),
            Self::ReadStartingTrials => Command::new(0x3C, 0x3981, 0x81),
            Self::ReadBusVoltage => Command::new(0x3C, 0x3984, 0x84),
            Self::ReadPowerLimitation => Command::new(0x3C, 0x398A, 0x8A),
            Self::SetSpeedOverwrite(enable) => Command::new(0x69, 0x9300 | enable as u16, 0xC3),
        }
    }
}

/// Embraco CF10B compressor driver.
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

    /// Sends `request` and returns the u16 it answers with.
    pub async fn request(&mut self, request: Request) -> Result<u16, Error<TX::Error, RX::Error>> {
        self.inner.transact(&request.command()).await
    }

    /// Set the speed (RPM), and read back the resulting status.
    pub async fn set_speed(&mut self, rpm: u16) -> Result<Status, Error<TX::Error, RX::Error>> {
        Ok(Status::from_bits(self.request(Request::SetSpeed(rpm)).await?))
    }

    /// Read the currently set speed (RPM).
    pub async fn read_set_speed(&mut self) -> Result<u16, Error<TX::Error, RX::Error>> {
        self.request(Request::ReadSetSpeed).await
    }

    /// Read the operation-status word.
    pub async fn read_status(&mut self) -> Result<Status, Error<TX::Error, RX::Error>> {
        Ok(Status::from_bits(self.request(Request::ReadStatus).await?))
    }

    /// Read the power consumption (W).
    pub async fn read_power(&mut self) -> Result<u16, Error<TX::Error, RX::Error>> {
        self.request(Request::ReadPower).await
    }

    /// Read the compressor temperature in 0.1 °C.
    pub async fn read_temperature(&mut self) -> Result<i16, Error<TX::Error, RX::Error>> {
        Ok(self.request(Request::ReadTemperature).await? as i16)
    }

    /// Read the number of starting trials.
    pub async fn read_starting_trials(&mut self) -> Result<u16, Error<TX::Error, RX::Error>> {
        self.request(Request::ReadStartingTrials).await
    }

    /// Read the DC bus voltage (V).
    pub async fn read_bus_voltage(&mut self) -> Result<u16, Error<TX::Error, RX::Error>> {
        self.request(Request::ReadBusVoltage).await
    }

    /// Read the active power limitation (W).
    pub async fn read_power_limitation(&mut self) -> Result<u16, Error<TX::Error, RX::Error>> {
        self.request(Request::ReadPowerLimitation).await
    }

    /// Choose whether a serial set speed overrides the thermostat set point.
    ///
    /// Returns the state the compressor reports, which need not match what was requested.
    /// Reverts to `false` after 4 hours without serial communication.
    pub async fn set_speed_overwrite(&mut self, enable: bool) -> Result<bool, Error<TX::Error, RX::Error>> {
        Ok(self.request(Request::SetSpeedOverwrite(enable)).await? & 0x00FF != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type FaultCase = (u16, fn(&Faults) -> bool);

    #[test]
    fn requests_send_their_frames() {
        let cases: [(Request, [u8; 5]); 10] = [
            (Request::SetSpeed(2000), [0xA5, 0xC3, 0xD0, 0x07, 0xC1]),
            (Request::ReadSetSpeed, [0xA5, 0x3C, 0x80, 0x39, 0x66]),
            (Request::ReadStartingTrials, [0xA5, 0x3C, 0x81, 0x39, 0x65]),
            (Request::ReadPower, [0xA5, 0x3C, 0x82, 0x39, 0x64]),
            (Request::ReadStatus, [0xA5, 0x3C, 0x83, 0x39, 0x63]),
            (Request::ReadBusVoltage, [0xA5, 0x3C, 0x84, 0x39, 0x62]),
            (Request::ReadTemperature, [0xA5, 0x3C, 0x88, 0x39, 0x5E]),
            (Request::ReadPowerLimitation, [0xA5, 0x3C, 0x8A, 0x39, 0x5C]),
            (Request::SetSpeedOverwrite(false), [0xA5, 0x69, 0x00, 0x93, 0x5F]),
            (Request::SetSpeedOverwrite(true), [0xA5, 0x69, 0x01, 0x93, 0x5E]),
        ];
        for (request, expected) in cases {
            assert_eq!(request.command().encode(), expected, "frame mismatch for {request:?}");
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
