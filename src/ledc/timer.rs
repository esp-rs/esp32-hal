use super::{HighSpeed, LowSpeed, Speed};
use esp32::ledc;
use crate::{clock_control::ClockControlConfig, units::*};

/// Timer errors
#[derive(Debug)]
pub enum Error {
    /// Invalid Divisor
    Divisor,
}

/// Clock source for HS Timers
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum HSClockSource {
    RefTick,
    APBClk,
}

/// Clock source for LS Timers
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum LSClockSource {
    RefTick,
    SlowClk,
}

/// Timer number
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Number {
    Timer0,
    Timer1,
    Timer2,
    Timer3,
}

/// Timer configuration
pub mod config {
    use crate::units::*;

    /// Number of bits reserved for duty cycle adjustment
    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    pub enum Duty {
        Duty1Bit = 1,
        Duty2Bit,
        Duty3Bit,
        Duty4Bit,
        Duty5Bit,
        Duty6Bit,
        Duty7Bit,
        Duty8Bit,
        Duty9Bit,
        Duty10Bit,
        Duty11Bit,
        Duty12Bit,
        Duty13Bit,
        Duty14Bit,
        Duty15Bit,
        Duty16Bit,
        Duty17Bit,
        Duty18Bit,
        Duty19Bit,
        Duty20Bit,
    }

    /// Timer configuration
    #[derive(Copy, Clone)]
    pub struct Config<CS> {
        pub duty: Duty,
        pub clock_source: CS,
        pub frequency: Hertz,
    }
}

/// Trait defining the type of timer source
pub trait TimerSpeed: Speed {
    type ClockSourceType;
}

/// Timer source type for LowSpeed timers
impl TimerSpeed for LowSpeed {
    type ClockSourceType = LSClockSource;
}

/// Timer source type for HighSpeed timers
impl TimerSpeed for HighSpeed {
    type ClockSourceType = HSClockSource;
}

/// Interface for Timers
pub trait TimerIFace<S: TimerSpeed> {
    /// Return the frequency of the timer
    fn get_freq(&self) -> Option<Hertz>;

    /// Configure the timer
    fn configure(&mut self, config: config::Config<S::ClockSourceType>) -> Result<(), Error>;

    /// Check if the timer has been configured
    fn is_configured(&self) -> bool;

    /// Return the duty resolution of the timer
    fn get_duty(&self) -> Option<config::Duty>;

    /// Return the timer number
    fn get_number(&self) -> Number;
}

/// Interface for HW configuration of timer
pub trait TimerHW<S: TimerSpeed> {
    /// Get the current source timer frequency from the HW
    fn get_freq_hw(&self) -> Option<Hertz>;

    /// Configure the HW for the timer
    fn configure_hw(&self, divisor: u32);

    /// Update the timer in HW
    fn update_hw(&self);
}

/// Timer struct
pub struct Timer<'a, S: TimerSpeed> {
    ledc: &'a ledc::RegisterBlock,
    clock_control_config: ClockControlConfig,
    number: Number,
    duty: Option<config::Duty>,
    configured: bool,
    clock_source: Option<S::ClockSourceType>,
}

impl<'a, S: TimerSpeed> TimerIFace<S> for Timer<'a, S>
where
    Timer<'a, S>: TimerHW<S>,
{
    /// Return the frequency of the timer
    fn get_freq(&self) -> Option<Hertz> {
        self.get_freq_hw()
    }

    /// Configure the timer
    fn configure(&mut self, config: config::Config<S::ClockSourceType>) -> Result<(), Error> {
        self.duty = Some(config.duty);
        self.clock_source = Some(config.clock_source);

        let src_freq: u32 = self.get_freq().unwrap().into();
        let precision = 2_u64.pow(config.duty as u32);
        let frequency: u32 = config.frequency.into();

        let divisor = (((src_freq as u64) << 8) + ((frequency as u64 * precision) / 2))
            / (frequency as u64 * precision);

        if divisor >= 0x10_0000 || divisor == 0 {
            return Err(Error::Divisor);
        }

        self.configure_hw(divisor as u32);
        self.update_hw();

        self.configured = true;

        Ok(())
    }

    /// Check if the timer has been configured
    fn is_configured(&self) -> bool {
        self.configured
    }

    /// Return the duty resolution of the timer
    fn get_duty(&self) -> Option<config::Duty> {
        self.duty
    }

    /// Return the timer number
    fn get_number(&self) -> Number {
        self.number
    }
}

impl<'a, S: TimerSpeed> Timer<'a, S> {
    /// Create a new intance of a timer
    pub fn new(
        ledc: &'a ledc::RegisterBlock,
        clock_control_config: ClockControlConfig,
        number: Number,
    ) -> Self {
        Timer {
            ledc,
            clock_control_config,
            number,
            duty: None,
            configured: false,
            clock_source: None,
        }
    }

    /// Helper function that return the current frequency of the LowSpeed global source
    fn get_slow_clock_freq(&self) -> Hertz {
        if self.ledc.conf.read().apb_clk_sel().bit_is_clear() {
            8.MHz().into()
        } else {
            self.clock_control_config.apb_frequency()
        }
    }
}

