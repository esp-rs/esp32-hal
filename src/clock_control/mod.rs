//! Clock and RTC watchdog control.
//!
//! Controls the clock source for CPU, RTC, APB, etc.
//! Also controls RTC watchdog timer.
//!
//! # TODO
//! - Finish PLL support
//! - Auto detect CPU frequency
//! - Auto detect flash frequency
//! - Calibrate internal oscillators
//! - Changing clock sources
//! - Make thread & interrupt safe
//! - Low Power Clock (LPClock, regs: DPORT_BT_LPCK_DIV_FRAC_REG,DPORT_BT_LPCK_DIV_INT_REG)
//! - LED clock selection in ledc peripheral

mod pll;
pub mod watchdog;

use crate::prelude::*;
use esp32::dport::cpu_per_conf::CPUPERIOD_SEL_A;
use esp32::generic::Variant::*;
use esp32::rtccntl::bias_conf::*;
use esp32::rtccntl::clk_conf::*;
use esp32::{APB_CTRL, RTCCNTL};

type CoreBias = DBIAS_WAK_A;

const DEFAULT_XTAL_FREQUENCY: Hertz = Hertz(26_000_000);

#[allow(dead_code)]
const DIG_DBIAS_240M_OR_FLASH_80M: CoreBias = CoreBias::BIAS_1V25;
#[allow(dead_code)]
const DIG_DBIAS_80M_160M: CoreBias = CoreBias::BIAS_1V10;

const DIG_DBIAS_XTAL: CoreBias = CoreBias::BIAS_1V10;
const DIG_DBIAS_2M: CoreBias = CoreBias::BIAS_1V00;

/// RTC Clock errors
#[derive(Debug)]
pub enum Error {
    /// Unsupported frequency configuration
    UnsupportedFreqConfig,
    UnsupportedWatchdogConfig,
    UnsupportedPLLConfig,
    FrequencyTooHigh,
    FrequencyTooLow,
    LockAlreadyReleased,
    TooManyCallBacks,
}

/// Reference clock frequency always at 1MHz
pub const REF_CLK_FREQ_1M: Hertz = Hertz(1_000_000);

#[allow(dead_code)]
const RTC_PLL_FREQ_320M: MegaHertz = MegaHertz(320);
#[allow(dead_code)]
const RTC_PLL_FREQ_480M: MegaHertz = MegaHertz(480);

const RTC_SLOW_CLK_FREQ_32K: Hertz = Hertz(32_768);
const RTC_SLOW_CLK_FREQ_150K: Hertz = Hertz(150_000);
const RTC_FAST_CLK_FREQ_8M: Hertz = Hertz(8_500_000); //With the default value of CK8M_DFREQ, 8M clock frequency is 8.5 MHz +/- 7%
const RTC_SLOW_CLK_FREQ_8MD256: Hertz = Hertz(RTC_FAST_CLK_FREQ_8M.0 / 256);

/* Delays for various clock sources to be enabled/switched.
 * All values are in microseconds.
 * TODO according to esp-idf: some of these are excessive, and should be reduced.
 */
const DELAY_FAST_CLK_SWITCH: MicroSeconds = MicroSeconds(3);
const DELAY_SLOW_CLK_SWITCH: MicroSeconds = MicroSeconds(300);
const DELAY_8M_ENABLE: MicroSeconds = MicroSeconds(50);

/// CPU/APB/REF clock source
#[derive(Debug, Copy, Clone)]
pub enum CPUSource {
    /// High frequency Xtal (26MHz or 40MHz)
    Xtal,
    /// PLL generated frequency from high frequency Xtal
    PLL,
    /// PLL generated frequency from high frequency Xtal
    APLL,
    /// 8MHz internal oscillator
    RTC8M,
}

/// Slow RTC clock source
#[derive(Debug, Copy, Clone)]
pub enum SlowRTCSource {
    /// 150kHz internal oscillator
    RTC150k,
    /// Low frequency Xtal (32kHz)
    Xtal32k,
    /// 8MHz internal oscillator (divided by 256)
    RTC8MD256,
}

/// Fast RTC clock source
#[derive(Debug, Copy, Clone)]
pub enum FastRTCSource {
    /// 8MHz internal oscillator
    RTC8M,
    /// High frequency Xtal (26MHz or 40MHz)
    XtalD4,
}

