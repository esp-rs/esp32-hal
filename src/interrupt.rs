//! ESP32 specific interrupt handling
//!
//! ESP32 uses 2-level interrupt handling: peripheral interrupts are mapped to cpu interrupts.
//! This module redirects the cpu interrupts handler to registered peripheral interrupt handlers.
//!
//! Interrupt handlers are defined using the [Interrupt](attr.interrupt.html) attribute.
//! (Note that this is a distinct attribute from the one in the [xtensa_lx6_rt](xtensa_lx6_rt)
//! crate.)
//!
//! To enable the interrupt and assign to a specific interrupt level use
//! the [enable] or [enable_with_priority] functions. (This is in addition to enabling the
//! interrupt in the respective peripherals.)
//!
//! To have lowest latency possible you can use the
//! [Interrupt](../../xtensa_lx6_rt/attr.interrupt.html) attribute from the xtensa_lx6_rt crate
//! to define low level/naked interrupt handlers. (This will override the interrupt
//! handling offered by this crate for that specific interrupt level. This should especially be
//! considered when using Level 7 = Non Maskable Interrupt level as these will not be turned off
//! during [interrupt::free](interrupt::free) sections.)
//!
//! **Note: If multiple edge triggered interrupts are assigned to the same [level][InterruptLevel],
//!   it is not possible to detect which peripheral triggered the interrupt. Therefore all
//!   registered handlers will be called.**
//!
//! **Note: Edge triggered interrupts can be lost when triggered after handling of another edge
//!   triggered interrupt has started.**
//!
//! *Note: routines and variables in this module are stored in RAM because otherwise it may lead
//! to exceptions when the flash is programmed or erased while the interrupt is called.*
use crate::ram;

use crate::target;
pub use crate::target::Interrupt::{self, *};
use crate::target::DPORT;
use crate::Core::{self, APP, PRO};
use bare_metal::Nr;
pub use proc_macros::interrupt;
pub use xtensa_lx6::interrupt::{self, free};

/// Interrupt errors
#[derive(Debug)]
pub enum Error {
    InvalidCore,
    InvalidCPUInterrupt,
    InvalidInterruptLevel,
    InternalInterruptsCannotBeMapped,
    InvalidInterrupt,
}

/// Interrupt level.
///
/// Valid levels are 1 through 7. Level 6 is typically used for debug exceptions
/// and level 7 for Non-Maskable Interrupts (NMI).
///
/// Level 0 is used to disable interrupts.
///
/// **Note: Level 7 (NMI) will not be disabled by the
/// [interrupt::free](interrupt::free) section. This risks race conditions in
/// various places.
/// **
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default)]
pub struct InterruptLevel(pub usize);

#[ram]
const CPU_INTERRUPT_EDGE: u32 = 0b_0111_0000_0100_0000_0000_1100_1000_0000;

#[ram]
const INTERRUPT_EDGE: u128 =
    0b_0000_0000_0000_0000_0000_0000_0000_0000__0000_0000_0000_0000_0000_0000_0000_0011__1111_1100_0000_0000_0000_0000_0000_0000__0000_0000_0000_0000_0000_0000_0000_0000;

#[ram]
const CPU_INTERRUPT_INTERNAL: u32 = 0b_0010_0000_0000_0001_1000_1000_1100_0000;

#[ram]
const CPU_INTERRUPT_LEVELS: [u32; 8] = [
    0b_0000_0000_0000_0000_0000_0000_0000_0000, // Dummy level 0
    0b_0000_0000_0000_0110_0011_0111_1111_1111, // Level_1
    0b_0000_0000_0011_1000_0000_0000_0000_0000, // Level 2
    0b_0010_1000_1100_0000_1000_1000_0000_0000, // Level 3
    0b_0101_0011_0000_0000_0000_0000_0000_0000, // Level 4
    0b_1000_0100_0000_0001_0000_0000_0000_0000, // Level 5
    0b_0000_0000_0000_0000_0000_0000_0000_0000, // Level 6
    0b_0000_0000_0000_0000_0100_0000_0000_0000, // Level 7
];

#[ram]
fn interrupt_is_edge(interrupt: Interrupt) -> bool {
    [
        TG0_T0_EDGE_INTR,
        TG0_T1_EDGE_INTR,
        TG0_WDT_EDGE_INTR,
        TG0_LACT_EDGE_INTR,
        TG1_T0_EDGE_INTR,
        TG1_T1_EDGE_INTR,
        TG1_WDT_EDGE_INTR,
        TG1_LACT_EDGE_INTR,
    ]
    .contains(&interrupt)
}

