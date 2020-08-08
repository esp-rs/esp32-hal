//! Memory allocators for various memory types
//!
//! The allocators are thread and interrupt safe. (By blocking interrupts and using a spin type mutex)
//!
//! The allocators can be safely used in a mixed fashion. (Including multiple GeneralAllocators
//! with different thresholds)
//!
//! **NOTE: iram can only be accessed by aligned 32-bit accesses, as structures can indicate
//! alignment >= 4 even when members are smaller, it cannot be used for general rust allocations.
//! (Unless a load/store exception handler is used, but this will be very slow.)
//!
//! **NOTE: the default implementations of memcpy, memset etc. which are used behind the scenes use
//! unaligned accesses.** This causes exceptions when used together with IRAM.
//! The replacements in the mem module do handle alignment properly. They can be enable by
//! including the following in Cargo.toml:
//! ```
//! [package.metadata.cargo-xbuild]
//! memcpy = false
//! [features]
//! mem=[]
//! ```
//!
//!
//! # TODO:
//! - Improve underlying heap allocator: support for realloc, speed etc.
//!
use crate::prelude::*;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

use linked_list_allocator::Heap;

const DEFAULT_EXTERNAL_THRESHOLD: usize = 32 * 1024;
const DEFAULT_USE_IRAM: bool = false;

/// Default allocator using a mix of memories.
///
/// It will use external RAM for all blocks larger then 32kBytes.
///
/// It will use IRAM for 4-byte aligned requests.
///
/// It will use DRAM for the remaining requests.
///
/// If the default memory type is not available it will fall back to DRAM followed by external RAM.
///
/// *NOTE: the default implementations of memcpy, memset etc. which are used behind the scenes use
/// unaligned accesses. The replacements in the mem module do handle alignment properly.*
pub static DEFAULT_ALLOCATOR: Allocator = Allocator::new(&DEFAULT_HEAP);

/// Heap allocator using the DRAM.
///
/// DRAM is most flexible, it can be used for non-aligned and byte accesses and DMA and it allows atomic access.
pub static DRAM_ALLOCATOR: Allocator = Allocator::new(&DRAM_HEAP);

/// Heap allocator using the IRAM.
///
/// IRAM only supports aligned 4-byte data, it also allows atomic access, but no DMA
///
/// *NOTE: the default implementations of memcpy, memset etc. which are used behind the scenes use
/// unaligned accesses. The replacements in the mem module do handle alignment properly.*
pub static IRAM_ALLOCATOR: Allocator = Allocator::new(&IRAM_HEAP);

/// Heap allocator using the external RAM
///
/// External RAM can be used non-aligned and byte accesses, but DMA and atomic access are not supported
#[cfg(feature = "external_ram")]
pub static EXTERNAL_ALLOCATOR: Allocator = Allocator::new(&EXTERNAL_HEAP);

// These symbols come from `memory.x`
extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
    static _text_heap_start: u8;
    static _text_heap_end: u8;
    static _external_heap_start: u8;
    static _external_heap_end: u8;
}

static DEFAULT_HEAP: GeneralAllocator =
    GeneralAllocator::new(DEFAULT_EXTERNAL_THRESHOLD, DEFAULT_USE_IRAM);

#[allow(dead_code)]
static DRAM_HEAP: LockedHeap = unsafe { LockedHeap::new(&|| &_heap_start, &|| &_heap_end) };

#[allow(dead_code)]
static IRAM_HEAP: LockedHeap =
    unsafe { LockedHeap::new(&|| &_text_heap_start, &|| &_text_heap_end) };

#[allow(dead_code)]
#[cfg(feature = "external_ram")]
static EXTERNAL_HEAP: LockedHeap = unsafe {
    LockedHeap::new(&|| &_external_heap_start, &|| {
        core::cmp::min(
            &_external_heap_end,
            (&_external_heap_start as *const u8).add(crate::external_ram::get_size()),
        )
    })
};

