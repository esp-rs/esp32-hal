//! Analog to digital (ADC) conversion support.
//!
//! This module provides functions for reading analog values from two
//! analog to digital converters available on the ESP32: `ADC1` and `ADC2`.
//!
//! The following pins can be configured for analog readout:
//!
//! | Channel | ADC1                 | ADC2          |
//! |---------|----------------------|---------------|
//! | 0       | GPIO36 (SENSOR_VP)   | GPIO4         |
//! | 1       | GPIO37 (SENSOR_CAPP) | GPIO0         |
//! | 2       | GPIO38 (SENSOR_CAPN) | GPIO2         |
//! | 3       | GPIO39 (SENSOR_VN)   | GPIO15 (MTDO) |
//! | 4       | GPIO33 (32K_XP)      | GPIO13 (MTCK) |
//! | 5       | GPIO32 (32K_XN)      | GPIO12 (MTDI) |
//! | 6       | GPIO34 (VDET_1)      | GPIO14 (MTMS) |
//! | 7       | GPIO35 (VDET_2)      | GPIO27        |
//! | 8       |                      | GPIO25        |
//! | 9       |                      | GPIO26        |
//!

use core::marker::PhantomData;
use embedded_hal::adc::{Channel, OneShot};

use crate::analog::config;
use crate::analog::{ADC1, ADC2};
use crate::gpio::*;
use crate::target::{RTCIO, SENS};

pub struct ADC<ADC> {
    adc: PhantomData<ADC>,
    attenuations: [Option<config::Attenuation>; 10],
    active_channel: Option<u8>,
}

macro_rules! impl_adc_setup {
    ($config:expr, $bit_width:ident, $read_reg:ident, $sample_bit:ident, $atten_reg: ident,
        $atten_field:ident, $dig_force:ident, $meas_start_reg:ident,
        $start_force_field:ident, $en_pad_force:ident) => {
        let sensors = unsafe { &*SENS::ptr() };

        /* Set reading and sampling resolution */
        let resolution: u8 = $config.resolution as u8;

        sensors
            .sar_start_force
            .modify(|_, w| unsafe { w.$bit_width().bits(resolution) });
        sensors
            .$read_reg
            .modify(|_, w| unsafe { w.$sample_bit().bits(resolution) });

        /* Set attenuation for pins */
        let attenuations = $config.attenuations;

        for channel in 0..attenuations.len() {
            if let Some(attenuation) = attenuations[channel] {
                sensors.$atten_reg.modify(|r, w| {
                    let new_value = (r.bits() & !(0b11 << (channel * 2)))
                        | (((attenuation as u8 & 0b11) as u32) << (channel * 2));

                    unsafe { w.$atten_field().bits(new_value) }
                });
            }
        }

        /* Set controller to RTC */
        sensors.$read_reg.modify(|_, w| w.$dig_force().clear_bit());
        sensors
            .$meas_start_reg
            .modify(|_, w| w.$start_force_field().set_bit());
        sensors
            .$meas_start_reg
            .modify(|_, w| w.$en_pad_force().set_bit());
        sensors
            .sar_touch_ctrl1
            .modify(|_, w| w.xpd_hall_force().set_bit());
        sensors
            .sar_touch_ctrl1
            .modify(|_, w| w.hall_phase_force().set_bit());

        /* Set power to SW power on */
        sensors
            .sar_meas_wait2
            .modify(|_, w| unsafe { w.force_xpd_sar().bits(0b11) });

        /* disable AMP */
        sensors
            .sar_meas_wait2
            .modify(|_, w| unsafe { w.force_xpd_amp().bits(0b10) });
        sensors
            .sar_meas_ctrl
            .modify(|_, w| unsafe { w.amp_rst_fb_fsm().bits(0) });
        sensors
            .sar_meas_ctrl
            .modify(|_, w| unsafe { w.amp_short_ref_fsm().bits(0) });
        sensors
            .sar_meas_ctrl
            .modify(|_, w| unsafe { w.amp_short_ref_gnd_fsm().bits(0) });
        sensors
            .sar_meas_wait1
            .modify(|_, w| unsafe { w.sar_amp_wait1().bits(1) });
        sensors
            .sar_meas_wait1
            .modify(|_, w| unsafe { w.sar_amp_wait2().bits(1) });
        sensors
            .sar_meas_wait2
            .modify(|_, w| unsafe { w.sar_amp_wait3().bits(1) });
    };
}

