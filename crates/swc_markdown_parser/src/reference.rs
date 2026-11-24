use std::borrow::Cow;

use html_escape::NAMED_ENTITIES;

/// Decode one semicolon-terminated character reference at the start of the input.
/// The first result is the exact source spelling, including its delimiters.
pub fn character_reference(source: &str) -> Option<(&str, Cow<'static, str>)> {
    let (spelling, value) = recognize(source)?;
    let decoded = match value {
        Value::Named(text) => Cow::Borrowed(text),
        Value::Scalar(ch) => Cow::Owned(ch.to_string()),
    };
    Some((spelling, decoded))
}

pub(crate) fn length(source: &str) -> Option<usize> {
    recognize(source).map(|(spelling, _)| spelling.len())
}

enum Value {
    Named(&'static str),
    Scalar(char),
}

fn recognize(source: &str) -> Option<(&str, Value)> {
    let rest = source.strip_prefix('&')?;
    // The longest named reference has 31 ASCII bytes; numeric forms have at most 7 digits.
    let end = rest.as_bytes().iter().take(32).position(|&b| b == b';')?;
    let name = &rest[..end];
    let value = if let Some(number) = name.strip_prefix('#') {
        let (digits, base, maximum) = match number.strip_prefix(['x', 'X']) {
            Some(digits) => (digits, 16, 6),
            None => (number, 10, 7),
        };
        if digits.is_empty() || digits.len() > maximum || !digits.as_bytes()[0].is_ascii_hexdigit()
        {
            return None;
        }
        let code = u32::from_str_radix(digits, base).ok()?;
        let disallowed =
            matches!(code,0..=8|11|14..=31|127..=159|0xfdd0..=0xfdef) || code & 0xffff >= 0xfffe;
        Value::Scalar(
            char::from_u32(code)
                .filter(|_| !disallowed)
                .unwrap_or(char::REPLACEMENT_CHARACTER),
        )
    } else {
        let index = NAMED_ENTITIES
            .binary_search_by_key(&name.as_bytes(), |(entity, _)| *entity)
            .ok()?;
        Value::Named(NAMED_ENTITIES[index].1)
    };
    Some((&source[..end + 2], value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_references() {
        let (raw, text) = character_reference("&NotEqualTilde; another").unwrap();
        assert_eq!(raw, "&NotEqualTilde;");
        assert_eq!(text, "≂\u{338}");

        assert!(character_reference("&unknown;").is_none());
        assert!(character_reference("&amp").is_none());
        assert!(character_reference("x&amp;").is_none());
    }

    #[test]
    fn numeric_references() {
        assert_eq!(character_reference("&#X1F980;").unwrap().1, "🦀");

        assert_eq!(character_reference("&#x0b;").unwrap().1, "�");
        assert_eq!(character_reference("&#127;").unwrap().1, "�");
        assert_eq!(character_reference("&#xFFFFE;").unwrap().1, "�");
        assert_eq!(character_reference("&#55296;").unwrap().1, "�");
        assert_eq!(character_reference("&#9;").unwrap().1, "\t");

        assert!(character_reference("&#x0000000;").is_none());
        assert!(character_reference("&#+123;").is_none());
    }
}
