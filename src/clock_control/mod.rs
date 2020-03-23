//! Clock and RTC watchdog control.
//!
//! Controls the clock source for CPU, RTC, APB, etc.
//! Also controls RTC watchdog timer.
//!
//! # TODO
//! - Auto detect CPU frequency
//! - Auto detect flash frequency
//! - Calibrate internal oscillators
//! - Changing clock sources
//! - Low Power Clock (LPClock, regs: DPORT_BT_LPCK_DIV_FRAC_REG,DPORT_BT_LPCK_DIV_INT_REG)
//! - LED clock selection in ledc peripheral
//! - 8M and 8MD256 enable/disable
//! - CPU from 8M
//! - APLL support
//! - Implement light sleep
//! - Check possibility of 40M or 26M APB clock from PLL

mod dfs;
mod pll;
pub mod watchdog;

use crate::prelude::*;
use esp32::dport::cpu_per_conf::CPUPERIOD_SEL_A;
use esp32::generic::Variant::*;
use esp32::rtccntl::bias_conf::*;
use esp32::rtccntl::clk_conf::*;
use esp32::{APB_CTRL, RTCCNTL};

const DEFAULT_XTAL_FREQUENCY: Hertz = Hertz(26_000_000);

const DIG_DBIAS_240M_OR_FLASH_80M: DBIAS_WAK_A = DBIAS_WAK_A::BIAS_1V25;
const DIG_DBIAS_80M_160M: DBIAS_WAK_A = DBIAS_WAK_A::BIAS_1V10;

const DIG_DBIAS_XTAL: DBIAS_WAK_A = DBIAS_WAK_A::BIAS_1V10;
const DIG_DBIAS_2M: DBIAS_WAK_A = DBIAS_WAK_A::BIAS_1V00;

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
    TooManyCallbacks,
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
const DELAY_DBIAS_RAISE: MicroSeconds = MicroSeconds(3);

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

static mut CLOCK_CONTROL: Option<ClockControl> = None;

impl<'a> ClockControlConfig {
    pub fn cpu_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.cpu_frequency }
    }
    pub fn apb_frequency(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.apb_frequency }
    }
    pub fn cpu_frequency_min(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().cpu_frequency_min }
    }
    pub fn cpu_frequency_max(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().cpu_frequency_max }
    }
    pub fn cpu_frequency_apb(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().cpu_frequency_apb }
    }

    pub fn apb_frequency_max(&self) -> Hertz {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().apb_frequency_max }
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

    pub fn lock_cpu_frequency(&self) -> dfs::ExecuteGuardCPU {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().lock_cpu_frequency() }
    }
    pub fn lock_apb_frequency(&self) -> dfs::ExecuteGuardAPB {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().lock_apb_frequency() }
    }
    pub fn lock_awake(&self) -> dfs::ExecuteGuardAwake {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().lock_awake() }
    }

    /// Add callback which will be called when clock speeds are changed.
    ///
    /// NOTE: these callbacks are called in an interrupt free environment,
    /// so should be as short as possible
    ///
    /// TODO: at the moment only static lifetime callbacks are allow
    pub fn add_callback<F>(&self, f: &'static F) -> Result<(), Error>
    where
        F: Fn(),
    {
        unsafe { CLOCK_CONTROL.as_mut().unwrap().add_callback(f) }
    }
}

use core::fmt;