impl ADC<ADC1> {
    pub fn adc1(_adc_instance: ADC1, config: config::Adc1Config) -> Result<Self, ()> {
        impl_adc_setup!(
            config,
            sar1_bit_width,
            sar_read_ctrl,
            sar1_sample_bit,
            sar_atten1,
            sar1_atten,
            sar1_dig_force,
            sar_meas_start1,
            meas1_start_force,
            sar1_en_pad_force
        );

        /* Connect or disconnect hall sensor */
        let rtcio = unsafe { &*RTCIO::ptr() };

        if config.hall_sensor {
            rtcio
                .rtc_io_hall_sens
                .modify(|_, w| w.rtc_io_xpd_hall().set_bit());
        } else {
            rtcio
                .rtc_io_hall_sens
                .modify(|_, w| w.rtc_io_xpd_hall().clear_bit());
        }

        let adc = ADC {
            adc: PhantomData,
            attenuations: config.attenuations,
            active_channel: None,
        };

        Ok(adc)
    }
}

impl ADC<ADC2> {
    pub fn adc2(_adc_instance: ADC2, config: config::Adc2Config) -> Result<Self, ()> {
        impl_adc_setup!(
            config,
            sar2_bit_width,
            sar_read_ctrl2,
            sar2_sample_bit,
            sar_atten2,
            sar2_atten,
            sar2_dig_force,
            sar_meas_start2,
            meas2_start_force,
            sar2_en_pad_force
        );

        let adc = ADC {
            adc: PhantomData,
            attenuations: config.attenuations,
            active_channel: None,
        };

        Ok(adc)
    }
}

macro_rules! impl_adc_interface {
    ($adc:ident ($start_reg:ident, $en_pad:ident, $start:ident, $done:ident, $data:ident): [
        $( ($pin:ident, $channel:expr) ,)+
    ]) => {

        impl<WORD, PIN> OneShot<$adc, WORD, PIN> for ADC<$adc>
        where
        WORD: From<u16>,
        PIN: Channel<$adc, ID=u8>,
        {
            type Error = ();

            fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
                let sensors = unsafe { &*SENS::ptr() };

                if self.attenuations[PIN::channel() as usize] == None {
                    panic!("Channel {} is not configured reading!", PIN::channel());
                }

                if let Some(active_channel) = self.active_channel {
                    // There is conversion in progress:
                    // - if it's for a different channel try again later
                    // - if it's for the given channel, go ahaid and check progress
                    if active_channel != PIN::channel() {
                        return Err(nb::Error::WouldBlock);
                    }
                }
                else {
                    // If no conversions are in progress, start a new one for given channel
                    self.active_channel = Some(PIN::channel());

                    sensors.$start_reg.modify(|_, w| {
                        unsafe { w.$en_pad().bits(1 << PIN::channel() as u8) }
                    });

                    sensors.$start_reg.modify(|_,w| w.$start().clear_bit());
                    sensors.$start_reg.modify(|_,w| w.$start().set_bit());
                }

                // Wait for ADC to finish conversion
                let conversion_finished = sensors.$start_reg.read().$done().bit_is_set();
                if !conversion_finished {
                    return Err(nb::Error::WouldBlock);
                }

                // Get converted value
                let converted_value = sensors.$start_reg.read().$data().bits() as u16;

                // Mark that no conversions are currently in progress
                self.active_channel = None;

                Ok(converted_value.into())
            }
        }


        $(
            impl Channel<$adc> for $pin<Analog> {
                type ID = u8;

                fn channel() -> u8 { $channel }
            }
        )+
    }
}

impl_adc_interface! {
    ADC1 (sar_meas_start1, sar1_en_pad, meas1_start_sar, meas1_done_sar, meas1_data_sar): [
        (Gpio36, 0), // Alt. name: SENSOR_VP
        (Gpio37, 1), // Alt. name: SENSOR_CAPP
        (Gpio38, 2), // Alt. name: SENSOR_CAPN
        (Gpio39, 3), // Alt. name: SENSOR_VN
        (Gpio33, 4), // Alt. name: 32K_XP
        (Gpio32, 5), // Alt. name: 32K_XN
        (Gpio34, 6), // Alt. name: VDET_1
        (Gpio35, 7), // Alt. name: VDET_2
    ]
}

impl_adc_interface! {
    ADC2 (sar_meas_start2, sar2_en_pad, meas2_start_sar, meas2_done_sar, meas2_data_sar): [
        (Gpio4, 0),
        (Gpio0, 1),
        (Gpio2, 2),
        (Gpio15, 3), // Alt. name: MTDO
        (Gpio13, 4), // Alt. name: MTCK
        (Gpio12, 5), // Alt. name: MTDI
        (Gpio14, 6), // Alt. name: MTMS
        (Gpio27, 7),
        (Gpio25, 8),
        (Gpio26, 9),
    ]
}
