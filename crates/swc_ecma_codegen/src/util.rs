use swc_next_allocator::wtf8::Wtf8;
use swc_next_simd::prelude::*;

const HEX: &[u8; 16] = b"0123456789ABCDEF";

pub(crate) fn push_code_point_escape(out: &mut Vec<u8>, code_point: u32) {
    out.extend_from_slice(b"\\u{");
    let mut shift = 28;
    while shift > 0 && code_point >> shift == 0 {
        shift -= 4;
    }
    loop {
        out.push(HEX[((code_point >> shift) & 15) as usize]);
        if shift == 0 {
            break;
        }
        shift -= 4;
    }
    out.push(b'}');
}

pub(crate) fn quote_into(
    out: &mut Vec<u8>,
    value: &Wtf8,
    quote: u8,
    ascii_only: bool,
    inline_script: bool,
) {
    let value = value.as_bytes();
    out.push(quote);
    let quote_escape = [b'\\', quote];
    let mut start = 0;
    let mut i = 0;
    while i < value.len() {
        while let Some(chunk) = value[i..].first_chunk::<16>() {
            let bytes = u8x16::from_array(*chunk);
            let mut special = bytes.simd_lt(u8x16::splat(0x20))
                | bytes.simd_ge(u8x16::splat(0x7f))
                | bytes.simd_eq(u8x16::splat(b'\\'))
                | bytes.simd_eq(u8x16::splat(quote));
            if inline_script {
                special = special | bytes.simd_eq(u8x16::splat(b'/'));
            }
            if quote == b'`' {
                special = special | bytes.simd_eq(u8x16::splat(b'$'));
            }
            let run = special.first_set().unwrap_or(16);
            i += run;
            if run < 16 {
                break;
            }
        }
        if i == value.len() {
            break;
        }

        let position = i;
        let (cp, size) = next_wtf8_code_point(value, i);
        i += size;
        let escape: Option<&[u8]> = match cp {
            // Avoid consuming a following digit as part of a legacy octal escape.
            0 if i < value.len() && value[i].is_ascii_digit() => Some(b"\\x00"),
            0 => Some(b"\\0"),
            7 => Some(b"\\x07"),
            8 => Some(b"\\b"),
            10 if quote != b'`' => Some(b"\\n"),
            11 => Some(b"\\v"),
            12 => Some(b"\\f"),
            13 => Some(b"\\r"),
            27 => Some(b"\\x1B"),
            92 => Some(b"\\\\"),
            47 if inline_script
                && value[..position].last() == Some(&b'<')
                && value
                    .get(i..i.saturating_add(6))
                    .is_some_and(|tail| tail.eq_ignore_ascii_case(b"script")) =>
            {
                Some(b"\\/")
            }
            // Escape JavaScript line separators and the BOM.
            0x2028 => Some(b"\\u2028"),
            0x2029 => Some(b"\\u2029"),
            0xfeff => Some(b"\\uFEFF"),
            36 if quote == b'`' && i < value.len() && value[i] == b'{' => Some(b"\\$"),
            x if x == quote as u32 => Some(&quote_escape),
            _ => None,
        };
        let surrogate = (0xd800..=0xdfff).contains(&cp);
        if escape.is_none() && !surrogate && !(ascii_only && cp > 0x7e) {
            continue;
        }

        out.extend_from_slice(&value[start..position]);
        if let Some(escape) = escape {
            out.extend_from_slice(escape);
        } else if surrogate {
            push_u16_escape(out, cp as u16);
        } else if cp <= 0xff {
            out.extend_from_slice(b"\\x");
            out.push(HEX[(cp >> 4) as usize]);
            out.push(HEX[(cp & 15) as usize]);
        } else if cp <= 0xffff {
            push_u16_escape(out, cp as u16);
        } else {
            push_code_point_escape(out, cp);
        }
        start = i;
    }
    out.extend_from_slice(&value[start..]);
    out.push(quote);
}

fn next_wtf8_code_point(bytes: &[u8], i: usize) -> (u32, usize) {
    // AST strings use WTF-8 so lone UTF-16 surrogates survive parsing. Go's
    // printer receives UTF-16 directly and escapes these code units verbatim.
    let byte = bytes[i];
    if byte < 0x80 {
        return (byte as u32, 1);
    }
    if byte == 0xed
        && let (Some(&b1 @ 0xa0..=0xbf), Some(&b2 @ 0x80..=0xbf)) =
            (bytes.get(i + 1), bytes.get(i + 2))
    {
        return (
            (((byte & 0x0f) as u32) << 12) | (((b1 & 0x3f) as u32) << 6) | (b2 & 0x3f) as u32,
            3,
        );
    }
    next_code_point(bytes, i)
}
pub(crate) fn push_u16_escape(out: &mut Vec<u8>, c: u16) {
    out.extend_from_slice(&[
        b'\\',
        b'u',
        HEX[(c >> 12) as usize],
        HEX[((c >> 8) & 15) as usize],
        HEX[((c >> 4) & 15) as usize],
        HEX[(c & 15) as usize],
    ]);
}
pub(crate) fn next_code_point(bytes: &[u8], i: usize) -> (u32, usize) {
    let b = bytes[i];
    if b < 0x80 {
        return (b as u32, 1);
    }

    // Match Go's UTF-8 range decoding: malformed, overlong, surrogate, and
    // out-of-range encodings each consume one byte and produce U+FFFD.
    let continuation = |offset: usize| {
        bytes
            .get(i + offset)
            .copied()
            .filter(|byte| byte & 0xc0 == 0x80)
    };
    match b {
        0xc2..=0xdf => {
            let Some(b1) = continuation(1) else {
                return (0xfffd, 1);
            };
            ((((b & 0x1f) as u32) << 6) | (b1 & 0x3f) as u32, 2)
        }
        0xe0..=0xef => {
            let Some(b1) = continuation(1) else {
                return (0xfffd, 1);
            };
            let Some(b2) = continuation(2) else {
                return (0xfffd, 1);
            };
            if (b == 0xe0 && b1 < 0xa0) || (b == 0xed && b1 >= 0xa0) {
                return (0xfffd, 1);
            }
            (
                (((b & 0x0f) as u32) << 12) | (((b1 & 0x3f) as u32) << 6) | (b2 & 0x3f) as u32,
                3,
            )
        }
        0xf0..=0xf4 => {
            let Some(b1) = continuation(1) else {
                return (0xfffd, 1);
            };
            let Some(b2) = continuation(2) else {
                return (0xfffd, 1);
            };
            let Some(b3) = continuation(3) else {
                return (0xfffd, 1);
            };
            if (b == 0xf0 && b1 < 0x90) || (b == 0xf4 && b1 >= 0x90) {
                return (0xfffd, 1);
            }
            (
                (((b & 0x07) as u32) << 18)
                    | (((b1 & 0x3f) as u32) << 12)
                    | (((b2 & 0x3f) as u32) << 6)
                    | (b3 & 0x3f) as u32,
                4,
            )
        }
        _ => (0xfffd, 1),
    }
}
