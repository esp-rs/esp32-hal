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

extern "C" {

    // These symbols come from `memory.x`
    static mut _rtc_fast_bss_start: u32;
    static mut _rtc_fast_bss_end: u32;

    static mut _rtc_fast_data_start: u32;
    static mut _rtc_fast_data_end: u32;
    static _rtc_fast_data_start_loadaddr: u32;

    static mut _rtc_slow_bss_start: u32;
    static mut _rtc_slow_bss_end: u32;

    static mut _rtc_slow_data_start: u32;
    static mut _rtc_slow_data_end: u32;
    static _rtc_slow_data_start_loadaddr: u32;

}

//#[xtensa_lx6_rt::pre_init]

#[xtensa_lx6_rt::pre_init]
unsafe fn pre_init() {
    loop {}
    // Initialize RTC RAM
    xtensa_lx6_rt::zero_bss(&mut _rtc_fast_bss_start, &mut _rtc_fast_bss_end);
    xtensa_lx6_rt::init_data(
        &mut _rtc_fast_data_start,
        &mut _rtc_fast_data_end,
        &_rtc_fast_data_start_loadaddr,
    );

    xtensa_lx6_rt::zero_bss(&mut _rtc_slow_bss_start, &mut _rtc_slow_bss_end);
    xtensa_lx6_rt::init_data(
        &mut _rtc_slow_data_start,
        &mut _rtc_slow_data_end,
        &_rtc_slow_data_start_loadaddr,
    );
}
