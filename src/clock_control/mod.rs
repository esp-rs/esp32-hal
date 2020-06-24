//! Clock and RTC watchdog control.
//!
//! Controls the clock source for CPU, RTC, APB, etc.
//! Also controls RTC watchdog timer.
//!
//! # TODO
//! - Auto detect flash frequency
//! - Low Power Clock (LPClock, regs: DPORT_BT_LPCK_DIV_FRAC_REG,DPORT_BT_LPCK_DIV_INT_REG)
//! - LED clock selection in ledc peripheral
//! - 8M and 8MD256 enable/disable
//! - 150kHz enable/disable
//! - APLL support
//! - Implement light sleep
//! - 32kHz Xtal support
//! - Allow 8.5MHz clock to be tuned
//! - Automatic enabling/disabling of 8MHz source (when not in use for rtc_fast_clk or cpu frequency)

use crate::prelude::*;
use crate::target;
use crate::target::dport::cpu_per_conf::CPUPERIOD_SEL_A;
use crate::target::generic::Variant::*;
use crate::target::rtccntl::clk_conf::*;
use crate::target::rtccntl::cntl::*;
use crate::target::{APB_CTRL, RTCCNTL, TIMG0};
use core::fmt;
use xtensa_lx6::timer::{delay, get_cycle_count};

pub mod config;
pub mod cpu;
pub mod dfs;
mod pll;
pub mod watchdog;

/////////////////////////////////
// Configurable constants
//

// default Xtal frequency
const DEFAULT_XTAL_FREQUENCY: Hertz = Hertz(40_000_000);

// default frequencies for Dynamic Frequency Switching
const CPU_SOURCE_DEFAULT_DEFAULT: CPUSource = CPUSource::PLL;
const CPU_FREQ_MIN_DEFAULT: Hertz = Hertz(80_000_000);
const CPU_SOURCE_LOCKED_DEFAULT: CPUSource = CPUSource::PLL;
const CPU_FREQ_MAX_DEFAULT: Hertz = Hertz(240_000_000);
const CPU_SOURCE_APB_LOCKED_DEFAULT: CPUSource = CPUSource::PLL;
const CPU_FREQ_APB_DEFAULT: Hertz = Hertz(80_000_000);

/////////////////////////////////
// Non-configurable constants
//

const FREQ_OFF: Hertz = Hertz(0);

// Reference clock frequency always at 1MHz
const REF_CLK_FREQ_1M: Hertz = Hertz(1_000_000);

// PLL slow (for 80 and 160MHz CPU) and fast (for 240MHz CPU) frequency
const PLL_FREQ_320M: Hertz = Hertz(320_000_000);
const PLL_FREQ_480M: Hertz = Hertz(480_000_000);

// Possible Xtal frequencies
pub const XTAL_FREQUENCY_40M: Hertz = Hertz(40_000_000);
pub const XTAL_FREQUENCY_26M: Hertz = Hertz(26_000_000);
pub const XTAL_FREQUENCY_24M: Hertz = Hertz(24_000_000);
pub const XTAL_FREQUENCY_AUTO: Hertz = Hertz(0);

// Thresholds for auto frequency detection
const XTAL_FREQUENCY_THRESHOLD: Hertz = Hertz(50_000_000);
const XTAL_FREQUENCY_40M_THRESHOLD: Hertz = Hertz(33_000_000);
const XTAL_FREQUENCY_26M_THRESHOLD: Hertz = Hertz(24_500_000);
const XTAL_FREQUENCY_24M_THRESHOLD: Hertz = Hertz(20_000_000);

// Xtal 32kHz frequency
const XTAL32K_FREQUENCY: Hertz = Hertz(32_768);

// minimum CPU frequency
const CPU_FREQ_MIN: Hertz = Hertz(1_000);

// CPU frequency at which higher frequency is required
const CPU_FREQ_2M: Hertz = Hertz(2_000_000);

// CPU frequencies when using PLL
const CPU_FREQ_80M: Hertz = Hertz(80_000_000);
const CPU_FREQ_160M: Hertz = Hertz(160_000_000);
const CPU_FREQ_240M: Hertz = Hertz(240_000_000);

// standard APB frequency when using PLL
const APB_FREQ_PLL: Hertz = Hertz(80_000_000);

// default 8M frequency
const RTC_FREQ_8M_DEFAULT: Hertz = Hertz(8_500_000); //With the default value of CK8M_DFREQ, 8M clock frequency is 8.5 MHz +/- 7%

// default slow rtc frequencies
const RTC_FREQ_150K_DEFAULT: Hertz = Hertz(150_000);

// Delays for various clock sources to be enabled/switched.
// All values are in microseconds.
//
// TODO according to esp-idf: some of these are excessive, and should be reduced.
const DELAY_FAST_CLK_SWITCH: MicroSeconds = MicroSeconds(3);
const DELAY_SLOW_CLK_SWITCH: MicroSeconds = MicroSeconds(300);
const DELAY_8M_ENABLE: MicroSeconds = MicroSeconds(50);
const DELAY_DBIAS_RAISE: MicroSeconds = MicroSeconds(3);

// number of wait cycles when enabling 8MHz clock
const CK8M_WAIT_DEFAULT: u8 = 20;

// Bias voltages for various clock speeds
const DIG_DBIAS_240M_OR_FLASH_80M: DBIAS_WAK_A = DBIAS_WAK_A::BIAS_1V25;
const DIG_DBIAS_80M_160M: DBIAS_WAK_A = DBIAS_WAK_A::BIAS_1V10;
const DIG_DBIAS_XTAL: DBIAS_WAK_A = DBIAS_WAK_A::BIAS_1V10;
const DIG_DBIAS_2M: DBIAS_WAK_A = DBIAS_WAK_A::BIAS_1V00;

// Default for clock tuning
const SCK_DCAP_DEFAULT: u8 = 255;
const CK8M_DFREQ_DEFAULT: u8 = 172;

// Number of slow cycles to measure for Xtal frequency measurement
const CYCLES_XTAL_CALIBRATION: u16 = 10;

