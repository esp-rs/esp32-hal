//! Implementation of embedded hal delay traits using busy waiting
use crate::units::{MicroSeconds, MilliSeconds};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

#[derive(Clone, Default)]
pub struct Delay {}

impl Delay {
    pub fn new() -> Delay {
        Self::default()
    }
}

/// Delay in ms
///
/// *Note: Maximum duration is 2e32-1 ns ~ 4.29s *
impl<UXX: Into<u32>> DelayMs<UXX> for Delay {
    fn delay_ms(&mut self, ms: UXX) {
        crate::clock_control::sleep(MilliSeconds(ms.into()))
    }
}

/// Delay in us
///
/// *Note: Maximum duration is 2e32-1 ns ~ 4.29s *
impl<UXX: Into<u32>> DelayUs<UXX> for Delay {
    fn delay_us(&mut self, us: UXX) {
        crate::clock_control::sleep(MicroSeconds(us.into()))
    }
}
