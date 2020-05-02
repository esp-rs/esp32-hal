pub use crate::analog::SensExt;
pub use crate::gpio::GpioExt;
pub use crate::proc_macros::*;
pub use crate::units::*;

pub use embedded_hal::digital::v2::InputPin as _;
pub use embedded_hal::digital::v2::OutputPin as _;
pub use embedded_hal::digital::v2::StatefulOutputPin as _;
pub use embedded_hal::digital::v2::ToggleableOutputPin as _;
pub use embedded_hal::prelude::*;

pub use xtensa_lx6_rt::entry;
