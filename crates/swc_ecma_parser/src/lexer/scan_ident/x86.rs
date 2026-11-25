//! Stateless x86-64 scanning with SSE2. The first load also supplies the keyword bytes.

use swc_next_simd::prelude::*;

use crate::lexer::keyword_type_of_words;

use super::{
    TokenTag, ascii_identifier_end, is_ascii_identifier_continue, lexeme_mask,
    scan_ascii_identifier_or_keyword_slow,
};

/// Tests an identifier continuation byte in the final partial batch.
#[inline]
fn continues<const JSX: bool>(byte: u8) -> bool {
    is_ascii_identifier_continue(byte) || (JSX && byte == b'-')
}

/// The first lane that stops an ASCII identifier.
///
/// `byte | 0x20` folds `A-Z` onto `a-z`. A range test subtracts the range start and compares
/// the offset with the width as unsigned lanes. SSE2 compares signed lanes only, so the
/// comparison flips the offset's high bit, which folds into the subtraction, and it compares
/// with the constant in memory. Testing for stop lanes instead of continue lanes keeps every
/// test a single comparison with no inversion. A lane-wise table lookup of the class would be
/// shorter but needs `pshufb`, which is SSSE3, and the build target is x86-64-v1.
///
/// Where the next token starts follows the lane that stops this identifier, so what a scan costs
/// is this chain's depth rather than its instruction count: the tests are independent, so they
/// are joined by depth, the letters test last because it is the deepest.
#[inline]
fn first_identifier_stop<const JSX: bool>(bytes: u8x16) -> Option<usize> {
    #[inline]
    fn lanes_outside(bytes: u8x16, lo: u8, hi: u8) -> mask8x16 {
        (bytes - u8x16::splat(lo)).simd_gt(u8x16::splat(hi - lo))
    }

    let not_letters = lanes_outside(bytes | u8x16::splat(0x20), b'a', b'z');
    let not_digits = lanes_outside(bytes, b'0', b'9');
    let not_marks = bytes.simd_ne(u8x16::splat(b'_')) & bytes.simd_ne(u8x16::splat(b'$'));
    let stops = if JSX {
        // `-` joins the marks, which keeps the letters test off the deepest path.
        not_letters & (not_digits & (not_marks & bytes.simd_ne(u8x16::splat(b'-'))))
    } else {
        not_letters & (not_digits & not_marks)
    };

    stops.first_set()
}

/// Bytes one [`u8x16`] batch covers.
const IDENTIFIER_BATCH_SIZE: usize = 16;

// Keep the hot loop in the caller.
/// Returns the end of the ASCII identifier part at `start`, or `None` if an
/// escape or a non-ASCII byte follows it and the full scanner is needed.
#[inline(always)]
pub(in crate::lexer) fn scan_ascii_identifier_continue<const JSX: bool>(
    source: &[u8],
    start: u32,
) -> Option<u32> {
    debug_assert!(u32::try_from(source.len()).is_ok());
    debug_assert!((start as usize) <= source.len());
    // `scan_ascii_identifier` passes the byte after a start byte it already read from `source`.
    let mut position = start as usize;

    // One load and one branch per batch; only the first byte that stops the identifier is
    // looked at again.
    while let Some(chunk) = source[position..].first_chunk::<IDENTIFIER_BATCH_SIZE>() {
        if let Some(lane) = first_identifier_stop::<JSX>(u8x16::from_array(*chunk)) {
            // SAFETY: `lane` indexes the 16-byte chunk that starts at `position`.
            let byte = unsafe { *source.get_unchecked(position + lane) };
            return ascii_identifier_end(byte, position + lane);
        }
        position += IDENTIFIER_BATCH_SIZE;
    }
    // Fewer than a batch of bytes remain, so scan them one at a time.
    while let Some(&byte) = source.get(position) {
        if !continues::<JSX>(byte) {
            return ascii_identifier_end(byte, position);
        }
        position += 1;
    }

    debug_assert!(position >= start as usize);
    debug_assert!(position <= source.len());
    Some(position as u32)
}

const LEXEME_MASKS: [u128; IDENTIFIER_BATCH_SIZE] = {
    let mut masks = [0; IDENTIFIER_BATCH_SIZE];
    let mut len = 0;
    while len < IDENTIFIER_BATCH_SIZE {
        masks[len] = lexeme_mask(len);
        len += 1;
    }
    masks
};

/// Scans the ASCII identifier that starts with the identifier start byte at `start`, and
/// returns its end and its keyword tag. Returns `None` like
/// [`scan_ascii_identifier_continue`].
///
/// An identifier that ends in the first batch takes its keyword words from that batch, so
/// the keyword lookup neither loads the lexeme again nor branches on its length.
#[inline(always)]
pub(super) fn scan_ascii_identifier_or_keyword(
    source: &[u8],
    start: u32,
) -> Option<(u32, TokenTag)> {
    let position = start as usize;
    let Some(chunk) = source[position..].first_chunk::<IDENTIFIER_BATCH_SIZE>() else {
        return scan_ascii_identifier_or_keyword_slow(source, start);
    };
    let Some(lane) = first_identifier_stop::<false>(u8x16::from_array(*chunk)) else {
        // No keyword is this long.
        let end =
            scan_ascii_identifier_continue::<false>(source, start + IDENTIFIER_BATCH_SIZE as u32)?;
        return Some((end, TokenTag::Identifier));
    };
    // SAFETY: `lane` indexes the 16-byte chunk that starts at `position`.
    let byte = unsafe { *source.get_unchecked(position + lane) };
    let end = ascii_identifier_end(byte, position + lane)?;
    // Every keyword is at most 11 bytes, so a longer lexeme keeps a nonzero byte 11 in `hi`
    // and matches no table slot, which has zero there.
    let word = u128::from_le_bytes(*chunk) & LEXEME_MASKS[lane];
    Some((end, keyword_type_of_words(word as u64, (word >> 64) as u32)))
}
