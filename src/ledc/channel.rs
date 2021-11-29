use super::{
    timer::{TimerIFace, TimerSpeed},
    HighSpeed, LowSpeed,
};
use crate::gpio::{OutputPin, OutputSignal};
use esp32::ledc::RegisterBlock;

/// Channel errors
#[derive(Debug)]
pub enum Error {
    /// Invalid duty % value
    Duty,
    /// Timer not configured
    Timer,
}

/// Channel number
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Number {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
    Channel4,
    Channel5,
    Channel6,
    Channel7,
}

/// Channel configuration
pub mod config {
    use crate::gpio::OutputPin;
    use crate::ledc::timer::{TimerIFace, TimerSpeed};

    /// Channel configuration
    #[derive(Copy, Clone)]
    pub struct Config<'a, S: TimerSpeed, O: OutputPin> {
        pub timer: &'a dyn TimerIFace<S>,
        pub duty: f32,
        pub output_pin: O,
    }
}

/// Channel interface
pub trait ChannelIFace<'a, S: TimerSpeed + 'a, O: OutputPin>
where
    Channel<'a, S>: ChannelHW<O>,
{
    /// Configure channel
    fn configure(&mut self, config: config::Config<'a, S, O>) -> Result<(), Error>;
}

/// Channel HW interface
pub trait ChannelHW<O: OutputPin> {
    /// Configure Channel HW
    fn configure_hw(&self, duty_value: u32, output_pin: O) -> Result<(), Error>;
}

/// Channel struct
pub struct Channel<'a, S: TimerSpeed> {
    ledc: &'a RegisterBlock,
    timer: Option<&'a dyn TimerIFace<S>>,
    number: Number,
}

impl<'a, S: TimerSpeed> Channel<'a, S> {
    /// Return a new channel
    pub fn new(number: Number) -> Self {
        let ledc = unsafe { &*esp32::LEDC::ptr() };
        Channel {
            ledc,
            timer: None,
            number,
        }
    }
}

impl<'a, S: TimerSpeed, O: OutputPin> ChannelIFace<'a, S, O> for Channel<'a, S>
where
    Channel<'a, S>: ChannelHW<O>,
{
    /// Configure channel
    fn configure(&mut self, config: config::Config<'a, S, O>) -> Result<(), Error> {
        let duty_range = 2_u32.pow(config.timer.get_duty().unwrap() as u32);
        let duty_value = (duty_range as f32 * config.duty) as u32;

        if duty_value == 0 || config.duty > 1.0 {
            // Not enough bits to represent the requested duty %
            return Err(Error::Duty);
        }

        self.timer = Some(config.timer);

        self.configure_hw(duty_value, config.output_pin)?;

        Ok(())
    }
}