#[ram]
fn interrupt_level_to_cpu_interrupt(
    interrupt_level: InterruptLevel,
    edge: bool,
) -> Result<CPUInterrupt, Error> {
    #[ram]
    const INTERRUPT_LEVEL_TO_CPU_INTERRUPT_EDGE: [Option<CPUInterrupt>; 8] = [
        Some(CPUInterrupt(6)),  // Disable (assign to internal interrupt)
        Some(CPUInterrupt(10)), // Level 1 edge triggered
        None,                   // Level 2 edge triggered not supported
        Some(CPUInterrupt(22)), // Level 3 edge triggered
        Some(CPUInterrupt(28)), // Level 4 edge triggered
        None,                   // Level 5 edge triggered not supported
        None,                   // Level 6 = Debug not supported for peripherals
        Some(CPUInterrupt(14)), // Level 7 = NMI edge triggered
    ];
    #[ram]
    const INTERRUPT_LEVEL_TO_CPU_INTERRUPT_LEVEL: [Option<CPUInterrupt>; 8] = [
        Some(CPUInterrupt(6)),  // Disable (assign to internal interrupt)
        Some(CPUInterrupt(0)),  // Level 1 level triggered
        Some(CPUInterrupt(19)), // Level 2 level triggered
        Some(CPUInterrupt(23)), // Level 3 level triggered
        Some(CPUInterrupt(24)), // Level 4 level triggered
        Some(CPUInterrupt(31)), // Level 5 level triggered
        None,                   // Level 6 = Debug not supported for peripherals
        Some(CPUInterrupt(14)), // Level 7 = NMI level triggered (not supported for peripherals,
                                //                                      forward to edge interrupt)
    ];
    if edge {
        INTERRUPT_LEVEL_TO_CPU_INTERRUPT_EDGE[interrupt_level.0].ok_or(Error::InvalidInterruptLevel)
    } else {
        INTERRUPT_LEVEL_TO_CPU_INTERRUPT_LEVEL[interrupt_level.0]
            .ok_or(Error::InvalidInterruptLevel)
    }
}

#[ram]
const CPU_INTERRUPT_USED_LEVELS: u32 = 0b_1001_0001_1100_1000_0100_0100_0000_0001;

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default)]
struct CPUInterrupt(pub usize);

fn cpu_interrupt_to_interrupt(cpu_interrupt: CPUInterrupt) -> Result<target::Interrupt, Error> {
    #[ram]
    const CPU_INTERRUPT_TO_INTERRUPT: [Option<target::Interrupt>; 32] = [
        None,
        None,
        None,
        None,
        None,
        None,
        Some(target::Interrupt::INTERNAL_TIMER0_INTR),
        Some(target::Interrupt::INTERNAL_SOFTWARE_LEVEL_1_INTR),
        None,
        None,
        None,
        Some(target::Interrupt::INTERNAL_PROFILING_INTR),
        None,
        None,
        None,
        Some(target::Interrupt::INTERNAL_TIMER1_INTR),
        Some(target::Interrupt::INTERNAL_TIMER2_INTR),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(target::Interrupt::INTERNAL_SOFTWARE_LEVEL_3_INTR),
        None,
        None,
    ];

    CPU_INTERRUPT_TO_INTERRUPT[cpu_interrupt.0].ok_or(Error::InvalidCPUInterrupt)
}

#[ram]
fn interrupt_to_cpu_interrupt(interrupt: target::Interrupt) -> Result<CPUInterrupt, Error> {
    match interrupt {
        target::Interrupt::INTERNAL_TIMER0_INTR => Ok(CPUInterrupt(6)),
        target::Interrupt::INTERNAL_SOFTWARE_LEVEL_1_INTR => Ok(CPUInterrupt(7)),
        target::Interrupt::INTERNAL_PROFILING_INTR => Ok(CPUInterrupt(11)),
        target::Interrupt::INTERNAL_TIMER1_INTR => Ok(CPUInterrupt(15)),
        target::Interrupt::INTERNAL_TIMER2_INTR => Ok(CPUInterrupt(16)),
        target::Interrupt::INTERNAL_SOFTWARE_LEVEL_3_INTR => Ok(CPUInterrupt(29)),
        _ => Err(Error::InvalidCPUInterrupt),
    }
}

#[ram]
fn cpu_interrupt_to_level(cpu_interrupt: CPUInterrupt) -> InterruptLevel {
    #[ram]
    const CPU_INTERRUPT_TO_LEVEL: [usize; 32] = [
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 7, 3, 5, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5, 3, 4, 3,
        4, 5,
    ];
    InterruptLevel(CPU_INTERRUPT_TO_LEVEL[cpu_interrupt.0 as usize])
}

