//! External RAM (PSRAM) control
//!

use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};

// must be uninitialized, because otherwise will be zeroed in the Reset routine
#[crate::ram(uninitialized)]
static mut EXTERNAL_RAM_SIZE: core::mem::MaybeUninit<Option<usize>> =
    core::mem::MaybeUninit::uninit();

/// Info about the cache to be able to trash it
const CACHE_LINE_SIZE: usize = 32;
const NR_CACHE_LINES: usize = 1024;
/// number of steps to check RAM in. 8 steps limits to 0.5MB steps
const RAM_STEPS: usize = 8;

// These symbols come from `memory.x`
extern "C" {
    static mut _external_bss_start: u32;
    static mut _external_bss_end: u32;

    static _external_ram_start: u32;
    static _external_ram_end: u32;

    static mut _external_heap_start: u32;
}

/// Get the size of the external RAM (also called PSRAM).
pub fn get_size() -> usize {
    unsafe { EXTERNAL_RAM_SIZE.assume_init().unwrap() }
}

/// Initialize external RAM
pub(super) unsafe fn init() {
    EXTERNAL_RAM_SIZE = core::mem::MaybeUninit::new(Some(calculate_external_ram_size()));

    if &_external_heap_start as *const u32 > (&_external_ram_start as *const u32).add(get_size()) {
        panic!("External RAM too small for data");
    }
    xtensa_lx6_rt::zero_bss(&mut _external_bss_start, &mut _external_bss_end);
}

/// Calculate the size of external RAM by reading and writing at defined intervals while
/// thrashing the cache in between.
///
/// TODO: should be replaced by reading the size via SPI
unsafe fn calculate_external_ram_size() -> usize {
    let ram_start_addr: usize = &_external_ram_start as *const u32 as usize;
    let ram_end_addr: usize = &_external_ram_end as *const u32 as usize;

    let ram: *mut u32 = ram_start_addr as _;
    let mut buffer = [0u32; RAM_STEPS];

    let step_size = (ram_end_addr - ram_start_addr) / RAM_STEPS / size_of::<u32>();

    // write recognition pattern in the ram
    for i in 0..RAM_STEPS {
        buffer[i] = read_volatile(ram.add(i * step_size));
        write_volatile(ram.add(i * step_size), 0xdeadbeef + i as u32);
    }

    // trash the cache
    for i in (1..=NR_CACHE_LINES).step_by(CACHE_LINE_SIZE) {
        let addr = ram.add(i * CACHE_LINE_SIZE / size_of::<u32>());
        write_volatile(addr, read_volatile(addr));
    }

    // check the recognition pattern and restore content
    let mut ram_size = 0;
    let mut end_found = false;
    for i in 0..RAM_STEPS {
        if !end_found && read_volatile(ram.add(i * step_size)) == 0xdeadbeef + i as u32 {
            ram_size += step_size;
        } else {
            end_found = true;
        }
        write_volatile(ram.add(i * step_size), buffer[i]);
    }

    ram_size * size_of::<u32>()
}
