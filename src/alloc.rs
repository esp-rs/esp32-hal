use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ptr::NonNull;

use linked_list_allocator::Heap;

use spin::Mutex;
use xtensa_lx6_rt::interrupt;

#[global_allocator]
pub static DRAM_HEAP: LockedHeap = unsafe { LockedHeap::new(&_heap_start, &_heap_end) };

pub static IRAM_HEAP: LockedHeap = unsafe { LockedHeap::new(&_iram_heap_start, &_iram_heap_end) };

pub static EXTERNAL_HEAP: LockedHeap =
    unsafe { LockedHeap::new(&_external_heap_start, &_external_heap_end) };

// These symbols come from `memory.x`
extern "C" {
    static _heap_start: u32;
    static _heap_end: u32;
    static _iram_heap_start: u32;
    static _iram_heap_end: u32;
    static _external_heap_start: u32;
    static _external_heap_end: u32;
}

pub struct LockedHeap {
    heap: RefCell<Option<Mutex<Heap>>>,
    start_addr: *const u32,
    end_addr: *const u32,
}

unsafe impl Sync for LockedHeap {}

impl LockedHeap {
    /// Create a new heap allocator
    ///
    /// `start_addr` is the address where the heap will be located.
    /// `end_addr` is the address after the heap
    /// The heap uses the memory range [start_addr, end_addr)
    pub const fn new(start_addr: *const u32, end_addr: *const u32) -> LockedHeap {
        LockedHeap {
            heap: RefCell::new(None),
            start_addr,
            end_addr,
        }
    }
}
unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        interrupt::free(|_| {
            if self.heap.borrow().as_ref().is_none() {
                self.heap.replace(Some(Mutex::new(Heap::new(
                    self.start_addr as usize,
                    self.end_addr as usize - self.start_addr as usize,
                ))));
            }
            self.heap
                .borrow()
                .as_ref()
                .unwrap()
                .lock()
                .allocate_first_fit(layout)
                .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        interrupt::free(|_| {
            self.heap
                .borrow()
                .as_ref()
                .unwrap()
                .lock()
                .deallocate(NonNull::new_unchecked(ptr), layout)
        });
    }
}
