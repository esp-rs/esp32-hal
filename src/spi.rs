//! SPI peripheral control
//!
//! Currently only implements full duplex controller mode support.
//!
//! SPI0 is reserved for accessing flash and sram and therefore not usable for other purposes.
//! SPI1 shares its external pins with SPI0 and therefore has severe restrictions in use.
//!
//! SPI2 &3 can be used freely.
//!
//! The CS pin is controlled by hardware on esp32 (contrary to the description of embedded_hal).
//!
//! The [Transfer::transfer], [Write::write] and [WriteIter::write_iter] functions lock the
//! APB frequency and therefore the requests are always run at the requested baudrate.
//! The primitive [FullDuplex::read] and [FullDuplex::send] do not lock the APB frequency and
//! therefore may run at a different frequency.
//!
//! # TODO
//! - Quad SPI
//! - Half Duplex
//! - DMA
//! - Multiple CS pins

use crate::prelude::*;

use {
    crate::{
        clock_control::ClockControlConfig,
        gpio::{self, InputPin, OutputPin},
        target::{SPI1, SPI2, SPI3},
    },
    core::convert::TryInto,
    embedded_hal::blocking::spi::{Transfer, Write, WriteIter},
    embedded_hal::spi::FullDuplex,
};

use private::Instance;

/// SPI Errors
#[derive(Debug)]
pub enum Error {
    BaudrateTooHigh,
    BaudrateTooLow,
    ConversionFailed,
    PinError,
}

/// Pins used by the SPI interface
pub struct Pins<
    SCLK: OutputPin,
    SDO: OutputPin,
    // default pins to allow type inference
    SDI: InputPin + OutputPin = crate::gpio::Gpio1<crate::gpio::Input<crate::gpio::Floating>>,
    CS: OutputPin = crate::gpio::Gpio2<crate::gpio::Output<crate::gpio::PushPull>>,
> {
    pub sclk: SCLK,
    pub sdo: SDO,
    pub sdi: Option<SDI>,
    pub cs: Option<CS>,
}

/// SPI configuration
pub mod config {
    use crate::units::*;
    pub use embedded_hal::spi::{Mode, MODE_0, MODE_1, MODE_2, MODE_3};

    /// SPI Bit Order
    #[derive(PartialEq, Eq, Copy, Clone)]
    pub enum BitOrder {
        LSBFirst,
        MSBFirst,
    }

    /// SPI configuration
    #[derive(Copy, Clone)]
    pub struct Config {
        pub baudrate: Hertz,
        pub data_mode: embedded_hal::spi::Mode,
        pub bit_order: BitOrder,
    }

    impl Config {
        pub fn baudrate(mut self, baudrate: Hertz) -> Self {
            self.baudrate = baudrate;
            self
        }

        pub fn data_mode(mut self, data_mode: embedded_hal::spi::Mode) -> Self {
            self.data_mode = data_mode;
            self
        }

        pub fn bit_order(mut self, bit_order: BitOrder) -> Self {
            self.bit_order = bit_order;
            self
        }
    }

    impl Default for Config {
        fn default() -> Config {
            Config {
                baudrate: Hertz(1_000_000),
                data_mode: MODE_0,
                bit_order: BitOrder::LSBFirst,
            }
        }
    }
}

/// SPI abstraction
pub struct SPI<
    INSTANCE: Instance,
    SCLK: OutputPin,
    SDO: OutputPin,
    // default pins to allow type inference
    SDI: InputPin + OutputPin = crate::gpio::Gpio1<crate::gpio::Input<crate::gpio::Floating>>,
    CS: OutputPin = crate::gpio::Gpio2<crate::gpio::Output<crate::gpio::PushPull>>,
> {
    instance: INSTANCE,
    pins: Pins<SCLK, SDO, SDI, CS>,
    clock_control: ClockControlConfig,
}

