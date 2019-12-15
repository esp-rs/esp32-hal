#[doc = "Reader of register GPIO_FUNC_OUT_SEL_CFG_REG"]
pub type R = crate::R<u32, super::GPIO_FUNC_OUT_SEL_CFG_REG>;
#[doc = "Writer for register GPIO_FUNC_OUT_SEL_CFG_REG"]
pub type W = crate::W<u32, super::GPIO_FUNC_OUT_SEL_CFG_REG>;
#[doc = "Register GPIO_FUNC_OUT_SEL_CFG_REG `reset()`'s with value 0"]
impl crate::ResetValue for super::GPIO_FUNC_OUT_SEL_CFG_REG {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `GPIO_FUNC_OUT_SEL`"]
pub type GPIO_FUNC_OUT_SEL_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `GPIO_FUNC_OUT_SEL`"]
pub struct GPIO_FUNC_OUT_SEL_W<'a> {
    w: &'a mut W,
}
impl<'a> GPIO_FUNC_OUT_SEL_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x01ff) | ((value as u32) & 0x01ff);
        self.w
    }
}
#[doc = "Invert the input value. 1: invert; 0: do not invert.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPIO_FUNC_OUT_INV_SEL_A {
    #[doc = "0: No invert."]
    NO_INVERT,
    #[doc = "1: Invert."]
    INVERT,
}
impl From<GPIO_FUNC_OUT_INV_SEL_A> for bool {
    #[inline(always)]
    fn from(variant: GPIO_FUNC_OUT_INV_SEL_A) -> Self {
        match variant {
            GPIO_FUNC_OUT_INV_SEL_A::NO_INVERT => false,
            GPIO_FUNC_OUT_INV_SEL_A::INVERT => true,
        }
    }
}
#[doc = "Reader of field `GPIO_FUNC_OUT_INV_SEL`"]
pub type GPIO_FUNC_OUT_INV_SEL_R = crate::R<bool, GPIO_FUNC_OUT_INV_SEL_A>;
impl GPIO_FUNC_OUT_INV_SEL_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> GPIO_FUNC_OUT_INV_SEL_A {
        match self.bits {
            false => GPIO_FUNC_OUT_INV_SEL_A::NO_INVERT,
            true => GPIO_FUNC_OUT_INV_SEL_A::INVERT,
        }
    }
    #[doc = "Checks if the value of the field is `NO_INVERT`"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == GPIO_FUNC_OUT_INV_SEL_A::NO_INVERT
    }
    #[doc = "Checks if the value of the field is `INVERT`"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == GPIO_FUNC_OUT_INV_SEL_A::INVERT
    }
}
#[doc = "Write proxy for field `GPIO_FUNC_OUT_INV_SEL`"]
pub struct GPIO_FUNC_OUT_INV_SEL_W<'a> {
    w: &'a mut W,
}
impl<'a> GPIO_FUNC_OUT_INV_SEL_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: GPIO_FUNC_OUT_INV_SEL_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "No invert."]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut W {
        self.variant(GPIO_FUNC_OUT_INV_SEL_A::NO_INVERT)
    }
    #[doc = "Invert."]
    #[inline(always)]
    pub fn invert(self) -> &'a mut W {
        self.variant(GPIO_FUNC_OUT_INV_SEL_A::INVERT)
    }
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u32) & 0x01) << 9);
        self.w
    }
}
#[doc = "1: Force the output enable signal to be sourced from bitnofGPIO_ENABLE_REG; 0: use output enable signal from peripheral. (R/W)\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPIO_FUNC_OEN_SEL_A {
    #[doc = "0: Use output enable signal from peripheral. "]
    LOW,
    #[doc = "1: 1:  Force the output enable signal to be sourced from bit n of GPIO_ENABLE_REG"]
    HIGH,
}
impl From<GPIO_FUNC_OEN_SEL_A> for bool {
    #[inline(always)]
    fn from(variant: GPIO_FUNC_OEN_SEL_A) -> Self {
        match variant {
            GPIO_FUNC_OEN_SEL_A::LOW => false,
            GPIO_FUNC_OEN_SEL_A::HIGH => true,
        }
    }
}
#[doc = "Reader of field `GPIO_FUNC_OEN_SEL`"]
pub type GPIO_FUNC_OEN_SEL_R = crate::R<bool, GPIO_FUNC_OEN_SEL_A>;
impl GPIO_FUNC_OEN_SEL_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> GPIO_FUNC_OEN_SEL_A {
        match self.bits {
            false => GPIO_FUNC_OEN_SEL_A::LOW,
            true => GPIO_FUNC_OEN_SEL_A::HIGH,
        }
    }
    #[doc = "Checks if the value of the field is `LOW`"]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == GPIO_FUNC_OEN_SEL_A::LOW
    }
    #[doc = "Checks if the value of the field is `HIGH`"]
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == GPIO_FUNC_OEN_SEL_A::HIGH
    }
}
#[doc = "Write proxy for field `GPIO_FUNC_OEN_SEL`"]
pub struct GPIO_FUNC_OEN_SEL_W<'a> {
    w: &'a mut W,
}
impl<'a> GPIO_FUNC_OEN_SEL_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: GPIO_FUNC_OEN_SEL_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Use output enable signal from peripheral."]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(GPIO_FUNC_OEN_SEL_A::LOW)
    }
    #[doc = "1: Force the output enable signal to be sourced from bit n of GPIO_ENABLE_REG"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(GPIO_FUNC_OEN_SEL_A::HIGH)
    }
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u32) & 0x01) << 10);
        self.w
    }
}
#[doc = "1: Invert the output enable signal; 0: do not invert the output enablesignal. (R/W)\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPIO_FUNC_OEN_INV_SEL_A {
    #[doc = "0: Do not invert the output enable signal."]
    NO_INVERT,
    #[doc = "1: Invert the output enable signal"]
    INVERT,
}
impl From<GPIO_FUNC_OEN_INV_SEL_A> for bool {
    #[inline(always)]
    fn from(variant: GPIO_FUNC_OEN_INV_SEL_A) -> Self {
        match variant {
            GPIO_FUNC_OEN_INV_SEL_A::NO_INVERT => false,
            GPIO_FUNC_OEN_INV_SEL_A::INVERT => true,
        }
    }
}
#[doc = "Reader of field `GPIO_FUNC_OEN_INV_SEL`"]
pub type GPIO_FUNC_OEN_INV_SEL_R = crate::R<bool, GPIO_FUNC_OEN_INV_SEL_A>;
impl GPIO_FUNC_OEN_INV_SEL_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> GPIO_FUNC_OEN_INV_SEL_A {
        match self.bits {
            false => GPIO_FUNC_OEN_INV_SEL_A::NO_INVERT,
            true => GPIO_FUNC_OEN_INV_SEL_A::INVERT,
        }
    }
    #[doc = "Checks if the value of the field is `NO_INVERT`"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == GPIO_FUNC_OEN_INV_SEL_A::NO_INVERT
    }
    #[doc = "Checks if the value of the field is `INVERT`"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == GPIO_FUNC_OEN_INV_SEL_A::INVERT
    }
}
#[doc = "Write proxy for field `GPIO_FUNC_OEN_INV_SEL`"]
pub struct GPIO_FUNC_OEN_INV_SEL_W<'a> {
    w: &'a mut W,
}
impl<'a> GPIO_FUNC_OEN_INV_SEL_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: GPIO_FUNC_OEN_INV_SEL_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Do not invert the output enable signal."]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut W {
        self.variant(GPIO_FUNC_OEN_INV_SEL_A::NO_INVERT)
    }
    #[doc = "Invert the output enable signal"]
    #[inline(always)]
    pub fn invert(self) -> &'a mut W {
        self.variant(GPIO_FUNC_OEN_INV_SEL_A::INVERT)
    }
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 11)) | (((value as u32) & 0x01) << 11);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:8 - Selection control for GPIO outputn. A value ofs(0<=s<256)connects peripheral outputsto GPIO outputn. A value of 256 selects bitnofGPIO_OUT_REG/GPIO_OUT1_REG and GPIO_ENABLE_REG/GPIO_ENABLE1_REG as the out-put value and output enable."]
    #[inline(always)]
    pub fn gpio_func_out_sel(&self) -> GPIO_FUNC_OUT_SEL_R {
        GPIO_FUNC_OUT_SEL_R::new((self.bits & 0x01ff) as u16)
    }
    #[doc = "Bit 9 - Invert the input value. 1: invert; 0: do not invert."]
    #[inline(always)]
    pub fn gpio_func_out_inv_sel(&self) -> GPIO_FUNC_OUT_INV_SEL_R {
        GPIO_FUNC_OUT_INV_SEL_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - 1: Force the output enable signal to be sourced from bitnofGPIO_ENABLE_REG; 0: use output enable signal from peripheral. (R/W)"]
    #[inline(always)]
    pub fn gpio_func_oen_sel(&self) -> GPIO_FUNC_OEN_SEL_R {
        GPIO_FUNC_OEN_SEL_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - 1: Invert the output enable signal; 0: do not invert the output enablesignal. (R/W)"]
    #[inline(always)]
    pub fn gpio_func_oen_inv_sel(&self) -> GPIO_FUNC_OEN_INV_SEL_R {
        GPIO_FUNC_OEN_INV_SEL_R::new(((self.bits >> 11) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:8 - Selection control for GPIO outputn. A value ofs(0<=s<256)connects peripheral outputsto GPIO outputn. A value of 256 selects bitnofGPIO_OUT_REG/GPIO_OUT1_REG and GPIO_ENABLE_REG/GPIO_ENABLE1_REG as the out-put value and output enable."]
    #[inline(always)]
    pub fn gpio_func_out_sel(&mut self) -> GPIO_FUNC_OUT_SEL_W {
        GPIO_FUNC_OUT_SEL_W { w: self }
    }
    #[doc = "Bit 9 - Invert the input value. 1: invert; 0: do not invert."]
    #[inline(always)]
    pub fn gpio_func_out_inv_sel(&mut self) -> GPIO_FUNC_OUT_INV_SEL_W {
        GPIO_FUNC_OUT_INV_SEL_W { w: self }
    }
    #[doc = "Bit 10 - 1: Force the output enable signal to be sourced from bitnofGPIO_ENABLE_REG; 0: use output enable signal from peripheral. (R/W)"]
    #[inline(always)]
    pub fn gpio_func_oen_sel(&mut self) -> GPIO_FUNC_OEN_SEL_W {
        GPIO_FUNC_OEN_SEL_W { w: self }
    }
    #[doc = "Bit 11 - 1: Invert the output enable signal; 0: do not invert the output enablesignal. (R/W)"]
    #[inline(always)]
    pub fn gpio_func_oen_inv_sel(&mut self) -> GPIO_FUNC_OEN_INV_SEL_W {
        GPIO_FUNC_OEN_INV_SEL_W { w: self }
    }
}
