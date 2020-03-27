//! Units of measurement implementation for times and frequencies.
//!
//! It provides type safety, easy conversion and limited arithmetic support.
//!
//! #Usage
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

/// defines and implements extension traits for quantities with units
macro_rules! define {
    ($primitive:ident, $( $quantity:ident, $unit:ident),+) => {
        $(
            #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default)]
            pub struct $quantity(pub $primitive);
        )*

        #[allow(non_snake_case)]
        pub trait Quantity {
            $(
                fn $unit(self) -> $quantity;
            )*
        }

        impl Quantity for $primitive {
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


            impl core::ops::Div for $quantity {
                type Output = $primitive;
                fn div(self, rhs: Self) -> Self::Output {
                    self.0/rhs.0
                }
            }

            impl core::ops::Add for $quantity {
                type Output = Self;
                fn add(self, rhs: Self) -> Self::Output {
                    Self(self.0+rhs.0)
                }
            }

            impl core::ops::Sub for $quantity {
                type Output = Self;
                fn sub(self, rhs: Self) -> Self::Output {
                    Self(self.0-rhs.0)
                }
            }
        )*
    };
}

/// defines From trait for pair or quantities with scaling
macro_rules! convert {
    ($from: ty, $into: ident, $factor: expr) => {
        impl From<$from> for $into {
            fn from(x: $from) -> Self {
                $into(x.0 * $factor)
            }
        }
    };
}

define!(
    u32,
    Hertz,
    Hz,
    KiloHertz,
    kHz,
    MegaHertz,
    MHz,
    NanoSeconds,
    ns,
    MicroSeconds,
    us,
    MilliSeconds,
    ms,
    Seconds,
    s
);

convert!(KiloHertz, Hertz, 1_000);
convert!(MegaHertz, Hertz, 1_000_000);

convert!(MegaHertz, KiloHertz, 1_000);

convert!(Seconds, MilliSeconds, 1_000);
convert!(Seconds, MicroSeconds, 1_000_000);
convert!(Seconds, NanoSeconds, 1_000_000_000);

convert!(MilliSeconds, MicroSeconds, 1_000);
convert!(MilliSeconds, NanoSeconds, 1_000_000);

convert!(MicroSeconds, NanoSeconds, 1_000);
