use {
    crate::target::{GPIO, IO_MUX, RTCIO},
    core::{convert::Infallible, marker::PhantomData},
    embedded_hal::digital::v2::{OutputPin as _, StatefulOutputPin as _},
};

mod mux;
pub use mux::*;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

pub trait Pin {
    fn set_drive_strength(&mut self, strength: DriveStrength) -> &mut Self;
    fn enable_open_drain(&mut self, on: bool) -> &mut Self;
    fn enable_input(&mut self, on: bool) -> &mut Self;
    fn internal_pull_up(&mut self, on: bool) -> &mut Self;
    fn internal_pull_down(&mut self, on: bool) -> &mut Self;
    fn set_drive_strength_in_sleep_mode(&mut self, strength: DriveStrength) -> &mut Self;
    fn input_in_sleep_mode(&mut self, on: bool) -> &mut Self;
    fn internal_pull_up_in_sleep_mode(&mut self, on: bool) -> &mut Self;
    fn internal_pull_down_in_sleep_mode(&mut self, on: bool) -> &mut Self;
    fn output_in_sleep_mode(&mut self, on: bool) -> &mut Self;
    fn sleep_mode(&mut self, on: bool) -> &mut Self;
    fn connect_input_to_peripheral(&mut self, signal: InputSignal, invert: bool) -> &mut Self;
    fn connect_peripheral_to_output(
        &mut self,
        signal: OutputSignal,
        invert: bool,
        enable_from_gpio: bool,
        invert_enable: bool,
    ) -> &mut Self;
}

pub trait InputPin: Pin + embedded_hal::digital::v2::InputPin {}
pub trait OutputPin:
    Pin
    + embedded_hal::digital::v2::OutputPin
    + embedded_hal::digital::v2::StatefulOutputPin
    + embedded_hal::digital::v2::ToggleableOutputPin
{
}

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;

/// Pulled down input (type state)
pub struct PullDown;

/// Pulled up input (type state)
pub struct PullUp;

/// Open drain input or output (type state)
pub struct OpenDrain;

/// Push pull output (type state)
pub struct PushPull;

/// Analog mode (type state)
pub struct Analog;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Alternate function
pub struct Alternate<MODE> {
    _mode: PhantomData<MODE>,
}

/// Alternate Function 1
pub struct AF1;

/// Alternate Function 2
pub struct AF2;

/// Alternate Function 4
pub struct AF4;

/// Alternate Function 5
pub struct AF5;

/// Alternate Function 6
pub struct AF6;

pub enum DriveStrength {
    I5mA = 0,
    I10mA = 1,
    I20mA = 2,
    I40mA = 3,
}

macro_rules! gpio {
    ( $($pxi:ident: ($pname:ident, $MODE:ty),)+ ) => {

        impl GpioExt for GPIO {
            type Parts = Parts;

            fn split(self) -> Self::Parts {
                Parts {
                    $(
                        $pname: $pxi { _mode: PhantomData },
                    )+
                }
            }
        }

        pub struct Parts {
            $(
                /// Pin
                pub $pname: $pxi<$MODE>,
            )+
        }

        // create all the pins, we can also add functionality
        // applicable to all pin states here
        $(
            /// Pin
            pub struct $pxi<MODE> {
                _mode: PhantomData<MODE>,
            }
        )+
    };
}

