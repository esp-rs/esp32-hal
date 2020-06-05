//! Integrated timer control
//!
//!
//!
//!
//!

use embedded_hal::timer::{CountDown, Periodic};

use crate::clock_control::ClockControlConfig;
use crate::prelude::*;
use core::marker::PhantomData;
use esp32::{TIMG0, TIMG1};

pub mod watchdog;

/// Timer errors
#[derive(Debug)]
pub enum Error {
    /// Unsupported frequency configuration
    UnsupportedWatchdogConfig,
    OutOfRange,
}

/// Hardware timers
pub struct Timer<TIMG, INST> {
    clock_control: ClockControlConfig,
    timg: *const esp32::timg::RegisterBlock,
    _group: PhantomData<TIMG>,
    _timer: PhantomData<INST>,
}

unsafe impl<TIMG, INST> Send for Timer<TIMG, INST> {}

/// Interrupt events
pub enum Event {
    /// Timer timed out / count down ended
    TimeOut,
}

pub trait TimerGroup: core::ops::Deref {}
impl TimerGroup for esp32::TIMG0 {}
impl TimerGroup for esp32::TIMG1 {}

pub trait TimerInst {}

pub struct TimerX {}
impl TimerInst for TimerX {}
pub struct Timer0 {}
impl TimerInst for Timer0 {}
pub struct Timer1 {}
impl TimerInst for Timer1 {}

impl<TIMG> Timer<TIMG, TimerX>
where
    TIMG: TimerGroup,
{
    pub fn new(
        timg: TIMG,
        clock_control: ClockControlConfig,
    ) -> (
        Timer<TIMG, Timer0>,
        Timer<TIMG, Timer1>,
        watchdog::WatchDog<TIMG>,
    ) {
        let timer0 = Timer::<TIMG, Timer0> {
            clock_control,
            timg: &*timg as *const _ as *const esp32::timg::RegisterBlock,
            _group: PhantomData {},
            _timer: PhantomData {},
        };
        let timer1 = Timer::<TIMG, Timer1> {
            clock_control,
            timg: &*timg as *const _ as *const esp32::timg::RegisterBlock,
            _group: PhantomData {},
            _timer: PhantomData {},
        };
        (timer0, timer1, watchdog::WatchDog::new(timg, clock_control))
    }
}

impl<INST> Timer<TIMG0, INST>
where
    INST: TimerInst,
{
    pub fn release(_timer0: Timer<TIMG0, Timer0>, _timer1: Timer<TIMG0, Timer1>) -> TIMG0 {
        unsafe { esp32::Peripherals::steal().TIMG0 }
    }
}

impl<INST> Timer<TIMG1, INST>
where
    INST: TimerInst,
{
    pub fn release(_timer0: Timer<TIMG1, Timer0>, _timer1: Timer<TIMG1, Timer1>) -> TIMG1 {
        unsafe { esp32::Peripherals::steal().TIMG1 }
    }
}

static TIMER_MUTEX: spin::Mutex<()> = spin::Mutex::new(());

