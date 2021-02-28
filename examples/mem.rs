#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(raw_vec_internals)]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::alloc::{Allocator, DRAM_ALLOCATOR};

use esp32_hal::clock_control::{sleep, CPUSource::PLL, ClockControl, ClockControlConfig};
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::mem::{memcmp, memcpy, memcpy_reverse, memset};
use esp32_hal::serial::{config::Config, Serial};
use esp32_hal::target;

use xtensa_lx::timer::get_cycle_count;

#[macro_use]
extern crate alloc;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: Allocator = DRAM_ALLOCATOR;

#[entry]
fn main() -> ! {
    let dp = target::Peripherals::take().expect("Failed to obtain Peripherals");

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timers on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let (_, dport_clock_control) = dp.DPORT.split();

    // setup clocks & watchdog
    let mut clock_control = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    // set desired clock frequencies
    clock_control
        .set_cpu_frequencies(PLL, 240.MHz(), PLL, 240.MHz(), PLL, 240.MHz())
        .unwrap();

    let (clock_control_config, mut watchdog) = clock_control.freeze().unwrap();

    watchdog.start(20.s());

    let gpios = dp.GPIO.split();

    // setup serial controller
    let mut uart0: Serial<_, _, _> = Serial::new(
        dp.UART0,
        esp32_hal::serial::Pins {
            tx: gpios.gpio1,
            rx: gpios.gpio3,
            cts: None,
            rts: None,
        },
        Config::default(),
        clock_control_config,
    )
    .unwrap();

    uart0.change_baudrate(115200).unwrap();

    // print startup message
    writeln!(uart0, "\n\nReboot!\n",).unwrap();

    const BUF_LEN: usize = 1024 * 128;

    writeln!(uart0, "Initializing").unwrap();

    let mut dst = vec![0u8; BUF_LEN];
    let mut src = vec![0u8; BUF_LEN];

    let start = get_cycle_count();
    for i in 0..src.len() {
        src[i] = i as u8;
    }
    let end = get_cycle_count();

    let inittime = end.wrapping_sub(start) as f32 / ClockControlConfig {}.cpu_frequency().0 as f32;

    writeln!(
        uart0,
        "{:>40}: {:.3}s, {:.3}KB/s",
        format!("initialized src: {}", src.len()),
        inittime,
        src.len() as f32 / inittime / 1024.0,
    )
    .unwrap();

    time(
        &mut uart0,
        "memset aligned, sized 4 bytes",
        BUF_LEN,
        &|| unsafe {
            memset(&(dst[0]) as *const _ as *mut _, 0, BUF_LEN);
        },
    );

    time(&mut uart0, "memset aligned", BUF_LEN, &|| unsafe {
        memset(&(dst[0]) as *const _ as *mut _, 0, BUF_LEN - 1);
    });

    time(&mut uart0, "memset", BUF_LEN, &|| unsafe {
        memset(&(dst[1]) as *const _ as *mut _, 0, BUF_LEN - 1);
    });

    let tx = &mut uart0;
    unsafe {
        for f in &[memcpy, memcpy_reverse] {
            time_memcpy(tx, &mut (dst[0]), &mut (src[0]), BUF_LEN, *f);
            time_memcpy(tx, &mut (dst[0]), &mut (src[0]), BUF_LEN - 1, *f);
            time_memcpy(tx, &mut (dst[1]), &mut (src[1]), BUF_LEN - 1, *f);
            time_memcpy(tx, &mut (dst[1]), &mut (src[0]), BUF_LEN - 1, *f);
            time_memcpy(tx, &mut (dst[0]), &mut (src[1]), BUF_LEN - 1, *f);
        }
    }

    loop {
        sleep(1.s());
        writeln!(uart0, "Alive and waiting for watchdog reset").unwrap();
    }
}

const REPEAT: usize = 20;

fn time(output: &mut dyn core::fmt::Write, text: &str, bytes: usize, f: &dyn Fn() -> ()) {
    let start = get_cycle_count();
    for _ in 0..REPEAT {
        f();
    }
    let end = get_cycle_count();

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

unsafe fn time_memcpy(
    output: &mut dyn core::fmt::Write,
    dst: &mut u8,
    src: &mut u8,
    len: usize,
    f: unsafe extern "C" fn(dst: *mut u8, src: *const u8, n: usize) -> *mut u8,
) {
    let start = get_cycle_count();
    for _ in 0..REPEAT {
        f(dst as *const _ as *mut _, src as *const _ as *mut _, len);
    }
    let end = get_cycle_count();

    let time = end.wrapping_sub(start) as f32 / ClockControlConfig {}.cpu_frequency().0 as f32;

    let cmp_res = memcmp(dst as *const _ as *mut _, src as *const _ as *mut _, len);

    writeln!(
        output,
        "{:>40}: {:.3}s, {:.3}KB/s, Result: {}",
        format!(
            "memcpy: {} {} {}",
            (dst as *const _ as usize) % core::mem::size_of::<usize>(),
            (src as *const _ as usize) % core::mem::size_of::<usize>(),
            len
        ),
        time,
        (len * REPEAT) as f32 / time / 1024.0,
        cmp_res
    )
    .unwrap();

    memset(dst as *const _ as *mut _, 0, len);
}

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

fn disable_timg_wdts(timg0: &mut target::TIMG0, timg1: &mut target::TIMG1) {
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
