use memchr::memchr2;
use swc_next_allocator::wtf8::{CodePoint as Wtf8CodePoint, Wtf8Buf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodePoint {
    pub len: u8,
    pub value: Wtf8CodePoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Utf8Error {
    InvalidUtf8,
}

pub fn code_point_at(source: &[u8], i: usize) -> Result<CodePoint, Utf8Error> {
    debug_assert!(i < source.len());
    let first = source[i];
    let len = if first < 0x80 {
        1
    } else if first & 0b1110_0000 == 0b1100_0000 {
        2
    } else if first & 0b1111_0000 == 0b1110_0000 {
        3
    } else if first & 0b1111_1000 == 0b1111_0000 {
        4
    } else {
        return Err(Utf8Error::InvalidUtf8);
    };
    if source.len() - i < len {
        return Err(Utf8Error::InvalidUtf8);
    }
    let s = std::str::from_utf8(&source[i..i + len]).map_err(|_| Utf8Error::InvalidUtf8)?;
    let value = Wtf8CodePoint::from_char(s.chars().next().ok_or(Utf8Error::InvalidUtf8)?);
    Ok(CodePoint {
        len: len as u8,
        value,
    })
}

pub fn unicode_separator_len(source: &[u8], pos: usize) -> u8 {
    if pos + 2 < source.len()
        && source[pos] == 0xE2
        && source[pos + 1] == 0x80
        && matches!(source[pos + 2], 0xA8 | 0xA9)
    {
        3
    } else {
        0
    }
}

#[inline]
pub fn is_multi_byte_space(cp: u32) -> bool {
    matches!(
        cp,
        0xFEFF | 0x00A0 | 0x0085 | 0x2000..=0x200A | 0x202F | 0x205F | 0x3000 | 0x1680
    )
}

#[inline]
pub fn is_octal_digit(digit: u8) -> bool {
    matches!(digit, b'0'..=b'7')
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedCodePoint {
    pub value: Wtf8CodePoint,
    pub end: usize,
}

pub fn parse_octal(input: &[u8], start: usize) -> ParsedCodePoint {
    debug_assert!(start < input.len());
    let mut value = 0u8;
    let mut i = start;
    let max = if input[start] <= b'3' { 3 } else { 2 };
    let mut count = 0usize;

    while i < input.len() && count < max {
        if is_octal_digit(input[i]) {
            value = (value << 3) | (input[i] - b'0');
            count += 1;
            i += 1;
        } else {
            break;
        }
    }

    ParsedCodePoint {
        value: Wtf8CodePoint::from_char(char::from(value)),
        end: i,
    }
}

pub fn parse_hex2(input: &[u8], start: usize) -> Option<ParsedCodePoint> {
    if start + 2 > input.len() {
        return None;
    }
    let hi = hex_val(input[start])?;
    let lo = hex_val(input[start + 1])?;
    Some(ParsedCodePoint {
        value: Wtf8CodePoint::from_u32((u32::from(hi) << 4) | u32::from(lo))?,
        end: start + 2,
    })
}

pub fn parse_hex4(input: &[u8], start: usize) -> Option<ParsedCodePoint> {
    if start + 4 > input.len() {
        return None;
    }
    let mut value = 0u32;
    for j in 0..4 {
        let digit = hex_val(input[start + j])?;
        value = (value << 4) | u32::from(digit);
    }
    Some(ParsedCodePoint {
        value: Wtf8CodePoint::from_u32(value)?,
        end: start + 4,
    })
}

pub fn parse_unicode_escape(input: &[u8], start: usize) -> Option<ParsedCodePoint> {
    if input.get(start) != Some(&b'{') {
        return parse_hex4(input, start);
    }

    // Searching for `}` before validating digits makes template recovery quadratic.
    let digit_start = start + 1;
    let mut value = 0u32;
    for (offset, &byte) in input[digit_start..].iter().enumerate() {
        if byte == b'}' {
            if offset == 0 {
                return None;
            }
            return Some(ParsedCodePoint {
                value: Wtf8CodePoint::from_u32(value)?,
                end: digit_start + offset + 1,
            });
        }

        let digit = hex_val(byte)?;
        value = (value << 4) | u32::from(digit);
        if value > 0x10FFFF {
            return None;
        }
    }
    None
}

pub fn decode_identifier_escapes<'a>(raw: &str, buf: &'a mut [u8]) -> Option<&'a [u8]> {
    let raw = raw.as_bytes();
    let mut out = 0usize;
    let mut i = 0usize;
    while i < raw.len() {
        if raw[i] == b'\\' && i + 1 < raw.len() && raw[i + 1] == b'u' {
            let parsed = parse_unicode_escape(raw, i + 2)?;
            let value = parsed.value.to_char()?;
            let len = value.len_utf8();
            if out + len > buf.len() {
                return None;
            }
            value.encode_utf8(&mut buf[out..out + len]);
            out += len;
            i = parsed.end;
        } else {
            if out >= buf.len() {
                return None;
            }
            buf[out] = raw[i];
            out += 1;
            i += 1;
        }
    }
    Some(&buf[..out])
}

pub fn decode_string_escapes(raw_str: &str, out: &mut Wtf8Buf) {
    let raw = raw_str.as_bytes();
    let mut i = 0usize;
    while i < raw.len() {
        let run_start = i;

        // Both delimiters are ASCII, so the copied run ends on a UTF-8 boundary.
        if raw.len() - i >= 32 {
            i += memchr2(b'\\', b'\r', &raw[i..]).unwrap_or(raw.len() - i);
        } else {
            while i < raw.len() && raw[i] != b'\\' && raw[i] != b'\r' {
                i += 1;
            }
        }

        if i > run_start {
            out.push_str(&raw_str[run_start..i]);
        }
        if i >= raw.len() {
            break;
        }

        if raw[i] == b'\r' {
            out.push_str("\n");
            i += 1;
            if i < raw.len() && raw[i] == b'\n' {
                i += 1;
            }
            continue;
        }

        i += 1;
        if i >= raw.len() {
            break;
        }

        match raw[i] {
            b'n' => {
                out.push_str("\n");
                i += 1;
            }
            b'r' => {
                out.push_str("\r");
                i += 1;
            }
            b't' => {
                out.push_str("\t");
                i += 1;
            }
            b'b' => {
                out.push_str("\u{0008}");
                i += 1;
            }
            b'f' => {
                out.push_str("\u{000c}");
                i += 1;
            }
            b'v' => {
                out.push_str("\u{000b}");
                i += 1;
            }
            b'0' => {
                if i + 1 < raw.len() && raw[i + 1].is_ascii_digit() {
                    let parsed = parse_octal(raw, i);
                    out.push(parsed.value);
                    i = parsed.end;
                } else {
                    out.push_str("\0");
                    i += 1;
                }
            }
            b'1'..=b'7' => {
                let parsed = parse_octal(raw, i);
                out.push(parsed.value);
                i = parsed.end;
            }
            b'x' => {
                if let Some(parsed) = parse_hex2(raw, i + 1) {
                    out.push(parsed.value);
                    i = parsed.end;
                } else {
                    out.push_str("x");
                    i += 1;
                }
            }
            b'u' => {
                if let Some(parsed) = parse_unicode_escape(raw, i + 1) {
                    out.push(parsed.value);
                    i = parsed.end;
                } else {
                    out.push_str("u");
                    i += 1;
                }
            }
            b'\n' => i += 1,
            b'\r' => {
                i += 1;
                if i < raw.len() && raw[i] == b'\n' {
                    i += 1;
                }
            }
            byte if byte.is_ascii() => {
                out.push_char(char::from(byte));
                i += 1;
            }
            _ => {
                let us_len = unicode_separator_len(raw, i);
                if us_len > 0 {
                    i += usize::from(us_len);
                } else {
                    let Some(value) = raw_str[i..].chars().next() else {
                        break;
                    };
                    out.push_char(value);
                    i += value.len_utf8();
                }
            }
        }
    }
}

fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
