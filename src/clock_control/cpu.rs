//! Control Cores
//!

use super::Error;
use crate::prelude::*;
use esp32::generic::Variant::Val;

impl super::ClockControl {
    fn park_core(&mut self, core: u32) -> Result<(), Error> {
        match core {
            //TODO: check if necessary to set to 0 like in cpu_start.c?
            0 => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| unsafe { w.sw_stall_procpu_c1().bits(0x21) });
                self.rtc_control
                    .options0
                    .modify(|_, w| unsafe { w.sw_stall_procpu_c0().bits(0x02) });
            }
            1 => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| unsafe { w.sw_stall_appcpu_c1().bits(0x21) });
                self.rtc_control
                    .options0
                    .modify(|_, w| unsafe { w.sw_stall_appcpu_c0().bits(0x02) });
            }
            _ => return Err(Error::InvalidCore),
        };
        Ok(())
    }

    fn unpark_core(&mut self, core: u32) -> Result<(), Error> {
        match core {
            //TODO: check if necessary to set to 0 like in cpu_start.c?
            0 => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| unsafe { w.sw_stall_procpu_c1().bits(0) });
                self.rtc_control
                    .options0
                    .modify(|_, w| unsafe { w.sw_stall_procpu_c0().bits(0) });
            }
            1 => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| unsafe { w.sw_stall_appcpu_c1().bits(0) });
                self.rtc_control
                    .options0
                    .modify(|_, w| unsafe { w.sw_stall_appcpu_c0().bits(0) });
            }
            _ => return Err(Error::InvalidCore),
        };
        Ok(())
    }

    pub fn start_core(&mut self, core: u32, f: fn() -> !) -> Result<(), Error> {
        match core {
            0 => return Err(Error::CoreAlreadyRunning),
            1 => {
                if self
                    .dport_control
                    .appcpu_ctrl_b()
                    .read()
                    .appcpu_clkgate_en()
                    .bit_is_set()
                {
                    return Err(Error::CoreAlreadyRunning);
                }

                self.dport_control
                    .appcpu_ctrl_d()
                    .write(|w| unsafe { w.appcpu_boot_addr().bits(f as u32) });

                self.dport_control
                    .appcpu_ctrl_b()
                    .modify(|_, w| w.appcpu_clkgate_en().set_bit());
                self.dport_control
                    .appcpu_ctrl_c()
                    .modify(|_, w| w.appcpu_runstall().clear_bit());
                self.dport_control
                    .appcpu_ctrl_a()
                    .modify(|_, w| w.appcpu_resetting().set_bit());
                self.dport_control
                    .appcpu_ctrl_a()
                    .modify(|_, w| w.appcpu_resetting().clear_bit());

                self.unpark_core(core);
            }
            _ => return Err(Error::InvalidCore),
        }

        Ok(())
    }
}

/*
APP_CPU is reset when DPORT_APPCPU_RESETTING=1. It is released when
DPORT_APPCPU_RESETTING=0.
• When DPORT_APPCPU_CLKGATE_EN=0, the APP_CPU clock can be disabled to reduce power
consumption.
• When DPORT_APPCPU_RUNSTALL=1, the APP_CPU can be put into a stalled state.
• When APP_CPU is booted up with a ROM code, it will jump to the address stored in the
DPORT_APPCPU_BOOT_ADDR register.

// Enable clock and reset APP CPU. Note that OpenOCD may have already
    // enabled clock and taken APP CPU out of reset. In this case don't reset
    // APP CPU again, as that will clear the breakpoints which may have already
    // been set.


   esp_cpu_unstall(1);

        if (!DPORT_GET_PERI_REG_MASK(DPORT_APPCPU_CTRL_B_REG, DPORT_APPCPU_CLKGATE_EN)) {
        DPORT_SET_PERI_REG_MASK(DPORT_APPCPU_CTRL_B_REG, DPORT_APPCPU_CLKGATE_EN);
        DPORT_CLEAR_PERI_REG_MASK(DPORT_APPCPU_CTRL_C_REG, DPORT_APPCPU_RUNSTALL);
        DPORT_SET_PERI_REG_MASK(DPORT_APPCPU_CTRL_A_REG, DPORT_APPCPU_RESETTING);
        DPORT_CLEAR_PERI_REG_MASK(DPORT_APPCPU_CTRL_A_REG, DPORT_APPCPU_RESETTING);
    }
    ets_set_appcpu_boot_addr((uint32_t)call_start_cpu1);

*/
