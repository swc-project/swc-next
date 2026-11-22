//! JSX whitespace handling ported from SWC's React transform.

use swc_next_allocator::wtf8::{Wtf8, Wtf8Buf};

use super::entities::decode_jsx_entity;

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_parser/src/lexer/state.rs#L607>.
pub(super) fn decode_entities(mut raw: &str) -> Wtf8Buf {
    let mut value = Wtf8Buf::with_capacity(raw.len());
    while let Some(character) = raw.chars().next() {
        if let Some((code_point, len)) = decode_jsx_entity(raw) {
            value.push(code_point);
            raw = &raw[len..];
        } else {
            value.push_char(character);
            raw = &raw[character.len_utf8()..];
        }
    }
    value
}

/// Decode quoted attribute contents with SWC's `read_jsx_str` chunk boundaries.
/// In particular, preserve its CRLF chunk-start behavior instead of applying
/// JSX child whitespace normalization to attribute strings.
///
/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_parser/src/lexer/mod.rs#L1427>.
pub(super) fn decode_attribute(raw: &str) -> Wtf8Buf {
    let mut value = Wtf8Buf::new();
    let mut chunk_start = 0;
    let mut position = 0;
    while let Some(character) = raw[position..].chars().next() {
        if character == '\\' {
            value.push_str(&raw[chunk_start..position]);
            value.push_char(character);
            position += character.len_utf8();
            chunk_start = position;
        } else if character == '&' {
            value.push_str(&raw[chunk_start..position]);
            if let Some((code_point, len)) = decode_jsx_entity(&raw[position..]) {
                value.push(code_point);
                position += len;
            } else {
                value.push_char('&');
                position += 1;
            }
            chunk_start = position;
        } else if is_line_terminator(character) {
            value.push_str(&raw[chunk_start..position]);
            chunk_start = position + character.len_utf8();
            if raw[position..].starts_with("\r\n") {
                value.push_str("\r\n");
                position += 2;
            } else {
                value.push_char(character);
                position += character.len_utf8();
            }
        } else {
            position += character.len_utf8();
        }
    }
    value.push_str(&raw[chunk_start..]);
    value
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/str.rs#L101>.
fn is_line_terminator(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1590>.
pub(super) fn jsx_text_to_str_with_raw(value: &Wtf8, raw: &str) -> Wtf8Buf {
    // Fast path: if no HTML entities (raw == value), use the simple algorithm
    if let Some(value) = value.as_str()
        && value == raw
    {
        return Wtf8Buf::from_string(jsx_text_to_str_impl(value));
    }
    // Build a mask of which code point positions in value came from HTML
    // entities.
    let entity_mask = build_entity_mask(value, raw);

    jsx_text_to_str_with_entity_mask(value, &entity_mask)
}

/// Build a mask indicating which character positions in `value` came from HTML
/// entities in `raw`.
///
/// Returns a Vec<bool> where true means the character at that index was from an
/// entity.
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1609>.
fn build_entity_mask(value: &Wtf8, mut raw: &str) -> Vec<bool> {
    let mut mask = Vec::with_capacity(value.code_points().count());
    let mut previous_is_lead_surrogate = false;

    while let Some(raw_char) = raw.chars().next() {
        let entity = if raw_char == '&' {
            decode_jsx_entity(raw)
        } else {
            None
        };
        let Some((code_point, len)) = entity else {
            raw = &raw[raw_char.len_utf8()..];
            mask.push(false);
            previous_is_lead_surrogate = false;
            continue;
        };
        raw = &raw[len..];
        let code_point_value = code_point.to_u32();

        // Wtf8Buf::push folds an adjacent surrogate pair into one code point.
        // Both halves come from entities because raw text is valid UTF-8.
        if !previous_is_lead_surrogate || !matches!(code_point_value, 0xdc00..=0xdfff) {
            mask.push(true);
        }
        previous_is_lead_surrogate = matches!(code_point_value, 0xd800..=0xdbff);
    }

    mask
}

/// JSX text processing with entity mask - preserves whitespace from HTML
/// entities.
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1641>.
fn jsx_text_to_str_with_entity_mask(t: &Wtf8, entity_mask: &[bool]) -> Wtf8Buf {
    // Fast path: if no line terminators and no trimmable whitespace
    // (whitespace that's not from entities at the leading edge)
    let chars: Vec<_> = wtf8_chars(t).collect();
    let has_line_terminator = chars.iter().any(|(_, _, c)| is_line_terminator(*c));

    // For single-line text, we keep all whitespace (matching original behavior)
    // The original jsx_text_to_str_impl preserves leading/trailing whitespace on
    // single-line text
    if !t.is_empty() && !has_line_terminator {
        return t.to_owned();
    }

    let mut acc: Option<Wtf8Buf> = None;
    let mut only_line: Option<(usize, usize)> = None;
    let mut line_start: Option<usize> = Some(0);
    let mut line_end: Option<usize> = None;
    // The first line preserves leading whitespace; subsequent lines trim it.
    let mut is_first_line = true;

    for (char_idx, (_, _, c)) in chars.iter().enumerate() {
        let is_from_entity = *entity_mask.get(char_idx).unwrap_or(&false);

        if is_line_terminator(*c) {
            // Process current line - trim both leading AND trailing (intermediate
            // line)
            if let (Some(start), Some(end)) = (line_start, line_end) {
                let line_range =
                    extract_line_content(&chars, start, end, entity_mask, !is_first_line, true);
                if let Some((line_start, line_end)) = line_range {
                    add_line_of_jsx_text_wtf8(line_start, line_end, t, &mut acc, &mut only_line);
                }
            }
            is_first_line = false;
            line_start = None;
            line_end = None;
        } else if !is_white_space_single_line(*c) || is_from_entity {
            // Non-whitespace or entity-derived whitespace - counts as content
            line_end = Some(char_idx + 1);
            if line_start.is_none() {
                line_start = Some(char_idx);
            }
        }
    }

    // Handle final line. Leading whitespace is preserved only if this is still
    // the first line (single-line input).
    if let Some(start) = line_start {
        let line_range = extract_line_content(
            &chars,
            start,
            chars.len(),
            entity_mask,
            !is_first_line,
            false,
        );
        if let Some((line_start, line_end)) = line_range {
            add_line_of_jsx_text_wtf8(line_start, line_end, t, &mut acc, &mut only_line);
        }
    }

    if let Some(acc) = acc {
        acc
    } else if let Some((start, end)) = only_line {
        t.slice(start, end).to_owned()
    } else {
        Wtf8Buf::default()
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1711>.
fn wtf8_chars(t: &Wtf8) -> impl Iterator<Item = (usize, usize, char)> + '_ {
    let mut byte_pos = 0;

    t.code_points().map(move |cp| {
        let start = byte_pos;
        let cp_value = cp.to_u32();
        let cp_byte_len = if cp_value < 0x80 {
            1
        } else if cp_value < 0x800 {
            2
        } else if cp_value < 0x10000 {
            3
        } else {
            4
        };
        byte_pos += cp_byte_len;

        (start, byte_pos, cp.to_char_lossy())
    })
}

/// Extract line content, optionally trimming non-entity whitespace from edges
///
/// - `trim_leading`: if true, trim leading non-entity whitespace
/// - `trim_trailing`: if true, trim trailing non-entity whitespace
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1736>.
fn extract_line_content(
    chars: &[(usize, usize, char)],
    start: usize,
    end: usize,
    entity_mask: &[bool],
    trim_leading: bool,
    trim_trailing: bool,
) -> Option<(usize, usize)> {
    // Find first non-trimmable position (if trim_leading is true)
    let mut actual_start = start;
    if trim_leading {
        while actual_start < end {
            let c = chars[actual_start].2;
            let is_from_entity = *entity_mask.get(actual_start).unwrap_or(&false);
            if !is_white_space_single_line(c) || is_from_entity {
                break;
            }
            actual_start += 1;
        }
    }

    // Find last non-trimmable position (if trim_trailing is true)
    let mut actual_end = end;
    if trim_trailing {
        while actual_end > actual_start {
            let c = chars[actual_end - 1].2;
            let is_from_entity = *entity_mask.get(actual_end - 1).unwrap_or(&false);
            if !is_white_space_single_line(c) || is_from_entity {
                break;
            }
            actual_end -= 1;
        }
    }

    if actual_start == actual_end {
        None
    } else {
        Some((chars[actual_start].0, chars[actual_end - 1].1))
    }
}

/// Helper for adding lines of JSX text when handling Wtf8 with surrogates
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1847>.
fn add_line_of_jsx_text_wtf8(
    line_start: usize,
    line_end: usize,
    source: &Wtf8,
    acc: &mut Option<Wtf8Buf>,
    only_line: &mut Option<(usize, usize)>,
) {
    if let Some((only_start, only_end)) = only_line.take() {
        // Second line - create accumulator
        let mut buffer = Wtf8Buf::with_capacity(source.len());
        buffer.push_wtf8(source.slice(only_start, only_end));
        buffer.push_str(" ");
        buffer.push_wtf8(source.slice(line_start, line_end));
        *acc = Some(buffer);
    } else if let Some(buffer) = acc {
        // Subsequent lines
        buffer.push_str(" ");
        buffer.push_wtf8(source.slice(line_start, line_end));
    } else {
        // First line
        *only_line = Some((line_start, line_end));
    }
}

/// Internal implementation that works with &str
#[inline]
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1873>.
fn jsx_text_to_str_impl(t: &str) -> String {
    // Fast path: if no line terminators and no leading/trailing whitespace
    if !t.is_empty()
        && !t.chars().any(is_line_terminator)
        && !t.starts_with(is_white_space_single_line)
        && !t.ends_with(is_white_space_single_line)
    {
        return t.to_owned();
    }

    let mut acc: Option<String> = None;
    let mut only_line: Option<&str> = None;
    let mut first_non_whitespace: Option<usize> = Some(0);
    let mut last_non_whitespace: Option<usize> = None;

    for (index, c) in t.char_indices() {
        if is_line_terminator(c) {
            if let (Some(first), Some(last)) = (first_non_whitespace, last_non_whitespace) {
                let line_text = &t[first..last];
                add_line_of_jsx_text(line_text, &mut acc, &mut only_line);
            }
            first_non_whitespace = None;
        } else if !is_white_space_single_line(c) {
            last_non_whitespace = Some(index + c.len_utf8());
            if first_non_whitespace.is_none() {
                first_non_whitespace.replace(index);
            }
        }
    }

    if let Some(first) = first_non_whitespace {
        let line_text = &t[first..];
        add_line_of_jsx_text(line_text, &mut acc, &mut only_line);
    }

    if let Some(acc) = acc {
        acc
    } else if let Some(only_line) = only_line {
        only_line.into()
    } else {
        "".into()
    }
}

/// [TODO]: Re-validate this whitespace handling logic.
///
/// We cannot use [swc_ecma_utils::str::is_white_space_single_line] because
/// HTML entities (like `&nbsp;` → `\u{00a0}`) are pre-processed by the parser,
/// making it impossible to distinguish them from literal Unicode characters. We
/// should never trim HTML entities.
///
/// As a reference, Babel only trims regular spaces and tabs, so this is a
/// simplified implementation already in use.
/// https://github.com/babel/babel/blob/e5c8dc7330cb2f66c37637677609df90b31ff0de/packages/babel-types/src/utils/react/cleanJSXElementLiteralChild.ts#L28-L39
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1927>.
fn is_white_space_single_line(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

// less allocations trick from OXC
// https://github.com/oxc-project/oxc/blob/4c35f4abb6874bd741b84b34df7889637425e9ea/crates/oxc_transformer/src/jsx/jsx_impl.rs#L1061-L1091
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1933>.
fn add_line_of_jsx_text<'a>(
    trimmed_line: &'a str,
    acc: &mut Option<String>,
    only_line: &mut Option<&'a str>,
) {
    if let Some(buffer) = acc.as_mut() {
        // Already some text in accumulator. Push a space before this line is added to
        // `acc`.
        buffer.push(' ');
    } else if let Some(only_line_content) = only_line.take() {
        // This is the 2nd line containing text. Previous line did not contain any HTML
        // entities. Generate an accumulator containing previous line and a
        // trailing space. Current line will be added to the accumulator after
        // it.
        let mut buffer = String::with_capacity(trimmed_line.len() * 2); // rough estimate
        buffer.push_str(only_line_content);
        buffer.push(' ');
        *acc = Some(buffer);
    }

    // [TODO]: Decode any HTML entities in this line

    // For now, just use the trimmed line directly
    if let Some(buffer) = acc.as_mut() {
        buffer.push_str(trimmed_line);
    } else {
        // This is the first line containing text, and there are no HTML entities in
        // this line. Record this line in `only_line`.
        // If this turns out to be the only line, we won't need to construct a String,
        // so avoid all copying.
        *only_line = Some(trimmed_line);
    }
}
