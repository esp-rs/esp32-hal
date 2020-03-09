#![no_std]
#![no_main]

use xtensa_lx6_rt as _;

use core::fmt::Write;
use core::panic::PanicInfo;
use esp32;

//use embedded_hal::watchdog::{WatchdogDisable, WatchdogEnable};
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

use embedded_hal::watchdog::*;
use esp32_hal::clock_control::watchdog::*;
use esp32_hal::clock_control::*;

use esp32_hal::units::*;

/// The default clock source is the onboard crystal
/// In most cases 40mhz (but can be as low as 2mhz depending on the board)
const CORE_HZ: u32 = 40_000_000;

const BLINK_HZ: u32 = CORE_HZ / 1;

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timers on halt
    // we will do it manually on startup
    //    disable_timg_wdts(&mut timg0, &mut timg1);

    disable_timg_wdts(&mut timg0, &mut timg1);

    let mut clkcntrl = ClockControl::new(dp.RTCCNTL, dp.APB_CTRL);

    let serial = Serial::uart0(dp.UART0, (NoTx, NoRx), Config::default(), &clkcntrl).unwrap();
    let baudrate = serial.get_baudrate();
    let (mut tx, mut _rx) = serial.split();

    let rtc_config = clkcntrl.cpu_frequency_config().unwrap();

    writeln!(tx, "\n\nReboot!\n").unwrap();
    writeln!(tx, "baudrate {:?}", baudrate).unwrap();
    writeln!(tx, "core {:0x}", xtensa_lx6_rt::get_core_id()).unwrap();
    writeln!(
        tx,
        "CPU Frequency {}, APB Frequency {}, Slow Frequency {}",
        clkcntrl.cpu_frequency(),
        clkcntrl.apb_frequency(),
        clkcntrl.slow_frequency()
    )
    .unwrap();

    clkcntrl.set_slow_source(SlowClockSource::SLOW_CK);

    let mut watchdog = clkcntrl.watchdog();

    let mut wdtconfig = watchdog.config().unwrap();
    wdtconfig.action1 = WatchdogAction::RESETCPU;
    wdtconfig.period1 = 10.s().into();
    wdtconfig.reset_cpu[0] = true;

    //TODO: frequencies are not correct
    watchdog.start(wdtconfig);

    writeln!(tx, "new watchdog config {:?}", watchdog.config().unwrap()).unwrap();
    writeln!(
        tx,
        "new watchdog config0 {:0x} {:0x} {:0x}",
        unsafe { &(*esp32::RTCCNTL::ptr()).wdtconfig0.read().bits() },
        unsafe { &(*esp32::RTCCNTL::ptr()).wdtconfig0.read().wdt_stg0().bits() },
        unsafe { &(*esp32::RTCCNTL::ptr()).wdtconfig1.read().bits() }
    )
    .unwrap();

    loop {
        writeln!(tx, "{:?}", rtc_config).unwrap();

        for _x in 0..10 {
            delay(BLINK_HZ / 10);
            // comment out next line to check watchdog behavior
            clkcntrl.watchdog().feed();
        }
    }
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
    let start = xtensa_lx6_rt::get_cycle_count();
    loop {
        if xtensa_lx6_rt::get_cycle_count().wrapping_sub(start) >= clocks {
            break;
        }
    }
}

// ugly panic handling: directly output to UART0
pub fn count() -> u16 {
    unsafe {
        ((*esp32::UART0::ptr())
            .mem_cnt_status
            .read()
            .tx_mem_cnt()
            .bits() as u16)
            << 8
            | (*esp32::UART0::ptr()).status.read().txfifo_cnt().bits() as u16
    }
}

fn write(byte: u8) -> nb::Result<(), core::convert::Infallible> {
    if count() < 128 {
        unsafe {
            (*esp32::UART0::ptr())
                .tx_fifo
                .write_with_zero(|w| w.bits(byte))
        }
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}
struct TX {}

impl core::fmt::Write for TX {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

/// Basic panic handler - just loops
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut tx = TX {};
    writeln!(tx, "\n\n*** {:?}", info).unwrap();
    loop {}
}
