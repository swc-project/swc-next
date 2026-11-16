//! Decimal components retain their source spelling unless normalization is needed.

use std::borrow::Cow;

struct Exponent<'a> {
    sign: &'a str,
    digits: &'a str,
}

pub(super) fn lowercase(text: &str) -> Cow<'_, str> {
    if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
        Cow::Owned(text.to_ascii_lowercase())
    } else {
        Cow::Borrowed(text)
    }
}

pub(super) fn format_number(raw: &str) -> Cow<'_, str> {
    let text = lowercase(raw);
    if text.starts_with("0x") || text.starts_with("0o") || text.starts_with("0b") {
        return text;
    }
    let (mantissa, exponent) = match text.split_once('e') {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (text.as_ref(), None),
    };
    let (integer, fraction) = match mantissa.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (mantissa, None),
    };
    let normalized_integer = if integer.is_empty() { "0" } else { integer };
    let normalized_fraction = fraction.filter(|text| !text.is_empty()).map(|text| {
        if text.bytes().all(|byte| byte.is_ascii_digit()) {
            let trimmed = text.trim_end_matches('0');
            if trimmed.is_empty() { "0" } else { trimmed }
        } else {
            text
        }
    });
    let normalized_exponent = exponent.and_then(|text| normalize_exponent(mantissa, text));
    let exponent_unchanged = match (exponent, normalized_exponent.as_ref()) {
        (None, None) => true,
        (Some(text), Some(normalized)) => {
            text.strip_prefix(normalized.sign) == Some(normalized.digits)
        }
        _ => false,
    };
    if integer == normalized_integer && fraction == normalized_fraction && exponent_unchanged {
        return text;
    }

    let mut output = String::with_capacity(text.len() + 1);
    output.push_str(normalized_integer);
    if let Some(fraction) = normalized_fraction {
        output.push('.');
        output.push_str(fraction);
    }
    if let Some(exponent) = normalized_exponent {
        output.push('e');
        output.push_str(exponent.sign);
        output.push_str(exponent.digits);
    }
    Cow::Owned(output)
}

fn normalize_exponent<'a>(mantissa: &str, text: &'a str) -> Option<Exponent<'a>> {
    let mantissa = match mantissa.as_bytes().first() {
        Some(b'+' | b'-') => &mantissa[1..],
        _ => mantissa,
    };
    if !mantissa
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return Some(Exponent {
            sign: "",
            digits: text,
        });
    }
    let (sign, digits) = match text.as_bytes().first() {
        Some(b'-') => ("-", &text[1..]),
        Some(b'+') => ("", &text[1..]),
        _ => ("", text),
    };
    if digits.bytes().all(|byte| byte == b'0') {
        return None;
    }
    let zeros = digits.bytes().take_while(|byte| *byte == b'0').count();
    let remove = if digits.as_bytes().get(zeros).is_some_and(u8::is_ascii_digit) {
        zeros
    } else {
        // A separator after leading zeroes still requires its preceding digit.
        zeros.saturating_sub(1)
    };
    Some(Exponent {
        sign,
        digits: &digits[remove..],
    })
}
