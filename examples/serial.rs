#![no_std]
#![no_main]
#![feature(asm)]

use xtensa_lx6_rt as _;

use core::fmt::Write;
use core::panic::PanicInfo;
use esp32;
use esp32_hal::gpio::GpioExt;
use esp32_hal::hal::digital::v2::OutputPin;

use esp32_hal::hal::serial::Read as _;

use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

/// The default clock source is the onboard crystal
/// In most cases 40mhz (but can be as low as 2mhz depending on the board)
const CORE_HZ: u32 = 40_000_000;

const BLINK_HZ: u32 = CORE_HZ / 1;

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut rtccntl = dp.RTCCNTL;
    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the wdt's on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);
    disable_rtc_wdt(&mut rtccntl);

    let gpios = dp.GPIO.split();
    let mut blinky = gpios.gpio13.into_push_pull_output();

    let serial = Serial::uart0(dp.UART0, (NoTx, NoRx), Config::default()).unwrap();
    let baudrate = serial.get_baudrate();

    let (mut tx, mut rx) = serial.split();
    writeln!(tx,"baudrate {:?}",baudrate).unwrap();

    loop {
        writeln!(tx,"Characters received:  {:?}",rx.count()).unwrap();

        while let Ok(x) = rx.read() {
            write!(tx,"{} ({:#x}) ", if x >= 32 { x as char } else { '?' }, x).unwrap()
        }
        writeln!(tx,"").unwrap();

        blinky.set_high().unwrap();
        delay(BLINK_HZ);
        blinky.set_low().unwrap();
        delay(BLINK_HZ);
    }
}

fn disable_rtc_wdt(rtccntl: &mut esp32::RTCCNTL) {
    /* Disables the RTCWDT */
    rtccntl
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });
    rtccntl.wdtconfig0.modify(|_, w| unsafe {
        w.wdt_stg0()
            .bits(0x0)
            .wdt_stg1()
            .bits(0x0)
            .wdt_stg2()
            .bits(0x0)
            .wdt_stg3()
            .bits(0x0)
            .wdt_flashboot_mod_en()
            .clear_bit()
            .wdt_en()
            .clear_bit()
    });
    rtccntl.wdtwprotect.write(|w| unsafe { w.bits(0x0) });
}

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

/// rough delay - as a guess divide your cycles by 20 (results will differ on opt level)
pub fn delay2(clocks: u32) {
    let dummy_var: u32 = 0;
    for _ in 0..clocks {
        unsafe { core::ptr::read_volatile(&dummy_var) };
    }
}

/// cycle accurate delay using the cycle counter register
pub fn delay(clocks: u32) {
    let start = get_ccount();
    loop {
        if get_ccount().wrapping_sub(start) >= clocks {
            break;
        }
    }
}

/// Performs a special register read to read the current cycle count.
/// In the future, this can be precompiled to a archive (.a) and linked to so we don't
/// have to require the asm nightly feature - see cortex-m-rt for more details
pub fn get_ccount() -> u32 {
    let x: u32;
    unsafe { asm!("rsr.ccount a2" : "={a2}"(x) ) };
    x
}

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        /*        blinky.set_high().unwrap();
        delay(CORE_HZ/10);
        blinky.set_low().unwrap();
        delay(CORE_HZ/10);
        */
    }
}
