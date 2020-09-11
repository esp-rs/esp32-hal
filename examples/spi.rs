//! Example of using the SPI interface using the ESP32 WROVER DEVKIT
//!
//! This examples writes to the display. As this pushes many pixels it is quite slow in debug mode,
//! so please run it in release mode to get an impression of the obtainable speed.

#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};

use esp32_hal::{
    clock_control::{sleep, ClockControl, XTAL_FREQUENCY_AUTO},
    dport::Split,
    dprintln,
    gpio::{InputPin, OutputPin},
    prelude::*,
    serial::{self, Serial},
    spi::{self, SPI},
    target,
    timer::Timer,
};

use embedded_hal::blocking::spi::WriteIter;

use ili9341;

use embedded_graphics::{
    fonts::{Font12x16, Text},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Rectangle},
    style::{PrimitiveStyleBuilder, TextStyle},
};

// Interface for ili9341 driver
// ili9341 uses separate command/data pin, this interface set this pin to the appropriate state
struct SPIInterface<
    CMD: embedded_hal::digital::v2::OutputPin,
    SCLK: OutputPin,
    SDO: OutputPin,
    SDI: InputPin + OutputPin,
    CS: OutputPin,
> {
    spi: SPI<esp32::SPI2, SCLK, SDO, SDI, CS>,
    cmd: CMD,
}

impl<
        CMD: embedded_hal::digital::v2::OutputPin,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > ili9341::Interface for SPIInterface<CMD, SCLK, SDO, SDI, CS>
{
    type Error = esp32_hal::spi::Error;

    fn write(&mut self, command: u8, data: &[u8]) -> Result<(), Self::Error> {
        self.cmd
            .set_low()
            .map_err(|_| esp32_hal::spi::Error::PinError)?;
        self.spi.write(&[command])?;
        self.cmd
            .set_high()
            .map_err(|_| esp32_hal::spi::Error::PinError)?;
        self.spi.write(data)?;
        Ok(())
    }

    fn write_iter(
        &mut self,
        command: u8,
        data: impl IntoIterator<Item = u16>,
    ) -> Result<(), Self::Error> {
        self.cmd
            .set_low()
            .map_err(|_| esp32_hal::spi::Error::PinError)?;
        self.spi.write(&[command])?;
        self.cmd
            .set_high()
            .map_err(|_| esp32_hal::spi::Error::PinError)?;
        self.spi.write_iter(data)?;
        Ok(())
    }
}

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
    let (_, _, _, mut watchdog0) = Timer::new(dp.TIMG0, clkcntrl_config);
    let (_, _, _, mut watchdog1) = Timer::new(dp.TIMG1, clkcntrl_config);

    watchdog.disable();
    watchdog0.disable();
    watchdog1.disable();

    let _lock = clkcntrl_config.lock_cpu_frequency();

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
        clkcntrl_config,
        &mut dport,
    )
    .unwrap();

    // Official ili9341 spec is 10MHz, but overdrive up to 80MHz actually works.
    // 26MHz chosen here: will be 26MHz when using 26MHz crystal, 20MHz when using 40MHz crystal,
    // due to integer clock division.
    // Faster is no use as the cpu is not keeping up with the embedded_graphics library.
    let spi: SPI<_, _, _, _, _> = SPI::<esp32::SPI2, _, _, _, _>::new(
        dp.SPI2,
        spi::Pins {
            sclk: pins.gpio19,
            sdo: pins.gpio23,
            sdi: Some(pins.gpio25),
            cs: Some(pins.gpio22),
        },
        spi::config::Config {
            baudrate: 26.MHz().into(),
            bit_order: spi::config::BitOrder::MSBFirst,
            data_mode: spi::config::MODE_0,
        },
        clkcntrl_config,
        &mut dport,
    )
    .unwrap();

    let mut gpio_backlight = pins.gpio5.into_push_pull_output();
    let mut gpio_reset = pins.gpio18.into_push_pull_output();
    let gpio_cmd = pins.gpio21.into_push_pull_output();

    gpio_reset.set_low().unwrap();
    sleep(100.ms());
    gpio_reset.set_high().unwrap();
    sleep(100.ms());

    gpio_backlight.set_low().unwrap();

    let spi_if = SPIInterface { spi, cmd: gpio_cmd };

    let mut display =
        ili9341::Ili9341::new(spi_if, gpio_reset, &mut esp32_hal::delay::Delay::new()).unwrap();

    display
        .set_orientation(ili9341::Orientation::Landscape)
        .unwrap();

    Rectangle::new(Point::new(0, 0), Point::new(320, 240))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::WHITE)
                .stroke_width(4)
                .stroke_color(Rgb565::BLUE)
                .build(),
        )
        .draw(&mut display)
        .unwrap();

    let rect = Rectangle::new(Point::new(10, 80), Point::new(30, 100)).into_styled(
        PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::RED)
            .stroke_width(1)
            .stroke_color(Rgb565::WHITE)
            .build(),
    );

    let circle = Circle::new(Point::new(20, 50), 10).into_styled(
        PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::GREEN)
            .stroke_width(1)
            .stroke_color(Rgb565::WHITE)
            .build(),
    );

    Text::new("Hello Rust!", Point::new(20, 16))
        .into_styled(TextStyle::new(Font12x16, Rgb565::RED))
        .draw(&mut display)
        .unwrap();

    writeln!(serial, "\n\nESP32 Started\n\n").unwrap();

    loop {
        for x in (0..280).chain((0..280).rev()) {
            rect.translate(Point::new(x, 0)).draw(&mut display).unwrap();
        }

        for x in (0..280).chain((0..280).rev()) {
            circle
                .translate(Point::new(x, 0))
                .draw(&mut display)
                .unwrap();
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}
