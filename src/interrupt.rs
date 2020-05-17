//! ESP32 specific interrupt handling
//!
//!

use crate::ram;

use crate::Core::{self, APP, PRO};
use bare_metal::Nr;
use esp32::{Interrupt, DPORT};

/// Interrupt errors
#[derive(Debug)]
pub enum Error {
    InvalidCore,
    InvalidCPUInterrupt,
    InvalidInterruptLevel,
    InternalInterruptsCannotBeMapped,
}

const CPU_INTERRUPT_EDGE: u32 = 0b_0111_0000_0100_0000_0000_1100_0000_0000;
const CPU_INTERRUPT_INTERNAL: u32 = 0b_0010_0000_0000_0001_1000_1000_1100_0000;

const CPU_INTERRUPT_LEVELS: [u32; 8] = [
    0b_0000_0000_0000_0000_0000_0000_0000_0000, // Dummy level 0
    0b_0000_0000_0000_0110_0011_0111_1111_1111, // Level_1
    0b_0000_0000_0011_1000_0000_0000_0000_0000, // Level 2
    0b_0010_1000_1100_0000_0000_1000_0000_0000, // Level 3
    0b_0101_0011_0000_0000_0000_0000_0000_0000, // Level 4
    0b_1000_0100_0000_0000_0000_0000_0000_0000, // Level 5
    0b_0000_0000_0000_0000_0000_0000_0000_0000, // Level 6
    0b_0000_0000_0000_0000_0000_0000_0000_0000, // Level 7
];

const INTERRUPT_TO_CPU_LEVEL: [u32; 8] = [
    6,  // Disable (assign to internal interrupt)
    1,  // Level 1 level triggered
    19, // Level 2 level triggered
    23, // Level 3 level triggered
    24, // Level 4 level triggered
    31, // Level 5 level triggered
    6,  // Level 6=Debug not supported for peripherals (assign to internal interrupt)
    6,  // Level 7=NMI level triggered not supported for peripherals (assign to internal interrupt)
];

const INTERRUPT_TO_CPU_EDGE: [u32; 8] = [
    6,  // Disable (assign to internal interrupt)
    10, // Level 1 edge triggered
    6,  // Level 2 edge triggered not supported (assign to internal interrupt)
    22, // Level 3 edge triggered
    28, // Level 4 edge triggered
    31, // Level 5 edge triggered not supported (assign to internal interrupt)
    6,  // Level 6=Debug not supported for peripherals (assign to internal interrupt)
    14, // Level 7=NMI edge triggered
];

const CPU_INTERRUPT_USED_LEVELS: u32 = 0b_1001_0001_1100_1000_0100_0100_0000_0001;

const CPU_INTERRUPT_TO_INTERRUPT: [Option<esp32::Interrupt>; 32] = [
    None,
    None,
    None,
    None,
    None,
    None,
    Some(esp32::Interrupt::INTERNAL_TIMER0_INTR),
    Some(esp32::Interrupt::INTERNAL_SOFTWARE_LEVEL_1_INTR),
    None,
    None,
    None,
    Some(esp32::Interrupt::INTERNAL_PROFILING_INTR),
    None,
    None,
    None,
    Some(esp32::Interrupt::INTERNAL_TIMER1_INTR),
    Some(esp32::Interrupt::INTERNAL_TIMER2_INTR),
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
    Some(esp32::Interrupt::INTERNAL_SOFTWARE_LEVEL_3_INTR),
    None,
    None,
];

#[ram]
static INTERRUPT_LEVELS: spin::Mutex<[u128; 8]> = spin::Mutex::new([0u128; 8]);

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

#[inline(always)]
#[ram]
unsafe fn handle_interrupts(level: u32) {
    let cpu_interrupt_mask = xtensa_lx6_rt::interrupt::get();
    let interrupt = if (cpu_interrupt_mask
        & CPU_INTERRUPT_INTERNAL
        & CPU_INTERRUPT_LEVELS[level as usize])
        != 0
    {
        let cpu_interrupt_nr =
            (cpu_interrupt_mask & CPU_INTERRUPT_INTERNAL & CPU_INTERRUPT_LEVELS[level as usize])
                .trailing_zeros();

        xtensa_lx6_rt::interrupt::clear(1 << cpu_interrupt_nr);

        CPU_INTERRUPT_TO_INTERRUPT[cpu_interrupt_nr as usize].unwrap()
    } else {
        let interrupt_mask = get_interrupt_status(crate::get_core());

        let interrupt_nr =
            (interrupt_mask & INTERRUPT_LEVELS.lock()[level as usize]).trailing_zeros();

        esp32::Interrupt::try_from(interrupt_nr as u8).unwrap()
    };

    if esp32::__INTERRUPTS[interrupt.nr() as usize]._handler as *const unsafe extern "C" fn()
        == DefaultHandler as *const unsafe extern "C" fn()
    {
        DefaultHandler(level, interrupt);
    } else {
        (esp32::__INTERRUPTS[interrupt.nr() as usize]._handler)();
    }
}

