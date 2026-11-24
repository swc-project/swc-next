//! Print the semantic values of link destinations and titles using Markdown delimiters.

use std::borrow::Cow;

use swc_next_markdown_parser::{BlockKind, character_reference};
use swc_next_simd::memchr::AsciiByteSet;

pub(super) struct Decoded<'a> {
    pub value: Cow<'a, str>,
    contains_encoded_line_break: bool,
}

impl Decoded<'_> {
    // Block and table boundaries are recognized before inline character references.
    fn keep_source_spelling(&self, block: &BlockKind) -> bool {
        self.contains_encoded_line_break
            || matches!(block, BlockKind::Table(_)) && self.value.contains('|')
            // A literal spelling such as the text produced by `&amp;#10;`
            // must not turn into a line ending on the next parse.
            || matches!(self.value, Cow::Owned(_))
                && self.value.match_indices('&').any(|(offset, _)| {
                    character_reference(&self.value[offset..])
                        .is_some_and(|(_, value)| matches!(&*value, "\r" | "\n"))
                })
    }
}

pub(super) fn decoded(raw: &str) -> Decoded<'_> {
    let mut rest = raw;
    let mut written = 0;
    let mut output = String::new();
    let mut contains_encoded_line_break = false;
    while let Some(index) = rest.find(['\\', '&']) {
        let token = &rest[index..];
        let replacement = if token.starts_with('\\') {
            token
                .as_bytes()
                .get(1)
                .filter(|b| b.is_ascii_punctuation())
                .map(|_| (&token[..2], Cow::Borrowed(&token[1..2])))
        } else {
            character_reference(token)
        };
        if let Some((spelling, value)) = replacement {
            contains_encoded_line_break |= matches!(&*value, "\r" | "\n");
            let offset = raw.len() - token.len();
            output.push_str(&raw[written..offset]);
            output.push_str(&value);
            written = offset + spelling.len();
            rest = &raw[written..];
        } else {
            rest = &token[1..];
        }
    }
    let value = if written == 0 {
        Cow::Borrowed(raw)
    } else {
        output.push_str(&raw[written..]);
        Cow::Owned(output)
    };
    Decoded {
        value,
        contains_encoded_line_break,
    }
}

// Guard even unrecognized entity-shaped strings when printing literal ampersands.
fn reference_candidate(text: &str) -> bool {
    let bytes = text.as_bytes();
    let Some(end) = memchr::memchr(b';', &bytes[..bytes.len().min(32)]) else {
        return false;
    };
    let name = &bytes[..end];
    if let Some(number) = name.strip_prefix(b"#") {
        if let Some(hex) = number
            .strip_prefix(b"x")
            .or_else(|| number.strip_prefix(b"X"))
        {
            return (1..=6).contains(&hex.len()) && hex.iter().all(u8::is_ascii_hexdigit);
        }
        return (1..=7).contains(&number.len()) && number.iter().all(u8::is_ascii_digit);
    }
    !name.is_empty() && name.iter().all(u8::is_ascii_alphanumeric)
}

fn bare_parentheses(text: &str, definition: bool) -> bool {
    let mut depth = 0;
    for byte in text.bytes() {
        match byte {
            b'(' if !definition || depth == 3 => return false,
            b'(' => depth += 1,
            b')' if depth == 0 => return false,
            b')' => depth -= 1,
            _ => {}
        }
    }
    depth == 0
}

pub(super) fn destination<'a>(raw: &'a str, block: &BlockKind) -> Cow<'a, str> {
    let content = raw
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(raw);
    const SPECIAL: AsciiByteSet = AsciiByteSet::new(b"()<>\\&|\x7f").with_range(0, b' ');
    if !content.is_empty() {
        let mut rest = content.as_bytes();
        loop {
            let Some(index) = SPECIAL.find(rest) else {
                return Cow::Borrowed(content);
            };
            if rest[index] != b'&'
                || reference_candidate(&content[content.len() - rest.len() + index + 1..])
            {
                break;
            }
            rest = &rest[index + 1..];
        }
    }
    let decoded = decoded(content);
    if decoded.keep_source_spelling(block) {
        return Cow::Borrowed(raw);
    }
    let value = decoded.value;
    let angle = value.is_empty()
        || value.starts_with('<')
        || value.bytes().any(|b| b <= b' ' || b == 0x7f)
        || !bare_parentheses(&value, matches!(block, BlockKind::Definition(_)));
    let mut output = String::with_capacity(value.len() + 2);
    if angle {
        output.push('<');
    }
    for (i, ch) in value.char_indices() {
        let rest = &value[i + ch.len_utf8()..];
        let escape = match ch {
            '\\' => rest.as_bytes().first().is_none_or(u8::is_ascii_punctuation),
            '&' => reference_candidate(rest),
            '<' | '>' => angle,
            _ => false,
        };
        if escape {
            output.push('\\');
        }
        output.push(ch);
    }
    if angle {
        output.push('>');
    }
    Cow::Owned(output)
}

pub(super) fn title(raw: &str, prefer_single: bool, block: &BlockKind) -> Option<String> {
    let content = raw.get(1..raw.len().checked_sub(1)?)?;
    if content.is_empty() {
        return None;
    }
    let decoded = decoded(content);
    if decoded.keep_source_spelling(block) {
        return Some(raw.to_owned());
    }
    let value = decoded.value;
    let singles = value.bytes().filter(|&b| b == b'\'').count();
    let doubles = value.bytes().filter(|&b| b == b'"').count();
    let (open, close) = if singles > 0 && doubles > 0 && !value.contains(['(', ')']) {
        ('(', ')')
    } else if singles < doubles || (singles == doubles && prefer_single) {
        ('\'', '\'')
    } else {
        ('"', '"')
    };
    let mut output = String::with_capacity(value.len() + 2);
    output.push(open);
    for (i, ch) in value.char_indices() {
        if ch == '\\' || ch == close || (ch == '&' && reference_candidate(&value[i + 1..])) {
            output.push('\\');
        }
        output.push(ch);
    }
    output.push(close);
    Some(output)
}
