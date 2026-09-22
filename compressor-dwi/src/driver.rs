//! Frame exchange over the two signal lines.

use embedded_io_async::{Read, ReadReady, Write};

use crate::{Command, Error};

/// Serial driver.
pub struct Driver<TX, RX> {
    tx: TX,
    rx: RX,
}

impl<TX: Write, RX: Read + ReadReady> Driver<TX, RX> {
    /// Create a driver from separate halves of the serial interface.
    pub const fn new(tx: TX, rx: RX) -> Self {
        Self { tx, rx }
    }

    /// Give back the serial halves.
    pub fn release(self) -> (TX, RX) {
        (self.tx, self.rx)
    }

    /// Send a request, await its reply, and return the response.
    pub async fn transact(&mut self, cmd: &Command) -> Result<u16, Error> {
        let mut sink = [0u8; 16];
        while self.rx.read_ready().map_err(|_| Error::Read)? {
            self.rx.read(&mut sink).await.map_err(|_| Error::Read)?;
        }

        self.tx.write_all(&cmd.encode()).await.map_err(|_| Error::Write)?;

        let mut resp = [0u8; 5];
        self.rx.read_exact(&mut resp).await.map_err(|_| Error::Read)?;

        cmd.validate(&resp)?;

        Ok(u16::from_le_bytes([resp[2], resp[3]]))
    }
}
