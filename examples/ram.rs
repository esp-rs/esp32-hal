#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::clock_control::{sleep, CPUSource, ClockControl, ClockControlConfig};
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};
const BLINK_HZ: Hertz = Hertz(1);

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timers on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    // setup clocks & watchdog
    let mut clock_control = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    unsafe { esp32_hal::ESP32PreInit() };

    let (clock_control_config, mut watchdog) = clock_control.freeze().unwrap();

    watchdog.start(2.s());

    // setup serial controller
    let mut uart0 = Serial::uart0(
        dp.UART0,
        (NoTx, NoRx),
        Config::default(),
        clock_control_config,
        &mut dport,
    )
    .unwrap();

    uart0.change_baudrate(115200).unwrap();

    // print startup message
    writeln!(uart0, "\n\nReboot!\n",).unwrap();

    writeln!(
        uart0,
        "Running on core {:0x}\n",
        xtensa_lx6_rt::get_core_id()
    )
    .unwrap();

    ram_tests(&mut uart0);

    loop {}
}

fn attr_none_fn(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    writeln!(
        uart,
        "attr_none_fn: {:0x}",
        xtensa_lx6_rt::get_program_counter()
    )
    .unwrap();
}

#[ram]
fn attr_ram_fn(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    writeln!(
        uart,
        "attr_ram_fn: {:0x}",
        xtensa_lx6_rt::get_program_counter()
    )
    .unwrap();
}

#[ram(rtc_slow)]
fn attr_ram_fn_rtc_slow(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    writeln!(
        uart,
        "attr_ram_fn_rtc_slow: {:0x}",
        xtensa_lx6_rt::get_program_counter()
    )
    .unwrap();
}

#[ram(rtc_fast)]
fn attr_ram_fn_rtc_fast(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    writeln!(
        uart,
        "attr_ram_fn_rtc_fast: {:0x}",
        xtensa_lx6_rt::get_program_counter()
    )
    .unwrap();
}

static ATTR_NONE_STATIC: [u8; 16] = *b"ATTR_NONE_STATIC";

#[ram]
static ATTR_RAM_STATIC: [u8; 15] = *b"ATTR_RAM_STATIC";

#[ram(rtc_slow)]
static ATTR_RAM_STATIC_RTC_SLOW: [u8; 24] = *b"ATTR_RAM_STATIC_RTC_SLOW";

#[ram(rtc_fast)]
static ATTR_RAM_STATIC_RTC_FAST: [u8; 24] = *b"ATTR_RAM_STATIC_RTC_FAST";

#[cfg(feature = "external_ram")]
#[ram(external)]
static mut ATTR_RAM_STATIC_EXTERNAL: [u8; 24] = *b"ATTR_RAM_STATIC_EXTERNAL";

#[cfg(feature = "external_ram")]
#[ram(external, zeroed)]
static mut ATTR_RAM_STATIC_EXTERNAL_BSS: [u8; 1024] = [0; 1024];

fn ram_tests(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    attr_none_fn(uart);
    attr_ram_fn(uart);
    attr_ram_fn_rtc_slow(uart);
    attr_ram_fn_rtc_fast(uart);

    writeln!(
        uart,
        "ATTR_NONE_STATIC: {:x}: {:02x?}",
        &ATTR_NONE_STATIC as *const u8 as usize, ATTR_NONE_STATIC
    )
    .unwrap();

    writeln!(
        uart,
        "ATTR_RAM_STATIC: {:x}: {:02x?}",
        &ATTR_RAM_STATIC as *const u8 as usize, ATTR_RAM_STATIC
    )
    .unwrap();

    writeln!(
        uart,
        "ATTR_RAM_STATIC_RTC_SLOW: {:x}: {:02x?}",
        &ATTR_RAM_STATIC_RTC_SLOW as *const u8 as usize, ATTR_RAM_STATIC_RTC_SLOW
    )
    .unwrap();

    writeln!(
        uart,
        "ATTR_RAM_STATIC_RTC_FAST: {:x}: {:02x?}",
        &ATTR_RAM_STATIC_RTC_FAST as *const u8 as usize, ATTR_RAM_STATIC_RTC_FAST
    )
    .unwrap();

    if cfg!(feature = "external_ram") {
        external_ram();
    }
}

#[cfg(not(feature = "external_ram"))]
fn external_ram() {}

#[cfg(feature = "external_ram")]
fn external_ram() {
    unsafe {
        writeln!(
            uart,
            "ATTR_RAM_STATIC_EXTERNAL: {:x}: {:02x?}",
            &ATTR_RAM_STATIC_EXTERNAL as *const u8 as usize, ATTR_RAM_STATIC_EXTERNAL
        )
        .unwrap();

        writeln!(
            uart,
            "ATTR_RAM_STATIC_EXTERNAL_BSS: {:x}: {:02x?}",
            &ATTR_RAM_STATIC_EXTERNAL_BSS as *const u8 as usize,
            &ATTR_RAM_STATIC_EXTERNAL_BSS[0..20]
        )
        .unwrap();
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}
