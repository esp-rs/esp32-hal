use super::Error;
use crate::prelude::*;
use core::fmt;

use super::{
    dfs, CPUSource, ClockControlConfig, FastRTCSource, SlowRTCSource, CLOCK_CONTROL,
    CLOCK_CONTROL_MUTEX,
};

impl<'a> super::ClockControlConfig {
    // All the single word reads of frequencies and sources are thread and interrupt safe
    // as these are atomic.

    /// The current CPU frequency
    pub fn cpu_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().cpu_frequency }
    }

    /// The current APB frequency
    pub fn apb_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().apb_frequency }
    }

    /// The CPU frequency in the default state
    pub fn cpu_frequency_default(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().cpu_frequency_default }
    }

    /// The CPU frequency in the CPU lock state
    pub fn cpu_frequency_locked(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().cpu_frequency_locked }
    }

    /// The CPU frequency in the APB lock state
    pub fn cpu_frequency_apb_locked(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().cpu_frequency_apb_locked }
    }

    /// The APB frequency in the APB lock state
    pub fn apb_frequency_apb_locked(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().apb_frequency_apb_locked }
    }

    /// Is the reference clock 1MHz under all clock conditions
    pub fn is_ref_clock_stable(&self) -> bool {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().ref_clock_stable }
    }

    /// The current reference frequency
    pub fn ref_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().ref_frequency }
    }

    /// The current slow RTC frequency
    pub fn slow_rtc_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().slow_rtc_frequency }
    }

    /// The current fast RTC frequency
    pub fn fast_rtc_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().fast_rtc_frequency }
    }

    /// The current APLL frequency
    pub fn apll_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().apll_frequency }
    }

    /// The current PLL/2 frequency
    pub fn pll_d2_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().pll_d2_frequency }
    }

    /// The Xtal frequency
    pub fn xtal_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().xtal_frequency }
    }

    /// The 32kHz Xtal frequency
    pub fn xtal32k_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().xtal32k_frequency }
    }

    /// The current PLL frequency
    pub fn pll_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().pll_frequency }
    }

    /// The current 8MHz oscillator frequency
    pub fn rtc8m_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().rtc8m_frequency }
    }

    /// The current 8MHz oscillator frequency / 256
    pub fn rtc8md256_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().rtc8md256_frequency }
    }

    /// The current 150kHz oscillator frequency
    pub fn rtc_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().rtc_frequency }
    }

    /// The current source for the CPU and APB frequencies
    pub fn cpu_source(&self) -> CPUSource {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().cpu_source }
    }

    /// The current source for the slow RTC frequency
    pub fn slow_rtc_source(&self) -> SlowRTCSource {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().slow_rtc_source }
    }

    /// The current source for the fast RTC frequency
    pub fn fast_rtc_source(&self) -> FastRTCSource {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().fast_rtc_source }
    }

    // The lock and unlock calls are thread and interrupt safe because this is handled inside
    // the DFS routines

    /// Obtain a RAII lock to use the high CPU frequency
    pub fn lock_cpu_frequency(&self) -> dfs::LockCPU {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().lock_cpu_frequency() }
    }

    /// Obtain a RAII lock to use the APB CPU frequency
    pub fn lock_apb_frequency(&self) -> dfs::LockAPB {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().lock_apb_frequency() }
    }

    /// Obtain a RAII lock to keep the CPU from sleeping
    pub fn lock_awake(&self) -> dfs::LockAwake {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().lock_awake() }
    }

    /// Obtain a RAII lock to keep the PLL/2 from being turned off
    pub fn lock_plld2(&self) -> dfs::LockPllD2 {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().lock_plld2() }
    }

    /// Add callback which will be called when clock speeds are changed.
    ///
    /// NOTE: these callbacks are called in an interrupt free environment,
    /// so should be as short as possible
    // TODO: at the moment only static lifetime callbacks are allow
    pub fn add_callback<F>(&self, f: &'static F) -> Result<(), Error>
    where
        F: Fn(),
    {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().add_callback(f) }
    }

    /// Get the current count of the PCU, APB, Awake and PLL/2 locks
    pub fn get_lock_count(&self) -> dfs::Locks {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().get_lock_count() }
    }

    // The following routines are made thread and interrupt safe here

    /// Halt the designated core
    pub unsafe fn park_core(&mut self, core: crate::Core) {
        interrupt::free(|_| {
            CLOCK_CONTROL_MUTEX.lock();
            CLOCK_CONTROL.as_mut().unwrap().park_core(core);
        })
    }

    /// Start the APP (second) core
    ///
    /// The second core will start running with the function `entry`.
    pub fn unpark_core(&mut self, core: crate::Core) {
        interrupt::free(|_| {
            CLOCK_CONTROL_MUTEX.lock();
            unsafe { CLOCK_CONTROL.as_mut().unwrap().unpark_core(core) }
        })
    }

    /// Start the APP (second) core
    ///
    /// The second core will start running with the function `entry`.
    pub fn start_app_core(&mut self, entry: fn() -> !) -> Result<(), Error> {
        interrupt::free(|_| {
            CLOCK_CONTROL_MUTEX.lock();
            unsafe { CLOCK_CONTROL.as_mut().unwrap().start_app_core(entry) }
        })
    }

    // The following routines handle thread and interrupt safety themselves

    /// Get RTC tick count since boot
    ///
    /// *Note: this function takes up to one slow RTC clock cycle (can be up to 300us) and
    /// interrupts are blocked during this time.*
    pub fn rtc_tick_count(&self) -> TicksU64 {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().rtc_tick_count() }
    }

    /// Get nanoseconds since boot based on RTC tick count
    ///
    /// *Note: this function takes up to one slow RTC clock cycle (can be up to 300us) and
    /// interrupts are blocked during this time.*
    pub fn rtc_nanoseconds(&self) -> NanoSecondsU64 {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().rtc_nanoseconds() }
    }
}

impl fmt::Debug for ClockControlConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().fmt(f) }
    }
}
