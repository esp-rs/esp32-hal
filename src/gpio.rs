use {
    crate::pac::{GPIO, IO_MUX},
    core::{convert::Infallible, marker::PhantomData},
    embedded_hal::digital::v2::{InputPin, OutputPin},
};

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

/// Alternate function
pub struct Alternate<MODE> {
    _mode: PhantomData<MODE>,
}

macro_rules! gpio {
    ($GPIO:ident: [
        $($pxi:ident: ($pname:ident, $MODE:ty),)+
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
        )+

    };
}

// All info on reset state pulled from 4.10 IO_MUX Pad List in the reference manual
gpio! {
   GPIO: [
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
    // TODO all these really missing?
    //    Gpio24: (gpio24, Input<Floating>),
    //    Gpio28: (gpio28, Input<Floating>),
    //    Gpio29: (gpio29, Input<Floating>),
    //    Gpio30: (gpio30, Input<Floating>),
    //    Gpio31: (gpio31, Input<Floating>),
       Gpio32: (gpio32, Input<Floating>),
       Gpio33: (gpio33, Input<Floating>),
       Gpio34: (gpio34, Input<Floating>),
       Gpio35: (gpio35, Input<Floating>),
       Gpio36: (gpio36, Input<Floating>),
       Gpio37: (gpio37, Input<Floating>),
       Gpio38: (gpio38, Input<Floating>),
       Gpio39: (gpio39, Input<Floating>),
   ]
}

macro_rules! impl_output {
    ($en:ident, $outs:ident, $outc:ident, [
        // index, gpio pin name, funcX name, iomux pin name
        $($pxi:ident: ($i:expr, $pin:ident, $funcXout:ident, $iomux:ident),)+
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

                    iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                    $pxi { _mode: PhantomData }
                }
            }
        )+
    };
}

impl_output! {
    enable_w1ts, out_w1ts, out_w1tc, [
        Gpio0: (0, pin0, func0_out_sel_cfg, gpio0),
        Gpio1: (1, pin1, func1_out_sel_cfg, u0txd),
        Gpio2: (2, pin2, func2_out_sel_cfg, gpio2),
        Gpio3: (3, pin3, func3_out_sel_cfg, u0rxd),
        Gpio4: (4, pin4, func4_out_sel_cfg, gpio4),
        Gpio5: (5, pin5, func5_out_sel_cfg, gpio5),
        Gpio6: (6, pin6, func6_out_sel_cfg, sd_clk),
        Gpio7: (7, pin7, func7_out_sel_cfg, sd_data0),
        Gpio8: (8, pin8, func8_out_sel_cfg, sd_data1),
        Gpio9: (9, pin9, func9_out_sel_cfg, sd_data2),
        Gpio10: (10, pin10, func10_out_sel_cfg, sd_data3),
        Gpio11: (11, pin11, func11_out_sel_cfg, sd_cmd),
        Gpio12: (12, pin12, func12_out_sel_cfg, mtdi),
        Gpio13: (13, pin13, func13_out_sel_cfg, mtck),
        Gpio14: (14, pin14, func14_out_sel_cfg, mtms),
        Gpio15: (15, pin15, func15_out_sel_cfg, mtdo),
        Gpio16: (16, pin16, func16_out_sel_cfg, gpio16),
        Gpio17: (17, pin17, func17_out_sel_cfg, gpio17),
        Gpio18: (18, pin18, func18_out_sel_cfg, gpio18),
        Gpio19: (19, pin19, func19_out_sel_cfg, gpio19),
        Gpio20: (20, pin20, func20_out_sel_cfg, gpio20),
        Gpio21: (21, pin21, func21_out_sel_cfg, gpio21),
        Gpio22: (22, pin22, func22_out_sel_cfg, gpio22),
        Gpio23: (23, pin23, func23_out_sel_cfg, gpio23),
        Gpio25: (25, pin25, func25_out_sel_cfg, gpio25),
        Gpio26: (26, pin26, func26_out_sel_cfg, gpio26),
        Gpio27: (27, pin27, func27_out_sel_cfg, gpio27),
    ]
}

impl_output! {
    enable_w1ts, out1_w1ts, out1_w1tc, [
        Gpio32: (0, pin32, func32_out_sel_cfg, gpio32),
        Gpio33: (1, pin33, func33_out_sel_cfg, gpio33),
    ]
}