// The minimum APB frequency to guarantee proper ref clock (10MHz according to documentation)
const MINIMUM_APB_FREQ_FOR_STABLE_REF_CLK: Hertz = Hertz(10_000_000);
// The accuracy of the clock to guarantee proper ref clock (as denominator for fraction,
// so 1/100 = 1%, 1/50 = 2%)
const MINIMUM_REF_CLOCK_ACCURACY_FOR_STABLE_REF_CLOCK: u32 = 100;

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
    CalibrationTimeOut,
    CalibrationSetupError,
    InvalidRegisterValue,
    InvalidCore,
    CoreAlreadyRunning,
}

/// CPU/APB/REF clock source
#[derive(Debug, Copy, Clone, PartialEq)]
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

/// Slow RTC clock source
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
enum CalibrateRTCSource {
    /// Slow RTC Source
    SlowRTC,
    /// Low frequency Xtal (32kHz)
    Xtal32k,
    /// 8MHz internal oscillator (divided by 256)
    RTC8MD256,
}

// static ClockControl to allow DFS, etc.
static mut CLOCK_CONTROL: Option<ClockControl> = None;
// mutex to allow safe multi-threaded access
static CLOCK_CONTROL_MUTEX: CriticalSectionSpinLockMutex<()> =
    CriticalSectionSpinLockMutex::new(());

/// Clock configuration & locking for Dynamic Frequency Switching.
/// It allows thread and interrupt safe way to switch between default,
/// high CPU and APB frequency configuration.
#[derive(Copy, Clone)]
pub struct ClockControlConfig {}

/// Clock Control for initialization. Once initialization is done, call the freeze function to lock
/// the clock configuration. This will return a [ClockControlConfig](ClockControlConfig), which can
/// be copied for e.g. use in multiple peripherals.
///
pub struct ClockControl {
    rtc_control: RTCCNTL,
    apb_control: APB_CTRL,
    dport_control: crate::dport::ClockControl,

    cpu_frequency_default: Hertz,
    cpu_source_default: CPUSource,
    cpu_frequency_locked: Hertz,
    cpu_source_locked: CPUSource,
    cpu_frequency_apb_locked: Hertz,
    cpu_source_apb_locked: CPUSource,
    #[allow(dead_code)]
    light_sleep_enabled: bool,

    apb_frequency_apb_locked: Hertz,

    rtc8m_frequency_measured: Hertz,
    rtc8md256_frequency_measured: Hertz,
    rtc_frequency_measured: Hertz,

    cpu_frequency: Hertz,
    apb_frequency: Hertz,
    ref_frequency: Hertz,
    apll_frequency: Hertz,
    pll_d2_frequency: Hertz,
    slow_rtc_frequency: Hertz,
    fast_rtc_frequency: Hertz,

    xtal_frequency: Hertz,
    xtal32k_frequency: Hertz,
    rtc8m_frequency: Hertz,
    rtc8md256_frequency: Hertz,

    rtc_frequency: Hertz,
    pll_frequency: Hertz,

    cpu_source: CPUSource,
    slow_rtc_source: SlowRTCSource,
    fast_rtc_source: FastRTCSource,

    ref_clock_stable: bool,

    dfs: dfs::DFS,
}

/// Function only available once clock if frozen
pub fn sleep<T: Into<NanoSeconds>>(time: T) {
    unsafe { CLOCK_CONTROL.as_ref().unwrap().delay(time) };
}

impl ClockControl {
    /// Create new ClockControl structure
    pub fn new<T: Into<Hertz> + Copy>(
        rtc_control: RTCCNTL,
        apb_control: APB_CTRL,
        dport_control: crate::dport::ClockControl,
        xtal_frequency: T,
    ) -> Result<Self, Error> {
        let mut cc = ClockControl {
            rtc_control,
            apb_control,
            dport_control,

            cpu_frequency_default: CPU_FREQ_MIN_DEFAULT,
            cpu_source_default: CPU_SOURCE_DEFAULT_DEFAULT,
            cpu_frequency_locked: CPU_FREQ_MAX_DEFAULT,
            cpu_source_locked: CPU_SOURCE_LOCKED_DEFAULT,
            cpu_frequency_apb_locked: CPU_FREQ_APB_DEFAULT,
            cpu_source_apb_locked: CPU_SOURCE_APB_LOCKED_DEFAULT,
            light_sleep_enabled: false,

            apb_frequency_apb_locked: APB_FREQ_PLL,

            rtc8m_frequency_measured: FREQ_OFF,
            rtc8md256_frequency_measured: FREQ_OFF,
            rtc_frequency_measured: FREQ_OFF,

            cpu_frequency: FREQ_OFF,
            apb_frequency: FREQ_OFF,
            ref_frequency: FREQ_OFF,
            apll_frequency: FREQ_OFF,
            pll_d2_frequency: FREQ_OFF,
            slow_rtc_frequency: FREQ_OFF,
            fast_rtc_frequency: FREQ_OFF,

            xtal_frequency: FREQ_OFF,
            xtal32k_frequency: FREQ_OFF,
            rtc8m_frequency: FREQ_OFF,
            rtc8md256_frequency: FREQ_OFF,

            rtc_frequency: FREQ_OFF,
            pll_frequency: FREQ_OFF,

            cpu_source: CPUSource::Xtal,
            slow_rtc_source: SlowRTCSource::RTC150k,
            fast_rtc_source: FastRTCSource::XtalD4,

            ref_clock_stable: true,

            dfs: dfs::DFS::new(),
        };
        cc.init(xtal_frequency)?;
        Ok(cc)
    }

    /// Freeze clock settings and return ClockControlConfig
    pub fn freeze(self) -> Result<(ClockControlConfig, watchdog::Watchdog), Error> {
        // can only occur one time as ClockControl is moved by this function and
        // the RTCCNTL and APBCTRL peripherals are moved when ClockControl is created
        unsafe { CLOCK_CONTROL = Some(self) };

        let res = ClockControlConfig {};
        Ok((res, watchdog::Watchdog::new(res)))
    }

    // TODO: check what dig_clk8m and dig_clk8m_256 are used for

    /// Check if 8MHz oscillator is enabled
    pub fn is_rtc8m_enabled(&self) -> bool {
        let value = self.rtc_control.clk_conf.read();
        value.ck8m_force_pu().bit_is_set()
            && value.ck8m_force_pd().bit_is_clear()
            && value.enb_ck8m().bit_is_clear()
            && value.dig_clk8m_en().bit_is_set()
    }

