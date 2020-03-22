//! RTC Watchdog implementation
//!
//! # TODO:
//! - Add convenience methods for configuration
//! - Consider add default configuration for start with time only

use crate::prelude::*;
use embedded_hal::watchdog::{Watchdog, WatchdogDisable, WatchdogEnable};
use esp32::generic::Variant::*;
use esp32::rtccntl::wdtconfig0::*;
use esp32::RTCCNTL;

pub type WatchdogAction = WDT_STG0_A;
pub type WatchDogResetDuration = WDT_CPU_RESET_LENGTH_A;
const WATCHDOG_UNBLOCK_KEY: u32 = 0x50D83AA1;
const WATCHDOG_BLOCK_VALUE: u32 = 0x89ABCDEF;

pub struct WatchDog {
    clock_control_config: super::ClockControlConfig,
    //    rtc_control: &'static esp32::rtccntl::RegisterBlock,
}

// TODO: make T for different periods independent
#[derive(Debug)]
pub struct WatchdogConfig<T: Into<MicroSeconds>> {
    pub period1: T,
    pub action1: WatchdogAction,
    pub period2: T,
    pub action2: WatchdogAction,
    pub period3: T,
    pub action3: WatchdogAction,
    pub period4: T,
    pub action4: WatchdogAction,
    pub cpu_reset_duration: WatchDogResetDuration,
    pub sys_reset_duration: WatchDogResetDuration,
    pub pause_in_sleep: bool,
    pub reset_cpu: [bool; 2],
}

unsafe impl Sync for WatchDog {}

impl WatchDog {
    pub(crate) fn new(clock_control_config: super::ClockControlConfig) -> Self {
        WatchDog {
            clock_control_config,
        }
    }

    fn access_registers<A, F: FnMut(&esp32::rtccntl::RegisterBlock) -> A>(
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

    fn calc_period(&self, value: u32) -> MicroSeconds {
        return (((1000000u64 * value as u64)
            / (u32::from(self.clock_control_config.slow_rtc_frequency()) as u64))
            as u32)
            .us();
    }
    fn calc_ticks(&self, value: MicroSeconds) -> u32 {
        return (u32::from(value) as u64
            * u32::from(self.clock_control_config.slow_rtc_frequency()) as u64
            / 1000000u64) as u32;
    }

    pub fn config(&self) -> Result<WatchdogConfig<MicroSeconds>, super::Error> {
        let rtc_control = unsafe { &(*RTCCNTL::ptr()) };
        let wdtconfig0 = rtc_control.wdtconfig0.read();

        let stg0 = match wdtconfig0.wdt_stg0().variant() {
            Val(x) => x,
            _ => return Err(super::Error::UnsupportedWatchdogConfig),
        };
        let stg1 = match wdtconfig0.wdt_stg1().variant() {
            Val(x) => x,
            _ => return Err(super::Error::UnsupportedWatchdogConfig),
        };
        let stg2 = match wdtconfig0.wdt_stg2().variant() {
            Val(x) => x,
            _ => return Err(super::Error::UnsupportedWatchdogConfig),
        };
        let stg3 = match wdtconfig0.wdt_stg3().variant() {
            Val(x) => x,
            _ => return Err(super::Error::UnsupportedWatchdogConfig),
        };

        Ok(WatchdogConfig::<MicroSeconds> {
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
            reset_cpu: [
                wdtconfig0.wdt_procpu_reset_en().bit(),
                wdtconfig0.wdt_appcpu_reset_en().bit(),
            ],
        })
    }

    pub fn set_config<T: Into<MicroSeconds> + Copy>(&mut self, config: &WatchdogConfig<T>) {
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
                    .bit(config.reset_cpu[0])
                    .wdt_appcpu_reset_en()
                    .bit(config.reset_cpu[1])
            });
            rtc_control.wdtconfig1.write(|w| unsafe { w.bits(per1) });
            rtc_control.wdtconfig2.write(|w| unsafe { w.bits(per2) });
            rtc_control.wdtconfig3.write(|w| unsafe { w.bits(per3) });
            rtc_control.wdtconfig4.write(|w| unsafe { w.bits(per4) });
        });
    }
}

impl WatchdogEnable for WatchDog {
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

impl<'a> WatchdogDisable for WatchDog {
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
// function is safe with immutable reference as it is a single atomic write
impl Watchdog for WatchDog {
    fn feed(&mut self) {
        // TODO: doesn't need the access protection
        self.access_registers(|rtc_control| {
            rtc_control.wdtfeed.write(|w| w.wdt_feed().set_bit());
        });
    }
}
