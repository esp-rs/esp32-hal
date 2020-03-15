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

use crate::prelude::*;
use esp32::dport::cpu_per_conf::CPUPERIOD_SEL_A;
use esp32::generic::Variant::*;
use esp32::rtccntl::bias_conf::*;
use esp32::rtccntl::clk_conf::*;
use esp32::{APB_CTRL, RTCCNTL};

pub mod watchdog;

type CoreBias = DBIAS_WAK_A;

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
 * TODO: some of these are excessive, and should be reduced.
 */
#[allow(dead_code)]
const DELAY_PLL_DBIAS_RAISE: MicroSeconds = MicroSeconds(3);
const DELAY_PLL_ENABLE_WITH_150K: MicroSeconds = MicroSeconds(80);
const DELAY_PLL_ENABLE_WITH_32K: MicroSeconds = MicroSeconds(160);
const DELAY_FAST_CLK_SWITCH: MicroSeconds = MicroSeconds(3);
const DELAY_SLOW_CLK_SWITCH: MicroSeconds = MicroSeconds(300);
const DELAY_8M_ENABLE: MicroSeconds = MicroSeconds(50);

const PLL_ENDIV5_VAL_320M: u8 = 0x43;
const PLL_BBADC_DSMP_VAL_320M: u8 = 0x84;
const PLL_ENDIV5_VAL_480M: u8 = 0xc3;
const PLL_BBADC_DSMP_VAL_480M: u8 = 0x74;
const PLL_IR_CAL_DELAY_VAL: u8 = 0x18;
const PLL_IR_CAL_EXT_CAP_VAL: u8 = 0x20;
const PLL_OC_ENB_FCAL_VAL: u8 = 0x9a;
const PLL_OC_ENB_VCON_VAL: u8 = 0x00;
const PLL_BBADC_CAL_7_0_VAL: u8 = 0x00;

// div_ref, div7_0, div10_8, lref,dcur,bw
struct PLLConfig(u8, u8, u8, u8, u8, u8);

const PLL_CONFIG_320M_XTAL_40M: PLLConfig = PLLConfig(0, 32, 0, 0, 6, 3);
const PLL_CONFIG_320M_XTAL_26M: PLLConfig = PLLConfig(2, 224, 4, 1, 0, 1);
const PLL_CONFIG_320M_XTAL_24M: PLLConfig = PLLConfig(11, 224, 4, 1, 0, 1);
const PLL_CONFIG_320M_XTAL_UNKNOWN: PLLConfig = PLLConfig(12, 224, 4, 0, 0, 0);

const PLL_CONFIG_480M_XTAL_40M: PLLConfig = PLLConfig(0, 28, 0, 0, 6, 3);
const PLL_CONFIG_480M_XTAL_26M: PLLConfig = PLLConfig(12, 144, 4, 1, 0, 1);
const PLL_CONFIG_480M_XTAL_24M: PLLConfig = PLLConfig(11, 144, 4, 1, 0, 1);
const PLL_CONFIG_480M_XTAL_UNKNOWN: PLLConfig = PLLConfig(12, 224, 4, 0, 0, 0);

/// CPU/APB/REF clock source
#[derive(Debug)]
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
#[derive(Debug)]
pub enum SlowRTCSource {
    /// 150kHz internal oscillator
    RTC150k,
    /// Low frequency Xtal (32kHz)
    Xtal32k,
    /// 8MHz internal oscillator (divided by 256)
    RTC8MD256,
}

/// Fast RTC clock source
#[derive(Debug)]
pub enum FastRTCSource {
    /// 8MHz internal oscillator
    RTC8M,
    /// High frequency Xtal (26MHz or 40MHz)
    Xtal,
}