#[derive(Debug)]
struct ClockControlCurrent {
    /// CPU Frequency
    cpu_frequency: Hertz,
    /// APB Frequency
    pub apb_frequency: Hertz,
    /// REF Frequency
    pub ref_frequency: Hertz,
    /// APLL Frequency
    pub apll_frequency: Hertz,
    /// PLL/2 Frequency
    pub pll_d2_frequency: Hertz,
    /// Slow RTC Frequency
    pub slow_rtc_frequency: Hertz,
    /// Fast RTC Frequency
    pub fast_rtc_frequency: Hertz,

    /// XTAL Frequency
    pub xtal_frequency: Hertz,
    /// XTAL32K Frequency
    pub xtal32k_frequency: Hertz,
    /// RTC8M Frequency
    pub rtc8m_frequency: Hertz,
    /// RTC Frequency
    pub rtc_frequency: Hertz,
    /// PLL Frequency
    pub pll_frequency: Hertz,

    /// Source routing

    /// CPU/APB/REF Source
    pub cpu_source: CPUSource,
    /// Slow RTC Source
    pub slow_rtc_source: SlowRTCSource,
    /// Fast RTC Source
    pub fast_rtc_source: FastRTCSource,
}

impl Default for ClockControlCurrent {
    fn default() -> Self {
        ClockControlCurrent {
            cpu_frequency: Hertz(0),
            apb_frequency: Hertz(0),
            ref_frequency: Hertz(0),
            apll_frequency: Hertz(0),
            pll_d2_frequency: Hertz(0),
            slow_rtc_frequency: Hertz(0),
            fast_rtc_frequency: Hertz(0),
            xtal_frequency: Hertz(0),
            xtal32k_frequency: Hertz(0),
            rtc8m_frequency: Hertz(0),
            rtc_frequency: Hertz(0),
            pll_frequency: Hertz(0),

            cpu_source: CPUSource::Xtal,
            slow_rtc_source: SlowRTCSource::RTC150k,
            fast_rtc_source: FastRTCSource::XtalD4,
        }
    }
}

/// Clock configuration
#[derive(Copy, Clone)]
pub struct ClockControlConfig {}

fn do_callbacks() {
    // copy the callbacks to prevent needing to have interrupts disabled the entire time
    // as callback cannot be deleted this is ok.
    let (nr, callbacks) = xtensa_lx6_rt::interrupt::free(|_| unsafe {
        let nr = NR_CALLBACKS.lock();
        (*nr, CALLBACKS)
    });

    for i in 0..nr {
        callbacks[i]();
    }
}

use core::fmt::Write;

static mut CLOCK_CONTROL: Option<ClockControl> = None;

fn lock_apb_frequency_start() {
    do_callbacks()
}
fn lock_apb_frequency_stop() {
    do_callbacks()
}
fn lock_cpu_frequency_start() {
    do_callbacks()
}
fn lock_cpu_frequency_stop() {
    do_callbacks()
}
fn lock_awake_start() {
    do_callbacks()
}
fn lock_awake_stop() {
    do_callbacks()
}

static LOCK_APB_FREQUENCY: crate::lock_execute::LockExecute =
    crate::lock_execute::LockExecute::new(lock_apb_frequency_start, lock_apb_frequency_stop);
static LOCK_CPU_FREQUENCY: crate::lock_execute::LockExecute =
    crate::lock_execute::LockExecute::new(lock_cpu_frequency_start, lock_cpu_frequency_stop);
static LOCK_AWAKE: crate::lock_execute::LockExecute =
    crate::lock_execute::LockExecute::new(lock_awake_start, lock_awake_stop);

const MAX_CALLBACKS: usize = 10;
static mut CALLBACKS: [&dyn Fn(); MAX_CALLBACKS] = [&|| {}; MAX_CALLBACKS];

static mut NR_CALLBACKS: spin::Mutex<usize> = spin::Mutex::new(0);

