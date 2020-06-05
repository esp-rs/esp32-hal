//! The prelude.
//!
//! To use the esp32_hal  effectively, a lot of traits and types need to be imported.
//! Instead of importing them one by one manually, the prelude contains the most
//! commonly used imports that are used around application runtime management.
//!
//! This can be imported as use `esp32_hal::prelude::*`.

pub use xtensa_lx6_rt::entry;
pub use xtensa_lx6_rt::exception;

pub use crate::analog::SensExt;
pub use crate::gpio::GpioExt;
pub use crate::interrupt;
pub use crate::proc_macros::*;
pub use crate::units::*;

pub use embedded_hal::digital::v2::InputPin as _embedded_hal_digital_v2_InputPin;
pub use embedded_hal::digital::v2::OutputPin as _embedded_hal_digital_v2_OutputPin;
pub use embedded_hal::digital::v2::StatefulOutputPin as _embedded_hal_digital_v2_StatefulOutputPin;
pub use embedded_hal::digital::v2::ToggleableOutputPin as _embedded_hal_digital_v2_ToggleableOutputPin;
pub use embedded_hal::prelude::*;
