//! The two-block identifier cache and NEON mask reduction follow Yuku's lexer
//! (MIT, see `THIRD_PARTY_NOTICES`).

use core::arch::aarch64::{
    uint8x16_t, vandq_u8, vbicq_u8, vceqzq_u8, vdupq_n_u8, vgetq_lane_u64, vld1q_u8, vpaddq_u8,
    vqtbl1q_u8, vreinterpretq_u64_u8, vshrq_n_u8,
};

use super::{
    TokenTag, get_keyword_type, is_ascii_identifier_continue, lexeme_mask,
    needs_full_identifier_scan, scan_ascii_identifier_or_keyword_slow,
};
use crate::lexer::keyword_type_of_words;

const BLOCK: u32 = 64;

/// Two adjacent masks indexed by absolute source offset. Rewinding the lexer
/// does not need to restore them: a scan outside the first block reloads.
#[derive(Clone, Debug)]
pub(in crate::lexer) struct IdentifierBlocks {
    start: u32,
    first: u64,
    second: u64,
}

impl IdentifierBlocks {
    pub(in crate::lexer) fn new(source: &[u8]) -> Self {
        Self {
            start: 0,
            first: block_at(source, 0),
            second: block_at(source, BLOCK as usize),
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
    fn run_length(&self, shift: u32) -> u32 {
        // Shift the second mask in without shifting by 64 when `shift` is zero.
        let run = (self.first >> shift) | ((self.second << 1) << (BLOCK - 1 - shift));
        (!run).trailing_zeros()
    }

    #[inline(always)]
    fn scan_in_blocks(&self, source: &[u8], start: u32, shift: u32) -> Option<(u32, TokenTag)> {
        let len = self.run_length(shift);
        if len == BLOCK {
            return scan_ascii_identifier_or_keyword_slow(source, start);
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
        return 0;
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
    const LANE_BITS: [u8; 16] = [1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128];

    // SAFETY: This module is built only with NEON. The four unaligned loads
    // cover the 64-byte array, and each lookup table has 16 readable bytes.
    unsafe {
        let low = vld1q_u8(CLASSES.low.as_ptr());
        let high = vld1q_u8(CLASSES.high.as_ptr());
        let lane_bits = vld1q_u8(LANE_BITS.as_ptr());
        let classify = |bytes: uint8x16_t| {
            let low = vqtbl1q_u8(low, vandq_u8(bytes, vdupq_n_u8(0x0f)));
            let high = vqtbl1q_u8(high, vshrq_n_u8::<4>(bytes));
            vbicq_u8(lane_bits, vceqzq_u8(vandq_u8(low, high)))
        };
        let a = classify(vld1q_u8(block.as_ptr()));
        let b = classify(vld1q_u8(block.as_ptr().add(16)));
        let c = classify(vld1q_u8(block.as_ptr().add(32)));
        let d = classify(vld1q_u8(block.as_ptr().add(48)));

        // Reduce adjacent byte lanes. Each group of eight lane weights
        // becomes one mask byte, with the first source byte in the low bit.
        let halves = vpaddq_u8(vpaddq_u8(a, b), vpaddq_u8(c, d));
        vgetq_lane_u64::<0>(vreinterpretq_u64_u8(vpaddq_u8(halves, halves)))
    }
}
