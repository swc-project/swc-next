//! Parsing JavaScript numeric literals into their ECMAScript `Number` value.

use std::borrow::Cow;

pub(crate) fn parse_decimal_literal(raw: &str) -> f64 {
    const MAX_FAST_DECIMAL_LEN: usize = 19;

    if let [byte] = raw.as_bytes() {
        return f64::from(decimal_digit(*byte));
    }

    if raw.len() <= MAX_FAST_DECIMAL_LEN {
        let mut value = 0_u64;
        for byte in raw.bytes() {
            match byte {
                b'0'..=b'9' => {
                    value *= 10;
                    value += u64::from(decimal_digit(byte));
                }
                b'_' => {}
                _ => return parse_float(raw, raw.as_bytes().contains(&b'_')),
            }
        }
        return value as f64;
    }

    parse_float(raw, raw.as_bytes().contains(&b'_'))
}

pub(crate) fn parse_hex_literal(raw: &str) -> f64 {
    let has_separator = raw.as_bytes().contains(&b'_');
    parse_radix(&raw[2..], 16, 16, has_separator)
}

pub(crate) fn parse_octal_literal(raw: &str) -> f64 {
    let bytes = raw.as_bytes();
    let digits = if matches!(bytes.get(1), Some(b'o' | b'O')) {
        &raw[2..]
    } else {
        &raw[1..]
    };
    let has_separator = raw.as_bytes().contains(&b'_');
    parse_radix(digits, 8, 21, has_separator)
}

pub(crate) fn parse_binary_literal(raw: &str) -> f64 {
    let has_separator = raw.as_bytes().contains(&b'_');
    parse_radix(&raw[2..], 2, 64, has_separator)
}

fn parse_float(raw: &str, has_separator: bool) -> f64 {
    let clean = without_separators(raw, has_separator);
    clean
        .parse::<f64>()
        .expect("the lexer only produces valid decimal numeric literals")
}

fn parse_radix(raw: &str, radix: u8, max_fast_len: usize, has_separator: bool) -> f64 {
    if raw.len() > max_fast_len {
        return parse_radix_slow(raw, radix, has_separator);
    }

    let mut value = 0_u64;
    for byte in raw.bytes() {
        if byte != b'_' {
            value *= u64::from(radix);
            value += u64::from(radix_digit(byte));
        }
    }
    value as f64
}

#[cold]
#[inline(never)]
fn parse_radix_slow(raw: &str, radix: u8, has_separator: bool) -> f64 {
    debug_assert!(radix.is_power_of_two());

    // These radices encode an integer as fixed-width bit groups. Retain its
    // leading significand plus guard and sticky bits, then round exactly once.
    const SIGNIFICAND_BITS: usize = f64::MANTISSA_DIGITS as usize;
    const MAX_EXPONENT: usize = f64::MAX_EXP as usize - 1;
    const EXPONENT_BIAS: u64 = 1023;

    let bits_per_digit = radix.trailing_zeros();
    let mut significand = 0_u64;
    let mut bit_length = 0_usize;
    let mut guard = false;
    let mut sticky = false;
    let mut started = false;

    for byte in raw.bytes() {
        if has_separator && byte == b'_' {
            continue;
        }

        let digit = radix_digit(byte);
        let digit_bits = if started {
            bits_per_digit
        } else {
            if digit == 0 {
                continue;
            }
            started = true;
            u8::BITS - digit.leading_zeros()
        };

        for shift in (0..digit_bits).rev() {
            let bit = u64::from((digit >> shift) & 1);
            if bit_length < SIGNIFICAND_BITS {
                significand = (significand << 1) | bit;
            } else if bit_length == SIGNIFICAND_BITS {
                guard = bit != 0;
            } else {
                sticky |= bit != 0;
            }
            bit_length += 1;
            if bit_length > MAX_EXPONENT + 1 {
                return f64::INFINITY;
            }
        }
    }

    if bit_length == 0 {
        return 0.0;
    }
    if bit_length <= SIGNIFICAND_BITS {
        return significand as f64;
    }

    let round_up = guard && (sticky || significand & 1 != 0);
    let mut exponent = bit_length - 1;
    if round_up {
        significand += 1;
        if significand == 1 << SIGNIFICAND_BITS {
            significand >>= 1;
            exponent += 1;
        }
    }
    if exponent > MAX_EXPONENT {
        return f64::INFINITY;
    }

    let exponent = exponent as u64 + EXPONENT_BIAS;
    let fraction_mask = (1_u64 << (SIGNIFICAND_BITS - 1)) - 1;
    f64::from_bits((exponent << (SIGNIFICAND_BITS - 1)) | (significand & fraction_mask))
}

fn without_separators(raw: &str, has_separator: bool) -> Cow<'_, str> {
    if !has_separator {
        return Cow::Borrowed(raw);
    }
    let mut clean = Vec::<u8>::with_capacity(raw.len());
    let dst = clean.as_mut_ptr();
    let mut len = 0;
    for byte in raw.bytes() {
        if byte != b'_' {
            // SAFETY: Each input byte produces at most one write, so `len < raw.len()`.
            // The buffer has at least that capacity and never reallocates in this loop.
            unsafe { dst.add(len).write(byte) };
            len += 1;
        }
    }
    // SAFETY: The loop initialized exactly the first `len` bytes, within capacity.
    unsafe { clean.set_len(len) };
    // SAFETY: Removing ASCII `_` from valid UTF-8 preserves every multibyte character.
    Cow::Owned(unsafe { String::from_utf8_unchecked(clean) })
}

#[inline]
fn decimal_digit(byte: u8) -> u8 {
    debug_assert!(byte.is_ascii_digit());
    byte & 15
}

#[inline]
fn radix_digit(byte: u8) -> u8 {
    debug_assert!(byte.is_ascii_hexdigit());
    if byte.is_ascii_digit() {
        byte & 15
    } else {
        (byte & 15) + 9
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{parse_binary_literal, parse_hex_literal, parse_octal_literal, without_separators};

    #[test]
    fn numeric_separators() {
        let clean = without_separators("1_234.5_678e9_0", true);
        assert_eq!(clean, "1234.5678e90");

        let clean = without_separators("", true);
        assert_eq!(clean, "");

        let clean = without_separators("____", true);
        assert_eq!(clean, "");

        let clean = without_separators("1234", true);
        assert_eq!(clean, "1234");

        let clean = without_separators("_é_中_🦀_\0_", true);
        assert_eq!(clean, "é中🦀\0");

        let clean = without_separators("1_234", false);
        assert!(matches!(clean, Cow::Borrowed("1_234")));

        let clean = without_separators("", false);
        assert!(matches!(clean, Cow::Borrowed("")));
    }

    #[test]
    fn parse_slow_radix_boundaries() {
        let binary = format!("0b1{}1", "0".repeat(63));
        assert_eq!(
            parse_binary_literal(&binary).to_bits(),
            0x43f0_0000_0000_0000
        );
        assert_eq!(
            parse_octal_literal("0o2000000000000000000001").to_bits(),
            0x43f0_0000_0000_0000
        );
        assert_eq!(
            parse_hex_literal("0x10000000000000001").to_bits(),
            0x43f0_0000_0000_0000
        );

        // Halfway values round toward the result with an even significand.
        assert_eq!(
            parse_hex_literal("0x10000000000000800").to_bits(),
            0x43f0_0000_0000_0000
        );
        assert_eq!(
            parse_hex_literal("0x10000000000001800").to_bits(),
            0x43f0_0000_0000_0002
        );
        assert!(parse_hex_literal(&format!("0x{}", "f".repeat(256))).is_infinite());
    }
}
