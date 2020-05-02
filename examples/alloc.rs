#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::alloc::{Allocator, DEFAULT_ALLOCATOR};
use esp32_hal::clock_control::{sleep, ClockControl};
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

#[macro_use]
extern crate alloc;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: Allocator = DEFAULT_ALLOCATOR;

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

    watchdog.start(15.s());

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

    crate::dprintln!("External RAM size: {}", unsafe {
        esp32_hal::external_ram::get_size()
    });

    let mut vec = vec![1, 2, 3];

    for x in 0..vec.len() {
        writeln!(
            uart0,
            "vec: address: {:08x?} {}",
            &vec[x] as *const _ as usize, vec[x]
        )
        .unwrap();
    }

    vec = vec![0xabu8; 12];

    for x in 0..vec.len() {
        writeln!(
            uart0,
            "vec: address: {:08x?} {}",
            &vec[x] as *const _ as usize, vec[x]
        )
        .unwrap();
    }

    let too_large_for_dram = vec![0u8; 1024 * 1024];

    writeln!(
        uart0,
        "vec: address: {:08x?} ",
        &too_large_for_dram[0] as *const _ as usize
    )
    .unwrap();

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
