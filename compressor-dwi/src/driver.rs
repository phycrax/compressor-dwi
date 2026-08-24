//! Frame exchange over the two signal lines.

use embassy_futures::join::join;
use embassy_futures::select::{Either, select};
use embedded_hal_async::delay::DelayNs;
use embedded_io_async::{Read, Write};

use crate::{Command, Error};

/// Configuration for the driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Timeout for a complete transaction in milliseconds.
    ///
    /// At 600 baud a 5-byte frame takes ~83 ms each way. Default 500 ms.
    pub response_timeout_ms: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            response_timeout_ms: 500,
        }
    }
}

/// Serial driver.
pub struct Driver<TX, RX, D> {
    tx: TX,
    rx: RX,
    delay: D,
    cfg: Config,
}

impl<TX: Write, RX: Read, D: DelayNs> Driver<TX, RX, D> {
    /// Create a driver from separate halves of the serial interface.
    pub const fn new(tx: TX, rx: RX, delay: D, cfg: Config) -> Self {
        Self { tx, rx, delay, cfg }
    }

    /// Give back the serial halves and the delay provider.
    pub fn release(self) -> (TX, RX, D) {
        (self.tx, self.rx, self.delay)
    }

    /// Send a request, await its reply, and return the response.
    pub async fn transact(&mut self, cmd: &Command) -> Result<u16, Error> {
        let req = cmd.encode();
        let mut resp = [0u8; 5];

        let exchange = join(self.rx.read_exact(&mut resp), self.tx.write_all(&req));

        match select(exchange, self.delay.delay_ms(u32::from(self.cfg.response_timeout_ms))).await {
            Either::First((rx, tx)) => {
                rx.map_err(|_| Error::Read)?;
                tx.map_err(|_| Error::Write)?;
            }
            Either::Second(()) => return Err(Error::Timeout),
        }

        cmd.validate(&resp)?;

        Ok(u16::from_le_bytes([resp[2], resp[3]]))
    }
}
