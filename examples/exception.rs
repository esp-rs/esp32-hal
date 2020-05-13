#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::clock_control::{sleep, CPUSource::PLL, ClockControl};
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

static TX: spin::Mutex<Option<esp32_hal::serial::Tx<esp32::UART0>>> = spin::Mutex::new(None);

#[exception]
#[ram]
fn other_exception2(cause: xtensa_lx6_rt::exception::ExceptionCause) {
    writeln!(TX.lock().as_mut().unwrap(), "Exception {:?}", cause).unwrap();
    //unsafe { asm!("quos $0,$0,$0":"+r"(0)) }
    loop {}
}

#[interrupt(1)]
#[ram]
fn interrupt_level_1(level: u32) {
    xtensa_lx6_rt::interrupt::get();
    unsafe {
        xtensa_lx6_rt::interrupt::clear(1 << 7);
    }
    writeln!(TX.lock().as_mut().unwrap(), "Interrupt {} Start", level).unwrap();
    unsafe {
        //   xtensa_lx6_rt::interrupt::set(1 << 29);
    }
    writeln!(TX.lock().as_mut().unwrap(), "Interrupt {} End", level).unwrap();
}

#[interrupt(3)]
#[ram]
fn interrupt_level_3(level: u32) {
    xtensa_lx6_rt::interrupt::get();
    unsafe {
        xtensa_lx6_rt::interrupt::clear(1 << 29);
    }
    writeln!(TX.lock().as_mut().unwrap(), "Interrupt {} Start", level).unwrap();
    unsafe {
        xtensa_lx6_rt::interrupt::set(1 << 7);
    }
    writeln!(TX.lock().as_mut().unwrap(), "Interrupt {} End", level).unwrap();
}

#[interrupt(4)]
#[ram]
#[naked]
fn interrupt_level_4() {
    writeln!(TX.lock().as_mut().unwrap(), "Interrupt 4").unwrap();
}

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
    let mut clock_control = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    // set desired clock frequencies
    clock_control
        .set_cpu_frequencies(PLL, 240.MHz(), PLL, 240.MHz(), PLL, 240.MHz())
        .unwrap();

    let (clock_control_config, mut watchdog) = clock_control.freeze().unwrap();

    watchdog.start(20.s());

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

    let (tx, _) = uart0.split();
    *TX.lock() = Some(tx);

    unsafe {
        xtensa_lx6_rt::interrupt::enable_mask(1 << 7 | 1 << 29);
        xtensa_lx6_rt::interrupt::set(1 << 29);
        //        xtensa_lx6_rt::interrupt::disable_mask(0x00000080);
        //        xtensa_lx6_rt::interrupt::disable_mask(xtensa_lx6_rt::get_cycle_count());
    };

    /*
    #[link_section = ".rwtext"]
    static mut IRAM: [u8; 12] = [0; 12];
    unsafe { IRAM[1] = 10 };
    */

    unsafe { asm!("quos $0,$0,$0":"+r"(0)) }

    loop {
        sleep(1.s());
        xtensa_lx6_rt::interrupt::free(|_| {
            writeln!(TX.lock().as_mut().unwrap(), "Wait for watchdog reset",).unwrap()
        });
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

#[ram]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}
