/*

Early UART support.
It currently depends on GPIO pins and clock to be configured with default settings. 
(Tested for UART 0)

Also DPORT changes are made inside this peripheral: this should be moved to a dedicated 
dport driver as there is a risk for race conditions this way.

*/

use core::marker::PhantomData;

use embedded_hal::serial;

use crate::esp32::{DPORT, UART0, UART1, UART2};

const APB_CLK_FREQ: u32 = 40_000_000; // TODO: get clk frequency dynamically
const REF_CLK_FREQ: u32 = 1_000_000; // TODO: get clk frequency dynamically

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
    #[doc(hidden)]
    _Extensible,
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
        pub baudrate: u32,
        pub databits: DataBits,
        pub parity: Parity,
        pub stopbits: StopBits,
    }

    impl Config {
        pub fn baudrate(mut self, baudrate: u32) -> Self {
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

        pub fn databits(mut self, databits: DataBits) -> Self {
            self.databits = databits;
            self
        }

        pub fn stopbits(mut self, stopbits: StopBits) -> Self {
            self.stopbits = stopbits;
            self
        }
    }

    #[derive(Debug)]
    pub struct InvalidConfig;

    impl Default for Config {
        fn default() -> Config {
            Config {
                baudrate: 19_200,
                databits: DataBits::DataBits8,
                parity: Parity::ParityNone,
                stopbits: StopBits::STOP1,
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

/// Serial abstraction
pub struct Serial<UART, PINS> {
    uart: UART,
    pins: PINS,
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
            impl<PINS> Serial<$UARTX, PINS> {
                pub fn $uartX(
                    uart: $UARTX,
                    pins: PINS,
                    config: config::Config,
                ) -> Result<Self, config::InvalidConfig>
                where
                    PINS: Pins<$UARTX>,
                {
                    Ok(
                        Serial { uart, pins }
                            .reset()
                            .enable()
                            .change_baudrate(config.baudrate)
                            .change_stopbits(config.stopbits)
                            .change_databits(config.databits)
                            .change_parity(config.parity)

                    )
                }

                fn reset(self) -> Self {
                    let dportreg = unsafe{ &*DPORT::ptr() };
                    dportreg.perip_rst_en.modify(|_,w| w.$uartX().set_bit());
                    dportreg.perip_rst_en.modify(|_,w| w.$uartX().clear_bit());
                    self
                }

                pub fn enable(self) -> Self {
                    let dportreg = unsafe{ &*DPORT::ptr() };
                    dportreg.perip_clk_en.modify(|_,w| w.uart_mem().set_bit());
                    dportreg.perip_clk_en.modify(|_,w| w.$uartX().set_bit());
                    dportreg.perip_rst_en.modify(|_,w| w.$uartX().clear_bit());
                    self
                }

                pub fn disable(self) -> Self {
                    let dportreg = unsafe{ &*DPORT::ptr() };
                    dportreg.perip_clk_en.modify(|_,w| w.$uartX().clear_bit());
                    dportreg.perip_rst_en.modify(|_,w| w.$uartX().set_bit());

                    if     dportreg.perip_clk_en.read().uart0().bit_is_clear()
                        && dportreg.perip_clk_en.read().uart1().bit_is_clear()
                        && dportreg.perip_clk_en.read().uart2().bit_is_clear()
                    {
                        dportreg.perip_clk_en.modify(|_,w| w.uart_mem().clear_bit());
                    }
                    self
                }


                fn change_stopbits(self, stopbits: config::StopBits) -> Self {
                    let uartreg = unsafe{ &*$UARTX::ptr() };

                    //workaround for hardware issue, when UART stop bit set as 2-bit mode.
                    uartreg.rs485_conf.modify(|_,w|
                        w.dl1_en().bit(stopbits==config::StopBits::STOP2)
                    );

                    uartreg.conf0.modify(|_,w|
                        match stopbits {
                            config::StopBits::STOP1 => w.stop_bit_num().stop_bits_1(),
                            config::StopBits::STOP1P5 => w.stop_bit_num().stop_bits_1p5(),
                            //workaround for hardware issue, when UART stop bit set as 2-bit mode.
                            config::StopBits::STOP2 => w.stop_bit_num().stop_bits_1(),
                        }
                    );

                    self
                }

                fn change_databits(self, databits: config::DataBits) -> Self {
                    let uartreg = unsafe{ &*$UARTX::ptr() };

                    uartreg.conf0.modify(|_,w|
                        match databits {
                            config::DataBits::DataBits5 => w.bit_num().data_bits_5(),
                            config::DataBits::DataBits6 => w.bit_num().data_bits_6(),
                            config::DataBits::DataBits7 => w.bit_num().data_bits_7(),
                            config::DataBits::DataBits8 => w.bit_num().data_bits_8(),
                        }
                    );

                    self
                }

                fn change_parity(self, parity: config::Parity) -> Self {
                    let uartreg = unsafe{ &*$UARTX::ptr() };

                    uartreg.conf0.modify(|_,w|
                        match parity {
                            config::Parity::ParityNone => w.parity_en().clear_bit(),
                            config::Parity::ParityEven => w.parity_en().set_bit().parity().clear_bit(),
                            config::Parity::ParityOdd => w.parity_en().set_bit().parity().set_bit(),
                        }
                    );

                    self
                }


                fn change_baudrate(self, baudrate: u32) -> Self {
                    let uartreg = unsafe{ &*$UARTX::ptr() };

                    let tick_ref_always_on = uartreg.conf0.read().tick_ref_always_on().bit_is_set();
                    let sclk_freq = if tick_ref_always_on {APB_CLK_FREQ} else {REF_CLK_FREQ};
                    let clk_div = (sclk_freq*16)/baudrate;

                    unsafe {uartreg.clkdiv.modify(|_, w| w
                        .clkdiv().bits(clk_div>>4)
                        .clkdiv_frag().bits((clk_div&0xf) as u8)
                    )};

                    self
                }

                pub fn get_baudrate(&self) -> u32 {
                    let uartreg = unsafe{ &*$UARTX::ptr() };

                    let tick_ref_always_on = uartreg.conf0.read().tick_ref_always_on().bit_is_set();
                    let sclk_freq = if tick_ref_always_on {APB_CLK_FREQ} else {REF_CLK_FREQ};

                    return (sclk_freq<<4)/(uartreg.clkdiv.read().clkdiv().bits()<<4 | (uartreg.clkdiv.read().clkdiv_frag().bits() as u32))
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
                    unsafe { (*$UARTX::ptr()).status.read().st_urx_out().is_rx_idle() }
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

            impl<PINS> serial::Read<u8> for Serial<$UARTX, PINS> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    let mut rx: Rx<$UARTX> = Rx {
                        _uart: PhantomData,
                    };
                    rx.read()
                }
            }


            impl  Rx<$UARTX> {
                pub fn count(& self) -> u16 {
                    unsafe {
                        ((*$UARTX::ptr()).mem_cnt_status.read().rx_mem_cnt().bits() as u16) << 8
                            | (*$UARTX::ptr()).status.read().rxfifo_cnt().bits() as u16
                        }
                }
            }

            impl  Tx<$UARTX> {
                pub fn count(& self) -> u16 {
                    unsafe {
                        ((*$UARTX::ptr()).mem_cnt_status.read().tx_mem_cnt().bits() as u16) << 8
                            | (*$UARTX::ptr()).status.read().txfifo_cnt().bits() as u16
                        }
                }
            }

            impl serial::Read<u8> for Rx<$UARTX> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {

                    if self.count()==0 {
                        Err(nb::Error::WouldBlock)
                    } else {
                        Ok(unsafe { (*$UARTX::ptr()).rx_fifo.read().bits() })
                    }
                }
            }

            impl<PINS> serial::Write<u8> for Serial<$UARTX, PINS> {
                type Error = Error;

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
                type Error = Error;

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