impl<CS: OutputPin>
    SPI<
        SPI1,
        gpio::Gpio6<gpio::Output<gpio::PushPull>>,
        gpio::Gpio7<gpio::Output<gpio::PushPull>>,
        gpio::Gpio8<gpio::Input<gpio::Floating>>,
        CS,
    >
{
    /// Create new instance of SPI controller for SPI1
    ///
    /// SPI1 can only use fixed pin for SCLK, SDO and SDI as they are shared with SPI0.
    pub fn new(
        instance: SPI1,
        pins: Pins<
            gpio::Gpio6<gpio::Output<gpio::PushPull>>,
            gpio::Gpio7<gpio::Output<gpio::PushPull>>,
            gpio::Gpio8<gpio::Input<gpio::Floating>>,
            CS,
        >,
        config: config::Config,
        clock_control: ClockControlConfig,
    ) -> Result<Self, Error> {
        SPI::new_internal(instance, pins, config, clock_control)
    }
}

impl<SCLK: OutputPin, SDO: OutputPin, SDI: InputPin + OutputPin, CS: OutputPin>
    SPI<SPI2, SCLK, SDO, SDI, CS>
{
    /// Create new instance of SPI controller for SPI2
    pub fn new(
        instance: SPI2,
        pins: Pins<SCLK, SDO, SDI, CS>,
        config: config::Config,
        clock_control: ClockControlConfig,
    ) -> Result<Self, Error> {
        SPI::new_internal(instance, pins, config, clock_control)
    }
}

impl<SCLK: OutputPin, SDO: OutputPin, SDI: InputPin + OutputPin, CS: OutputPin>
    SPI<SPI3, SCLK, SDO, SDI, CS>
{
    /// Create new instance of SPI controller for SPI3
    pub fn new(
        instance: SPI3,
        pins: Pins<SCLK, SDO, SDI, CS>,
        config: config::Config,
        clock_control: ClockControlConfig,
    ) -> Result<Self, Error> {
        SPI::new_internal(instance, pins, config, clock_control)
    }
}

impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    /// Internal implementation of new shared by all SPI controllers
    fn new_internal(
        instance: INSTANCE,
        pins: Pins<SCLK, SDO, SDI, CS>,
        config: config::Config,
        clock_control: ClockControlConfig,
    ) -> Result<Self, Error> {
        let mut spi = SPI {
            instance,
            pins,
            clock_control,
        };

        spi.instance.init_pins(&mut spi.pins);
        spi.instance.reset().enable();

        // initialize registers to defaults (this is for a large part also done by the peripheral
        // reset), however SPI0 and 1 cannot be reset independently.
        spi.instance
            .slave
            .modify(|_, w| w.trans_done().clear_bit().slave_mode().clear_bit());

        unsafe {
            spi.instance.user.write(|w| {
                w.bits(0)
                    .usr_mosi()
                    .set_bit()
                    .usr_miso()
                    .set_bit()
                    .doutdin()
                    .set_bit()
                    .cs_setup()
                    .set_bit()
                    .cs_hold()
                    .set_bit()
            });
            spi.instance.user1.write(|w| w.bits(0));
            spi.instance.ctrl.write(|w| w.bits(0));
            spi.instance.ctrl1.write(|w| w.bits(0));
            spi.instance.ctrl2.write(|w| w.bits(0));
            spi.instance.clock.write(|w| w.bits(0));
        }

        spi.change_data_mode(config.data_mode)
            .change_bit_order(config.bit_order)
            .change_baudrate(config.baudrate)?;

        Ok(spi)
    }

    /// Convert SPI division factor back to frequency
    fn divider_to_frequency(apb_freq: Hertz, div1: u32, div2: u32) -> Hertz {
        apb_freq / ((div1 + 1) * (div2 + 1))
    }

    /// Change the SPI baudrate
    pub fn change_baudrate<T: Into<Hertz> + Copy>(
        &mut self,
        baudrate: T,
    ) -> Result<&mut Self, Error> {
        let baudrate = baudrate.into();
        let apb_freq = self.clock_control.apb_frequency_apb_locked();

        if baudrate > apb_freq {
            return Err(Error::BaudrateTooHigh);
        }

        if baudrate == apb_freq {
            self.instance.clock.write(|w| unsafe {
                w.clk_equ_sysclk()
                    .set_bit()
                    .clkdiv_pre()
                    .bits(0)
                    .clkcnt_n()
                    .bits(0)
                    .clkcnt_h()
                    .bits(0)
                    .clkcnt_l()
                    .bits(0)
            });
            return Ok(self);
        }

        if baudrate < Self::divider_to_frequency(apb_freq, 0x1fff, 0x3f) {
            return Err(Error::BaudrateTooLow);
        }

        let mut div1: u16 = 1;
        let mut div2: u8 = 1;
        let mut freq_best: Hertz = Hertz(0);

        'outer: for div2_guess in 1..=0x3f {
            for var in -2i32..=1 {
                let mut div1_guess = (((apb_freq / (div2_guess + 1)) / baudrate) as i32 - 1) + var;
                if div1_guess > 0x1fff {
                    div1_guess = 0x1fff;
                } else if div1_guess <= 0 {
                    div1_guess = 0;
                }
                let freq_guess =
                    Self::divider_to_frequency(apb_freq, div1_guess as u32, div2_guess);

                if freq_guess <= baudrate
                    && (u32::from(baudrate) as i32 - u32::from(freq_guess) as i32).abs()
                        < (u32::from(baudrate) as i32 - u32::from(freq_best) as i32).abs()
                {
                    freq_best = freq_guess;
                    div1 = div1_guess as u16;
                    div2 = div2_guess as u8;

                    if baudrate == freq_guess {
                        break 'outer;
                    }
                }
            }
        }

        self.instance.clock.write(|w| unsafe {
            w.clk_equ_sysclk()
                .clear_bit()
                .clkdiv_pre()
                .bits(div1)
                .clkcnt_n()
                .bits(div2)
                .clkcnt_l()
                .bits((div2 + 1) / 2)
                .clkcnt_h()
                .bits(0)
        });

        Ok(self)
    }

    /// Returns the current baudrate
    pub fn baudrate(&self) -> Hertz {
        if self.instance.clock.read().clk_equ_sysclk().bit_is_set() {
            self.clock_control.apb_frequency_apb_locked()
        } else {
            Self::divider_to_frequency(
                self.clock_control.apb_frequency_apb_locked(),
                self.instance.clock.read().clkdiv_pre().bits() as u32,
                self.instance.clock.read().clkcnt_n().bits() as u32,
            )
        }
    }

    /// Change the bit order
    pub fn change_bit_order(&mut self, data_mode: config::BitOrder) -> &mut Self {
        let spi = &self.instance;
        match data_mode {
            config::BitOrder::LSBFirst => spi
                .ctrl
                .modify(|_, w| w.wr_bit_order().set_bit().rd_bit_order().set_bit()),
            config::BitOrder::MSBFirst => spi
                .ctrl
                .modify(|_, w| w.wr_bit_order().clear_bit().rd_bit_order().clear_bit()),
        }
        self
    }

    /// Change the data mode
    pub fn change_data_mode(&mut self, data_mode: embedded_hal::spi::Mode) -> &mut Self {
        let spi = &self.instance;
        match data_mode {
            embedded_hal::spi::MODE_0 => {
                spi.pin.modify(|_, w| w.ck_idle_edge().clear_bit());
                spi.user.modify(|_, w| w.ck_out_edge().clear_bit());
            }
            embedded_hal::spi::MODE_1 => {
                spi.pin.modify(|_, w| w.ck_idle_edge().clear_bit());
                spi.user.modify(|_, w| w.ck_out_edge().set_bit());
            }
            embedded_hal::spi::MODE_2 => {
                spi.pin.modify(|_, w| w.ck_idle_edge().set_bit());
                spi.user.modify(|_, w| w.ck_out_edge().set_bit());
            }
            embedded_hal::spi::MODE_3 => {
                spi.pin.modify(|_, w| w.ck_idle_edge().set_bit());
                spi.user.modify(|_, w| w.ck_out_edge().clear_bit());
            }
        }
        self
    }

    /// Release and return the raw interface to the underlying SPI peripheral
    pub fn release(self) -> INSTANCE {
        self.instance
    }

    /// Generic transfer function
    ///
    /// This function locks the APB bus frequency and chunks the output
    /// for maximum write performance.
    fn transfer_internal<'a, T>(&mut self, words: &'a mut [T]) -> Result<&'a [T], Error>
    where
        T: U8orU16orU32,
    {
        let bytes = core::mem::size_of::<T>();
        let bits = bytes * 8;
        let divider = 4 / bytes;
        let buffer_item_count = 64 / bytes;

        let apb_lock = self.clock_control.lock_apb_frequency();

        let mut item_count = 0;
        let mut read_item_count = 0;

        while read_item_count < words.len() {
            // wait till SPI is finished with previous command
            while self.instance.cmd.read().usr().bit_is_set() {}

            // get data from previous SPI chunk
            if item_count > 0 {
                for count in 0..buffer_item_count {
                    words[read_item_count] = ((self.instance.w[count / divider].read().bits()
                        >> ((count % divider) * bits))
                        & ((!0u32) >> (32 - bits)))
                        .try_into()
                        .map_err(|_| Error::ConversionFailed)?;

                    read_item_count += 1;
                    if read_item_count >= words.len() {
                        break;
                    }
                }
            }

            if item_count < words.len() {
                // write next SPI chunk to buffer
                let mut count = 0;
                let mut buffer = 0;
                while count < buffer_item_count && item_count < words.len() {
                    if count % divider == 0 {
                        buffer = words[item_count].into();
                    } else {
                        buffer |= (words[item_count].into()) << ((count % divider) * bits);
                    }
                    if count % divider == divider - 1 || item_count == words.len() - 1 {
                        self.instance.w[count / divider].write(|w| unsafe { w.bits(buffer) });
                    }

                    count += 1;
                    item_count += 1;
                }

                self.instance
                    .mosi_dlen
                    .write(|w| unsafe { w.usr_mosi_dbitlen().bits((count * bits - 1) as u32) });
                self.instance
                    .miso_dlen
                    .write(|w| unsafe { w.usr_miso_dbitlen().bits((count * bits - 1) as u32) });

                self.instance.cmd.modify(|_, w| w.usr().set_bit());
            }
        }

        drop(apb_lock);

        Ok(words)
    }

    /// Generic write function for iterators
    ///
    /// This function locks the APB bus frequency and chunks the output of the iterator
    /// for maximum write performance.
    fn write_iter_internal<T, WI>(&mut self, words: WI) -> Result<(), Error>
    where
        T: U8orU16orU32,
        WI: IntoIterator<Item = T>,
    {
        let bytes = core::mem::size_of::<T>();
        let bits = bytes * 8;
        let divider = 4 / bytes;
        let buffer_item_count = 64 / bytes;

        let apb_lock = self.clock_control.lock_apb_frequency();

        let mut iter = words.into_iter().peekable();

        let mut buffer: [u32; 16] = [0; 16];

        while iter.peek().is_some() {
            let chunk = iter.by_ref().take(buffer_item_count);

            let mut count = 0;
            for value in chunk {
                if count % divider == 0 {
                    buffer[count / divider] = value.into();
                } else {
                    buffer[count / divider] |= (value.into()) << ((count % divider) * bits);
                }
                count += 1;
            }

            while self.instance.cmd.read().usr().bit_is_set() {}

            for i in 0..((count + divider - 1) / divider) {
                self.instance.w[i].write(|w| unsafe { w.bits(buffer[i]) });
            }

            self.instance
                .mosi_dlen
                .write(|w| unsafe { w.usr_mosi_dbitlen().bits((count * bits - 1) as u32) });
            self.instance
                .miso_dlen
                .write(|w| unsafe { w.usr_miso_dbitlen().bits((count * bits - 1) as u32) });

            self.instance.cmd.modify(|_, w| w.usr().set_bit());
        }

        while self.instance.cmd.read().usr().bit_is_set() {}

        drop(apb_lock);

        Ok(())
    }
}

