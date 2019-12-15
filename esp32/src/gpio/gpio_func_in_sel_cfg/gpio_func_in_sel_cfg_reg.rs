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
#[doc = "Invert the input value. 1: invert; 0: do not invert.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPIO_FUNC_IN_INV_SEL_A {
    #[doc = "0: No invert."]
    NO_INVERT,
    #[doc = "1: Invert."]
    INVERT,
}
impl From<GPIO_FUNC_IN_INV_SEL_A> for bool {
    #[inline(always)]
    fn from(variant: GPIO_FUNC_IN_INV_SEL_A) -> Self {
        match variant {
            GPIO_FUNC_IN_INV_SEL_A::NO_INVERT => false,
            GPIO_FUNC_IN_INV_SEL_A::INVERT => true,
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
            false => GPIO_FUNC_IN_INV_SEL_A::NO_INVERT,
            true => GPIO_FUNC_IN_INV_SEL_A::INVERT,
        }
    }
    #[doc = "Checks if the value of the field is `NO_INVERT`"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == GPIO_FUNC_IN_INV_SEL_A::NO_INVERT
    }
    #[doc = "Checks if the value of the field is `INVERT`"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == GPIO_FUNC_IN_INV_SEL_A::INVERT
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
    #[doc = "No invert."]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut W {
        self.variant(GPIO_FUNC_IN_INV_SEL_A::NO_INVERT)
    }
    #[doc = "Invert."]
    #[inline(always)]
    pub fn invert(self) -> &'a mut W {
        self.variant(GPIO_FUNC_IN_INV_SEL_A::INVERT)
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
#[doc = "Bypass the GPIO Matrix. 1: route through GPIO Matrix, 0: connect signaldirectly to peripheral configured in the IO_MUX.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPIO_SIG_IN_SEL_A {
    #[doc = "0: Connect signal directly to peripheral configured in the IO_MUX"]
    CONNECT,
    #[doc = "1: Route through GPIO Matrix"]
    ROUTE,
}
impl From<GPIO_SIG_IN_SEL_A> for bool {
    #[inline(always)]
    fn from(variant: GPIO_SIG_IN_SEL_A) -> Self {
        match variant {
            GPIO_SIG_IN_SEL_A::CONNECT => false,
            GPIO_SIG_IN_SEL_A::ROUTE => true,
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
            false => GPIO_SIG_IN_SEL_A::CONNECT,
            true => GPIO_SIG_IN_SEL_A::ROUTE,
        }
    }
    #[doc = "Checks if the value of the field is `CONNECT`"]
    #[inline(always)]
    pub fn is_connect(&self) -> bool {
        *self == GPIO_SIG_IN_SEL_A::CONNECT
    }
    #[doc = "Checks if the value of the field is `ROUTE`"]
    #[inline(always)]
    pub fn is_route(&self) -> bool {
        *self == GPIO_SIG_IN_SEL_A::ROUTE
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
    #[doc = "Connect signal directly to peripheral configured in the IO_MUX"]
    #[inline(always)]
    pub fn connect(self) -> &'a mut W {
        self.variant(GPIO_SIG_IN_SEL_A::CONNECT)
    }
    #[doc = "Route through GPIO Matrix"]
    #[inline(always)]
    pub fn route(self) -> &'a mut W {
        self.variant(GPIO_SIG_IN_SEL_A::ROUTE)
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
    #[doc = "Bits 0:5 - Selection control for peripheral input. A value of 0-39 selects which of the40 GPIO Matrix input pins this signal is connected to, or 0x38 for a constantly high input or 0x30 for a constantly low input."]
    #[inline(always)]
    pub fn gpio_func_in_sel(&self) -> GPIO_FUNC_IN_SEL_R {
        GPIO_FUNC_IN_SEL_R::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bit 6 - Invert the input value. 1: invert; 0: do not invert."]
    #[inline(always)]
    pub fn gpio_func_in_inv_sel(&self) -> GPIO_FUNC_IN_INV_SEL_R {
        GPIO_FUNC_IN_INV_SEL_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Bypass the GPIO Matrix. 1: route through GPIO Matrix, 0: connect signaldirectly to peripheral configured in the IO_MUX."]
    #[inline(always)]
    pub fn gpio_sig_in_sel(&self) -> GPIO_SIG_IN_SEL_R {
        GPIO_SIG_IN_SEL_R::new(((self.bits >> 7) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:5 - Selection control for peripheral input. A value of 0-39 selects which of the40 GPIO Matrix input pins this signal is connected to, or 0x38 for a constantly high input or 0x30 for a constantly low input."]
    #[inline(always)]
    pub fn gpio_func_in_sel(&mut self) -> GPIO_FUNC_IN_SEL_W {
        GPIO_FUNC_IN_SEL_W { w: self }
    }
    #[doc = "Bit 6 - Invert the input value. 1: invert; 0: do not invert."]
    #[inline(always)]
    pub fn gpio_func_in_inv_sel(&mut self) -> GPIO_FUNC_IN_INV_SEL_W {
        GPIO_FUNC_IN_INV_SEL_W { w: self }
    }
    #[doc = "Bit 7 - Bypass the GPIO Matrix. 1: route through GPIO Matrix, 0: connect signaldirectly to peripheral configured in the IO_MUX."]
    #[inline(always)]
    pub fn gpio_sig_in_sel(&mut self) -> GPIO_SIG_IN_SEL_W {
        GPIO_SIG_IN_SEL_W { w: self }
    }
}