#[no_mangle]
#[ram]
extern "C" fn DefaultHandler(level: u32, interrupt: esp32::Interrupt) {
    crate::dprintln!("Unhandled interrupt level {} {:?}", level, interrupt);
}

#[no_mangle]
#[ram]
extern "C" fn FROM_CPU_INTR0() {
    crate::dprintln!("CPU_INTR0");
    clear_cpu_interrupt(0).unwrap();
}

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

/// Trigger a (cross-)core interrupt
///
/// Valid interrupts are 0-3. Mapping to a certain core and interrupt level is done via
/// set_interrupt_priority.
pub fn set_cpu_interrupt(nr: u8) -> Result<(), Error> {
    unsafe {
        match nr {
            0 => (*DPORT::ptr())
                .cpu_intr_from_cpu_0
                .write(|w| w.cpu_intr_from_cpu_0().set_bit()),
            1 => (*DPORT::ptr())
                .cpu_intr_from_cpu_1
                .write(|w| w.cpu_intr_from_cpu_1().set_bit()),
            2 => (*DPORT::ptr())
                .cpu_intr_from_cpu_2
                .write(|w| w.cpu_intr_from_cpu_2().set_bit()),
            3 => (*DPORT::ptr())
                .cpu_intr_from_cpu_3
                .write(|w| w.cpu_intr_from_cpu_3().set_bit()),
            _ => return Err(Error::InvalidCore),
        }
    };
    Ok(())
}
/// Trigger a (cross-)core interrupt
///
/// Valid interrupts are 0-3. Mapping to a certain core and interrupt level is done via
/// set_interrupt_priority.
pub fn clear_cpu_interrupt(nr: u8) -> Result<(), Error> {
    unsafe {
        match nr {
            0 => (*DPORT::ptr())
                .cpu_intr_from_cpu_0
                .write(|w| w.cpu_intr_from_cpu_0().clear_bit()),
            1 => (*DPORT::ptr())
                .cpu_intr_from_cpu_1
                .write(|w| w.cpu_intr_from_cpu_1().clear_bit()),
            2 => (*DPORT::ptr())
                .cpu_intr_from_cpu_2
                .write(|w| w.cpu_intr_from_cpu_2().clear_bit()),
            3 => (*DPORT::ptr())
                .cpu_intr_from_cpu_3
                .write(|w| w.cpu_intr_from_cpu_3().clear_bit()),
            _ => return Err(Error::InvalidCore),
        }
    };
    Ok(())
}

fn map_interrupt(core: u8, interrupt: Interrupt, cpu_interrupt: u32) -> Result<(), Error> {
    if cpu_interrupt >= 32 {
        return Err(Error::InvalidCPUInterrupt);
    }
    if interrupt.nr() >= Interrupt::INTERNAL_TIMER0_INTR.nr() {
        return Err(Error::InternalInterruptsCannotBeMapped);
    }
    unsafe {
        let base_reg = match core {
            0 => (*DPORT::ptr()).pro_mac_intr_map.as_ptr(),
            1 => (*DPORT::ptr()).app_mac_intr_map.as_ptr(),
            _ => return Err(Error::InvalidCore),
        };

        let reg = base_reg.add(interrupt.nr() as usize);
        *reg = cpu_interrupt;
    };
    Ok(())
}

/// Set interrupt priority for a particular core
///
/// Valid levels are 1-7. 0 is used to disable the interrupt
pub fn set_interrupt_priority(
    core: u8,
    interrupt: Interrupt,
    level: u8,
    edge: bool,
) -> Result<(), Error> {
    let cpu_interrupt = match edge {
        true => match level {
            0 | 1 | 3 | 4 | 7 => INTERRUPT_TO_CPU_EDGE[level as usize],
            _ => return Err(Error::InvalidInterruptLevel),
        },
        false => match level {
            0..=5 | 7 => INTERRUPT_TO_CPU_LEVEL[level as usize],
            _ => return Err(Error::InvalidInterruptLevel),
        },
    };

    xtensa_lx6_rt::interrupt::free(|_| {
        let mut data = INTERRUPT_LEVELS.lock();

        for i in 0..=7 {
            (*data)[i] &= !(1 << interrupt.nr());
        }

        (*data)[level as usize] |= 1 << interrupt.nr();

        map_interrupt(core, interrupt, cpu_interrupt)
    })
}

pub fn enable() {
    unsafe {
        xtensa_lx6_rt::interrupt::enable_mask(CPU_INTERRUPT_USED_LEVELS);
    }
}