pub trait U8orU16orU32: core::convert::TryFrom<u32> + Into<u32> + Sized + Copy + Clone {}

impl U8orU16orU32 for u8 {}
impl U8orU16orU32 for u16 {}
impl U8orU16orU32 for u32 {}

/// Full-duplex implementation for writing/reading via SPI
///
/// *Note: these functions do not lock the frequency of the APB bus, so transactions may be
/// at lower frequency if APB bus is not locked in caller.*
impl<
        T: U8orU16orU32,
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > FullDuplex<T> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn read(&mut self) -> nb::Result<T, Error> {
        let spi = &self.instance;

        if spi.cmd.read().usr().bit_is_set() {
            return Err(nb::Error::WouldBlock);
        }

        let bits = (core::mem::size_of::<T>() * 8) as u32;

        (spi.w[0].read().bits() & (0xffffffff >> (32 - bits)))
            .try_into()
            .map_err(|_| nb::Error::Other(Error::ConversionFailed))
    }

    fn send(&mut self, value: T) -> nb::Result<(), Error> {
        let spi = &self.instance;

        if spi.cmd.read().usr().bit_is_set() {
            return Err(nb::Error::WouldBlock);
        }

        let bits = (core::mem::size_of::<T>() * 8 - 1) as u32;

        spi.mosi_dlen
            .write(|w| unsafe { w.usr_mosi_dbitlen().bits(bits) });
        spi.miso_dlen
            .write(|w| unsafe { w.usr_miso_dbitlen().bits(bits) });
        spi.w[0].write(|w| unsafe { w.bits(value.into()) });

        spi.cmd.modify(|_, w| w.usr().set_bit());

        Ok(())
    }
}

