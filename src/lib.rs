#![no_std]

pub use embedded_hal as hal;
pub use esp32;

extern crate esp32_hal_proc_macros as proc_macros;
pub use proc_macros::ram;

pub mod analog;
pub mod clock_control;
pub mod dport;
pub mod efuse;
pub mod gpio;
pub mod prelude;
pub mod serial;
pub mod units;

#[macro_use]
pub mod dprint;