impl<'a> ClockControlConfig {
    pub fn cpu_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.cpu_frequency }
    }
    pub fn apb_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.apb_frequency }
    }
    pub fn min_cpu_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().min_cpu_frequency }
    }
    pub fn max_cpu_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().max_cpu_frequency }
    }
    pub fn min_apb_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().min_cpu_frequency }
    }
    pub fn max_apb_frequency(&self) -> Hertz {
        Hertz(80_000_000)
    }
    pub fn ref_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.ref_frequency }
    }
    pub fn slow_rtc_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.slow_rtc_frequency }
    }
    pub fn fast_rtc_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.fast_rtc_frequency }
    }
    pub fn apll_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.apll_frequency }
    }
    pub fn pll_d2_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.pll_d2_frequency }
    }
    pub fn xtal_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.xtal_frequency }
    }
    pub fn xtal32k_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.xtal32k_frequency }
    }
    pub fn pll_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.pll_frequency }
    }
    pub fn rtc8m_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.rtc8m_frequency }
    }
    pub fn rtc_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.rtc_frequency }
    }
    pub fn cpu_source(&self) -> CPUSource {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.cpu_source }
    }
    pub fn slow_rtc_source(&self) -> SlowRTCSource {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.slow_rtc_source }
    }
    pub fn fast_rtc_source(&self) -> FastRTCSource {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.fast_rtc_source }
    }

    pub fn lock_apb_frequency(&self) -> crate::lock_execute::LockExecuteGuard<'a> {
        LOCK_APB_FREQUENCY.lock()
    }
    pub fn lock_cpu_frequency(&self) -> crate::lock_execute::LockExecuteGuard<'a> {
        LOCK_CPU_FREQUENCY.lock()
    }
    pub fn lock_awake(&self) -> crate::lock_execute::LockExecuteGuard<'a> {
        LOCK_AWAKE.lock()
    }

    pub fn add_callback<F>(&self, f: &'static F) -> Result<(), Error>
    where
        F: Fn(),
    {
        // need to disable interrupts, because otherwise deadlock can arise
        // when interrupt is called after mutex is obtained and interrupt
        // routine also adds callback
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut nr = unsafe { NR_CALLBACKS.lock() };

            if *nr >= MAX_CALLBACKS {
                return Err(Error::TooManyCallBacks);
            }

            unsafe { CALLBACKS[*nr] = f };
            *nr += 1;
            Ok(())
        })
    }
}

use core::fmt;

impl fmt::Debug for ClockControlConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            f.write_fmt(format_args!(
                "{:?}",
                CLOCK_CONTROL.as_ref().unwrap().current
            ))
        }
    }
}

/// cycle accurate delay using the cycle counter register
pub fn delay_cycles(clocks: u32) {
    let start = xtensa_lx6_rt::get_cycle_count();
    loop {
        if xtensa_lx6_rt::get_cycle_count().wrapping_sub(start) >= clocks {
            break;
        }
    }
}

/// Clock Control
pub struct ClockControl {
    rtc_control: RTCCNTL,
    apb_control: APB_CTRL,
    dport_control: crate::dport::ClockControl,

    min_cpu_frequency: Hertz,
    min_cpu_source: CPUSource,
    max_cpu_frequency: Hertz,
    max_cpu_source: CPUSource,
    light_sleep_enabled: bool,

    current: ClockControlCurrent,
}

pub fn delay<T: Into<NanoSeconds>>(time: T) {
    unsafe { CLOCK_CONTROL.as_ref().unwrap().delay(time) };
}

impl ClockControl {
    /// Create new ClockControl structure
    pub fn new(
        rtc_control: RTCCNTL,
        apb_control: APB_CTRL,
        dport_control: crate::dport::ClockControl,
    ) -> Self {
        let mut cc = ClockControl {
            rtc_control,
            apb_control,
            dport_control,
            min_cpu_frequency: Hertz(2_000_000),
            min_cpu_source: CPUSource::Xtal,
            max_cpu_frequency: Hertz(240_000_000),
            max_cpu_source: CPUSource::PLL,
            light_sleep_enabled: false,
            current: ClockControlCurrent::default(),
        };
        cc.init();
        cc
    }

    pub fn freeze(self) -> Result<(ClockControlConfig, watchdog::WatchDog), Error> {
        // can only occur one time as ClockControl is moved by this function and
        // the RTCNTRL and APBCNTRL peripherals are moved when ClockControl is created
        unsafe { CLOCK_CONTROL = Some(self) };

        let res = ClockControlConfig {};
        Ok((res, watchdog::WatchDog::new(res)))
    }

    fn update_current_config(&mut self) {
        self.current = ClockControlCurrent {
            cpu_frequency: self.cpu_frequency(),
            apb_frequency: self.apb_frequency(),
            ref_frequency: self.ref_frequency(),
            slow_rtc_frequency: self.slow_rtc_frequency(),
            fast_rtc_frequency: self.fast_rtc_frequency(),

            apll_frequency: 0.Hz(),
            pll_d2_frequency: self.pll_frequency() / 2,

            xtal_frequency: self.xtal_frequency(),
            xtal32k_frequency: 0.Hz(),
            pll_frequency: self.pll_frequency(),
            rtc8m_frequency: 0.Hz(),
            rtc_frequency: 0.Hz(),

            cpu_source: self.cpu_source(),
            slow_rtc_source: self.slow_rtc_source().unwrap_or(SlowRTCSource::RTC150k),
            fast_rtc_source: self.fast_rtc_source(),
        };
    }