// cannot use generics as it conflicts with the Default implementation
impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > Transfer<u8> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> core::result::Result<&'w [u8], Self::Error> {
        self.transfer_internal(words)
    }
}

impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > Transfer<u16> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn transfer<'w>(
        &mut self,
        words: &'w mut [u16],
    ) -> core::result::Result<&'w [u16], Self::Error> {
        self.transfer_internal(words)
    }
}

impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > Transfer<u32> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn transfer<'w>(
        &mut self,
        words: &'w mut [u32],
    ) -> core::result::Result<&'w [u32], Self::Error> {
        self.transfer_internal(words)
    }
}

// cannot use generics as it conflicts with the Default implementation
impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > Write<u8> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write_iter_internal(words.iter().copied())
    }
}

impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > Write<u16> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn write(&mut self, words: &[u16]) -> Result<(), Self::Error> {
        self.write_iter_internal(words.iter().copied())
    }
}

// this could be further optimized with a dedicated function to skip the buffer
impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > Write<u32> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn write(&mut self, words: &[u32]) -> Result<(), Self::Error> {
        self.write_iter_internal(words.iter().copied())
    }
}

// cannot use generics as it conflicts with the Default implementation
impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > WriteIter<u8> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item = u8>,
    {
        self.write_iter_internal(words)
    }
}

impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > WriteIter<u16> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item = u16>,
    {
        self.write_iter_internal(words)
    }
}

impl<
        INSTANCE: Instance,
        SCLK: OutputPin,
        SDO: OutputPin,
        SDI: InputPin + OutputPin,
        CS: OutputPin,
    > WriteIter<u32> for SPI<INSTANCE, SCLK, SDO, SDI, CS>
{
    type Error = Error;

    fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item = u32>,
    {
        self.write_iter_internal(words)
    }
}

mod private {

    use super::Pins;
    use crate::gpio::{InputPin, InputSignal, OutputPin, OutputSignal};
    use crate::prelude::*;
    use crate::target::{spi, SPI1, SPI2, SPI3};
    use core::ops::Deref;

    pub trait Instance: Deref<Target = spi::RegisterBlock> {
        fn ptr() -> *const spi::RegisterBlock;
        /// Enable peripheral
        fn enable(&mut self) -> &mut Self;
        /// Disable peripheral
        fn disable(&mut self) -> &mut Self;
        /// Reset peripheral
        fn reset(&mut self) -> &mut Self;