/// Get heap sizes
pub trait AllocatorSize {
    /// Get total heap size
    fn size(&self) -> usize;
    /// Get used heap size
    fn used(&self) -> usize;
    /// Get free heap size
    fn free(&self) -> usize;
}

unsafe trait GlobalAllocSize: GlobalAlloc + AllocatorSize {}

#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct Allocator {
    allocator: &'static (dyn GlobalAllocSize + 'static),
}

unsafe impl Sync for Allocator {}

impl Allocator {
    const fn new(allocator: &'static dyn GlobalAllocSize) -> Self {
        Self { allocator }
    }
}

unsafe impl GlobalAllocSize for Allocator {}

impl AllocatorSize for Allocator {
    fn size(&self) -> usize {
        self.allocator.size()
    }

    fn used(&self) -> usize {
        self.allocator.used()
    }

    fn free(&self) -> usize {
        self.allocator.free()
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocator.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.dealloc(ptr, layout)
    }
}

extern crate alloc;
use alloc::alloc::{AllocErr, AllocInit, AllocRef, MemoryBlock};

unsafe impl AllocRef for Allocator {
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        if layout.size() == 0 {
            return Ok(MemoryBlock {
                ptr: layout.dangling(),
                size: 0,
            });
        }
        let ptr = unsafe { GlobalAlloc::alloc(self, layout) };
        if ptr != 0 as *mut u8 {
            let block = MemoryBlock {
                ptr: NonNull::new(ptr).ok_or(AllocErr)?,
                size: layout.size(),
            };
            unsafe { init.init(block) };
            Ok(block)
        } else {
            Err(AllocErr)
        }
    }
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            GlobalAlloc::dealloc(self, ptr.as_ptr(), layout);
        }
    }
}

/// General allocator using a mix of all memories.
///
/// It will use external RAM for all blocks larger then the threshold specified during creation.
///
/// It will use IRAM for blocks larger than and aligned to 4 bytes or larger, when use_iram is enabled.
///
/// It will use DRAM for the remaining requests.
///
/// If the default memory type is not available it will fall back to DRAM followed by external RAM.
///
/// *NOTE when using IRAM: the default implementations of memcpy, memset etc. which are used behind the scenes use
/// unaligned accesses. Either care must be taken that such function are avoided
/// (e.g. by using uninitialized memory) or they need to be replaced.*

pub struct GeneralAllocator {
    #[cfg(feature = "external_ram")]
    external_threshold: usize,
    use_iram: bool,
}

unsafe impl Sync for GeneralAllocator {}

impl GeneralAllocator {
    /// Create a new general allocation with a specified threshold for external memory allocations.
    pub const fn new(_external_threshold: usize, use_iram: bool) -> Self {
        Self {
            #[cfg(feature = "external_ram")]
            external_threshold: _external_threshold,
            use_iram,
        }
    }
}

unsafe impl GlobalAllocSize for GeneralAllocator {}

impl AllocatorSize for GeneralAllocator {
    fn size(&self) -> usize {
        #[cfg(not(feature = "external_ram"))]
        let res = DRAM_ALLOCATOR.size() + IRAM_ALLOCATOR.size();

        #[cfg(feature = "external_ram")]
        let res = DRAM_ALLOCATOR.size() + IRAM_ALLOCATOR.size() + EXTERNAL_ALLOCATOR.size();

        res
    }

    fn used(&self) -> usize {
        #[cfg(not(feature = "external_ram"))]
        let res = DRAM_ALLOCATOR.used() + IRAM_ALLOCATOR.used();

        #[cfg(feature = "external_ram")]
        let res = DRAM_ALLOCATOR.used() + IRAM_ALLOCATOR.used() + EXTERNAL_ALLOCATOR.used();

        res
    }

    fn free(&self) -> usize {
        #[cfg(not(feature = "external_ram"))]
        let res = DRAM_ALLOCATOR.free() + IRAM_ALLOCATOR.free();

        #[cfg(feature = "external_ram")]
        let res = DRAM_ALLOCATOR.free() + IRAM_ALLOCATOR.free() + EXTERNAL_ALLOCATOR.free();

        res
    }
}

