#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(raw_vec_internals)]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use alloc::raw_vec::RawVec;
#[cfg(feature = "external_ram")]
use esp32_hal::alloc::EXTERNAL_ALLOCATOR;
use esp32_hal::alloc::{
    Allocator, AllocatorSize, DEFAULT_ALLOCATOR, DRAM_ALLOCATOR, IRAM_ALLOCATOR,
};

use esp32_hal::clock_control::{sleep, ClockControl};
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

#[macro_use]
extern crate alloc;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: Allocator = DEFAULT_ALLOCATOR;

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

    unsafe {
        print_heap_info(&mut uart0);

        let global_initialized_vec_unique = vec![1, 2, 3];
        print_info!(uart0, global_initialized_vec_unique[0]);

        let global_initialized_vec_same = vec![0x23u8; 12];
        print_info!(uart0, global_initialized_vec_same[0]);

        let dram_rawvec: RawVec<u8, _> = RawVec::with_capacity_in(50, DRAM_ALLOCATOR);
        print_info!(uart0, *dram_rawvec.ptr());

        let iram_rawvec: RawVec<u8, _> = RawVec::with_capacity_in(50, IRAM_ALLOCATOR);
        print_info!(uart0, *iram_rawvec.ptr());

        #[cfg(feature = "external_ram")]
        {
            writeln!(
                uart0,
                "\nExternal RAM size: {}\n",
                esp32_hal::external_ram::get_size()
            )
            .unwrap();

            let global_initialized_vec_large = vec![0u8; 1024 * 1024];
            print_info!(uart0, global_initialized_vec_large[0]);

            let external_ram_rawvec: RawVec<u8, _> =
                RawVec::with_capacity_in(50, EXTERNAL_ALLOCATOR);
            print_info!(uart0, *external_ram_rawvec.ptr());

            print_heap_info(&mut uart0);
        }
        #[cfg(not(feature = "external_ram"))]
        {
            print_heap_info(&mut uart0);
        }
    }

    loop {
        sleep(1.s());
        writeln!(uart0, "Alive and waiting for watchdog reset").unwrap();
    }
}

fn print_single_heap_info(output: &mut dyn core::fmt::Write, allocator: &Allocator, text: &str) {
    writeln!(
        output,
        "{:>15}: free {:>8.3}KB, used {:>8.3}KB out of {:>8.3}KB",
        text,
        allocator.free() as f32 / 1024.0,
        allocator.used() as f32 / 1024.0,
        allocator.size() as f32 / 1024.0
    )
    .unwrap();
}

fn print_heap_info(output: &mut dyn core::fmt::Write) {
    writeln!(output).unwrap();
    print_single_heap_info(output, &GLOBAL_ALLOCATOR, "Global");
    print_single_heap_info(output, &DRAM_ALLOCATOR, "DRAM");
    print_single_heap_info(output, &IRAM_ALLOCATOR, "IRAM");
    #[cfg(feature = "external_ram")]
    print_single_heap_info(output, &EXTERNAL_ALLOCATOR, "External RAM");
    writeln!(output).unwrap();
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
