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

use crate::prelude::*;
use esp32::dport::cpu_per_conf::CPUPERIOD_SEL_A;
use esp32::generic::Variant::*;
use esp32::rtccntl::bias_conf::*;
use esp32::rtccntl::clk_conf::*;
use esp32::{APB_CTRL, DPORT, RTCCNTL};

pub type SlowClockSource = ANA_CLK_RTC_SEL_A;

/// Reference clock frequency always at 1MHz
pub const REF_CLK_FREQ_1M: Hertz = Hertz(1_000_000);

const RTC_PLL_FREQ_320M: MegaHertz = MegaHertz(320);
const RTC_PLL_FREQ_480M: MegaHertz = MegaHertz(480);

const RTC_SLOW_CLK_FREQ_32K: Hertz = Hertz(32_768);
const RTC_SLOW_CLK_FREQ_150K: Hertz = Hertz(150_000);
const RTC_FAST_CLK_FREQ_8M: Hertz = Hertz(8_500_000); //With the default value of CK8M_DFREQ, 8M clock frequency is 8.5 MHz +/- 7%
const RTC_SLOW_CLK_FREQ_8MD256: Hertz = Hertz(RTC_FAST_CLK_FREQ_8M.0 / 256);

type CoreBias = DBIAS_WAK_A;

const DIG_DBIAS_240M_OR_FLASH_80M: CoreBias = CoreBias::BIAS_1V25;

const DIG_DBIAS_80M_160M: CoreBias = CoreBias::BIAS_1V10;

const DIG_DBIAS_XTAL: CoreBias = CoreBias::BIAS_1V10;
const DIG_DBIAS_2M: CoreBias = CoreBias::BIAS_1V00;

/// RTC Clock errors
#[derive(Debug)]
pub enum Error {
    /// Unsupported frequency configuration
    UnsupportedFreqConfig,
    UnsupportedWatchdogConfig,
}

/// CPU frequency source
#[derive(Debug)]
pub enum CpuFreqSource {
    /// High frequency Xtal (26MHz or 40MHz)
    Xtal,
    /// PLL generated frequency from high frequency Xtal
    Pll,
    /// 8MHz internal oscillator
    Src8M,
}

/// CPU frequency configuration
#[derive(Debug)]
pub struct CpuFreqConfig {
    /// CPU Frequency Source
    pub source: CpuFreqSource,
    /// CPU Source Frequency
    pub source_frequency: Hertz,
    /// Frequency divider
    pub divider: u32,
    /// CPU Frequency
    pub frequency: Hertz,
}

pub mod watchdog;

pub struct ClockControl {
    rtccntl: RTCCNTL,
    apbctrl: APB_CTRL,
}

impl ClockControl {
    /// Create new ClockControl structure
    pub fn new(rtccntl: RTCCNTL, apbctrl: APB_CTRL) -> Self {
        let mut cc = ClockControl { rtccntl, apbctrl };
        cc.init();
        cc
    }

    /// Initialize clock configuration
    fn init(&mut self) -> &mut Self {
        if self.rtccntl.clk_conf.read().soc_clk_sel().is_pll() {
            self.set_cpu_freq_to_xtal::<Hertz>(40.MHz().into(), 1);
        }

        self
    }

