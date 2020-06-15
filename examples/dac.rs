#![no_std]
#![no_main]

use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::analog::dac::DAC;
use esp32_hal::clock_control::sleep;
use esp32_hal::dport::Split;
use esp32_hal::target;

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { target::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    let (_dport, dport_clock_control) = dp.DPORT.split();

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

    let (_clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();

    /* Set DAC pins to analog mode. The pins are specified by hardware and cannot be changed */
    let gpios = dp.GPIO.split();
    let pin25 = gpios.gpio25.into_analog();
    let pin26 = gpios.gpio26.into_analog();

    /* Create DAC instances */
    let analog = dp.SENS.split();
    let mut dac1 = DAC::dac1(analog.dac1, pin25).unwrap();
    let mut dac2 = DAC::dac2(analog.dac2, pin26).unwrap();

    let mut voltage_dac1: u8 = 0;
    let mut voltage_dac2: u8 = 255;
    loop {
        /* Change voltage on the pins using write function */
        voltage_dac1 = voltage_dac1.wrapping_add(1);
        dac1.write(voltage_dac1);

        voltage_dac2 = voltage_dac2.wrapping_sub(1);
        dac2.write(voltage_dac2);

        sleep(250.ms());
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
