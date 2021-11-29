use channel::Channel;
use crate::{clock_control::ClockControlConfig, dport};
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

/// LEDC
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
    pub fn get_channel<S: TimerSpeed>(&self, number: channel::Number) -> channel::Channel<S> {
        Channel::new(number)
    }
}
