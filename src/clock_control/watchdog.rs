//! RTC Watchdog implementation
//!
//! # TODO:
//! - Add convenience methods for configuration
//! - Consider add default configuration for start with time only

use crate::prelude::*;
use crate::target;
use crate::target::rtccntl::wdtconfig0::*;
use crate::target::RTCCNTL;
use embedded_hal::watchdog::{WatchdogDisable, WatchdogEnable};

pub type WatchdogAction = WDT_STG0_A;
pub type WatchDogResetDuration = WDT_CPU_RESET_LENGTH_A;

const WATCHDOG_UNBLOCK_KEY: u32 = 0x50D83AA1;

const WATCHDOG_BLOCK_VALUE: u32 = 0x89ABCDEF;

pub struct Watchdog {
    clock_control_config: super::ClockControlConfig,
}

/// Watchdog configuration
///
/// The watchdog has four stages.
/// Each of these stages can take a configurable action after expiry of the corresponding period.
/// When this action is done, it will move to the next stage.
/// The stage is reset to the first when the watchdog timer is fed.
#[derive(Debug)]
pub struct WatchdogConfig {
    // Delay before the first action to be taken
    pub period1: MicroSeconds,
    // First action
    pub action1: WatchdogAction,
    // Delay before the second action to be taken
    pub period2: MicroSeconds,
    // Second action
    pub action2: WatchdogAction,
    // Delay before the third action to be taken
    pub period3: MicroSeconds,
    // Third action
    pub action3: WatchdogAction,
    // Delay before the fourth action to be taken
    pub period4: MicroSeconds,
    // Fourth action
    pub action4: WatchdogAction,
    /// Duration of the cpu reset signal
    pub cpu_reset_duration: WatchDogResetDuration,
    /// Duration of the system reset signal
    pub sys_reset_duration: WatchDogResetDuration,
    /// Pause the watchdog timer when the system is in sleep mode
    pub pause_in_sleep: bool,
    /// Indicates which cpu(s) will be reset when action is RESETCPU
    pub reset_cpu: (bool, bool),
}

impl Watchdog {
    /// internal function to create new watchdog structure
    pub(crate) fn new(clock_control_config: super::ClockControlConfig) -> Self {
        Watchdog {
            clock_control_config,
        }
    }

    /// function to unlock the watchdog (write unblock key) and lock after use
    fn access_registers<A, F: FnMut(&target::rtccntl::RegisterBlock) -> A>(
        &mut self,
        mut f: F,
    ) -> A {
        // Unprotect write access to registers
        let rtc_control = unsafe { &(*RTCCNTL::ptr()) };

        rtc_control
            .wdtwprotect
            .write(|w| unsafe { w.bits(WATCHDOG_UNBLOCK_KEY) });

        let a = f(rtc_control);

        // Protect again
        rtc_control
            .wdtwprotect
            .write(|w| unsafe { w.bits(WATCHDOG_BLOCK_VALUE) });

        a
    }

    /// Calculate period from ref ticks
    fn calc_period(&self, value: u32) -> MicroSeconds {
        (((1000000u64 * value as u64)
            / (u32::from(self.clock_control_config.slow_rtc_frequency()) as u64)) as u32)
            .us()
    }

    /// Calculate ref ticks from period
    fn calc_ticks(&self, value: MicroSeconds) -> u32 {
        (u32::from(value) as u64 * u32::from(self.clock_control_config.slow_rtc_frequency()) as u64
            / 1000000u64) as u32
    }

