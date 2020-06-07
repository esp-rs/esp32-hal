#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use core::cell::RefCell;
use core::ops::DerefMut;
use embedded_hal::timer::{Cancel, CountDown};
use esp32_hal::clock_control::sleep;
use esp32_hal::dport::Split;
use esp32_hal::dprintln;
use esp32_hal::interrupt::{Interrupt, InterruptLevel};
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};
use esp32_hal::timer::watchdog::{self, WatchDogResetDuration, WatchdogAction, WatchdogConfig};
use esp32_hal::timer::{Timer, Timer0, Timer1, TimerLact, TimerWithInterrupt};
use esp32_hal::Core::PRO;
use spin::Mutex;

const BLINK_HZ: Hertz = Hertz(2);

static TIMER0: Mutex<RefCell<Option<Timer<esp32::TIMG0, Timer0>>>> = Mutex::new(RefCell::new(None));
static TIMER2: Mutex<RefCell<Option<Timer<esp32::TIMG0, TimerLact>>>> =
    Mutex::new(RefCell::new(None));
static TIMER3: Mutex<RefCell<Option<Timer<esp32::TIMG1, Timer0>>>> = Mutex::new(RefCell::new(None));
static TIMER4: Mutex<RefCell<Option<Timer<esp32::TIMG1, Timer1>>>> = Mutex::new(RefCell::new(None));
static TIMER5: Mutex<RefCell<Option<Timer<esp32::TIMG1, TimerLact>>>> =
    Mutex::new(RefCell::new(None));

static WATCHDOG1: Mutex<RefCell<Option<watchdog::Watchdog<esp32::TIMG1>>>> =
    Mutex::new(RefCell::new(None));
static TX: Mutex<Option<esp32_hal::serial::Tx<esp32::UART0>>> = spin::Mutex::new(None);

#[no_mangle]
fn main() -> ! {
    let dp = esp32::Peripherals::take().unwrap();

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    let clkcntrl = esp32_hal::clock_control::ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    let (clkcntrl_config, mut watchdog_rtc) = clkcntrl.freeze().unwrap();
    let (mut timer0, mut timer1, mut timer2, mut watchdog0) = Timer::new(dp.TIMG0, clkcntrl_config);
    let (mut timer3, mut timer4, mut timer5, mut watchdog1) = Timer::new(dp.TIMG1, clkcntrl_config);

    watchdog_rtc.disable();
    watchdog0.disable();

    let wdconfig = WatchdogConfig {
        action1: WatchdogAction::INTERRUPT,
        action2: WatchdogAction::RESETSYSTEM,
        action3: WatchdogAction::DISABLE,
        action4: WatchdogAction::DISABLE,
        period1: 6.s().into(),
        period2: 8.s().into(),
        period3: 0.us().into(),
        period4: 0.us().into(),
        cpu_reset_duration: WatchDogResetDuration::T800NS,
        sys_reset_duration: WatchDogResetDuration::T800NS,
        divider: 1,
    };

    watchdog1.set_config(&wdconfig).unwrap();

    *WATCHDOG1.lock().borrow_mut() = Some(watchdog1);

    let config = Config {
        baudrate: Hertz(115_200),
        ..Default::default()
    };

    let serial =
        Serial::uart0(dp.UART0, (NoTx, NoRx), config, clkcntrl_config, &mut dport).unwrap();

    let (mut tx, _) = serial.split();

    writeln!(tx, "\n\nESP32 Started\n\n").unwrap();
    writeln!(tx, "Clock Config: {:#?}", clkcntrl_config).unwrap();

    timer0.start(1000.ms());
    timer1.start(4.s());
    timer2.start(2.s());
    timer3.start(900.ms());
    timer4.start(1100.ms());
    timer5.start(1300.ms());

    writeln!(tx, "Waiting for timer0").unwrap();
    nb::block!(timer0.wait()).unwrap();

    writeln!(tx, "Finished waiting for timer0").unwrap();

    timer0.listen(esp32_hal::timer::Event::TimeOut);
    timer0.listen(esp32_hal::timer::Event::TimeOutEdge);

    timer2.listen(esp32_hal::timer::Event::TimeOut);
    timer2.listen(esp32_hal::timer::Event::TimeOutEdge);

    timer3.listen(esp32_hal::timer::Event::TimeOut);
    timer4.listen(esp32_hal::timer::Event::TimeOut);
    timer5.listen(esp32_hal::timer::Event::TimeOut);

    *TIMER0.lock().borrow_mut() = Some(timer0);
    *TIMER2.lock().borrow_mut() = Some(timer2);
    *TIMER3.lock().borrow_mut() = Some(timer3);
    *TIMER4.lock().borrow_mut() = Some(timer4);
    *TIMER5.lock().borrow_mut() = Some(timer5);
    *TX.lock() = Some(tx);

    interrupt::enable_with_priority(PRO, Interrupt::TG0_T0_LEVEL_INTR, InterruptLevel(1)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::TG0_T0_EDGE_INTR, InterruptLevel(1)).unwrap();

    interrupt::enable_with_priority(PRO, Interrupt::TG1_WDT_LEVEL_INTR, InterruptLevel(1)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::TG1_WDT_EDGE_INTR, InterruptLevel(3)).unwrap();

    interrupt::enable_with_priority(PRO, Interrupt::TG0_LACT_LEVEL_INTR, InterruptLevel(1))
        .unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::TG0_LACT_EDGE_INTR, InterruptLevel(4)).unwrap();

    interrupt::enable_with_priority(PRO, Interrupt::TG1_T0_LEVEL_INTR, InterruptLevel(1)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::TG1_T1_LEVEL_INTR, InterruptLevel(1)).unwrap();
    interrupt::enable_with_priority(PRO, Interrupt::TG1_LACT_LEVEL_INTR, InterruptLevel(1))
        .unwrap();

    let mut x = 0;
    loop {
        x = x + 1;
        interrupt::free(|_| {
            if let Some(ref mut timer0) = TIMER0.lock().borrow_mut().deref_mut() {
                if let Some(ref mut timer2) = TIMER2.lock().borrow_mut().deref_mut() {
                    if let Some(ref mut tx) = TX.lock().deref_mut() {
                        writeln!(
                            tx,
                            "Loop: {} {} {} {} {}",
                            x,
                            timer0.get_value(),
                            timer1.get_value(),
                            timer2.get_value(),
                            xtensa_lx6::get_cycle_count()
                        )
                        .unwrap();
                        if let Ok(_) = timer1.wait() {
                            writeln!(tx, "CANCELLING Timers").unwrap();
                            timer0.cancel().unwrap();
                            timer1.cancel().unwrap();
                        }
                    }
                }
            }
        });

        sleep((Hertz(1_000_000) / BLINK_HZ).us());
    }
}

