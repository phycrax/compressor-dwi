//! DWI 5-byte frame codec and the verified command table.
//!
//! Frame layout:
//! `[0] identification  [1] command  [2] data low  [3] data high  [4] checksum`

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
    /// Command byte expected in the response.
    pub expect: u8,
}

impl Command {
    /// Encode a request frame with a computed checksum.
    fn encode(&self) -> [u8; 5] {
        let bytes = &[REQ_ID, self.cmd, self.lo, self.hi];
        let sum: u32 = bytes.iter().map(|&b| u32::from(b)).sum();
        let ck = (sum as u8).wrapping_neg();

        [REQ_ID, self.cmd, self.lo, self.hi, ck]
    }

    /// Command 1 — Set Speed.
    pub const fn set_speed(speed: u16) -> Command {
        Command {
            cmd: 0xC3,
            lo: (speed & 0xFF) as u8,
            hi: (speed >> 8) as u8,
            expect: 0x83,
        }
    }

    /// Command 2 — Read Set Speed (request checksum 0x66).
    pub const fn read_set_speed() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x80,
            hi: 0x39,
            expect: 0x80,
        }
    }

    /// Command 3 — Read Operation Status (request checksum 0x63).
    pub const fn read_operation_status() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x83,
            hi: 0x39,
            expect: 0x83,
        }
    }

    /// Command 4 — Read Actual Speed (request checksum 0x60).
    pub const fn read_actual_speed() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x86,
            hi: 0x39,
            expect: 0x86,
        }
    }

    /// Command 5 — Read Power (request checksum 0x64).
    pub const fn read_power() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x82,
            hi: 0x39,
            expect: 0x82,
        }
    }

    /// Command 6 — Read Temperature (request checksum 0x5E).
    pub const fn read_temperature() -> Command {
        Command {
            cmd: 0x3C,
            lo: 0x88,
            hi: 0x39,
            expect: 0x88,
        }
    }

    /// Command 7 — Read Config Data (request checksum 0x1F; fixed ack
    /// `5A 00 00 00 A6`).
    pub const fn read_config_data() -> Command {
        Command {
            cmd: 0x3B,
            lo: 0x00,
            hi: 0x01,
            expect: 0x00,
        }
    }

    /// Command 8 — Read Input Voltage (request checksum 0x1E).
    pub const fn read_input_voltage() -> Command {
        Command {
            cmd: 0x3B,
            lo: 0x01,
            hi: 0x01,
            expect: 0x01,
        }
    }

    /// Command 9 — Read Temperature, alternate (request checksum 0x1D).
    pub const fn read_temperature_alt() -> Command {
        Command {
            cmd: 0x3B,
            lo: 0x02,
            hi: 0x01,
            expect: 0x02,
        }
    }

    /// Command 10 — Read Power, alternate (request checksum 0x1C).
    pub const fn read_power_alt() -> Command {
        Command {
            cmd: 0x3B,
            lo: 0x03,
            hi: 0x01,
            expect: 0x03,
        }
    }

    /// Command 11 — Read Status I (request checksum 0x0F; fixed ack
    /// `5A 3E 00 00 68`).
    pub const fn read_status_i() -> Command {
        Command {
            cmd: 0x3E,
            lo: 0x0E,
            hi: 0x00,
            expect: 0x3E,
        }
    }

    /// Command 12 — Read Status II (request checksum 0x1B; fixed ack
    /// `5A 3F 00 00 67`).
    pub const fn read_status_ii() -> Command {
        Command {
            cmd: 0x3F,
            lo: 0x01,
            hi: 0x00,
            expect: 0x3F,
        }
    }
}

/// Validate a received frame: the sum of all five bytes must be 0 modulo 256.
pub fn validate(frame: &[u8; 5]) -> bool {
    let sum: u32 = frame.iter().map(|&b| u32::from(b)).sum();
    (sum & 0xFF) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_checksums() {
        let cases: [(&Command, u8); _] = [
            (&Command::set_speed(3000), 0xD5),
            (&Command::read_set_speed(), 0x66),
            (&Command::read_operation_status(), 0x63),
            (&Command::read_actual_speed(), 0x60),
            (&Command::read_power(), 0x64),
            (&Command::read_temperature(), 0x5E),
            (&Command::read_config_data(), 0x1F),
            (&Command::read_input_voltage(), 0x1E),
            (&Command::read_temperature_alt(), 0x1D),
            (&Command::read_power_alt(), 0x1C),
            (&Command::read_status_i(), 0x0F),
            (&Command::read_status_ii(), 0x1B),
        ];
        for (c, expected_ck) in cases {
            let frame = c.encode();
            assert_eq!(
                frame[4], expected_ck,
                "checksum mismatch for cmd {:#04X} {:#04X} {:#04X}",
                c.cmd, c.lo, c.hi
            );
            assert!(validate(&frame));
        }
    }

    /// A single corrupted bit fails validation.
    #[test]
    fn validate_rejects_corruption() {
        let mut frame = Command::read_set_speed().encode();
        assert!(validate(&frame));
        frame[2] ^= 0x01;
        assert!(!validate(&frame));
    }
}
