#![no_std]
#![no_main]
#![feature(asm)]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use core::cell::RefCell;
use core::ops::DerefMut;
use esp32_hal::clock_control::sleep;
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::interrupt::{Interrupt, InterruptLevel};
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};
use esp32_hal::timer::watchdog::{self, WatchDogResetDuration, WatchdogAction, WatchdogConfig};
use esp32_hal::timer::{Timer, Timer0};
use esp32_hal::Core::PRO;
use spin::Mutex;

const BLINK_HZ: Hertz = Hertz(2);

static TIMER: Mutex<RefCell<Option<Timer<esp32::TIMG0, Timer0>>>> = Mutex::new(RefCell::new(None));
static WATCHDOG1: Mutex<RefCell<Option<watchdog::Watchdog<esp32::TIMG1>>>> =
    Mutex::new(RefCell::new(None));

#[no_mangle]
fn main() -> ! {
    let dp = esp32::Peripherals::take().unwrap();

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    disable_timg_wdts(&mut timg0, &mut timg1);

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    let clkcntrl = esp32_hal::clock_control::ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    let (clkcntrl_config, mut watchdog_rtc) = clkcntrl.freeze().unwrap();
    let (mut timer0, mut timer1, mut watchdog0) = Timer::new(timg0, clkcntrl_config);
    let (_, _, mut watchdog1) = Timer::new(timg1, clkcntrl_config);
    watchdog_rtc.disable();
    watchdog0.disable();

    let wdconfig = WatchdogConfig {
        action1: WatchdogAction::INTERRUPT,
        action2: WatchdogAction::RESETSYSTEM,
        action3: WatchdogAction::DISABLE,
        action4: WatchdogAction::DISABLE,
        period1: 2.s().into(),
        period2: 10.s().into(),
        period3: 0.us(),
        period4: 0.us(),
        cpu_reset_duration: WatchDogResetDuration::T800NS,
        sys_reset_duration: WatchDogResetDuration::T800NS,
        divider: 1,
    };

    watchdog1.set_config(&wdconfig).unwrap();
    //watchdog1.start(3.s());

    timer0.enable(true);
    timer1.enable(true);

    let config = Config {
        baudrate: Hertz(115_200),
        ..Default::default()
    };

    let serial =
        Serial::uart0(dp.UART0, (NoTx, NoRx), config, clkcntrl_config, &mut dport).unwrap();

    let (mut tx, _) = serial.split();

    writeln!(tx, "\n\nESP32 Started\n\n").unwrap();

    timer0.set_alarm(70_000_000);
    timer0.enable_alarm(true);
    timer0.autoreload(true);
    timer0.enable_level_interrupt(true);
    timer0.enable_edge_interrupt(true);

    timer0.listen(esp32_hal::timer::Event::TimeOut);
    interrupt::enable_with_priority(PRO, Interrupt::TG0_T0_LEVEL_INTR, InterruptLevel(1)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::TG0_T0_EDGE_INTR, InterruptLevel(1)).unwrap();

    interrupt::enable_with_priority(PRO, Interrupt::TG1_WDT_LEVEL_INTR, InterruptLevel(1)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::TG1_WDT_EDGE_INTR, InterruptLevel(3)).unwrap();

    *TIMER.lock().borrow_mut() = Some(timer0);
    *WATCHDOG1.lock().borrow_mut() = Some(watchdog1);

    loop {
        interrupt::free(|cs| {
            if let Some(ref mut timer0) = TIMER.lock().borrow_mut().deref_mut() {
                writeln!(
                    tx,
                    "Timers: {} {} {} {} {:x} {:x} {}",
                    timer0.get_value(),
                    timer0.alarm_active(),
                    timer0.interrupt_active_raw(),
                    timer0.interrupt_active(),
                    interrupt::get_interrupt_status(PRO),
                    xtensa_lx6_rt::interrupt::get(),
                    timer1.get_value()
                )
                .unwrap();
            }
        });

        sleep((Hertz(1_000_000) / BLINK_HZ).us());
        sleep((Hertz(1_000_000) / BLINK_HZ).us());
    }
}

#[interrupt]
fn TG0_T0_LEVEL_INTR() {
    interrupt::free(|cs| {
        if let Some(ref mut timer0) = TIMER.lock().borrow_mut().deref_mut() {
            timer0.clear_interrupt();
            timer0.enable_alarm(true);
        }
    });
    esp32_hal::dprintln!("  TG0_T0_LEVEL_INTR");
}

#[interrupt]
fn TG0_T0_EDGE_INTR() {
    interrupt::free(|cs| {
        if let Some(ref mut timer0) = TIMER.lock().borrow_mut().deref_mut() {
            esp32_hal::dprintln!("  TG0_T0_EDGE_INTR");

            dprintln!(
                "    Timers: {} {} {} {} {:x} {:x}",
                timer0.get_value(),
                timer0.alarm_active(),
                timer0.interrupt_active_raw(),
                timer0.interrupt_active(),
                interrupt::get_interrupt_status(PRO),
                xtensa_lx6_rt::interrupt::get(),
            );
            timer0.enable_alarm(true);
        }
    });
}

#[interrupt]
fn TG1_WDT_LEVEL_INTR() {
    interrupt::free(|cs| {
        if let Some(ref mut watchdog1) = WATCHDOG1.lock().borrow_mut().deref_mut() {
            watchdog1.clear_interrupt();
        }
    });
    esp32_hal::dprintln!("  TG1_WDT_LEVEL_INTR");
}

#[interrupt]
fn TG1_WDT_EDGE_INTR() {
    esp32_hal::dprintln!("  TG1_WDT_EDGE_INTR");
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

/// Basic panic handler - just loops
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}
