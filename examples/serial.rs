#![no_std]
#![no_main]
#![feature(asm)]

use core::{fmt::Write, panic::PanicInfo};

use esp32_hal::{
    clock_control::{sleep, ClockControl, XTAL_FREQUENCY_AUTO},
    dport::Split,
    dprintln,
    prelude::*,
    serial::{config::Config, Pins, Serial},
    target,
    timer::Timer,
};

const BLINK_HZ: Hertz = Hertz(2);

#[entry]
fn main() -> ! {
    let dp = target::Peripherals::take().expect("Failed to obtain Peripherals");

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    let clkcntrl = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    let (clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();

    let (_, _, _, mut watchdog0) = Timer::new(dp.TIMG0, clkcntrl_config);
    let (_, _, _, mut watchdog1) = Timer::new(dp.TIMG1, clkcntrl_config);
    watchdog0.disable();
    watchdog1.disable();

    let pins = dp.GPIO.split();

    let mut blinky = pins.gpio0.into_push_pull_output();

    // Use UART1 as example: will cause dprintln statements not to be printed
    let serial: Serial<_, _, _> = Serial::new(
        dp.UART1,
        Pins {
            tx: pins.gpio1,
            rx: pins.gpio3,
            cts: None,
            rts: None,
        },
        Config {
            // default configuration is 19200 baud, 8 data bits, 1 stop bit & no parity (8N1)
            baudrate: 115200.Hz(),
            ..Config::default()
        },
        clkcntrl_config,
        &mut dport,
    )
    .unwrap();

    let (mut tx, mut rx) = serial.split();

    writeln!(tx, "\n\nESP32 Started\n\n").unwrap();

    // line will not be printed as using UART1
    dprintln!("UART0\n");

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

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
