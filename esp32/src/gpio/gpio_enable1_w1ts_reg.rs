#[doc = "Writer for register GPIO_ENABLE1_W1TS_REG"]
pub type W = crate::W<u32, super::GPIO_ENABLE1_W1TS_REG>;
#[doc = "Register GPIO_ENABLE1_W1TS_REG `reset()`'s with value 0"]
impl crate::ResetValue for super::GPIO_ENABLE1_W1TS_REG {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN32_AW {
    #[doc = "0: `0`"]
    LOW,
    #[doc = "1: `1`"]
    HIGH,
}
impl From<PIN32_AW> for bool {
    #[inline(always)]
    fn from(variant: PIN32_AW) -> Self {
        match variant {
            PIN32_AW::LOW => false,
            PIN32_AW::HIGH => true,
        }
    }
}
#[doc = "Write proxy for field `PIN32`"]
pub struct PIN32_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN32_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN32_AW) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "`0`"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(PIN32_AW::LOW)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(PIN32_AW::HIGH)
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u32) & 0x01);
        self.w
    }
}
#[doc = "\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN33_AW {
    #[doc = "0: `0`"]
    LOW,
    #[doc = "1: `1`"]
    HIGH,
}
impl From<PIN33_AW> for bool {
    #[inline(always)]
    fn from(variant: PIN33_AW) -> Self {
        match variant {
            PIN33_AW::LOW => false,
            PIN33_AW::HIGH => true,
        }
    }
}
#[doc = "Write proxy for field `PIN33`"]
pub struct PIN33_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN33_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN33_AW) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "`0`"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(PIN33_AW::LOW)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(PIN33_AW::HIGH)
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u32) & 0x01) << 1);
        self.w
    }
}
#[doc = "\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN34_AW {
    #[doc = "0: `0`"]
    LOW,
    #[doc = "1: `1`"]
    HIGH,
}
impl From<PIN34_AW> for bool {
    #[inline(always)]
    fn from(variant: PIN34_AW) -> Self {
        match variant {
            PIN34_AW::LOW => false,
            PIN34_AW::HIGH => true,
        }
    }
}
#[doc = "Write proxy for field `PIN34`"]
pub struct PIN34_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN34_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN34_AW) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "`0`"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(PIN34_AW::LOW)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(PIN34_AW::HIGH)
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u32) & 0x01) << 2);
        self.w
    }
}
#[doc = "\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN35_AW {
    #[doc = "0: `0`"]
    LOW,
    #[doc = "1: `1`"]
    HIGH,
}
impl From<PIN35_AW> for bool {
    #[inline(always)]
    fn from(variant: PIN35_AW) -> Self {
        match variant {
            PIN35_AW::LOW => false,
            PIN35_AW::HIGH => true,
        }
    }
}
#[doc = "Write proxy for field `PIN35`"]
pub struct PIN35_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN35_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN35_AW) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "`0`"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(PIN35_AW::LOW)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(PIN35_AW::HIGH)
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u32) & 0x01) << 3);
        self.w
    }
}
#[doc = "\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN36_AW {
    #[doc = "0: `0`"]
    LOW,
    #[doc = "1: `1`"]
    HIGH,
}
impl From<PIN36_AW> for bool {
    #[inline(always)]
    fn from(variant: PIN36_AW) -> Self {
        match variant {
            PIN36_AW::LOW => false,
            PIN36_AW::HIGH => true,
        }
    }
}
#[doc = "Write proxy for field `PIN36`"]
pub struct PIN36_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN36_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN36_AW) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "`0`"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(PIN36_AW::LOW)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(PIN36_AW::HIGH)
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u32) & 0x01) << 4);
        self.w
    }
}
#[doc = "\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN37_AW {
    #[doc = "0: Disables pin output"]
    LOW,
    #[doc = "1: Enables pin output"]
    HIGH,
}
impl From<PIN37_AW> for bool {
    #[inline(always)]
    fn from(variant: PIN37_AW) -> Self {
        match variant {
            PIN37_AW::LOW => false,
            PIN37_AW::HIGH => true,
        }
    }
}
#[doc = "Write proxy for field `PIN37`"]
pub struct PIN37_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN37_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN37_AW) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Disables pin output"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(PIN37_AW::LOW)
    }
    #[doc = "Enables pin output"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(PIN37_AW::HIGH)
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u32) & 0x01) << 5);
        self.w
    }
}
#[doc = "\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN38_AW {
    #[doc = "0: `0`"]
    LOW,
    #[doc = "1: `1`"]
    HIGH,
}
impl From<PIN38_AW> for bool {
    #[inline(always)]
    fn from(variant: PIN38_AW) -> Self {
        match variant {
            PIN38_AW::LOW => false,
            PIN38_AW::HIGH => true,
        }
    }
}
#[doc = "Write proxy for field `PIN38`"]
pub struct PIN38_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN38_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN38_AW) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "`0`"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(PIN38_AW::LOW)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(PIN38_AW::HIGH)
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
pub enum PIN39_AW {
    #[doc = "0: `0`"]
    LOW,
    #[doc = "1: `1`"]
    HIGH,
}
impl From<PIN39_AW> for bool {
    #[inline(always)]
    fn from(variant: PIN39_AW) -> Self {
        match variant {
            PIN39_AW::LOW => false,
            PIN39_AW::HIGH => true,
        }
    }
}
#[doc = "Write proxy for field `PIN39`"]
pub struct PIN39_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN39_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN39_AW) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "`0`"]
    #[inline(always)]
    pub fn low(self) -> &'a mut W {
        self.variant(PIN39_AW::LOW)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn high(self) -> &'a mut W {
        self.variant(PIN39_AW::HIGH)
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
impl W {
    #[doc = "Bit 0"]
    #[inline(always)]
    pub fn pin32(&mut self) -> PIN32_W {
        PIN32_W { w: self }
    }
    #[doc = "Bit 1"]
    #[inline(always)]
    pub fn pin33(&mut self) -> PIN33_W {
        PIN33_W { w: self }
    }
    #[doc = "Bit 2"]
    #[inline(always)]
    pub fn pin34(&mut self) -> PIN34_W {
        PIN34_W { w: self }
    }
    #[doc = "Bit 3"]
    #[inline(always)]
    pub fn pin35(&mut self) -> PIN35_W {
        PIN35_W { w: self }
    }
    #[doc = "Bit 4"]
    #[inline(always)]
    pub fn pin36(&mut self) -> PIN36_W {
        PIN36_W { w: self }
    }
    #[doc = "Bit 5"]
    #[inline(always)]
    pub fn pin37(&mut self) -> PIN37_W {
        PIN37_W { w: self }
    }
    #[doc = "Bit 6"]
    #[inline(always)]
    pub fn pin38(&mut self) -> PIN38_W {
        PIN38_W { w: self }
    }
    #[doc = "Bit 7"]
    #[inline(always)]
    pub fn pin39(&mut self) -> PIN39_W {
        PIN39_W { w: self }
    }
}
