//! Error types

use embedded_io_async::ReadExactError;

/// Error type for dual-wire operations.
///
/// `TX` and `RX` are the error types of the serial halves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<TX, RX> {
    /// Read error from the underlying serial interface.
    Read(RX),
    /// Write error from the underlying serial interface.
    Write(TX),
    /// The serial interface hit end-of-file before a full response arrived.
    UnexpectedEof,
    /// Bad response from the device.
    Response(ResponseError),
}

impl<TX, RX> From<ResponseError> for Error<TX, RX> {
    fn from(e: ResponseError) -> Self {
        Self::Response(e)
    }
}

impl<TX, RX> From<ReadExactError<RX>> for Error<TX, RX> {
    fn from(e: ReadExactError<RX>) -> Self {
        match e {
            ReadExactError::UnexpectedEof => Self::UnexpectedEof,
            ReadExactError::Other(e) => Self::Read(e),
        }
    }
}

/// Error type for responses from the compressor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResponseError {
    /// Bad checksum in a received frame.
    Integrity,
    /// Bad Rx identification in a received frame.
    RxId,
    /// Bad command code in a received frame.
    InvalidResponse,
    /// The compressor rejected the request with a communication-error.
    Communication(CommunicationError),
}

/// Communication-error reported by the compressor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommunicationError {
    /// `0xF0` error in byte 4 (data high).
    DataHigh,
    /// `0xF2` checksum error (byte 5).
    Checksum,
    /// `0xF4` bad command code (byte 2).
    Command,
    /// `0xF8` error in byte 3 (data low).
    DataLow,
}

impl CommunicationError {
    /// Map an error-frame code byte, or `None` if it is not a known code.
    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        match code {
            0xF0 => Some(Self::DataHigh),
            0xF2 => Some(Self::Checksum),
            0xF4 => Some(Self::Command),
            0xF8 => Some(Self::DataLow),
            _ => None,
        }
    }
}

impl<TX, RX> core::fmt::Display for Error<TX, RX> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Read(_) => "serial read failed",
            Self::Write(_) => "serial write failed",
            Self::UnexpectedEof => "serial input ended before a full response",
            Self::Response(_) => "bad response from the compressor",
        })
    }
}

impl<TX, RX> core::error::Error for Error<TX, RX>
where
    TX: core::error::Error + 'static,
    RX: core::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Read(e) => Some(e),
            Self::Write(e) => Some(e),
            Self::UnexpectedEof => None,
            Self::Response(e) => Some(e),
        }
    }
}

impl core::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Integrity => "checksum mismatch",
            Self::RxId => "frame does not start with 0x5A",
            Self::InvalidResponse => "unexpected command byte",
            Self::Communication(_) => "compressor rejected the request",
        })
    }
}

impl core::error::Error for ResponseError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Communication(e) => Some(e),
            _ => None,
        }
    }
}

impl core::fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::DataHigh => "error in data high byte",
            Self::Checksum => "checksum error",
            Self::Command => "unknown command code",
            Self::DataLow => "error in data low byte",
        })
    }
}

impl core::error::Error for CommunicationError {}
