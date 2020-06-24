//! Control Cores
//!

use super::Error;
use crate::target;
use crate::Core::{self, APP, PRO};
use xtensa_lx6::set_stack_pointer;

static mut START_CORE1_FUNCTION: Option<fn() -> !> = None;

impl super::ClockControl {
    pub unsafe fn park_core(&mut self, core: Core) {
        match core {
            PRO => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| w.sw_stall_procpu_c1().bits(0x21));
                self.rtc_control
                    .options0
                    .modify(|_, w| w.sw_stall_procpu_c0().bits(0x02));
            }
            APP => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| w.sw_stall_appcpu_c1().bits(0x21));
                self.rtc_control
                    .options0
                    .modify(|_, w| w.sw_stall_appcpu_c0().bits(0x02));
            }
        }
    }

    pub fn unpark_core(&mut self, core: Core) {
        match core {
            PRO => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| unsafe { w.sw_stall_procpu_c1().bits(0) });
                self.rtc_control
                    .options0
                    .modify(|_, w| unsafe { w.sw_stall_procpu_c0().bits(0) });
            }
            APP => {
                self.rtc_control
                    .sw_cpu_stall
                    .modify(|_, w| unsafe { w.sw_stall_appcpu_c1().bits(0) });
                self.rtc_control
                    .options0
                    .modify(|_, w| unsafe { w.sw_stall_appcpu_c0().bits(0) });
            }
        }
    }

    fn flush_cache(&mut self, core: Core) {
        match core {
            PRO => {
                self.dport_control
                    .pro_cache_ctrl()
                    .modify(|_, w| w.pro_cache_flush_ena().clear_bit());
                self.dport_control
                    .pro_cache_ctrl()
                    .modify(|_, w| w.pro_cache_flush_ena().set_bit());
                while self
                    .dport_control
                    .pro_cache_ctrl()
                    .read()
                    .pro_cache_flush_done()
                    .bit_is_clear()
                {}
                self.dport_control
                    .pro_cache_ctrl()
                    .modify(|_, w| w.pro_cache_flush_ena().clear_bit());
            }
            APP => {
                self.dport_control
                    .app_cache_ctrl()
                    .modify(|_, w| w.app_cache_flush_ena().clear_bit());
                self.dport_control
                    .app_cache_ctrl()
                    .modify(|_, w| w.app_cache_flush_ena().set_bit());
                while self
                    .dport_control
                    .app_cache_ctrl()
                    .read()
                    .app_cache_flush_done()
                    .bit_is_clear()
                {}
                self.dport_control
                    .app_cache_ctrl()
                    .modify(|_, w| w.app_cache_flush_ena().clear_bit());
            }
        };
    }

    fn enable_cache(&mut self, core: Core) {
        // get timer group 0 registers, do it this way instead of
        // having to pass in yet another peripheral for this clock control
        let spi0 = unsafe { &(*target::SPI0::ptr()) };

        match core {
            PRO => {
                spi0.cache_fctrl.modify(|_, w| w.cache_req_en().set_bit());
                self.dport_control
                    .pro_cache_ctrl()
                    .modify(|_, w| w.pro_cache_enable().set_bit());
            }
            APP => {
                spi0.cache_fctrl.modify(|_, w| w.cache_req_en().set_bit());
                self.dport_control
                    .app_cache_ctrl()
                    .modify(|_, w| w.app_cache_enable().set_bit());
            }
        };
    }

    unsafe fn start_core1_init() -> ! {
        extern "C" {
            static mut _stack_end_cpu1: u32;
        }

        // disables interrupts
        xtensa_lx6::interrupt::set_mask(0);

        // reset cycle compare registers
        xtensa_lx6::timer::set_ccompare0(0);
        xtensa_lx6::timer::set_ccompare1(0);
        xtensa_lx6::timer::set_ccompare2(0);

        // set stack pointer to end of memory: no need to retain stack up to this point
        set_stack_pointer(&mut _stack_end_cpu1);

        START_CORE1_FUNCTION.unwrap()();
    }

    /// Start the APP (second) core
    ///
    /// The second core will start running with the function `entry`.
    pub fn start_app_core(&mut self, entry: fn() -> !) -> Result<(), Error> {
        if self
            .dport_control
            .appcpu_ctrl_b()
            .read()
            .appcpu_clkgate_en()
            .bit_is_set()
        {
            return Err(Error::CoreAlreadyRunning);
        }

        self.flush_cache(Core::APP);
        self.enable_cache(Core::APP);

        unsafe {
            START_CORE1_FUNCTION = Some(entry);
        }

        self.dport_control.appcpu_ctrl_d().write(|w| unsafe {
            w.appcpu_boot_addr()
                .bits(Self::start_core1_init as *const u32 as u32)
        });

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

        self.unpark_core(Core::APP);

        Ok(())
    }
}