unsafe impl GlobalAlloc for GeneralAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // if bigger then threshold use external ram
        #[cfg(feature = "external_ram")]
        if layout.size() > self.external_threshold {
            let res = EXTERNAL_HEAP.alloc(layout);
            if res != 0 as *mut u8 {
                return res;
            }
        }

        // if IRAM usage allowed, aligned and sized correctly use IRAM
        if self.use_iram
            && layout.size() >= core::mem::size_of::<usize>()
            && layout.align() >= core::mem::size_of::<usize>()
        {
            let res = IRAM_ALLOCATOR.alloc(layout);
            if res != 0 as *mut u8 {
                return res;
            }
        }

        // if not external or IRAM then place in DRAM
        let res = DRAM_ALLOCATOR.alloc(layout);
        if res != 0 as *mut u8 {
            return res;
        }

        // use external ram as fallback
        #[cfg(feature = "external_ram")]
        return EXTERNAL_HEAP.alloc(layout);
        #[cfg(not(feature = "external_ram"))]
        return res;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match ptr as *const _ {
            x if DRAM_HEAP.is_in_range(x) => DRAM_HEAP.dealloc(ptr, layout),
            x if IRAM_HEAP.is_in_range(x) => IRAM_HEAP.dealloc(ptr, layout),
            #[cfg(feature = "external_ram")]
            x if EXTERNAL_HEAP.is_in_range(x) => EXTERNAL_HEAP.dealloc(ptr, layout),
            _ => (),
        }
    }
}

struct LockedHeap<'a> {
    heap: CriticalSectionSpinLockMutex<Option<Heap>>,
    start_addr: &'a dyn Fn() -> *const u8,
    end_addr: &'a dyn Fn() -> *const u8,
}

unsafe impl Sync for LockedHeap<'_> {}

unsafe impl GlobalAllocSize for LockedHeap<'_> {}

/// Multi-core and interrupt safe heap allocator with constant constructor
///
impl<'a> LockedHeap<'a> {
    /// Create a new heap allocator
    ///
    /// `start_addr` is the address where the heap will be located.
    /// `end_addr` is the address after the heap
    /// The heap uses the memory range [start_addr, end_addr)
    const fn new(
        start_addr: &'a dyn Fn() -> *const u8,
        end_addr: &'a dyn Fn() -> *const u8,
    ) -> Self {
        Self {
            heap: CriticalSectionSpinLockMutex::new(None),
            start_addr,
            end_addr,
        }
    }

    fn with_locked_heap<R>(&self, f: impl FnOnce(&mut Heap) -> R) -> R {
        (&self.heap).lock(|heap| match heap {
            None => {
                let start = (self.start_addr)() as usize;
                let size = (self.end_addr)() as usize - (self.start_addr)() as usize;
                let mut temp_heap = unsafe { Heap::new(start, size) };
                let res = f(&mut temp_heap);
                *heap = Some(temp_heap);
                res
            }
            Some(heap) => f(heap),
        })
    }

    fn is_in_range(&self, ptr: *const u8) -> bool {
        self.with_locked_heap(|heap| (ptr as usize) >= heap.bottom() && (ptr as usize) < heap.top())
    }
}

impl AllocatorSize for LockedHeap<'_> {
    fn size(&self) -> usize {
        self.with_locked_heap(|heap| heap.size())
    }

    fn used(&self) -> usize {
        self.with_locked_heap(|heap| heap.used())
    }

    fn free(&self) -> usize {
        self.with_locked_heap(|heap| heap.free())
    }
}

unsafe impl GlobalAlloc for LockedHeap<'_> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.with_locked_heap(|heap| {
            heap.allocate_first_fit(layout)
                .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.with_locked_heap(|heap| heap.deallocate(NonNull::new_unchecked(ptr), layout));
    }
}
