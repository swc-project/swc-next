use std::fmt;

use swc_next_allocator::{vec::Vec as ArenaVec, wtf8::Wtf8};

const HEX: &[u8; 16] = b"0123456789ABCDEF";

#[derive(Default)]
pub(crate) struct Utf8QuoteSelector {
    single: usize,
    double: usize,
    backtick: usize,
    previous_dollar: bool,
}

impl Utf8QuoteSelector {
    pub(crate) fn quote(&self, allow_backtick: bool) -> u8 {
        if self.double > self.single {
            if allow_backtick && self.single > self.backtick {
                b'`'
            } else {
                b'\''
            }
        } else if allow_backtick && self.double > self.backtick {
            b'`'
        } else {
            b'"'
        }
    }
}

impl fmt::Write for Utf8QuoteSelector {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let bytes = value.as_bytes();
        self.single += bytes.iter().filter(|&&byte| byte == b'\'').count();
        self.double += bytes.iter().filter(|&&byte| byte == b'"').count();
        self.backtick += bytes.iter().filter(|&&byte| byte == b'`').count();
        self.backtick += bytes.windows(2).filter(|pair| *pair == b"${").count();
        if self.previous_dollar && bytes.first() == Some(&b'{') {
            self.backtick += 1;
        }
        if let Some(&last) = bytes.last() {
            self.previous_dollar = last == b'$';
        }
        Ok(())
    }
}

pub(crate) struct QuotedUtf8Writer<'out, 'alloc> {
    out: &'out mut ArenaVec<'alloc, u8>,
    quote: u8,
    ascii_only: bool,
    pending_zero: bool,
}

impl<'out, 'alloc> QuotedUtf8Writer<'out, 'alloc> {
    pub(crate) fn new(out: &'out mut ArenaVec<'alloc, u8>, quote: u8, ascii_only: bool) -> Self {
        out.push(quote);
        Self {
            out,
            quote,
            ascii_only,
            pending_zero: false,
        }
    }

    pub(crate) fn finish(mut self) {
        if self.pending_zero {
            self.out.extend_from_slice(b"\\0");
            self.pending_zero = false;
        }
        self.out.push(self.quote);
    }
}

