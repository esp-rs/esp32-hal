#[doc = "Reader of register GPIO_FUNC_IN_SEL_CFG_REG"]
pub type R = crate::R<u32, super::GPIO_FUNC_IN_SEL_CFG_REG>;
#[doc = "Writer for register GPIO_FUNC_IN_SEL_CFG_REG"]
pub type W = crate::W<u32, super::GPIO_FUNC_IN_SEL_CFG_REG>;
#[doc = "Register GPIO_FUNC_IN_SEL_CFG_REG `reset()`'s with value 0"]
impl crate::ResetValue for super::GPIO_FUNC_IN_SEL_CFG_REG {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `GPIO_FUNC_IN_SEL`"]
pub type GPIO_FUNC_IN_SEL_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `GPIO_FUNC_IN_SEL`"]
pub struct GPIO_FUNC_IN_SEL_W<'a> {
    w: &'a mut W,
}
impl<'a> GPIO_FUNC_IN_SEL_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x3f) | ((value as u32) & 0x3f);
        self.w
    }
}
#[doc = "\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPIO_FUNC_IN_INV_SEL_A {
    #[doc = "0: `0`"]
    LOW,
    #[doc = "1: `1`"]
    HIGH,
}
impl From<GPIO_FUNC_IN_INV_SEL_A> for bool {
    #[inline(always)]
    fn from(variant: GPIO_FUNC_IN_INV_SEL_A) -> Self {
        match variant {
            GPIO_FUNC_IN_INV_SEL_A::LOW => false,
            GPIO_FUNC_IN_INV_SEL_A::HIGH => true,
        }
    }
}
#[doc = "Reader of field `GPIO_FUNC_IN_INV_SEL`"]
pub type GPIO_FUNC_IN_INV_SEL_R = crate::R<bool, GPIO_FUNC_IN_INV_SEL_A>;
impl GPIO_FUNC_IN_INV_SEL_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> GPIO_FUNC_IN_INV_SEL_A {
        match self.bits {
            false => GPIO_FUNC_IN_INV_SEL_A::LOW,
            true => GPIO_FUNC_IN_INV_SEL_A::HIGH,
        }
    }
    #[doc = "Checks if the value of the field is `LOW`"]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == GPIO_FUNC_IN_INV_SEL_A::LOW
    }
    #[doc = "Checks if the value of the field is `HIGH`"]
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == GPIO_FUNC_IN_INV_SEL_A::HIGH
    }
}
#[doc = "Write proxy for field `GPIO_FUNC_IN_INV_SEL`"]
pub struct GPIO_FUNC_IN_INV_SEL_W<'a> {
    w: &'a mut W,
}
impl<'a> GPIO_FUNC_IN_INV_SEL_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: GPIO_FUNC_IN_INV_SEL_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "`0`"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(GPIO_FUNC_IN_INV_SEL_A::LOW)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(GPIO_FUNC_IN_INV_SEL_A::HIGH)
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
        self.w.bits = (self.w.bits & !(0x01 << 6)) | (((value as u32) & 0x01) << 6);
        self.w
    }
}
#[doc = "\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPIO_SIG_IN_SEL_A {
    #[doc = "0: `0`"]
    LOW,
    #[doc = "1: `1`"]
    HIGH,
}
impl From<GPIO_SIG_IN_SEL_A> for bool {
    #[inline(always)]
    fn from(variant: GPIO_SIG_IN_SEL_A) -> Self {
        match variant {
            GPIO_SIG_IN_SEL_A::LOW => false,
            GPIO_SIG_IN_SEL_A::HIGH => true,
        }
    }
}
#[doc = "Reader of field `GPIO_SIG_IN_SEL`"]
pub type GPIO_SIG_IN_SEL_R = crate::R<bool, GPIO_SIG_IN_SEL_A>;
impl GPIO_SIG_IN_SEL_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> GPIO_SIG_IN_SEL_A {
        match self.bits {
            false => GPIO_SIG_IN_SEL_A::LOW,
            true => GPIO_SIG_IN_SEL_A::HIGH,
        }
    }
    #[doc = "Checks if the value of the field is `LOW`"]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == GPIO_SIG_IN_SEL_A::LOW
    }
    #[doc = "Checks if the value of the field is `HIGH`"]
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == GPIO_SIG_IN_SEL_A::HIGH
    }
}
#[doc = "Write proxy for field `GPIO_SIG_IN_SEL`"]
pub struct GPIO_SIG_IN_SEL_W<'a> {
    w: &'a mut W,
}
impl<'a> GPIO_SIG_IN_SEL_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: GPIO_SIG_IN_SEL_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "`0`"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(GPIO_SIG_IN_SEL_A::LOW)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(GPIO_SIG_IN_SEL_A::HIGH)
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
        self.w.bits = (self.w.bits & !(0x01 << 7)) | (((value as u32) & 0x01) << 7);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:5"]
    #[inline(always)]
    pub fn gpio_func_in_sel(&self) -> GPIO_FUNC_IN_SEL_R {
        GPIO_FUNC_IN_SEL_R::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bit 6"]
    #[inline(always)]
    pub fn gpio_func_in_inv_sel(&self) -> GPIO_FUNC_IN_INV_SEL_R {
        GPIO_FUNC_IN_INV_SEL_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7"]
    #[inline(always)]
    pub fn gpio_sig_in_sel(&self) -> GPIO_SIG_IN_SEL_R {
        GPIO_SIG_IN_SEL_R::new(((self.bits >> 7) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:5"]
    #[inline(always)]
    pub fn gpio_func_in_sel(&mut self) -> GPIO_FUNC_IN_SEL_W {
        GPIO_FUNC_IN_SEL_W { w: self }
    }
    #[doc = "Bit 6"]
    #[inline(always)]
    pub fn gpio_func_in_inv_sel(&mut self) -> GPIO_FUNC_IN_INV_SEL_W {
        GPIO_FUNC_IN_INV_SEL_W { w: self }
    }
    #[doc = "Bit 7"]
    #[inline(always)]
    pub fn gpio_sig_in_sel(&mut self) -> GPIO_SIG_IN_SEL_W {
        GPIO_SIG_IN_SEL_W { w: self }
    }
}