    /// Check if 8MHz oscillator is enabled
    pub fn is_rtc8md256_enabled(&self) -> bool {
        let value = self.rtc_control.clk_conf.read();
        self.is_rtc8m_enabled()
            && value.enb_ck8m_div().bit_is_clear()
            && value.dig_clk8m_d256_en().bit_is_set()
    }

    /// Enable 8MHz oscillator
    fn rtc8m_enable(&mut self) -> &mut Self {
        self.rtc_control.clk_conf.modify(|_, w| {
            w.ck8m_force_pu()
                .set_bit()
                .ck8m_force_pd()
                .clear_bit()
                .enb_ck8m()
                .clear_bit()
                .dig_clk8m_en()
                .set_bit()
        });

        // no need to wait for auto enable if enabled by software
        unsafe { self.rtc_control.timer1.modify(|_, w| w.ck8m_wait().bits(1)) };

        self.delay(DELAY_8M_ENABLE);

        self.rtc8m_frequency = self.rtc8m_frequency_measured;

        self
    }

    /// Enable 8MHz/256 (and therefore also 8MHz)
    fn rtc8md256_enable(&mut self) -> &mut Self {
        if !self.is_rtc8m_enabled() {
            self.rtc8m_enable();
        }

        self.rtc_control
            .clk_conf
            .modify(|_, w| w.enb_ck8m_div().clear_bit().dig_clk8m_d256_en().set_bit());

        self.rtc8md256_frequency = self.rtc8md256_frequency_measured;

        self
    }

    /// Disable 8MHz/256
    #[allow(dead_code)]
    fn rtc8md256_disable(&mut self) -> &mut Self {
        self.rtc_control
            .clk_conf
            .modify(|_, w| w.enb_ck8m_div().set_bit().dig_clk8m_d256_en().clear_bit());

        self.rtc8md256_frequency = FREQ_OFF;

        self
    }

    /// Disable 8MHz oscillator (and therefore also 8MHz/256)
    #[allow(dead_code)]
    fn rtc8m_disable(&mut self) -> &mut Self {
        self.rtc_control.clk_conf.modify(|_, w| {
            w.ck8m_force_pu()
                .clear_bit()
                .ck8m_force_pd()
                .set_bit()
                .enb_ck8m()
                .set_bit()
                .enb_ck8m_div()
                .set_bit()
                .dig_clk8m_en()
                .clear_bit()
                .dig_clk8m_d256_en()
                .clear_bit()
        });

        // need to wait for auto enable when disabled
        unsafe {
            self.rtc_control
                .timer1
                .modify(|_, w| w.ck8m_wait().bits(CK8M_WAIT_DEFAULT));
        }

        self.rtc8md256_frequency = FREQ_OFF;
        self.rtc8m_frequency = FREQ_OFF;

        self
    }

    /// Function to calibrate clocks against each other.
    /// Returns the number of XTAL clock cycles within the number of slow clock cycles.
    /// Clock must already be enabled on entry to this routine
    fn measure_clock_ticks(
        &mut self,
        source: CalibrateRTCSource,
        slow_cycles: u16,
    ) -> Result<u32, Error> {
        if slow_cycles > 32767 {
            return Err(Error::CalibrationSetupError);
        }

        // enable proper clock: should be done in advance

        let slow_freq = match source {
            CalibrateRTCSource::SlowRTC => RTC_FREQ_150K_DEFAULT,
            CalibrateRTCSource::RTC8MD256 => RTC_FREQ_8M_DEFAULT / 256,
            CalibrateRTCSource::Xtal32k => XTAL32K_FREQUENCY,
        };

        let estimated_time = (Hertz(1_000_000) * (slow_cycles as u32) / slow_freq).us();
        let estimated_cycle_count = 2 * self.time_to_cpu_cycles(estimated_time);

        let max_cycle_count = 0x01FFFFFF; // bit 7:31 = 25 bits
        if estimated_cycle_count > max_cycle_count {
            return Err(Error::CalibrationSetupError);
        }

        let rtc_source = match source {
            CalibrateRTCSource::SlowRTC => target::timg::rtccalicfg::CLK_SEL_A::RTC_MUX,
            CalibrateRTCSource::RTC8MD256 => target::timg::rtccalicfg::CLK_SEL_A::CK8M_D256,
            CalibrateRTCSource::Xtal32k => target::timg::rtccalicfg::CLK_SEL_A::XTAL32K,
        };

        // get timer group 0 registers, do it this way instead of
        // having to pass in yet another peripheral for this clock control
        let timg0 = unsafe { &(*TIMG0::ptr()) };

        // setup measurement
        unsafe {
            timg0.rtccalicfg.modify(|_, w| {
                w.clk_sel()
                    .variant(rtc_source)
                    .max()
                    .bits(slow_cycles)
                    .start_cycling()
                    .clear_bit()
                    .start()
                    .clear_bit()
                    .rdy()
                    .clear_bit()
            })
        };

        // start measurement
        timg0.rtccalicfg.modify(|_, w| w.start().set_bit());

        // check if finished or timeout
        let start = get_cycle_count();
        while timg0.rtccalicfg.read().rdy().bit_is_clear() {
            if get_cycle_count().wrapping_sub(start) > estimated_cycle_count {
                return Err(Error::CalibrationTimeOut);
            }
            self.delay(1.us()); // prevent flooding of RTC bus
        }

        Ok(timg0.rtccalicfg1.read().value().bits())
    }

