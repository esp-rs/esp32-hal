#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::clock_control::{delay, ClockControl};
use esp32_hal::dport::Split;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

const BLINK_HZ: Hertz = Hertz(1);

struct Context {
    watchdog: esp32_hal::clock_control::watchdog::WatchDog,
    rx: esp32_hal::serial::Rx<esp32::UART0>,
    tx: esp32_hal::serial::Tx<esp32::UART0>,
}

// TODO: replace spinning mutex as it is not thread or interrupt safe
static GLOBAL_CONTEXT: spin::Mutex<Option<Context>> = spin::Mutex::new(None);

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timers on halt
    // we will do it manually on startup
    //    disable_timg_wdts(&mut timg0, &mut timg1);

    disable_timg_wdts(&mut timg0, &mut timg1);

    // setup clocks & watchdog
    let mut clock_control = ClockControl::new(dp.RTCCNTL, dp.APB_CTRL, dport_clock_control);
    clock_control.set_cpu_frequency_to_pll(240.MHz()).unwrap();
    let (clock_control_config, mut watchdog) = clock_control.freeze().unwrap();
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

    let (mut tx, rx) = uart0.split();

    // print startup message
    writeln!(tx, "\n\nReboot!\n").unwrap();

    writeln!(tx, "Running on core {:0x}\n", xtensa_lx6_rt::get_core_id()).unwrap();
    writeln!(tx, "{:?}\n", clock_control_config).unwrap();
    writeln!(tx, "{:?}\n", watchdog.config().unwrap()).unwrap();

    // move to global context to allow access in panic handler
    *GLOBAL_CONTEXT.lock() = Some(Context { watchdog, rx, tx });

    // uncomment next line to test panic exit
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
