//! A two-block identifier cache for targets with vector byte table lookups.
//! The cache follows Yuku's lexer (MIT, see `THIRD_PARTY_NOTICES`).

use swc_next_simd::prelude::*;

use super::{
    TokenTag, get_keyword_type, is_ascii_identifier_continue, lexeme_mask,
    needs_full_identifier_scan, scan_ascii_identifier_or_keyword_slow,
};
use crate::lexer::keyword_type_of_words;

const BLOCK: u32 = 64;
/// Sentinel block offset for an empty cache: `start.wrapping_sub(UNINITIALIZED) >= BLOCK`
/// for every valid source offset, and `UNINITIALIZED.wrapping_add(BLOCK)` is not block-aligned.
const UNINITIALIZED: u32 = u32::MAX - BLOCK;

/// Two adjacent stop-bit masks indexed by absolute source offset. Rewinding the
/// lexer does not need to restore them: a scan outside the first block reloads.
#[derive(Clone, Debug)]
pub(in crate::lexer) struct IdentifierBlocks {
    start: u32,
    first: u64,
    second: u64,
}

impl IdentifierBlocks {
    #[inline(always)]
    pub(in crate::lexer) fn new(_source: &[u8]) -> Self {
        Self {
            start: UNINITIALIZED,
            first: 0,
            second: 0,
        }
    }

    #[inline(always)]
    pub(in crate::lexer) fn scan(&mut self, source: &[u8], start: u32) -> Option<(u32, TokenTag)> {
        let shift = start.wrapping_sub(self.start);
        if shift >= BLOCK {
            return self.scan_reloaded(source, start);
        }
        self.scan_in_blocks(source, start, shift)
    }

    #[inline(never)]
    fn scan_reloaded(&mut self, source: &[u8], start: u32) -> Option<(u32, TokenTag)> {
        let block = start & !(BLOCK - 1);
        self.reload(source, block);
        self.scan_in_blocks(source, start, start - block)
    }

    #[inline(always)]
    fn reload(&mut self, source: &[u8], block: u32) {
        self.first = if self.start.wrapping_add(BLOCK) == block {
            self.second
        } else {
            block_at(source, block as usize)
        };
        self.second = block_at(source, block as usize + BLOCK as usize);
        self.start = block;
    }

    #[inline(always)]
    fn scan_in_blocks(&self, source: &[u8], start: u32, shift: u32) -> Option<(u32, TokenTag)> {
        // Most identifiers end in the first block. Consult the second one
        // only when the first block has no stop at or after the start.
        let mut len = (self.first >> shift).trailing_zeros();
        if len == BLOCK {
            len = BLOCK - shift + self.second.trailing_zeros();
            if len >= BLOCK {
                return scan_ascii_identifier_or_keyword_slow(source, start);
            }
        }

        let end = start + len;
        if source
            .get(end as usize)
            .is_some_and(|&byte| needs_full_identifier_scan(byte))
        {
            return None;
        }

        let tail = &source[start as usize..];
        let tag = if let Some(bytes) = tail.first_chunk::<12>() {
            keyword_at(bytes, len as usize)
        } else {
            get_keyword_type(&tail[..len as usize])
        };
        Some((end, tag))
    }
}

#[inline(always)]
fn keyword_at(bytes: &[u8; 12], len: usize) -> TokenTag {
    const MASKS: [(u64, u32); 13] = {
        let mut masks = [(0, 0); 13];
        let mut len = 1;
        while len <= 12 {
            let mask = lexeme_mask(len);
            masks[len] = (mask as u64, (mask >> 64) as u32);
            len += 1;
        }
        masks
    };
    let (low_mask, high_mask) = MASKS[len.min(12)];
    let (low, high) = bytes.split_at(8);
    let low = u64::from_le_bytes(low.try_into().unwrap()) & low_mask;
    let high = u32::from_le_bytes(high.try_into().unwrap()) & high_mask;
    // A lexeme of at least 12 bytes leaves a nonzero twelfth byte. All keyword
    // entries have zero there, so long identifiers cannot match a keyword.
    keyword_type_of_words(low, high)
}

#[inline(always)]
fn block_at(source: &[u8], start: usize) -> u64 {
    let Some(tail) = source.get(start..) else {
        return u64::MAX;
    };
    if let Some(block) = tail.first_chunk::<{ BLOCK as usize }>() {
        return identifier_mask(block);
    }

    identifier_tail_mask(tail)
}

#[cold]
#[inline(never)]
fn identifier_tail_mask(tail: &[u8]) -> u64 {
    let mut padded = [0; BLOCK as usize];
    padded[..tail.len()].copy_from_slice(tail);
    identifier_mask(&padded)
}

/// A high nibble chooses one class bit; a low nibble lists the classes it
/// belongs to. High nibbles with the same set of low nibbles share a class.
struct NibbleClasses {
    low: [u8; 16],
    high: [u8; 16],
}

impl NibbleClasses {
    const fn identifiers() -> Self {
        let mut tables = Self {
            low: [0; 16],
            high: [0; 16],
        };
        let mut classes = [0u16; 8];
        let mut used = 0;
        let mut high = 0;
        while high < 16 {
            let mut members = 0u16;
            let mut low = 0;
            while low < 16 {
                if is_ascii_identifier_continue((high * 16 + low) as u8) {
                    members |= 1 << low;
                }
                low += 1;
            }
            if members != 0 {
                let mut class = 0;
                while class < used && classes[class] != members {
                    class += 1;
                }
                if class == used {
                    classes[class] = members;
                    used += 1;
                }
                let bit = 1 << class;
                tables.high[high] = bit;
                let mut low = 0;
                while low < 16 {
                    if members & (1 << low) != 0 {
                        tables.low[low] |= bit;
                    }
                    low += 1;
                }
            }
            high += 1;
        }
        tables
    }
}

#[inline(always)]
fn identifier_mask(block: &[u8; BLOCK as usize]) -> u64 {
    const CLASSES: NibbleClasses = NibbleClasses::identifiers();

    let low_table = u8x16::from_array(CLASSES.low);
    let high_table = u8x16::from_array(CLASSES.high);
    let classify = |chunk: &[u8; 16]| {
        let bytes = u8x16::from_array(*chunk);
        let low = low_table.swizzle_dyn(bytes & u8x16::splat(0x0f));
        let high = high_table.swizzle_dyn(bytes >> 4);
        (low & high).to_array()
    };
    let (chunks, _) = block.as_chunks::<16>();
    let classes = [
        classify(&chunks[0]),
        classify(&chunks[1]),
        classify(&chunks[2]),
        classify(&chunks[3]),
    ];
    u8x64::from_slice(classes.as_flattened())
        .simd_eq(u8x64::splat(0))
        .to_bitmask()
}
