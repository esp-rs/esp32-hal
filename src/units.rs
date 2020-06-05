//! Units of measurement implementation for times and frequencies.
//!
//! It provides type safety, easy conversion and limited arithmetic support.
//!
//! # Usage
//!
//! ```
//! let frequency_mhz_1 = MegaHertz(10);
//! let frequency_mhz_2 = 10.MHz();
//! let frequency_khz_1: KiloHertz = frequency_mhz_1.into();
//! let frequency_khz_2 = KiloHertz::from(frequency_mhz_2);
//! let frequency_khz_3 = frequency_khz_1 + 10.MHz().into();
//! let frequency_hz_1 = 1.Hz() + frequency_khz_3.into();
//! ```

use core::fmt;

pub trait Quantity: Sized {}
pub trait Time: Quantity + Into<NanoSecondsU64> {}
pub trait Frequency: Quantity + Into<HertzU64> {}
pub trait Count: Quantity + Into<TicksU64> {}

pub type ValueType = u32;
pub type LargeValueType = u64;

/// defines and implements extension traits for quantities with units
macro_rules! define {
    ($primitive:ident, $trait:ident, $( ($type: ident, $quantity:ident, $unit:ident), )+) => {
        $(
            #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default)]
            pub struct $quantity(pub $primitive);

            impl Quantity for $quantity {}
            impl $type for $quantity {}
        )*


        pub trait $trait {
            $(
                #[allow(non_snake_case)]
                fn $unit(self) -> $quantity;
            )*
        }

        impl $trait for $primitive {
            $(
                fn $unit(self) -> $quantity {
                    $quantity(self)
                }
            )*
        }

        $(
            impl From<$quantity> for $primitive {
                fn from(x: $quantity) -> Self {
                    x.0
                }
            }

            impl From<$primitive> for $quantity {
                fn from(x: $primitive) -> $quantity {
                    $quantity(x)
                }
            }

            impl fmt::Debug for $quantity {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{}{}", self.0, stringify!($unit))
                }
            }

            impl fmt::Display for $quantity {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{}{}", self.0, stringify!($unit))
                }
            }

            impl core::ops::Div<$primitive> for $quantity {
                type Output = Self;
                fn div(self, rhs: $primitive) -> Self::Output {
                    $quantity(self.0/rhs)
                }
            }

            impl core::ops::Mul<$primitive> for $quantity {
                type Output = Self;
                fn mul(self, rhs: $primitive) -> Self::Output {
                    $quantity(self.0*rhs)
                }
            }

            impl core::ops::Mul<$quantity> for $primitive {
                type Output = $quantity;
                fn mul(self, rhs: $quantity) -> Self::Output {
                    $quantity(self*rhs.0)
                }
            }

            impl core::ops::Div<$quantity> for $quantity {
                type Output = $primitive;
                fn div(self, rhs: Self) -> Self::Output {
                    self.0/rhs.0
                }
            }

            impl core::ops::Add<$quantity> for $quantity {
                type Output = Self;
                fn add(self, rhs: Self) -> Self::Output {
                    Self(self.0+rhs.0)
                }
            }

            impl core::ops::Sub<$quantity> for $quantity {
                type Output = Self;
                fn sub(self, rhs: Self) -> Self::Output {
                    Self(self.0-rhs.0)
                }
            }
        )*
    };
}

/// Define ValueType and LargeValueType quantities and conversion from ValueType to LargeValueType
macro_rules! define_u64 {
    ($( ($type: ident, $quantity: ident, $unit:ident,
        $quantity_u64:ident, $unit_u64:ident) ),+) => {
        define!(
            ValueType,
            FromValueType,
            $(($type, $quantity, $unit),)*
        );

        define!(
            LargeValueType,
            FromLargeValueType,
            $(($type, $quantity_u64, $unit_u64),)*
        );

        $(
        impl From<$quantity> for $quantity_u64 {
            fn from(x: $quantity) -> Self {
                Self(x.0 as LargeValueType)
            }
        })*

    };
}

/// defines From trait for pair or quantities with scaling
macro_rules! convert {
    ($from: ty, $from_u64: ty, $into: ty, $into_u64: ty, $factor: expr) => {
        impl From<$from> for $into {
            fn from(x: $from) -> Self {
                Self(x.0 * $factor)
            }
        }
        impl From<$from> for $into_u64 {
            fn from(x: $from) -> Self {
                Self(x.0 as u64 * $factor)
            }
        }
        impl From<$from_u64> for $into_u64 {
            fn from(x: $from_u64) -> Self {
                Self(x.0 * $factor)
            }
        }
    };
}

