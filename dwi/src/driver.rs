use embassy_futures::{
    join::join,
    select::{Either, select},
};
use embedded_hal_async::delay::DelayNs;
use embedded_io_async::{Read, Write};

use crate::{
    error::Error,
    frame::{Command, validate},
    status::OperationStatus,
};

/// Configuration for the driver
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Timeout for a complete transaction in milliseconds.
    ///
    /// At 600 baud a frame takes ~83 ms each way, the default is **500 ms**.
    pub response_timeout_ms: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            response_timeout_ms: 500,
        }
    }
}

/// Dual Wire Interface (DWI) driver
pub struct Driver<TX, RX, D> {
    tx: TX,
    rx: RX,
    delay: D,
    cfg: Config,
}

impl<TX: Write, RX: Read, D: DelayNs> Driver<TX, RX, D> {
    /// Create a driver from separate transmit and receive halves of the serial interface.
    pub fn new(tx: TX, rx: RX, delay: D, cfg: Config) -> Self {
        Self { tx, rx, delay, cfg }
    }

    /// Set the speed (RPM) of the inverter.
    ///
    /// | Range | Impact |
    /// | --- | --- |
    /// | 0 – 1499 | stop |
    /// | 1500 – 1999 | out of range |
    /// | 2000 (NLVE) or 2200 (SLVE) – 4500 | speed set |
    /// | 4501 – 6000 | out of range |
    /// | Above 6000 | stop |
    pub async fn set_speed(&mut self, val: u16) -> Result<OperationStatus, Error> {
        let bytes = self.req_resp(&Command::set_speed(val)).await?;
        Ok(OperationStatus::from_bits(u16::from_le_bytes(bytes)))
    }

    /// Read the currently set speed (RPM).
    pub async fn read_set_speed(&mut self) -> Result<u16, Error> {
        let bytes = self.req_resp(&Command::read_set_speed()).await?;
        Ok(u16::from_le_bytes(bytes))
    }

    /// Read the operation-status word.
    pub async fn read_operation_status(&mut self) -> Result<OperationStatus, Error> {
        let bytes = self.req_resp(&Command::read_operation_status()).await?;
        Ok(OperationStatus::from_bits(u16::from_le_bytes(bytes)))
    }

    /// Read the measured compressor speed (RPM).
    pub async fn read_actual_speed(&mut self) -> Result<u16, Error> {
        let bytes = self.req_resp(&Command::read_actual_speed()).await?;
        Ok(u16::from_le_bytes(bytes))
    }

    /// Read the power consumption.
    pub async fn read_power(&mut self) -> Result<u16, Error> {
        let bytes = self.req_resp(&Command::read_power()).await?;
        Ok(u16::from_le_bytes(bytes))
    }

    /// Read the inverter temperature in 0.1 °C
    pub async fn read_temperature(&mut self) -> Result<i16, Error> {
        let bytes = self.req_resp(&Command::read_temperature()).await?;
        Ok(i16::from_le_bytes(bytes))
    }

    /// Read the input voltage.
    pub async fn read_input_voltage(&mut self) -> Result<u16, Error> {
        let bytes = self.req_resp(&Command::read_input_voltage()).await?;
        Ok(u16::from_le_bytes(bytes))
    }

    async fn req_resp(&mut self, cmd: &Command) -> Result<[u8; 2], Error> {
        let mut resp = [0u8; 5];
        let req = cmd.encode();

        let fut = join(self.rx.read_exact(&mut resp), self.tx.write_all(&req));

        match select(
            fut,
            self.delay.delay_ms(self.cfg.response_timeout_ms as u32),
        )
        .await
        {
            Either::First((rx, tx)) => {
                rx.map_err(|_| Error::Read)?;
                tx.map_err(|_| Error::Write)?;
            }
            Either::Second(()) => Err(Error::Timeout)?,
        };

        validate(cmd, &resp).map_err(Error::Response)?;

        Ok([resp[2], resp[3]])
    }
}
