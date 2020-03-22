//! UART peripheral control
//!
//! Controls the 3 uart peripherals (UART0, UART1, UART2)
//!
//! **It currently depends on GPIO pins and clock to be configured with default settings.**
//! (Tested for UART 0)
//!
//! # TODO
//! - Automatic GPIO configuration
//! - Use clock_control for clock frequency detection
//! - Add all extra features esp32 supports (eg rs485, etc. etc.)
//! - Create separate dport peripheral (as otherwise risk for race conditions)

use core::convert::Infallible;
use core::marker::PhantomData;

use embedded_hal::serial;

use crate::esp32::{UART0, UART1, UART2};
use crate::units::*;

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

pub mod config {
    use crate::units::*;

    #[derive(PartialEq, Eq)]
    pub enum DataBits {
        DataBits5,
        DataBits6,
        DataBits7,
        DataBits8,
    }

    #[derive(PartialEq, Eq)]
    pub enum Parity {
        ParityNone,
        ParityEven,
        ParityOdd,
    }

    #[derive(PartialEq, Eq)]
    pub enum StopBits {
        #[doc = "1 stop bit"]
        STOP1,
        #[doc = "1.5 stop bits"]
        STOP1P5,
        #[doc = "2 stop bits"]
        STOP2,
    }

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

pub trait Pins<UART> {}
pub trait PinTx<UART> {}
pub trait PinRx<UART> {}

impl<UART, TX, RX> Pins<UART> for (TX, RX)
where
    TX: PinTx<UART>,
    RX: PinRx<UART>,
{
}

/// A filler type for when the Tx pin is unnecessary
pub struct NoTx;
/// A filler type for when the Rx pin is unnecessary
pub struct NoRx;

impl PinTx<UART0> for NoTx {}
impl PinRx<UART0> for NoRx {}
impl PinTx<UART1> for NoTx {}
impl PinRx<UART1> for NoRx {}
impl PinTx<UART2> for NoTx {}
impl PinRx<UART2> for NoRx {}

/// Serial abstraction
///
pub struct Serial<UART, PINS> {
    uart: UART,
    pins: PINS,
    clock_control: crate::clock_control::ClockControlConfig,
}

/// Serial receiver
pub struct Rx<UART> {
    _uart: PhantomData<UART>,
}

/// Serial transmitter
pub struct Tx<UART> {
    _uart: PhantomData<UART>,
}

macro_rules! halUart {
    ($(
        $UARTX:ident: ($uartX:ident),
    )+) => {
        $(
            impl<'a, PINS> Serial<$UARTX, PINS> {
                pub fn $uartX(
                    uart: $UARTX,
                    pins: PINS,
                    config: config::Config,
                    clock_control:  crate::clock_control::ClockControlConfig,
                    dport: &mut esp32::DPORT
                ) -> Result<Self, Error>
                where
                    PINS: Pins<$UARTX>,
                {
                        let mut serial=Serial { uart, pins, clock_control };

                        serial
                            .reset(dport)
                            .enable(dport)
                            .change_stop_bits(config.stop_bits)
                            .change_data_bits(config.data_bits)
                            .change_parity(config.parity)
                            .change_baudrate(config.baudrate)?;

                        Ok(serial)
                }

                fn reset(&mut self, dport:&mut esp32::DPORT) -> &mut Self {
                    dport.perip_rst_en.modify(|_,w| w.$uartX().set_bit());
                    dport.perip_rst_en.modify(|_,w| w.$uartX().clear_bit());
                    self
                }

                pub fn enable(&mut self, dport:&mut esp32::DPORT) -> &mut Self {
                    dport.perip_clk_en.modify(|_,w| w.uart_mem().set_bit());
                    dport.perip_clk_en.modify(|_,w| w.$uartX().set_bit());
                    dport.perip_rst_en.modify(|_,w| w.$uartX().clear_bit());
                    self
                }

                pub fn disable(&mut self, dport:&mut esp32::DPORT) -> &mut Self {
                    dport.perip_clk_en.modify(|_,w| w.$uartX().clear_bit());
                    dport.perip_rst_en.modify(|_,w| w.$uartX().set_bit());

                    if     dport.perip_clk_en.read().uart0().bit_is_clear()
                        && dport.perip_clk_en.read().uart1().bit_is_clear()
                        && dport.perip_clk_en.read().uart2().bit_is_clear()
                    {
                        dport.perip_clk_en.modify(|_,w| w.uart_mem().clear_bit());
                    }
                    self
                }


                pub fn change_stop_bits(&mut self, stop_bits: config::StopBits) -> &mut Self {

                    //workaround for hardware issue, when UART stop bit set as 2-bit mode.
                    self.uart.rs485_conf.modify(|_,w|
                        w.dl1_en().bit(stop_bits==config::StopBits::STOP2)
                    );

                    self.uart.conf0.modify(|_,w|
                        match stop_bits {
                            config::StopBits::STOP1 => w.stop_bit_num().stop_bits_1(),
                            config::StopBits::STOP1P5 => w.stop_bit_num().stop_bits_1p5(),
                            //workaround for hardware issue, when UART stop bit set as 2-bit mode.
                            config::StopBits::STOP2 => w.stop_bit_num().stop_bits_1(),
                        }
                    );

                    self
                }

                pub fn change_data_bits(&mut self, data_bits: config::DataBits) -> &mut Self {

                    self.uart.conf0.modify(|_,w|
                        match data_bits {
                            config::DataBits::DataBits5 => w.bit_num().data_bits_5(),
                            config::DataBits::DataBits6 => w.bit_num().data_bits_6(),
                            config::DataBits::DataBits7 => w.bit_num().data_bits_7(),
                            config::DataBits::DataBits8 => w.bit_num().data_bits_8(),
                        }
                    );

                    self
                }

                pub fn change_parity(&mut self, parity: config::Parity) -> &mut Self {

                    self.uart.conf0.modify(|_,w|
                        match parity {
                            config::Parity::ParityNone => w.parity_en().clear_bit(),
                            config::Parity::ParityEven => w.parity_en().set_bit().parity().clear_bit(),
                            config::Parity::ParityOdd => w.parity_en().set_bit().parity().set_bit(),
                        }
                    );

                    self
                }

                pub fn change_baudrate <T: Into<Hertz> + Copy>(&mut self, baudrate: T) -> Result<&mut Self,Error> {
                    let mut use_apb_frequency = false;

                    // if APB frequency is <10MHz (according to documentation, in practice 5MHz),
                    // the ref clock is no longer accurate or if the baudrate > Ref frequency
                    if self.clock_control.max_apb_frequency() < 10_000_000.Hz()
                            || baudrate.into() > self.clock_control.ref_frequency() {
                        use_apb_frequency = true;
                    } else if baudrate.into() < self.clock_control.max_apb_frequency()/(1<<20-1) {
                        // if baudrate is lower then can be achieved via the APB frequency
                        use_apb_frequency = false;
                    }
                    else {
                        let clk_div =
                            (self.clock_control.ref_frequency() * 16 + baudrate.into() / 2) / baudrate.into();
                        // if baudrate too high use APB clock
                        if clk_div == 0 {
                            use_apb_frequency = true
                        } else {
                            // if baudrate cannot be reached within 1.5% use APB frequency
                            // use 203 as multiplier (2*101.5), because 1Mhz * 16 * 203 still fits in 2^32
                            let calc_baudrate = (self.clock_control.ref_frequency() * 16 * 200) / clk_div;
                            if calc_baudrate > baudrate.into() * 203
                                || calc_baudrate < baudrate.into() * 197
                            {
                                use_apb_frequency = true;
                            }
                        }
                    }

                    // set clock source
                    self.uart.conf0.modify(|_, w| w.tick_ref_always_on().bit(use_apb_frequency));

                    let sclk_freq = if use_apb_frequency {self.clock_control.max_apb_frequency()} else {self.clock_control.ref_frequency()};

                    // calculate nearest divider
                    let clk_div = (sclk_freq * 16 + baudrate.into()/2 ) / baudrate.into();

                    if clk_div == 0 {
                        return Err(Error::BaudrateTooHigh)
                    }
                    if clk_div > (1<<24)-1 {
                        return Err(Error::BaudrateTooLow)
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

                pub fn get_baudrate(& self) -> Hertz {
                    let use_apb_frequency = self.uart.conf0.read().tick_ref_always_on().bit_is_set();
                    let sclk_freq = if use_apb_frequency {self.clock_control.apb_frequency()} else {self.clock_control.ref_frequency()};
                    let div = self.uart.clkdiv.read().clkdiv().bits()<<4 | (self.uart.clkdiv.read().clkdiv_frag().bits() as u32);

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

                /// Return true if the line idle status is set
                pub fn is_idle(& self) -> bool {
                    self.uart.status.read().st_urx_out().is_rx_idle()
                }

                pub fn split(self) -> (Tx<$UARTX>, Rx<$UARTX>) {
                    (
                        Tx {
                            _uart: PhantomData,
                        },
                        Rx {
                            _uart: PhantomData,
                        },
                    )
                }

                pub fn release(self) -> ($UARTX, PINS) {
                    (self.uart, self.pins)
                }

            }

            impl<'a, PINS> serial::Read<u8> for Serial<$UARTX, PINS> {
                type Error = Infallible;

                fn read(&mut self) -> nb::Result<u8, Self::Error> {
                    let mut rx: Rx<$UARTX> = Rx {
                        _uart: PhantomData,
                    };
                    rx.read()
                }
            }


            impl  Rx<$UARTX> {
                pub fn count(& self) -> u8 {
                    unsafe {
                            (*$UARTX::ptr()).status.read().rxfifo_cnt().bits()
                        }
                }
            }

            impl  Tx<$UARTX> {
                pub fn count(& self) -> u8 {
                    unsafe {
                            (*$UARTX::ptr()).status.read().txfifo_cnt().bits()
                        }
                }
            }

            impl serial::Read<u8> for Rx<$UARTX> {
                type Error = Infallible;

                fn read(&mut self) -> nb::Result<u8, Self::Error> {

                    if self.count()==0 {
                        Err(nb::Error::WouldBlock)
                    } else {
                        Ok(unsafe { (*$UARTX::ptr()).rx_fifo.read().bits() })
                    }
                }
            }

            impl<'a, PINS> serial::Write<u8> for Serial<$UARTX, PINS> {
                type Error = Infallible;

                fn flush(&mut self) -> nb::Result<(), Self::Error> {
                    let mut tx: Tx<$UARTX> = Tx {
                        _uart: PhantomData,
                    };
                    tx.flush()
                }

                fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
                    let mut tx: Tx<$UARTX> = Tx {
                        _uart: PhantomData,
                    };
                    tx.write(byte)
                }
            }

            impl serial::Write<u8> for Tx<$UARTX> {
                type Error = Infallible;

                fn flush(&mut self) -> nb::Result<(), Self::Error> {
                    if self.count()==0 {
                        Ok(())
                    }
                    else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
                    if self.count()<128 {
                        unsafe { (*$UARTX::ptr()).tx_fifo.write_with_zero(|w| { w.bits(byte)}) }
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }
        )+
    }
}

impl<UART> core::fmt::Write for Tx<UART>
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

halUart! {
    UART0: (uart0),
    UART1: (uart1),
    UART2: (uart2),
}
