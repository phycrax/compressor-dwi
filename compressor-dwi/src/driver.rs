//! Frame exchange over the two signal lines.

use embedded_io_async::{Read, ReadReady, Write};

use crate::{Command, Error};

/// Serial driver.
#[derive(Debug)]
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
    ///
    /// Stale bytes waiting in `rx` are dropped before the request is sent.
    /// There is no timeout: wrap the call in one if the compressor may not answer.
    pub async fn transact(&mut self, cmd: &Command) -> Result<u16, Error<TX::Error, RX::Error>> {
        let mut sink = [0u8; 16];
        while self.rx.read_ready().map_err(Error::Read)? {
            // A reader at end-of-file stays "ready" but yields nothing.
            if self.rx.read(&mut sink).await.map_err(Error::Read)? == 0 {
                break;
            }
        }

        self.tx.write_all(&cmd.encode()).await.map_err(Error::Write)?;

        let mut resp = [0u8; 5];
        self.rx.read_exact(&mut resp).await?;

        cmd.validate(&resp)?;

        Ok(u16::from_le_bytes([resp[2], resp[3]]))
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};

    use embedded_io_async::ErrorType;

    use super::*;

    fn block_on<F: Future>(f: F) -> F::Output {
        let mut f = pin!(f);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(out) = f.as_mut().poll(&mut cx) {
                return out;
            }
        }
    }

    /// Holds `stale` bytes that are ready before the request, then `reply`.
    struct MockRx<'a> {
        stale: &'a [u8],
        reply: &'a [u8],
    }

    impl ErrorType for MockRx<'_> {
        type Error = Infallible;
    }

    impl ReadReady for MockRx<'_> {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            Ok(!self.stale.is_empty())
        }
    }

    impl Read for MockRx<'_> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            let src = if self.stale.is_empty() {
                &mut self.reply
            } else {
                &mut self.stale
            };
            let bytes = core::mem::take(src);
            let (head, tail) = bytes.split_at(buf.len().min(bytes.len()));
            buf[..head.len()].copy_from_slice(head);
            *src = tail;
            Ok(head.len())
        }
    }

    const READ_SET_SPEED: Command = Command::new(0x3C, 0x3980, 0x80);

    #[test]
    fn transact_drops_stale_bytes_and_decodes_reply() {
        let mut sent = [0u8; 5];
        let rx = MockRx {
            stale: &[0x00, 0x13, 0x37],
            reply: &[0x5A, 0x80, 0xB8, 0x0B, 0x63],
        };
        let mut driver = Driver::new(&mut sent[..], rx);

        assert_eq!(block_on(driver.transact(&READ_SET_SPEED)), Ok(3000));
        driver.release();
        assert_eq!(sent, READ_SET_SPEED.encode());
    }

    #[test]
    fn transact_reports_eof_instead_of_spinning() {
        let mut sent = [0u8; 5];
        let mut driver = Driver::new(&mut sent[..], &[] as &[u8]);

        assert_eq!(block_on(driver.transact(&READ_SET_SPEED)), Err(Error::UnexpectedEof));
    }
}
