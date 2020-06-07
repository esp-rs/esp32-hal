//! RTC Watchdog implementation
//!
//! # TODO:
//! - Add convenience methods for configuration
//! - Consider add default configuration for start with time only

use super::Error;
use super::TimerGroup;
use crate::prelude::*;
use core::marker::PhantomData;
use embedded_hal::watchdog::{WatchdogDisable, WatchdogEnable};
use esp32::timg::wdtconfig0::WDT_STG0_A;
use esp32::timg::wdtconfig0::*;

pub type WatchdogAction = WDT_STG0_A;
pub type WatchDogResetDuration = WDT_CPU_RESET_LENGTH_A;

const WATCHDOG_UNBLOCK_KEY: u32 = 0x50D83AA1;
const WATCHDOG_BLOCK_VALUE: u32 = 0x89ABCDEF;

pub struct Watchdog<TIMG: TimerGroup> {
    clock_control_config: super::ClockControlConfig,
    timg: *const esp32::timg::RegisterBlock,
    _group: PhantomData<TIMG>,
}

unsafe impl<TIMG: TimerGroup> Send for Watchdog<TIMG> {}

/// Watchdog configuration
///
/// The watchdog has four stages.
/// Each of these stages can take a configurable action after expiry of the corresponding period.
/// When this action is done, it will move to the next stage.
/// The stage is reset to the first when the watchdog timer is fed.
#[derive(Debug)]
pub struct WatchdogConfig {
    // Delay before the first action to be taken
    pub period1: NanoSecondsU64,
    // First action
    pub action1: WatchdogAction,
    // Delay before the second action to be taken
    pub period2: NanoSecondsU64,
    // Second action
    pub action2: WatchdogAction,
    // Delay before the third action to be taken
    pub period3: NanoSecondsU64,
    // Third action
    pub action3: WatchdogAction,
    // Delay before the fourth action to be taken
    pub period4: NanoSecondsU64,
    // Fourth action
    pub action4: WatchdogAction,
    /// Duration of the cpu reset signal
    pub cpu_reset_duration: WatchDogResetDuration,
    /// Duration of the system reset signal
    pub sys_reset_duration: WatchDogResetDuration,
    /// Clock pre-scaling divider
    pub divider: u16,
}

impl<TIMG: TimerGroup> Watchdog<TIMG> {
    /// internal function to create new watchdog structure
    pub(crate) fn new(timg: TIMG, clock_control_config: super::ClockControlConfig) -> Self {
        Watchdog {
            clock_control_config,
            timg: &*timg as *const _ as *const esp32::timg::RegisterBlock,
            _group: PhantomData,
        }
    }

    /// function to unlock the watchdog (write unblock key) and lock after use
    fn access_registers<A, F: FnMut(&esp32::timg::RegisterBlock) -> A>(&self, mut f: F) -> A {
        // Unprotect write access to registers

        let timg = unsafe { &*(self.timg) };

        timg.wdtwprotect
            .write(|w| unsafe { w.bits(WATCHDOG_UNBLOCK_KEY) });

        let a = f(&timg);

        // Protect again
        timg.wdtwprotect
            .write(|w| unsafe { w.bits(WATCHDOG_BLOCK_VALUE) });

        a
    }

    /// Calculate period from ticks
    fn calc_period<T: Count>(&self, value: T) -> NanoSecondsU64 {
        let divider: u32 = unsafe { &*(self.timg) }
            .wdtconfig1
            .read()
            .wdt_clk_prescale()
            .bits()
            .into();
        TicksU64::from(value.into()) / (self.clock_control_config.apb_frequency() / divider)
    }

    /// Calculate ticks from period
    fn calc_ticks_with_divider<T: TimeU64>(&self, value: T, divider: u16) -> Ticks {
        let ticks = self.clock_control_config.apb_frequency() / u32::from(divider) * value.into();
        use core::convert::TryInto;
        ticks.try_into().unwrap()
    }

    /// Calculate ticks from period
    fn calc_ticks<T: TimeU64>(&self, value: T) -> Ticks {
        self.calc_ticks_with_divider(
            value,
            unsafe { &*(self.timg) }
                .wdtconfig1
                .read()
                .wdt_clk_prescale()
                .bits()
                .into(),
        )
    }