/// 75 Peripheral interrupts on the esp32, hence we use u128 (128 bits) to signal what interrupts
/// are enabled at each level. I.e setting the 75th bit inside `INTERRUPT_LEVELS[1]`would indicate the 75th interrupt defined in the `esp32`
/// crate is enabled with a priority of 1.
#[ram]
static mut INTERRUPT_LEVELS: [u128; 8] = [0u128; 8];

#[ram]
static INTERRUPT_LEVELS_MUTEX: spin::Mutex<bool> = spin::Mutex::new(false);

#[xtensa_lx6_rt::interrupt(1)]
#[ram]
unsafe fn level_1_handler(level: u32) {
    handle_interrupts(level)
}

#[xtensa_lx6_rt::interrupt(2)]
#[ram]
unsafe fn level_2_handler(level: u32) {
    handle_interrupts(level)
}

#[xtensa_lx6_rt::interrupt(3)]
#[ram]
unsafe fn level_3_handler(level: u32) {
    handle_interrupts(level)
}

#[xtensa_lx6_rt::interrupt(4)]
#[ram]
unsafe fn level_4_handler(level: u32) {
    handle_interrupts(level)
}

#[xtensa_lx6_rt::interrupt(5)]
#[ram]
unsafe fn level_5_handler(level: u32) {
    handle_interrupts(level)
}

#[xtensa_lx6_rt::interrupt(6)]
#[ram]
unsafe fn level_6_handler(level: u32) {
    handle_interrupts(level)
}

#[xtensa_lx6_rt::interrupt(7)]
#[ram]
unsafe fn level_7_handler(level: u32) {
    handle_interrupts(level)
}

#[ram]
unsafe fn handle_interrupt(level: u32, interrupt: Interrupt) {
    let handler = target::__INTERRUPTS[interrupt.nr() as usize]._handler;
    if handler as *const _ == DefaultHandler as *const unsafe extern "C" fn() {
        DefaultHandler(level, interrupt);
    } else {
        handler();
    }
}

#[inline(always)]
#[ram]
unsafe fn handle_interrupts(level: u32) {
    let cpu_interrupt_mask =
        interrupt::get() & interrupt::get_mask() & CPU_INTERRUPT_LEVELS[level as usize];

    // first check if the pending interrupt is a CPU interrupt
    if cpu_interrupt_mask & CPU_INTERRUPT_INTERNAL != 0 {
        let cpu_interrupt_mask = cpu_interrupt_mask & CPU_INTERRUPT_INTERNAL;
        let cpu_interrupt_nr = cpu_interrupt_mask.trailing_zeros();

        if (cpu_interrupt_mask & CPU_INTERRUPT_EDGE) != 0 {
            interrupt::clear(1 << cpu_interrupt_nr);
        }

        // cpu_interrupt_to_interrupt can fail if interrupt already de-asserted: silently ignore
        if let Ok(interrupt) = cpu_interrupt_to_interrupt(CPUInterrupt(cpu_interrupt_nr as usize)) {
            handle_interrupt(level, interrupt);
        }
    } else {
        let cpu_interrupt_mask = cpu_interrupt_mask & !CPU_INTERRUPT_INTERNAL;

        // then check if the pending interrupt is an edge triggered interrupt
        if (cpu_interrupt_mask & CPU_INTERRUPT_EDGE) != 0 {
            let cpu_interrupt_mask = cpu_interrupt_mask & CPU_INTERRUPT_EDGE;
            let cpu_interrupt_nr = cpu_interrupt_mask.trailing_zeros();
            interrupt::clear(1 << cpu_interrupt_nr);

            // for edge interrupts cannot rely on the interrupt status register, therefore call all
            // registered handlers for current level
            let mut interrupt_mask = INTERRUPT_LEVELS[level as usize] & INTERRUPT_EDGE;
            loop {
                let interrupt_nr = interrupt_mask.trailing_zeros();
                if let Ok(interrupt) = target::Interrupt::try_from(interrupt_nr as u8) {
                    handle_interrupt(level, interrupt)
                } else {
                    break;
                }
                interrupt_mask &= !(1u128 << interrupt_nr);
            }
        } else {
            // finally, if it is not a CPU or edge interrupt is must a be peripheral interrupt
            // bitwise and the dport interrupt status bits with the enabled interrupts for that level
            let interrupt_mask =
                get_interrupt_status(crate::get_core()) & INTERRUPT_LEVELS[level as usize];
            let interrupt_nr = interrupt_mask.trailing_zeros();

            // target::Interrupt::try_from can fail if interrupt already de-asserted: silently ignore
            if let Ok(interrupt) = target::Interrupt::try_from(interrupt_nr as u8) {
                handle_interrupt(level, interrupt);
            }
        }
    }
}

