//! Search for individual bytes or ASCII byte classes, extending the `memchr` crate.

use crate::{cmp::SimdPartialEq, mask8x16, u8x16};

/// A set of ASCII bytes used to find delimiters in arbitrary UTF-8 or binary input.
pub struct AsciiByteSet {
    /// For each low nibble, one bit for each of the eight possible ASCII high nibbles.
    low: [u8; 16],
    scalar: [bool; 256],
}

impl AsciiByteSet {
    pub const fn new(bytes: &[u8]) -> Self {
        Self {
            low: [0; 16],
            scalar: [false; 256],
        }
        .with_bytes(bytes)
    }

    pub const fn with_bytes(mut self, bytes: &[u8]) -> Self {
        let mut i = 0;
        while i < bytes.len() {
            self = self.with_range(bytes[i], bytes[i]);
            i += 1;
        }
        self
    }

    pub const fn with_range(mut self, first: u8, last: u8) -> Self {
        assert!(first <= last && last.is_ascii());
        let mut byte = first;
        while byte <= last {
            self.low[(byte & 15) as usize] |= 1 << (byte >> 4);
            self.scalar[byte as usize] = true;
            byte += 1;
        }
        self
    }

    #[inline]
    pub const fn contains(&self, byte: u8) -> bool {
        self.scalar[byte as usize]
    }

    /// Returns the offset of the first member of this set.
    #[inline]
    pub fn find(&self, bytes: &[u8]) -> Option<usize> {
        let mut offset = 0;
        if cfg!(any(
            all(
                target_arch = "aarch64",
                target_endian = "little",
                target_feature = "neon"
            ),
            all(target_arch = "wasm32", target_feature = "simd128"),
        )) {
            let low = u8x16::from_array(self.low);
            let high = u8x16::from_array([1, 2, 4, 8, 16, 32, 64, 128, 0, 0, 0, 0, 0, 0, 0, 0]);
            for block in bytes.as_chunks::<16>().0 {
                let chunk = u8x16::from_array(*block);
                let mask = low.swizzle_dyn(chunk & u8x16::splat(15)) & high.swizzle_dyn(chunk >> 4);
                if let Some(lane) = mask.simd_ne(u8x16::splat(0)).first_set() {
                    return Some(offset + lane);
                }
                offset += 16;
            }
        }
        bytes[offset..]
            .iter()
            .position(|&byte| self.contains(byte))
            .map(|pos| offset + pos)
    }
}

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
