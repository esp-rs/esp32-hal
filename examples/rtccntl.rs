#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use embedded_hal::watchdog::*;
use esp32;
use esp32_hal::clock_control::*;
use esp32_hal::dport::Split;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};
use esp32_hal::units::*;

const BLINK_HZ: Hertz = Hertz(1);

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

pub struct Context {
    watchdog: esp32_hal::clock_control::watchdog::WatchDog,
    /*  uart0: esp32_hal::serial::Serial<
        'a,
        esp32::UART0,
        (esp32_hal::serial::NoTx, esp32_hal::serial::NoRx),
    >,*/
    rx: esp32_hal::serial::Rx<esp32::UART0>,
    tx: esp32_hal::serial::Tx<esp32::UART0>,
}

pub static GLOBAL_CONTEXT: spin::Mutex<Option<Context>> = spin::Mutex::new(None);

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    let (_dport, dport_clock_control) = dp.DPORT.split();

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timers on halt
    // we will do it manually on startup
    //    disable_timg_wdts(&mut timg0, &mut timg1);

    disable_timg_wdts(&mut timg0, &mut timg1);

    let mut clock_control = ClockControl::new(dp.RTCCNTL, dp.APB_CTRL, dport_clock_control);

    clock_control.set_cpu_frequency_to_pll(240.MHz()).unwrap();

    let (clock_control_config, mut watchdog) = clock_control.freeze().unwrap();
    watchdog.start(3.s());

    let mut uart0 = Serial::uart0(
        dp.UART0,
        (NoTx, NoRx),
        Config::default(),
        clock_control_config,
    )
    .unwrap();

    uart0.change_baudrate(115200).unwrap();

    let (mut tx, rx) = uart0.split();

    writeln!(tx, "\n\nReboot!\n").unwrap();

    writeln!(tx, "Running on core {:0x}\n", xtensa_lx6_rt::get_core_id()).unwrap();
    writeln!(tx, "{:?}\n", clock_control_config).unwrap();
    writeln!(tx, "{:?}\n", watchdog.config().unwrap()).unwrap();

    *GLOBAL_CONTEXT.lock() = Some(Context { watchdog, rx, tx });

    // panic!("panic test");

    let mut x = 1;
    let mut prev_ccount = 0;

    loop {
        let ccount = xtensa_lx6_rt::get_cycle_count();
        writeln!(
            GLOBAL_CONTEXT.lock().as_mut().unwrap().tx,
            "Loop: {}, CCOUNT: {}, {}",
            x,
            ccount,
            ccount - prev_ccount
        )
        .unwrap();

        prev_ccount = ccount;
        x += 1;

        delay((Hertz(1_000_000) / BLINK_HZ).us());

        // comment out next line to check watchdog behavior
        GLOBAL_CONTEXT.lock().as_mut().unwrap().watchdog.feed();
    }
}

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

/// panic handler using static tx
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    writeln!(
        GLOBAL_CONTEXT.lock().as_mut().unwrap().tx,
        "\n\n*** {:?}",
        info
    )
    .unwrap();
    loop {}
}