impl fmt::Write for QuotedUtf8Writer<'_, '_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let bytes = value.as_bytes();
        if bytes.is_empty() {
            return Ok(());
        }
        if self.pending_zero {
            if bytes[0].is_ascii_digit() {
                self.out.extend_from_slice(b"\\x00");
            } else {
                self.out.extend_from_slice(b"\\0");
            }
            self.pending_zero = false;
        }

        let mut i = 0;
        while i < bytes.len() {
            let (cp, size) = next_code_point(bytes, i);
            i += size;
            match cp {
                0 if i == bytes.len() => self.pending_zero = true,
                0 if bytes[i].is_ascii_digit() => self.out.extend_from_slice(b"\\x00"),
                0 => self.out.extend_from_slice(b"\\0"),
                7 => self.out.extend_from_slice(b"\\x07"),
                8 => self.out.extend_from_slice(b"\\b"),
                9 => self.out.push(b'\t'),
                12 => self.out.extend_from_slice(b"\\f"),
                10 if self.quote == b'`' => self.out.push(b'\n'),
                10 => self.out.extend_from_slice(b"\\n"),
                13 => self.out.extend_from_slice(b"\\r"),
                11 => self.out.extend_from_slice(b"\\v"),
                27 => self.out.extend_from_slice(b"\\x1B"),
                92 => self.out.extend_from_slice(b"\\\\"),
                47 if self.out.last() == Some(&b'<')
                    && bytes
                        .get(i..i.saturating_add(6))
                        .is_some_and(|tail| tail.eq_ignore_ascii_case(b"script")) =>
                {
                    self.out.extend_from_slice(b"\\/")
                }
                0x2028 => self.out.extend_from_slice(b"\\u2028"),
                0x2029 => self.out.extend_from_slice(b"\\u2029"),
                0xfeff => self.out.extend_from_slice(b"\\uFEFF"),
                36 if self.quote == b'`' && i < bytes.len() && bytes[i] == b'{' => {
                    self.out.extend_from_slice(b"\\$")
                }
                x if x == self.quote as u32 => {
                    self.out.push(b'\\');
                    self.out.push(self.quote);
                }
                x if x <= 0x7e => self.out.push(x as u8),
                x if self.ascii_only && x <= 0xff => {
                    self.out.extend_from_slice(b"\\x");
                    self.out.push(HEX[(x >> 4) as usize]);
                    self.out.push(HEX[(x & 15) as usize]);
                }
                x if self.ascii_only && x <= 0xffff => push_u16_escape(self.out, x as u16),
                x if self.ascii_only => push_code_point_escape(self.out, x),
                x => {
                    let mut buf = [0; 4];
                    self.out.extend_from_slice(
                        char::from_u32(x)
                            .unwrap_or('\u{fffd}')
                            .encode_utf8(&mut buf)
                            .as_bytes(),
                    );
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn push_code_point_escape(out: &mut ArenaVec<'_, u8>, code_point: u32) {
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

pub(crate) fn quote_into(out: &mut ArenaVec<'_, u8>, value: &Wtf8, quote: u8, ascii_only: bool) {
    let value = value.as_bytes();
    out.push(quote);
    let mut i = 0;
    while i < value.len() {
        let (cp, size) = next_wtf8_code_point(value, i);
        i += size;
        match cp {
            0 => {
                // Use "\x00" before a digit so the escape cannot consume the
                // digit as part of a legacy octal escape.
                if i < value.len() && value[i].is_ascii_digit() {
                    out.extend_from_slice(b"\\x00")
                } else {
                    out.extend_from_slice(b"\\0")
                }
            }
            7 => out.extend_from_slice(b"\\x07"),
            8 => out.extend_from_slice(b"\\b"),
            9 => out.push(b'\t'),
            12 => out.extend_from_slice(b"\\f"),
            10 => {
                if quote == b'`' {
                    out.push(b'\n')
                } else {
                    out.extend_from_slice(b"\\n")
                }
            }
            13 => out.extend_from_slice(b"\\r"),
            11 => out.extend_from_slice(b"\\v"),
            27 => out.extend_from_slice(b"\\x1B"),
            92 => out.extend_from_slice(b"\\\\"),
            47 if out.last() == Some(&b'<')
                && value
                    .get(i..i.saturating_add(6))
                    .is_some_and(|tail| tail.eq_ignore_ascii_case(b"script")) =>
            {
                // Avoid generating the sequence "</script" inside an inline
                // script.
                out.extend_from_slice(b"\\/")
            }
            // U+2028 and U+2029 are JavaScript line terminators and must be
            // escaped inside string literals. Escape the BOM as well.
            0x2028 => out.extend_from_slice(b"\\u2028"),
            0x2029 => out.extend_from_slice(b"\\u2029"),
            0xfeff => out.extend_from_slice(b"\\uFEFF"),
            36 if quote == b'`' && i < value.len() && value[i] == b'{' => {
                // Avoid starting a template substitution.
                out.extend_from_slice(b"\\$")
            }
            x if x == quote as u32 => {
                out.push(b'\\');
                out.push(quote)
            }
            x if x <= 0x7e => out.push(x as u8),
            x if (0xd800..=0xdfff).contains(&x) => push_u16_escape(out, x as u16),
            x if ascii_only && x <= 0xff => {
                out.extend_from_slice(b"\\x");
                out.push(HEX[(x >> 4) as usize]);
                out.push(HEX[(x & 15) as usize]);
            }
            x if ascii_only && x <= 0xffff => push_u16_escape(out, x as u16),
            x if ascii_only => push_code_point_escape(out, x),
            x => {
                let mut buf = [0; 4];
                out.extend_from_slice(
                    char::from_u32(x)
                        .unwrap_or('\u{fffd}')
                        .encode_utf8(&mut buf)
                        .as_bytes(),
                );
            }
        }
    }
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
pub(crate) fn push_u16_escape(out: &mut ArenaVec<'_, u8>, c: u16) {
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