        /// Initialize pins
        fn init_pins<SCLK: OutputPin, SDO: OutputPin, SDI: InputPin + OutputPin, CS: OutputPin>(
            &mut self,
            pins: &mut Pins<SCLK, SDO, SDI, CS>,
        ) -> &mut Self;
    }

    // SPI0 is reserved for accessing flash/sram

    impl Instance for SPI1 {
        fn ptr() -> *const spi::RegisterBlock {
            SPI1::ptr()
        }

        fn reset(&mut self) -> &mut Self {
            // SPI0 and 1 share reset, should not reset SPI0 as it is used for flash
            // therefore only clear data registers

            for i in 0..=15 {
                unsafe { self.w[i].write(|w| w.bits(0)) };
            }

            self
        }

        fn enable(&mut self) -> &mut Self {
            dport::enable_peripheral(Peripheral::SPI0_SPI1);
            self
        }

        fn disable(&mut self) -> &mut Self {
            // SPI0 and 1 share reset, should not disable SPI0 as it is used for flash
            self
        }

        fn init_pins<SCLK: OutputPin, SDO: OutputPin, SDI: InputPin + OutputPin, CS: OutputPin>(
            &mut self,
            pins: &mut Pins<SCLK, SDO, SDI, CS>,
        ) -> &mut Self {
            // SCLK, SDO & SDI, pins are initialized and in use by SPI0, cannot change

            // use CS2 signal, as CS is shared between SPI0 and SPI1 and CS0 is for flash,
            // CS1 is for psram?

            if let Some(cs) = &mut pins.cs {
                cs.set_to_push_pull_output()
                    .connect_peripheral_to_output(OutputSignal::SPICS2);
            }

            self.pin
                .write(|w| unsafe { w.bits(0).cs0_dis().set_bit().cs1_dis().set_bit() });

            self
        }
    }

    macro_rules! modules {
        ($(
            $MODULE:ident: ($sclk:ident, $sdo:ident, $sdi:ident, $cs:ident),
        )+) => {
            $(
                impl Instance for $MODULE {
                    fn ptr() -> *const spi::RegisterBlock {
                        $MODULE::ptr()
                    }

                    fn reset(&mut self) -> &mut Self {
                        dport::reset_peripheral(dport::Peripheral::$MODULE);
                        self
                    }

                    fn enable(&mut self) -> &mut Self {
                        dport::enable_peripheral(dport::Peripheral::$MODULE);
                        self
                    }

                    fn disable(&mut self) -> &mut Self {
                        dport::disable_peripheral(dport::Peripheral::$MODULE);
                        self

                    }

                    fn init_pins<SCLK: OutputPin, SDO: OutputPin, SDI: InputPin + OutputPin, CS: OutputPin>(
                        &mut self, pins: &mut Pins<SCLK,SDO,SDI,CS>
                    ) -> &mut Self {
                        pins
                            .sclk
                            .set_to_push_pull_output()
                            .connect_peripheral_to_output(OutputSignal::$sclk);

                        pins
                            .sdo
                            .set_to_push_pull_output()
                            .connect_peripheral_to_output(OutputSignal::$sdo);

                        if let Some(sdi)=&mut pins.sdi {
                            sdi
                                .set_to_input()
                                .connect_input_to_peripheral(InputSignal::$sdi);
                            sdi.internal_pull_up(true);
                        }

                        if let Some(cs) = & mut pins.cs {
                            cs
                                .set_to_push_pull_output()
                                .connect_peripheral_to_output(OutputSignal::$cs);
                        }

                        // Use CS0
                        self
                            .pin
                            .write(|w| unsafe {w.bits(0).cs1_dis().set_bit().cs2_dis().set_bit()});


                        self
                    }
                }
            )+
        }
    }

    modules! {
        SPI2: (HSPICLK, HSPID, HSPIQ, HSPICS0),
        SPI3: (VSPICLK, VSPID, VSPIQ, VSPICS0),
    }
}
