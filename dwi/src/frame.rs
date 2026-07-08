//! DWI 5-byte frame
//!
//! Frame layout
//! `[0] identification  [1] command  [2] data low  [3] data high  [4] checksum`

use crate::error::{CommunicationError, ResponseError};

/// Identification bytes
const REQ_ID: u8 = 0xA5;
const RESP_ID: u8 = 0x5A;

pub struct Command {
    /// Request command byte
    pub cmd: u8,
    /// Request data-low byte
    pub lo: u8,
    /// Request data-high byte
    pub hi: u8,
    /// Command byte expected in the response
    pub expected_cmd: u8,
}

impl Command {
    /// Encode a request frame with a computed checksum.
    pub fn encode(&self) -> [u8; 5] {
        let sum: u32 = [REQ_ID, self.cmd, self.lo, self.hi]
            .iter()
            .map(|&b| u32::from(b))
            .sum();
        let ck = (sum as u8).wrapping_neg();

        [REQ_ID, self.cmd, self.lo, self.hi, ck]
    }

    /// Command 1 - Set Speed.
    pub const fn set_speed(speed: u16) -> Command {
        Command {
            cmd: 0xC3,
            lo: (speed & 0xFF) as u8,
            hi: (speed >> 8) as u8,
            expected_cmd: 0x83,
        }
    }

    pub const fn read_set_speed() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x80,
            hi: 0x39,
            expected_cmd: 0x80,
        }
    }

    pub const fn read_operation_status() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x83,
            hi: 0x39,
            expected_cmd: 0x83,
        }
    }

    pub const fn read_actual_speed() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x86,
            hi: 0x39,
            expected_cmd: 0x86,
        }
    }

    pub const fn read_power() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x82,
            hi: 0x39,
            expected_cmd: 0x82,
        }
    }

    pub const fn read_temperature() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x88,
            hi: 0x39,
            expected_cmd: 0x88,
        }
    }

    pub const fn _read_config_data() -> Command {
        Command {
            cmd: 0x3B,
            lo: 0x00,
            hi: 0x01,
            expected_cmd: 0x00,
        }
    }

    pub const fn read_input_voltage() -> Command {
        Command {
            cmd: 0x3B,
            lo: 0x01,
            hi: 0x01,
            expected_cmd: 0x01,
        }
    }

    pub const fn _read_temperature_alt() -> Command {
        Command {
            cmd: 0x3B,
            lo: 0x02,
            hi: 0x01,
            expected_cmd: 0x02,
        }
    }

    pub const fn _read_power_alt() -> Command {
        Command {
            cmd: 0x3B,
            lo: 0x03,
            hi: 0x01,
            expected_cmd: 0x03,
        }
    }

    pub const fn _read_status_i() -> Command {
        Command {
            cmd: 0x3E,
            lo: 0x0E,
            hi: 0x00,
            expected_cmd: 0x3E,
        }
    }

    pub const fn _read_status_ii() -> Command {
        Command {
            cmd: 0x3F,
            lo: 0x01,
            hi: 0x00,
            expected_cmd: 0x3F,
        }
    }
}

/// Validate a received frame: the sum of all five bytes must be 0 modulo 256.
pub fn validate(cmd: &Command, resp: &[u8; 5]) -> Result<(), ResponseError> {
    if resp[0] != RESP_ID {
        return Err(ResponseError::RxId);
    }

    // Communication-error frame: `5A <code> FF FF <ck>`.
    if resp[2] == 0xFF && resp[3] == 0xFF {
        if let Some(code) = CommunicationError::from_code(resp[1]) {
            return Err(ResponseError::Communication(code));
        }
    }

    if resp[1] != cmd.expected_cmd {
        return Err(ResponseError::InvalidResponse);
    }

    let sum: u32 = resp.iter().map(|&b| u32::from(b)).sum();
    if (sum & 0xFF) != 0 {
        return Err(ResponseError::Integrity);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_valid_response_frames() {
        let cases: [(Command, [u8; 5]); 3] = [
            (Command::_read_config_data(), [0x5A, 0x00, 0x00, 0x00, 0xA6]),
            (Command::_read_status_i(), [0x5A, 0x3E, 0x00, 0x00, 0x68]),
            (Command::_read_status_ii(), [0x5A, 0x3F, 0x00, 0x00, 0x67]),
        ];
        for (cmd, resp) in cases {
            validate(&cmd, &resp).unwrap();
        }
    }

    #[test]
    fn validate_rejects_bad_rx_id() {
        let cmd = Command::_read_config_data();
        let resp = [0xA5, 0x00, 0x00, 0x00, 0x5B];
        assert!(matches!(validate(&cmd, &resp), Err(ResponseError::RxId)));
    }

    #[test]
    fn validate_rejects_unexpected_command_code() {
        let cmd = Command::_read_config_data();
        // resp[1] doesn't match cmd.expect (0x00), but checksum is valid.
        let resp = [0x5A, 0x01, 0x00, 0x00, 0xA5];
        assert!(matches!(
            validate(&cmd, &resp),
            Err(ResponseError::InvalidResponse)
        ));
    }

    #[test]
    fn validate_rejects_bad_checksum() {
        let cmd = Command::_read_config_data();
        // Same as the valid 0x00 frame but with a corrupted checksum byte.
        let resp = [0x5A, 0x00, 0x00, 0x00, 0x00];
        assert!(matches!(
            validate(&cmd, &resp),
            Err(ResponseError::Integrity)
        ));
    }

    #[test]
    fn validate_reports_communication_errors() {
        let cmd = Command::_read_config_data();
        let cases: [([u8; 5], fn(&CommunicationError) -> bool); 4] = [
            ([0x5A, 0xF0, 0xFF, 0xFF, 0xB8], |e| {
                matches!(e, CommunicationError::DataHigh)
            }),
            ([0x5A, 0xF2, 0xFF, 0xFF, 0xB6], |e| {
                matches!(e, CommunicationError::Checksum)
            }),
            ([0x5A, 0xF4, 0xFF, 0xFF, 0xB4], |e| {
                matches!(e, CommunicationError::Command)
            }),
            ([0x5A, 0xF8, 0xFF, 0xFF, 0xB0], |e| {
                matches!(e, CommunicationError::DataLow)
            }),
        ];
        for (resp, is_expected) in cases {
            match validate(&cmd, &resp) {
                Err(ResponseError::Communication(code)) => {
                    assert!(is_expected(&code), "unexpected communication error code")
                }
                other => panic!("expected communication error, got {other:?}"),
            }
        }
    }

    #[test]
    fn request_checksums() {
        let cases: [(&Command, u8); _] = [
            (&Command::set_speed(3000), 0xD5),
            (&Command::read_set_speed(), 0x66),
            (&Command::read_operation_status(), 0x63),
            (&Command::read_actual_speed(), 0x60),
            (&Command::read_power(), 0x64),
            (&Command::read_temperature(), 0x5E),
            (&Command::_read_config_data(), 0x1F),
            (&Command::read_input_voltage(), 0x1E),
            (&Command::_read_temperature_alt(), 0x1D),
            (&Command::_read_power_alt(), 0x1C),
            (&Command::_read_status_i(), 0x0F),
            (&Command::_read_status_ii(), 0x1B),
        ];
        for (c, expected_ck) in cases {
            let frame = c.encode();
            assert_eq!(
                frame[4], expected_ck,
                "checksum mismatch for cmd {:#04X} {:#04X} {:#04X}",
                c.cmd, c.lo, c.hi
            );
        }
    }
}