    /// Get watchdog configuration
    pub fn config(&self) -> Result<WatchdogConfig, super::Error> {
        let rtc_control = unsafe { &(*RTCCNTL::ptr()) };
        let wdtconfig0 = rtc_control.wdtconfig0.read();

        let stg0 = match wdtconfig0.wdt_stg0().variant() {
            Some(x) => x,
            _ => return Err(super::Error::UnsupportedWatchdogConfig),
        };
        let stg1 = match wdtconfig0.wdt_stg1().variant() {
            Some(x) => x,
            _ => return Err(super::Error::UnsupportedWatchdogConfig),
        };
        let stg2 = match wdtconfig0.wdt_stg2().variant() {
            Some(x) => x,
            _ => return Err(super::Error::UnsupportedWatchdogConfig),
        };
        let stg3 = match wdtconfig0.wdt_stg3().variant() {
            Some(x) => x,
            _ => return Err(super::Error::UnsupportedWatchdogConfig),
        };

        Ok(WatchdogConfig {
            period1: self.calc_period(rtc_control.wdtconfig1.read().bits()),
            action1: stg0,
            period2: self.calc_period(rtc_control.wdtconfig2.read().bits()),
            action2: stg1,
            period3: self.calc_period(rtc_control.wdtconfig3.read().bits()),
            action3: stg2,
            period4: self.calc_period(rtc_control.wdtconfig4.read().bits()),
            action4: stg3,
            cpu_reset_duration: wdtconfig0.wdt_cpu_reset_length().variant(),
            sys_reset_duration: wdtconfig0.wdt_sys_reset_length().variant(),
            pause_in_sleep: wdtconfig0.wdt_pause_in_slp().bit(),
            reset_cpu: (
                wdtconfig0.wdt_procpu_reset_en().bit(),
                wdtconfig0.wdt_appcpu_reset_en().bit(),
            ),
        })
    }

    /// Change watchdog timer configuration and start
    pub fn set_config(&mut self, config: &WatchdogConfig) {
        let per1 = self.calc_ticks(config.period1.into());
        let per2 = self.calc_ticks(config.period2.into());
        let per3 = self.calc_ticks(config.period3.into());
        let per4 = self.calc_ticks(config.period4.into());

        self.access_registers(|rtc_control| {
            rtc_control.wdtfeed.write(|w| w.wdt_feed().set_bit());
            rtc_control.wdtconfig0.modify(|_, w| {
                w.wdt_stg0()
                    .variant(config.action1)
                    .wdt_stg1()
                    .variant(config.action2)
                    .wdt_stg2()
                    .variant(config.action3)
                    .wdt_stg3()
                    .variant(config.action4)
                    .wdt_en()
                    .set_bit()
                    .wdt_flashboot_mod_en()
                    .clear_bit()
                    .wdt_cpu_reset_length()
                    .variant(config.cpu_reset_duration)
                    .wdt_sys_reset_length()
                    .variant(config.sys_reset_duration)
                    .wdt_pause_in_slp()
                    .bit(config.pause_in_sleep)
                    .wdt_procpu_reset_en()
                    .bit(config.reset_cpu.0)
                    .wdt_appcpu_reset_en()
                    .bit(config.reset_cpu.1)
            });
            rtc_control.wdtconfig1.write(|w| unsafe { w.bits(per1) });
            rtc_control.wdtconfig2.write(|w| unsafe { w.bits(per2) });
            rtc_control.wdtconfig3.write(|w| unsafe { w.bits(per3) });
            rtc_control.wdtconfig4.write(|w| unsafe { w.bits(per4) });
        });
    }
}

/// Enable watchdog timer, only change stage 1 period, don't change default action
impl WatchdogEnable for Watchdog {
    type Time = MicroSeconds;

    fn start<T: Into<Self::Time>>(&mut self, period: T) {
        let per = self.calc_ticks(period.into());

        self.access_registers(|rtc_control| {
            rtc_control.wdtfeed.write(|w| w.wdt_feed().set_bit());
            rtc_control
                .wdtconfig1
                .write(|w| unsafe { w.wdt_stg0_hold().bits(per) });
            rtc_control.wdtconfig0.modify(|_, w| {
                w.wdt_flashboot_mod_en()
                    .clear_bit()
                    .wdt_en()
                    .set_bit()
                    .wdt_pause_in_slp()
                    .set_bit()
                    .wdt_stg0()
                    .variant(WatchdogAction::RESETRTC)
            });
        });
    }
}

/// Disable watchdog timer
impl<'a> WatchdogDisable for Watchdog {
    fn disable(&mut self) {
        self.access_registers(|rtc_control| {
            rtc_control.wdtfeed.write(|w| w.wdt_feed().set_bit());
            rtc_control
                .wdtconfig0
                .modify(|_, w| w.wdt_flashboot_mod_en().clear_bit().wdt_en().clear_bit());
        });
    }
}

/// Feed (=reset) the watchdog timer
impl embedded_hal::watchdog::Watchdog for Watchdog {
    fn feed(&mut self) {
        self.access_registers(|rtc_control| {
            rtc_control.wdtfeed.write(|w| w.wdt_feed().set_bit());
        });
    }
}
