/// Dport peripheral configuration
///
/// This peripheral contains many registers, which are used for various different functions
/// Registers needed in other blocks can be split off.
///
use esp32::{dport, DPORT};

pub struct DPort {
    dport: DPORT,
}

/// Cpu Period Configuration Register
pub struct ClockControl {}

impl ClockControl {
    pub(crate) fn cpu_per_conf(&self) -> &dport::CPU_PER_CONF {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).cpu_per_conf }
    }
}

pub trait Split {
    fn split(self) -> (DPort, ClockControl);
}

impl Split for DPORT {
    fn split(self) -> (DPort, ClockControl) {
        (DPort::new(self), ClockControl {})
    }
}

impl DPort {
    /// Create new ClockControl structure
    fn new(dport: DPORT) -> Self {
        DPort { dport }
    }
}
