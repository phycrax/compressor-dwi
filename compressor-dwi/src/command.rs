//! The 5-byte frame: `[id][cmd][lo][hi][ck]`, little-endian payload.

use crate::{CommunicationError, ResponseError};

const REQ_ID: u8 = 0xA5;
const RESP_ID: u8 = 0x5A;

/// A request and the response command byte it expects back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Command {
    /// Request command.
    cmd: u8,
    /// Request data.
    data: u16,
    /// Expected command byte in the response.
    expect: u8,
}

impl Command {
    /// Create a new command.
    pub const fn new(cmd: u8, data: u16, expect: u8) -> Self {
        Self { cmd, data, expect }
    }

    /// Encode a request frame with a computed checksum.
    pub const fn encode(&self) -> [u8; 5] {
        let cmd = self.cmd;
        let [lo, hi] = self.data.to_le_bytes();

        let checksum = REQ_ID
            .wrapping_add(cmd)
            .wrapping_add(lo)
            .wrapping_add(hi)
            .wrapping_neg();

        [REQ_ID, cmd, lo, hi, checksum]
    }

    /// Validate a received frame against the request that produced it.
    pub fn validate(&self, resp: &[u8; 5]) -> Result<(), ResponseError> {
        if resp[0] != RESP_ID {
            return Err(ResponseError::RxId);
        }

        let sum = resp.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

        if sum != 0 {
            return Err(ResponseError::Integrity);
        }

        if resp[2] == 0xFF
            && resp[3] == 0xFF
            && let Some(code) = CommunicationError::from_code(resp[1])
        {
            return Err(ResponseError::Communication(code));
        }

        if resp[1] != self.expect {
            return Err(ResponseError::InvalidResponse);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROBE: Command = Command::new(0x3B, 0x0100, 0x00);

    #[test]
    fn encoded_frames_sum_to_zero() {
        for cmd in 0u8..=255 {
            let data = u16::from_le_bytes([cmd ^ 0x5A, cmd.wrapping_mul(3)]);
            let c = Command::new(cmd, data, cmd);
            let sum = c.encode().iter().fold(0u8, |a, &b| a.wrapping_add(b));
            assert_eq!(sum, 0, "frame for cmd {cmd:#04X} does not sum to zero");
        }
    }

    #[test]
    fn validate_accepts_valid_response_frames() {
        let cases: [(Command, [u8; 5]); 3] = [
            (PROBE, [0x5A, 0x00, 0x00, 0x00, 0xA6]),
            (Command::new(0x3E, 0x000E, 0x3E), [0x5A, 0x3E, 0x00, 0x00, 0x68]),
            (Command::new(0x3F, 0x0001, 0x3F), [0x5A, 0x3F, 0x00, 0x00, 0x67]),
        ];
        for (cmd, resp) in cases {
            cmd.validate(&resp).unwrap();
        }
    }

    #[test]
    fn validate_rejects_bad_rx_id() {
        let resp = [0xA5, 0x00, 0x00, 0x00, 0x5B];
        assert!(matches!(PROBE.validate(&resp), Err(ResponseError::RxId)));
    }

    #[test]
    fn validate_rejects_unexpected_command_code() {
        let resp = [0x5A, 0x01, 0x00, 0x00, 0xA5];
        assert!(matches!(PROBE.validate(&resp), Err(ResponseError::InvalidResponse)));
    }

    #[test]
    fn validate_rejects_bad_checksum() {
        let resp = [0x5A, 0x00, 0x00, 0x00, 0x00];
        assert!(matches!(PROBE.validate(&resp), Err(ResponseError::Integrity)));
    }

    #[test]
    fn validate_reports_communication_errors() {
        let cases: [([u8; 5], CommunicationError); 4] = [
            ([0x5A, 0xF0, 0xFF, 0xFF, 0xB8], CommunicationError::DataHigh),
            ([0x5A, 0xF2, 0xFF, 0xFF, 0xB6], CommunicationError::Checksum),
            ([0x5A, 0xF4, 0xFF, 0xFF, 0xB4], CommunicationError::Command),
            ([0x5A, 0xF8, 0xFF, 0xFF, 0xB0], CommunicationError::DataLow),
        ];
        for (resp, expected) in cases {
            match PROBE.validate(&resp) {
                Err(ResponseError::Communication(code)) => assert_eq!(code, expected),
                other => panic!("expected communication error, got {other:?}"),
            }
        }
    }
}