    /// Gets RTC watchdog control
    pub fn watchdog<'a>(&'a mut self) -> watchdog::WatchDog<'a> {
        watchdog::WatchDog::new(self)
    }

    /// Sets high frequency Xtal as source for CPU clock and configure the APB divider
    pub fn set_cpu_freq_to_xtal<T: Into<Hertz> + Copy>(
        &mut self,
        frequency: T,
        div: u16,
    ) -> &mut Self {
        // set divider from XTAL to APB clock
        self.apbctrl
            .sysclk_conf
            .modify(|_, w| unsafe { w.pre_div_cnt().bits(div - 1) });
        // adjust ref tick
        self.apbctrl
            .xtal_tick_conf
            .write(|w| unsafe { w.bits(frequency.into() / REF_CLK_FREQ_1M - 1) });
        // switch clock source
        self.rtccntl.clk_conf.modify(|_, w| w.soc_clk_sel().xtal());

        // select appropriate voltage
        self.rtccntl.bias_conf.modify(|_, w| {
            w.dig_dbias_wak()
                .variant(if frequency.into() < 2.MHz().into() {
                    DIG_DBIAS_2M
                } else {
                    DIG_DBIAS_XTAL
                })
        });

        self
    }

    /// Get Ref Tick frequency
    ///
    /// This frequency is usually 1MHz, but cannot be maintained when the APB_CLK is < 10MHz
    pub fn ref_frequency(&self) -> Hertz {
        1.MHz().into()
    }

    /// Get APB frequency
    ///
    /// This gets the APB frequency from the scratch register, which is initialized during the clock calibration
    pub fn apb_frequency(&self) -> Hertz {
        // We may have already written APB value into RTC_APB_FREQ_REG
        let apb_freq_reg = self.rtccntl.store5.read().scratch5().bits();

        if Self::clk_val_is_valid(apb_freq_reg) {
            // return 40MHz as default (this is recommended value)
            return 40.MHz().into();
        }

        let mut freq = (apb_freq_reg & 0x7fff) << 12;
        // round to nearest megaHertz
        freq += 500_000;
        freq -= freq % 1_000_000;
        Hertz(freq)
    }

    /// Set APB frequency
    ///
    /// Write the APB frequency to the scratch register for later retrieval
    fn set_apb_frequency<T: Into<MicroSeconds>>(&self, frequency: T) {
        // Write APB value into RTC_APB_FREQ_REG
        let mut val = u32::from(frequency.into()) >> 12;
        val = val | (val << 16); // value needs to be copied in lower and upper 16 bits
        self.rtccntl
            .store5
            .write(|w| unsafe { w.scratch5().bits(val) });
    }

    /// Get CPU frequency
    pub fn cpu_frequency(&self) -> Hertz {
        match self.cpu_frequency_config() {
            Ok(config) => config.frequency,
            _ => 0.Hz(),
        }
    }

    /// Get RTC/Slow frequency
    pub fn slow_frequency(&self) -> Hertz {
        match self.rtccntl.clk_conf.read().ana_clk_rtc_sel().variant() {
            Val(ANA_CLK_RTC_SEL_A::SLOW_CK) => RTC_SLOW_CLK_FREQ_150K,
            Val(ANA_CLK_RTC_SEL_A::CK_XTAL_32K) => RTC_SLOW_CLK_FREQ_32K,
            Val(ANA_CLK_RTC_SEL_A::CK8M_D256_OUT) => RTC_SLOW_CLK_FREQ_8MD256,
            _ => 0.Hz(),
        }
    }

    /// Set the RTC/Slow clock source
    pub fn set_slow_source(&mut self, source: SlowClockSource) {
        self.rtccntl
            .clk_conf
            .modify(|_, w| w.ana_clk_rtc_sel().variant(source));
    }

    /// Check if a value from RTC_XTAL_FREQ_REG or RTC_APB_FREQ_REG are valid clocks
    fn clk_val_is_valid(val: u32) -> bool {
        (val & 0xffff) == ((val >> 16) & 0xffff) && val != 0 && val != u32::max_value()
    }

    /// Get XTAL frequency.
    ///
    /// This gets the XTAL frequency from a scratch register, which is initialized during the clock calibration
    pub fn xtal_freq(&self) -> MegaHertz {
        // We may have already written XTAL value into RTC_XTAL_FREQ_REG
        let xtal_freq_reg = self.rtccntl.store4.read().scratch4().bits();
        if !Self::clk_val_is_valid(xtal_freq_reg) {
            // return 40MHz as default (this is recommended )
            return 40.MHz();
        }

        (xtal_freq_reg & 0x7fff).MHz() // bit15 is RTC_DISABLE_ROM_LOG flag
    }

    /// Get current `CpuFreqConfig`
    pub fn cpu_frequency_config(&self) -> Result<CpuFreqConfig, Error> {
        match self.rtccntl.clk_conf.read().soc_clk_sel().variant() {
            SOC_CLK_SEL_A::XTAL => {
                let mut config = CpuFreqConfig {
                    source: CpuFreqSource::Xtal,
                    source_frequency: 0.Hz(),
                    divider: 0,
                    frequency: 0.Hz(),
                };
                config.divider = (self.apbctrl.sysclk_conf.read().pre_div_cnt().bits() + 1).into();
                config.source_frequency = self.xtal_freq().into();
                config.frequency = (u32::from(config.source_frequency) / config.divider).into();
                Ok(config)
            }
            SOC_CLK_SEL_A::PLL => {
                let mut config = CpuFreqConfig {
                    source: CpuFreqSource::Pll,
                    source_frequency: 0.Hz(),
                    divider: 0,
                    frequency: 0.Hz(),
                };
                let dport = unsafe { &(*DPORT::ptr()) };
                match dport.cpu_per_conf.read().cpuperiod_sel().variant() {
                    Val(CPUPERIOD_SEL_A::SEL_80) => {
                        config.source_frequency = RTC_PLL_FREQ_320M.into();
                        config.divider = 4;
                        config.frequency = 80.MHz().into();
                    }
                    Val(CPUPERIOD_SEL_A::SEL_160) => {
                        config.source_frequency = RTC_PLL_FREQ_320M.into();
                        config.divider = 2;
                        config.frequency = 160.MHz().into();
                    }
                    Val(CPUPERIOD_SEL_A::SEL_240) => {
                        config.source_frequency = RTC_PLL_FREQ_480M.into();
                        config.divider = 2;
                        config.frequency = 240.MHz().into();
                    }
                    Res(_) => {
                        return Err(Error::UnsupportedFreqConfig);
                    }
                }
                Ok(config)
            }
            SOC_CLK_SEL_A::CK8M => Ok(CpuFreqConfig {
                source: CpuFreqSource::Src8M,
                source_frequency: 8.MHz().into(),
                divider: 1,
                frequency: 8.MHz().into(),
            }),
            SOC_CLK_SEL_A::APLL => Err(Error::UnsupportedFreqConfig),
        }
    }
}
