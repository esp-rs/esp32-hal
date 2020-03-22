#![no_std]
#![feature(const_raw_ptr_deref)]
#![feature(const_fn)]

pub use embedded_hal as hal;
pub use esp32;

pub mod clock_control;
pub mod dport;
pub mod gpio;
pub mod prelude;
pub mod serial;
pub mod units;

mod rom;

#[macro_use]
pub mod dprint;