    /// Initialize clock configuration
    fn init(&mut self) -> &mut Self {
        if self.rtc_control.clk_conf.read().soc_clk_sel().is_pll() {
            self.set_cpu_frequency_to_xtal(self.xtal_frequency())
                .unwrap();
        }

        self.update_current_config();

        self
    }

    fn time_to_cpu_cycles<T: Into<NanoSeconds>>(&self, time: T) -> u32 {
        (((self.current.cpu_frequency / 1000000.Hz()) as u64) * (u32::from(time.into()) as u64)
            / 1000) as u32
    }

    fn delay<T: Into<NanoSeconds>>(&self, time: T) {
        delay_cycles(self.time_to_cpu_cycles(time));
    }

    /// Check if a value from RTC_XTAL_FREQ_REG or RTC_APB_FREQ_REG are valid clocks
    fn clk_val_is_valid(val: u32) -> bool {
        (val & 0xffff) == ((val >> 16) & 0xffff) && val != 0 && val != u32::max_value()
    }

    /// Sets the CPU frequency using the Xtal to closest possible frequency (rounding up).
    ///
    /// The APB frequency follows the CPU frequency.
    /// Below 10MHz, the ref clock is not guaranteed to be at 1MHz
    ///
    /// So for a 40Mhz Xtal, valid frequencies are: 40, 20, 13.33, 10, 8, 6.67, 5.71, 5, 4.44, 4, ...
    /// So for a 26Mhz Xtal, valid frequencies are: 26, 13, 8.67, 6.5, 5.2, 4.33, 3.71, ...
    /// So for a 24Mhz Xtal, valid frequencies are: 24, 12, 8, 6, 4.8, 4, ...
    pub fn set_cpu_frequency_to_xtal<T: Into<Hertz> + Copy + PartialOrd>(
        &mut self,
        frequency: T,
    ) -> Result<&mut Self, Error> {
        match frequency.into() {
            f if f <= 1.kHz().into() => return Err(Error::FrequencyTooLow),
            f if f <= self.xtal_frequency() => {
                // calculate divider, only integer fractions of xtal_frequency are possible
                let div = self.xtal_frequency() / frequency.into();
                if div > u16::max_value() as u32 {
                    return Err(Error::FrequencyTooLow);
                }
                let actual_frequency = self.xtal_frequency() / (div as u32);

                let div_1m = actual_frequency / REF_CLK_FREQ_1M;

                // set divider from XTAL to CPU clock
                self.apb_control
                    .sysclk_conf
                    .modify(|_, w| unsafe { w.pre_div_cnt().bits(div as u16 - 1) });
                // adjust ref tick
                self.apb_control
                    .xtal_tick_conf
                    .write(|w| unsafe { w.xtal_tick_num().bits(div_1m as u8 - 1) });

                // switch clock source
                self.rtc_control
                    .clk_conf
                    .modify(|_, w| w.soc_clk_sel().xtal());

                self.pll_disable();

                // select appropriate voltage
                self.rtc_control.bias_conf.modify(|_, w| {
                    w.dig_dbias_wak()
                        .variant(if actual_frequency < 2.MHz().into() {
                            DIG_DBIAS_2M
                        } else {
                            DIG_DBIAS_XTAL
                        })
                });

                self.set_apb_frequency_to_scratch(actual_frequency);
            }
            _ => return Err(Error::FrequencyTooHigh),
        }

        self.update_current_config();

        self.wait_for_slow_cycle();

        self.pll_disable();

        Ok(self)
    }

