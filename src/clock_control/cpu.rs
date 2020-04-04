//! Control Cores
//!

use super::Error;

impl super::ClockControl {
    pub unsafe fn park_core(&mut self, core: u32) -> Result<(), Error> {
        match core {
            0 => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| w.sw_stall_procpu_c1().bits(0x21));
                self.rtc_control
                    .options0
                    .modify(|_, w| w.sw_stall_procpu_c0().bits(0x02));
            }
            1 => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| w.sw_stall_appcpu_c1().bits(0x21));
                self.rtc_control
                    .options0
                    .modify(|_, w| w.sw_stall_appcpu_c0().bits(0x02));
            }
            _ => return Err(Error::InvalidCore),
        };
        Ok(())
    }

    pub fn unpark_core(&mut self, core: u32) -> Result<(), Error> {
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

                self.unpark_core(core)?;
            }
            _ => return Err(Error::InvalidCore),
        }

        Ok(())
    }
}