macro_rules! timer {
    ($TIMX:ident, $INT_ENA:ident, $CONFIG:ident, $HI:ident, $LO: ident,
        $LOAD: ident, $LOADHI: ident, $LOADLO:ident, $UPDATE:ident, $ALARMHI:ident,
        $ALARMLO:ident, $EN:ident, $INCREASE:ident, $AUTORELOAD:ident, $DIVIDER:ident,
        $EDGE_INT_EN:ident, $LEVEL_INT_EN:ident, $ALARM_EN:ident,
        $INT_RAW:ident, $INT_ST:ident, $INT_CLR:ident
    ) => {
        impl<TIMG> Timer<TIMG, $TIMX>
        where
            TIMG: TimerGroup,
        {
            /// Starts listening for an `event`
            //  Needs multi-threaded protection as timer0 and 1 use same register
            pub fn listen(&mut self, event: Event) {
                match event {
                    Event::TimeOut => unsafe {
                        interrupt::free(|_| {
                            TIMER_MUTEX.lock();
                            (*(self.timg))
                                .int_ena_timers
                                .modify(|_, w| w.$INT_ENA().set_bit());
                        });
                    },
                }
            }

            /// Stops listening for an `event`
            //  Needs multi-threaded protection as timer0 and 1 use same register
            pub fn unlisten(&mut self, event: Event) {
                match event {
                    Event::TimeOut => unsafe {
                        interrupt::free(|_| {
                            TIMER_MUTEX.lock();
                            (*(self.timg))
                                .int_ena_timers
                                .modify(|_, w| w.$INT_ENA().clear_bit())
                        });
                    },
                }
            }

            pub fn set_value(&mut self, value: u64) {
                unsafe {
                    (*(self.timg)).$LOADLO.write(|w| w.bits(value as u32));
                    (*(self.timg))
                        .$LOADHI
                        .write(|w| w.bits((value >> 32) as u32));
                    (*(self.timg)).$LOAD.write(|w| w.bits(1));
                }
            }

            pub fn get_value(&mut self) -> u64 {
                unsafe {
                    (*(self.timg)).$UPDATE.write(|w| w.bits(1));
                    (((*(self.timg)).$HI.read().bits() as u64) << 32)
                        | ((*(self.timg)).$LO.read().bits() as u64)
                }
            }

            pub fn get_alarm(&mut self) -> u64 {
                unsafe {
                    (((*(self.timg)).$ALARMHI.read().bits() as u64) << 32)
                        | ((*(self.timg)).$ALARMLO.read().bits() as u64)
                }
            }

            pub fn set_alarm(&mut self, value: u64) {
                unsafe {
                    // TODO: surround by disable and enable to prevent false trigger
                    (*(self.timg)).$ALARMLO.write(|w| w.bits(value as u32));
                    (*(self.timg))
                        .$ALARMHI
                        .write(|w| w.bits((value >> 32) as u32));
                }
            }

            pub fn enable(&mut self, enable: bool) {
                unsafe { (*(self.timg)).$CONFIG.modify(|_, w| w.$EN().bit(enable)) }
            }

            pub fn increasing(&mut self, enable: bool) {
                unsafe {
                    (*(self.timg))
                        .$CONFIG
                        .modify(|_, w| w.$INCREASE().bit(enable))
                }
            }

            pub fn autoreload(&mut self, enable: bool) {
                unsafe {
                    (*(self.timg))
                        .$CONFIG
                        .modify(|_, w| w.$AUTORELOAD().bit(enable))
                }
            }

            pub fn enable_edge_interrupt(&mut self, enable: bool) {
                unsafe {
                    (*(self.timg))
                        .$CONFIG
                        .modify(|_, w| w.$EDGE_INT_EN().bit(enable))
                }
            }

            pub fn enable_level_interrupt(&mut self, enable: bool) {
                unsafe {
                    (*(self.timg))
                        .$CONFIG
                        .modify(|_, w| w.$LEVEL_INT_EN().bit(enable))
                }
            }

            pub fn enable_alarm(&mut self, enable: bool) {
                unsafe {
                    (*(self.timg))
                        .$CONFIG
                        .modify(|_, w| w.$ALARM_EN().bit(enable))
                }
            }

            pub fn alarm_active(&mut self) -> bool {
                unsafe { (*(self.timg)).$CONFIG.read().$ALARM_EN().bit_is_set() }
            }

            pub fn interrupt_active_raw(&mut self) -> bool {
                unsafe { (*(self.timg)).int_raw_timers.read().$INT_RAW().bit_is_set() }
            }

            pub fn interrupt_active(&mut self) -> bool {
                unsafe { (*(self.timg)).int_st_timers.read().$INT_ST().bit_is_set() }
            }

            pub fn clear_interrupt(&mut self) {
                unsafe {
                    (*(self.timg))
                        .int_clr_timers
                        .write(|w| w.$INT_CLR().set_bit())
                }
            }
        }
    };
}

timer!(
    Timer0,
    t0_int_ena,
    t0config,
    t0hi,
    t0lo,
    t0load,
    t0loadhi,
    t0loadlo,
    t0update,
    t0alarmhi,
    t0alarmlo,
    t0_en,
    t0_increase,
    t0_autoreload,
    t0_divider,
    t0_edge_int_en,
    t0_level_int_en,
    t0_alarm_en,
    t0_int_raw,
    t0_int_st,
    t0_int_clr
);

timer!(
    Timer1,
    t1_int_ena,
    t1config,
    t1hi,
    t1lo,
    t1load,
    t1loadhi,
    t1loadlo,
    t1update,
    t1alarmhi,
    t1alarmlo,
    t1_en,
    t1_increase,
    t1_autoreload,
    t1_divider,
    t1_edge_int_en,
    t1_level_int_en,
    t1_alarm_en,
    t1_int_raw,
    t1_int_st,
    t1_int_clr
);

/*
impl Timer<esp32::TIMG0> {
    pub fn timer0(
        _timg: esp32::TIMG0,
        clock_control: ClockControlConfig,
    ) -> (Self, Self, watchdog::WatchDog) {
        let timer0 = Timer::<esp32::TIMG0> {
            clock_control,
            timg: TIMG0::ptr(),
            _group: PhantomData {},
        };
        let timer1 = Timer::<esp32::TIMG0> {
            clock_control,
            timg: TIMG0::ptr(),
            _group: PhantomData {},
        };
        (timer0, timer1, watchdog::WatchDog::new(clock_control))
    }
}
*/