#[no_mangle]
#[ram]
extern "C" fn DefaultHandler(level: u32, interrupt: target::Interrupt) {
    crate::dprintln!("Unhandled interrupt (level {}): {:?}", level, interrupt);
}

/// Get status of peripheral interrupts
#[ram]
pub fn get_interrupt_status(core: Core) -> u128 {
    unsafe {
        match core {
            PRO => {
                ((*DPORT::ptr()).pro_intr_status_0.read().bits() as u128)
                    | ((*DPORT::ptr()).pro_intr_status_1.read().bits() as u128) << 32
                    | ((*DPORT::ptr()).pro_intr_status_2.read().bits() as u128) << 64
            }
            APP => {
                ((*DPORT::ptr()).app_intr_status_0.read().bits() as u128)
                    | ((*DPORT::ptr()).app_intr_status_1.read().bits() as u128) << 32
                    | ((*DPORT::ptr()).app_intr_status_2.read().bits() as u128) << 64
            }
        }
    }
}

/// Map an interrupt to a CPU interrupt
#[ram]
fn map_interrupt(
    core: crate::Core,
    interrupt: Interrupt,
    cpu_interrupt: CPUInterrupt,
) -> Result<(), Error> {
    if cpu_interrupt.0 >= 32 {
        return Err(Error::InvalidCPUInterrupt);
    }
    if interrupt.nr() >= Interrupt::INTERNAL_TIMER0_INTR.nr() {
        return Err(Error::InternalInterruptsCannotBeMapped);
    }
    unsafe {
        let base_reg = match core {
            crate::Core::PRO => (*DPORT::ptr()).pro_mac_intr_map.as_ptr(),
            crate::Core::APP => (*DPORT::ptr()).app_mac_intr_map.as_ptr(),
        };

        let reg = base_reg.add(interrupt.nr() as usize);
        *reg = cpu_interrupt.0 as u32;
    };
    Ok(())
}

/// Enable interrupt and set priority for a particular core
///
/// Valid levels are 1-7. Level 0 is used to disable the interrupt.
///
/// *Note: CPU internal interrupts can only be set on the current core.*
///
/// *Note: take care when mapping multiple peripheral edge triggered interrupts to the same level:
/// this will cause all handlers to be called.*
#[ram]
pub fn enable_with_priority(
    core: crate::Core,
    interrupt: Interrupt,
    level: InterruptLevel,
) -> Result<(), Error> {
    match interrupt_to_cpu_interrupt(interrupt) {
        Ok(cpu_interrupt) => {
            if core != crate::get_core() {
                return Err(Error::InvalidCore);
            }
            if level == InterruptLevel(0) {
                interrupt::disable_mask(1 << cpu_interrupt.0);
                return Ok(());
            } else if level == cpu_interrupt_to_level(cpu_interrupt) {
                unsafe { interrupt::enable_mask(1 << cpu_interrupt.0) };
                return Ok(());
            } else {
                return Err(Error::InvalidInterruptLevel);
            }
        }
        Err(_) => {
            let cpu_interrupt =
                interrupt_level_to_cpu_interrupt(level, interrupt_is_edge(interrupt))?;

            return interrupt::free(|_| unsafe {
                let _data = INTERRUPT_LEVELS_MUTEX.lock();
                for i in 0..=7 {
                    INTERRUPT_LEVELS[i] &= !(1 << interrupt.nr());
                }
                INTERRUPT_LEVELS[level.0 as usize] |= 1 << interrupt.nr();

                interrupt::enable_mask(CPU_INTERRUPT_USED_LEVELS);

                map_interrupt(core, interrupt, cpu_interrupt)
            });
        }
    }
}

/// Enable interrupt
///
/// For CPU internal interrupts use the default level, for others use level 1
///
/// *Note: CPU internal interrupts can only be set on the current core.*
///
/// *Note: take care when mapping multiple peripheral edge triggered interrupts to the same level:
/// this will cause all handlers to be called.*
#[ram]
pub fn enable(interrupt: Interrupt) -> Result<(), Error> {
    match interrupt_to_cpu_interrupt(interrupt) {
        Ok(cpu_interrupt) => {
            unsafe { interrupt::enable_mask(1 << cpu_interrupt.0) };
            return Ok(());
        }
        Err(_) => enable_with_priority(crate::get_core(), interrupt, InterruptLevel(1)),
    }
}

