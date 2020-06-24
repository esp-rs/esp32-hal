#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::analog::adc::ADC;
use esp32_hal::analog::config::{Adc1Config, Adc2Config, Attenuation};
use esp32_hal::clock_control::sleep;
use esp32_hal::dport::Split;
use esp32_hal::serial::{config::Config, Serial};
use esp32_hal::target;

#[no_mangle]
fn main() -> ! {
    let dp = target::Peripherals::take().expect("Failed to obtain Peripherals");

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timer on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let clkcntrl = esp32_hal::clock_control::ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    let (clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();

    let gpios = dp.GPIO.split();

    let serial: Serial<_, _, _> = Serial::new(
        dp.UART0,
        esp32_hal::serial::Pins {
            tx: gpios.gpio1,
            rx: gpios.gpio3,
            cts: None,
            rts: None,
        },
        Config::default(),
        clkcntrl_config,
        &mut dport,
    )
    .unwrap();

    let (mut tx, _rx) = serial.split();

    /* Set ADC pins to analog mode */
    let mut pin36 = gpios.gpio36.into_analog();
    let mut pin25 = gpios.gpio25.into_analog();

    /* Prepare ADC configs and enable pins, which will be used */
    let mut adc1_config = Adc1Config::new();
    adc1_config.enable_pin(&pin36, Attenuation::Attenuation11dB);

    let mut adc2_config = Adc2Config::new();
    adc2_config.enable_pin(&pin25, Attenuation::Attenuation11dB);

    /* Create ADC instances */
    let analog = dp.SENS.split();
    let mut adc1 = ADC::adc1(analog.adc1, adc1_config).unwrap();
    let mut adc2 = ADC::adc2(analog.adc2, adc2_config).unwrap();

    loop {
        /* Read ADC values every second and print them out */
        let pin36_value: u16 = nb::block!(adc1.read(&mut pin36)).unwrap();
        writeln!(tx, "ADC1 pin 36 raw value: {:?}", pin36_value).unwrap();

        let pin25_value: u16 = nb::block!(adc2.read(&mut pin25)).unwrap();
        writeln!(tx, "ADC2 pin 25 raw value: {:?}", pin25_value).unwrap();

        sleep(1.s());
    }
}

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

fn disable_timg_wdts(timg0: &mut target::TIMG0, timg1: &mut target::TIMG1) {
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
