#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    _reserved0: [u8; 4usize],
    #[doc = "0x04 - GPIO 0-31 output register"]
    pub gpio_out_reg: GPIO_OUT_REG,
    #[doc = "0x08 - GPIO 0-31 output set register"]
    pub gpio_out_w1ts_reg: GPIO_OUT_W1TS_REG,
    #[doc = "0x0c - GPIO 0-31 output clear register"]
    pub gpio_out_w1tc_reg: GPIO_OUT_W1TC_REG,
    #[doc = "0x10 - GPIO 32-39 output register"]
    pub gpio_out1_reg: GPIO_OUT1_REG,
    #[doc = "0x14 - GPIO 32-39 output set register"]
    pub gpio_out1_w1ts_reg: GPIO_OUT1_W1TS_REG,
    #[doc = "0x18 - GPIO 32-39 output clear register"]
    pub gpio_out1_w1tc_reg: GPIO_OUT1_W1TC_REG,
    _reserved6: [u8; 4usize],
    #[doc = "0x20 - GPIO 0-31 output enable register"]
    pub gpio_enable_reg: GPIO_ENABLE_REG,
    #[doc = "0x24 - GPIO 0-31 output enable set register"]
    pub gpio_enable_w1ts_reg: GPIO_ENABLE_W1TS_REG,
    #[doc = "0x28 - GPIO 0-31 output enable clear register"]
    pub gpio_enable_w1tc_reg: GPIO_ENABLE_W1TC_REG,
    #[doc = "0x2c - GPIO 32-39 output enable register"]
    pub gpio_enable1_reg: GPIO_ENABLE1_REG,
    #[doc = "0x30 - GPIO 32-39 output enable set register"]
    pub gpio_enable1_w1ts_reg: GPIO_ENABLE1_W1TS_REG,
    #[doc = "0x34 - GPIO 32-39 output enable clear register"]
    pub gpio_enable1_w1tc_reg: GPIO_ENABLE1_W1TC_REG,
    _reserved12: [u8; 4usize],
    #[doc = "0x3c - GPIO 0-31 in register"]
    pub gpio_in_reg: GPIO_IN_REG,
    #[doc = "0x40 - GPIO 32-39 in register"]
    pub gpio_in1_reg: GPIO_IN1_REG,
    _reserved14: [u8; 236usize],
    #[doc = "0x130 - Peripheral function input selection register"]
    pub gpio_func_in_sel_cfg: [GPIO_FUNC_IN_SEL_CFG; 256],
    #[doc = "0x530 - Peripheral function input selection register"]
    pub gpio_func0_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x534 - Peripheral function input selection register"]
    pub gpio_func1_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x538 - Peripheral function input selection register"]
    pub gpio_func2_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x53c - Peripheral function input selection register"]
    pub gpio_func3_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x540 - Peripheral function input selection register"]
    pub gpio_func4_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x544 - Peripheral function input selection register"]
    pub gpio_func5_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x548 - Peripheral function input selection register"]
    pub gpio_func6_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x54c - Peripheral function input selection register"]
    pub gpio_func7_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x550 - Peripheral function input selection register"]
    pub gpio_func8_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x554 - Peripheral function input selection register"]
    pub gpio_func9_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x558 - Peripheral function input selection register"]
    pub gpio_func10_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x55c - Peripheral function input selection register"]
    pub gpio_func11_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x560 - Peripheral function input selection register"]
    pub gpio_func12_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x564 - Peripheral function input selection register"]
    pub gpio_func13_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x568 - Peripheral function input selection register"]
    pub gpio_func14_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x56c - Peripheral function input selection register"]
    pub gpio_func15_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x570 - Peripheral function input selection register"]
    pub gpio_func16_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x574 - Peripheral function input selection register"]
    pub gpio_func17_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x578 - Peripheral function input selection register"]
    pub gpio_func18_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x57c - Peripheral function input selection register"]
    pub gpio_func19_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x580 - Peripheral function input selection register"]
    pub gpio_func21_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x584 - Peripheral function input selection register"]
    pub gpio_func22_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x588 - Peripheral function input selection register"]
    pub gpio_func23_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x58c - Peripheral function input selection register"]
    pub gpio_func25_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x590 - Peripheral function input selection register"]
    pub gpio_func26_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x594 - Peripheral function input selection register"]
    pub gpio_func27_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x598 - Peripheral function input selection register"]
    pub gpio_func32_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
    #[doc = "0x59c - Peripheral function input selection register"]
    pub gpio_func33_out_sel_cfg: GPIO_FUNC_OUT_SEL_CFG,
}
#[doc = r"Register block"]
#[repr(C)]
pub struct GPIO_FUNC_IN_SEL_CFG {
    #[doc = "0x00 - Peripheral function input selection register"]
    pub gpio_func_in_sel_cfg_reg: self::gpio_func_in_sel_cfg::GPIO_FUNC_IN_SEL_CFG_REG,
}
#[doc = r"Register block"]
#[doc = "Peripheral function input selection register"]
pub mod gpio_func_in_sel_cfg;
#[doc = r"Register block"]
#[repr(C)]
pub struct GPIO_FUNC_OUT_SEL_CFG {
    #[doc = "0x00 - Peripheral output selection"]
    pub gpio_func_out_sel_cfg_reg: self::gpio_func_out_sel_cfg::GPIO_FUNC_OUT_SEL_CFG_REG,
}
#[doc = r"Register block"]
#[doc = "Peripheral function input selection register"]
pub mod gpio_func_out_sel_cfg;
#[doc = "GPIO 0-31 output register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_out_reg](gpio_out_reg) module"]
pub type GPIO_OUT_REG = crate::Reg<u32, _GPIO_OUT_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_OUT_REG;
#[doc = "`read()` method returns [gpio_out_reg::R](gpio_out_reg::R) reader structure"]
impl crate::Readable for GPIO_OUT_REG {}
#[doc = "`write(|w| ..)` method takes [gpio_out_reg::W](gpio_out_reg::W) writer structure"]
impl crate::Writable for GPIO_OUT_REG {}
#[doc = "GPIO 0-31 output register"]
pub mod gpio_out_reg;
#[doc = "GPIO 0-31 output set register\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_out_w1ts_reg](gpio_out_w1ts_reg) module"]
pub type GPIO_OUT_W1TS_REG = crate::Reg<u32, _GPIO_OUT_W1TS_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_OUT_W1TS_REG;
#[doc = "`write(|w| ..)` method takes [gpio_out_w1ts_reg::W](gpio_out_w1ts_reg::W) writer structure"]
impl crate::Writable for GPIO_OUT_W1TS_REG {}
#[doc = "GPIO 0-31 output set register"]
pub mod gpio_out_w1ts_reg;
#[doc = "GPIO 0-31 output clear register\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_out_w1tc_reg](gpio_out_w1tc_reg) module"]
pub type GPIO_OUT_W1TC_REG = crate::Reg<u32, _GPIO_OUT_W1TC_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_OUT_W1TC_REG;
#[doc = "`write(|w| ..)` method takes [gpio_out_w1tc_reg::W](gpio_out_w1tc_reg::W) writer structure"]
impl crate::Writable for GPIO_OUT_W1TC_REG {}
#[doc = "GPIO 0-31 output clear register"]
pub mod gpio_out_w1tc_reg;
#[doc = "GPIO 32-39 output register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_out1_reg](gpio_out1_reg) module"]
pub type GPIO_OUT1_REG = crate::Reg<u32, _GPIO_OUT1_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_OUT1_REG;
#[doc = "`read()` method returns [gpio_out1_reg::R](gpio_out1_reg::R) reader structure"]
impl crate::Readable for GPIO_OUT1_REG {}
#[doc = "`write(|w| ..)` method takes [gpio_out1_reg::W](gpio_out1_reg::W) writer structure"]
impl crate::Writable for GPIO_OUT1_REG {}
#[doc = "GPIO 32-39 output register"]
pub mod gpio_out1_reg;
#[doc = "GPIO 32-39 output set register\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_out1_w1ts_reg](gpio_out1_w1ts_reg) module"]
pub type GPIO_OUT1_W1TS_REG = crate::Reg<u32, _GPIO_OUT1_W1TS_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_OUT1_W1TS_REG;
#[doc = "`write(|w| ..)` method takes [gpio_out1_w1ts_reg::W](gpio_out1_w1ts_reg::W) writer structure"]
impl crate::Writable for GPIO_OUT1_W1TS_REG {}
#[doc = "GPIO 32-39 output set register"]
pub mod gpio_out1_w1ts_reg;
#[doc = "GPIO 32-39 output clear register\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_out1_w1tc_reg](gpio_out1_w1tc_reg) module"]
pub type GPIO_OUT1_W1TC_REG = crate::Reg<u32, _GPIO_OUT1_W1TC_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_OUT1_W1TC_REG;
#[doc = "`write(|w| ..)` method takes [gpio_out1_w1tc_reg::W](gpio_out1_w1tc_reg::W) writer structure"]
impl crate::Writable for GPIO_OUT1_W1TC_REG {}
#[doc = "GPIO 32-39 output clear register"]
pub mod gpio_out1_w1tc_reg;
#[doc = "GPIO 0-31 output enable register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_enable_reg](gpio_enable_reg) module"]
pub type GPIO_ENABLE_REG = crate::Reg<u32, _GPIO_ENABLE_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_ENABLE_REG;
#[doc = "`read()` method returns [gpio_enable_reg::R](gpio_enable_reg::R) reader structure"]
impl crate::Readable for GPIO_ENABLE_REG {}
#[doc = "`write(|w| ..)` method takes [gpio_enable_reg::W](gpio_enable_reg::W) writer structure"]
impl crate::Writable for GPIO_ENABLE_REG {}
#[doc = "GPIO 0-31 output enable register"]
pub mod gpio_enable_reg;
#[doc = "GPIO 0-31 output enable set register\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_enable_w1ts_reg](gpio_enable_w1ts_reg) module"]
pub type GPIO_ENABLE_W1TS_REG = crate::Reg<u32, _GPIO_ENABLE_W1TS_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_ENABLE_W1TS_REG;
#[doc = "`write(|w| ..)` method takes [gpio_enable_w1ts_reg::W](gpio_enable_w1ts_reg::W) writer structure"]
impl crate::Writable for GPIO_ENABLE_W1TS_REG {}
#[doc = "GPIO 0-31 output enable set register"]
pub mod gpio_enable_w1ts_reg;
#[doc = "GPIO 0-31 output enable clear register\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_enable_w1tc_reg](gpio_enable_w1tc_reg) module"]
pub type GPIO_ENABLE_W1TC_REG = crate::Reg<u32, _GPIO_ENABLE_W1TC_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_ENABLE_W1TC_REG;
#[doc = "`write(|w| ..)` method takes [gpio_enable_w1tc_reg::W](gpio_enable_w1tc_reg::W) writer structure"]
impl crate::Writable for GPIO_ENABLE_W1TC_REG {}
#[doc = "GPIO 0-31 output enable clear register"]
pub mod gpio_enable_w1tc_reg;
#[doc = "GPIO 32-39 output enable register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_enable1_reg](gpio_enable1_reg) module"]
pub type GPIO_ENABLE1_REG = crate::Reg<u32, _GPIO_ENABLE1_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_ENABLE1_REG;
#[doc = "`read()` method returns [gpio_enable1_reg::R](gpio_enable1_reg::R) reader structure"]
impl crate::Readable for GPIO_ENABLE1_REG {}
#[doc = "`write(|w| ..)` method takes [gpio_enable1_reg::W](gpio_enable1_reg::W) writer structure"]
impl crate::Writable for GPIO_ENABLE1_REG {}
#[doc = "GPIO 32-39 output enable register"]
pub mod gpio_enable1_reg;
#[doc = "GPIO 32-39 output enable set register\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_enable1_w1ts_reg](gpio_enable1_w1ts_reg) module"]
pub type GPIO_ENABLE1_W1TS_REG = crate::Reg<u32, _GPIO_ENABLE1_W1TS_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_ENABLE1_W1TS_REG;
#[doc = "`write(|w| ..)` method takes [gpio_enable1_w1ts_reg::W](gpio_enable1_w1ts_reg::W) writer structure"]
impl crate::Writable for GPIO_ENABLE1_W1TS_REG {}
#[doc = "GPIO 32-39 output enable set register"]
pub mod gpio_enable1_w1ts_reg;
#[doc = "GPIO 32-39 output enable clear register\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_enable1_w1tc_reg](gpio_enable1_w1tc_reg) module"]
pub type GPIO_ENABLE1_W1TC_REG = crate::Reg<u32, _GPIO_ENABLE1_W1TC_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_ENABLE1_W1TC_REG;
#[doc = "`write(|w| ..)` method takes [gpio_enable1_w1tc_reg::W](gpio_enable1_w1tc_reg::W) writer structure"]
impl crate::Writable for GPIO_ENABLE1_W1TC_REG {}
#[doc = "GPIO 32-39 output enable clear register"]
pub mod gpio_enable1_w1tc_reg;
#[doc = "GPIO 0-31 in register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_in_reg](gpio_in_reg) module"]
pub type GPIO_IN_REG = crate::Reg<u32, _GPIO_IN_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_IN_REG;
#[doc = "`read()` method returns [gpio_in_reg::R](gpio_in_reg::R) reader structure"]
impl crate::Readable for GPIO_IN_REG {}
#[doc = "`write(|w| ..)` method takes [gpio_in_reg::W](gpio_in_reg::W) writer structure"]
impl crate::Writable for GPIO_IN_REG {}
#[doc = "GPIO 0-31 in register"]
pub mod gpio_in_reg;
#[doc = "GPIO 32-39 in register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_in1_reg](gpio_in1_reg) module"]
pub type GPIO_IN1_REG = crate::Reg<u32, _GPIO_IN1_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_IN1_REG;
#[doc = "`read()` method returns [gpio_in1_reg::R](gpio_in1_reg::R) reader structure"]
impl crate::Readable for GPIO_IN1_REG {}
#[doc = "`write(|w| ..)` method takes [gpio_in1_reg::W](gpio_in1_reg::W) writer structure"]
impl crate::Writable for GPIO_IN1_REG {}
#[doc = "GPIO 32-39 in register"]
pub mod gpio_in1_reg;