// All info on reset state pulled from 4.10 IO_MUX Pad List in the reference manual
gpio! {
       Gpio0: (gpio0, Input<PullUp>),
       Gpio1: (gpio1, Input<PullUp>),
       Gpio2: (gpio2, Input<PullDown>),
       Gpio3: (gpio3, Input<PullUp>),
       Gpio4: (gpio4, Input<PullDown>),
       Gpio5: (gpio5, Input<PullUp>),
       Gpio6: (gpio6, Input<PullUp>),
       Gpio7: (gpio7, Input<PullUp>),
       Gpio8: (gpio8, Input<PullUp>),
       Gpio9: (gpio9, Input<PullUp>),
       Gpio10: (gpio10, Input<PullUp>),
       Gpio11: (gpio11, Input<PullUp>),
       Gpio12: (gpio12, Input<PullDown>),
       Gpio13: (gpio13, Input<Floating>),
       Gpio14: (gpio14, Input<Floating>),
       Gpio15: (gpio15, Input<PullUp>),
       Gpio16: (gpio16, Input<Floating>),
       Gpio17: (gpio17, Input<Floating>),
       Gpio18: (gpio18, Input<Floating>),
       Gpio19: (gpio19, Input<Floating>),
       Gpio20: (gpio20, Input<Floating>),
       Gpio21: (gpio21, Input<Floating>),
       Gpio22: (gpio22, Input<Floating>),
       Gpio23: (gpio23, Input<Floating>),
       // TODO these pins have a reset mode of 0 (apart from Gpio27),
       // input disable, does that mean they are actually in output mode on reset?
       Gpio25: (gpio25, Input<Floating>),
       Gpio26: (gpio26, Input<Floating>),
       Gpio27: (gpio27, Input<Floating>),
       Gpio32: (gpio32, Input<Floating>),
       Gpio33: (gpio33, Input<Floating>),
       Gpio34: (gpio34, Input<Floating>),
       Gpio35: (gpio35, Input<Floating>),
       Gpio36: (gpio36, Input<Floating>),
       Gpio37: (gpio37, Input<Floating>),
       Gpio38: (gpio38, Input<Floating>),
       Gpio39: (gpio39, Input<Floating>),
}

