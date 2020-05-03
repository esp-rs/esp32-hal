//! Example to test memcpy function
//! To properly benchmark run in release mode
//!

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(raw_vec_internals)]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::alloc::{Allocator, DRAM_ALLOCATOR};

use esp32_hal::clock_control::{sleep, ClockControl, ClockControlConfig};
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

#[macro_use]
extern crate alloc;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: Allocator = DRAM_ALLOCATOR;

// Macro to simplify printing of the various different memory allocations
macro_rules! print_info {
    ( $uart:expr, $x:expr ) => {
        let mem_type = match &$x as *const _ as usize {
            0x3f80_0000..=0x3fbf_ffff => "External",
            0x3ff8_0000..=0x3fff_ffff => "DRAM",
            0x4007_0000..=0x4009_ffff => "IRAM",
            _ => "?",
        };
        writeln!(
            $uart,
            "{:<40}: {:#08x?}   {}",
            stringify!($x),
            &$x as *const _,
            mem_type
        )
        .unwrap();
    };
}

fn time(output: &mut dyn core::fmt::Write, text: &str, bytes: usize, f: &dyn Fn() -> ()) {
    const REPEAT: usize = 100;
    let start = xtensa_lx6_rt::get_cycle_count();
    for _ in 0..REPEAT {
        f();
    }
    let end = xtensa_lx6_rt::get_cycle_count();

    let time = (end - start) as f32 / ClockControlConfig {}.cpu_frequency().0 as f32;
    writeln!(
        output,
        "{:>40}: {:.3}s, {:.3}KB/s",
        text,
        time,
        (bytes * REPEAT) as f32 / time / 1024.0
    )
    .unwrap();
}

#[entry]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timers on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    // setup clocks & watchdog
    let clock_control = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    let (clock_control_config, mut watchdog) = clock_control.freeze().unwrap();

    watchdog.start(30.s());

    // setup serial controller
    let mut uart0 = Serial::uart0(
        dp.UART0,
        (NoTx, NoRx),
        Config::default(),
        clock_control_config,
        &mut dport,
    )
    .unwrap();

    uart0.change_baudrate(115200).unwrap();

    // print startup message
    writeln!(uart0, "\n\nReboot!\n",).unwrap();

    const BUF_LEN: usize = 1024 * 64;

    let dst = vec![0u8; BUF_LEN];
    let mut src = vec![0u8; BUF_LEN];

    print_info!(uart0, dst[0]);

    for i in 0..BUF_LEN {
        src[i] = i as u8;
    }

    time(
        &mut uart0,
        "memset aligned, sized 4 bytes",
        BUF_LEN,
        &|| unsafe {
            esp32_hal::mem::memset(&(dst[0]) as *const _ as *mut _, 0, BUF_LEN);
        },
    );

    time(&mut uart0, "memset aligned", BUF_LEN, &|| unsafe {
        esp32_hal::mem::memset(&(dst[0]) as *const _ as *mut _, 0, BUF_LEN - 1);
    });

    time(&mut uart0, "memset", BUF_LEN, &|| unsafe {
        esp32_hal::mem::memset(&(dst[1]) as *const _ as *mut _, 0, BUF_LEN - 1);
    });

    time(
        &mut uart0,
        "memcpy aligned, sized 4 bytes",
        BUF_LEN,
        &|| unsafe {
            esp32_hal::mem::memcpy(
                &(dst[0]) as *const _ as *mut _,
                &(src[0]) as *const _ as *mut _,
                BUF_LEN,
            );
        },
    );

    writeln!(uart0, "Result: {:?}", unsafe {
        esp32_hal::mem::memcmp(
            &(dst[0]) as *const _ as *mut _,
            &(src[0]) as *const _ as *mut _,
            BUF_LEN,
        )
    })
    .unwrap();
    unsafe {
        esp32_hal::mem::memset(&(dst[0]) as *const _ as *mut _, 0, BUF_LEN);
    }

    time(
        &mut uart0,
        "memcpy aligned 4 bytes",
        BUF_LEN - 1,
        &|| unsafe {
            esp32_hal::mem::memcpy(
                &(dst[0]) as *const _ as *mut _,
                &(src[0]) as *const _ as *mut _,
                BUF_LEN - 1,
            );
        },
    );

    writeln!(uart0, "Result: {:?}", unsafe {
        esp32_hal::mem::memcmp(
            &(dst[0]) as *const _ as *mut _,
            &(src[0]) as *const _ as *mut _,
            BUF_LEN - 1,
        )
    })
    .unwrap();
    unsafe {
        esp32_hal::mem::memset(&(dst[0]) as *const _ as *mut _, 0, BUF_LEN);
    }

    time(
        &mut uart0,
        "memcpy aligned src 4 bytes",
        BUF_LEN,
        &|| unsafe {
            esp32_hal::mem::memcpy(
                &(dst[1]) as *const _ as *mut _,
                &(src[0]) as *const _ as *mut _,
                BUF_LEN - 1,
            );
        },
    );

    writeln!(uart0, "Result: {:?}", unsafe {
        esp32_hal::mem::memcmp(
            &(dst[1]) as *const _ as *mut _,
            &(src[0]) as *const _ as *mut _,
            BUF_LEN - 1,
        )
    })
    .unwrap();
    unsafe {
        esp32_hal::mem::memset(&(dst[0]) as *const _ as *mut _, 0, BUF_LEN);
    }

    time(
        &mut uart0,
        "memcpy aligned dst 4 bytes",
        BUF_LEN,
        &|| unsafe {
            esp32_hal::mem::memcpy(
                &(dst[0]) as *const _ as *mut _,
                &(src[1]) as *const _ as *mut _,
                BUF_LEN - 1,
            );
        },
    );

    writeln!(uart0, "Result: {:?}", unsafe {
        esp32_hal::mem::memcmp(
            &(dst[0]) as *const _ as *mut _,
            &(src[1]) as *const _ as *mut _,
            BUF_LEN - 1,
        )
    })
    .unwrap();
    unsafe {
        esp32_hal::mem::memset(&(dst[0]) as *const _ as *mut _, 0, BUF_LEN);
    }

    loop {
        sleep(1.s());
        writeln!(uart0, "Alive and waiting for watchdog reset").unwrap();
    }
}

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

fn disable_timg_wdts(timg0: &mut esp32::TIMG0, timg1: &mut esp32::TIMG1) {
    timg0
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });
    timg1
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });

    timg0.wdtconfig0.write(|w| unsafe { w.bits(0x0) });
    timg1.wdtconfig0.write(|w| unsafe { w.bits(0x0) });
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!(
        "Error allocating  {} bytes of memory with alignment {}",
        layout.size(),
        layout.align()
    );
}
