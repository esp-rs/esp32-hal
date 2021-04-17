#![no_std]
#![no_main]

use core::panic::PanicInfo;
use embedded_graphics::{
    fonts::{Font8x16, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyle,
};
use embedded_hal::blocking::i2c::{Write, WriteRead};
use esp32_hal::{
    clock_control::{self, sleep, CPUSource, ClockControl},
    delay::Delay,
    dport::Split,
    dprintln,
    i2c::{self, Error, I2C},
    prelude::*,
    target::{I2C0, Peripherals},
    timer::Timer,
};
use mpu6050::Mpu6050;
use ssd1306::{prelude::*, Builder};
use xtensa_lx::mutex::SpinLockMutex;

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

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
    let i2c0 = i2c::I2C::new(
        dp.I2C0,
        i2c::Pins {
            sda: pins.gpio4,
            scl: pins.gpio15,
        },
        400_000,
        &mut dport,
    );
    let i2c0 = SpinLockMutex::new(i2c0);

    // Display
    let mut display = {
        let i2c_wrapper = I2CWrapper::new(&i2c0);
        let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c_wrapper).into();

        let mut rst = pins.gpio16.into_push_pull_output();
        rst.set_low().unwrap();
        sleep(10.ms());
        rst.set_high().unwrap();

        display.init().unwrap();
        display.clear();
        display.flush().unwrap();

        display
    };

    // IMU
    let mut imu = {
        let i2c_wrapper = I2CWrapper::new(&i2c0);
        let mut imu = Mpu6050::new(i2c_wrapper);

        let mut delay = Delay::new();
        imu.init(&mut delay).unwrap();
        imu
    };

    Text::new("Hello world!", Point::new(2, 28))
        .into_styled(TextStyle::new(Font8x16, BinaryColor::On))
        .draw(&mut display)
        .unwrap();
    display.flush().unwrap();

    sleep(3.s());

    loop {
        let temp = imu.get_temp().unwrap();
        let gyro = imu.get_gyro().unwrap();
        let acc = imu.get_acc().unwrap();
        dprintln!("temp: {}, gyro: {:?}, acc: {:?}", temp, gyro, acc);
        sleep(1.s());
    }
}

struct I2CWrapper<'a> {
    i2c: &'a SpinLockMutex<I2C<I2C0>>,
}

impl<'a> I2CWrapper<'a> {
    fn new(i2c: &'a SpinLockMutex<I2C<I2C0>>) -> Self {
        Self { i2c }
    }
}

impl<'a> Write for I2CWrapper<'a> {
    type Error = Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.i2c.lock(|x| x.write(addr, bytes))
    }
}

impl<'a> WriteRead for I2CWrapper<'a> {
    type Error = Error;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.lock(|x| x.write_read(address, bytes, buffer))
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("----- PANIC -----");
    dprintln!("{:?}", info);
    loop {}
}
