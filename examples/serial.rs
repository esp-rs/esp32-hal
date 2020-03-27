#![no_std]
#![no_main]
#![feature(asm)]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::clock_control::sleep;
use esp32_hal::dport::Split;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

const BLINK_HZ: Hertz = Hertz(2);

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timer on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let clkcntrl =
        esp32_hal::clock_control::ClockControl::new(dp.RTCCNTL, dp.APB_CTRL, dport_clock_control)
            .unwrap();

    let (clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();

    let gpios = dp.GPIO.split();
    let mut blinky = gpios.gpio13.into_push_pull_output();

    let serial = Serial::uart0(
        dp.UART0,
        (NoTx, NoRx),
        Config::default(),
        clkcntrl_config,
        &mut dport,
    )
    .unwrap();

    let (mut tx, mut rx) = serial.split();

    writeln!(tx, "\n\nESP32 Started\n\n").unwrap();

    loop {
        writeln!(tx, "Characters received:  {:?}", rx.count()).unwrap();

        while let Ok(x) = rx.read() {
            write!(tx, "{} ({:#x}) ", if x >= 32 { x as char } else { '?' }, x).unwrap()
        }
        writeln!(tx, "").unwrap();

        blinky.set_high().unwrap();
        sleep((Hertz(1_000_000) / BLINK_HZ).us());
        blinky.set_low().unwrap();
        sleep((Hertz(1_000_000) / BLINK_HZ).us());
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

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
