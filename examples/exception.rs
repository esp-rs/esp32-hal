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
use esp32_hal::interrupt::{clear_software_interrupt, Interrupt, Interrupt::*, InterruptLevel};
use esp32_hal::serial::{config::Config, Serial};
use esp32_hal::target;
use esp32_hal::Core::PRO;

// !!! Cannot use CriticalSectionSpinLockMutex here, because an NMI is fires from within a locked
// section which leads to a deadlock in the NMI interrupt handler. This is not a problem in this
// case as this is a single threaded example. !!!
static TX: xtensa_lx::mutex::CriticalSectionMutex<Option<esp32_hal::serial::Tx<esp32::UART0>>> =
    xtensa_lx::mutex::CriticalSectionMutex::new(None);

fn locked_print(str: &str) {
    (&TX).lock(|tx| {
        let tx = tx.as_mut().unwrap();

        writeln!(
            tx,
            "    {}, Level: {}",
            str,
            xtensa_lx::interrupt::get_level()
        )
        .unwrap();
    });
}

#[interrupt]
fn FROM_CPU_INTR0() {
    locked_print("FROM_CPU_INTR0");
    clear_software_interrupt(Interrupt::FROM_CPU_INTR0).unwrap();
}

#[interrupt]
fn FROM_CPU_INTR1() {
    locked_print("Start FROM_CPU_INTR1");
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR0).unwrap();
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR2).unwrap();
    locked_print("End FROM_CPU_INTR1");
    clear_software_interrupt(Interrupt::FROM_CPU_INTR1).unwrap();
}

#[interrupt]
fn FROM_CPU_INTR2() {
    locked_print("FROM_CPU_INTR2");
    clear_software_interrupt(Interrupt::FROM_CPU_INTR2).unwrap();
}

#[interrupt]
fn FROM_CPU_INTR3() {
    locked_print("FROM_CPU_INTR3");
    clear_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
}

#[interrupt(INTERNAL_SOFTWARE_LEVEL_3_INTR)]
fn software_level_3() {
    locked_print("INTERNAL_SOFTWARE_LEVEL_3_INTR");
    clear_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
}

#[interrupt(INTERNAL_SOFTWARE_LEVEL_1_INTR)]
fn random_name() {
    locked_print("INTERNAL_SOFTWARE_LEVEL_1_INTR");
    clear_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
}

#[exception]
#[ram]
fn other_exception(
    cause: xtensa_lx_rt::exception::ExceptionCause,
    frame: xtensa_lx_rt::exception::Context,
) {
    (&TX).lock(|tx| {
        let tx = tx.as_mut().unwrap();
        writeln!(tx, "Exception {:?}, {:08x?}", cause, frame).unwrap();
    });
    loop {}
}

#[entry]
fn main() -> ! {
    let dp = target::Peripherals::take().unwrap();

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timers on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let (_, dport_clock_control) = dp.DPORT.split();

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

    let gpios = dp.GPIO.split();
    // setup serial controller
    let mut uart0: Serial<_, _, _> = Serial::new(
        dp.UART0,
        esp32_hal::serial::Pins {
            tx: gpios.gpio1,
            rx: gpios.gpio3,
            cts: None,
            rts: None,
        },
        Config::default(),
        clock_control_config,
    )
    .unwrap();

    uart0.change_baudrate(115200).unwrap();

    // print startup message
    writeln!(uart0, "\n\nReboot!\n",).unwrap();

    let (tx, _) = uart0.split();
    (&TX).lock(|tx_locked| *tx_locked = Some(tx));

    interrupt::enable_with_priority(PRO, Interrupt::FROM_CPU_INTR0, InterruptLevel(2)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::FROM_CPU_INTR1, InterruptLevel(4)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::FROM_CPU_INTR2, InterruptLevel(5)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::FROM_CPU_INTR3, InterruptLevel(7)).unwrap();
    interrupt::enable(INTERNAL_SOFTWARE_LEVEL_1_INTR).unwrap();
    interrupt::enable(INTERNAL_SOFTWARE_LEVEL_3_INTR).unwrap();

    // Trigger various software interrupts, because done in an interrupt free section will
    // actually trigger at the end in order of priority
    (&TX).lock(|tx| {
        let tx = tx.as_mut().unwrap();

        writeln!(tx, "Start Trigger Interrupts",).unwrap();
        interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR0).unwrap();
        interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR1).unwrap();
        interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR2).unwrap();
        // this one will trigger immediately as level 7 is Non-Maskable Intterupt (NMI)
        interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
        interrupt::set_software_interrupt(Interrupt::INTERNAL_SOFTWARE_LEVEL_1_INTR).unwrap();
        interrupt::set_software_interrupt(Interrupt::INTERNAL_SOFTWARE_LEVEL_3_INTR).unwrap();
        writeln!(tx, "End Trigger Interrupts",).unwrap();
    });

    // Trigger outside of interrupt free section, triggers immediately
    (&TX).lock(|tx| {
        let tx = tx.as_mut().unwrap();
        writeln!(tx, "Start Trigger Interrupt",).unwrap();
    });
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR0).unwrap();
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR1).unwrap();
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR2).unwrap();
    // this one will trigger immediately as level 7 is Non-Maskable Intterupt (NMI)
    interrupt::set_software_interrupt(Interrupt::FROM_CPU_INTR3).unwrap();
    interrupt::set_software_interrupt(Interrupt::INTERNAL_SOFTWARE_LEVEL_1_INTR).unwrap();
    interrupt::set_software_interrupt(Interrupt::INTERNAL_SOFTWARE_LEVEL_3_INTR).unwrap();

    (&TX).lock(|tx| {
        let tx = tx.as_mut().unwrap();
        writeln!(tx, "End Trigger Interrupt",).unwrap();
        writeln!(tx, "\nTrigger exception:",).unwrap();
    });

    // Trigger a LoadStoreError due to unaligned access in the IRAM

    #[link_section = ".rwtext"]
    static mut IRAM: [u8; 12] = [0; 12];
    unsafe { IRAM[1] = 10 };

    // Trigger a DivideByZeroError
    unsafe { asm!("quos {0},{0},{0}",in(reg)0) }

    loop {
        sleep(1.s());
        (&TX).lock(|tx| {
            let tx = tx.as_mut().unwrap();

            writeln!(tx, "Wait for watchdog reset").unwrap()
        });
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

#[ram]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}