/// Disable interrupt
#[ram]
pub fn disable(interrupt: Interrupt) -> Result<(), Error> {
    match interrupt_to_cpu_interrupt(interrupt) {
        Ok(cpu_interrupt) => {
            unsafe { interrupt::enable_mask(1 << cpu_interrupt.0) };
            return Ok(());
        }
        Err(_) => enable_with_priority(crate::get_core(), interrupt, InterruptLevel(0)),
    }
}

/// Trigger a (cross-)core interrupt
///
/// Valid interrupts are FROM_CPU_INTR[0-3],
/// INTERNAL_SOFTWARE_LEVEL_1_INTR and INTERNAL_SOFTWARE_LEVEL_3_INTR.
#[ram]
pub fn set_software_interrupt(interrupt: Interrupt) -> Result<(), Error> {
    unsafe {
        match interrupt {
            FROM_CPU_INTR0 => (*DPORT::ptr())
                .cpu_intr_from_cpu_0
                .write(|w| w.cpu_intr_from_cpu_0().set_bit()),
            FROM_CPU_INTR1 => (*DPORT::ptr())
                .cpu_intr_from_cpu_1
                .write(|w| w.cpu_intr_from_cpu_1().set_bit()),
            FROM_CPU_INTR2 => (*DPORT::ptr())
                .cpu_intr_from_cpu_2
                .write(|w| w.cpu_intr_from_cpu_2().set_bit()),
            FROM_CPU_INTR3 => (*DPORT::ptr())
                .cpu_intr_from_cpu_3
                .write(|w| w.cpu_intr_from_cpu_3().set_bit()),
            INTERNAL_SOFTWARE_LEVEL_1_INTR | INTERNAL_SOFTWARE_LEVEL_3_INTR => {
                interrupt::set(1 << interrupt_to_cpu_interrupt(interrupt)?.0)
            }

            _ => return Err(Error::InvalidInterrupt),
        }
    };
    Ok(())
}

/// Clear a (cross-)core interrupt
///
/// Valid interrupts are FROM_CPU_INTR[0-3],
/// INTERNAL_SOFTWARE_LEVEL_1_INTR and INTERNAL_SOFTWARE_LEVEL_3_INTR.
#[ram]
pub fn clear_software_interrupt(interrupt: Interrupt) -> Result<(), Error> {
    unsafe {
        match interrupt {
            FROM_CPU_INTR0 => (*DPORT::ptr())
                .cpu_intr_from_cpu_0
                .write(|w| w.cpu_intr_from_cpu_0().clear_bit()),
            FROM_CPU_INTR1 => (*DPORT::ptr())
                .cpu_intr_from_cpu_1
                .write(|w| w.cpu_intr_from_cpu_1().clear_bit()),
            FROM_CPU_INTR2 => (*DPORT::ptr())
                .cpu_intr_from_cpu_2
                .write(|w| w.cpu_intr_from_cpu_2().clear_bit()),
            FROM_CPU_INTR3 => (*DPORT::ptr())
                .cpu_intr_from_cpu_3
                .write(|w| w.cpu_intr_from_cpu_3().clear_bit()),
            INTERNAL_SOFTWARE_LEVEL_1_INTR | INTERNAL_SOFTWARE_LEVEL_3_INTR => {
                interrupt::clear(1 << interrupt_to_cpu_interrupt(interrupt)?.0)
            }

            _ => return Err(Error::InvalidInterrupt),
        }
    };
    Ok(())
}


use xtensa_lx6::interrupt::InterruptMatrix;

// pub struct Nvic {
//     interrupt_levels: [u128; 8]
// }

pub struct Nvic;

impl InterruptMatrix for Nvic {
    type CpuInterruptStorage = u8;
    type PriorityStorage = usize;

    /// Mask an enabled interrupt
    fn mask<I: bare_metal::Nr>(_interrupt: I) {}

    /// Unmask an enabled interrupt
    fn unmask<I: bare_metal::Nr>(_interrupt: I) {}

    /// Enable an interrupt and assign to a CPU interrupt with a priority
    fn enable<I: bare_metal::Nr>(interrupt: I , _cpu_intr: Self::CpuInterruptStorage, prio: Self::PriorityStorage) {
        enable_with_priority(crate::get_core(), target::Interrupt::try_from(interrupt.nr()).unwrap(), InterruptLevel(prio)).unwrap();
    }

    /// Disable an interrupt
    fn disable<I: bare_metal::Nr>(interrupt: I) {
        disable(target::Interrupt::try_from(interrupt.nr()).unwrap()).unwrap();
    }

    /// Pend an interrupt
    fn pend<I: bare_metal::Nr>(_interrupt: I) {}

    /// Unpend an interrupt
    fn unpend<I: bare_metal::Nr>(_interrupt: I) {}
}