macro_rules! impl_output {
    ($out_en_set:ident, $outs:ident, $outc:ident, [
        // index, gpio pin name, funcX name, iomux pin name
        $($pxi:ident: ($pin_num:expr, $bit:expr, $iomux:ident),)+
    ]) => {
        $(
            impl<MODE> OutputPin for $pxi<Output<MODE>> {
            }
            impl<MODE> embedded_hal::digital::v2::OutputPin for $pxi<Output<MODE>> {
                type Error = Infallible;

                fn set_high(&mut self) -> Result<(), Self::Error> {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*GPIO::ptr()).$outs.write(|w| w.bits(1 << $bit)) };
                    Ok(())
                }

                fn set_low(&mut self) -> Result<(), Self::Error> {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*GPIO::ptr()).$outc.write(|w| w.bits(1 << $bit)) };
                    Ok(())
                }
            }

            impl<MODE> embedded_hal::digital::v2::StatefulOutputPin for $pxi<Output<MODE>> {
                fn is_set_high(&self) -> Result<bool, Self::Error> {
                     // NOTE(unsafe) atomic read to a stateless register
                    unsafe { Ok((*GPIO::ptr()).$outs.read().bits() & (1 << $bit) != 0) }
                }

                fn is_set_low(&self) -> Result<bool, Self::Error> {
                    Ok(!self.is_set_high()?)
                }
            }

            impl<MODE> embedded_hal::digital::v2::ToggleableOutputPin for $pxi<Output<MODE>> {
                type Error = Infallible;

                fn toggle(&mut self) -> Result<(), Self::Error> {
                    if self.is_set_high()? {
                        Ok(self.set_low()?)
                    } else {
                        Ok(self.set_high()?)
                    }
                }
            }

            impl<MODE> $pxi<MODE> {

                fn set_output(&self, alternate: u8, open_drain: bool) {
                    let gpio = unsafe{ &*GPIO::ptr() };
                    let iomux = unsafe{ &*IO_MUX::ptr() };

                    self.disable_analog();

                    // NOTE(unsafe) atomic read to a stateless register
                    gpio.$out_en_set.write(|w| unsafe  { w.bits(1 << $bit) });
                    gpio.pin[$pin_num].modify(|_,w| w.pad_driver().bit(open_drain));
                    gpio.func_out_sel_cfg[$pin_num].modify(|_, w| unsafe {
                        w.out_sel().bits(OutputSignal::GPIO as u16)
                    });

                    iomux.$iomux.modify(|_, w| unsafe {
                        w
                            .mcu_sel().bits(alternate)
                            .fun_wpd().clear_bit()
                            .fun_wpu().clear_bit()
                            .fun_drv().bits(DriveStrength::I20mA as u8)
                            .slp_sel().clear_bit()
                    });
                }

                pub fn into_push_pull_output(self) -> $pxi<Output<PushPull>> {
                    self.set_output(2, false);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_open_drain_output(self) -> $pxi<Output<OpenDrain>> {
                    self.set_output(2, true);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_alternate_1(self) -> $pxi<Alternate<AF1>> {
                    self.set_output(0, false);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_alternate_2(self) -> $pxi<Alternate<AF2>> {
                    self.set_output(1, false);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_alternate_4(self) -> $pxi<Alternate<AF4>> {
                    self.set_output(3, false);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_alternate_5(self) -> $pxi<Alternate<AF5>> {
                    self.set_output(4, false);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_alternate_6(self) -> $pxi<Alternate<AF6>> {
                    self.set_output(5, false);
                    $pxi { _mode: PhantomData }
                }
            }
        )+
    };
}

impl_output! {
    enable_w1ts, out_w1ts, out_w1tc, [
        Gpio0: (0, 0, gpio0),
        Gpio1: (1, 1, u0txd),
        Gpio2: (2, 2, gpio2),
        Gpio3: (3, 3, u0rxd),
        Gpio4: (4, 4, gpio4),
        Gpio5: (5, 5, gpio5),
        Gpio6: (6, 6, sd_clk),
        Gpio7: (7, 7, sd_data0),
        Gpio8: (8, 8, sd_data1),
        Gpio9: (9, 9, sd_data2),
        Gpio10: (10, 10, sd_data3),
        Gpio11: (11, 11, sd_cmd),
        Gpio12: (12, 12, mtdi),
        Gpio13: (13, 13, mtck),
        Gpio14: (14, 14, mtms),
        Gpio15: (15, 15, mtdo),
        Gpio16: (16, 16, gpio16),
        Gpio17: (17, 17, gpio17),
        Gpio18: (18, 18, gpio18),
        Gpio19: (19, 19, gpio19),
        Gpio20: (20, 20, gpio20),
        Gpio21: (21, 21, gpio21),
        Gpio22: (22, 22, gpio22),
        Gpio23: (23, 23, gpio23),
        Gpio25: (25, 25, gpio25),
        Gpio26: (26, 26, gpio26),
        Gpio27: (27, 27, gpio27),
    ]
}

impl_output! {
    enable1_w1ts, out1_w1ts, out1_w1tc, [
        Gpio32: (32, 0, gpio32),
        Gpio33: (33, 1, gpio33),
        /* Deliberately omitting 34-39 as these can *only* be inputs */
    ]
}

macro_rules! impl_pullup_pulldown {
    ($out_en_clear:ident, $pxi:ident, $pin_num:expr, $bit:expr, $iomux:ident,
        has_pullup_pulldown) => {
        pub fn into_pull_up_input(self) -> $pxi<Input<PullUp>> {
            self.set_input(false, false);
            $pxi { _mode: PhantomData }
        }

        pub fn into_pull_down_input(self) -> $pxi<Input<PullDown>> {
            self.set_input(true, false);
            $pxi { _mode: PhantomData }
        }
    };
    ($out_en_clear:ident, $pxi:ident, $pin_num:expr, $bit:expr, $iomux:ident,
        no_pullup_pulldown) => {
        /* No pullup/pulldown resistor is available on this pin. */
    };
    ($out_en_clear:ident, $pxi:ident, $pin_num:expr, $bit:expr, $iomux:ident,
        $pullup_flag:ident) => {
        compile_error!(
            "The GPIO pin has to be marked with either \
            has_pullup_pulldown or no_pullup_pulldown."
        );
    };
}

macro_rules! impl_input {
    ($out_en_clear:ident, $reg:ident, $reader:ident [
        // index, gpio pin name, funcX name, iomux pin name, has pullup/down resistors
        $($pxi:ident: ($pin_num:expr, $bit:expr, $iomux:ident, $resistors:ident),)+
    ]) => {
        $(
//            impl<MODE> InputPin for $pxi<Output<MODE>> {}
            impl<MODE> embedded_hal::digital::v2::InputPin for $pxi<Input<MODE>> {
                type Error = Infallible;

                fn is_high(&self) -> Result<bool, Self::Error> {
                    Ok(unsafe {& *GPIO::ptr() }.$reg.read().$reader().bits() & (1 << $bit) != 0)
                }

                fn is_low(&self) -> Result<bool, Self::Error> {
                    Ok(!self.is_high()?)
                }
            }

            impl<MODE> $pxi<MODE> {
                fn set_input(&self, pulldown: bool, pullup: bool) {
                    let gpio = unsafe { &*GPIO::ptr() };
                    let iomux = unsafe { &*IO_MUX::ptr() };
                    self.disable_analog();

                    // NOTE(unsafe) atomic read to a stateless register
                    gpio.$out_en_clear.modify(|_, w| unsafe { w.bits(1 << $bit) });

                    gpio.func_out_sel_cfg[$pin_num].modify(|_, w| unsafe {
                        w.out_sel().bits(OutputSignal::GPIO as u16)
                    });

                    iomux.$iomux.modify(|_, w| unsafe {
                        w
                            .mcu_sel().bits(0b10)
                            .fun_ie().set_bit()
                            .fun_wpd().bit(pulldown)
                            .fun_wpu().bit(pullup)
                            .slp_sel().clear_bit()
                    });
                }

                pub fn into_floating_input(self) -> $pxi<Input<Floating>> {
                    self.set_input(false, false);
                    $pxi { _mode: PhantomData }
                }

                impl_pullup_pulldown!($out_en_clear, $pxi, $pin_num, $bit, $iomux, $resistors);
            }

            impl<MODE> Pin for $pxi<MODE> {

                /// Set drive strength
                fn set_drive_strength(&mut self, strength: DriveStrength) -> &mut Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| unsafe {
                        w.fun_drv().bits(strength as u8)
                    });
                    self
                }

                /// Enable/Disable input circuitry
                fn enable_input(&mut self, on: bool) -> &mut Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| w.fun_ie().bit(on));
                    self
                }

                /// Enable/Disable open drain
                fn enable_open_drain(&mut self, on: bool) -> &mut Self {
                    unsafe{ &*GPIO::ptr() }.pin[$pin_num].modify(|_, w| w.pad_driver().bit(on));
                    self
                }

                /// Enable/Disable internal pull up resistor
                fn internal_pull_up(&mut self, on: bool) ->&mut  Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| w.fun_wpu().bit(on));
                    self
                }

                /// Enable/Disable internal pull down resistor
                fn internal_pull_down(&mut self, on: bool) -> &mut Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| w.fun_wpd().bit(on));
                    self
                }

                /// Set drive strength
                fn set_drive_strength_in_sleep_mode(&mut self, strength: DriveStrength) -> &mut Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| unsafe {
                        w.mcu_drv().bits(strength as u8)
                    });
                    self
                }

                /// Enable/Disable input circuitry while in sleep mode
                fn input_in_sleep_mode(&mut self, on: bool) -> &mut Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| w.mcu_ie().bit(on));
                    self
                }

                /// Enable/Disable internal pull up resistor while in sleep mode
                fn internal_pull_up_in_sleep_mode(&mut self, on: bool) -> &mut Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| w.mcu_wpu().bit(on));
                    self
                }

                /// Enable/Disable internal pull down resistor while in sleep mode
                fn internal_pull_down_in_sleep_mode(&mut self, on: bool) -> &mut Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| w.mcu_wpd().bit(on));
                    self
                }

                /// Enable/Disable internal pull down resistor while in sleep mode
                fn output_in_sleep_mode(&mut self, on: bool) -> &mut Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| w.mcu_oe().bit(on));
                    self
                }

                /// Enable/Disable the sleep mode of the pad
                fn sleep_mode(&mut self, on: bool) -> &mut Self {
                    unsafe{ &*IO_MUX::ptr() }.$iomux.modify(|_, w| w.slp_sel().bit(on));
                    self
                }

                /// Connect input to peripheral
                fn connect_input_to_peripheral(&mut self, signal: InputSignal,
                    invert: bool) -> &mut Self {
                    unsafe{ &*GPIO::ptr() }.func_in_sel_cfg[signal as usize].modify(|_, w| unsafe {
                        w
                        .sel().set_bit()
                        .in_inv_sel().bit(invert)
                        .in_sel().bits($pin_num)
                    });
                    self
                }

                /// Connect peripheral to output
                fn connect_peripheral_to_output(&mut self, signal: OutputSignal, invert: bool,
                    enable_from_gpio: bool, invert_enable: bool) -> &mut Self {
                    unsafe{ &*GPIO::ptr() }.func_out_sel_cfg[$pin_num].modify(|_, w| unsafe {
                        w
                        .out_sel().bits(signal as u16)
                        .out_inv_sel().bit(invert)
                        .oen_sel().bit(enable_from_gpio)
                        .oen_inv_sel().bit(invert_enable)
                    });
                    self
                }



            }

        )+
    };
}