fn locked_print(str: &str) {
    interrupt::free(|_| {
        if let Some(ref mut tx) = TX.lock().deref_mut() {
            writeln!(tx, "{}", str).unwrap();
        }
    });
}

fn locked_clear(timer_mutex: &Mutex<RefCell<Option<impl TimerWithInterrupt>>>) {
    interrupt::free(|_| {
        if let Some(ref mut timer) = timer_mutex.lock().borrow_mut().deref_mut() {
            timer.clear_interrupt();
        }
    });
}

#[interrupt]
fn TG0_T0_LEVEL_INTR() {
    locked_print("  TG0_T0_LEVEL_INTR");
    locked_clear(&TIMER0);
}

#[interrupt]
fn TG0_T0_EDGE_INTR() {
    locked_print("  TG0_T0_EDGE_INTR");
}

#[interrupt]
fn TG0_LACT_LEVEL_INTR() {
    locked_print("  TG0_LACT_LEVEL_INTR");
    locked_clear(&TIMER2);
}

#[interrupt]
fn TG0_LACT_EDGE_INTR() {
    locked_print("  TG0_LACT_EDGE_INTR");
}

#[interrupt]
fn TG1_T0_LEVEL_INTR() {
    locked_print("  TG1_T0_LEVEL_INTR");
    locked_clear(&TIMER3);
}

#[interrupt]
fn TG1_T1_LEVEL_INTR() {
    locked_print("  TG1_T1_LEVEL_INTR");
    locked_clear(&TIMER4);
}

#[interrupt]
fn TG1_LACT_LEVEL_INTR() {
    locked_print("  TG1_LACT_LEVEL_INTR");
    locked_clear(&TIMER5);
}

#[interrupt]
fn TG1_WDT_LEVEL_INTR() {
    locked_print("  TG1_WDT_LEVEL_INTR");

    interrupt::free(|_| {
        if let Some(ref mut watchdog1) = WATCHDOG1.lock().borrow_mut().deref_mut() {
            watchdog1.clear_interrupt();
        }
    });
}

#[interrupt]
fn TG1_WDT_EDGE_INTR() {
    locked_print("  TG1_WDT_EDGE_INTR");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}
