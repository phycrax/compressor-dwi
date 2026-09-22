//! Error types

/// Error type for dual-wire operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Read error from the underlying serial interface.
    Read,
    /// Write error from the underlying serial interface.
    Write,
    /// Bad response from the device.
    Response(ResponseError),
}

impl From<ResponseError> for Error {
    fn from(e: ResponseError) -> Self {
        Self::Response(e)
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
