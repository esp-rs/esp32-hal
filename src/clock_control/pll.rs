//! PLL control
//!

use crate::prelude::*;

// Delays (in microseconds) for changing pll settings
// TODO according to esp-idf: some of these are excessive, and should be reduced.

const DELAY_PLL_DBIAS_RAISE: MicroSeconds = MicroSeconds(3);
const DELAY_PLL_ENABLE_WITH_150K: MicroSeconds = MicroSeconds(80);
const DELAY_PLL_ENABLE_WITH_32K: MicroSeconds = MicroSeconds(160);

// Addresses for internal I2C bus for PLL
const I2C_BLOCK: u8 = 0x66;
const I2C_HOSTID: u8 = 4;

// Register addresses for internal I2C bus
mod i2c {
    pub const IR_CAL_DELAY: u8 = 0;
    pub const IR_CAL_EXT_CAP: u8 = 1;
    pub const OC_LREF: u8 = 2;
    pub const OC_DIV_7_0: u8 = 3;
    pub const OC_ENB_FCAL: u8 = 4;
    pub const OC_DCUR: u8 = 5;
    pub const BBADC_DSMP: u8 = 9;
    pub const OC_ENB_VCON: u8 = 10;
    pub const ENDIV5: u8 = 11;
    pub const BBADC_CAL_7_0: u8 = 12;
}

// Values for internal I2C registers
mod val {
    pub const ENDIV5_VAL_320M: u8 = 0x43;
    pub const BBADC_DSMP_VAL_320M: u8 = 0x84;
    pub const ENDIV5_VAL_480M: u8 = 0xc3;
    pub const BBADC_DSMP_VAL_480M: u8 = 0x74;
    pub const IR_CAL_DELAY_VAL: u8 = 0x18;
    pub const IR_CAL_EXT_CAP_VAL: u8 = 0x20;
    pub const OC_ENB_FCAL_VAL: u8 = 0x9a;
    pub const OC_ENB_VCON_VAL: u8 = 0x00;
    pub const BBADC_CAL_7_0_VAL: u8 = 0x00;
}

// COnfiguration values depending on Xtal frequency for internal I2C registers
// div_ref, div7_0, div10_8, lref,dcur,bw
struct Config(u8, u8, u8, u8, u8, u8);

impl Config {
    const PLL_320M_XTAL_40M: Config = Config(0, 32, 0, 0, 6, 3);
    const PLL_320M_XTAL_26M: Config = Config(12, 224, 4, 1, 0, 1);
    const PLL_320M_XTAL_24M: Config = Config(11, 224, 4, 1, 0, 1);

    const PLL_480M_XTAL_40M: Config = Config(0, 28, 0, 0, 6, 3);
    const PLL_480M_XTAL_26M: Config = Config(12, 144, 4, 1, 0, 1);
    const PLL_480M_XTAL_24M: Config = Config(11, 144, 4, 1, 0, 1);

    fn get_lref(&self) -> u8 {
        (self.3 << 7) | (self.2 << 4) | self.0
    }
    fn get_div7_0(&self) -> u8 {
        self.1
    }
    fn get_dcur(&self) -> u8 {
        (self.5 << 6) | self.4
    }
}

use super::Error;

impl super::ClockControl {
    fn write_i2c(address: u8, data: u8) {
        unsafe {
            crate::rom::rom_i2c_writeReg(I2C_BLOCK, I2C_HOSTID, address, data);
        }
    }

    pub(crate) fn pll_disable(&mut self) {
        self.rtc_control.options0.modify(|_, w| {
            w.bias_i2c_force_pd()
                // is APLL under force power down? then also power down the internal I2C bus
                .bit(self.rtc_control.ana_conf.read().plla_force_pd().bit())
                .bb_i2c_force_pd()
                .set_bit()
                .bbpll_force_pd()
                .set_bit()
                .bbpll_i2c_force_pd()
                .set_bit()
        });
    }

    pub(crate) fn pll_enable(&mut self) {
        self.rtc_control.options0.modify(|_, w| {
            w.bias_i2c_force_pd()
                .clear_bit()
                .bb_i2c_force_pd()
                .clear_bit()
                .bbpll_force_pd()
                .clear_bit()
                .bbpll_i2c_force_pd()
                .clear_bit()
        });

        /* reset BBPLL configuration */
        Self::write_i2c(i2c::IR_CAL_DELAY, val::IR_CAL_DELAY_VAL);
        Self::write_i2c(i2c::IR_CAL_EXT_CAP, val::IR_CAL_EXT_CAP_VAL);
        Self::write_i2c(i2c::OC_ENB_FCAL, val::OC_ENB_FCAL_VAL);
        Self::write_i2c(i2c::OC_ENB_VCON, val::OC_ENB_VCON_VAL);
        Self::write_i2c(i2c::BBADC_CAL_7_0, val::BBADC_CAL_7_0_VAL);
    }

    pub(crate) fn set_pll_frequency(&mut self, high: bool) -> Result<(), Error> {
        let pll_config = match high {
            false => {
                Self::write_i2c(i2c::ENDIV5, val::ENDIV5_VAL_320M);
                Self::write_i2c(i2c::BBADC_DSMP, val::BBADC_DSMP_VAL_320M);

                match self.xtal_frequency() {
                    Hertz(40_000_000) => Config::PLL_320M_XTAL_40M,
                    Hertz(26_000_000) => Config::PLL_320M_XTAL_26M,
                    Hertz(24_000_000) => Config::PLL_320M_XTAL_24M,
                    _ => return Err(Error::UnsupportedPLLConfig),
                }
            }
            true => {
                Self::write_i2c(i2c::ENDIV5, val::ENDIV5_VAL_480M);
                Self::write_i2c(i2c::BBADC_DSMP, val::BBADC_DSMP_VAL_480M);

                match self.xtal_frequency() {
                    Hertz(40_000_000) => Config::PLL_480M_XTAL_40M,
                    Hertz(26_000_000) => Config::PLL_480M_XTAL_26M,
                    Hertz(24_000_000) => Config::PLL_480M_XTAL_24M,
                    _ => return Err(Error::UnsupportedPLLConfig),
                }
            }
        };

        Self::write_i2c(i2c::OC_LREF, pll_config.get_lref());
        Self::write_i2c(i2c::OC_DIV_7_0, pll_config.get_div7_0());
        Self::write_i2c(i2c::OC_DCUR, pll_config.get_dcur());

        let delay_us = if let Ok(super::SlowRTCSource::RTC150k) = self.slow_rtc_source() {
            DELAY_PLL_ENABLE_WITH_150K
        } else {
            DELAY_PLL_ENABLE_WITH_32K
        };

        self.delay(delay_us);
        Ok(())
    }
}
