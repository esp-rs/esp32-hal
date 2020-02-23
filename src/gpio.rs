
use core::marker::PhantomData;
use core::convert::Infallible;

use esp32::{GPIO, IO_MUX};
use embedded_hal::digital::v2::OutputPin;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
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

/// Generic GPIO type
pub struct Gpio<MODE> {
    /// The GPIO pin number
    pub pin: u8,
    _mode: PhantomData<MODE>,
}

// TODO: implement into_*_output functions for `Gpio`

impl<MODE> OutputPin for Gpio<Output<MODE>> {
    type Error = Infallible;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        // NOTE(unsafe) atomic write to a stateless register
        let gpio = unsafe { &(*GPIO::ptr()) };
        match self.pin {
            0..=31 => {
                unsafe {
                    gpio.out_w1ts.write(|w| w.bits(1 << self.pin))
                };
            }
            32..=33 => {
                unsafe {
                    gpio.out_w1ts.write(|w| w.bits(1 << (self.pin - 32)))
                };
            }
            _ => unreachable!()
        }
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        // NOTE(unsafe) atomic write to a stateless register
        let gpio = unsafe { &(*GPIO::ptr()) };
        match self.pin {
            0..=31 => {
                unsafe {
                    gpio.out_w1tc.write(|w| w.bits(1 << self.pin))
                };
            }
            32..=33 => {
                unsafe {
                    gpio.out_w1tc.write(|w| w.bits(1 << (self.pin - 32)))
                };
            }
            _ => unreachable!()
        }
        Ok(())
    }
}

macro_rules! gpio {
    ($GPIO:ident: [
        $($pxi:ident: ($i:expr, $pname:ident, $MODE:ty),)+
    ]) => {

        impl GpioExt for $GPIO {
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

            impl<MODE> $pxi<MODE> {
                /// Downgrade this pin to a generic Gpio type.
                pub fn downgrade(self) -> Gpio<MODE> {
                    Gpio {
                        pin: $i,
                        _mode: PhantomData,
                    }
                }
            }
        )+
    };
}

// All info on reset state pulled from 4.10 IO_MUX Pad List in the reference manual
gpio! {
   GPIO: [
       Gpio0: (0, gpio0, Input<PullUp>),
       Gpio1: (1, gpio1, Input<PullUp>),
       Gpio2: (2, gpio2, Input<PullDown>),
       Gpio3: (3, gpio3, Input<PullUp>),
       Gpio4: (4, gpio4, Input<PullDown>),
       Gpio5: (5, gpio5, Input<PullUp>),
       Gpio6: (6, gpio6, Input<PullUp>),
       Gpio7: (7, gpio7, Input<PullUp>),
       Gpio8: (8, gpio8, Input<PullUp>),
       Gpio9: (9, gpio9, Input<PullUp>),
       Gpio10: (10, gpio10, Input<PullUp>),
       Gpio11: (11, gpio11, Input<PullUp>),
       Gpio12: (12, gpio12, Input<PullDown>),
       Gpio13: (13, gpio13, Input<Floating>),
       Gpio14: (14, gpio14, Input<Floating>),
       Gpio15: (15, gpio15, Input<PullUp>),
       Gpio16: (16, gpio16, Input<Floating>),
       Gpio17: (17, gpio17, Input<Floating>),
       Gpio18: (18, gpio18, Input<Floating>),
       Gpio19: (19, gpio19, Input<Floating>),
       Gpio20: (10, gpio20, Input<Floating>),
       Gpio21: (21, gpio21, Input<Floating>),
       Gpio22: (22, gpio22, Input<Floating>),
       Gpio23: (23, gpio23, Input<Floating>),
       // TODO these pins have a reset mode of 0 (apart from Gpio27),
       // input disable, does that mean they are actually in output mode on reset?
       Gpio25: (25, gpio25, Input<Floating>),
       Gpio26: (26, gpio26, Input<Floating>),
       Gpio27: (27, gpio27, Input<Floating>),
    // TODO all these really missing?
    //    Gpio24: (24, gpio24, Input<Floating>),
    //    Gpio28: (28, gpio28, Input<Floating>),
    //    Gpio29: (29, gpio29, Input<Floating>),
    //    Gpio30: (30, gpio30, Input<Floating>),
    //    Gpio31: (31, gpio31, Input<Floating>),
       Gpio32: (32, gpio32, Input<Floating>),
       Gpio33: (33, gpio33, Input<Floating>),
       Gpio34: (34, gpio34, Input<Floating>),
       Gpio35: (35, gpio35, Input<Floating>),
       Gpio36: (36, gpio36, Input<Floating>),
       Gpio37: (37, gpio37, Input<Floating>),
       Gpio38: (38, gpio38, Input<Floating>),
       Gpio39: (39, gpio39, Input<Floating>),
   ]
}