    /// Get watchdog configuration
    pub fn config(&self) -> Result<WatchdogConfig, super::Error> {
        let timg = unsafe { &*(self.timg) };
        let wdtconfig0 = timg.wdtconfig0.read();

        let stg0 = wdtconfig0.wdt_stg0().variant();
        let stg1 = wdtconfig0.wdt_stg1().variant();
        let stg2 = wdtconfig0.wdt_stg2().variant();
        let stg3 = wdtconfig0.wdt_stg3().variant();

        Ok(WatchdogConfig {
            period1: self.calc_period(Ticks(timg.wdtconfig2.read().bits())),
            action1: stg0,
            period2: self.calc_period(Ticks(timg.wdtconfig3.read().bits())),
            action2: stg1,
            period3: self.calc_period(Ticks(timg.wdtconfig4.read().bits())),
            action3: stg2,
            period4: self.calc_period(Ticks(timg.wdtconfig5.read().bits())),
            action4: stg3,
            cpu_reset_duration: wdtconfig0.wdt_cpu_reset_length().variant(),
            sys_reset_duration: wdtconfig0.wdt_sys_reset_length().variant(),
            divider: timg.wdtconfig1.read().wdt_clk_prescale().bits(),
        })
    }

    /// Change watchdog timer configuration and start
    pub fn set_config(&mut self, config: &WatchdogConfig) -> Result<(), Error> {
        self.access_registers(|timg| {
            timg.wdtconfig1
                .write(|w| unsafe { w.wdt_clk_prescale().bits(config.divider) });

            let per1: u32 = self.calc_ticks(config.period1).into();
            let per2: u32 = self.calc_ticks(config.period2).into();
            let per3: u32 = self.calc_ticks(config.period3).into();
            let per4: u32 = self.calc_ticks(config.period4).into();

            unsafe { timg.wdtfeed.write(|w| w.wdt_feed().bits(0)) }
            timg.wdtconfig0.modify(|_, w| {
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
                    .wdt_edge_int_en()
                    .set_bit()
                    .wdt_level_int_en()
                    .set_bit()
                    .wdt_flashboot_mod_en()
                    .clear_bit()
                    .wdt_cpu_reset_length()
                    .variant(config.cpu_reset_duration)
                    .wdt_sys_reset_length()
                    .variant(config.sys_reset_duration)
            });
            timg.wdtconfig2.write(|w| unsafe { w.bits(per1) });
            timg.wdtconfig3.write(|w| unsafe { w.bits(per2) });
            timg.wdtconfig4.write(|w| unsafe { w.bits(per3) });
            timg.wdtconfig5.write(|w| unsafe { w.bits(per4) });

            interrupt::free(|_| {
                super::TIMER_MUTEX.lock();
                timg.int_ena_timers.modify(|_, w| w.wdt_int_ena().set_bit());
            });
            Ok(())
        })
    }

    pub fn clear_interrupt(&mut self) {
        unsafe {
            (*(self.timg))
                .int_clr_timers
                .write(|w| w.wdt_int_clr().set_bit());
        }
    }
}

/// Enable watchdog timer, only change stage 1 period, don't change default action
impl<TIMG: TimerGroup> WatchdogEnable for Watchdog<TIMG> {
    type Time = NanoSecondsU64;

    fn start<T: Into<NanoSecondsU64>>(&mut self, period: T) {
        let divider = 1;
        let ticks = self.calc_ticks_with_divider(period.into(), divider);
        self.access_registers(|timg| {
            timg.wdtfeed.write(|w| unsafe { w.wdt_feed().bits(0) });
            timg.wdtconfig1
                .write(|w| unsafe { w.wdt_clk_prescale().bits(divider) });
            timg.wdtconfig2.write(|w| unsafe { w.bits(ticks.into()) });
            timg.wdtconfig0.modify(|_, w| {
                w.wdt_flashboot_mod_en()
                    .clear_bit()
                    .wdt_en()
                    .set_bit()
                    .wdt_stg0()
                    .variant(WatchdogAction::RESETSYSTEM)
            });
        });
    }
}

/// Disable watchdog timer
//#[allow(trivial_bounds)]
impl<'a, TIMG: TimerGroup> WatchdogDisable for Watchdog<TIMG> {
    fn disable(&mut self) {
        self.access_registers(|timg| {
            unsafe { timg.wdtfeed.write(|w| w.wdt_feed().bits(0)) }
            timg.wdtconfig0
                .modify(|_, w| w.wdt_flashboot_mod_en().clear_bit().wdt_en().clear_bit());
        });
    }
}

/// Feed (=reset) the watchdog timer
#[allow(trivial_bounds)]
impl<TIMG: TimerGroup> embedded_hal::watchdog::Watchdog for Watchdog<TIMG> {
    fn feed(&mut self) {
        self.access_registers(|timg| unsafe { timg.wdtfeed.write(|w| w.wdt_feed().bits(0)) });
    }
}
