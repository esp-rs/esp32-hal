//! UART peripheral control
//!
//! Controls the 3 uart peripherals (UART0, UART1, UART2).
//!
//! # Example
//!
//! Creation of the serial peripheral and writing formatted info.
//! ```
//! let serial: Serial<_, _, _> = Serial::new(
//!     dp.UART0,
//!     esp32_hal::serial::Pins {
//!         tx: gpios.gpio1,
//!         rx: gpios.gpio3,
//!         cts: None,
//!         rts: None,
//!     },
//!     config,
//!     clkcntrl_config,
//!     &mut dport,
//!     )
//!     .unwrap();
//!
//! writeln!(serial, "Serial output").unwrap();
//! ```
//!
//! # TODO
//! - Add all extra features esp32 supports (eg rs485, etc. etc.)
//! - Free APB lock when TX is idle (and no RX used)
//! - Address errata 3.17: UART fifo_cnt is inconsistent with FIFO pointer

use core::{convert::Infallible, marker::PhantomData};

use crate::gpio::{InputPin, OutputPin};
use crate::prelude::*;
use crate::target;

use embedded_hal::serial;

const UART_FIFO_SIZE: u8 = 128;

/// Serial error
#[derive(Debug)]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
    /// Baudrate too low
    BaudrateTooLow,
    /// Baudrate too high
    BaudrateTooHigh,
}

/// Interrupt event
pub enum Event {
    /// New data has been received
    Rxne,
    /// New data can be sent
    Txe,
    /// Idle line state detected
    Idle,
}

/// UART configuration
pub mod config {
    use crate::units::*;

    /// Number of data bits
    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    pub enum DataBits {
        DataBits5,
        DataBits6,
        DataBits7,
        DataBits8,
    }

    /// Parity check
    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    pub enum Parity {
        ParityNone,
        ParityEven,
        ParityOdd,
    }

    /// Number of stop bits
    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    pub enum StopBits {
        /// 1 stop bit
        STOP1,
        /// 1.5 stop bits
        STOP1P5,
        /// 2 stop bits
        STOP2,
    }

    /// UART configuration
    #[derive(Debug, Copy, Clone)]
    pub struct Config {
        pub baudrate: Hertz,
        pub data_bits: DataBits,
        pub parity: Parity,
        pub stop_bits: StopBits,
    }

    impl Config {
        pub fn baudrate(mut self, baudrate: Hertz) -> Self {
            self.baudrate = baudrate;
            self
        }

        pub fn parity_none(mut self) -> Self {
            self.parity = Parity::ParityNone;
            self
        }

        pub fn parity_even(mut self) -> Self {
            self.parity = Parity::ParityEven;
            self
        }

        pub fn parity_odd(mut self) -> Self {
            self.parity = Parity::ParityOdd;
            self
        }

        pub fn data_bits(mut self, data_bits: DataBits) -> Self {
            self.data_bits = data_bits;
            self
        }

        pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
            self.stop_bits = stop_bits;
            self
        }
    }

    impl Default for Config {
        fn default() -> Config {
            Config {
                baudrate: Hertz(19_200),
                data_bits: DataBits::DataBits8,
                parity: Parity::ParityNone,
                stop_bits: StopBits::STOP1,
            }
        }
    }
}

/// Pins used by the UART interface
///
/// Note that any two pins may be used
pub struct Pins<
    TX: OutputPin,
    RX: InputPin,
    // default pins to allow type inference
    CTS: InputPin = crate::gpio::Gpio19<crate::gpio::Input<crate::gpio::Floating>>,
    RTS: OutputPin = crate::gpio::Gpio22<crate::gpio::Output<crate::gpio::PushPull>>,
> {
    pub tx: TX,
    pub rx: RX,
    pub cts: Option<CTS>,
    pub rts: Option<RTS>,
}

use private::Instance;

/// Serial abstraction
///
pub struct Serial<
    UART: Instance,
    TX: OutputPin,
    RX: InputPin,
    // default pins to allow type inference
    CTS: InputPin = crate::gpio::Gpio19<crate::gpio::Input<crate::gpio::Floating>>,
    RTS: OutputPin = crate::gpio::Gpio22<crate::gpio::Output<crate::gpio::PushPull>>,
> {
    uart: UART,
    pins: Pins<TX, RX, CTS, RTS>,
    clock_control: crate::clock_control::ClockControlConfig,
    rx: Rx<UART>,
    tx: Tx<UART>,
}

/// Serial receiver
pub struct Rx<UART: Instance> {
    _uart: PhantomData<UART>,
    apb_lock: Option<crate::clock_control::dfs::LockAPB>,
}

/// Serial transmitter
pub struct Tx<UART: Instance> {
    _uart: PhantomData<UART>,
    apb_lock: Option<crate::clock_control::dfs::LockAPB>,
}

