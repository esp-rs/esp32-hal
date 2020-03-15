/// Dport peripheral configuration
///
/// This peripheral contains many registers, which are used for various different functions
/// Registers needed in other blocks can be split off.
///
use esp32::{dport, DPORT};

pub struct DPort {
    dport: DPORT,
    pub clock_control: ClockControl,
}

/// Cpu Period Configuration Register
pub struct ClockControl {}

impl ClockControl {
    pub(crate) fn cpu_per_conf(&self) -> &dport::CPU_PER_CONF {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).cpu_per_conf }
    }
}

impl DPort {
    /// Create new ClockControl structure
    pub fn new(dport: DPORT) -> Self {
        DPort {
            dport,
            clock_control: ClockControl {},
        }
    }
}
