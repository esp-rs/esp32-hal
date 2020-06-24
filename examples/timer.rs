#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};

use esp32_hal::{
    clock_control::sleep,
    dport::Split,
    dprintln,
    interrupt::{Interrupt, InterruptLevel},
    prelude::*,
    serial::{config::Config, NoRx, NoTx, Serial},
    target,
    timer::{
        watchdog::{self, WatchDogResetDuration, WatchdogAction, WatchdogConfig},
        Timer, Timer0, Timer1, TimerLact, TimerWithInterrupt,
    },
    Core::PRO,
};

const BLINK_HZ: Hertz = Hertz(2);

static TIMER0: CriticalSectionSpinLockMutex<Option<Timer<esp32::TIMG0, Timer0>>> =
    CriticalSectionSpinLockMutex::new(None);
static TIMER1: CriticalSectionSpinLockMutex<Option<Timer<esp32::TIMG0, Timer1>>> =
    CriticalSectionSpinLockMutex::new(None);
static TIMER2: CriticalSectionSpinLockMutex<Option<Timer<esp32::TIMG0, TimerLact>>> =
    CriticalSectionSpinLockMutex::new(None);
static TIMER3: CriticalSectionSpinLockMutex<Option<Timer<esp32::TIMG1, Timer0>>> =
    CriticalSectionSpinLockMutex::new(None);
static TIMER4: CriticalSectionSpinLockMutex<Option<Timer<esp32::TIMG1, Timer1>>> =
    CriticalSectionSpinLockMutex::new(None);
static TIMER5: CriticalSectionSpinLockMutex<Option<Timer<esp32::TIMG1, TimerLact>>> =
    CriticalSectionSpinLockMutex::new(None);
static WATCHDOG1: CriticalSectionSpinLockMutex<Option<watchdog::Watchdog<esp32::TIMG1>>> =
    CriticalSectionSpinLockMutex::new(None);
static TX: CriticalSectionSpinLockMutex<Option<esp32_hal::serial::Tx<esp32::UART0>>> =
    CriticalSectionSpinLockMutex::new(None);

#[entry]
fn main() -> ! {
    let dp = target::Peripherals::take().unwrap();

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

    (&WATCHDOG1).lock(|data| *data = Some(watchdog1));

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

    (&TIMER0).lock(|data| *data = Some(timer0));
    (&TIMER1).lock(|data| *data = Some(timer1));
    (&TIMER2).lock(|data| *data = Some(timer2));
    (&TIMER3).lock(|data| *data = Some(timer3));
    (&TIMER4).lock(|data| *data = Some(timer4));
    (&TIMER5).lock(|data| *data = Some(timer5));

    (&TX).lock(|data| *data = Some(tx));

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
        (&TX, &TIMER0, &TIMER1, &TIMER2).lock(|tx, timer0, timer1, timer2| {
            let tx = tx.as_mut().unwrap();
            let timer0 = timer0.as_mut().unwrap();
            let timer1 = timer1.as_mut().unwrap();
            let timer2 = timer2.as_mut().unwrap();
            writeln!(
                tx,
                "Loop: {} {} {} {} {}",
                x,
                timer0.get_value(),
                timer1.get_value(),
                timer2.get_value(),
                xtensa_lx6::timer::get_cycle_count()
            )
            .unwrap();
            if let Ok(_) = timer1.wait() {
                writeln!(tx, "CANCELLING Timers").unwrap();
                timer0.cancel().unwrap();
                timer1.cancel().unwrap();
            }
        });

        sleep((Hertz(1_000_000) / BLINK_HZ).us());
    }
}

fn locked_print(str: &str) {
    (&TX).lock(|tx| {
        let tx = tx.as_mut().unwrap();

        writeln!(tx, "{}", str).unwrap();
    });
}

fn locked_clear(mut timer_mutex: &CriticalSectionSpinLockMutex<Option<impl TimerWithInterrupt>>) {
    timer_mutex.lock(|timer| {
        let timer = timer.as_mut().unwrap();
        timer.clear_interrupt();
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

    (&WATCHDOG1).lock(|watchdog1| {
        let watchdog1 = watchdog1.as_mut().unwrap();
        watchdog1.clear_interrupt();
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
