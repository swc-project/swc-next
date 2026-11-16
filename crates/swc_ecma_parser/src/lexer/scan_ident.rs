//! Scans the ASCII identifier part after an identifier start byte, and says whether a byte can
//! belong to an identifier. The scan lives in `imp`, defined once per target: x86-64 classifies a
//! whole batch at once, where a lane mask is one `pmovmskb`, and other targets read a byte table.

use super::{TokenTag, get_keyword_type};

/// Whether `byte` continues a non-JSX ASCII identifier.
#[inline(always)]
const fn is_ascii_identifier_continue(byte: u8) -> bool {
    matches!(byte, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'$')
}

/// Bytes that go to the full scanner instead of ending the identifier: `\\` starts an escape
/// and non-ASCII bytes need the identifier tables. `byte ^ b'\\'` covers both in one signed
/// comparison.
#[inline(always)]
fn needs_full_identifier_scan(byte: u8) -> bool {
    (byte ^ b'\\') as i8 <= 0
}

/// Whether `byte` can still belong to an identifier, as an ASCII identifier part or through
/// the full identifier scanner.
#[inline(always)]
pub(super) fn is_identifier_byte(byte: u8) -> bool {
    is_ascii_identifier_continue(byte) || needs_full_identifier_scan(byte)
}

/// Ends an ASCII identifier part at `position`, where `byte` is not an identifier part. Returns
/// `None` for `\\` and non-ASCII bytes, which need the full scanner. Testing the byte instead of
/// the class keeps the scan loop from holding the class of each byte.
#[inline(always)]
fn ascii_identifier_end(byte: u8, position: usize) -> Option<u32> {
    (!needs_full_identifier_scan(byte)).then_some(position as u32)
}

#[cfg(target_arch = "x86_64")]
mod imp {
    use swc_next_simd::prelude::*;

    use crate::lexer::keyword_type_of_words;

    use super::{
        TokenTag, ascii_identifier_end, is_ascii_identifier_continue,
        scan_ascii_identifier_or_keyword_slow,
    };

    /// Whether `byte` continues an identifier, the complement of the set that
    /// [`identifier_stop_bits`] computes a batch at a time.
    #[inline(always)]
    fn continues<const JSX: bool>(byte: u8) -> bool {
        is_ascii_identifier_continue(byte) || (JSX && byte == b'-')
    }

    /// The lanes that stop an ASCII identifier, as one bit per lane.
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
    #[inline(always)]
    fn identifier_stop_bits<const JSX: bool>(bytes: u8x16) -> u16 {
        #[inline(always)]
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

        stops.to_bitmask()
    }

    /// Bytes one [`u8x16`] batch covers.
    const IDENTIFIER_BATCH_SIZE: usize = 16;