impl_input! {
    enable_w1tc, in_, in_data [
        Gpio0: (0, 0, gpio0, has_pullup_pulldown),
        Gpio1: (1, 1, u0txd, has_pullup_pulldown),
        Gpio2: (2, 2, gpio2, has_pullup_pulldown),
        Gpio3: (3, 3, u0rxd, has_pullup_pulldown),
        Gpio4: (4, 4, gpio4, has_pullup_pulldown),
        Gpio5: (5, 5, gpio5, has_pullup_pulldown),
        Gpio6: (6, 6, sd_clk, has_pullup_pulldown),
        Gpio7: (7, 7, sd_data0, has_pullup_pulldown),
        Gpio8: (8, 8, sd_data1, has_pullup_pulldown),
        Gpio9: (9, 9, sd_data2, has_pullup_pulldown),
        Gpio10: (10, 10, sd_data3, has_pullup_pulldown),
        Gpio11: (11, 11, sd_cmd, has_pullup_pulldown),
        Gpio12: (12, 12, mtdi, has_pullup_pulldown),
        Gpio13: (13, 13, mtck, has_pullup_pulldown),
        Gpio14: (14, 14, mtms, has_pullup_pulldown),
        Gpio15: (15, 15, mtdo, has_pullup_pulldown),
        Gpio16: (16, 16, gpio16, has_pullup_pulldown),
        Gpio17: (17, 17, gpio17, has_pullup_pulldown),
        Gpio18: (18, 18, gpio18, has_pullup_pulldown),
        Gpio19: (19, 19, gpio19, has_pullup_pulldown),
        Gpio20: (20, 20, gpio20, has_pullup_pulldown),
        Gpio21: (21, 21, gpio21, has_pullup_pulldown),
        Gpio22: (22, 22, gpio22, has_pullup_pulldown),
        Gpio23: (23, 23, gpio23, has_pullup_pulldown),
        Gpio25: (25, 25, gpio25, has_pullup_pulldown),
        Gpio26: (26, 26, gpio26, has_pullup_pulldown),
        Gpio27: (27, 27, gpio27, has_pullup_pulldown),
    ]
}

