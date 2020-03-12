// Based on a research done by Wojciech Muła and Daniel Lemire
// https://arxiv.org/abs/1704.00605
// Copyright (c) 2015-2016, Wojciech Muła, Alfred Klomp, Daniel Lemire
// All rights reserved.
// Licensed under BSD 2-Clause (see LICENSES/fastbase64)

use super::scalar;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::cast_ptr_alignment, clippy::unreadable_literal)
)]
#[inline(always)]
/// SAFETY: the caller must ensure that buf can hold AT LEAST (s.len() * 3 / 4) more elements
pub(crate) unsafe fn decode(s: &[u8], buf: &mut Vec<u8>) -> Result<(), &'static str> {
    let mut len = s.len();
    let mut out_len = buf.len();

    let mut ptr = s.as_ptr();
    let mut out_ptr = buf[out_len..].as_mut_ptr();

    let lut_lo = _mm_setr_epi8(
        0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x10, 0x13, 0x1b, 0x1b, 0x1b, 0x1b, 0x1b,
    );
    let lut_hi = _mm_setr_epi8(
        0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
    );
    let lut_roll = _mm_setr_epi8(0, 22, 22, 4, -39, -39, -97, -97, 0, 0, 0, 0, 0, 0, 0, 0);

    let mask_lo_nibble = _mm_set1_epi8(0x0f);

    while len >= 22 {
        let src = _mm_loadu_si128(ptr as *const _);
        let hi_nibbles = _mm_and_si128(_mm_srli_epi32(src, 4), mask_lo_nibble);
        let lo_nibbles = _mm_and_si128(src, mask_lo_nibble);
        let lo = _mm_shuffle_epi8(lut_lo, lo_nibbles);
        let hi = _mm_shuffle_epi8(lut_hi, hi_nibbles);
        let roll = _mm_shuffle_epi8(lut_roll, hi_nibbles);

        if _mm_testz_si128(lo, hi) == 0 {
            return Err("failed to decode base64");
        }

        let merged = _mm_maddubs_epi16(_mm_add_epi8(src, roll), _mm_set1_epi32(0x40014001));
        let swapped = _mm_madd_epi16(merged, _mm_set1_epi32(0x10000001));
        let shuffled = _mm_shuffle_epi8(
            swapped,
            _mm_setr_epi8(0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, -1, -1, -1, -1),
        );
        _mm_storeu_si128(out_ptr as *mut _, shuffled);
        out_ptr = out_ptr.add(12);
        out_len += 12;

        len -= 16;
        ptr = ptr.add(16);
    }
    buf.set_len(out_len);

    scalar::decode(core::slice::from_raw_parts(ptr, len), buf)
}