    // This is a very hot path. Inline this loop too
    // to avoid losing the single-call-site bonus and
    // introducing a new helper call.
    /// Returns the end of the ASCII identifier part at `start`, or `None` if an
    /// escape or a non-ASCII byte follows it and the full scanner is needed.
    #[inline(always)]
    pub fn scan_ascii_identifier_continue<const JSX: bool>(
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
            let stopped = identifier_stop_bits::<JSX>(u8x16::from_array(*chunk));
            let lane = stopped.trailing_zeros() as usize;
            if lane < IDENTIFIER_BATCH_SIZE {
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

    /// `LEXEME_MASKS[len]` keeps the first `len` bytes of a little-endian 16-byte word.
    const LEXEME_MASKS: [u128; IDENTIFIER_BATCH_SIZE] = {
        let mut masks = [0; IDENTIFIER_BATCH_SIZE];
        let mut len = 0;
        while len < IDENTIFIER_BATCH_SIZE {
            masks[len] = (1 << (8 * len)) - 1;
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
    pub fn scan_ascii_identifier_or_keyword(source: &[u8], start: u32) -> Option<(u32, TokenTag)> {
        let position = start as usize;
        let Some(chunk) = source[position..].first_chunk::<IDENTIFIER_BATCH_SIZE>() else {
            return scan_ascii_identifier_or_keyword_slow(source, start);
        };
        let stopped = identifier_stop_bits::<false>(u8x16::from_array(*chunk));
        let lane = stopped.trailing_zeros() as usize;
        if lane >= IDENTIFIER_BATCH_SIZE {
            // No keyword is this long.
            let end = scan_ascii_identifier_continue::<false>(
                source,
                start + IDENTIFIER_BATCH_SIZE as u32,
            )?;
            return Some((end, TokenTag::Identifier));
        }
        // SAFETY: `lane` indexes the 16-byte chunk that starts at `position`.
        let byte = unsafe { *source.get_unchecked(position + lane) };
        let end = ascii_identifier_end(byte, position + lane)?;
        // Every keyword is at most 11 bytes, so a longer lexeme keeps a nonzero byte 11 in `hi`
        // and matches no table slot, which has zero there.
        let word = u128::from_le_bytes(*chunk) & LEXEME_MASKS[lane];
        Some((end, keyword_type_of_words(word as u64, (word >> 64) as u32)))
    }
}

#[cfg(not(target_arch = "x86_64"))]
mod imp {
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

    // This is a very hot path. Inline this loop too
    // to avoid losing the single-call-site bonus and
    // introducing a new helper call.
    /// Returns the end of the ASCII identifier part at `start`, or `None` if an
    /// escape or a non-ASCII byte follows it and the full scanner is needed.
    #[inline(always)]
    pub fn scan_ascii_identifier_continue<const JSX: bool>(
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
}

pub use imp::*;

/// Scans the ASCII identifier that starts with the identifier start byte at `start`, and
/// returns its end and its keyword tag. Returns `None` like [`scan_ascii_identifier_continue`].
#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn scan_ascii_identifier_or_keyword(source: &[u8], start: u32) -> Option<(u32, TokenTag)> {
    scan_ascii_identifier_or_keyword_slow(source, start)
}

#[inline(always)]
fn scan_ascii_identifier_or_keyword_slow(source: &[u8], start: u32) -> Option<(u32, TokenTag)> {
    let end = scan_ascii_identifier_continue::<false>(source, start + 1)?;
    // SAFETY: `start` indexes the identifier start byte, and the scan returns a position after it
    // that is at most `source.len()`.
    let lexeme = unsafe { source.get_unchecked(start as usize..end as usize) };
    Some((end, get_keyword_type(lexeme)))
}

#[cfg(test)]
mod tests {
    use super::{is_identifier_byte, scan_ascii_identifier_continue};

    fn end<const JSX: bool>(source: &str, start: u32) -> Option<u32> {
        scan_ascii_identifier_continue::<JSX>(source.as_bytes(), start)
    }

    /// `len` identifier bytes followed by `terminator`.
    fn ident(len: usize, terminator: &str) -> String {
        "a".repeat(len) + terminator
    }

    #[test]
    fn scan_ends_at_a_terminator() {
        assert_eq!(end::<false>("ab c", 1), Some(2));
        assert_eq!(end::<false>("ab", 1), Some(2));
        // Identifiers longer than one batch, so the stop byte is not in the first one.
        assert_eq!(end::<false>(&ident(20, ";"), 1), Some(20));
        assert_eq!(end::<false>(&ident(48, ";"), 1), Some(48));
        // `-` continues a JSX tag name only.
        assert_eq!(end::<false>("a-b", 1), Some(1));
        assert_eq!(end::<true>("a-b", 1), Some(3));
    }

    #[test]
    fn scan_classifies_every_byte() {
        for byte in 0..=u8::MAX {
            // The byte falls in the middle of a full batch.
            let mut source = b"a".repeat(24);
            source[5] = byte;
            let end = scan_ascii_identifier_continue::<false>(&source, 1);
            let expected = if super::is_ascii_identifier_continue(byte) {
                Some(24)
            } else if is_identifier_byte(byte) {
                None
            } else {
                Some(5)
            };
            assert_eq!((byte, end), (byte, expected));
            let jsx_end = scan_ascii_identifier_continue::<true>(&source, 1);
            let jsx_expected = if byte == b'-' { Some(24) } else { expected };
            assert_eq!((byte, jsx_end), (byte, jsx_expected));
        }
    }

    #[test]
    fn scan_needs_the_full_scanner() {
        assert_eq!(end::<false>(r"a\u0062", 1), None);
        assert_eq!(end::<false>("a\u{e9}", 1), None);
        assert_eq!(end::<true>(r"a\", 1), None);
    }
}
