//! Custom versions of routines used by LLVM (like memcpy, memset, etc.)
//!
//! These are normally part of the compiler-builtins crate. However the default routines
//! do not use word sized aligned instructions, which is slow and moreover leads to crashes
//! when using IRAM (which only allows aligned accesses).
//!
#[allow(warnings)]
#[cfg(target_pointer_width = "64")]
type c_int = u64;
#[allow(warnings)]
#[cfg(target_pointer_width = "16")]
type c_int = u16;
#[allow(warnings)]
#[cfg(not(any(target_pointer_width = "16", target_pointer_width = "64")))]
type c_int = u32;

const PTR_SIZE: usize = size_of::<c_int>();

use core::mem::size_of;
use core::slice::{from_raw_parts, from_raw_parts_mut};

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

extern "C" {
    #[link_name = "llvm.fshr.u32"]
    fn fshr_u32_backup2(a: u32, b: u32, n: u32) -> u32;
}

fn fshr_u32_backup(a: u32, b: u32, n: u32) -> u32 {
    let z;
    unsafe {
        asm!("
            ssr $3
            src $0,$1,$2
        "
            : "=r" (z)
            : "r" (a), "r" (b), "r" (n)
        );
    };
    z
}

fn fshr_u32(a: u32, b: u32, n: u32) -> u32 {
    (a << (32 - n)) | (b >> n)
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcpy_fast_chunk(mut dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // fast paths for small n
    if core::intrinsics::likely(n == PTR_SIZE) {
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

    if dst_align == src_align {
        // handle copy with same alignment offset

        let min_align = core::cmp::min(dst_align, src_align);
        let core = dst.wrapping_add(min_align);

        // fill initial non-aligned bytes
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
        const CHUNK_SIZES: [usize; 3] = [32, 4, 1];
        for chunk_size in &CHUNK_SIZES {
            while dst <= end.wrapping_sub(*chunk_size * PTR_SIZE) {
                // keep loop as tight as possible for maximum speed
                for i in 0..*chunk_size {
                    *(dst as *mut c_int).wrapping_add(i) =
                        *(dst.wrapping_add(src_off) as *mut c_int).wrapping_add(i);
                }
                dst = dst.wrapping_add(*chunk_size * PTR_SIZE);
            }
        }

        // fill final non-aligned bytes
        while dst < end {
            *dst = *dst.wrapping_add(src_off);
            dst = dst.wrapping_add(1);
        }

        return dst;
    } else {
        // handle copy with different alignment offset

        let core = dst.wrapping_add(dst_align);

        // fill initial non-aligned bytes
        while dst < core {
            *dst = *dst.wrapping_add(src_off);
            dst = dst.wrapping_add(1);
            if dst >= end {
                return dst;
            }
        }

        //     src.wrapping_add(dst_align).align_offset(PTR_SIZE);

        let mut prev_source = *(dst.wrapping_add(src_off) as *mut c_int);

        let off = (src_align - dst_align) * 8;

        // copy the core bytes in fixed size chunks: this allows loop unrolling
        // don't use too many different chunk sizes: need to fall through them before reaching
        // single c_int
        const CHUNK_SIZES: [usize; 3] = [32, 4, 1];
        for chunk_size in &CHUNK_SIZES {
            while dst <= end.wrapping_sub(*chunk_size * PTR_SIZE) {
                // keep loop as tight as possible for maximum speed
                for i in 0..*chunk_size {
                    let source = *(dst.wrapping_add(src_off) as *mut c_int).wrapping_add(i);
                    *(dst as *mut c_int).wrapping_add(i) =
                        fshr_u32(prev_source, source, off as u32);
                    //                    *(dst as *mut c_int).wrapping_add(i) =
                    //                        (((prev_source as u32) << (32 - off)) | ((source as u32) >> off)) as i32;
                    prev_source = source;
                }
                dst = dst.wrapping_add(*chunk_size * PTR_SIZE);
            }
        }

        return dst;
    }
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
pub unsafe extern "C" fn memset_old(s: *mut u8, c: c_int, n: usize) -> *mut u8 {
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
pub unsafe extern "C" fn memset_old2(s: *mut u8, c: c_int, n: usize) -> *mut u8 {
    // get unaligned pre- and postfix, set these as bytes, core as c_int
    let (prefix, mut core, postfix) = core::slice::from_raw_parts_mut(s, n).align_to_mut::<c_int>();

    for x in prefix {
        *x = c as u8;
    }

    const CHUNK_SIZES: [usize; 3] = [64, 8, 1];

    // Handle the core in chunk sizes: this allows loop unrolling, which greatly improves speed.
    // On xtensa-lx6 a loop needs at minimum 2 instructions (add and conditional jump) + 2 cycles
    // branch overhead. So when copying per single word only 20% is actual write instructions,
    // with 64 words per loop this is ~ 90% (extra instruction needed for long jump)
    for chunk_size in &CHUNK_SIZES {
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
pub unsafe extern "C" fn memset(mut s: *mut u8, c: c_int, n: usize) -> *mut u8 {
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

    // copy the core bytes in fixed size chunks: this allows loop unrolling
    // don't use too many different chunk sizes: need to fall through them before reaching
    // single c_int
    const CHUNK_SIZES: [usize; 3] = [64, 8, 1];
    for chunk_size in &CHUNK_SIZES {
        while s <= end.wrapping_sub(*chunk_size * PTR_SIZE) {
            for i in 0..*chunk_size {
                *(s as *mut c_int).wrapping_add(i) = c_int::from_ne_bytes([c as u8; PTR_SIZE]);
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

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcpy_chunk(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let src_off = (src as usize).wrapping_sub(dst as usize);
    let dst = from_raw_parts_mut(dst, n);

    // get unaligned pre- and postfix, set these as bytes, core as c_int
    let (dst_prefix, mut dst_core, dst_postfix) = dst.align_to_mut::<c_int>();

    for d in dst_prefix {
        *d = *(d as *const u8).wrapping_add(src_off);
    }

    const CHUNK_SIZES: [usize; 6] = [32, 16, 8, 4, 2, 1];

    for chunk_size in &CHUNK_SIZES {
        let mut chunks = dst_core.chunks_exact_mut(*chunk_size);
        for chunk in chunks.by_ref() {
            let source =
                ((&chunk[0] as *const u32 as *const u8).wrapping_add(src_off)) as *const c_int;
            for i in 0..*chunk_size {
                chunk[i] = *source.add(i);
            }
        }
        dst_core = chunks.into_remainder();
    }
    for d in dst_postfix {
        *d = *(d as *const u8).wrapping_add(src_off);
    }

    dst.as_mut_ptr()
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