    /// Measure an estimated Xtal frequency based on the 8MHz oscillator
    fn detect_xtal_frequency(&mut self) -> Result<(), Error> {
        let ticks =
            self.measure_clock_ticks(CalibrateRTCSource::RTC8MD256, CYCLES_XTAL_CALIBRATION)?;

        let xtal_frequency_measure =
            RTC_FREQ_8M_DEFAULT / 256 * ticks / (CYCLES_XTAL_CALIBRATION as u32);

        if xtal_frequency_measure > XTAL_FREQUENCY_THRESHOLD {
            return Err(Error::FrequencyTooHigh);
        } else if xtal_frequency_measure > XTAL_FREQUENCY_40M_THRESHOLD {
            self.xtal_frequency = XTAL_FREQUENCY_40M;
        } else if xtal_frequency_measure > XTAL_FREQUENCY_26M_THRESHOLD {
            self.xtal_frequency = XTAL_FREQUENCY_26M;
        } else if xtal_frequency_measure > XTAL_FREQUENCY_24M_THRESHOLD {
            self.xtal_frequency = XTAL_FREQUENCY_24M;
        } else {
            return Err(Error::FrequencyTooLow);
        }

        self.cpu_frequency = self.xtal_frequency;
        self.apb_frequency = self.xtal_frequency;

        self.set_xtal_frequency_to_scratch(self.xtal_frequency);

        Ok(())
    }

    /// Measure the frequency of one of the clock oscillators based on the Xtal frequency
    fn measure_slow_frequency(&mut self, source: CalibrateRTCSource) -> Result<Hertz, Error> {
        let ticks = self.measure_clock_ticks(source, CYCLES_XTAL_CALIBRATION)?;

        Ok(self.xtal_frequency * (CYCLES_XTAL_CALIBRATION as u32) / ticks)
    }

    /// Initialize clock configuration
    fn init<T: Into<Hertz> + Copy>(&mut self, xtal_frequency: T) -> Result<&mut Self, Error> {
        // if auto is selected check if the frequency has already been stored during
        // a previous run in the scratch register
        if xtal_frequency.into() == XTAL_FREQUENCY_AUTO {
            self.xtal_frequency = match self.xtal_frequency_from_scratch() {
                Ok(frequency) => frequency,
                _ => DEFAULT_XTAL_FREQUENCY,
            };
        } else {
            self.xtal_frequency = xtal_frequency.into();
        }

        // switch from pll to xtal (pll can still be enabled when previously in deep sleep)
        // xtal_frequency might be incorrect here, but by setting teh cpu to current xtal frequency
        // divider will be initialized to 1
        if !self.rtc_control.clk_conf.read().soc_clk_sel().is_xtal() {
            self.set_cpu_frequency_to_xtal(self.xtal_frequency).unwrap();
        } else {
            self.cpu_frequency = self.xtal_frequency;
        }

        // set default calibration for 150kHz and 8MHz oscillators
        unsafe {
            self.rtc_control
                .cntl
                .modify(|_, w| w.sck_dcap().bits(SCK_DCAP_DEFAULT));
            self.rtc_control
                .clk_conf
                .modify(|_, w| w.ck8m_dfreq().bits(CK8M_DFREQ_DEFAULT));
        }

        // TODO: Enable the internal bus used to configure PLL's: seems to be done by default,
        // but maybe not during deep sleep?
        // SET_PERI_REG_BITS(ANA_CONFIG_REG, ANA_CONFIG_M, ANA_CONFIG_M, ANA_CONFIG_S);
        // CLEAR_PERI_REG_MASK(ANA_CONFIG_REG, I2C_APLL_M | I2C_BBPLL_M);

        self.rtc8md256_enable();

        if xtal_frequency.into() == XTAL_FREQUENCY_AUTO {
            self.detect_xtal_frequency()?;
        }

        self.rtc8md256_frequency_measured =
            self.measure_slow_frequency(CalibrateRTCSource::RTC8MD256)?;
        self.rtc8md256_frequency = self.rtc8md256_frequency_measured;
        self.rtc8m_frequency_measured = self.rtc8md256_frequency_measured * 256;
        self.rtc8m_frequency = self.rtc8m_frequency_measured;

        self.set_slow_rtc_source(SlowRTCSource::RTC150k);
        self.rtc_frequency_measured = self.measure_slow_frequency(CalibrateRTCSource::SlowRTC)?;
        self.rtc_frequency = self.rtc_frequency_measured;

        self.set_slow_rtc_source(SlowRTCSource::RTC8MD256);
        self.set_fast_rtc_source(FastRTCSource::RTC8M);

        self.set_cpu_frequency_default(false)?;

        Ok(self)
    }

    /// calculate the number of cpu cycles from a time at the current CPU frequency
    fn time_to_cpu_cycles<T: Into<NanoSeconds>>(&self, time: T) -> u32 {
        (((self.cpu_frequency / Hertz(1_000_000)) as u64) * (u32::from(time.into()) as u64) / 1000)
            as u32
    }

    /// delay a certain time by spinning
    fn delay<T: Into<NanoSeconds>>(&self, time: T) {
        delay(self.time_to_cpu_cycles(time));
    }

    /// Check if a value from RTC_XTAL_FREQ_REG or RTC_APB_FREQ_REG are valid clocks
    fn clk_val_is_valid(val: u32) -> bool {
        (val & 0xffff) == ((val >> 16) & 0xffff) && val != 0 && val != u32::max_value()
    }

