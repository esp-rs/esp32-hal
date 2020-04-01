//! Digital to analog (DAC) conversion.
//! 
//! This module provides functions for controling two digital to
//! analog converters, available on ESP32: `DAC1` and `DAC2`.
//! 
//! The DAC1 is avilable on the GPIO pin 25, and DAC2 on pin 26.
//! 

use core::marker::PhantomData;
use esp32::{RTCIO, SENS};

use crate::analog::{DAC1, DAC2};
use crate::gpio::{Analog, Gpio25, Gpio26};

pub struct DAC<DAC> {
    _dac: PhantomData<DAC>,
}

impl DAC<DAC1> {
    pub fn dac1(_dac: DAC1, _pin: Gpio25<Analog>) -> Result<Self, ()> {
        let dac = DAC::<DAC1> { _dac: PhantomData }.set_power();

        Ok(dac)
    }

    fn set_power(self) -> Self {
        let rtcio = unsafe { &*RTCIO::ptr() };

        rtcio.rtc_io_pad_dac1.modify(|_, w| {
            w.rtc_io_pdac1_dac_xpd_force().set_bit();
            w.rtc_io_pdac1_xpd_dac().set_bit()
        });

        self
    }

    pub fn write(&mut self, value: u8) {
        let rtcio = unsafe { &*RTCIO::ptr() };
        let sensors = unsafe { &*SENS::ptr() };

        sensors
            .sar_dac_ctrl2
            .modify(|_, w| w.dac_cw_en1().clear_bit());
        rtcio
            .rtc_io_pad_dac1
            .modify(|_, w| unsafe { w.rtc_io_pdac1_dac().bits(value) });
    }
}

impl DAC<DAC2> {
    pub fn dac2(_dac: DAC2, _pin: Gpio26<Analog>) -> Result<Self, ()> {
        let dac = DAC::<DAC2> { _dac: PhantomData }.set_power();

        Ok(dac)
    }

    fn set_power(self) -> Self {
        let rtcio = unsafe { &*RTCIO::ptr() };

        rtcio.rtc_io_pad_dac2.modify(|_, w| {
            w.rtc_io_pdac2_dac_xpd_force().set_bit();
            w.rtc_io_pdac2_xpd_dac().set_bit()
        });

        self
    }

    pub fn write(&mut self, value: u8) {
        let rtcio = unsafe { &*RTCIO::ptr() };
        let sensors = unsafe { &*SENS::ptr() };

        sensors
            .sar_dac_ctrl2
            .modify(|_, w| w.dac_cw_en2().clear_bit());
        rtcio
            .rtc_io_pad_dac2
            .modify(|_, w| unsafe { w.rtc_io_pdac2_dac().bits(value) });
    }
}