/// Channel HW interface for HighSpeed channels
impl<'a, O: OutputPin> ChannelHW<O> for Channel<'a, HighSpeed> {
    /// Configure Channel HW
    fn configure_hw(&self, duty_value: u32, mut output_pin: O) -> Result<(), Error> {
        let timer = self.timer.unwrap();
        if !timer.is_configured() {
            return Err(Error::Timer);
        }

        output_pin.set_to_push_pull_output();

        let channel_number = timer.get_number() as u8;
        match self.number {
            Number::Channel0 => {
                self.ledc
                    .hsch0_hpoint
                    .write(|w| unsafe { w.hpoint_hsch0().bits(0x0) });
                self.ledc
                    .hsch0_duty
                    .write(|w| unsafe { w.duty_hsch0().bits(duty_value << 4) });
                self.ledc.hsch0_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_hsch0()
                        .set_bit()
                        .timer_sel_hsch0()
                        .bits(channel_number)
                });
                self.ledc.hsch0_conf1.write(|w| unsafe {
                    w.duty_start_hsch0()
                        .set_bit()
                        .duty_inc_hsch0()
                        .set_bit()
                        .duty_num_hsch0()
                        .bits(0x1)
                        .duty_cycle_hsch0()
                        .bits(0x1)
                        .duty_scale_hsch0()
                        .bits(0x0)
                });
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_HS_SIG_0);
            }
            Number::Channel1 => {
                self.ledc
                    .hsch1_hpoint
                    .write(|w| unsafe { w.hpoint_hsch1().bits(0x0) });
                self.ledc
                    .hsch1_duty
                    .write(|w| unsafe { w.duty_hsch1().bits(duty_value << 4) });
                self.ledc.hsch1_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_hsch1()
                        .set_bit()
                        .timer_sel_hsch1()
                        .bits(channel_number)
                });
                self.ledc.hsch1_conf1.write(|w| unsafe {
                    w.duty_start_hsch1()
                        .set_bit()
                        .duty_inc_hsch1()
                        .set_bit()
                        .duty_num_hsch1()
                        .bits(0x1)
                        .duty_cycle_hsch1()
                        .bits(0x1)
                        .duty_scale_hsch1()
                        .bits(0x0)
                });
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_HS_SIG_1);
            }
            Number::Channel2 => {
                self.ledc
                    .hsch2_hpoint
                    .write(|w| unsafe { w.hpoint_hsch2().bits(0x0) });
                self.ledc
                    .hsch2_duty
                    .write(|w| unsafe { w.duty_hsch2().bits(duty_value << 4) });
                self.ledc.hsch2_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_hsch2()
                        .set_bit()
                        .timer_sel_hsch2()
                        .bits(channel_number)
                });
                self.ledc.hsch2_conf1.write(|w| unsafe {
                    w.duty_start_hsch2()
                        .set_bit()
                        .duty_inc_hsch2()
                        .set_bit()
                        .duty_num_hsch2()
                        .bits(0x1)
                        .duty_cycle_hsch2()
                        .bits(0x1)
                        .duty_scale_hsch2()
                        .bits(0x0)
                });
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_HS_SIG_2);
            }
            Number::Channel3 => {
                self.ledc
                    .hsch3_hpoint
                    .write(|w| unsafe { w.hpoint_hsch3().bits(0x0) });
                self.ledc
                    .hsch3_duty
                    .write(|w| unsafe { w.duty_hsch3().bits(duty_value << 4) });
                self.ledc.hsch3_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_hsch3()
                        .set_bit()
                        .timer_sel_hsch3()
                        .bits(channel_number)
                });
                self.ledc.hsch3_conf1.write(|w| unsafe {
                    w.duty_start_hsch3()
                        .set_bit()
                        .duty_inc_hsch3()
                        .set_bit()
                        .duty_num_hsch3()
                        .bits(0x1)
                        .duty_cycle_hsch3()
                        .bits(0x1)
                        .duty_scale_hsch3()
                        .bits(0x0)
                });
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_HS_SIG_3);
            }
            Number::Channel4 => {
                self.ledc
                    .hsch4_hpoint
                    .write(|w| unsafe { w.hpoint_hsch4().bits(0x0) });
                self.ledc
                    .hsch4_duty
                    .write(|w| unsafe { w.duty_hsch4().bits(duty_value << 4) });
                self.ledc.hsch4_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_hsch4()
                        .set_bit()
                        .timer_sel_hsch4()
                        .bits(channel_number)
                });
                self.ledc.hsch4_conf1.write(|w| unsafe {
                    w.duty_start_hsch4()
                        .set_bit()
                        .duty_inc_hsch4()
                        .set_bit()
                        .duty_num_hsch4()
                        .bits(0x1)
                        .duty_cycle_hsch4()
                        .bits(0x1)
                        .duty_scale_hsch4()
                        .bits(0x0)
                });
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_HS_SIG_4);
            }
            Number::Channel5 => {
                self.ledc
                    .hsch5_hpoint
                    .write(|w| unsafe { w.hpoint_hsch5().bits(0x0) });
                self.ledc
                    .hsch5_duty
                    .write(|w| unsafe { w.duty_hsch5().bits(duty_value << 4) });
                self.ledc.hsch5_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_hsch5()
                        .set_bit()
                        .timer_sel_hsch5()
                        .bits(channel_number)
                });
                self.ledc.hsch5_conf1.write(|w| unsafe {
                    w.duty_start_hsch5()
                        .set_bit()
                        .duty_inc_hsch5()
                        .set_bit()
                        .duty_num_hsch5()
                        .bits(0x1)
                        .duty_cycle_hsch5()
                        .bits(0x1)
                        .duty_scale_hsch5()
                        .bits(0x0)
                });
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_HS_SIG_5);
            }
            Number::Channel6 => {
                self.ledc
                    .hsch6_hpoint
                    .write(|w| unsafe { w.hpoint_hsch6().bits(0x0) });
                self.ledc
                    .hsch6_duty
                    .write(|w| unsafe { w.duty_hsch6().bits(duty_value << 4) });
                self.ledc.hsch6_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_hsch6()
                        .set_bit()
                        .timer_sel_hsch6()
                        .bits(channel_number)
                });
                self.ledc.hsch6_conf1.write(|w| unsafe {
                    w.duty_start_hsch6()
                        .set_bit()
                        .duty_inc_hsch6()
                        .set_bit()
                        .duty_num_hsch6()
                        .bits(0x1)
                        .duty_cycle_hsch6()
                        .bits(0x1)
                        .duty_scale_hsch6()
                        .bits(0x0)
                });
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_HS_SIG_6);
            }
            Number::Channel7 => {
                self.ledc
                    .hsch7_hpoint
                    .write(|w| unsafe { w.hpoint_hsch7().bits(0x0) });
                self.ledc
                    .hsch7_duty
                    .write(|w| unsafe { w.duty_hsch7().bits(duty_value << 4) });
                self.ledc.hsch7_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_hsch7()
                        .set_bit()
                        .timer_sel_hsch7()
                        .bits(channel_number)
                });
                self.ledc.hsch7_conf1.write(|w| unsafe {
                    w.duty_start_hsch7()
                        .set_bit()
                        .duty_inc_hsch7()
                        .set_bit()
                        .duty_num_hsch7()
                        .bits(0x1)
                        .duty_cycle_hsch7()
                        .bits(0x1)
                        .duty_scale_hsch7()
                        .bits(0x0)
                });
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_HS_SIG_7);
            }
        }

        Ok(())
    }
}

