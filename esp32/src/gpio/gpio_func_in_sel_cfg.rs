#[doc = "Peripheral function input selection register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about avaliable fields see [gpio_func_in_sel_cfg_reg](gpio_func_in_sel_cfg_reg) module"]
pub type GPIO_FUNC_IN_SEL_CFG_REG = crate::Reg<u32, _GPIO_FUNC_IN_SEL_CFG_REG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GPIO_FUNC_IN_SEL_CFG_REG;
#[doc = "`read()` method returns [gpio_func_in_sel_cfg_reg::R](gpio_func_in_sel_cfg_reg::R) reader structure"]
impl crate::Readable for GPIO_FUNC_IN_SEL_CFG_REG {}
#[doc = "`write(|w| ..)` method takes [gpio_func_in_sel_cfg_reg::W](gpio_func_in_sel_cfg_reg::W) writer structure"]
impl crate::Writable for GPIO_FUNC_IN_SEL_CFG_REG {}
#[doc = "Peripheral function input selection register"]
pub mod gpio_func_in_sel_cfg_reg;