    /// Sets the CPU frequency using the PLL to closest possible frequency (rounding up).
    ///
    /// The APB frequency is fixed at 80MHz.
    pub fn set_cpu_frequency_to_pll<T>(&mut self, frequency: T) -> Result<&mut Self, Error>
    where
        T: Into<Hertz> + Copy + PartialOrd,
    {
        // TODO: adjust bias if flash at 80MHz
        let (pll_frequency_high, cpuperiod_sel, dbias) = match frequency.into() {
            f if f <= 80.MHz().into() => (false, CPUPERIOD_SEL_A::SEL_80, DIG_DBIAS_80M_160M),
            f if f <= 160.MHz().into() => (false, CPUPERIOD_SEL_A::SEL_160, DIG_DBIAS_80M_160M),
            f if f <= 240.MHz().into() => {
                (true, CPUPERIOD_SEL_A::SEL_240, DIG_DBIAS_240M_OR_FLASH_80M)
            }
            _ => {
                return Err(Error::FrequencyTooHigh);
            }
        };

        self.pll_enable();
        self.wait_for_slow_cycle();

        self.dport_control
            .cpu_per_conf()
            .modify(|_, w| w.cpuperiod_sel().variant(cpuperiod_sel));

        self.rtc_control
            .bias_conf
            .modify(|_, w| w.dig_dbias_wak().variant(dbias));

        self.set_pll_frequency(pll_frequency_high)?;

        // switch clock source
        self.rtc_control
            .clk_conf
            .modify(|_, w| w.soc_clk_sel().pll());

        self.set_apb_frequency_to_scratch(80.MHz());
        self.wait_for_slow_cycle();

        self.update_current_config();
        Ok(self)
    }

    fn wait_for_slow_cycle(&mut self) {
        // TODO: properly implement wait_for_slow_cycles (https://github.com/espressif/esp-idf/blob/c1d0daf36d0dca81c23c226001560edfa51c30ea/components/soc/src/esp32/rtc_time.c#L155)
        self.delay(100.ms());
        //      unimplemented!()
    }

    /// Get Ref Tick frequency
    ///
    /// This frequency is usually 1MHz, but cannot be maintained when the APB_CLK is < 10MHz
    pub fn ref_frequency(&self) -> Hertz {
        match self.cpu_source() {
            CPUSource::PLL => 1.MHz().into(),
            CPUSource::Xtal => {
                let div = self
                    .apb_control
                    .xtal_tick_conf
                    .read()
                    .xtal_tick_num()
                    .bits();

                self.apb_frequency() / (div + 1) as u32
            }
            CPUSource::APLL => unimplemented!(),
            CPUSource::RTC8M => unimplemented!(),
        }
    }

    /// Get APB frequency
    ///
    /// This gets the APB frequency from the scratch register, which is initialized during the clock calibration
    pub fn apb_frequency(&self) -> Hertz {
        match self.cpu_source() {
            CPUSource::PLL => 80.MHz().into(),
            _ => self.cpu_frequency(),
        }
    }

    /// Set APB frequency
    ///
    /// Write the APB frequency to the scratch register for later retrieval
    fn set_apb_frequency_to_scratch<T: Into<Hertz>>(&mut self, frequency: T) -> &mut Self {
        // Write APB value into RTC_APB_FREQ_REG for compatibility with esp-idf
        let mut val = u32::from(frequency.into()) >> 12;
        val = val | (val << 16); // value needs to be copied in lower and upper 16 bits
        self.rtc_control
            .store5
            .write(|w| unsafe { w.scratch5().bits(val) });

        self
    }

    pub fn slow_rtc_source(&self) -> Result<SlowRTCSource, Error> {
        match self.rtc_control.clk_conf.read().ana_clk_rtc_sel().variant() {
            Val(ANA_CLK_RTC_SEL_A::SLOW_CK) => Ok(SlowRTCSource::RTC150k),
            Val(ANA_CLK_RTC_SEL_A::CK_XTAL_32K) => Ok(SlowRTCSource::Xtal32k),
            Val(ANA_CLK_RTC_SEL_A::CK8M_D256_OUT) => Ok(SlowRTCSource::RTC8MD256),
            _ => Err(Error::UnsupportedFreqConfig),
        }
    }

    /// Get RTC/Slow frequency
    pub fn slow_rtc_frequency(&self) -> Hertz {
        match self.slow_rtc_source() {
            Ok(SlowRTCSource::RTC150k) => RTC_SLOW_CLK_FREQ_150K,
            Ok(SlowRTCSource::Xtal32k) => RTC_SLOW_CLK_FREQ_32K,
            Ok(SlowRTCSource::RTC8MD256) => RTC_SLOW_CLK_FREQ_8MD256,
            _ => 0.Hz(),
        }
    }

