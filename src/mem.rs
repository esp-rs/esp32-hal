//! Custom versions of routines used by LLVM (like memcpy, memset, etc.)
//!
//! These are normally part of the compiler-builtins crate. However the default routines
//! do not use word sized aligned instructions, which is slow and moreover leads to crashes
//! when using memories/processors which only allows aligned accesses.
//!
//! Implementation is optimized for large blocks of data. Assumption is that for small data,
//! they are inlined by the compiler. Some optimization done for often used small sizes as
//! otherwise significant slowdown in debug mode.
//!
//! Implementation is optimized when dst/s1 and src/s2 have the same alignment.
//! If alignment of s1 and s2 is unequal, then either s1 or s2 accesses are not aligned
//! resulting in slower performance. (If s1 or s2 is aligned, then those accesses are aligned.)
//!
//! Further optimization is possible by having a dedicated code path for unaligned accesses,
//! which uses 2*PTR_SIZE to PTR_SIZE shift operation (i.e. llvm.fshr);
//! but implementation of this intrinsic is not yet optimized and currently leads to worst results.
//!
//! Also loop unrolling in the memcpy_reverse function is not fully optimal due to limited current
//! llvm optimization: uses add with negative offset + store, instead of store with positive
//! offset; so 3 instructions per loop instead of 2
//!
//! A further future optimization possibility is using zero overhead loop, but again
//! currently not yet supported by llvm for xtensa.
//!
//! For large aligned memset and memcpy reaches ~88% of maximum memory bandwidth;
//! for memcpy_reverse ~60%.
#[allow(warnings)]
#[cfg(target_pointer_width = "64")]
type c_int = u64;
#[allow(warnings)]
#[cfg(target_pointer_width = "16")]
type c_int = u16;
#[allow(warnings)]
#[cfg(not(any(target_pointer_width = "16", target_pointer_width = "64")))]
type c_int = u32;

use core::mem::size_of;

const PTR_SIZE: usize = size_of::<c_int>();

// below CHUNK sizes
const CHUNK_SIZES_MEMCPY: [usize; 3] = [32, 4, 1]; // bigger chunks will not be unrolled
const CHUNK_SIZES_MEMSET: [usize; 3] = [64, 8, 1]; // bigger chunks will not be unrolled
const CHUNK_SIZES_MEMCMP: [usize; 2] = [8, 1]; // bigger chunks require long jumps

/// Copies n-bytes of data from src to dst
///
/// If data overlaps and src < dst, the data will be corrupted.
#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcpy(mut dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // fast paths for small n
    if n == PTR_SIZE {
        *(dst as *mut c_int) = *(src as *mut c_int);
        return dst;
    } else if PTR_SIZE > 4 && n == 4 {
        *(dst as *mut u32) = *(src as *mut u32);
        return dst;
    } else if PTR_SIZE > 2 && n == 2 {
        *(dst as *mut u16) = *(src as *mut u16);
        return dst;
    } else if PTR_SIZE > 1 && n == 1 {
        *(dst as *mut u8) = *(src as *mut u8);
        return dst;
    } else if PTR_SIZE > 3 && n == 3 {
        *(dst as *mut u16) = *(src as *mut u16);
        *dst.wrapping_add(2) = *src.wrapping_add(2);
        return dst;
    }

    let src_off = (src as usize).wrapping_sub(dst as usize);

    // select minimum of src or dst alignment
    let dst_align = dst.align_offset(PTR_SIZE);
    let src_align = src.align_offset(PTR_SIZE);

    let end = dst.wrapping_add(n);

    let min_align = core::cmp::min(dst_align, src_align);
    let core = dst.wrapping_add(min_align);

    // copy initial non-aligned bytes
    while dst < core {
        *dst = *dst.wrapping_add(src_off);
        dst = dst.wrapping_add(1);
        if dst >= end {
            return dst;
        }
    }

    // copy the core bytes in fixed size chunks: this allows loop unrolling
    // don't use too many different chunk sizes: need to fall through them before reaching
    // single c_int
    for chunk_size in &CHUNK_SIZES_MEMCPY {
        while dst <= end.wrapping_sub(*chunk_size * PTR_SIZE) {
            // keep loop as tight as possible for maximum speed
            // while instead of for in range, as for in range calls memcpy in debug mode,
            // so we get infinite recursion
            let mut i = 0;
            while i < *chunk_size {
                *(dst as *mut c_int).wrapping_add(i) =
                    *(dst.wrapping_add(src_off) as *mut c_int).wrapping_add(i);
                i = i + 1;
            }
            dst = dst.wrapping_add(*chunk_size * PTR_SIZE);
        }
    }

    // copy final non-aligned bytes
    while dst < end {
        *dst = *dst.wrapping_add(src_off);
        dst = dst.wrapping_add(1);
    }

    return dst;
}