/// Channel HW interface for LowSpeed channels
impl<'a, O: OutputPin> ChannelHW<O> for Channel<'a, LowSpeed> {
    /// Configure Channel HW
    fn configure_hw(&self, duty_value: u32, mut output_pin: O) -> Result<(), Error> {
        let timer = self.timer.unwrap();
        if !timer.is_configured() {
            return Err(Error::Timer);
        }

        output_pin.set_to_push_pull_output();

        let channel_number = timer.get_number() as u8;
        match self.number {
            Number::Channel0 => {
                self.ledc
                    .lsch0_hpoint
                    .write(|w| unsafe { w.hpoint_lsch0().bits(0x0) });
                self.ledc
                    .lsch0_duty
                    .write(|w| unsafe { w.duty_lsch0().bits(duty_value << 4) });
                self.ledc.lsch0_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_lsch0()
                        .set_bit()
                        .timer_sel_lsch0()
                        .bits(channel_number)
                });
                self.ledc.lsch0_conf1.write(|w| unsafe {
                    w.duty_start_lsch0()
                        .set_bit()
                        .duty_inc_lsch0()
                        .set_bit()
                        .duty_num_lsch0()
                        .bits(0x1)
                        .duty_cycle_lsch0()
                        .bits(0x1)
                        .duty_scale_lsch0()
                        .bits(0x0)
                });
                self.ledc
                    .lsch0_conf0
                    .modify(|_, w| w.para_up_lsch0().set_bit());
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_LS_SIG_0);
            }
            Number::Channel1 => {
                self.ledc
                    .lsch1_hpoint
                    .write(|w| unsafe { w.hpoint_lsch1().bits(0x0) });
                self.ledc
                    .lsch1_duty
                    .write(|w| unsafe { w.duty_lsch1().bits(duty_value << 4) });
                self.ledc.lsch1_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_lsch1()
                        .set_bit()
                        .timer_sel_lsch1()
                        .bits(channel_number)
                });
                self.ledc.lsch1_conf1.write(|w| unsafe {
                    w.duty_start_lsch1()
                        .set_bit()
                        .duty_inc_lsch1()
                        .set_bit()
                        .duty_num_lsch1()
                        .bits(0x1)
                        .duty_cycle_lsch1()
                        .bits(0x1)
                        .duty_scale_lsch1()
                        .bits(0x0)
                });
                self.ledc
                    .lsch1_conf0
                    .modify(|_, w| w.para_up_lsch1().set_bit());
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_LS_SIG_1);
            }
            Number::Channel2 => {
                self.ledc
                    .lsch2_hpoint
                    .write(|w| unsafe { w.hpoint_lsch2().bits(0x0) });
                self.ledc
                    .lsch2_duty
                    .write(|w| unsafe { w.duty_lsch2().bits(duty_value << 4) });
                self.ledc.lsch2_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_lsch2()
                        .set_bit()
                        .timer_sel_lsch2()
                        .bits(channel_number)
                });
                self.ledc.lsch2_conf1.write(|w| unsafe {
                    w.duty_start_lsch2()
                        .set_bit()
                        .duty_inc_lsch2()
                        .set_bit()
                        .duty_num_lsch2()
                        .bits(0x1)
                        .duty_cycle_lsch2()
                        .bits(0x1)
                        .duty_scale_lsch2()
                        .bits(0x0)
                });
                self.ledc
                    .lsch2_conf0
                    .modify(|_, w| w.para_up_lsch2().set_bit());
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_LS_SIG_2);
            }
            Number::Channel3 => {
                self.ledc
                    .lsch3_hpoint
                    .write(|w| unsafe { w.hpoint_lsch3().bits(0x0) });
                self.ledc
                    .lsch3_duty
                    .write(|w| unsafe { w.duty_lsch3().bits(duty_value << 4) });
                self.ledc.lsch3_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_lsch3()
                        .set_bit()
                        .timer_sel_lsch3()
                        .bits(channel_number)
                });
                self.ledc.lsch3_conf1.write(|w| unsafe {
                    w.duty_start_lsch3()
                        .set_bit()
                        .duty_inc_lsch3()
                        .set_bit()
                        .duty_num_lsch3()
                        .bits(0x1)
                        .duty_cycle_lsch3()
                        .bits(0x1)
                        .duty_scale_lsch3()
                        .bits(0x0)
                });
                self.ledc
                    .lsch3_conf0
                    .modify(|_, w| w.para_up_lsch3().set_bit());
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_LS_SIG_3);
            }
            Number::Channel4 => {
                self.ledc
                    .lsch4_hpoint
                    .write(|w| unsafe { w.hpoint_lsch4().bits(0x0) });
                self.ledc
                    .lsch4_duty
                    .write(|w| unsafe { w.duty_lsch4().bits(duty_value << 4) });
                self.ledc.lsch4_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_lsch4()
                        .set_bit()
                        .timer_sel_lsch4()
                        .bits(channel_number)
                });
                self.ledc.lsch4_conf1.write(|w| unsafe {
                    w.duty_start_lsch4()
                        .set_bit()
                        .duty_inc_lsch4()
                        .set_bit()
                        .duty_num_lsch4()
                        .bits(0x1)
                        .duty_cycle_lsch4()
                        .bits(0x1)
                        .duty_scale_lsch4()
                        .bits(0x0)
                });
                self.ledc
                    .lsch4_conf0
                    .modify(|_, w| w.para_up_lsch4().set_bit());
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_LS_SIG_4);
            }
            Number::Channel5 => {
                self.ledc
                    .lsch5_hpoint
                    .write(|w| unsafe { w.hpoint_lsch5().bits(0x0) });
                self.ledc
                    .lsch5_duty
                    .write(|w| unsafe { w.duty_lsch5().bits(duty_value << 4) });
                self.ledc.lsch5_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_lsch5()
                        .set_bit()
                        .timer_sel_lsch5()
                        .bits(channel_number)
                });
                self.ledc.lsch5_conf1.write(|w| unsafe {
                    w.duty_start_lsch5()
                        .set_bit()
                        .duty_inc_lsch5()
                        .set_bit()
                        .duty_num_lsch5()
                        .bits(0x1)
                        .duty_cycle_lsch5()
                        .bits(0x1)
                        .duty_scale_lsch5()
                        .bits(0x0)
                });
                self.ledc
                    .lsch5_conf0
                    .modify(|_, w| w.para_up_lsch5().set_bit());
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_LS_SIG_5);
            }
            Number::Channel6 => {
                self.ledc
                    .lsch6_hpoint
                    .write(|w| unsafe { w.hpoint_lsch6().bits(0x0) });
                self.ledc
                    .lsch6_duty
                    .write(|w| unsafe { w.duty_lsch6().bits(duty_value << 4) });
                self.ledc.lsch6_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_lsch6()
                        .set_bit()
                        .timer_sel_lsch6()
                        .bits(channel_number)
                });
                self.ledc.lsch6_conf1.write(|w| unsafe {
                    w.duty_start_lsch6()
                        .set_bit()
                        .duty_inc_lsch6()
                        .set_bit()
                        .duty_num_lsch6()
                        .bits(0x1)
                        .duty_cycle_lsch6()
                        .bits(0x1)
                        .duty_scale_lsch6()
                        .bits(0x0)
                });
                self.ledc
                    .lsch6_conf0
                    .modify(|_, w| w.para_up_lsch6().set_bit());
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_LS_SIG_6);
            }
            Number::Channel7 => {
                self.ledc
                    .lsch7_hpoint
                    .write(|w| unsafe { w.hpoint_lsch7().bits(0x0) });
                self.ledc
                    .lsch7_duty
                    .write(|w| unsafe { w.duty_lsch7().bits(duty_value << 4) });
                self.ledc.lsch7_conf0.modify(|_, w| unsafe {
                    w.sig_out_en_lsch7()
                        .set_bit()
                        .timer_sel_lsch7()
                        .bits(channel_number)
                });
                self.ledc.lsch7_conf1.write(|w| unsafe {
                    w.duty_start_lsch7()
                        .set_bit()
                        .duty_inc_lsch7()
                        .set_bit()
                        .duty_num_lsch7()
                        .bits(0x1)
                        .duty_cycle_lsch7()
                        .bits(0x1)
                        .duty_scale_lsch7()
                        .bits(0x0)
                });
                self.ledc
                    .lsch7_conf0
                    .modify(|_, w| w.para_up_lsch7().set_bit());
                output_pin.connect_peripheral_to_output(OutputSignal::LEDC_LS_SIG_7);
            }
        }

        Ok(())
    }
}
