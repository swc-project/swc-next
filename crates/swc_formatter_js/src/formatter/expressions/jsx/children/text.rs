//! JSX source words and whitespace, independent of the selected child layout.

use swc_next_ecma_ast::Span;

#[derive(Clone, Copy)]
pub(super) enum JsxSeparator {
    Words { spaces: usize },
    Space { spaces: usize },
    TextLine,
    Newline { empty_line: bool },
}

/// Words borrow the original JSX text. Only ASCII spaces, tabs, CR and LF
/// participate in JSX whitespace folding.
pub(super) struct JsxTextChunks<'ast> {
    remaining: &'ast str,
    span: Span,
    pending: Option<JsxTextGap>,
    pub(super) preserves_whitespace: bool,
}

pub(super) enum JsxTextChunk<'ast> {
    Word {
        text: &'ast str,
        starts_with_space: bool,
        ends_with_space: bool,
    },
    Separator(JsxSeparator),
}

struct JsxTextGap {
    len: usize,
    line_breaks: u8,
    has_tab: bool,
}

impl JsxTextGap {
    fn scan(source: &[u8]) -> Self {
        let mut len = 0;
        let mut line_breaks = 0_u8;
        let mut previous_cr = false;
        let mut has_tab = false;
        for &byte in source.iter().take_while(|&&byte| is_jsx_whitespace(byte)) {
            len += 1;
            if byte == b'\r' || byte == b'\n' && !previous_cr {
                line_breaks = (line_breaks + 1).min(2);
            }
            previous_cr = byte == b'\r';
            has_tab |= byte == b'\t';
        }
        Self {
            len,
            line_breaks,
            has_tab,
        }
    }
}

impl<'ast> JsxTextChunks<'ast> {
    pub(super) fn new(remaining: &'ast str, span: Span) -> Self {
        Self {
            remaining,
            span,
            pending: None,
            preserves_whitespace: false,
        }
    }

    fn advance(&mut self, len: usize) -> (&'ast str, Span) {
        let (value, remaining) = self.remaining.split_at(len);
        let end = self.span.end - remaining.len() as u32;
        let source = Span::new(self.span.start, end);
        self.span.start = end;
        self.remaining = remaining;
        (value, source)
    }

    fn whitespace(&mut self, gap: JsxTextGap, leading: bool) -> (JsxTextChunk<'ast>, Span) {
        let (_, source) = self.advance(gap.len);
        let trailing = self.remaining.is_empty();
        let separator = match (leading, trailing, gap.line_breaks) {
            (false, false, 0) => JsxSeparator::Words { spaces: gap.len },
            (false, false, _) => JsxSeparator::Words { spaces: 1 },
            (_, _, 0) => JsxSeparator::Space { spaces: gap.len },
            (true, true, breaks) => JsxSeparator::Newline {
                empty_line: breaks == 2,
            },
            _ => JsxSeparator::TextLine,
        };
        (JsxTextChunk::Separator(separator), source)
    }
}

impl<'ast> Iterator for JsxTextChunks<'ast> {
    type Item = (JsxTextChunk<'ast>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(gap) = self.pending.take() {
            return Some(self.whitespace(gap, false));
        }
        let bytes = self.remaining.as_bytes();
        let mut end = 0;
        let mut ends_with_space = false;
        while let Some(&byte) = bytes.get(end) {
            if !is_jsx_whitespace(byte) {
                if byte == b'&'
                    && let Some(length) = jsx_numeric_space_prefix(&bytes[end..])
                {
                    ends_with_space = true;
                    end += length;
                } else if byte.is_ascii() {
                    ends_with_space = matches!(byte, b'\x0b' | b'\x0c');
                    end += 1;
                } else {
                    let character = self.remaining[end..].chars().next().unwrap();
                    ends_with_space = is_jsx_line_edge_space(character);
                    end += character.len_utf8();
                }
                self.preserves_whitespace |= ends_with_space;
                continue;
            }
            let gap = JsxTextGap::scan(&bytes[end..]);
            // A break inside this horizontal run would trim its significant
            // spaces and tabs. Give the fill one entry for the whole word.
            if gap.line_breaks == 0
                && (gap.has_tab
                    || ends_with_space
                    || jsx_text_starts_with_edge_space(&self.remaining[end + gap.len..]))
            {
                self.preserves_whitespace = true;
                ends_with_space = true;
                end += gap.len;
                continue;
            }
            if end == 0 {
                return Some(self.whitespace(gap, true));
            }
            self.pending = Some(gap);
            break;
        }
        if end == 0 {
            return None;
        }
        // A boundary is an ASCII formatting character, so it cannot split UTF-8.
        let (word, source) = self.advance(end);
        Some((
            JsxTextChunk::Word {
                text: word,
                starts_with_space: jsx_text_starts_with_edge_space(word),
                ends_with_space,
            },
            source,
        ))
    }
}

fn is_jsx_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

fn is_jsx_line_edge_space(character: char) -> bool {
    character.is_whitespace() || matches!(character, '\u{200b}' | '\u{feff}')
}

fn jsx_text_starts_with_edge_space(text: &str) -> bool {
    text.chars().next().is_some_and(is_jsx_line_edge_space)
        || jsx_numeric_space_prefix(text.as_bytes()).is_some()
}

// JSX transforms can decode these references before trimming physical lines.
fn jsx_numeric_space_prefix(text: &[u8]) -> Option<usize> {
    let digits = text.strip_prefix(b"&#")?;
    let (digits, radix) = if let Some(digits) = digits
        .strip_prefix(b"x")
        .or_else(|| digits.strip_prefix(b"X"))
    {
        (digits, 16)
    } else {
        (digits, 10)
    };
    let mut value = 0;
    for (index, &byte) in digits.iter().enumerate() {
        if byte == b';' && matches!(value, 9 | 32) {
            return Some(text.len() - digits.len() + index + 1);
        }
        value = value * radix + char::from(byte).to_digit(radix)?;
        if value > 32 {
            break;
        }
    }
    None
}

pub(super) fn jsx_text_may_need_edge_protection(text: &str) -> bool {
    !text.is_ascii()
        || text.contains("&#")
        || memchr::memchr3(b'\t', b'\x0b', b'\x0c', text.as_bytes()).is_some()
}
