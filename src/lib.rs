#![no_std]
#![feature(const_raw_ptr_deref)]

pub use embedded_hal as hal;
pub use esp32;

pub mod clock_control;
pub mod dport;
pub mod gpio;
pub mod prelude;
pub mod serial;
pub mod units;

mod rom;
