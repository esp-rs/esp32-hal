#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};

use esp32_hal::{
    clock_control::sleep,
    dport::Split,
    dprintln,
    gpio::{Event, Floating, InputPin, Pin, Pull, RTCInputPin},
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

static GPIO: CriticalSectionSpinLockMutex<
    Option<esp32_hal::gpio::Gpio26<esp32_hal::gpio::RTCInput<Floating>>>,
> = CriticalSectionSpinLockMutex::new(None);

#[entry]
fn main() -> ! {
    let dp = target::Peripherals::take().unwrap();

    let (_, dport_clock_control) = dp.DPORT.split();

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

    let mut gpio = gpios.gpio26.into_floating_rtc_input();
    gpio.internal_pull_up(true);
    gpio.enable_hold(false);
    gpio.enable_input(false);
    gpio.rtc_enable_input(true);

    gpio.listen_with_options(Event::LowLevel, true, false, true, false, false);

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
    )
    .unwrap();

    writeln!(serial, "\n\nESP32 Started\n\n").unwrap();

    (&SERIAL).lock(|val| *val = Some(serial));
    (&GPIO).lock(|val| *val = Some(gpio));

    interrupt::enable(Interrupt::GPIO_INTR).unwrap();

    // Even though the interrupt is called GPIO_NMI is can be routed to any interrupt level.
    // Using NMI level (7) is in principle a risk for deadlocks because the
    // CriticalSectionSpinLockMutex does not disable the NMI. Therefore using level 5 instead.

    // Because the level 5 interrupt clears the interrupt, the regular level 1 handler
    // will not be called.
    // Comment out the next line to test the level 1 handler
    interrupt::enable_with_priority(Core::PRO, Interrupt::GPIO_NMI, InterruptLevel(5)).unwrap();

    let mut x = 0;
    loop {
        x += 1;
        (&SERIAL, &GPIO).lock(|serial, gpio| {
            let serial = serial.as_mut().unwrap();
            let gpio = gpio.as_mut().unwrap();
            writeln!(
                serial,
                "Loop: {} {} {} {}",
                x,
                gpio.is_high().unwrap(),
                gpio.is_input_high(),
                gpio.rtc_is_input_high()
            )
            .unwrap();
        });

        sleep(500.ms());
    }
}

fn handle_gpio_interrupt() {
    (&GPIO, &SERIAL).lock(|gpio, serial| {
        let gpio = gpio.as_mut().unwrap();
        let serial = serial.as_mut().unwrap();

        if gpio.is_interrupt_set() || gpio.is_non_maskable_interrupt_set() {
            writeln!(
                serial,
                "  Interrupt level: {}, pin state: {}",
                xtensa_lx::interrupt::get_level(),
                gpio.is_high().unwrap()
            )
            .unwrap();

            if gpio.is_high().unwrap() {
                gpio.listen_with_options(Event::LowLevel, true, false, true, false, false);
            } else {
                gpio.listen_with_options(Event::HighLevel, true, false, true, false, false);
            };
            // need to change listen before clearing interrupt, otherwise will fire
            // immediately again.
            gpio.clear_interrupt();
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
