//! This ESP32 hal crate provides support for the ESP32 peripherals
//!
//! ## Features
//! - `external_ram` (enabled by default)
//!     - Enables support for external ram (psram). However proper initialization
//!         of external ram relies on a customized bootloader
//! - `all_in_ram`
//!     - Forces all code and data in RAM instead of flash. This allows usage with
//!         the ROM bootloader and eases debugging
//! - `alloc`
//!     - Enables support for dynamic memory allocations via a GlobalAllocator
//!         and/or AllocRef
//! - `mem`
//!     - Include customized memcpy, memset, etc. which use word (4-byte) sized and aligned
//!         instructions to support IRAM usage and as optimization

#![no_std]
#![feature(const_fn)]
#![cfg_attr(feature = "alloc", feature(allocator_api))]
#![cfg_attr(feature = "alloc", feature(alloc_layout_extra))]

pub use embedded_hal as hal;
pub use esp32 as target;

extern crate esp32_hal_proc_macros as proc_macros;
pub use proc_macros::interrupt;
pub use proc_macros::ram;

pub mod analog;
pub mod clock_control;
pub mod dport;
pub mod efuse;
#[cfg(feature = "external_ram")]
pub mod external_ram;
pub mod gpio;
pub mod i2c;
#[cfg(feature = "rt")]
pub mod interrupt;
pub mod prelude;
pub mod serial;
pub mod timer;
pub mod units;

#[cfg(feature = "alloc")]
pub mod alloc;

#[macro_use]
pub mod dprint;

#[cfg(feature = "mem")]
pub mod mem;

/// Function initializes ESP32 specific memories (RTC slow and fast) and
/// then calls original Reset function
///
/// ENTRY point is defined in memory.x
/// *Note: the pre_init function is called in the original reset handler
/// after the initializations done in this function*
#[cfg(feature = "rt")]
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn ESP32Reset() -> ! {
    // These symbols come from `memory.x`
    extern "C" {
        static mut _rtc_fast_bss_start: u32;
        static mut _rtc_fast_bss_end: u32;

        static mut _rtc_slow_bss_start: u32;
        static mut _rtc_slow_bss_end: u32;

        static mut _stack_end_cpu0: u32;
    }

    // copying data from flash to various data segments is done by the bootloader
    // initialization to zero needs to be done by the application

    // Initialize RTC RAM
    xtensa_lx6_rt::zero_bss(&mut _rtc_fast_bss_start, &mut _rtc_fast_bss_end);
    xtensa_lx6_rt::zero_bss(&mut _rtc_slow_bss_start, &mut _rtc_slow_bss_end);

    #[cfg(feature = "external_ram")]
    external_ram::init();

    // set stack pointer to end of memory: no need to retain stack up to this point
    xtensa_lx6::set_stack_pointer(&mut _stack_end_cpu0);

    // continue with default reset handler
    xtensa_lx6_rt::Reset();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Core {
    PRO = 0,
    APP = 1,
}

pub fn get_core() -> Core {
    match ((xtensa_lx6::get_processor_id() >> 13) & 1) != 0 {
        false => Core::PRO,
        true => Core::APP,
    }
}

pub fn get_other_core() -> Core {
    match get_core() {
        Core::PRO => Core::APP,
        Core::APP => Core::PRO,
    }
}
