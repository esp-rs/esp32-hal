//! This ESP32 hal  crate provides support for the ESP32 peripherals
//!
//! ## Features
//! - `external_ram`
//!     - Optional and experimental
//!     - Enables support for external ram (psram). However proper initialization
//!         for psram is not yet included

#![no_std]

pub use embedded_hal as hal;
pub use esp32;

extern crate esp32_hal_proc_macros as proc_macros;
pub use proc_macros::ram;

pub mod analog;
pub mod clock_control;
pub mod dport;
pub mod efuse;
pub mod gpio;
pub mod prelude;
pub mod serial;
pub mod units;

#[macro_use]
pub mod dprint;

use xtensa_lx6_rt::{init_data, zero_bss};

extern "C" {
    // These symbols come from `memory.x`
    static mut _rtc_fast_bss_start: u32;
    static mut _rtc_fast_bss_end: u32;

    static mut _rtc_slow_bss_start: u32;
    static mut _rtc_slow_bss_end: u32;

    static mut _external_bss_start: u32;
    static mut _external_bss_end: u32;
    static mut _external_data_start: u32;
    static mut _external_data_end: u32;
    static _external_data_load: u32;
}

/// Function initializes ESP32 specific memories (RTC slow and fast)
///
/// if #\[pre_init\] is used to override the pre-initialization this function must be called manually

// function is used as default for __pre_init in memory.x
// (Using #[pre_init] from library does not work properly)
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn ESP32PreInit() {
    // Initialize RTC RAM
    zero_bss(&mut _rtc_fast_bss_start, &mut _rtc_fast_bss_end);
    zero_bss(&mut _rtc_slow_bss_start, &mut _rtc_slow_bss_end);

    if cfg!(feature = "external_ram") {
        zero_bss(&mut _external_bss_start, &mut _external_bss_end);

        // external SRAM initialization not done by bootloader
        //
        // TODO: correct load address or memory map:
        // _external_data_load points to address when flash address 0 is mapped to 3f400000,
        // however after bootloader is finished this is no longer true
        init_data(
            &mut _external_data_start,
            &mut _external_data_end,
            &_external_data_load,
        );
    }
}