impl<UART: Instance, TX: OutputPin, RX: InputPin, CTS: InputPin, RTS: OutputPin>
    Serial<UART, TX, RX, CTS, RTS>
{
    /// Create a new serial driver
    pub fn new(
        uart: UART,
        pins: Pins<TX, RX, CTS, RTS>,
        config: config::Config,
        clock_control: crate::clock_control::ClockControlConfig,
        dport: &mut target::DPORT,
    ) -> Result<Self, Error> {
        let mut serial = Serial {
            uart,
            pins,
            clock_control,
            rx: Rx {
                _uart: PhantomData,
                apb_lock: None,
            },
            tx: Tx {
                _uart: PhantomData,
                apb_lock: None,
            },
        };

        serial.uart.init_pins(&mut serial.pins);
        serial.uart.reset(dport).enable(dport);
        serial
            .change_stop_bits(config.stop_bits)
            .change_data_bits(config.data_bits)
            .change_parity(config.parity)
            .change_baudrate(config.baudrate)?;
        Ok(serial)
    }

    /// Change the number of stop bits
    pub fn change_stop_bits(&mut self, stop_bits: config::StopBits) -> &mut Self {
        //workaround for hardware issue, when UART stop bit set as 2-bit mode.
        self.uart
            .rs485_conf
            .modify(|_, w| w.dl1_en().bit(stop_bits == config::StopBits::STOP2));

        self.uart.conf0.modify(|_, w| match stop_bits {
            config::StopBits::STOP1 => w.stop_bit_num().stop_bits_1(),
            config::StopBits::STOP1P5 => w.stop_bit_num().stop_bits_1p5(),
            //workaround for hardware issue, when UART stop bit set as 2-bit mode.
            config::StopBits::STOP2 => w.stop_bit_num().stop_bits_1(),
        });

        self
    }

    /// Change the number of data bits
    pub fn change_data_bits(&mut self, data_bits: config::DataBits) -> &mut Self {
        self.uart.conf0.modify(|_, w| match data_bits {
            config::DataBits::DataBits5 => w.bit_num().data_bits_5(),
            config::DataBits::DataBits6 => w.bit_num().data_bits_6(),
            config::DataBits::DataBits7 => w.bit_num().data_bits_7(),
            config::DataBits::DataBits8 => w.bit_num().data_bits_8(),
        });

        self
    }

    /// Change the type of parity checking
    pub fn change_parity(&mut self, parity: config::Parity) -> &mut Self {
        self.uart.conf0.modify(|_, w| match parity {
            config::Parity::ParityNone => w.parity_en().clear_bit(),
            config::Parity::ParityEven => w.parity_en().set_bit().parity().clear_bit(),
            config::Parity::ParityOdd => w.parity_en().set_bit().parity().set_bit(),
        });

        self
    }

    /// Change the baudrate.
    ///
    /// Will automatically select the clock source. When possible the reference clock (1MHz) will
    /// be used, because this is constant when the clock source/frequency changes.
    /// However if one of the clock frequencies is below 10MHz or if the baudrate is above
    /// the reference clock or if the baudrate cannot be set within 1.5%
    /// then use the APB clock.
    pub fn change_baudrate<T: Into<Hertz> + Copy>(
        &mut self,
        baudrate: T,
    ) -> Result<&mut Self, Error> {
        let mut use_apb_frequency = false;

        // if APB frequency is <10MHz the ref clock is no longer accurate
        // or if the baudrate > Ref frequency then use the APB frequency
        if !self.clock_control.is_ref_clock_stable()
            || baudrate.into() > self.clock_control.ref_frequency()
        {
            use_apb_frequency = true;
        } else if baudrate.into() < self.clock_control.apb_frequency_apb_locked() / (1 << 20 - 1) {
            // if baudrate is lower then can be achieved via the APB frequency
            use_apb_frequency = false;
        } else {
            let clk_div =
                (self.clock_control.ref_frequency() * 16 + baudrate.into() / 2) / baudrate.into();
            // if baudrate too high use APB clock
            if clk_div == 0 {
                use_apb_frequency = true
            } else {
                // if baudrate cannot be reached within 1.5% use APB frequency
                // use 203 as multiplier (2*101.5), because 1Mhz * 16 * 203 still fits in 2^32
                let calc_baudrate = (self.clock_control.ref_frequency() * 16 * 200) / clk_div;
                if calc_baudrate > baudrate.into() * 203 || calc_baudrate < baudrate.into() * 197 {
                    use_apb_frequency = true;
                }
            }
        }

        self.change_baudrate_force_clock(baudrate, use_apb_frequency)
    }

    /// Change the baudrate choosing the reference or APB clock manually
    pub fn change_baudrate_force_clock<T: Into<Hertz> + Copy>(
        &mut self,
        baudrate: T,
        use_apb_frequency: bool,
    ) -> Result<&mut Self, Error> {
        if let None = self.rx.apb_lock {
            if use_apb_frequency {
                self.rx.apb_lock = Some(self.clock_control.lock_apb_frequency());
                self.tx.apb_lock = Some(self.clock_control.lock_apb_frequency());
            }
        } else {
            if !use_apb_frequency {
                self.rx.apb_lock = None;
                self.tx.apb_lock = None;
            }
        }

        // set clock source
        self.uart
            .conf0
            .modify(|_, w| w.tick_ref_always_on().bit(use_apb_frequency));

        let sclk_freq = if use_apb_frequency {
            self.clock_control.apb_frequency_apb_locked()
        } else {
            self.clock_control.ref_frequency()
        };

        // calculate nearest divider
        let clk_div = (sclk_freq * 16 + baudrate.into() / 2) / baudrate.into();

        if clk_div == 0 {
            return Err(Error::BaudrateTooHigh);
        }
        if clk_div > (1 << 24) - 1 {
            return Err(Error::BaudrateTooLow);
        }

        unsafe {
            self.uart.clkdiv.modify(|_, w| {
                w.clkdiv()
                    .bits(clk_div >> 4)
                    .clkdiv_frag()
                    .bits((clk_div & 0xf) as u8)
            })
        };

        Ok(self)
    }

    /// Returns if the reference or APB clock is used
    pub fn is_clock_apb(&self) -> bool {
        self.uart.conf0.read().tick_ref_always_on().bit_is_set()
    }

    /// Returns the current baudrate
    pub fn baudrate(&self) -> Hertz {
        let use_apb_frequency = self.uart.conf0.read().tick_ref_always_on().bit_is_set();
        let sclk_freq = if use_apb_frequency {
            self.clock_control.apb_frequency()
        } else {
            self.clock_control.ref_frequency()
        };
        let div = self.uart.clkdiv.read().clkdiv().bits() << 4
            | (self.uart.clkdiv.read().clkdiv_frag().bits() as u32);

        // round to nearest integer baudrate
        (sclk_freq * 16 + Hertz(div / 2)) / div
    }

    /// Starts listening for an interrupt event
    pub fn listen(&mut self, _event: Event) {
        unimplemented!();
    }

    /// Stop listening for an interrupt event
    pub fn unlisten(&mut self, _event: Event) {
        unimplemented!();
    }

    /// Return true if the receiver is idle
    pub fn is_rx_idle(&self) -> bool {
        self.uart.status.read().st_urx_out().is_rx_idle()
    }

    /// Return true if the transmitter is idle
    pub fn is_tx_idle(&self) -> bool {
        self.uart.status.read().st_utx_out().is_tx_idle()
    }

    /// Split the serial driver in separate TX and RX drivers
    pub fn split(self) -> (Tx<UART>, Rx<UART>) {
        (self.tx, self.rx)
    }

    /// Release the UART and GPIO resources
    pub fn release(self) -> (UART, Pins<TX, RX, CTS, RTS>) {
        (self.uart, self.pins)
    }
}

