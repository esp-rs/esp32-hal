//! Custom versions of routines used by LLVM (like memcpy, memset, etc.)
//!
//! These are normally part of the compiler-builtins crate. However the default routines
//! do not use word sized aligned instructions, which is slow and moreover leads to crashes
//! when using IRAM (which only allows aligned accesses).
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
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;

    if n > PTR_SIZE {
        let start_bytes_dst = (dst as usize).wrapping_neg() % PTR_SIZE;
        let start_bytes_src = (src as usize).wrapping_neg() % PTR_SIZE;

        // copy initial bytes to make either src or dst aligned
        while i < n && i < start_bytes_dst && i < start_bytes_src {
            *dst.offset(i as isize) = *src.offset(i as isize);
            i += 1;
        }

        // copy per word
        while i <= n - PTR_SIZE {
            *(dst.offset(i as isize) as *mut c_int) = *(src.offset(i as isize) as *mut c_int);
            i += PTR_SIZE;
        }
    }

    // copy remaining bytes
    while i < n {
        *dst.offset(i as isize) = *src.offset(i as isize);
        i += 1;
    }
    dst
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcpy_reverse(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = n;

    if n > PTR_SIZE {
        let end_byte_dst = n - ((dst as usize) + n) % PTR_SIZE;
        let end_byte_src = n - ((src as usize) + n) % PTR_SIZE;

        // copy initial bytes to make either src or dst aligned
        while i != 0 && i > end_byte_dst && i > end_byte_src {
            i -= 1;
            *dst.offset(i as isize) = *src.offset(i as isize);
        }

        // copy per word
        while i >= PTR_SIZE {
            i -= PTR_SIZE;
            *(dst.offset(i as isize) as *mut c_int) = *(src.offset(i as isize) as *mut c_int);
        }
    }

    // copy per byte
    while i != 0 {
        i -= 1;
        *dst.offset(i as isize) = *src.offset(i as isize);
    }
    dst
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if src < dst as *const u8 {
        memcpy_reverse(dst, src, n)
    } else {
        memcpy(dst, src, n)
    }
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memset2(s: *mut u8, c: c_int, n: usize) -> *mut u8 {
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
pub unsafe extern "C" fn memset3(mut s: *mut u8, c: c_int, n: usize) -> *mut u8 {
    let end = s.add(n);

    const LOOPS: usize = 64;
    const LOOP_SIZE: usize = LOOPS * PTR_SIZE;

    // fill initial non-aligned bytes
    while s < end && (s as usize) % (LOOP_SIZE) != 0 {
        *s = c as u8;
        s = s.offset(1);
    }

    if s < end {
        while s <= end.sub(LOOP_SIZE) {
            for _ in 0..LOOPS {
                *(s as *mut c_int) = c_int::from_ne_bytes([c as u8; PTR_SIZE]);
                s = s.add(PTR_SIZE);
            }
        }

        // fill remaining non-aligned bytes
        while s < end {
            *s = c as u8;
            s = s.offset(1);
        }
    }

    s
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8 {
    let (prefix, mut core, postfix) = core::slice::from_raw_parts_mut(s, n).align_to_mut::<c_int>();

    for x in prefix {
        *x = c as u8;
    }

    for chunk_size in &[64, 32, 16, 8, 4, 2, 1] {
        let mut chunks = core.chunks_exact_mut(*chunk_size);
        for chunk in chunks.by_ref() {
            for i in 0..*chunk_size {
                chunk[i] = c_int::from_ne_bytes([c as u8; PTR_SIZE]);
            }
        }
        core = chunks.into_remainder();
    }

    for x in postfix {
        *x = c as u8;
    }

    s
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;

    // copy per word (not necessarily aligned: does not matter for performance on esp32)
    while i + PTR_SIZE <= n {
        let a = *(s1.offset(i as isize) as *const [u8; PTR_SIZE]);
        let b = *(s2.offset(i as isize) as *const [u8; PTR_SIZE]);
        for i in 0..=3 {
            if a[i] != b[i] {
                return a[i] as i32 - b[i] as i32;
            }
        }
        i += PTR_SIZE;
    }

    // compare per byte
    while i < n {
        let a = *s1.offset(i as isize);
        let b = *s2.offset(i as isize);
        if a != b {
            return a as i32 - b as i32;
        }
        i += 1;
    }
    0
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    memcmp(s1, s2, n)
}