    /// Set CPU default, locked and apb frequencies for Dynamic Frequency Switching.
    ///
    /// The default source and frequency are used when no locks are acquired. (Typically
    /// this would be) the lowest frequency.)
    ///
    /// The cpu_locked source & frequency are used when the cpu frequency is locked, unless an
    /// apb lock is acquired and the apb_locked frequency is higher then the cpu_locked frequency.
    ///
    /// The apb_locked source & frequency is used when peripherals request a locked apb frequency.
    ///
    /// This function switches to the default source & frequency (locks can not have been
    /// acquired yet as this can only be done from ClockControlConfig).
    pub fn set_cpu_frequencies<T1, T2, T3>(
        &mut self,
        cpu_source_default: CPUSource,
        cpu_frequency_default: T1,
        cpu_source_locked: CPUSource,
        cpu_frequency_locked: T2,
        cpu_source_apb_locked: CPUSource,
        cpu_frequency_apb_locked: T3,
    ) -> Result<&mut Self, Error>
    where
        T1: Into<Hertz> + Copy + PartialOrd,
        T2: Into<Hertz> + Copy + PartialOrd,
        T3: Into<Hertz> + Copy + PartialOrd,
    {
        match cpu_source_default {
            CPUSource::APLL => return Err(Error::UnsupportedFreqConfig),
            _ => {}
        }
        match cpu_source_locked {
            CPUSource::APLL => return Err(Error::UnsupportedFreqConfig),
            _ => {}
        }
        match cpu_source_apb_locked {
            CPUSource::APLL => return Err(Error::UnsupportedFreqConfig),
            _ => {}
        }

        if cpu_frequency_default.into() < CPU_FREQ_MIN
            || cpu_frequency_locked.into() < CPU_FREQ_MIN
            || cpu_frequency_apb_locked.into() < CPU_FREQ_MIN
        {
            return Err(Error::FrequencyTooLow);
        }

        if cpu_frequency_default.into() > CPU_FREQ_240M
            || cpu_frequency_locked.into() > CPU_FREQ_240M
            || cpu_frequency_apb_locked.into() > CPU_FREQ_240M
        {
            return Err(Error::FrequencyTooHigh);
        }

        self.cpu_source_default = cpu_source_default;
        self.cpu_frequency_default =
            self.round_cpu_frequency(cpu_source_default, cpu_frequency_default);
        self.cpu_source_locked = cpu_source_locked;
        self.cpu_frequency_locked =
            self.round_cpu_frequency(cpu_source_locked, cpu_frequency_locked);
        self.cpu_source_apb_locked = cpu_source_apb_locked;
        self.cpu_frequency_apb_locked =
            self.round_cpu_frequency(cpu_source_apb_locked, cpu_frequency_apb_locked);

        self.apb_frequency_apb_locked = match cpu_source_apb_locked {
            CPUSource::PLL => APB_FREQ_PLL,
            _ => self.cpu_frequency_apb_locked,
        };

        self.ref_clock_stable = self
            .check_ref_clock_stable(self.cpu_source_default, self.cpu_frequency_default)
            && self.check_ref_clock_stable(self.cpu_source_locked, self.cpu_frequency_locked)
            && self
                .check_ref_clock_stable(self.cpu_source_apb_locked, self.cpu_frequency_apb_locked);

        self.set_cpu_frequency_default(false)?;
        Ok(self)
    }

    fn check_ref_clock_stable<T: Into<Hertz>>(&self, source: CPUSource, frequency: T) -> bool {
        let f_hz = frequency.into();
        match source {
            CPUSource::PLL => true,
            CPUSource::Xtal => f_hz >= MINIMUM_APB_FREQ_FOR_STABLE_REF_CLK,
            CPUSource::RTC8M => {
                let round = MINIMUM_APB_FREQ_FOR_STABLE_REF_CLK
                    / MINIMUM_REF_CLOCK_ACCURACY_FOR_STABLE_REF_CLOCK;
                let f_rounded = (f_hz + round / 2) / round * round;

                f_rounded >= MINIMUM_APB_FREQ_FOR_STABLE_REF_CLK
                    && f_rounded == (f_hz + REF_CLK_FREQ_1M / 2) / REF_CLK_FREQ_1M * REF_CLK_FREQ_1M
            }
            CPUSource::APLL => unimplemented!(),
        }
    }

    /// Calculate the nearest actually realizable frequency
    fn round_cpu_frequency<T: Into<Hertz>>(&self, source: CPUSource, frequency: T) -> Hertz {
        let f_hz = frequency.into();
        match source {
            CPUSource::PLL => {
                if f_hz <= CPU_FREQ_80M {
                    CPU_FREQ_80M
                } else if f_hz <= CPU_FREQ_160M {
                    CPU_FREQ_160M
                } else {
                    CPU_FREQ_240M
                }
            }
            CPUSource::Xtal => {
                let div = core::cmp::min(
                    u16::max_value() as u32,
                    core::cmp::max(1, self.xtal_frequency / f_hz),
                );

                self.xtal_frequency / div
            }
            CPUSource::RTC8M => {
                let div = core::cmp::min(
                    u16::max_value() as u32,
                    core::cmp::max(1, self.rtc8m_frequency_measured / f_hz),
                );

                self.rtc8m_frequency_measured / div
            }
            _ => unimplemented!(),
        }
    }

    /// Set CPU to minimum frequency
    fn set_cpu_frequency_default(&mut self, keep_pll_enabled: bool) -> Result<&mut Self, Error> {
        self.set_cpu_frequency(
            self.cpu_source_default,
            self.cpu_frequency_default,
            keep_pll_enabled,
        )
    }

    /// Set CPU to maximum frequency
    fn set_cpu_frequency_locked(&mut self, keep_pll_enabled: bool) -> Result<&mut Self, Error> {
        self.set_cpu_frequency(
            self.cpu_source_locked,
            self.cpu_frequency_locked,
            keep_pll_enabled,
        )
    }

    /// Set CPU to apb frequency
    fn set_cpu_frequency_apb_locked(&mut self, keep_pll_enabled: bool) -> Result<&mut Self, Error> {
        self.set_cpu_frequency(
            self.cpu_source_apb_locked,
            self.cpu_frequency_apb_locked,
            keep_pll_enabled,
        )
    }

    /// Set CPU source and frequency
    fn set_cpu_frequency<T: Into<Hertz> + Copy + PartialOrd + core::fmt::Debug>(
        &mut self,
        source: CPUSource,
        frequency: T,
        keep_pll_enabled: bool,
    ) -> Result<&mut Self, Error> {
        match source {
            CPUSource::Xtal => self.set_cpu_frequency_to_xtal(frequency)?,
            CPUSource::PLL => self.set_cpu_frequency_to_pll(frequency)?,
            CPUSource::RTC8M => self.set_cpu_frequency_to_8m(frequency)?,
            CPUSource::APLL => return Err(Error::UnsupportedFreqConfig),
        };

        if source != CPUSource::PLL && !keep_pll_enabled {
            self.pll_disable();
        }

        Ok(self)
    }

