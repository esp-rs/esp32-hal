#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::clock_control::{sleep, CPUSource, ClockControl, ClockControlConfig};
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};
const BLINK_HZ: Hertz = Hertz(1);

static GLOBAL_COUNT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
static TX: spin::Mutex<Option<esp32_hal::serial::Tx<esp32::UART0>>> = spin::Mutex::new(None);

#[no_mangle]
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
    let mut clock_control = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    // set desired clock frequencies
    clock_control
        .set_cpu_frequencies(
            CPUSource::Xtal,
            10.MHz(),
            CPUSource::PLL,
            240.MHz(),
            CPUSource::PLL,
            80.MHz(),
        )
        .unwrap();

    let (mut clock_control_config, mut watchdog) = clock_control.freeze().unwrap();

    watchdog.start(3.s());

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

    writeln!(
        uart0,
        "UART0 baudrate: {}, using apb clock instead of ref clock: {}\n",
        uart0.baudrate(),
        uart0.is_clock_apb()
    )
    .unwrap();

    // register callback which is called when the clock is switched
    clock_control_config
        .add_callback(&|| {
            let clock_control_config = ClockControlConfig {};
            dprintln!(
                "  Change Clock: CPU: {}, PLL: {}, APB: {}, REF: {}",
                clock_control_config.cpu_frequency(),
                clock_control_config.pll_frequency(),
                clock_control_config.apb_frequency(),
                clock_control_config.ref_frequency(),
            )
        })
        .unwrap();

    // uncomment next line to test panic exit
    // panic!("panic test");

    let (tx, _) = uart0.split();
    *TX.lock() = Some(tx);

    // start core 1 (APP_CPU)
    clock_control_config.start_core(1, cpu1_start).unwrap();

    // main loop, which in turn lock and unlocks apb and cpu locks
    let mut x: u32 = 0;
    let mut prev_ccount = 0;
    loop {
        for j in 0..2 {
            let apb_guard = if j == 1 {
                Some(clock_control_config.lock_apb_frequency())
            } else {
                None
            };

            for i in 0..2 {
                let cpu_guard = if i == 1 {
                    Some(clock_control_config.lock_cpu_frequency())
                } else {
                    None
                };

                x = x.wrapping_add(1);

                print_info(x, &mut prev_ccount);

                sleep((Hertz(1_000_000) / BLINK_HZ).us());

                // comment out next line to check watchdog behavior
                watchdog.feed();

                if cpu_guard.is_some() {
                    drop(cpu_guard.unwrap())
                }
            }
            if apb_guard.is_some() {
                drop(apb_guard.unwrap())
            }
        }
    }
}

fn cpu1_start() -> ! {
    let mut x: u32 = 0;
    let mut prev_ccount = 0;

    loop {
        print_info(x, &mut prev_ccount);
        sleep((Hertz(1_100_000) / BLINK_HZ).us());
        x = x.wrapping_add(1);
    }
}

fn print_info(loop_count: u32, prev_ccount: &mut u32) {
    let ccount = xtensa_lx6_rt::get_cycle_count();
    let ccount_diff = ccount.wrapping_sub(*prev_ccount);

    let total = GLOBAL_COUNT.fetch_add(ccount_diff, core::sync::atomic::Ordering::Relaxed);

    writeln!(
        TX.lock().as_mut().unwrap(),
        "Core: {}, Loop: {}, cycles: {}, cycles since previous {}, Total cycles: {}",
        xtensa_lx6_rt::get_core_id(),
        loop_count,
        ccount,
        ccount_diff,
        total
    )
    .unwrap();

    *prev_ccount = ccount;
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
    // park the other core
    unsafe {
        ClockControlConfig {}
            .park_core(1 - xtensa_lx6_rt::get_core_id())
            .ok()
    };

    // print panic message
    dprintln!("\n\n*** {:?}", info);

    // park this core
    unsafe {
        ClockControlConfig {}
            .park_core(xtensa_lx6_rt::get_core_id())
            .ok()
    };

    dprintln!("Not reached because core is parked.");

    // this statement will not be reached, but is needed to make this a diverging function
    loop {}
}