/// defines multiply trait for frequency and time
macro_rules! multiply {
    ($time: ty, $time_u64: ty, $freq: ty, $freq_u64: ty,
        $factor: expr, $divider: expr) => {
        impl core::ops::Mul<$freq> for $time {
            type Output = Ticks;
            fn mul(self, rhs: $freq) -> Self::Output {
                Ticks(self.0 * rhs.0 * $factor / $divider)
            }
        }

        impl core::ops::Mul<$time> for $freq {
            type Output = Ticks;
            fn mul(self, rhs: $time) -> Self::Output {
                Ticks(self.0 * rhs.0 * $factor / $divider)
            }
        }

        impl core::ops::Mul<$freq_u64> for $time_u64 {
            type Output = TicksU64;
            fn mul(self, rhs: $freq_u64) -> Self::Output {
                TicksU64(self.0 * rhs.0 * $factor / $divider)
            }
        }

        impl core::ops::Mul<$time_u64> for $freq_u64 {
            type Output = TicksU64;
            fn mul(self, rhs: $time_u64) -> Self::Output {
                TicksU64(self.0 * rhs.0 * $factor / $divider)
            }
        }

        impl core::ops::Mul<$freq> for $time_u64 {
            type Output = TicksU64;
            fn mul(self, rhs: $freq) -> Self::Output {
                TicksU64(self.0 * rhs.0 as u64 * $factor / $divider)
            }
        }

        impl core::ops::Mul<$time> for $freq_u64 {
            type Output = TicksU64;
            fn mul(self, rhs: $time) -> Self::Output {
                TicksU64(self.0 * rhs.0 as u64 * $factor / $divider)
            }
        }

        impl core::ops::Mul<$freq_u64> for $time {
            type Output = TicksU64;
            fn mul(self, rhs: $freq_u64) -> Self::Output {
                TicksU64(self.0 as u64 * rhs.0 * $factor / $divider)
            }
        }

        impl core::ops::Mul<$time_u64> for $freq {
            type Output = TicksU64;
            fn mul(self, rhs: $time_u64) -> Self::Output {
                TicksU64(self.0 as u64 * rhs.0 * $factor / $divider)
            }
        }
    };
}

macro_rules! divide {
    ($time: ty, $time_u64: ty, $freq: ty, $freq_u64: ty,
        $factor: expr, $divider: expr) => {
        impl core::ops::Div<$freq> for Ticks {
            type Output = $time;
            fn div(self, rhs: $freq) -> Self::Output {
                (self.0 * $divider / rhs.0 / $factor).into()
            }
        }
    };
}

divide!(Seconds, SecondsU64, Hertz, HertzU64, 1, 1);
divide!(Seconds, SecondsU64, KiloHertz, KiloHertzU64, 1_000, 1);
divide!(Seconds, SecondsU64, MegaHertz, MegaHertzU64, 1_000_000, 1);

define_u64!(
    (Frequency, Hertz, Hz, HertzU64, Hz_u64),
    (Frequency, KiloHertz, kHz, KiloHertzU64, kHz_u64),
    (Frequency, MegaHertz, MHz, MegaHertzU64, MHz_u64),
    (Time, NanoSeconds, ns, NanoSecondsU64, ns_u64),
    (Time, MicroSeconds, us, MicroSecondsU64, us_u64),
    (Time, MilliSeconds, ms, MilliSecondsU64, ms_u64),
    (Time, Seconds, s, SecondsU64, s_u64),
    (Count, Ticks, ticks, TicksU64, ticks_u64)
);

convert!(KiloHertz, KiloHertzU64, Hertz, HertzU64, 1000);

convert!(MegaHertz, MegaHertzU64, Hertz, HertzU64, 1000000);
convert!(MegaHertz, MegaHertzU64, KiloHertz, KiloHertzU64, 1000);

convert!(Seconds, SecondsU64, MilliSeconds, MilliSecondsU64, 1000);
convert!(Seconds, SecondsU64, MicroSeconds, MicroSecondsU64, 1000000);
convert!(Seconds, SecondsU64, NanoSeconds, NanoSecondsU64, 1000000000);

convert!(
    MilliSeconds,
    MilliSecondsU64,
    MicroSeconds,
    MicroSecondsU64,
    1000
);
convert!(
    MilliSeconds,
    MilliSecondsU64,
    NanoSeconds,
    NanoSecondsU64,
    1000000
);

convert!(
    MicroSeconds,
    MicroSecondsU64,
    NanoSeconds,
    NanoSecondsU64,
    1000
);

multiply!(Seconds, SecondsU64, Hertz, HertzU64, 1, 1);
multiply!(Seconds, SecondsU64, KiloHertz, KiloHertzU64, 1_000, 1);
multiply!(Seconds, SecondsU64, MegaHertz, MegaHertzU64, 1_000_000, 1);

multiply!(MilliSeconds, MilliSecondsU64, Hertz, HertzU64, 1, 1_000);
multiply!(MilliSeconds, MilliSecondsU64, KiloHertz, KiloHertzU64, 1, 1);
multiply!(
    MilliSeconds,
    MilliSecondsU64,
    MegaHertz,
    MegaHertzU64,
    1_000,
    1
);

multiply!(MicroSeconds, MicroSecondsU64, Hertz, HertzU64, 1, 1_000_000);
multiply!(
    MicroSeconds,
    MicroSecondsU64,
    KiloHertz,
    KiloHertzU64,
    1,
    1_000
);
multiply!(MicroSeconds, MicroSecondsU64, MegaHertz, MegaHertzU64, 1, 1);

multiply!(
    NanoSeconds,
    NanoSecondsU64,
    Hertz,
    HertzU64,
    1,
    1_000_000_000
);
multiply!(
    NanoSeconds,
    NanoSecondsU64,
    KiloHertz,
    KiloHertzU64,
    1,
    1_000_000
);
multiply!(
    NanoSeconds,
    NanoSecondsU64,
    MegaHertz,
    MegaHertzU64,
    1,
    1_000
);
