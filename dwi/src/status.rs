/// Inverter operation status and its fault flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OperationStatus {
    /// Whether the compressor is running
    pub running: bool,
    /// Faults, or `None` if no fault bit is set
    pub faults: Option<Faults>,
}

impl OperationStatus {
    pub(crate) const fn from_bits(raw: u16) -> Self {
        Self {
            running: raw & 0xFF00 == 0,
            faults: Faults::from_bits((raw & 0x00FF) as u8),
        }
    }
}

/// Fault flags derived from [`OperationStatus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Faults {
    /// `0x01`
    pub start_fail: bool,
    /// `0x04`
    pub under_speed: bool,
    /// `0x08`
    pub wrong_rotor_position: bool,
    /// `0x10`
    pub motor_cable_fail: bool,
    /// `0x20`
    pub over_temperature: bool,
    /// `0x40`
    pub serial_fail: bool,
    /// `0x80`
    pub speed_out_of_range: bool,
}

impl Faults {
    /// `None` when no fault bit is set.
    const fn from_bits(bits: u8) -> Option<Self> {
        if bits == 0 {
            return None;
        }

        Some(Self {
            start_fail: bits & 0x01 != 0,
            under_speed: bits & 0x04 != 0,
            wrong_rotor_position: bits & 0x08 != 0,
            motor_cable_fail: bits & 0x10 != 0,
            over_temperature: bits & 0x20 != 0,
            serial_fail: bits & 0x40 != 0,
            speed_out_of_range: bits & 0x80 != 0,
        })
    }
}