    /// Sets the CPU frequency using the 8MHz internal oscillator to closest possible frequency (rounding up).
    ///
    /// The APB frequency follows the CPU frequency.
    /// The ref clock is not guaranteed to be at 1MHz
    fn set_cpu_frequency_to_8m<T: Into<Hertz> + Copy + PartialOrd>(
        &mut self,
        frequency: T,
    ) -> Result<(), Error> {
        let mut f_hz: Hertz = frequency.into();

        if f_hz < 1.kHz().into() {
            return Err(Error::FrequencyTooLow);
        }

        if f_hz > self.rtc8m_frequency_measured {
            f_hz = self.rtc8m_frequency_measured;
        }

        // calculate divider, only integer fractions of 8MHz are possible
        let div = core::cmp::max(1, self.rtc8m_frequency_measured / f_hz);

        if div > u16::max_value() as u32 {
            return Err(Error::FrequencyTooLow);
        }

        self.cpu_frequency = self.rtc8m_frequency_measured / (div as u32);

        let div_1m = (self.cpu_frequency + REF_CLK_FREQ_1M / 2) / REF_CLK_FREQ_1M;

        self.ref_frequency = self.cpu_frequency / div_1m;

        // select appropriate voltage
        if self.cpu_frequency > CPU_FREQ_2M {
            self.rtc_control
                .cntl
                .modify(|_, w| w.dig_dbias_wak().variant(DIG_DBIAS_XTAL))
        };

        // TODO: check if this is all needed
        self.rtc_control.clk_conf.modify(|_, w| {
            w.ck8m_force_pu()
                .set_bit()
                .ck8m_force_pd()
                .clear_bit()
                .enb_ck8m()
                .clear_bit()
                .enb_ck8m_div()
                .clear_bit()
                .dig_clk8m_en()
                .set_bit()
                .dig_clk8m_d256_en()
                .set_bit()
        });

        // set divider from 8MHz to CPU clock
        self.apb_control
            .sysclk_conf
            .modify(|_, w| unsafe { w.pre_div_cnt().bits(div as u16 - 1) });

        // adjust ref tick
        self.apb_control
            .ck8m_tick_conf
            .write(|w| unsafe { w.ck8m_tick_num().bits(div_1m as u8 - 1) });

        // switch clock source
        self.rtc_control
            .clk_conf
            .modify(|_, w| w.soc_clk_sel().ck8m());

        // select appropriate voltage
        if self.cpu_frequency <= CPU_FREQ_2M {
            self.rtc_control
                .cntl
                .modify(|_, w| w.dig_dbias_wak().variant(DIG_DBIAS_2M))
        };

        self.cpu_source = CPUSource::RTC8M;
        self.apb_frequency = self.cpu_frequency;
        self.set_apb_frequency_to_scratch(self.apb_frequency);

        self.wait_for_slow_cycle();

        Ok(())
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
    ) -> Result<(), Error> {
        let mut f_hz: Hertz = frequency.into();

        if f_hz < 1.kHz().into() {
            return Err(Error::FrequencyTooLow);
        }

        if f_hz > self.xtal_frequency {
            f_hz = self.xtal_frequency;
        }

        // calculate divider, only integer fractions of xtal_frequency are possible
        let div = core::cmp::max(1, self.xtal_frequency / f_hz);

        if div > u16::max_value() as u32 {
            return Err(Error::FrequencyTooLow);
        }

        self.cpu_frequency = self.xtal_frequency / (div as u32);

        let div_1m = (self.cpu_frequency + REF_CLK_FREQ_1M / 2) / REF_CLK_FREQ_1M;

        self.ref_frequency = self.cpu_frequency / div_1m;

        // select appropriate voltage
        if self.cpu_frequency > CPU_FREQ_2M {
            self.rtc_control
                .cntl
                .modify(|_, w| w.dig_dbias_wak().variant(DIG_DBIAS_XTAL))
        };

        // set divider from Xtal to CPU clock
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
        if self.cpu_frequency <= CPU_FREQ_2M {
            self.rtc_control
                .cntl
                .modify(|_, w| w.dig_dbias_wak().variant(DIG_DBIAS_2M))
        };

        self.cpu_source = CPUSource::Xtal;
        self.apb_frequency = self.cpu_frequency;
        self.set_apb_frequency_to_scratch(self.apb_frequency);

        self.wait_for_slow_cycle();

        Ok(())
    }

    /// Sets the CPU frequency using the PLL to closest possible frequency (rounding up).
    ///
    /// The APB frequency is fixed at 80MHz.
    fn set_cpu_frequency_to_pll<T>(&mut self, frequency: T) -> Result<(), Error>
    where
        T: Into<Hertz> + Copy + PartialOrd,
    {
        // TODO: adjust bias if flash at 80MHz
        let (cpu_freq, pll_frequency_high, cpuperiod_sel, dbias) = match frequency.into() {
            f if f <= CPU_FREQ_80M => (
                CPU_FREQ_80M,
                false,
                CPUPERIOD_SEL_A::SEL_80,
                DIG_DBIAS_80M_160M,
            ),
            f if f <= CPU_FREQ_160M => (
                CPU_FREQ_160M,
                false,
                CPUPERIOD_SEL_A::SEL_160,
                DIG_DBIAS_80M_160M,
            ),
            _ => (
                CPU_FREQ_240M,
                true,
                CPUPERIOD_SEL_A::SEL_240,
                DIG_DBIAS_240M_OR_FLASH_80M,
            ),
        };

        // when pll frequency needs to be switched, temporarily go to Xtal to avoid lock-ups
        // TODO only needed on rev0? (ESP32 Errata 3.5)
        if (self.cpu_source == CPUSource::PLL)
            && pll_frequency_high != (self.pll_frequency == PLL_FREQ_480M)
        {
            self.set_cpu_frequency_to_xtal(self.xtal_frequency)?;
        }

        // pll frequency changes
        if self.pll_frequency == FREQ_OFF
            || pll_frequency_high != (self.pll_frequency == PLL_FREQ_480M)
        {
            // if going to high frequency raise voltage first and select large divider
            if pll_frequency_high {
                self.rtc_control
                    .cntl
                    .modify(|_, w| w.dig_dbias_wak().variant(dbias));

                self.delay(DELAY_DBIAS_RAISE);
                self.dport_control
                    .cpu_per_conf()
                    .modify(|_, w| w.cpuperiod_sel().variant(cpuperiod_sel));
            }

            // adjust ref tick
            self.apb_control
                .pll_tick_conf
                .write(|w| unsafe { w.pll_tick_num().bits(80 - 1) });

            // unwrap because switch leaves things in undefined state
            if self.pll_frequency == FREQ_OFF {
                self.pll_enable(pll_frequency_high).unwrap();
            } else {
                self.set_pll_frequency(pll_frequency_high).unwrap();
            }
            self.wait_for_slow_cycle();

            // if going to low frequency lower voltage last and select smaller divider
            if !pll_frequency_high {
                self.rtc_control
                    .cntl
                    .modify(|_, w| w.dig_dbias_wak().variant(dbias));

                self.dport_control
                    .cpu_per_conf()
                    .modify(|_, w| w.cpuperiod_sel().variant(cpuperiod_sel));
            }
        }

        // switch clock source
        self.rtc_control
            .clk_conf
            .modify(|_, w| w.soc_clk_sel().pll());

        self.cpu_source = CPUSource::PLL;
        self.cpu_frequency = cpu_freq;
        self.ref_frequency = REF_CLK_FREQ_1M;
        self.apb_frequency = APB_FREQ_PLL;
        self.set_apb_frequency_to_scratch(self.apb_frequency());

        self.wait_for_slow_cycle();

        Ok(())
    }

