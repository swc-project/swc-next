//! Search for any of four bytes, extending the `memchr` crate.

use crate::{cmp::SimdPartialEq, mask8x16, u8x16};

/// Returns the byte offset of the first occurrence of any of the four needles in `haystack`.
#[inline]
pub fn memchr4(
    needle1: u8,
    needle2: u8,
    needle3: u8,
    needle4: u8,
    haystack: &[u8],
) -> Option<usize> {
    let needles = [needle1, needle2, needle3, needle4];
    let mut position = 0;
    while let Some(chunk) = haystack[position..].first_chunk::<16>() {
        let bytes = u8x16::from_array(*chunk);
        let mut matches = mask8x16::splat(false);
        for needle in needles {
            matches = matches | bytes.simd_eq(u8x16::splat(needle));
        }
        if let Some(lane) = matches.first_set() {
            return Some(position + lane);
        }
        position += 16;
    }
    haystack[position..]
        .iter()
        .position(|byte| needles.contains(byte))
        .map(|offset| position + offset)
}
