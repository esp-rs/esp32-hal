//! Functions related to memory management
//!

use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};

static EXTERNAL_RAM_SIZE: Option<usize> = None;

pub unsafe fn get_external_ram_size() -> usize {
    EXTERNAL_RAM_SIZE.unwrap_or(calc_external_ram_size())
}

const CACHE_LINE_SIZE: usize = 32;
const NR_CACHE_LINES: usize = 1024;
const RAM_STEPS: usize = 8;

unsafe fn calc_external_ram_size() -> usize {
    // These symbols come from `memory.x`
    extern "C" {
        static _external_ram_start: u32;
        static _external_ram_end: u32;
    }

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
