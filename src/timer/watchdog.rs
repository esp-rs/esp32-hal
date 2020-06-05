//! RTC Watchdog implementation
//!
//! # TODO:
//! - Add convenience methods for configuration
//! - Consider add default configuration for start with time only

use super::TimerGroup;
use crate::prelude::*;
use core::marker::PhantomData;
use embedded_hal::watchdog::{Watchdog, WatchdogDisable, WatchdogEnable};
use esp32::timg::wdtconfig0::*;

pub type WatchdogAction = WDT_STG0_A;
pub type WatchDogResetDuration = WDT_CPU_RESET_LENGTH_A;

const WATCHDOG_UNBLOCK_KEY: u32 = 0x50D83AA1;
const WATCHDOG_BLOCK_VALUE: u32 = 0x89ABCDEF;

pub struct WatchDog<TIMG>
where
    TIMG: TimerGroup,
{
    clock_control_config: super::ClockControlConfig,
    timg: *const esp32::timg::RegisterBlock,
    _group: PhantomData<TIMG>,
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
    /// Clock pre-scaling divider
    pub divider: u16,
}

impl<TIMG> WatchDog<TIMG>
where
    TIMG: TimerGroup,
{
    /// internal function to create new watchdog structure
    pub(crate) fn new(timg: TIMG, clock_control_config: super::ClockControlConfig) -> Self {
        WatchDog {
            clock_control_config,
            timg: &*timg as *const _ as *const esp32::timg::RegisterBlock,
            _group: PhantomData,
        }
    }

    /// function to unlock the watchdog (write unblock key) and lock after use
    fn access_registers<A, F: FnMut(&esp32::timg::RegisterBlock) -> A>(&mut self, mut f: F) -> A {
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
    fn calc_period(&self, value: u32) -> MicroSeconds {
        ((1_000_000u64 * value as u64 / u32::from(self.clock_control_config.apb_frequency()) as u64)
            as u32)
            .us()
    }

    /// Calculate ticks from period
    fn calc_ticks(&self, value: MicroSeconds) -> u32 {
        ((u32::from(value) as u64 * u32::from(self.clock_control_config.apb_frequency()) as u64)
            / 1_000_000u64) as u32
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
            period1: self.calc_period(timg.wdtconfig2.read().bits()),
            action1: stg0,
            period2: self.calc_period(timg.wdtconfig3.read().bits()),
            action2: stg1,
            period3: self.calc_period(timg.wdtconfig4.read().bits()),
            action3: stg2,
            period4: self.calc_period(timg.wdtconfig5.read().bits()),
            action4: stg3,
            cpu_reset_duration: wdtconfig0.wdt_cpu_reset_length().variant(),
            sys_reset_duration: wdtconfig0.wdt_sys_reset_length().variant(),
            divider: timg.wdtconfig1.read().wdt_clk_prescale().bits(),
        })
    }

    /// Change watchdog timer configuration and start
    pub fn set_config(&mut self, config: &WatchdogConfig) {
        let per1 = self.calc_ticks(config.period1.into());
        let per2 = self.calc_ticks(config.period2.into());
        let per3 = self.calc_ticks(config.period3.into());
        let per4 = self.calc_ticks(config.period4.into());

        self.access_registers(|timg| {
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
            timg.wdtconfig1
                .write(|w| unsafe { w.wdt_clk_prescale().bits(config.divider) });
            timg.wdtconfig2.write(|w| unsafe { w.bits(per1) });
            timg.wdtconfig3.write(|w| unsafe { w.bits(per2) });
            timg.wdtconfig4.write(|w| unsafe { w.bits(per3) });
            timg.wdtconfig5.write(|w| unsafe { w.bits(per4) });
        });
    }
}

/// Enable watchdog timer, only change stage 1 period, don't change default action
impl<TIMG> WatchdogEnable for WatchDog<TIMG>
where
    TIMG: TimerGroup,
{
    type Time = MicroSeconds;

    fn start<T: Into<Self::Time>>(&mut self, period: T) {
        let per = self.calc_ticks(period.into());

        self.access_registers(|timg| {
            unsafe { timg.wdtfeed.write(|w| w.wdt_feed().bits(0)) }
            timg.wdtconfig2.write(|w| unsafe { w.bits(per) });
            timg.wdtconfig1
                .write(|w| unsafe { w.wdt_clk_prescale().bits(1) });
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
impl<'a, TIMG> WatchdogDisable for WatchDog<TIMG>
where
    TIMG: TimerGroup,
{
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
impl<TIMG> Watchdog for WatchDog<TIMG>
where
    TIMG: TimerGroup,
{
    fn feed(&mut self) {
        self.access_registers(|timg| unsafe { timg.wdtfeed.write(|w| w.wdt_feed().bits(0)) });
    }
}
