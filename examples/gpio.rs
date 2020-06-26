#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};

use esp32_hal::{
    clock_control::sleep,
    dport::Split,
    dprintln,
    gpio::{Event, Floating, Pin},
    interrupt::{Interrupt, InterruptLevel},
    prelude::*,
    serial::{config::Config, Serial},
    target,
    timer::Timer,
    Core,
};

static SERIAL: CriticalSectionSpinLockMutex<
    Option<
        esp32_hal::serial::Serial<
            esp32::UART0,
            esp32_hal::gpio::Gpio1<esp32_hal::gpio::Unknown>,
            esp32_hal::gpio::Gpio3<esp32_hal::gpio::Unknown>,
        >,
    >,
> = CriticalSectionSpinLockMutex::new(None);

static GPIO0: CriticalSectionSpinLockMutex<
    Option<esp32_hal::gpio::Gpio0<esp32_hal::gpio::Input<Floating>>>,
> = CriticalSectionSpinLockMutex::new(None);

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
    let (_, _, _, mut watchdog0) = Timer::new(dp.TIMG0, clkcntrl_config);
    let (_, _, _, mut watchdog1) = Timer::new(dp.TIMG1, clkcntrl_config);

    watchdog_rtc.disable();
    watchdog0.disable();
    watchdog1.disable();

    let gpios = dp.GPIO.split();

    let mut gpio0 = gpios.gpio0.into_floating_input();

    gpio0.listen_with_options(Event::LowLevel, true, false, true, false, false);
    interrupt::enable(Interrupt::GPIO_INTR).unwrap();

    // Even though the interrupt is called GPIO_NMI is can be routed to any interrupt level.
    // Using NMI level (7) is in principle a risk for deadlocks because the
    // CriticalSectionSpinLockMutex does not disable the NMI. Therefore using level 5 instead.

    // Because the level 5 interrupt clears the interrupt, the regular level 1 handler
    // will not be called.
    // Comment out the next line to test the level 1 handler
    interrupt::enable_with_priority(Core::PRO, Interrupt::GPIO_NMI, InterruptLevel(5)).unwrap();

    // setup serial controller
    let mut serial: Serial<_, _, _> = Serial::new(
        dp.UART0,
        esp32_hal::serial::Pins {
            tx: gpios.gpio1,
            rx: gpios.gpio3,
            cts: None,
            rts: None,
        },
        Config::default().baudrate(115_200.Hz()),
        clkcntrl_config,
        &mut dport,
    )
    .unwrap();

    writeln!(serial, "\n\nESP32 Started\n\n").unwrap();

    (&SERIAL).lock(|val| *val = Some(serial));
    (&GPIO0).lock(|val| *val = Some(gpio0));

    let mut x = 0;
    loop {
        x = x + 1;
        (&SERIAL, &GPIO0).lock(|serial, gpio0| {
            let serial = serial.as_mut().unwrap();
            let gpio0 = gpio0.as_mut().unwrap();
            writeln!(serial, "Loop: {} {}", x, gpio0.is_high().unwrap()).unwrap();
        });

        sleep(500.ms());
    }
}

fn handle_gpio_interrupt() {
    (&GPIO0, &SERIAL).lock(|gpio0, serial| {
        let gpio0 = gpio0.as_mut().unwrap();
        let serial = serial.as_mut().unwrap();

        if gpio0.is_non_maskable_interrupt_set() {
            writeln!(
                serial,
                "  Interrupt level: {}, pin state: {}",
                xtensa_lx6::interrupt::get_level(),
                gpio0.is_high().unwrap()
            )
            .unwrap();

            if gpio0.is_high().unwrap() {
                gpio0.listen_with_options(Event::LowLevel, true, false, true, false, false);
            } else {
                gpio0.listen_with_options(Event::HighLevel, true, false, true, false, false);
            };
            // need to change listen before clearing interrupt, otherwise will fire
            // immediately again.
            gpio0.clear_interrupt();
        }
    });
}

#[interrupt]
fn GPIO_INTR() {
    handle_gpio_interrupt();
}

#[interrupt]
fn GPIO_NMI() {
    handle_gpio_interrupt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}
