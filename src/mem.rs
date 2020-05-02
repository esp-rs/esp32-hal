//! Routines used by LLVM, these are normally part of the compiler-builtins crate.
//! However these routines do not use word sized aligned instructions, which leads to
//! problems with using teh IRAM (and is inefficient)
//!
//! #TODO:
//! - Optimize for non-aligned data: memcpy, memmove, memcmp (memset already done)
//!
#[allow(warnings)]
#[cfg(target_pointer_width = "16")]
type c_int = i16;
#[allow(warnings)]
#[cfg(not(target_pointer_width = "16"))]
type c_int = i32;

const PTR_SIZE: usize = size_of::<c_int>();

use core::mem::size_of;

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    if n % PTR_SIZE == 0 {
        // copy per c_int if n is multiple of c_int size
        // (not considering alignment, but if src and dst are aligned the accesses are as well)
        while i * PTR_SIZE < n {
            *(dest as *mut c_int).offset(i as isize) = *(src as *mut c_int).offset(i as isize);
            i += 1;
        }
    } else {
        // copy per byte
        while i < n {
            *dest.offset(i as isize) = *src.offset(i as isize);
            i += 1;
        }
    }
    dest
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcpy_reverse(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if n % PTR_SIZE == 0 {
        // copy per c_int if n is multiple of c_int size
        // (not considering alignment, but if src and dst are aligned the accesses are as well)
        let mut i = n / PTR_SIZE;
        while i != 0 {
            i -= 1;
            *(dest as *mut c_int).offset(i as isize) = *(src as *mut c_int).offset(i as isize);
        }
    } else {
        // copy per byte
        let mut i = n;
        while i != 0 {
            i -= 1;
            *dest.offset(i as isize) = *src.offset(i as isize);
        }
    }
    dest
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if src < dest as *const u8 {
        memcpy_reverse(dest, src, n)
    } else {
        memcpy(dest, src, n)
    }
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8 {
    let start_bytes = (s as usize).wrapping_neg() % PTR_SIZE;
    let mut i = 0;

    // fill initial non-aligned bytes
    while i < start_bytes && i < n {
        *s.offset(i as isize) = c as u8;
        i += 1;
    }

    if i < n {
        let end_c_int = n - ((s as usize + n) % PTR_SIZE);
        // fill aligned in c_int sized steps
        while i < end_c_int {
            *(s.offset(i as isize) as *mut c_int) = c_int::from_ne_bytes([c as u8; PTR_SIZE]);
            i += PTR_SIZE;
        }
        // fill remaining non-aligned bytes
        while i < n {
            *s.offset(i as isize) = c as u8;
            i += 1;
        }
    }

    s
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    if n % PTR_SIZE == 0 {
        // compare per c_int if n is multiple of c_int size
        // (not considering alignment, but if src and dst are aligned the accesses are as well)
        while i * PTR_SIZE < n {
            let a = *(s1 as *const [u8; PTR_SIZE]).offset(i as isize);
            let b = *(s2 as *const [u8; PTR_SIZE]).offset(i as isize);
            for i in 0..=3 {
                if a[i] != b[i] {
                    return a[i] as i32 - b[i] as i32;
                }
            }
            i += 1;
        }
    } else {
        // compare per byte
        while i < n {
            let a = *s1.offset(i as isize);
            let b = *s2.offset(i as isize);
            if a != b {
                return a as i32 - b as i32;
            }
            i += 1;
        }
    }
    0
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    memcmp(s1, s2, n)
}