/// Clock configuration
#[derive(Debug)]
pub struct ClockControlConfig {
    /// CPU Frequency
    pub cpu_frequency: Hertz,
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
        };
        cc.init();
        cc
    }

    /// Initialize clock configuration
    fn init(&mut self) -> &mut Self {
        if self.rtc_control.clk_conf.read().soc_clk_sel().is_pll() {
            self.set_cpu_frequency(self.xtal_frequency());
        }

        self
    }

    /// Gets RTC watchdog control
    pub fn watchdog<'a>(&'a mut self) -> watchdog::WatchDog<'a> {
        watchdog::WatchDog::new(self)
    }

    pub fn time_to_cpu_cycles<T: Into<NanoSeconds>>(&mut self, time: T) -> u32 {
        (((self.cpu_frequency() / 1000000.Hz()) as u64) * ((time.into() / 1000.ns()) as u64)) as u32
    }

    pub fn delay<T: Into<NanoSeconds>>(&mut self, time: T) {
        delay_cycles(self.time_to_cpu_cycles(time));
    }

    /// Check if a value from RTC_XTAL_FREQ_REG or RTC_APB_FREQ_REG are valid clocks
    fn clk_val_is_valid(val: u32) -> bool {
        (val & 0xffff) == ((val >> 16) & 0xffff) && val != 0 && val != u32::max_value()
    }

    /// Sets the CPU frequency to closest possible frequency (rounding up).
    ///
    /// Up to the Xtal frequency this is used directly (with possibly an integer division).
    /// The AHB frequency follows the CPU frequency.
    /// Below 10MHz, the ref clock is not guaranteed to be at 1MHz
    ///
    /// Above the PLL is used and 80, 160, 240MHz are possible configurations.
    /// The AHB frequency is fixed at 80MHz.
    ///
    /// So for a 40Mhz Xtal, valid frequencies are: 240, 160, 80, 40, 20, 13.33, 10, 8, 6.67, 5.71, 5, 4.44, 4
    /// So for a 26Mhz Xtal, valid frequencies are: 240, 160, 80, 26, 13, 8.67, 6.5, 5.2, 4.33, 3.71
    /// So for a 24Mhz Xtal, valid frequencies are: 240, 160, 80, 24, 12, 8, 6, 4.8, 4
    ///
    /// # TODO
    /// - PLL Frequency
    pub fn set_cpu_frequency<T: Into<Hertz> + Copy + PartialOrd>(
        &mut self,
        frequency: T,
    ) -> Result<&mut Self, Error> {
        match frequency.into() {
            Hertz(0) => {}
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
        Ok(self)
    }

    /// Get Ref Tick frequency
    ///
    /// This frequency is usually 1MHz, but cannot be maintained when the APB_CLK is < 10MHz
    pub fn ref_frequency(&self) -> Hertz {
        let div = self
            .apb_control
            .xtal_tick_conf
            .read()
            .xtal_tick_num()
            .bits();

        self.apb_frequency() / (div + 1) as u32
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
            FAST_CLK_RTC_SEL_A::XTAL => FastRTCSource::Xtal,
        }
    }

    /// Get RTC/Slow frequency
    pub fn fast_rtc_frequency(&self) -> Hertz {
        match self.fast_rtc_source() {
            FastRTCSource::RTC8M => RTC_FAST_CLK_FREQ_8M,
            FastRTCSource::Xtal => self.xtal_frequency(),
        }
    }

    /// Set the Fast RTC clock source
    pub fn set_fast_rtc_source(&mut self, source: FastRTCSource) -> &mut Self {
        match source {
            FastRTCSource::RTC8M => self
                .rtc_control
                .clk_conf
                .modify(|_, w| w.fast_clk_rtc_sel().ck8m()),
            FastRTCSource::Xtal => self
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
            return 40.MHz().into();
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

    pub fn set_pll_frequency<T: Into<Hertz> + Copy>(&self, frequency: T) -> Result<(), Error> {
        let pll_config = match frequency.into() {
            Hertz(320_000_000) => match self.xtal_frequency() {
                Hertz(40_000_000) => PLL_CONFIG_320M_XTAL_40M,
                Hertz(26_000_000) => PLL_CONFIG_320M_XTAL_26M,
                Hertz(24_000_000) => PLL_CONFIG_320M_XTAL_24M,
                _ => PLL_CONFIG_320M_XTAL_UNKNOWN,
            },
            Hertz(480_000_000) => match self.xtal_frequency() {
                Hertz(40_000_000) => PLL_CONFIG_480M_XTAL_40M,
                Hertz(26_000_000) => PLL_CONFIG_480M_XTAL_26M,
                Hertz(24_000_000) => PLL_CONFIG_480M_XTAL_24M,
                _ => PLL_CONFIG_480M_XTAL_UNKNOWN,
            },
            _ => return Err(Error::UnsupportedPLLConfig),
        };

        match frequency.into() {
            Hertz(320_000_000) => {
                self.rtc_control
                    .bias_conf
                    .write(|w| w.dig_dbias_wak().variant(DIG_DBIAS_80M_160M));

                crate::rom::rom_i2c_writeReg(0, 0, 0, 0);
            }
            Hertz(480_000_000) => {}
            _ => {}
        }

        Ok(())
    }

    pub fn get_config(&self) -> Result<ClockControlConfig, Error> {
        Ok(ClockControlConfig {
            cpu_frequency: self.cpu_frequency(),
            apb_frequency: self.apb_frequency(),
            ref_frequency: self.ref_frequency(),
            slow_rtc_frequency: self.slow_rtc_frequency(),
            fast_rtc_frequency: self.fast_rtc_frequency(),

            apll_frequency: 0.Hz(),
            pll_d2_frequency: 0.Hz(),

            xtal_frequency: self.xtal_frequency(),
            xtal32k_frequency: 0.Hz(),
            pll_frequency: 0.Hz(),
            rtc8m_frequency: 0.Hz(),
            rtc_frequency: 0.Hz(),

            cpu_source: self.cpu_source(),
            slow_rtc_source: self.slow_rtc_source()?,
            fast_rtc_source: self.fast_rtc_source(),
        })
    }
}
