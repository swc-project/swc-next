//! Source-based string normalization preserves escape spelling and directives.

use std::borrow::Cow;

use crate::QuoteStyle;

#[derive(Clone, Copy)]
pub(in crate::formatter) enum StringContext {
    Expression,
    Directive,
}

pub(super) fn format_string(
    raw: &str,
    preferred: QuoteStyle,
    context: StringContext,
) -> Cow<'_, str> {
    let current = match raw.as_bytes() {
        [b'"', .., b'"'] => b'"',
        [b'\'', .., b'\''] => b'\'',
        _ => panic!("string literal raw must contain matching quote delimiters"),
    };
    let content = &raw[1..raw.len() - 1];
    let preferred = match preferred {
        QuoteStyle::Double => b'"',
        QuoteStyle::Single => b'\'',
    };
    let mut preferred_count = 0_usize;
    let mut alternate_count = 0_usize;
    for byte in content.bytes() {
        if byte == preferred {
            preferred_count += 1;
        } else if matches!(byte, b'"' | b'\'') {
            alternate_count += 1;
        }
    }
    let quote = match context {
        StringContext::Directive if preferred_count + alternate_count != 0 => current,
        StringContext::Directive => preferred,
        StringContext::Expression if preferred_count > alternate_count => {
            if preferred == b'"' {
                b'\''
            } else {
                b'"'
            }
        }
        StringContext::Expression => preferred,
    };
    if quote == current {
        return Cow::Borrowed(raw);
    }

    let mut output = String::with_capacity(raw.len() + 2);
    output.push(quote as char);
    match context {
        StringContext::Directive => output.push_str(content),
        StringContext::Expression => {
            let mut copy_start = 0;
            let mut bytes = content.bytes().enumerate();
            while let Some((index, byte)) = bytes.next() {
                match byte {
                    b'\\' => {
                        if let Some((escaped_index, escaped)) = bytes.next()
                            && escaped == current
                        {
                            output.push_str(&content[copy_start..index]);
                            copy_start = escaped_index;
                        }
                    }
                    _ if byte == quote => {
                        output.push_str(&content[copy_start..index]);
                        output.push('\\');
                        copy_start = index;
                    }
                    _ => {}
                }
            }
            output.push_str(&content[copy_start..]);
        }
    }
    output.push(quote as char);
    Cow::Owned(output)
}
