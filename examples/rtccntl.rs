#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use embedded_hal::watchdog::*;
use esp32;
use esp32_hal::clock_control::watchdog::*;
use esp32_hal::clock_control::*;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};
use esp32_hal::units::*;

const BLINK_HZ: Hertz = Hertz(2);

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

pub struct Context {
    clock_control: esp32_hal::clock_control::ClockControl,
    /*  uart0: esp32_hal::serial::Serial<
        'static,
        esp32::UART0,
        (esp32_hal::serial::NoTx, esp32_hal::serial::NoRx),
    >,*/
    rx: esp32_hal::serial::Rx<esp32::UART0>,
    tx: esp32_hal::serial::Tx<esp32::UART0>,
}

pub static GLOBAL_CONTEXT: spin::Mutex<Option<Context>> = spin::Mutex::new(None);

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

    let mut clock_control = ClockControl::new(dp.RTCCNTL, dp.APB_CTRL);

    let uart0 = Serial::uart0(
        dp.UART0,
        (NoTx, NoRx),
        Config::default(),
        &mut clock_control,
    )
    .unwrap();

    let (tx, rx) = uart0.split();

    *GLOBAL_CONTEXT.lock() = Some(Context {
        clock_control,
        //        uart0,
        rx,
        tx,
    });

    {
        let mut lock = GLOBAL_CONTEXT.lock();
        let ctx = lock.as_mut().unwrap();
        let tx = &mut ctx.tx;
        let clock_control = &mut ctx.clock_control;

        clock_control.set_slow_source(SlowClockSource::SLOW_CK);
        let mut wdtconfig = clock_control.watchdog().config().unwrap();
        wdtconfig.action1 = WatchdogAction::RESETCPU;
        wdtconfig.period1 = 10.s().into();
        wdtconfig.reset_cpu[0] = true;

        //TODO: frequencies are not correct
        clock_control.watchdog().start(wdtconfig);

        writeln!(tx, "\n\nReboot!\n").unwrap();

        writeln!(tx, "core {:0x}", xtensa_lx6_rt::get_core_id()).unwrap();
        writeln!(
            tx,
            "CPU Frequency {}, APB Frequency {}, Slow Frequency {}",
            clock_control.cpu_frequency(),
            clock_control.apb_frequency(),
            clock_control.slow_frequency()
        )
        .unwrap();
        writeln!(tx, "{:?}", clock_control.cpu_frequency_config().unwrap()).unwrap();
    }

    // panic!("panic test");

    let mut x = 1;
    loop {
        writeln!(GLOBAL_CONTEXT.lock().as_mut().unwrap().tx, "Loop: {}", x).unwrap();
        x += 1;

        for _x in 0..10 {
            delay(
                GLOBAL_CONTEXT
                    .lock()
                    .as_mut()
                    .unwrap()
                    .clock_control
                    .cpu_frequency()
                    / BLINK_HZ
                    / 10,
            );
            // comment out next line to check watchdog behavior
            GLOBAL_CONTEXT
                .lock()
                .as_mut()
                .unwrap()
                .clock_control
                .watchdog()
                .feed();
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

/// cycle accurate delay using the cycle counter register
pub fn delay(clocks: u32) {
    let start = xtensa_lx6_rt::get_cycle_count();
    loop {
        if xtensa_lx6_rt::get_cycle_count().wrapping_sub(start) >= clocks {
            break;
        }
    }
}

/// panic handler using static tx
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    writeln!(
        GLOBAL_CONTEXT.lock().as_mut().unwrap().tx,
        "\n\n*** {:?}",
        info
    )
    .unwrap();
    loop {}
}