impl_input! {
    enable1_w1tc, in1, in1_data [
        Gpio32: (32, 0, gpio32, has_pullup_pulldown),
        Gpio33: (33, 1, gpio33, has_pullup_pulldown),
        Gpio34: (34, 2, gpio34, no_pullup_pulldown),
        Gpio35: (35, 3, gpio35, no_pullup_pulldown),
        Gpio36: (36, 4, gpio36, no_pullup_pulldown),
        Gpio37: (37, 5, gpio37, no_pullup_pulldown),
        Gpio38: (38, 6, gpio38, no_pullup_pulldown),
        Gpio39: (39, 7, gpio39, no_pullup_pulldown),
    ]
}

macro_rules! impl_no_analog {
    ([
        $($pxi:ident),+
    ]) => {
        $(
            impl<MODE> $pxi<MODE> {
                #[inline(always)]
                fn disable_analog(&self) {
                    /* No analog functionality on this pin, so nothing to do */
                }
            }
        )+
    };
}

macro_rules! impl_analog {
    ([
        $($pxi:ident: ($pin_num:expr, $pin_reg:ident, $mux_sel:ident, $fun_select:ident,
          $in_enable:ident, $($rue:ident, $rde:ident)?),)+
    ]) => {
        $(
            impl<MODE> $pxi<MODE> {
                pub fn into_analog(self) -> $pxi<Analog> {
                    let rtcio = unsafe{ &*RTCIO::ptr() };

                    rtcio.$pin_reg.modify(|_,w| {
                        // Connect pin to analog / RTC module instead of standard GPIO
                        w.$mux_sel().set_bit();

                        // Select function "RTC function 1" (GPIO) for analog use
                        unsafe { w.$fun_select().bits(0b00) }
                    });

                    // Configure RTC pin as normal output (instead of open drain)
                    rtcio.pin[$pin_num].modify(|_,w| w.pad_driver().clear_bit());

                    // Disable output
                    rtcio.enable_w1tc.modify(|_,w| {
                        unsafe { w.enable_w1tc().bits(1u32 << $pin_num) }
                    });

                    // Disable input
                    rtcio.$pin_reg.modify(|_,w| w.$in_enable().clear_bit());

                    // Disable pull-up and pull-down resistors on the pin, if it has them
                    $(
                        rtcio.$pin_reg.modify(|_,w| {
                            w.$rue().clear_bit();
                            w.$rde().clear_bit()
                        });
                    )?

                    $pxi { _mode: PhantomData }
                }

                #[inline(always)]
                fn disable_analog(&self) {
                    let rtcio = unsafe{ &*RTCIO::ptr() };
                    rtcio.$pin_reg.modify(|_,w| w.$mux_sel().clear_bit());
                }
            }
        )+
    }
}