    /// wait for slow clock cycle to synchronize
    fn wait_for_slow_cycle(&mut self) {
        // get timer group 0 registers, do it this way instead of
        // having to pass in yet another peripheral for this clock control
        let timg0 = unsafe { &(*TIMG0::ptr()) };

        // setup measurement
        unsafe {
            timg0.rtccalicfg.modify(|_, w| {
                w.clk_sel()
                    .variant(target::timg::rtccalicfg::CLK_SEL_A::RTC_MUX)
                    .max()
                    .bits(0)
                    .start_cycling()
                    .clear_bit()
                    .start()
                    .clear_bit()
                    .rdy()
                    .clear_bit()
            })
        };

        // start measurement
        timg0.rtccalicfg.modify(|_, w| w.start().set_bit());

        // wait for nearest slow clock cycle
        while timg0.rtccalicfg.read().rdy().bit_is_clear() {
            self.delay(1.us()); // prevent flooding of RTC bus
        }
    }

    /// Get Ref Tick frequency
    ///
    /// This frequency is usually 1MHz, but cannot be maintained when the APB_CLK is < 10MHz
    pub fn ref_frequency(&self) -> Hertz {
        match self.cpu_source() {
            CPUSource::PLL => REF_CLK_FREQ_1M,
            CPUSource::Xtal => {
                let div = self
                    .apb_control
                    .xtal_tick_conf
                    .read()
                    .xtal_tick_num()
                    .bits();

                self.apb_frequency / (div + 1) as u32
            }
            CPUSource::APLL => unimplemented!(),
            CPUSource::RTC8M => {
                let div = self
                    .apb_control
                    .ck8m_tick_conf
                    .read()
                    .ck8m_tick_num()
                    .bits();

                self.rtc8m_frequency_measured / (div + 1) as u32
            }
        }
    }

