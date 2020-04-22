//! Analog peripherals control.
//!
//! Provides functionality for using analog peripherals, such as ADCs, DACs
//! and other sesnsors.
//!

pub mod adc;
pub mod config;
pub mod dac;
pub mod hall;

use core::marker::PhantomData;
use esp32::SENS;

pub struct ADC1 {
    _private: PhantomData<()>,
}
pub struct ADC2 {
    _private: PhantomData<()>,
}

pub struct DAC1 {
    _private: PhantomData<()>,
}

pub struct DAC2 {
    _private: PhantomData<()>,
}

pub struct AvailableAnalog {
    pub adc1: ADC1,
    pub adc2: ADC2,
    pub dac1: DAC1,
    pub dac2: DAC2,
}

pub trait SensExt {
    fn split(self) -> AvailableAnalog;
}

impl SensExt for SENS {
    fn split(self) -> AvailableAnalog {
        AvailableAnalog {
            adc1: ADC1 {
                _private: PhantomData,
            },
            adc2: ADC2 {
                _private: PhantomData,
            },
            dac1: DAC1 {
                _private: PhantomData,
            },
            dac2: DAC2 {
                _private: PhantomData,
            },
        }
    }
}
