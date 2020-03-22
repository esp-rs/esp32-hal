//! DPort peripheral configuration
//!
//! This peripheral contains many registers, which are used for various different functions.
//! Registers needed in other blocks can be split off.
//!
use esp32::{dport, DPORT};

/// Cpu Period Configuration Register
pub struct ClockControl {}

/// DPort registers related to clock control
impl ClockControl {
    pub(crate) fn cpu_per_conf(&self) -> &dport::CPU_PER_CONF {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).cpu_per_conf }
    }
}

/// Trait to split the DPORT peripheral into subsets
pub trait Split {
    fn split(self) -> (DPORT, ClockControl);
}

impl Split for DPORT {
    /// function to split the DPORT peripheral into subsets
    fn split(self) -> (DPORT, ClockControl) {
        (self, ClockControl {})
    }
}
