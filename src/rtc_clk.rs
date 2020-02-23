use esp32::{APB_CTRL, DPORT, RTCCNTL};
use esp32::dport::cpu_per_conf::CPUPERIOD_SEL_A;
use esp32::rtccntl::clk_conf::SOC_CLK_SEL_A;
use esp32::generic::Variant::*;

/// Disable logging from the ROM code.
pub const RTC_DISABLE_ROM_LOG: u32 = ((1 << 0) | (1 << 16));

/// RTC PLL 320 MHz frequency
pub const RTC_PLL_FREQ_320M: u32 = 320;
/// RTC PLL 480 MHz frequency
pub const RTC_PLL_FREQ_480M: u32 = 480;

/// RTC Clock errors
pub enum Error {
    /// Unsupported frequency configuration
    UnsupportedFreqConfig,
}

/// CPU frequency source
pub enum CpuFreqSource {
    /// XTAL
    Xtal,
    /// PLL
    Pll,
    /// 8M
    Src8M,
}

/// CPU frequency configuration
pub struct CpuFreqConfig {
    /// CPU Frequency Source
    pub source: CpuFreqSource,
    /// CPU Source Frequency in MHz
    pub source_freq_mhz: u32,
    /// Frequency divider
    pub div: u32,
    /// CPU Frequency in MHz
    pub freq_mhz: u32,
}

impl CpuFreqConfig {
    /// Read `CpuFreqConfig` from the ESP32 registers
    pub fn read() -> Result<CpuFreqConfig, Error> {
        let rtc_cntl = unsafe { &(*RTCCNTL::ptr()) };

        match rtc_cntl.clk_conf.read().soc_clk_sel().variant() {
            SOC_CLK_SEL_A::XTAL => {
                let mut config = CpuFreqConfig {
                    source: CpuFreqSource::Xtal,
                    source_freq_mhz: 0,
                    div: 0,
                    freq_mhz: 0,
                };
                let apb_ctrl = unsafe { &(*APB_CTRL::ptr()) };
                config.div = (apb_ctrl.sysclk_conf.read().pre_div_cnt().bits() + 1).into();
                config.source_freq_mhz = xtal_freq_get();
                config.freq_mhz = config.source_freq_mhz / config.div;
                Ok(config)
            }
            SOC_CLK_SEL_A::PLL => {
                let mut config = CpuFreqConfig {
                    source: CpuFreqSource::Pll,
                    source_freq_mhz: 0,
                    div: 0,
                    freq_mhz: 0,
                };
                let dport = unsafe { &(*DPORT::ptr()) };
                match dport.cpu_per_conf.read().cpuperiod_sel().variant() {
                    Val(CPUPERIOD_SEL_A::SEL_80) => {
                        config.source_freq_mhz = RTC_PLL_FREQ_320M;
                        config.div = 4;
                        config.freq_mhz = 80;
                    }
                    Val(CPUPERIOD_SEL_A::SEL_160) => {
                        config.source_freq_mhz = RTC_PLL_FREQ_320M;
                        config.div = 2;
                        config.freq_mhz = 160;
                    }
                    Val(CPUPERIOD_SEL_A::SEL_240) => {
                        config.source_freq_mhz = RTC_PLL_FREQ_480M;
                        config.div = 2;
                        config.freq_mhz = 240;
                    }
                    Res(_) => {
                        return Err(Error::UnsupportedFreqConfig);
                    }
                }
                Ok(config)
            }
            SOC_CLK_SEL_A::CK8M => {
                Ok(CpuFreqConfig {
                    source: CpuFreqSource::Src8M,
                    source_freq_mhz: 8,
                    div: 1,
                    freq_mhz: 8,
                })
            }
            SOC_CLK_SEL_A::APLL => {
                Err(Error::UnsupportedFreqConfig)
            }
        }
    }
}

/// Get XTAL frequency
pub fn xtal_freq_get() -> u32 {
    let rtc_cntl = unsafe { &(*RTCCNTL::ptr()) };

    // We may have already written XTAL value into RTC_XTAL_FREQ_REG
    let xtal_freq_reg = rtc_cntl.store4.read().scratch4().bits();
    if !clk_val_is_valid(xtal_freq_reg) {
        return 0;
    }

    reg_val_to_clk_val(xtal_freq_reg & (!RTC_DISABLE_ROM_LOG))
}

/// Check if a value from RTC_XTAL_FREQ_REG or RTC_APB_FREQ_REG are valid clocks
#[inline(always)]
fn clk_val_is_valid(val: u32) -> bool {
    (val & 0xffff) == ((val >> 16) & 0xffff) && val != 0 && val != u32::max_value()
}

/// Convert a value from RTC_XTAL_FREQ_REG or RTC_APB_FREQ_REG to a clock value
#[inline(always)]
fn reg_val_to_clk_val(val: u32) -> u32 {
    val & u16::max_value() as u32
}
