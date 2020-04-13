//! Print debug information to UART0
//!
//! Directly writes to the UART0 TX uart queue.
//! This is unsafe! It is asynchronous with normal UART0 usage and
//! interrupts are not disabled.

use esp32::UART0;

pub struct DebugLog {}

pub enum Error {}

impl DebugLog {
    pub fn count(&mut self) -> u8 {
        unsafe { (*UART0::ptr()).status.read().txfifo_cnt().bits() }
    }

    pub fn is_idle(&mut self) -> bool {
        unsafe { (*UART0::ptr()).status.read().st_utx_out().is_tx_idle() }
    }

    pub fn write(&mut self, byte: u8) -> nb::Result<(), Error> {
        if self.count() < 128 {
            unsafe { (*UART0::ptr()).tx_fifo.write_with_zero(|w| w.bits(byte)) }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl core::fmt::Write for DebugLog {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

pub static mut DEBUG_LOG: DebugLog = DebugLog {};

/// Macro for sending a formatted string to UART0 for debugging
#[macro_export]
macro_rules! dprint {
    ($s:expr) => {
        unsafe {$crate::dprint::DEBUG_LOG.write_str($s).unwrap()};
    };
    ($($arg:tt)*) => {
        unsafe {$crate::dprint::DEBUG_LOG.write_fmt(format_args!($($arg)*)).unwrap()};
    };
}

/// Macro for sending a formatted string to UART0 for debugging, with a newline.
#[macro_export]
macro_rules! dprintln {
    () => {
        unsafe {$crate::dprint::DEBUG_LOG.write_str("\n").unwrap()};
    };
    ($fmt:expr) => {
        unsafe {$crate::dprint::DEBUG_LOG.write_str(concat!($fmt, "\n")).unwrap()};
    };
    ($fmt:expr, $($arg:tt)*) => {
        unsafe {$crate::dprint::DEBUG_LOG.write_fmt(format_args!(concat!($fmt, "\n"), $($arg)*)).unwrap()};
    };
}

/// Macro for sending a formatted string to UART0 for debugging, with a newline.
#[macro_export]
macro_rules! dflush {
    () => {
        unsafe { while !$crate::dprint::DEBUG_LOG.is_idle() {} };
    };
}
