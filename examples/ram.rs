#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::clock_control::{sleep, ClockControl};
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

#[entry]
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
    let clock_control = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    let (clock_control_config, mut watchdog) = clock_control.freeze().unwrap();

    watchdog.start(15.s());

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

    writeln!(uart0, "Running on core {:?}\n", esp32_hal::get_core()).unwrap();

    ram_tests(&mut uart0);

    loop {
        sleep(1.s());
        writeln!(uart0, "Alive and waiting for watchdog reset").unwrap();
    }
}

fn attr_none_fn(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    writeln!(
        uart,
        "{:<40}: {:08x?}",
        "attr_none_fn",
        xtensa_lx6_rt::get_program_counter()
    )
    .unwrap();
}

#[ram]
fn attr_ram_fn(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    writeln!(
        uart,
        "{:<40}: {:08x?}",
        "attr_ram_fn",
        xtensa_lx6_rt::get_program_counter()
    )
    .unwrap();
}

#[ram(rtc_slow)]
fn attr_ram_fn_rtc_slow(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    writeln!(
        uart,
        "{:<40}: {:08x?}",
        "attr_ram_fn_rtc_slow",
        xtensa_lx6_rt::get_program_counter()
    )
    .unwrap();
}

#[ram(rtc_fast)]
fn attr_ram_fn_rtc_fast(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    writeln!(
        uart,
        "{:<40}: {:08x?}",
        "attr_ram_fn_rtc_fast",
        xtensa_lx6_rt::get_program_counter()
    )
    .unwrap();
}

static ATTR_NONE_STATIC: [u8; 16] = *b"ATTR_NONE_STATIC";

static mut ATTR_NONE_STATIC_MUT: [u8; 20] = *b"ATTR_NONE_STATIC_MUT";

static ATTR_NONE_STATIC_BSS: [u8; 32] = [0; 32];

static mut ATTR_NONE_STATIC_MUT_BSS: [u8; 32] = [0; 32];

#[ram]
static ATTR_RAM_STATIC: [u8; 15] = *b"ATTR_RAM_STATIC";

#[ram(zeroed)]
static ATTR_RAM_STATIC_BSS: [u8; 32] = [0; 32];

#[ram(uninitialized)]
static ATTR_RAM_STATIC_UNINIT: [u8; 32] = [0; 32];

#[ram(rtc_slow)]
static ATTR_RAM_STATIC_RTC_SLOW: [u8; 24] = *b"ATTR_RAM_STATIC_RTC_SLOW";

#[ram(rtc_slow, zeroed)]
static ATTR_RAM_STATIC_RTC_SLOW_BSS: [u8; 32] = [0; 32];

#[ram(rtc_slow, uninitialized)]
static ATTR_RAM_STATIC_RTC_SLOW_UNINIT: [u8; 32] = [0; 32];

#[ram(rtc_fast)]
static ATTR_RAM_STATIC_RTC_FAST: [u8; 24] = *b"ATTR_RAM_STATIC_RTC_FAST";

#[ram(rtc_fast, zeroed)]
static ATTR_RAM_STATIC_RTC_FAST_BSS: [u8; 32] = [0; 32];

#[ram(rtc_fast, uninitialized)]
static ATTR_RAM_STATIC_RTC_FAST_UNINIT: [u8; 32] = [0; 32];

#[cfg(feature = "external_ram")]
#[ram(external)]
static mut ATTR_RAM_STATIC_EXTERNAL: [u8; 24] = *b"ATTR_RAM_STATIC_EXTERNAL";

#[cfg(feature = "external_ram")]
#[ram(external, zeroed)]
static mut ATTR_RAM_STATIC_EXTERNAL_BSS: [u8; 32] = [0; 32];

#[cfg(feature = "external_ram")]
#[ram(external, uninitialized)]
static mut ATTR_RAM_STATIC_EXTERNAL_UNINIT: [u8; 32] = [0; 32];

// Macro to simplify printing of the various different memory allocations
macro_rules! print_info {
    ( $uart:expr, $x:expr ) => {
        writeln!(
            $uart,
            "{:<40}: {:#08x?}: {:02x?}",
            stringify!($x),
            &$x as *const u8 as usize,
            $x
        )
        .unwrap();
    };
}

fn ram_tests(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    writeln!(uart).unwrap();

    attr_none_fn(uart);
    attr_ram_fn(uart);
    attr_ram_fn_rtc_slow(uart);
    attr_ram_fn_rtc_fast(uart);

    writeln!(uart).unwrap();

    unsafe {
        print_info!(uart, ATTR_NONE_STATIC);
        print_info!(uart, ATTR_NONE_STATIC_MUT);
        print_info!(uart, ATTR_NONE_STATIC_BSS);
        print_info!(uart, ATTR_NONE_STATIC_MUT_BSS);

        print_info!(uart, ATTR_RAM_STATIC);
        print_info!(uart, ATTR_RAM_STATIC_BSS);
        print_info!(uart, ATTR_RAM_STATIC_UNINIT);

        print_info!(uart, ATTR_RAM_STATIC_RTC_SLOW);
        print_info!(uart, ATTR_RAM_STATIC_RTC_SLOW_BSS);
        print_info!(uart, ATTR_RAM_STATIC_RTC_SLOW_UNINIT);

        print_info!(uart, ATTR_RAM_STATIC_RTC_FAST);
        print_info!(uart, ATTR_RAM_STATIC_RTC_FAST_BSS);
        print_info!(uart, ATTR_RAM_STATIC_RTC_FAST_UNINIT);
    }

    if cfg!(feature = "external_ram") {
        external_ram(uart);
    }

    writeln!(uart).unwrap();
}

#[cfg(not(feature = "external_ram"))]
fn external_ram(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {}

#[cfg(feature = "external_ram")]
fn external_ram(uart: &mut esp32_hal::serial::Serial<esp32::UART0, (NoTx, NoRx)>) {
    unsafe {
        print_info!(uart, ATTR_RAM_STATIC_EXTERNAL);
        print_info!(uart, ATTR_RAM_STATIC_EXTERNAL_BSS);
        print_info!(uart, ATTR_RAM_STATIC_EXTERNAL_UNINIT);
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