impl fmt::Debug for ClockControlConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { CLOCK_CONTROL.as_ref().unwrap().current.fmt(f) }
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

    cpu_frequency_min: Hertz,
    cpu_source_min: CPUSource,
    cpu_frequency_max: Hertz,
    cpu_source_max: CPUSource,
    cpu_frequency_apb: Hertz,
    cpu_source_apb: CPUSource,
    light_sleep_enabled: bool,

    apb_frequency_max: Hertz,

    current: ClockControlCurrent,
    dfs: dfs::DFS,
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
            cpu_frequency_min: Hertz(10_000_000),
            cpu_source_min: CPUSource::Xtal,
            cpu_frequency_max: Hertz(240_000_000),
            cpu_source_max: CPUSource::PLL,
            cpu_frequency_apb: Hertz(80_000_000),
            cpu_source_apb: CPUSource::PLL,
            apb_frequency_max: Hertz(80_000_000),
            light_sleep_enabled: false,
            current: ClockControlCurrent::default(),
            dfs: dfs::DFS::new(),
        };
        cc.init();
        cc
    }

    pub fn freeze(self) -> Result<(ClockControlConfig, watchdog::WatchDog), Error> {
        // can only occur one time as ClockControl is moved by this function and
        // the RTCCNTL and APBCTRL peripherals are moved when ClockControl is created
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

    pub fn set_cpu_frequencies<T1, T2, T3>(
        &mut self,
        cpu_source_min: CPUSource,
        cpu_frequency_min: T1,
        cpu_source_max: CPUSource,
        cpu_frequency_max: T2,
        cpu_source_apb: CPUSource,
        cpu_frequency_apb: T3,
    ) -> Result<&mut Self, Error>
    where
        T1: Into<Hertz> + Copy + PartialOrd,
        T2: Into<Hertz> + Copy + PartialOrd,
        T3: Into<Hertz> + Copy + PartialOrd,
    {
        match cpu_source_min {
            CPUSource::APLL | CPUSource::RTC8M => return Err(Error::UnsupportedFreqConfig),
            _ => {}
        }
        match cpu_source_max {
            CPUSource::APLL | CPUSource::RTC8M => return Err(Error::UnsupportedFreqConfig),
            _ => {}
        }
        match cpu_source_apb {
            CPUSource::APLL | CPUSource::RTC8M => return Err(Error::UnsupportedFreqConfig),
            _ => {}
        }

        if cpu_frequency_min.into() < 1.kHz().into()
            || cpu_frequency_max.into() < 1.kHz().into()
            || cpu_frequency_apb.into() < 1.kHz().into()
        {
            return Err(Error::FrequencyTooLow);
        }

        if cpu_frequency_min.into() > 240.MHz().into()
            || cpu_frequency_max.into() > 240.MHz().into()
            || cpu_frequency_apb.into() > 240.MHz().into()
        {
            return Err(Error::FrequencyTooHigh);
        }

        self.cpu_source_min = cpu_source_min;
        self.cpu_frequency_min = cpu_frequency_min.into();
        self.cpu_source_max = cpu_source_max;
        self.cpu_frequency_max = cpu_frequency_max.into();
        self.cpu_source_apb = cpu_source_apb;
        self.cpu_frequency_apb = cpu_frequency_apb.into();

        match self.cpu_source_apb {
            CPUSource::PLL => self.apb_frequency_max = Hertz(80_000_000),
            CPUSource::Xtal => {
                let div = core::cmp::max(1, self.xtal_frequency() / self.cpu_frequency_apb);
                self.apb_frequency_max = self.xtal_frequency() / div;
            }
            _ => return Err(Error::UnsupportedFreqConfig),
        }

        self.set_cpu_frequency_min()?;
        Ok(self)
    }

    fn set_cpu_frequency_min(&mut self) -> Result<&mut Self, Error> {
        self.set_cpu_frequency(self.cpu_source_min, self.cpu_frequency_min)
    }
    fn set_cpu_frequency_max(&mut self) -> Result<&mut Self, Error> {
        self.set_cpu_frequency(self.cpu_source_max, self.cpu_frequency_max)
    }
    fn set_cpu_frequency_apb(&mut self) -> Result<&mut Self, Error> {
        self.set_cpu_frequency(self.cpu_source_apb, self.cpu_frequency_apb)
    }

    fn set_cpu_frequency<T: Into<Hertz> + Copy + PartialOrd>(
        &mut self,
        source: CPUSource,
        frequency: T,
    ) -> Result<&mut Self, Error> {
        match source {
            CPUSource::Xtal => self.set_cpu_frequency_to_xtal(frequency),
            CPUSource::PLL => self.set_cpu_frequency_to_pll(frequency),
            _ => Err(Error::UnsupportedFreqConfig),
        }
    }

    /// Sets the CPU frequency using the Xtal to closest possible frequency (rounding up).
    ///
    /// The APB frequency follows the CPU frequency.
    /// Below 10MHz, the ref clock is not guaranteed to be at 1MHz
    ///
    /// So for a 40Mhz Xtal, valid frequencies are: 40, 20, 13.33, 10, 8, 6.67, 5.71, 5, 4.44, 4, ...
    /// So for a 26Mhz Xtal, valid frequencies are: 26, 13, 8.67, 6.5, 5.2, 4.33, 3.71, ...
    /// So for a 24Mhz Xtal, valid frequencies are: 24, 12, 8, 6, 4.8, 4, ...
    fn set_cpu_frequency_to_xtal<T: Into<Hertz> + Copy + PartialOrd>(
        &mut self,
        frequency: T,
    ) -> Result<&mut Self, Error> {
        let mut f_hz: Hertz = frequency.into();

        if f_hz < 1.kHz().into() {
            return Err(Error::FrequencyTooLow);
        }

        if f_hz > self.xtal_frequency() {
            f_hz = self.xtal_frequency();
        }
        // calculate divider, only integer fractions of xtal_frequency are possible
        let div = core::cmp::max(1, self.xtal_frequency() / f_hz);

        if div > u16::max_value() as u32 {
            return Err(Error::FrequencyTooLow);
        }

        let actual_frequency = self.xtal_frequency() / (div as u32);

        let div_1m = actual_frequency / REF_CLK_FREQ_1M;

        // select appropriate voltage
        if actual_frequency > 2.MHz().into() {
            self.rtc_control
                .bias_conf
                .modify(|_, w| w.dig_dbias_wak().variant(DIG_DBIAS_XTAL))
        };

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

        // select appropriate voltage
        if actual_frequency <= 2.MHz().into() {
            self.rtc_control
                .bias_conf
                .modify(|_, w| w.dig_dbias_wak().variant(DIG_DBIAS_2M))
        };

        self.pll_disable();

        self.set_apb_frequency_to_scratch(actual_frequency);

        self.update_current_config();

        self.wait_for_slow_cycle();

        // TODO: keep enabled if PLL_D2 is used in peripherals?
        self.pll_disable();

        Ok(self)
    }

    /// Sets the CPU frequency using the PLL to closest possible frequency (rounding up).
    ///
    /// The APB frequency is fixed at 80MHz.
    fn set_cpu_frequency_to_pll<T>(&mut self, frequency: T) -> Result<&mut Self, Error>
    where
        T: Into<Hertz> + Copy + PartialOrd,
    {
        // TODO: adjust bias if flash at 80MHz
        let (pll_frequency_high, cpuperiod_sel, dbias) = match frequency.into() {
            f if f <= 80.MHz().into() => (false, CPUPERIOD_SEL_A::SEL_80, DIG_DBIAS_80M_160M),
            f if f <= 160.MHz().into() => (false, CPUPERIOD_SEL_A::SEL_160, DIG_DBIAS_80M_160M),
            _ => (true, CPUPERIOD_SEL_A::SEL_240, DIG_DBIAS_240M_OR_FLASH_80M),
        };

        // TODO: optimize speed of switching temporarily to xtal when pll frequency needs to change
        if pll_frequency_high != (self.pll_frequency() == 480.MHz().into()) {
            self.set_cpu_frequency_to_xtal(self.xtal_frequency())?;
        }

        self.pll_enable();
        self.wait_for_slow_cycle();

        // if high frequency requested raise voltage first
        if pll_frequency_high {
            self.rtc_control
                .bias_conf
                .modify(|_, w| w.dig_dbias_wak().variant(dbias));

            delay(DELAY_DBIAS_RAISE);
        }

        self.set_pll_frequency(pll_frequency_high)?;

        self.dport_control
            .cpu_per_conf()
            .modify(|_, w| w.cpuperiod_sel().variant(cpuperiod_sel));

        // if low frequency requested lower voltage after
        if !pll_frequency_high {
            self.rtc_control
                .bias_conf
                .modify(|_, w| w.dig_dbias_wak().variant(dbias));
        }

        // switch clock source
        self.rtc_control
            .clk_conf
            .modify(|_, w| w.soc_clk_sel().pll());

        self.wait_for_slow_cycle();

        self.set_apb_frequency_to_scratch(80.MHz());
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