macro_rules! impl_output {
    ($en:ident, $outs:ident, $outc:ident, [
        // index, gpio pin name, funcX name, iomux pin name, iomux mcu_sel bits
        $($pxi:ident: ($i:expr, $pin:ident, $funcXout:ident, $iomux:ident, $mcu_sel_bits:expr),)+
    ]) => {
        $(
            impl<MODE> OutputPin for $pxi<Output<MODE>> {
                type Error = Infallible;

                fn set_high(&mut self) -> Result<(), Self::Error> {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*GPIO::ptr()).$outs.write(|w| w.bits(1 << $i)) };
                    Ok(())
                }

                fn set_low(&mut self) -> Result<(), Self::Error> {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*GPIO::ptr()).$outc.write(|w| w.bits(1 << $i)) };
                    Ok(())
                }
            }

            impl<MODE> $pxi<MODE> {
                pub fn into_push_pull_output(self) -> $pxi<Output<PushPull>> {
                    let gpio = unsafe{ &*GPIO::ptr() };
                    let iomux = unsafe{ &*IO_MUX::ptr() };
                    gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                    gpio.$funcXout.modify(|_, w| unsafe { w.bits(0x100) });

                    iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits($mcu_sel_bits) });
                    $pxi { _mode: PhantomData }
                }
            }
        )+
    };
}

impl_output! {
    enable_w1ts, out_w1ts, out_w1tc, [
        Gpio0: (0, pin0, func0_out_sel_cfg, gpio0, 0b00),
        Gpio1: (1, pin1, func1_out_sel_cfg, u0txd, 0b10),
        Gpio2: (2, pin2, func2_out_sel_cfg, gpio2, 0b00),
        Gpio3: (3, pin3, func3_out_sel_cfg, u0rxd, 0b10),
        Gpio4: (4, pin4, func4_out_sel_cfg, gpio4, 0b10),
        Gpio5: (5, pin5, func5_out_sel_cfg, gpio5, 0b10),
        Gpio6: (6, pin6, func6_out_sel_cfg, sd_clk, 0b10),
        Gpio7: (7, pin7, func7_out_sel_cfg, sd_data0, 0b10),
        Gpio8: (8, pin8, func8_out_sel_cfg, sd_data1, 0b10),
        Gpio9: (9, pin9, func9_out_sel_cfg, sd_data2, 0b10),
        Gpio10: (10, pin10, func10_out_sel_cfg, sd_data3, 0b10),
        Gpio11: (11, pin11, func11_out_sel_cfg, sd_cmd, 0b10),
        Gpio12: (12, pin12, func12_out_sel_cfg, mtdi, 0b10),
        Gpio13: (13, pin13, func13_out_sel_cfg, mtck, 0b10),
        Gpio14: (14, pin14, func14_out_sel_cfg, mtms, 0b10),
        Gpio15: (15, pin15, func15_out_sel_cfg, mtdo, 0b10),
        Gpio16: (16, pin16, func16_out_sel_cfg, gpio16, 0b10),
        Gpio17: (17, pin17, func17_out_sel_cfg, gpio17, 0b10),
        Gpio18: (18, pin18, func18_out_sel_cfg, gpio18, 0b10),
        Gpio19: (19, pin19, func19_out_sel_cfg, gpio19, 0b10),
        Gpio20: (20, pin20, func20_out_sel_cfg, gpio20, 0b10),
        Gpio21: (21, pin21, func21_out_sel_cfg, gpio21, 0b10),
        Gpio22: (22, pin22, func22_out_sel_cfg, gpio22, 0b10),
        Gpio23: (23, pin23, func23_out_sel_cfg, gpio23, 0b10),
        Gpio25: (25, pin25, func25_out_sel_cfg, gpio25, 0b10),
        Gpio26: (26, pin26, func26_out_sel_cfg, gpio26, 0b10),
        Gpio27: (27, pin27, func27_out_sel_cfg, gpio27, 0b10),
    ]
}

impl_output! {
    enable1_w1ts, out1_w1ts, out1_w1tc, [
        Gpio32: (0, pin32, func32_out_sel_cfg, gpio32, 0b00),
        Gpio33: (1, pin33, func33_out_sel_cfg, gpio33, 0b00),
        /* Deliberately omitting 34-39 as these can *only* be inputs */
    ]
}
