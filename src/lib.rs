//! This ESP32 hal crate provides support for the ESP32 peripherals
//!
//! ## Features
//! - `external_ram`
//!     - Enables support for external ram (psram). However proper initialization
//!         of external ram relies on a customized bootloader
//! - `all_in_ram`
//!     - Forces all code and data in RAM instead of flash. This allows usage with
//!         the ROM bootloader and eases debugging

#![no_std]
#![feature(const_fn)]

pub use embedded_hal as hal;
pub use esp32;

extern crate esp32_hal_proc_macros as proc_macros;
pub use proc_macros::ram;

<<<<<<< HEAD
pub mod analog;
=======
>>>>>>> e09c690... Added procedural macro #[ram]
pub mod clock_control;
pub mod dport;
pub mod efuse;
pub mod gpio;
pub mod prelude;
pub mod serial;
pub mod units;

#[macro_use]
pub mod dprint;

#[cfg(feature = "alloc")]
pub mod alloc;

/// Function initializes ESP32 specific memories (RTC slow and fast) and
/// then calls original Reset function
///
/// ENTRY point is defined in memory.x
/// *Note: the pre_init function is called in the original reset handler
/// after the initializations done in this function*
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn ESP32Reset() -> ! {
    // These symbols come from `memory.x`
    extern "C" {
        static mut _rtc_fast_bss_start: u32;
        static mut _rtc_fast_bss_end: u32;

        static mut _rtc_slow_bss_start: u32;
        static mut _rtc_slow_bss_end: u32;

        static mut _external_bss_start: u32;
        static mut _external_bss_end: u32;

        static mut _stack_end_cpu0: u32;
    }

    // copying data from flash to various data segments is done by the bootloader
    // initialization to zero needs to be done by the application

    // Initialize RTC RAM
    xtensa_lx6_rt::zero_bss(&mut _rtc_fast_bss_start, &mut _rtc_fast_bss_end);
    xtensa_lx6_rt::zero_bss(&mut _rtc_slow_bss_start, &mut _rtc_slow_bss_end);

    if cfg!(feature = "external_ram") {
        xtensa_lx6_rt::zero_bss(&mut _external_bss_start, &mut _external_bss_end);
    }

    // set stack pointer to end of memory: no need to retain stack up to this point
    xtensa_lx6_rt::set_stack_pointer(&mut _stack_end_cpu0);

    // continue with default reset handler
    xtensa_lx6_rt::Reset();
}