impl_no_analog! {[
    Gpio1, Gpio3, Gpio5, Gpio6, Gpio7, Gpio8, Gpio9, Gpio10, Gpio11,
    Gpio16, Gpio17, Gpio18, Gpio19, Gpio20, Gpio21, Gpio22, Gpio23
]}

impl_analog! {[
    Gpio36: (0, sensor_pads, sense1_mux_sel, sense1_fun_sel, sense1_fun_ie,),
    Gpio37: (1, sensor_pads, sense2_mux_sel, sense2_fun_sel, sense2_fun_ie,),
    Gpio38: (2, sensor_pads, sense3_mux_sel, sense3_fun_sel, sense3_fun_ie,),
    Gpio39: (3, sensor_pads, sense4_mux_sel, sense4_fun_sel, sense4_fun_ie,),
    Gpio34: (4, adc_pad, adc1_mux_sel, adc1_fun_sel, adc1_fun_ie,),
    Gpio35: (5, adc_pad, adc2_mux_sel, adc2_fun_sel, adc1_fun_ie,),
    Gpio25: (6, pad_dac1, pdac1_mux_sel, pdac1_fun_sel, pdac1_fun_ie, pdac1_rue, pdac1_rde),
    Gpio26: (7, pad_dac2, pdac2_mux_sel, pdac2_fun_sel, pdac2_fun_ie, pdac2_rue, pdac2_rde),
    Gpio33: (8, xtal_32k_pad, x32n_mux_sel, x32n_fun_sel, x32n_fun_ie, x32n_rue, x32n_rde),
    Gpio32: (9, xtal_32k_pad, x32p_mux_sel, x32p_fun_sel, x32p_fun_ie, x32p_rue, x32p_rde),
    Gpio4:  (10, touch_pad0, mux_sel, fun_sel, fun_ie, rue, rde),
    Gpio0:  (11, touch_pad1, mux_sel, fun_sel, fun_ie, rue, rde),
    Gpio2:  (12, touch_pad2, mux_sel, fun_sel, fun_ie, rue, rde),
    Gpio15: (13, touch_pad3, mux_sel, fun_sel, fun_ie, rue, rde),
    Gpio13: (14, touch_pad4, mux_sel, fun_sel, fun_ie, rue, rde),
    Gpio12: (15, touch_pad5, mux_sel, fun_sel, fun_ie, rue, rde),
    Gpio14: (16, touch_pad6, mux_sel, fun_sel, fun_ie, rue, rde),
    Gpio27: (17, touch_pad7, mux_sel, fun_sel, fun_ie, rue, rde),
]}
