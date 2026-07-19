#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodePoint {
    pub len: u8,
    pub value: u32,
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
    let value = s.chars().next().ok_or(Utf8Error::InvalidUtf8)? as u32;
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

pub fn is_multi_byte_space(cp: u32) -> bool {
    matches!(
        cp,
        0xFEFF | 0x00A0 | 0x2000..=0x200A | 0x202F | 0x205F | 0x3000 | 0x1680
    )
}

pub fn is_octal_digit(digit: u8) -> bool {
    matches!(digit, b'0'..=b'7')
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedCodePoint {
    pub value: u32,
    pub end: usize,
}

pub fn parse_octal(input: &[u8], start: usize) -> ParsedCodePoint {
    debug_assert!(start < input.len());
    let mut value = 0u32;
    let mut i = start;
    let max = if input[start] <= b'3' { 3 } else { 2 };
    let mut count = 0usize;

    while i < input.len() && count < max {
        if is_octal_digit(input[i]) {
            value = (value << 3) | u32::from(input[i] - b'0');
            count += 1;
            i += 1;
        } else {
            break;
        }
    }

    ParsedCodePoint { value, end: i }
}

pub fn parse_hex2(input: &[u8], start: usize) -> Option<ParsedCodePoint> {
    if start + 2 > input.len() {
        return None;
    }
    let hi = hex_val(input[start])?;
    let lo = hex_val(input[start + 1])?;
    Some(ParsedCodePoint {
        value: (u32::from(hi) << 4) | u32::from(lo),
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
        value,
        end: start + 4,
    })
}

pub fn parse_hex_variable(
    input: &[u8],
    start: usize,
    max_digits: usize,
) -> Option<ParsedCodePoint> {
    let mut value = 0u32;
    let mut i = start;
    let mut count = 0usize;
    while i < input.len() && count < max_digits {
        let digit = hex_val(input[i])?;
        value = (value << 4) | u32::from(digit);
        if value > 0x10FFFF {
            return None;
        }
        count += 1;
        i += 1;
    }
    if count == 0 {
        return None;
    }
    Some(ParsedCodePoint { value, end: i })
}

pub fn parse_unicode_escape(input: &[u8], start: usize) -> Option<ParsedCodePoint> {
    if start < input.len() && input[start] == b'{' {
        let digit_start = start + 1;
        let brace_end = input[digit_start..]
            .iter()
            .position(|byte| *byte == b'}')
            .map(|offset| digit_start + offset)?;
        let parsed = parse_hex_variable(input, digit_start, brace_end - digit_start)?;
        if parsed.end != brace_end {
            return None;
        }
        return Some(ParsedCodePoint {
            value: parsed.value,
            end: brace_end + 1,
        });
    }
    parse_hex4(input, start)
}

pub fn decode_identifier_escapes<'a>(raw: &'a [u8], buf: &'a mut [u8; 256]) -> &'a [u8] {
    let mut out = 0usize;
    let mut i = 0usize;
    while i < raw.len() {
        if raw[i] == b'\\' && i + 1 < raw.len() && raw[i + 1] == b'u' {
            let Some(parsed) = parse_unicode_escape(raw, i + 2) else {
                return raw;
            };
            if parsed.value < 0x80 {
                if out >= buf.len() {
                    return raw;
                }
                buf[out] = parsed.value as u8;
                out += 1;
            } else {
                let n = encode_code_point(parsed.value, &mut buf[out..]).unwrap_or(0);
                if n == 0 {
                    return raw;
                }
                out += n;
            }
            i = parsed.end;
        } else {
            if out >= buf.len() {
                return raw;
            }
            buf[out] = raw[i];
            out += 1;
            i += 1;
        }
    }
    &buf[..out]
}

pub fn decode_string_escapes(raw: &[u8], out: &mut Vec<u8>) {
    let mut i = 0usize;
    while i < raw.len() {
        let run_start = i;

        while i < raw.len() && raw[i] != b'\\' && raw[i] != b'\r' {
            i += 1;
        }

        if i > run_start {
            out.extend_from_slice(&raw[run_start..i]);
        }
        if i >= raw.len() {
            break;
        }

        if raw[i] == b'\r' {
            out.push(b'\n');
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
                out.push(b'\n');
                i += 1;
            }
            b'r' => {
                out.push(b'\r');
                i += 1;
            }
            b't' => {
                out.push(b'\t');
                i += 1;
            }
            b'b' => {
                out.push(0x08);
                i += 1;
            }
            b'f' => {
                out.push(0x0C);
                i += 1;
            }
            b'v' => {
                out.push(0x0B);
                i += 1;
            }
            b'0' => {
                if i + 1 < raw.len() && raw[i + 1].is_ascii_digit() {
                    let parsed = parse_octal(raw, i);
                    append_code_point(out, parsed.value);
                    i = parsed.end;
                } else {
                    out.push(0);
                    i += 1;
                }
            }
            b'1'..=b'7' => {
                let parsed = parse_octal(raw, i);
                append_code_point(out, parsed.value);
                i = parsed.end;
            }
            b'x' => {
                if let Some(parsed) = parse_hex2(raw, i + 1) {
                    append_code_point(out, parsed.value);
                    i = parsed.end;
                } else {
                    out.push(b'x');
                    i += 1;
                }
            }
            b'u' => {
                if let Some(parsed) = parse_unicode_escape(raw, i + 1) {
                    let mut cp = parsed.value;
                    let mut end = parsed.end;
                    if (0xD800..=0xDBFF).contains(&cp)
                        && end + 1 < raw.len()
                        && raw[end] == b'\\'
                        && raw[end + 1] == b'u'
                        && let Some(next) = parse_unicode_escape(raw, end + 2)
                        && (0xDC00..=0xDFFF).contains(&next.value)
                    {
                        cp = 0x10000 + (cp - 0xD800) * 0x400 + (next.value - 0xDC00);
                        end = next.end;
                    }
                    append_code_point(out, cp);
                    i = end;
                } else {
                    out.push(b'u');
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
            _ => {
                let us_len = unicode_separator_len(raw, i);
                if us_len > 0 {
                    i += usize::from(us_len);
                } else {
                    out.push(raw[i]);
                    i += 1;
                }
            }
        }
    }
}

fn append_code_point(out: &mut Vec<u8>, cp: u32) {
    debug_assert!(cp <= 0x10FFFF);
    if cp <= 0x7F {
        out.push(cp as u8);
    } else if cp <= 0x7FF {
        out.push(0xC0 | (cp >> 6) as u8);
        out.push(0x80 | (cp & 0x3F) as u8);
    } else if cp <= 0xFFFF {
        out.push(0xE0 | (cp >> 12) as u8);
        out.push(0x80 | ((cp >> 6) & 0x3F) as u8);
        out.push(0x80 | (cp & 0x3F) as u8);
    } else {
        out.push(0xF0 | (cp >> 18) as u8);
        out.push(0x80 | ((cp >> 12) & 0x3F) as u8);
        out.push(0x80 | ((cp >> 6) & 0x3F) as u8);
        out.push(0x80 | (cp & 0x3F) as u8);
    }
}

fn encode_code_point(cp: u32, buf: &mut [u8]) -> Option<usize> {
    let mut out = Vec::with_capacity(4);
    append_code_point(&mut out, cp);
    if out.len() > buf.len() {
        return None;
    }
    buf[..out.len()].copy_from_slice(&out);
    Some(out.len())
}

fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