impl<UART: Instance, TX: OutputPin, RX: InputPin, CTS: InputPin, RTS: OutputPin> serial::Read<u8>
    for Serial<UART, TX, RX, CTS, RTS>
{
    type Error = Infallible;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.rx.read()
    }
}

impl<UART: Instance, TX: OutputPin, RX: InputPin, CTS: InputPin, RTS: OutputPin> serial::Write<u8>
    for Serial<UART, TX, RX, CTS, RTS>
{
    type Error = Infallible;

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.flush()
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        self.tx.write(byte)
    }
}

impl<UART: Instance, TX: OutputPin, RX: InputPin, CTS: InputPin, RTS: OutputPin> core::fmt::Write
    for Serial<UART, TX, RX, CTS, RTS>
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        use embedded_hal::serial::Write;
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

impl<UART: Instance> Rx<UART> {
    /// Get count of bytes in the receive FIFO
    pub fn count(&self) -> u8 {
        unsafe { (*UART::ptr()).status.read().rxfifo_cnt().bits() }
    }

    /// Check if the receivers is idle
    pub fn is_idle(&self) -> bool {
        unsafe { (*UART::ptr()).status.read().st_urx_out().is_rx_idle() }
    }
}

impl<UART: Instance> serial::Read<u8> for Rx<UART> {
    type Error = Infallible;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.count() == 0 {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(unsafe { (*UART::ptr()).rx_fifo.read().bits() })
        }
    }
}

