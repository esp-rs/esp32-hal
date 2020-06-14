//! Built-in hall sensor readout.
//!
//! This module provides a function for reading current value of the built-in
//! hall sensor.
//!

use embedded_hal::adc::OneShot;

use crate::analog::adc::ADC;
use crate::analog::ADC1;
use crate::gpio::{Analog, Gpio36, Gpio39};
use crate::target::RTCIO;

impl ADC<ADC1> {
    pub fn read_hall_sensor(
        &mut self,
        vp_pin: &mut Gpio36<Analog>,
        vn_pin: &mut Gpio39<Analog>,
    ) -> i32 {
        let rtcio = unsafe { &*RTCIO::ptr() };

        rtcio.hall_sens.modify(|_, w| w.hall_phase().clear_bit());
        let vp1: u16 = nb::block!(self.read(vp_pin)).unwrap();
        let vn1: u16 = nb::block!(self.read(vn_pin)).unwrap();

        rtcio.hall_sens.modify(|_, w| w.hall_phase().set_bit());
        let vp2: u16 = nb::block!(self.read(vp_pin)).unwrap();
        let vn2: u16 = nb::block!(self.read(vn_pin)).unwrap();

        (vp2 as i32 - vp1 as i32) - (vn2 as i32 - vn1 as i32)
    }
}
