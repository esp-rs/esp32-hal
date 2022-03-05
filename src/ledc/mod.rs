//! LEDC (LED PWM Controller) peripheral control
//!
//! Currently only supports fixed frequency output. Hardware fade support and interrupts are not currently
//! implemented. High Speed and Low Speed channels are available.
//!
//! # Example:
//! The following will configure the Low Speed channel 0 to 24Mhz output with 50% duty using the ABP Clock
//! ```
//!     let mut ledc = LEDC::new(clock_control_config);
//!
//!     ledc.set_global_slow_clock(LSGlobalClkSource::ABPClk);
//!     let mut lstimer0 = ledc.get_timer::<LowSpeed>(timer::Number::Timer0);
//!     lstimer0
//!         .configure(timer::config::Config {
//!             duty: timer::config::Duty::Duty1Bit,
//!             clock_source: timer::LSClockSource::SlowClk,
//!             frequency: 24_000_000.Hz(),
//!         })
//!     .unwrap();
//!
//!     let mut channel0 = ledc.get_channel(channel::Number::Channel0, pins.gpio4);
//!     channel0
//!         .configure(channel::config::Config {
//!             timer: &lstimer0,
//!             duty: 0.5,
//!         })
//!     .unwrap();
//! ```
//! # TODO
//! - Hardware fade support
//! - Interrupts

use crate::{clock_control::ClockControlConfig, dport, gpio::OutputPin};
use channel::Channel;
use timer::Timer;

use self::timer::TimerSpeed;

pub mod channel;
pub mod timer;

/// Global slow clock source
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum LSGlobalClkSource {
    EightMHz,
    ABPClk,
}

/// LEDC (LED PWM Controller)
pub struct LEDC<'a> {
    ledc: &'a esp32::ledc::RegisterBlock,
    clock_control_config: ClockControlConfig,
}

/// Used to specify HighSpeed Timer/Channel
pub struct HighSpeed {}
/// Used to specify LowSpeed Timer/Channel
pub struct LowSpeed {}

pub trait Speed {}

impl Speed for HighSpeed {}
impl Speed for LowSpeed {}

impl<'a> LEDC<'a> {
    /// Return a new LEDC
    pub fn new(clock_control_config: ClockControlConfig) -> Self {
        dport::enable_peripheral(dport::Peripheral::LEDC);
        dport::reset_peripheral(dport::Peripheral::LEDC);

        let ledc = unsafe { &*esp32::LEDC::ptr() };
        LEDC {
            ledc,
            clock_control_config,
        }
    }

    /// Set global slow clock source
    pub fn set_global_slow_clock(&mut self, clock_source: LSGlobalClkSource) {
        match clock_source {
            LSGlobalClkSource::EightMHz => self.ledc.conf.write(|w| w.apb_clk_sel().clear_bit()),
            LSGlobalClkSource::ABPClk => self.ledc.conf.write(|w| w.apb_clk_sel().set_bit()),
        };
    }

    /// Return a new timer
    pub fn get_timer<S: TimerSpeed>(&self, number: timer::Number) -> timer::Timer<S> {
        Timer::new(self.ledc, self.clock_control_config, number)
    }

    /// Return a new channel
    pub fn get_channel<S: TimerSpeed, O: OutputPin>(
        &self,
        number: channel::Number,
        output_pin: O,
    ) -> channel::Channel<S, O> {
        Channel::new(number, output_pin)
    }
}
