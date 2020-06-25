#![no_std]
#![no_main]

extern crate esp32_hal as hal;
extern crate xtensa_lx6_rt;

use {
    core::panic::PanicInfo,
    embedded_graphics::{
        pixelcolor::BinaryColor, prelude::*, primitives::Circle, primitives::Rectangle,
        style::PrimitiveStyle, style::PrimitiveStyleBuilder,
    },
    hal::{
        clock_control::{self, sleep, CPUSource, ClockControl, ClockControlConfig},
        dport::Split,
        dprintln, i2c,
        prelude::*,
        timer::Timer,
    },
    ssd1306::{prelude::*, Builder},
};

#[no_mangle]
fn main() -> ! {
    let dp = esp32::Peripherals::take().unwrap();

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    // setup clocks & watchdog
    let mut clkcntrl = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    // set desired clock frequencies
    clkcntrl
        .set_cpu_frequencies(
            CPUSource::PLL,
            80.MHz(),
            CPUSource::PLL,
            240.MHz(),
            CPUSource::PLL,
            80.MHz(),
        )
        .unwrap();

    let (clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();
    let (_, _, _, mut watchdog0) = Timer::new(dp.TIMG0, clkcntrl_config);
    watchdog0.disable();
    let (_, _, _, mut watchdog1) = Timer::new(dp.TIMG1, clkcntrl_config);
    watchdog1.disable();

    let pins = dp.GPIO.split();

    let i2c0 = i2c::I2C::new(
        dp.I2C0,
        i2c::Pins {
            sda: pins.gpio4,
            scl: pins.gpio15,
        },
        400_000,
        &mut dport,
    );

    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c0).into();

    let mut rst = pins.gpio16.into_push_pull_output();
    rst.set_low().unwrap();
    sleep(10.ms());
    rst.set_high().unwrap();

    display.init().unwrap();

    loop {
        display.clear();
        Rectangle::new(Point::new(16, 24), Point::new(48, 40))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(BinaryColor::On)
                    .build(),
            )
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();
        sleep(500.ms());

        display.clear();
        Circle::new(Point::new(96, 32), 20)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();
        sleep(500.ms());
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // park the other core
    unsafe { ClockControlConfig {}.park_core(esp32_hal::get_other_core()) };

    // print panic message
    dprintln!("\n\n*** {:?}", info);

    // park this core
    unsafe { ClockControlConfig {}.park_core(esp32_hal::get_core()) };

    dprintln!("Not reached because core is parked.");

    // this statement will not be reached, but is needed to make this a diverging function
    loop {}
}