    /// Set the Slow RTC clock source
    pub fn set_slow_rtc_source(&mut self, source: SlowRTCSource) -> &mut Self {
        match source {
            SlowRTCSource::RTC150k => self
                .rtc_control
                .clk_conf
                .modify(|_, w| w.ana_clk_rtc_sel().slow_ck()),
            SlowRTCSource::Xtal32k => self
                .rtc_control
                .clk_conf
                .modify(|_, w| w.ana_clk_rtc_sel().ck_xtal_32k()),
            SlowRTCSource::RTC8MD256 => self
                .rtc_control
                .clk_conf
                .modify(|_, w| w.ana_clk_rtc_sel().ck8m_d256_out()),
        }
        self.delay(DELAY_SLOW_CLK_SWITCH);
        self
    }

    /// Get the Fast RTC clock source
    pub fn fast_rtc_source(&self) -> FastRTCSource {
        match self
            .rtc_control
            .clk_conf
            .read()
            .fast_clk_rtc_sel()
            .variant()
        {
            FAST_CLK_RTC_SEL_A::CK8M => FastRTCSource::RTC8M,
            FAST_CLK_RTC_SEL_A::XTAL => FastRTCSource::XtalD4,
        }
    }

    /// Get RTC/Slow frequency
    pub fn fast_rtc_frequency(&self) -> Hertz {
        match self.fast_rtc_source() {
            FastRTCSource::RTC8M => RTC_FAST_CLK_FREQ_8M,
            FastRTCSource::XtalD4 => self.xtal_frequency() / 4,
        }
    }

    /// Set the Fast RTC clock source
    pub fn set_fast_rtc_source(&mut self, source: FastRTCSource) -> &mut Self {
        match source {
            FastRTCSource::RTC8M => self
                .rtc_control
                .clk_conf
                .modify(|_, w| w.fast_clk_rtc_sel().ck8m()),
            FastRTCSource::XtalD4 => self
                .rtc_control
                .clk_conf
                .modify(|_, w| w.fast_clk_rtc_sel().xtal()),
        }
        self.delay(DELAY_FAST_CLK_SWITCH);
        self
    }

    /// Get XTAL frequency.
    ///
    /// This gets the XTAL frequency from a scratch register, which is initialized during the clock calibration
    pub fn xtal_frequency(&self) -> Hertz {
        // We may have already written XTAL value into RTC_XTAL_FREQ_REG
        let xtal_freq_reg = self.rtc_control.store4.read().scratch4().bits();
        if !Self::clk_val_is_valid(xtal_freq_reg) {
            // return 40MHz as default (this is recommended )
            return DEFAULT_XTAL_FREQUENCY;
        }

        (xtal_freq_reg & 0x7fff).MHz().into() // bit15 is RTC_DISABLE_ROM_LOG flag
    }

    pub fn cpu_source(&self) -> CPUSource {
        match self.rtc_control.clk_conf.read().soc_clk_sel().variant() {
            SOC_CLK_SEL_A::XTAL => CPUSource::Xtal,
            SOC_CLK_SEL_A::PLL => CPUSource::PLL,
            SOC_CLK_SEL_A::APLL => CPUSource::APLL,
            SOC_CLK_SEL_A::CK8M => CPUSource::RTC8M,
        }
    }

    /// Get PLL frequency
    pub fn pll_frequency(&self) -> Hertz {
        if self.rtc_control.options0.read().bbpll_force_pd().bit() {
            return 0.Hz();
        }

        match self
            .dport_control
            .cpu_per_conf()
            .read()
            .cpuperiod_sel()
            .variant()
        {
            Val(CPUPERIOD_SEL_A::SEL_80) => 320.MHz().into(),
            Val(CPUPERIOD_SEL_A::SEL_160) => 320.MHz().into(),
            Val(CPUPERIOD_SEL_A::SEL_240) => 480.MHz().into(),
            _ => 0.Hz(),
        }
    }

    /// Get CPU frequency
    pub fn cpu_frequency(&self) -> Hertz {
        match self.cpu_source() {
            CPUSource::Xtal => {
                let divider = self.apb_control.sysclk_conf.read().pre_div_cnt().bits() + 1;
                self.xtal_frequency() / divider as u32
            }
            CPUSource::PLL => match self
                .dport_control
                .cpu_per_conf()
                .read()
                .cpuperiod_sel()
                .variant()
            {
                Val(CPUPERIOD_SEL_A::SEL_80) => 80.MHz().into(),
                Val(CPUPERIOD_SEL_A::SEL_160) => 160.MHz().into(),
                Val(CPUPERIOD_SEL_A::SEL_240) => 240.MHz().into(),
                _ => 0.Hz(),
            },
            CPUSource::RTC8M => RTC_FAST_CLK_FREQ_8M,
            CPUSource::APLL => unimplemented!(),
        }
    }
}
