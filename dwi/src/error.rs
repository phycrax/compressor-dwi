/// Error type for DWI operations
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Read error from the underlying serial interface.
    Read,
    /// Write error from the underlying serial interface.
    Write,
    /// No response within the configured timeout.
    Timeout,
    /// Bad response from the device.
    Response(ResponseError),
}

/// Error type for responses from the inverter
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResponseError {
    /// Bad checksum in a received frame.
    Integrity,
    /// Bad Rx identification in a received frame.
    RxId,
    /// Bad command code in a received frame.
    InvalidResponse,
    /// The inverter rejected the request with a communication-error.
    Communication(CommunicationError),
}

/// Communication-error reported by the inverter
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommunicationError {
    /// `0xF0` error in byte 4 (data high; valid for commands 2 and 3).
    DataHigh,
    /// `0xF2` checksum error (byte 5).
    Checksum,
    /// `0xF4` bad command code (byte 2).
    Command,
    /// `0xF8` error in byte 3 (data low; valid for commands 2 and 3).
    DataLow,
}

impl CommunicationError {
    pub(crate) fn from_code(code: u8) -> Option<Self> {
        match code {
            0xF0 => Some(Self::DataHigh),
            0xF2 => Some(Self::Checksum),
            0xF4 => Some(Self::Command),
            0xF8 => Some(Self::DataLow),
            _ => None,
        }
    }
}
