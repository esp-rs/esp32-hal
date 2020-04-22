//! Configuration of analog modules.

use crate::analog::{ADC1, ADC2};
use embedded_hal::adc::Channel;

/// The sampling/readout resolution of the ADC
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Resolution {
    Resolution9Bit = 0b00,
    Resolution10Bit = 0b01,
    Resolution11Bit = 0b10,
    Resolution12Bit = 0b11,
}

/// The attenuation of the ADC pin
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Attenuation {
    Attenuation0dB = 0b00,
    Attenuation2p5dB = 0b01,
    Attenuation6dB = 0b10,
    Attenuation11dB = 0b11,
}

pub struct Adc1Config {
    pub resolution: Resolution,
    pub hall_sensor: bool,
    pub attenuations: [Option<Attenuation>; 10],
}

impl Adc1Config {
    pub fn new() -> Adc1Config {
        Adc1Config {
            resolution: Resolution::Resolution12Bit,
            hall_sensor: false,
            attenuations: [None; 10],
        }
    }

    pub fn enable_pin<PIN: Channel<ADC1, ID = u8>>(
        &mut self,
        _pin: &PIN,
        attenuation: Attenuation,
    ) {
        self.attenuations[PIN::channel() as usize] = Some(attenuation);
    }

    pub fn enable_hall_sensor(&mut self) {
        self.hall_sensor = true;
    }
}

pub struct Adc2Config {
    pub resolution: Resolution,
    pub attenuations: [Option<Attenuation>; 10],
}

impl Adc2Config {
    pub fn new() -> Adc2Config {
        Adc2Config {
            resolution: Resolution::Resolution12Bit,
            attenuations: [None; 10],
        }
    }

    pub fn enable_pin<PIN: Channel<ADC2, ID = u8>>(
        &mut self,
        _pin: &PIN,
        attenuation: Attenuation,
    ) {
        self.attenuations[PIN::channel() as usize] = Some(attenuation);
    }
}