impl<UART: Instance> Tx<UART> {
    /// Get count of bytes in the transmitter FIFO
    pub fn count(&self) -> u8 {
        unsafe { (*UART::ptr()).status.read().txfifo_cnt().bits() }
    }

    /// Check if the transmitter is idle
    pub fn is_idle(&self) -> bool {
        unsafe { (*UART::ptr()).status.read().st_utx_out().is_tx_idle() }
    }
}

impl<UART: Instance> serial::Write<u8> for Tx<UART> {
    type Error = Infallible;

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        if self.is_idle() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        if self.count() < UART_FIFO_SIZE {
            unsafe { (*UART::ptr()).tx_fifo.write_with_zero(|w| w.bits(byte)) }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<UART: Instance> core::fmt::Write for Tx<UART>
where
    Tx<UART>: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        use embedded_hal::serial::Write;
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

mod private {

    use super::Pins;
    use crate::gpio::{InputPin, InputSignal, OutputPin, OutputSignal};
    use crate::prelude::*;
    use crate::target::{self, uart, UART0, UART1, UART2};
    use core::ops::Deref;

    pub trait Instance: Deref<Target = uart::RegisterBlock> {
        fn ptr() -> *const uart::RegisterBlock;
        /// Enable peripheral
        fn enable(&mut self, dport: &mut target::DPORT) -> &mut Self;
        /// Disable peripheral
        fn disable(&mut self, dport: &mut target::DPORT) -> &mut Self;
        /// Reset peripheral
        fn reset(&mut self, dport: &mut target::DPORT) -> &mut Self;

        /// Initialize pins
        fn init_pins<TX: OutputPin, RX: InputPin, CTS: InputPin, RTS: OutputPin>(
            &mut self,
            pins: &mut Pins<TX, RX, CTS, RTS>,
        ) -> &mut Self;
    }

    static UART_MEM_LOCK: CriticalSectionSpinLockMutex<()> = CriticalSectionSpinLockMutex::new(());

    macro_rules! halUart {
        ($(
            $UARTX:ident: ($uartX:ident, $txd:ident, $rxd:ident, $cts:ident, $rts:ident),
        )+) => {
            $(
                impl Instance for $UARTX {
                    fn ptr() -> *const uart::RegisterBlock {
                        $UARTX::ptr()
                    }

                    fn reset(&mut self, dport: &mut target::DPORT) -> &mut Self {
                        dport.perip_rst_en.modify(|_, w| w.$uartX().set_bit());
                        dport.perip_rst_en.modify(|_, w| w.$uartX().clear_bit());
                        self
                    }

                    fn enable(&mut self, dport: &mut target::DPORT) -> &mut Self {
                        dport.perip_clk_en.modify(|_, w| w.uart_mem().set_bit());
                        dport.perip_clk_en.modify(|_, w| w.$uartX().set_bit());
                        dport.perip_rst_en.modify(|_, w| w.$uartX().clear_bit());
                        self
                    }

                    fn disable(&mut self, dport: &mut target::DPORT) -> &mut Self {
                        dport.perip_clk_en.modify(|_, w| w.$uartX().clear_bit());
                        dport.perip_rst_en.modify(|_, w| w.$uartX().set_bit());

                        (&UART_MEM_LOCK).lock(|_| {
                            if dport.perip_clk_en.read().uart0().bit_is_clear()
                                && dport.perip_clk_en.read().uart1().bit_is_clear()
                                && dport.perip_clk_en.read().uart2().bit_is_clear()
                            {
                                dport.perip_clk_en.modify(|_, w| w.uart_mem().clear_bit());
                            }
                        });
                        self

                    }

                    fn init_pins<TX: OutputPin, RX: InputPin, CTS: InputPin, RTS: OutputPin>(
                        &mut self, pins: &mut Pins<TX,RX,CTS,RTS>
                    ) -> &mut Self {
                        pins
                            .tx
                            .set_to_push_pull_output()
                            .connect_peripheral_to_output(OutputSignal::$txd);

                        pins
                            .rx
                            .set_to_input()
                            .connect_input_to_peripheral(InputSignal::$rxd);

                        if let Some(cts) = pins.cts.as_mut() {
                            cts
                            .set_to_input()
                            .connect_input_to_peripheral(InputSignal::$cts);
                        }

                        if let Some(rts) = pins.rts.as_mut() {
                            rts
                            .set_to_push_pull_output()
                            .connect_peripheral_to_output(OutputSignal::$rts);
                        }
                        self
                    }
                }
            )+
        }
    }

    halUart! {
        UART0: (uart0, U0TXD, U0RXD, U0CTS, U0RTS),
        UART1: (uart1, U1TXD, U1RXD, U1CTS, U1RTS),
        UART2: (uart2, U2TXD, U2RXD, U2CTS, U2RTS),
    }
}
