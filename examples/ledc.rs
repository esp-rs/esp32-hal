#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};
use esp32_hal::{
    clock_control::{self, ClockControl},
    dport::Split,
    dprintln,
    ledc::{
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
        LSGlobalClkSource, LowSpeed, LEDC,
    },
    prelude::*,
    serial::{self, Serial},
    target::Peripherals,
    timer::Timer,
};

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().expect("Failed to obtain peripherals");

    let (_, dport_clock_control) = dp.DPORT.split();
    let clock_control = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    let (clock_control_config, mut watchdog) = clock_control.freeze().unwrap();
    watchdog.disable();

    let (.., mut watchdog0) = Timer::new(dp.TIMG0, clock_control_config);
    let (.., mut watchdog1) = Timer::new(dp.TIMG1, clock_control_config);
    watchdog0.disable();
    watchdog1.disable();

    let pins = dp.GPIO.split();

    let mut serial: Serial<_, _, _> = Serial::new(
        dp.UART0,
        serial::Pins {
            tx: pins.gpio1,
            rx: pins.gpio3,
            cts: None,
            rts: None,
        },
        serial::config::Config {
            baudrate: 115200.Hz(),
            ..serial::config::Config::default()
        },
        clock_control_config,
    )
    .unwrap();

    writeln!(serial, "\nESP32 Started\n\n").unwrap();

    let mut ledc = LEDC::new(clock_control_config);
    ledc.set_global_slow_clock(LSGlobalClkSource::ABPClk);
    let mut lstimer0 = ledc.get_timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty1Bit,
            clock_source: timer::LSClockSource::SlowClk,
            frequency: 24_000_000.Hz(),
        })
        .unwrap();

    let mut channel0 = ledc.get_channel(channel::Number::Channel0, pins.gpio4);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0.5,
        })
        .unwrap();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}
