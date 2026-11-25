//! Byte-table continuation scan for non-x86 targets, including the NEON slow paths.

use super::{ascii_identifier_end, is_ascii_identifier_continue};

/// How a byte after an ASCII identifier prefix affects the identifier.
///
/// A `\` or a non-ASCII byte is [`End`](IdentifierClass::End) as well, because no caller reads
/// a third state: the scan tells the two apart by the byte itself, see
/// [`super::ascii_identifier_end`].
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum IdentifierClass {
    /// An ASCII identifier part.
    Continue,
    /// A byte that ends the identifier, including one the full scanner has to decide on.
    End,
}

const ASCII_IDENTIFIER_CLASS: [IdentifierClass; 256] = {
    let mut table = [IdentifierClass::End; 256];
    let mut byte = 0;
    while byte < 256 {
        if is_ascii_identifier_continue(byte as u8) {
            table[byte] = IdentifierClass::Continue;
        }
        byte += 1;
    }
    table
};

const ASCII_JSX_IDENTIFIER_CLASS: [IdentifierClass; 256] = {
    let mut table = ASCII_IDENTIFIER_CLASS;
    table[b'-' as usize] = IdentifierClass::Continue;
    table
};

const IDENTIFIER_BATCH_SIZE: usize = 32;

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
    let classes = if JSX {
        &ASCII_JSX_IDENTIFIER_CLASS
    } else {
        &ASCII_IDENTIFIER_CLASS
    };

    // Comparing the batch end instead of the remaining length keeps no extra counter live, so
    // the scan fits in caller-saved registers. `position + IDENTIFIER_BATCH_SIZE` cannot overflow
    // because a slice length is at most `isize::MAX`.
    while position + IDENTIFIER_BATCH_SIZE <= source.len() {
        let batch_end = position + IDENTIFIER_BATCH_SIZE;
        // SAFETY: `position <= source.len()` by the caller invariant above, and the loop condition
        // guarantees that `batch_end` is within `source`.
        let batch = unsafe { source.get_unchecked(position..batch_end) };
        for (offset, byte) in batch.iter().enumerate() {
            if classes[*byte as usize] != IdentifierClass::Continue {
                return ascii_identifier_end(*byte, position + offset);
            }
        }
        position += IDENTIFIER_BATCH_SIZE;
    }
    while let Some(byte) = source.get(position) {
        if classes[*byte as usize] != IdentifierClass::Continue {
            return ascii_identifier_end(*byte, position);
        }
        position += 1;
    }

    debug_assert!(position >= start as usize);
    debug_assert!(position <= source.len());
    Some(position as u32)
}
