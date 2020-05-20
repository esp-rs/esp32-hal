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
use esp32_hal::interrupt::{Interrupt, Interrupt::*, InterruptLevel};
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};
use esp32_hal::Core::PRO;

static TX: spin::Mutex<Option<esp32_hal::serial::Tx<esp32::UART0>>> = spin::Mutex::new(None);

#[interrupt]
fn FROM_CPU_INTR0() {
    writeln!(
        TX.lock().as_mut().unwrap(),
        "  FROM_CPU_INTR0, level: {}",
        xtensa_lx6_rt::interrupt::get_level()
    )
    .unwrap();
    interrupt::clear_software_interrupt(Interrupt::FROM_CPU_INTR0).unwrap();
}

#[interrupt]
fn FROM_CPU_INTR1() {
    writeln!(
        TX.lock().as_mut().unwrap(),
        "  Start FROM_CPU_INTR1, level: {}",
        xtensa_lx6_rt::interrupt::get_level()
    )
    .unwrap();
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR0).unwrap();
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR2).unwrap();
    writeln!(
        TX.lock().as_mut().unwrap(),
        "  End FROM_CPU_INTR1, level: {}",
        xtensa_lx6_rt::interrupt::get_level()
    )
    .unwrap();
    interrupt::clear_software_interrupt(Interrupt::FROM_CPU_INTR1).unwrap();
}

#[interrupt]
fn FROM_CPU_INTR2() {
    writeln!(
        TX.lock().as_mut().unwrap(),
        "  FROM_CPU_INTR2, level: {}",
        xtensa_lx6_rt::interrupt::get_level()
    )
    .unwrap();
    interrupt::clear_software_interrupt(Interrupt::FROM_CPU_INTR2).unwrap();
}

#[interrupt]
fn FROM_CPU_INTR3() {
    writeln!(
        TX.lock().as_mut().unwrap(),
        "  FROM_CPU_INTR3, level: {}",
        xtensa_lx6_rt::interrupt::get_level()
    )
    .unwrap();
    interrupt::clear_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
}

#[interrupt(INTERNAL_SOFTWARE_LEVEL_3_INTR)]
fn software_level_3() {
    writeln!(
        TX.lock().as_mut().unwrap(),
        "  INTERNAL_SOFTWARE_LEVEL_3_INTR, level: {}",
        xtensa_lx6_rt::interrupt::get_level()
    )
    .unwrap();
    interrupt::clear_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
}

#[interrupt(INTERNAL_SOFTWARE_LEVEL_1_INTR)]
fn random_name() {
    writeln!(
        TX.lock().as_mut().unwrap(),
        "  INTERNAL_SOFTWARE_LEVEL_1_INTR, level: {}",
        xtensa_lx6_rt::interrupt::get_level()
    )
    .unwrap();
    interrupt::clear_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
}

#[exception]
#[ram]
fn other_exception(
    cause: xtensa_lx6_rt::exception::ExceptionCause,
    frame: xtensa_lx6_rt::exception::Context,
) {
    writeln!(
        TX.lock().as_mut().unwrap(),
        "Exception {:?}, {:08x?}",
        cause,
        frame
    )
    .unwrap();
    loop {}
}

#[entry]
fn main() -> ! {
    let dp = esp32::Peripherals::take().unwrap();

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

    interrupt::enable_with_priority(PRO, Interrupt::FROM_CPU_INTR0, InterruptLevel(2)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::FROM_CPU_INTR1, InterruptLevel(4)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::FROM_CPU_INTR2, InterruptLevel(5)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::FROM_CPU_INTR3, InterruptLevel(7)).unwrap();
    interrupt::enable(INTERNAL_SOFTWARE_LEVEL_1_INTR).unwrap();
    interrupt::enable(INTERNAL_SOFTWARE_LEVEL_3_INTR).unwrap();

    // Trigger various software interrupts, because done in an interrupt free section will
    // actually trigger at the end in order of priority
    interrupt::free(|_| {
        writeln!(TX.lock().as_mut().unwrap(), "Start Trigger Interrupts",).unwrap();
        interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR0).unwrap();
        interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR1).unwrap();
        interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR2).unwrap();
        // this one will trigger immediately as level 7 is Non-Maskable Intterupt (NMI)
        interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
        interrupt::set_software_interrupt(Interrupt::INTERNAL_SOFTWARE_LEVEL_1_INTR).unwrap();
        interrupt::set_software_interrupt(Interrupt::INTERNAL_SOFTWARE_LEVEL_3_INTR).unwrap();
        writeln!(TX.lock().as_mut().unwrap(), "End Trigger Interrupts",).unwrap();
    });

    // Trigger outside of interrupt free section, triggers immediately
    writeln!(TX.lock().as_mut().unwrap(), "Start Trigger Interrupt",).unwrap();
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR0).unwrap();
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR1).unwrap();
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR2).unwrap();
    // this one will trigger immediately as level 7 is Non-Maskable Intterupt (NMI)
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
    interrupt::set_software_interrupt(Interrupt::INTERNAL_SOFTWARE_LEVEL_1_INTR).unwrap();
    interrupt::set_software_interrupt(Interrupt::INTERNAL_SOFTWARE_LEVEL_3_INTR).unwrap();
    writeln!(TX.lock().as_mut().unwrap(), "End Trigger Interrupt",).unwrap();

    // Trigger a LoadStoreError due to unaligned access in the IRAM

    writeln!(TX.lock().as_mut().unwrap(), "\nTrigger exception:",).unwrap();

    #[link_section = ".rwtext"]
    static mut IRAM: [u8; 12] = [0; 12];
    unsafe { IRAM[1] = 10 };

    // Trigger a DivideByZeroError
    unsafe { asm!("quos $0,$0,$0":"+r"(0)) }

    loop {
        sleep(1.s());
        xtensa_lx6_rt::interrupt::free(|_| {
            writeln!(TX.lock().as_mut().unwrap(), "Wait for watchdog reset").unwrap()
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
