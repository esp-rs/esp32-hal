#![no_std]
#![no_main]

use esp32_hal as hal;

use {
    core::panic::PanicInfo,
    embedded_graphics::{
        pixelcolor::BinaryColor, prelude::*, primitives::Circle, primitives::Rectangle,
        style::PrimitiveStyle, style::PrimitiveStyleBuilder,
    },
    embedded_hal::blocking::delay::{DelayMs, DelayUs},
    hal::{
        clock_control::{self, sleep, CPUSource, ClockControl, ClockControlConfig},
        dport::Split,
        dprintln, i2c,
        prelude::*,
        timer::Timer,
    },
    // mpu6050::Mpu6050,
    ssd1306::{prelude::*, Builder},
};

#[entry]
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

    // disable RTC watchdog
    let (clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();

    // disable MST watchdogs
    let (.., mut watchdog0) = Timer::new(dp.TIMG0, clkcntrl_config);
    let (.., mut watchdog1) = Timer::new(dp.TIMG1, clkcntrl_config);
    watchdog0.disable();
    watchdog1.disable();

    let pins = dp.GPIO.split();

    // Display
    let mut display = {
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
        display.clear();
        display.flush().unwrap();

        display
    };

    /*// IMU
    let mut imu = {
        let i2c1 = i2c::I2C::new(
            dp.I2C1,
            i2c::Pins {
                sda: pins.gpio22,
                scl: pins.gpio23,
            },
            200_000,
            &mut dport,
        );

        let mut imu = Mpu6050::new(i2c1, Delay);

        imu.verify().unwrap();

        imu.init().unwrap();
        imu.soft_calib(mpu6050::Steps(100)).unwrap();
        imu.calc_variance(mpu6050::Steps(50)).unwrap();

        imu
    };

    let temp = imu.get_temp().unwrap();
    let gyro = imu.get_gyro().unwrap();
    let acc = imu.get_acc().unwrap();
    dprintln!("temp: {}, gyro: {:?}, acc: {:?}", temp, gyro, acc);*/

    // let mut sensor = {
    //     let i2c1 = i2c::I2C::new(
    //         dp.I2C1,
    //         i2c::Pins {
    //             sda: pins.gpio22,
    //             scl: pins.gpio23,
    //         },
    //         200_000,
    //         &mut dport,
    //     );
    //
    //     let mut sensor = sgp30::Sgp30::new(i2c1, 0x58, Delay);
    //
    //     dprintln!("serial: {:?}", sensor.serial().unwrap());
    //
    //     sensor
    // };

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

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        sleep((ms as u32).ms());
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        sleep((ms as u32).ms());
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        sleep((us as u32).us());
    }
}

struct Delay;