macro_rules! impl_input {
    ($en:ident, $reg:ident, $reader:ident [
        // index, gpio pin name, funcX name, iomux pin name, iomux mcu_sel bits
        $($pxi:ident: ($i:expr, $pin:ident, $funcXin:ident, $iomux:ident),)+
    ]) => {
        $(
            impl<MODE> InputPin for $pxi<Input<MODE>> {
                type Error = Infallible;

                fn is_high(&self) -> Result<bool, Self::Error> {
                    Ok(unsafe {& *GPIO::ptr() }.$reg.read().$reader().bits() & (1 << $i) != 0)
                }

                fn is_low(&self) -> Result<bool, Self::Error> {
                    Ok(!self.is_high()?)
                }
            }

            impl<MODE> $pxi<MODE> {
                pub fn into_floating_input(self) -> $pxi<Input<Floating>> {
                    let gpio = unsafe{ &*GPIO::ptr() };
                    let iomux = unsafe{ &*IO_MUX::ptr() };
                    gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                    gpio.$funcXin.modify(|_, w| unsafe { w.bits(0x100) });

                    iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                    iomux.$iomux.modify(|_, w| w.fun_ie().set_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpd().clear_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpu().clear_bit());
                    $pxi { _mode: PhantomData }
                }

                pub fn into_pull_up_input(self) -> $pxi<Input<PullUp>> {
                    let gpio = unsafe{ &*GPIO::ptr() };
                    let iomux = unsafe{ &*IO_MUX::ptr() };
                    gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                    gpio.$funcXin.modify(|_, w| unsafe { w.bits(0x100) });

                    iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                    iomux.$iomux.modify(|_, w| w.fun_ie().set_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpd().clear_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpu().set_bit());
                    $pxi { _mode: PhantomData }
                }

                pub fn into_pull_down_input(self) -> $pxi<Input<PullDown>> {
                    let gpio = unsafe{ &*GPIO::ptr() };
                    let iomux = unsafe{ &*IO_MUX::ptr() };
                    gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                    gpio.$funcXin.modify(|_, w| unsafe { w.bits(0x100) });

                    iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                    iomux.$iomux.modify(|_, w| w.fun_ie().set_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpd().set_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpu().clear_bit());
                    $pxi { _mode: PhantomData }
                }
            }
        )+
    };
}

impl_input! {
    enable_w1ts, in_, in_data [
        Gpio0: (0, pin0, func0_in_sel_cfg, gpio0),
        Gpio1: (1, pin1, func1_in_sel_cfg, u0txd),
        Gpio2: (2, pin2, func2_in_sel_cfg, gpio2),
        Gpio3: (3, pin3, func3_in_sel_cfg, u0rxd),
        Gpio4: (4, pin4, func4_in_sel_cfg, gpio4),
        Gpio5: (5, pin5, func5_in_sel_cfg, gpio5),
        Gpio6: (6, pin6, func6_in_sel_cfg, sd_clk),
        Gpio7: (7, pin7, func7_in_sel_cfg, sd_data0),
        Gpio8: (8, pin8, func8_in_sel_cfg, sd_data1),
        Gpio9: (9, pin9, func9_in_sel_cfg, sd_data2),
        Gpio10: (10, pin10, func10_in_sel_cfg, sd_data3),
        Gpio11: (11, pin11, func11_in_sel_cfg, sd_cmd),
        Gpio12: (12, pin12, func12_in_sel_cfg, mtdi),
        Gpio13: (13, pin13, func13_in_sel_cfg, mtck),
        Gpio14: (14, pin14, func14_in_sel_cfg, mtms),
        Gpio15: (15, pin15, func15_in_sel_cfg, mtdo),
        Gpio16: (16, pin16, func16_in_sel_cfg, gpio16),
        Gpio17: (17, pin17, func17_in_sel_cfg, gpio17),
        Gpio18: (18, pin18, func18_in_sel_cfg, gpio18),
        Gpio19: (19, pin19, func19_in_sel_cfg, gpio19),
        Gpio20: (20, pin20, func20_in_sel_cfg, gpio20),
        Gpio21: (21, pin21, func21_in_sel_cfg, gpio21),
        Gpio22: (22, pin22, func22_in_sel_cfg, gpio22),
        Gpio23: (23, pin23, func23_in_sel_cfg, gpio23),
        Gpio25: (25, pin25, func25_in_sel_cfg, gpio25),
        Gpio26: (26, pin26, func26_in_sel_cfg, gpio26),
        Gpio27: (27, pin27, func27_in_sel_cfg, gpio27),
    ]
}

impl_input! {
    enable_w1ts, in1, in1_data [
        Gpio32: (0, pin32, func32_in_sel_cfg, gpio32),
        Gpio33: (1, pin33, func33_in_sel_cfg, gpio33),
        Gpio34: (2, pin34, func34_in_sel_cfg, gpio34),
        Gpio35: (3, pin35, func35_in_sel_cfg, gpio35),
        Gpio36: (4, pin36, func36_in_sel_cfg, gpio36),
        Gpio37: (5, pin37, func37_in_sel_cfg, gpio37),
        Gpio38: (6, pin38, func38_in_sel_cfg, gpio38),
        Gpio39: (7, pin39, func39_in_sel_cfg, gpio39),
    ]
}