    /// Get APB frequency
    ///
    /// This gets the APB frequency from the scratch register, which is initialized during the clock calibration
    pub fn apb_frequency(&self) -> Hertz {
        match self.cpu_source() {
            CPUSource::PLL => APB_FREQ_PLL,
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

    /// Get Slow RTC frequency
    pub fn slow_rtc_frequency(&self) -> Hertz {
        match self.slow_rtc_source() {
            Ok(SlowRTCSource::RTC150k) => self.rtc_frequency_measured,
            Ok(SlowRTCSource::Xtal32k) => XTAL32K_FREQUENCY,
            Ok(SlowRTCSource::RTC8MD256) => self.rtc8md256_frequency_measured,
            _ => FREQ_OFF,
        }
    }

    /// Get Slow RTC source
    pub fn slow_rtc_source(&self) -> Result<SlowRTCSource, Error> {
        match self.rtc_control.clk_conf.read().ana_clk_rtc_sel().variant() {
            Val(ANA_CLK_RTC_SEL_A::SLOW_CK) => Ok(SlowRTCSource::RTC150k),
            Val(ANA_CLK_RTC_SEL_A::CK_XTAL_32K) => Ok(SlowRTCSource::Xtal32k),
            Val(ANA_CLK_RTC_SEL_A::CK8M_D256_OUT) => Ok(SlowRTCSource::RTC8MD256),
            _ => Err(Error::UnsupportedFreqConfig),
        }
    }

    /// Set the Slow RTC clock source
    pub fn set_slow_rtc_source(&mut self, source: SlowRTCSource) -> &mut Self {
        match source {
            SlowRTCSource::RTC150k => {
                self.rtc_control
                    .clk_conf
                    .modify(|_, w| w.ana_clk_rtc_sel().slow_ck());

                self.slow_rtc_frequency = self.rtc_frequency_measured;
            }
            SlowRTCSource::Xtal32k => {
                self.rtc_control
                    .clk_conf
                    .modify(|_, w| w.ana_clk_rtc_sel().ck_xtal_32k());
                self.slow_rtc_frequency = XTAL32K_FREQUENCY;
            }
            SlowRTCSource::RTC8MD256 => {
                self.rtc_control
                    .clk_conf
                    .modify(|_, w| w.ana_clk_rtc_sel().ck8m_d256_out());
                self.slow_rtc_frequency = self.rtc8md256_frequency_measured;
            }
        }
        self.delay(DELAY_SLOW_CLK_SWITCH);
        self.slow_rtc_source = source;
        self
    }

    /// Get Fast RTC frequency
    pub fn fast_rtc_frequency(&self) -> Hertz {
        match self.fast_rtc_source() {
            FastRTCSource::RTC8M => self.rtc8m_frequency_measured,
            FastRTCSource::XtalD4 => self.xtal_frequency / 4,
        }
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

    /// Set the Fast RTC clock source
    pub fn set_fast_rtc_source(&mut self, source: FastRTCSource) -> &mut Self {
        match source {
            FastRTCSource::RTC8M => {
                self.rtc_control
                    .clk_conf
                    .modify(|_, w| w.fast_clk_rtc_sel().ck8m());
                self.fast_rtc_frequency = self.rtc8m_frequency_measured;
            }
            FastRTCSource::XtalD4 => {
                self.rtc_control
                    .clk_conf
                    .modify(|_, w| w.fast_clk_rtc_sel().xtal());
                self.fast_rtc_frequency = self.xtal_frequency / 4;
            }
        }
        self.delay(DELAY_FAST_CLK_SWITCH);
        self.fast_rtc_source = source;
        self
    }

    /// Get Xtal frequency.
    ///
    /// This gets the Xtal frequency from a scratch register, which is initialized during the clock calibration
    pub fn xtal_frequency_from_scratch(&self) -> Result<Hertz, Error> {
        // We may have already written Xtal value into RTC_XTAL_FREQ_REG
        let xtal_freq_reg = self.rtc_control.store4.read().scratch4().bits();
        if !Self::clk_val_is_valid(xtal_freq_reg) {
            return Err(Error::InvalidRegisterValue);
        }

        Ok((xtal_freq_reg & 0xfffe).MHz().into()) // bit0 is RTC_DISABLE_ROM_LOG flag
    }

    /// Set Xtal frequency.
    ///
    /// This sets the Xtal frequency to a scratch register, which is initialized during the clock calibration
    fn set_xtal_frequency_to_scratch<T: Into<Hertz>>(&mut self, frequency: T) -> &mut Self {
        // Write CPU value into RTC_XTAL_FREQ_REG for compatibility with esp-idf
        // bit 0 is RTC_ROM_DISABLE_ROM_LOG flag
        let mut val = ((frequency.into() / Hertz(1_000_000)) & 0xfffe)
            | (self.rtc_control.store4.read().scratch4().bits() & 0x1);

        val = val | (val << 16); // value needs to be copied in lower and upper 16 bits
        self.rtc_control
            .store4
            .write(|w| unsafe { w.scratch4().bits(val) });

        self
    }

    /// Set CPU frequency source
    fn cpu_source(&self) -> CPUSource {
        match self.rtc_control.clk_conf.read().soc_clk_sel().variant() {
            SOC_CLK_SEL_A::XTAL => CPUSource::Xtal,
            SOC_CLK_SEL_A::PLL => CPUSource::PLL,
            SOC_CLK_SEL_A::APLL => CPUSource::APLL,
            SOC_CLK_SEL_A::CK8M => CPUSource::RTC8M,
        }
    }

    /// Get CPU frequency
    fn cpu_frequency(&self) -> Hertz {
        match self.cpu_source() {
            CPUSource::Xtal => {
                let divider = self.apb_control.sysclk_conf.read().pre_div_cnt().bits() + 1;
                self.xtal_frequency / divider as u32
            }
            CPUSource::PLL => match self
                .dport_control
                .cpu_per_conf()
                .read()
                .cpuperiod_sel()
                .variant()
            {
                Val(CPUPERIOD_SEL_A::SEL_80) => CPU_FREQ_80M,
                Val(CPUPERIOD_SEL_A::SEL_160) => CPU_FREQ_160M,
                Val(CPUPERIOD_SEL_A::SEL_240) => CPU_FREQ_240M,
                _ => FREQ_OFF,
            },
            CPUSource::RTC8M => self.rtc8m_frequency_measured,
            CPUSource::APLL => unimplemented!(),
        }
    }

    /// Get RTC tick count since boot
    ///
    /// This function can usually take up to one RTC clock cycles (~300us).
    ///
    /// In exceptional circumstances it could take up to two RTC clock cycles. This can happen
    /// when an interrupt routine or the other core calls this function exactly in between
    /// the loop checking for the valid bit and entering the critical section.
    ///
    /// Interrupts are only blocked during the actual reading of the clock register,
    /// not during the wait for valid data.
    pub fn rtc_tick_count(&self) -> TicksU64 {
        self.rtc_control
            .time_update
            .modify(|_, w| w.time_update().set_bit());

        loop {
            // do this check outside the critical section, to prevent blocking interrupts and
            // the other core for a long time
            while self
                .rtc_control
                .time_update
                .read()
                .time_valid()
                .bit_is_clear()
            {}

            if let Some(ticks) = (&CLOCK_CONTROL_MUTEX).lock(|_| {
                // there is a small chance that this function is interrupted or called from
                // the other core between detecting the valid and entering the interrupt free
                // and mutex protection reading check again inside the critical section

                if self
                    .rtc_control
                    .time_update
                    .read()
                    .time_valid()
                    .bit_is_set()
                {
                    // this needs to be interrupt and thread safe, because if the time value
                    // changes in between reading the upper and lower part this results in an
                    // invalid value.
                    let hi = self.rtc_control.time1.read().time_hi().bits() as u64;
                    let lo = self.rtc_control.time0.read().bits() as u64;
                    let ticks: TicksU64 = TicksU64::from((hi << 32) | lo);
                    Some(ticks)
                } else {
                    None
                }
            }) {
                return ticks;
            }
        }
    }

    /// Get nanoseconds since boot based on RTC tick count
    pub fn rtc_nanoseconds(&self) -> NanoSecondsU64 {
        self.rtc_tick_count() / self.slow_rtc_frequency
    }
}

/// Custom debug formatter
impl fmt::Debug for ClockControl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClockControl")
            .field("cpu_frequency", &self.cpu_frequency)
            .field("apb_frequency", &self.apb_frequency)
            .field("ref_frequency", &self.ref_frequency)
            .field("apll_frequency", &self.apll_frequency)
            .field("pll_d2_frequency", &self.pll_d2_frequency)
            .field("slow_rtc_frequency", &self.slow_rtc_frequency)
            .field("fast_rtc_frequency", &self.fast_rtc_frequency)
            .field("xtal_frequency", &self.xtal_frequency)
            .field("xtal32k_frequency", &self.xtal32k_frequency)
            .field("rtc8m_frequency", &self.rtc8m_frequency)
            .field("rtc8md256_frequency", &self.rtc8md256_frequency)
            .field("rtc_frequency", &self.rtc_frequency)
            .field("pll_frequency", &self.pll_frequency)
            .field("cpu_source", &self.cpu_source)
            .field("slow_rtc_source", &self.slow_rtc_source)
            .field("fast_rtc_source", &self.fast_rtc_source)
            .finish()
    }
}