/// Copies n-bytes of data from src to dst
///
/// If data overlaps and src > dst, the data will be corrupted.
#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcpy_reverse(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // fast paths for small n
    if n == PTR_SIZE {
        *(dst as *mut c_int) = *(src as *mut c_int);
        return dst;
    } else if PTR_SIZE > 4 && n == 4 {
        *(dst as *mut u32) = *(src as *mut u32);
        return dst;
    } else if PTR_SIZE > 2 && n == 2 {
        *(dst as *mut u16) = *(src as *mut u16);
        return dst;
    } else if PTR_SIZE > 1 && n == 1 {
        *(dst as *mut u8) = *(src as *mut u8);
        return dst;
    } else if PTR_SIZE > 3 && n == 3 {
        *dst.wrapping_add(2) = *src.wrapping_add(2);
        *(dst as *mut u16) = *(src as *mut u16);
        return dst;
    }

    let src_off = (src as usize).wrapping_sub(dst as usize);

    // select minimum of src or dst alignment
    let dst_end_align = (dst.wrapping_add(n) as usize) % PTR_SIZE;
    let src_end_align = (src.wrapping_add(n) as usize) % PTR_SIZE;

    let min_align = core::cmp::min(dst_end_align, src_end_align);
    let core = dst.wrapping_add(n).wrapping_sub(min_align);

    let mut cur = dst.wrapping_add(n);

    // copy initial non-aligned bytes
    while cur > core {
        cur = cur.wrapping_sub(1);
        *cur = *cur.wrapping_add(src_off);
        if cur == dst {
            return dst;
        }
    }

    // copy the core bytes in fixed size chunks: this allows loop unrolling
    // don't use too many different chunk sizes: need to fall through them before reaching
    // single c_int
    for chunk_size in &CHUNK_SIZES_MEMCPY {
        while cur >= dst.wrapping_add(*chunk_size * PTR_SIZE) {
            cur = cur.wrapping_sub(*chunk_size * PTR_SIZE);

            // keep loop as tight as possible for maximum speed
            // while instead of for in range, as for in range calls memcpy in debug mode,
            // so we get infinite recursion
            //
            // current llvm optimization is not perfect: uses add with negative offset + store,
            // instead of store with positive offset; so 3 instructions per loop instead of 2
            let mut i = *chunk_size;
            while i > 0 {
                i = i - 1;
                *(cur as *mut c_int).wrapping_add(i) =
                    *(cur.wrapping_add(src_off) as *mut c_int).wrapping_add(i);
            }
        }
    }

    // copy final non-aligned bytes
    while cur > dst {
        cur = cur.wrapping_sub(1);
        *cur = *cur.wrapping_add(src_off);
    }

    return dst;
}

/// Copies n-bytes of data from src to dst and properly handles overlapping data
#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if src < dst as *const u8 {
        memcpy_reverse(dst, src, n)
    } else {
        memcpy(dst, src, n)
    }
}

/// Fills n-bytes with byte sized value
#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memset(mut s: *mut u8, c: c_int, n: usize) -> *mut u8 {
    // fast paths for small n
    if n == PTR_SIZE {
        *(s as *mut c_int) = c_int::from_ne_bytes([c as u8; PTR_SIZE]);
        return s;
    } else if PTR_SIZE > 4 && n == 4 {
        *(s as *mut u32) = u32::from_ne_bytes([c as u8; 4]);
        return s;
    } else if PTR_SIZE > 2 && n == 2 {
        *(s as *mut u16) = u16::from_ne_bytes([c as u8; 2]);
        return s;
    } else if PTR_SIZE > 1 && n == 1 {
        *(s as *mut u8) = c as u8;
        return s;
    } else if PTR_SIZE > 3 && n == 3 {
        *(s as *mut u16) = u16::from_ne_bytes([c as u8; 2]);
        *s.wrapping_add(2) = c as u8;
        return s;
    }

    let end = s.wrapping_add(n);
    let core = s.wrapping_add(s.align_offset(PTR_SIZE));

    // fill initial non-aligned bytes
    while s < core {
        *s = c as u8;
        s = s.wrapping_add(1);
        if s >= end {
            return s;
        }
    }

    // set the core bytes in fixed size chunks: this allows loop unrolling
    // don't use too many different chunk sizes: need to fall through them before reaching
    // single c_int
    for chunk_size in &CHUNK_SIZES_MEMSET {
        while s <= end.wrapping_sub(*chunk_size * PTR_SIZE) {
            // while instead of for in range, as for in range calls memcpy in debug mode,
            // so we get infinite recursion
            let mut i = 0;
            while i < *chunk_size {
                *(s as *mut c_int).wrapping_add(i) = c_int::from_ne_bytes([c as u8; PTR_SIZE]);
                i = i + 1;
            }
            s = s.wrapping_add(*chunk_size * PTR_SIZE);
        }
    }

    // fill the final non-aligned bytes
    while s < end {
        *s = c as u8;
        s = s.wrapping_add(1);
    }

    s
}

