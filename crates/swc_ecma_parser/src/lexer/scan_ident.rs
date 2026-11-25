//! ASCII identifiers and keywords use cached block masks on little-endian
//! AArch64 NEON. JSX names use a stateless continuation scan.

use super::{TokenTag, get_keyword_type};

#[cfg_attr(target_arch = "x86_64", path = "scan_ident/x86.rs")]
#[cfg_attr(not(target_arch = "x86_64"), path = "scan_ident/scalar.rs")]
mod imp;

pub(super) use imp::scan_ascii_identifier_continue;

cfg_select! {
    all(target_arch = "aarch64", target_endian = "little", target_feature = "neon") => {
        mod blocks;
        pub(super) use blocks::IdentifierBlocks as IdentifierScanner;
    }
    _ => {
        mod stateless;
        pub(super) use stateless::IdentifierScanner;
    }
}

/// Keeps the first `len` bytes of a little-endian keyword lookup word.
#[cfg(any(
    target_arch = "x86_64",
    all(
        target_arch = "aarch64",
        target_endian = "little",
        target_feature = "neon"
    )
))]
const fn lexeme_mask(len: usize) -> u128 {
    (1 << (len * 8)) - 1
}

/// Whether `byte` continues a non-JSX ASCII identifier.
#[inline]
const fn is_ascii_identifier_continue(byte: u8) -> bool {
    matches!(byte, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'$')
}

/// Bytes that go to the full scanner instead of ending the identifier: `\\` starts an escape
/// and non-ASCII bytes need the identifier tables. `byte ^ b'\\'` covers both in one signed
/// comparison.
#[inline]
fn needs_full_identifier_scan(byte: u8) -> bool {
    (byte ^ b'\\') as i8 <= 0
}

/// Whether `byte` can still belong to an identifier, as an ASCII identifier part or through
/// the full identifier scanner.
#[inline]
pub(super) fn is_identifier_byte(byte: u8) -> bool {
    is_ascii_identifier_continue(byte) || needs_full_identifier_scan(byte)
}

/// Ends an ASCII identifier part at `position`, where `byte` is not an identifier part. Returns
/// `None` for `\\` and non-ASCII bytes, which need the full scanner. Testing the byte instead of
/// the class keeps the scan loop from holding the class of each byte.
#[inline]
fn ascii_identifier_end(byte: u8, position: usize) -> Option<u32> {
    (!needs_full_identifier_scan(byte)).then_some(position as u32)
}

/// Scans a whole ASCII identifier when a single fast-path batch is not enough
/// or the target has no SIMD identifier scanner.
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

    #[test]
    fn scan_identifier_boundaries_and_keywords() {
        use super::{IdentifierScanner, TokenTag, is_ascii_identifier_continue};

        // Move each byte through the 64 lanes, including adjacent blocks.
        for start in [0, 63] {
            for byte in 0..=u8::MAX {
                for offset in 1..=64 {
                    let mut source = [b';'; 192];
                    source[start..start + offset].fill(b'x');
                    source[start + offset] = byte;
                    let mut scanner = IdentifierScanner::new(&source);
                    let expected = if is_ascii_identifier_continue(byte) {
                        Some((start + offset + 1, TokenTag::Identifier))
                    } else if is_identifier_byte(byte) {
                        None
                    } else {
                        Some((start + offset, TokenTag::Identifier))
                    };
                    let scanned = scanner.scan(&source, start as u32);
                    assert_eq!(scanned.map(|(end, tag)| (end as usize, tag)), expected);
                }
            }
        }

        // Lookahead, recovery, or a token-reader jump can move outside the cache.
        let source = format!("{:63}const{:61}let", "", "");
        let mut scanner = IdentifierScanner::new(source.as_bytes());
        assert_eq!(
            scanner.scan(source.as_bytes(), 63),
            Some((68, TokenTag::Const))
        );
        assert_eq!(
            scanner.scan(source.as_bytes(), 129),
            Some((132, TokenTag::Let))
        );
        assert_eq!(
            scanner.scan(source.as_bytes(), 63),
            Some((68, TokenTag::Const))
        );
    }
}
