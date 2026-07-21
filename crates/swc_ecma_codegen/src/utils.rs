pub fn is_word_op(op: &[u8]) -> bool {
    matches!(op, b"in" | b"instanceof" | b"typeof" | b"void" | b"delete")
}

#[inline]
pub fn is_id_cont(c: u8) -> bool {
    c == b'$' || c >= 0x80 || unicode_id_start::is_id_continue(char::from(c))
}

pub fn is_identifier_name(s: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(s) else {
        return false;
    };
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !unicode_id_start::is_id_start(first) {
        return false;
    }
    chars.all(unicode_id_start::is_id_continue)
}

pub fn script_escape(s: &[u8], i: usize) -> Option<&'static [u8]> {
    match s[i] {
        b'<' if script_open_at(s, i) => Some(b"<\\"),
        b'>' if i >= 2 && s[i - 1] == b'-' && s[i - 2] == b'-' => Some(b"\\>"),
        _ => None,
    }
}

fn script_open_at(s: &[u8], i: usize) -> bool {
    let rest = &s[i + 1..];
    if rest.starts_with(b"!--") {
        return true;
    }
    if rest.len() < 7 || rest[0] != b'/' || !rest[1..7].eq_ignore_ascii_case(b"script") {
        return false;
    }
    rest.len() == 7 || rest[7].is_ascii_whitespace() || matches!(rest[7], b'/' | b'>')
}

pub fn strip_underscores<'a>(raw: &'a [u8], buf: &'a mut [u8]) -> Option<&'a [u8]> {
    if !raw.contains(&b'_') {
        return Some(raw);
    }
    if raw.len() > buf.len() {
        return None;
    }
    let mut len = 0;
    for &c in raw {
        if c != b'_' {
            buf[len] = c;
            len += 1;
        }
    }
    Some(&buf[..len])
}

pub fn shortest_decimal<'a>(s: &'a [u8], scratch: &'a mut [u8]) -> &'a [u8] {
    let exp_at = s
        .iter()
        .position(|&byte| matches!(byte, b'e' | b'E'))
        .unwrap_or(s.len());
    let mantissa = &s[..exp_at];
    let dot_at = mantissa
        .iter()
        .position(|&byte| byte == b'.')
        .unwrap_or(mantissa.len());
    let int_part = &mantissa[..dot_at];
    let frac_part = if dot_at < mantissa.len() {
        &mantissa[dot_at + 1..]
    } else {
        b""
    };

    let mut exp = if exp_at == s.len() {
        0
    } else {
        let Ok(text) = std::str::from_utf8(&s[exp_at + 1..]) else {
            return s;
        };
        let Ok(exp) = text.parse::<i64>() else {
            return s;
        };
        if !(-100_000..=100_000).contains(&exp) {
            return s;
        }
        exp
    };
    exp -= frac_part.len() as i64;

    let mut digits = [0; 128];
    if int_part.len() + frac_part.len() > digits.len() {
        return s;
    }
    digits[..int_part.len()].copy_from_slice(int_part);
    digits[int_part.len()..int_part.len() + frac_part.len()].copy_from_slice(frac_part);
    let mut start = 0;
    let mut end = int_part.len() + frac_part.len();
    while start < end && digits[start] == b'0' {
        start += 1;
    }
    if start == end {
        return b"0";
    }
    while digits[end - 1] == b'0' {
        end -= 1;
        exp += 1;
    }
    let d = &digits[start..end];

    if exp == 0 {
        scratch[..d.len()].copy_from_slice(d);
        return if d.len() <= s.len() {
            &scratch[..d.len()]
        } else {
            s
        };
    }

    let exp_digits = exp.unsigned_abs().ilog10() as usize + 1 + usize::from(exp < 0);
    let exp_len = d.len() + 1 + exp_digits;
    let out_len = if exp_len < fixed_len(d.len(), exp, scratch.len()) {
        scratch[..d.len()].copy_from_slice(d);
        scratch[d.len()] = b'e';
        let mut cursor = d.len() + 1;
        let mut magnitude = exp.unsigned_abs();
        if exp < 0 {
            scratch[cursor] = b'-';
            cursor += 1;
        }
        let digits_start = cursor;
        loop {
            scratch[cursor] = b'0' + (magnitude % 10) as u8;
            cursor += 1;
            magnitude /= 10;
            if magnitude == 0 {
                break;
            }
        }
        scratch[digits_start..cursor].reverse();
        cursor
    } else {
        let Some(out_len) = write_fixed(scratch, d, exp) else {
            return s;
        };
        out_len
    };
    if out_len <= s.len() {
        &scratch[..out_len]
    } else {
        s
    }
}

fn fixed_len(dlen: usize, exp: i64, cap: usize) -> usize {
    if exp > 0 {
        let exp = exp as usize;
        if exp <= cap { dlen + exp } else { usize::MAX }
    } else {
        let fraction = (-exp) as usize;
        if fraction < dlen {
            dlen + 1
        } else if fraction < cap {
            1 + fraction
        } else {
            usize::MAX
        }
    }
}

fn write_fixed(scratch: &mut [u8], d: &[u8], exp: i64) -> Option<usize> {
    if exp > 0 {
        let exp = exp as usize;
        if d.len() + exp > scratch.len() {
            return None;
        }
        scratch[..d.len()].copy_from_slice(d);
        scratch[d.len()..d.len() + exp].fill(b'0');
        return Some(d.len() + exp);
    }
    let fraction = (-exp) as usize;
    if fraction < d.len() {
        let head = d.len() - fraction;
        if d.len() + 1 > scratch.len() {
            return None;
        }
        scratch[..head].copy_from_slice(&d[..head]);
        scratch[head] = b'.';
        scratch[head + 1..d.len() + 1].copy_from_slice(&d[head..]);
        return Some(d.len() + 1);
    }
    let zeros = fraction - d.len();
    if 1 + fraction > scratch.len() {
        return None;
    }
    scratch[0] = b'.';
    scratch[1..1 + zeros].fill(b'0');
    scratch[1 + zeros..1 + fraction].copy_from_slice(d);
    Some(1 + fraction)
}

pub fn is_jsdoc_body(value: &[u8]) -> bool {
    let mut lines = value.split(|&byte| byte == b'\n');
    lines.next();
    let mut multi = false;
    for line in lines {
        multi = true;
        let trimmed = line
            .iter()
            .copied()
            .find(|byte| !matches!(byte, b' ' | b'\t'));
        if trimmed.is_some_and(|byte| byte != b'*') {
            return false;
        }
    }
    multi
}

pub fn is_significant_block_comment(value: &[u8]) -> bool {
    if value
        .first()
        .is_some_and(|byte| matches!(byte, b'!' | b'*' | b'#' | b'@'))
    {
        return true;
    }
    let mut start = 0;
    while let Some(offset) = value[start..].iter().position(|&byte| byte == b'@') {
        let rest = &value[start + offset..];
        if rest.starts_with(b"@license")
            || rest.starts_with(b"@preserve")
            || rest.starts_with(b"@cc_on")
        {
            return true;
        }
        start += offset + 1;
    }
    false
}