/// Compare n-bytes of data from s1 and s2 and returns <0 for s1<s2, 0 for s1=s2 and >0 for s1>s2
#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcmp(mut s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let s2_off = (s2 as usize).wrapping_sub(s1 as usize);

    let end = s1.wrapping_add(n);

    // fast path for small n
    if n <= PTR_SIZE {
        while s1 < end {
            let res = *s1 - *s1.wrapping_add(s2_off);
            if res != 0 {
                return res as i8 as i32;
            }
            s1 = s1.wrapping_add(1);
        }
        return 0;
    }

    // select minimum of s2 or s1 alignment
    let s1_align = s1.align_offset(PTR_SIZE);
    let s2_align = s2.align_offset(PTR_SIZE);

    let min_align = core::cmp::min(s1_align, s2_align);
    let core = s1.wrapping_add(min_align);

    // compare initial non-aligned bytes
    while s1 < core {
        let res = *s1 - *s1.wrapping_add(s2_off);
        if res != 0 {
            return res as i8 as i32;
        }
        s1 = s1.wrapping_add(1);
        if s1 >= end {
            return 0;
        }
    }

    // compare the core bytes in fixed size chunks: this allows loop unrolling
    // don't use too many different chunk sizes: need to fall through them before reaching
    // single c_int
    for chunk_size in &CHUNK_SIZES_MEMCMP {
        while s1 <= end.wrapping_sub(*chunk_size * PTR_SIZE) {
            // keep loop as tight as possible for maximum speed
            // while instead of for in range, as for in range calls memcpy in debug mode,
            // so we get infinite recursion
            let mut i = 0;
            while i < *chunk_size {
                let res = *(s1 as *mut c_int).wrapping_add(i)
                    - *(s1.wrapping_add(s2_off) as *mut c_int).wrapping_add(i);
                if res != 0 {
                    let mut j = 0;
                    while j < 4 {
                        let res = *((s1 as *mut c_int).wrapping_add(i) as *mut u8).wrapping_add(j)
                            - *((s1.wrapping_add(s2_off) as *mut c_int).wrapping_add(i) as *mut u8)
                                .wrapping_add(j);
                        if res != 0 {
                            return res as i8 as i32;
                        }
                        j = j + 1;
                    }
                    unreachable!();
                }
                i = i + 1;
            }
            s1 = s1.wrapping_add(*chunk_size * PTR_SIZE);
        }
    }

    // compare final non-aligned bytes
    while s1 < end {
        let res = *s1 - *s1.wrapping_add(s2_off);
        if res != 0 {
            return res as i8 as i32;
        }
        s1 = s1.wrapping_add(1);
    }

    return 0;
}

/// Compare n-bytes of data from s1 and s2 and returns 0 for s1==s2 and !=0 otherwise
#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn bcmp(mut s1: *const u8, s2: *const u8, n: usize) -> i32 {
    // fast paths for small n
    if n == PTR_SIZE {
        return (*(s1 as *const c_int) != *(s2 as *const c_int)) as i32;
    } else if PTR_SIZE > 4 && n == 4 {
        return (*(s1 as *const u32) != *(s2 as *const u32)) as i32;
    } else if PTR_SIZE > 2 && n == 2 {
        return (*(s1 as *const u16) != *(s2 as *const u16)) as i32;
    } else if PTR_SIZE > 1 && n == 1 {
        return (*(s1 as *const u8) != *(s2 as *const u8)) as i32;
    } else if PTR_SIZE > 3 && n == 3 {
        return (*(s1 as *const u16) != *(s2 as *const u16)
            || *s1.wrapping_add(1) != *s2.wrapping_add(1)) as i32;
    }

    let s2_off = (s2 as usize).wrapping_sub(s1 as usize);

    // select minimum of s2 or s1 alignment
    let s1_align = s1.align_offset(PTR_SIZE);
    let s2_align = s2.align_offset(PTR_SIZE);

    let end = s1.wrapping_add(n);

    let min_align = core::cmp::min(s1_align, s2_align);
    let core = s1.wrapping_add(min_align);

    // compare initial non-aligned bytes
    while s1 < core {
        if *s1 != *s1.wrapping_add(s2_off) {
            return true as i32;
        }
        s1 = s1.wrapping_add(1);
        if s1 >= end {
            return false as i32;
        }
    }

    // compare the core bytes in fixed size chunks: this allows loop unrolling
    // don't use too many different chunk sizes: need to fall through them before reaching
    // single c_int
    for chunk_size in &CHUNK_SIZES_MEMCMP {
        while s1 <= end.wrapping_sub(*chunk_size * PTR_SIZE) {
            // keep loop as tight as possible for maximum speed
            // while instead of for in range, as for in range calls memcpy in debug mode,
            // so we get infinite recursion
            let mut i = 0;
            while i < *chunk_size {
                if *(s1 as *mut c_int).wrapping_add(i)
                    != *(s1.wrapping_add(s2_off) as *mut c_int).wrapping_add(i)
                {
                    return true as i32;
                }
                i = i + 1;
            }
            s1 = s1.wrapping_add(*chunk_size * PTR_SIZE);
        }
    }

    // compare final non-aligned bytes
    while s1 < end {
        if *s1 != *s1.wrapping_add(s2_off) {
            return true as i32;
        }
        s1 = s1.wrapping_add(1);
    }

    return false as i32;
}