/// Timer HW implementation for LowSpeed timers
impl<'a> TimerHW<LowSpeed> for Timer<'a, LowSpeed> {
    /// Get the current source timer frequency from the HW
    fn get_freq_hw(&self) -> Option<Hertz> {
        self.clock_source.map(|cs| match cs {
            LSClockSource::RefTick => self.clock_control_config.ref_frequency(),
            LSClockSource::SlowClk => self.get_slow_clock_freq(),
        })
    }

    /// Configure the HW for the timer
    fn configure_hw(&self, divisor: u32) {
        let duty = self.duty.unwrap() as u8;
        let sel_lstimer = self.clock_source.unwrap() == LSClockSource::SlowClk;
        match self.number {
            Number::Timer0 => self.ledc.lstimer0_conf.write(|w| unsafe {
                w.tick_sel_lstimer0()
                    .bit(sel_lstimer)
                    .lstimer0_rst()
                    .clear_bit()
                    .lstimer0_pause()
                    .clear_bit()
                    .div_num_lstimer0()
                    .bits(divisor)
                    .lstimer0_lim()
                    .bits(duty)
            }),
            Number::Timer1 => self.ledc.lstimer1_conf.modify(|_, w| unsafe {
                w.tick_sel_lstimer1()
                    .bit(sel_lstimer)
                    .lstimer1_rst()
                    .clear_bit()
                    .lstimer1_pause()
                    .clear_bit()
                    .div_num_lstimer1()
                    .bits(divisor)
                    .lstimer1_lim()
                    .bits(duty)
            }),
            Number::Timer2 => self.ledc.lstimer2_conf.modify(|_, w| unsafe {
                w.tick_sel_lstimer2()
                    .bit(sel_lstimer)
                    .lstimer2_rst()
                    .clear_bit()
                    .lstimer2_pause()
                    .clear_bit()
                    .div_num_lstimer2()
                    .bits(divisor)
                    .lstimer2_lim()
                    .bits(duty)
            }),
            Number::Timer3 => self.ledc.lstimer3_conf.modify(|_, w| unsafe {
                w.tick_sel_lstimer3()
                    .bit(sel_lstimer)
                    .lstimer3_rst()
                    .clear_bit()
                    .lstimer3_pause()
                    .clear_bit()
                    .div_num_lstimer3()
                    .bits(divisor)
                    .lstimer3_lim()
                    .bits(duty)
            }),
        };
    }

    /// Update the timer in HW
    fn update_hw(&self) {
        match self.number {
            Number::Timer0 => self
                .ledc
                .lstimer0_conf
                .modify(|_, w| w.lstimer0_para_up().set_bit()),
            Number::Timer1 => self
                .ledc
                .lstimer1_conf
                .modify(|_, w| w.lstimer1_para_up().set_bit()),
            Number::Timer2 => self
                .ledc
                .lstimer2_conf
                .modify(|_, w| w.lstimer2_para_up().set_bit()),
            Number::Timer3 => self
                .ledc
                .lstimer3_conf
                .modify(|_, w| w.lstimer3_para_up().set_bit()),
        };
    }
}

/// Timer HW implementation for HighSpeed timers
impl<'a> TimerHW<HighSpeed> for Timer<'a, HighSpeed> {
    /// Get the current source timer frequency from the HW
    fn get_freq_hw(&self) -> Option<Hertz> {
        self.clock_source.map(|cs| match cs {
            HSClockSource::RefTick => self.clock_control_config.ref_frequency(),
            HSClockSource::APBClk => self.clock_control_config.apb_frequency(),
        })
    }

    /// Configure the HW for the timer
    fn configure_hw(&self, divisor: u32) {
        let duty = self.duty.unwrap() as u8;
        let sel_hstimer = self.clock_source.unwrap() == HSClockSource::APBClk;
        match self.number {
            Number::Timer0 => self.ledc.hstimer0_conf.modify(|_, w| unsafe {
                w.tick_sel_hstimer0()
                    .bit(sel_hstimer)
                    .hstimer0_rst()
                    .clear_bit()
                    .hstimer0_pause()
                    .clear_bit()
                    .div_num_hstimer0()
                    .bits(divisor)
                    .hstimer0_lim()
                    .bits(duty)
            }),
            Number::Timer1 => self.ledc.hstimer1_conf.modify(|_, w| unsafe {
                w.tick_sel_hstimer1()
                    .bit(sel_hstimer)
                    .hstimer1_rst()
                    .clear_bit()
                    .hstimer1_pause()
                    .clear_bit()
                    .div_num_hstimer1()
                    .bits(divisor)
                    .hstimer1_lim()
                    .bits(duty)
            }),
            Number::Timer2 => self.ledc.hstimer2_conf.modify(|_, w| unsafe {
                w.tick_sel_hstimer2()
                    .bit(sel_hstimer)
                    .hstimer2_rst()
                    .clear_bit()
                    .hstimer2_pause()
                    .clear_bit()
                    .div_num_hstimer2()
                    .bits(divisor)
                    .hstimer2_lim()
                    .bits(duty)
            }),
            Number::Timer3 => self.ledc.hstimer3_conf.modify(|_, w| unsafe {
                w.tick_sel_hstimer3()
                    .bit(sel_hstimer)
                    .hstimer3_rst()
                    .clear_bit()
                    .hstimer3_pause()
                    .clear_bit()
                    .div_num_hstimer3()
                    .bits(divisor)
                    .hstimer3_lim()
                    .bits(duty)
            }),
        };
    }

    /// Update the timer in HW
    fn update_hw(&self) {
        // Nothing to do for HS timers
    }
}
