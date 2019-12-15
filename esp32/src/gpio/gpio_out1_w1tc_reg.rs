#[doc = "Reader of register GPIO_OUT1_W1TC_REG"]
pub type R = crate::R<u32, super::GPIO_OUT1_W1TC_REG>;
#[doc = "Writer for register GPIO_OUT1_W1TC_REG"]
pub type W = crate::W<u32, super::GPIO_OUT1_W1TC_REG>;
#[doc = "Register GPIO_OUT1_W1TC_REG `reset()`'s with value 0"]
impl crate::ResetValue for super::GPIO_OUT1_W1TC_REG {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Output set value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN32_A {
    #[doc = "1: Set pin output"]
    SET,
    #[doc = "0: Unset pin output"]
    UNSET,
}
impl From<PIN32_A> for bool {
    #[inline(always)]
    fn from(variant: PIN32_A) -> Self {
        match variant {
            PIN32_A::SET => true,
            PIN32_A::UNSET => false,
        }
    }
}
#[doc = "Reader of field `PIN32`"]
pub type PIN32_R = crate::R<bool, PIN32_A>;
impl PIN32_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN32_A {
        match self.bits {
            true => PIN32_A::SET,
            false => PIN32_A::UNSET,
        }
    }
    #[doc = "Checks if the value of the field is `SET`"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == PIN32_A::SET
    }
    #[doc = "Checks if the value of the field is `UNSET`"]
    #[inline(always)]
    pub fn is_unset(&self) -> bool {
        *self == PIN32_A::UNSET
    }
}
#[doc = "Write proxy for field `PIN32`"]
pub struct PIN32_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN32_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN32_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Set pin output"]
    #[inline(always)]
    pub fn set(self) -> &'a mut W {
        self.variant(PIN32_A::SET)
    }
    #[doc = "Unset pin output"]
    #[inline(always)]
    pub fn unset(self) -> &'a mut W {
        self.variant(PIN32_A::UNSET)
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
#[doc = "Output set value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN33_A {
    #[doc = "1: Set pin output"]
    SET,
    #[doc = "0: Unset pin output"]
    UNSET,
}
impl From<PIN33_A> for bool {
    #[inline(always)]
    fn from(variant: PIN33_A) -> Self {
        match variant {
            PIN33_A::SET => true,
            PIN33_A::UNSET => false,
        }
    }
}
#[doc = "Reader of field `PIN33`"]
pub type PIN33_R = crate::R<bool, PIN33_A>;
impl PIN33_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN33_A {
        match self.bits {
            true => PIN33_A::SET,
            false => PIN33_A::UNSET,
        }
    }
    #[doc = "Checks if the value of the field is `SET`"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == PIN33_A::SET
    }
    #[doc = "Checks if the value of the field is `UNSET`"]
    #[inline(always)]
    pub fn is_unset(&self) -> bool {
        *self == PIN33_A::UNSET
    }
}
#[doc = "Write proxy for field `PIN33`"]
pub struct PIN33_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN33_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN33_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Set pin output"]
    #[inline(always)]
    pub fn set(self) -> &'a mut W {
        self.variant(PIN33_A::SET)
    }
    #[doc = "Unset pin output"]
    #[inline(always)]
    pub fn unset(self) -> &'a mut W {
        self.variant(PIN33_A::UNSET)
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
#[doc = "Output set value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN34_A {
    #[doc = "1: Set pin output"]
    SET,
    #[doc = "0: Unset pin output"]
    UNSET,
}
impl From<PIN34_A> for bool {
    #[inline(always)]
    fn from(variant: PIN34_A) -> Self {
        match variant {
            PIN34_A::SET => true,
            PIN34_A::UNSET => false,
        }
    }
}
#[doc = "Reader of field `PIN34`"]
pub type PIN34_R = crate::R<bool, PIN34_A>;
impl PIN34_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN34_A {
        match self.bits {
            true => PIN34_A::SET,
            false => PIN34_A::UNSET,
        }
    }
    #[doc = "Checks if the value of the field is `SET`"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == PIN34_A::SET
    }
    #[doc = "Checks if the value of the field is `UNSET`"]
    #[inline(always)]
    pub fn is_unset(&self) -> bool {
        *self == PIN34_A::UNSET
    }
}
#[doc = "Write proxy for field `PIN34`"]
pub struct PIN34_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN34_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN34_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Set pin output"]
    #[inline(always)]
    pub fn set(self) -> &'a mut W {
        self.variant(PIN34_A::SET)
    }
    #[doc = "Unset pin output"]
    #[inline(always)]
    pub fn unset(self) -> &'a mut W {
        self.variant(PIN34_A::UNSET)
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
#[doc = "Output set value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN35_A {
    #[doc = "1: Set pin output"]
    SET,
    #[doc = "0: Unset pin output"]
    UNSET,
}
impl From<PIN35_A> for bool {
    #[inline(always)]
    fn from(variant: PIN35_A) -> Self {
        match variant {
            PIN35_A::SET => true,
            PIN35_A::UNSET => false,
        }
    }
}
#[doc = "Reader of field `PIN35`"]
pub type PIN35_R = crate::R<bool, PIN35_A>;
impl PIN35_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN35_A {
        match self.bits {
            true => PIN35_A::SET,
            false => PIN35_A::UNSET,
        }
    }
    #[doc = "Checks if the value of the field is `SET`"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == PIN35_A::SET
    }
    #[doc = "Checks if the value of the field is `UNSET`"]
    #[inline(always)]
    pub fn is_unset(&self) -> bool {
        *self == PIN35_A::UNSET
    }
}
#[doc = "Write proxy for field `PIN35`"]
pub struct PIN35_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN35_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN35_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Set pin output"]
    #[inline(always)]
    pub fn set(self) -> &'a mut W {
        self.variant(PIN35_A::SET)
    }
    #[doc = "Unset pin output"]
    #[inline(always)]
    pub fn unset(self) -> &'a mut W {
        self.variant(PIN35_A::UNSET)
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
#[doc = "Output set value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN36_A {
    #[doc = "1: Set pin output"]
    SET,
    #[doc = "0: Unset pin output"]
    UNSET,
}
impl From<PIN36_A> for bool {
    #[inline(always)]
    fn from(variant: PIN36_A) -> Self {
        match variant {
            PIN36_A::SET => true,
            PIN36_A::UNSET => false,
        }
    }
}
#[doc = "Reader of field `PIN36`"]
pub type PIN36_R = crate::R<bool, PIN36_A>;
impl PIN36_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN36_A {
        match self.bits {
            true => PIN36_A::SET,
            false => PIN36_A::UNSET,
        }
    }
    #[doc = "Checks if the value of the field is `SET`"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == PIN36_A::SET
    }
    #[doc = "Checks if the value of the field is `UNSET`"]
    #[inline(always)]
    pub fn is_unset(&self) -> bool {
        *self == PIN36_A::UNSET
    }
}
#[doc = "Write proxy for field `PIN36`"]
pub struct PIN36_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN36_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN36_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Set pin output"]
    #[inline(always)]
    pub fn set(self) -> &'a mut W {
        self.variant(PIN36_A::SET)
    }
    #[doc = "Unset pin output"]
    #[inline(always)]
    pub fn unset(self) -> &'a mut W {
        self.variant(PIN36_A::UNSET)
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
#[doc = "Output set value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN37_A {
    #[doc = "1: Set pin output"]
    SET,
    #[doc = "0: Unset pin output"]
    UNSET,
}
impl From<PIN37_A> for bool {
    #[inline(always)]
    fn from(variant: PIN37_A) -> Self {
        match variant {
            PIN37_A::SET => true,
            PIN37_A::UNSET => false,
        }
    }
}
#[doc = "Reader of field `PIN37`"]
pub type PIN37_R = crate::R<bool, PIN37_A>;
impl PIN37_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN37_A {
        match self.bits {
            true => PIN37_A::SET,
            false => PIN37_A::UNSET,
        }
    }
    #[doc = "Checks if the value of the field is `SET`"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == PIN37_A::SET
    }
    #[doc = "Checks if the value of the field is `UNSET`"]
    #[inline(always)]
    pub fn is_unset(&self) -> bool {
        *self == PIN37_A::UNSET
    }
}
#[doc = "Write proxy for field `PIN37`"]
pub struct PIN37_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN37_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN37_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Set pin output"]
    #[inline(always)]
    pub fn set(self) -> &'a mut W {
        self.variant(PIN37_A::SET)
    }
    #[doc = "Unset pin output"]
    #[inline(always)]
    pub fn unset(self) -> &'a mut W {
        self.variant(PIN37_A::UNSET)
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
#[doc = "Output set value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN38_A {
    #[doc = "1: Set pin output"]
    SET,
    #[doc = "0: Unset pin output"]
    UNSET,
}
impl From<PIN38_A> for bool {
    #[inline(always)]
    fn from(variant: PIN38_A) -> Self {
        match variant {
            PIN38_A::SET => true,
            PIN38_A::UNSET => false,
        }
    }
}
#[doc = "Reader of field `PIN38`"]
pub type PIN38_R = crate::R<bool, PIN38_A>;
impl PIN38_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN38_A {
        match self.bits {
            true => PIN38_A::SET,
            false => PIN38_A::UNSET,
        }
    }
    #[doc = "Checks if the value of the field is `SET`"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == PIN38_A::SET
    }
    #[doc = "Checks if the value of the field is `UNSET`"]
    #[inline(always)]
    pub fn is_unset(&self) -> bool {
        *self == PIN38_A::UNSET
    }
}
#[doc = "Write proxy for field `PIN38`"]
pub struct PIN38_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN38_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN38_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Set pin output"]
    #[inline(always)]
    pub fn set(self) -> &'a mut W {
        self.variant(PIN38_A::SET)
    }
    #[doc = "Unset pin output"]
    #[inline(always)]
    pub fn unset(self) -> &'a mut W {
        self.variant(PIN38_A::UNSET)
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
#[doc = "Output set value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN39_A {
    #[doc = "1: Set pin output"]
    SET,
    #[doc = "0: Unset pin output"]
    UNSET,
}
impl From<PIN39_A> for bool {
    #[inline(always)]
    fn from(variant: PIN39_A) -> Self {
        match variant {
            PIN39_A::SET => true,
            PIN39_A::UNSET => false,
        }
    }
}
#[doc = "Reader of field `PIN39`"]
pub type PIN39_R = crate::R<bool, PIN39_A>;
impl PIN39_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN39_A {
        match self.bits {
            true => PIN39_A::SET,
            false => PIN39_A::UNSET,
        }
    }
    #[doc = "Checks if the value of the field is `SET`"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == PIN39_A::SET
    }
    #[doc = "Checks if the value of the field is `UNSET`"]
    #[inline(always)]
    pub fn is_unset(&self) -> bool {
        *self == PIN39_A::UNSET
    }
}
#[doc = "Write proxy for field `PIN39`"]
pub struct PIN39_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN39_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN39_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Set pin output"]
    #[inline(always)]
    pub fn set(self) -> &'a mut W {
        self.variant(PIN39_A::SET)
    }
    #[doc = "Unset pin output"]
    #[inline(always)]
    pub fn unset(self) -> &'a mut W {
        self.variant(PIN39_A::UNSET)
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
    #[doc = "Bit 0 - Output set value"]
    #[inline(always)]
    pub fn pin32(&self) -> PIN32_R {
        PIN32_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Output set value"]
    #[inline(always)]
    pub fn pin33(&self) -> PIN33_R {
        PIN33_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Output set value"]
    #[inline(always)]
    pub fn pin34(&self) -> PIN34_R {
        PIN34_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Output set value"]
    #[inline(always)]
    pub fn pin35(&self) -> PIN35_R {
        PIN35_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Output set value"]
    #[inline(always)]
    pub fn pin36(&self) -> PIN36_R {
        PIN36_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Output set value"]
    #[inline(always)]
    pub fn pin37(&self) -> PIN37_R {
        PIN37_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Output set value"]
    #[inline(always)]
    pub fn pin38(&self) -> PIN38_R {
        PIN38_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Output set value"]
    #[inline(always)]
    pub fn pin39(&self) -> PIN39_R {
        PIN39_R::new(((self.bits >> 7) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Output set value"]
    #[inline(always)]
    pub fn pin32(&mut self) -> PIN32_W {
        PIN32_W { w: self }
    }
    #[doc = "Bit 1 - Output set value"]
    #[inline(always)]
    pub fn pin33(&mut self) -> PIN33_W {
        PIN33_W { w: self }
    }
    #[doc = "Bit 2 - Output set value"]
    #[inline(always)]
    pub fn pin34(&mut self) -> PIN34_W {
        PIN34_W { w: self }
    }
    #[doc = "Bit 3 - Output set value"]
    #[inline(always)]
    pub fn pin35(&mut self) -> PIN35_W {
        PIN35_W { w: self }
    }
    #[doc = "Bit 4 - Output set value"]
    #[inline(always)]
    pub fn pin36(&mut self) -> PIN36_W {
        PIN36_W { w: self }
    }
    #[doc = "Bit 5 - Output set value"]
    #[inline(always)]
    pub fn pin37(&mut self) -> PIN37_W {
        PIN37_W { w: self }
    }
    #[doc = "Bit 6 - Output set value"]
    #[inline(always)]
    pub fn pin38(&mut self) -> PIN38_W {
        PIN38_W { w: self }
    }
    #[doc = "Bit 7 - Output set value"]
    #[inline(always)]
    pub fn pin39(&mut self) -> PIN39_W {
        PIN39_W { w: self }
    }
}
