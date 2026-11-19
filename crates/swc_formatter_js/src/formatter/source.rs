//! Visual line boundaries and ECMAScript source line queries.

use std::ops::Range;

use swc_next_ecma_ast::{Comment, Span};

use super::Formatter;

pub(super) fn is_source_whitespace(character: char) -> bool {
    character.is_whitespace() || character == '\u{feff}'
}

pub(super) fn is_source_horizontal_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}' | '\u{000b}' | '\u{000c}' | ' ' | '\u{00a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
    )
}

// Layout uses CR/LF. Semantic source queries use the ECMAScript terminators.
#[inline]
fn layout_line_ending(source: &[u8]) -> Option<Range<usize>> {
    let start = memchr::memchr2(b'\n', b'\r', source)?;
    let end = start + 1;
    let crlf = source[start] == b'\r' && source.get(end) == Some(&b'\n');
    Some(start..end + usize::from(crlf))
}

pub(super) fn split_line(source: &str) -> (&str, Option<&str>) {
    let Some(ending) = layout_line_ending(source.as_bytes()) else {
        return (source, None);
    };
    (&source[..ending.start], Some(&source[ending.end..]))
}

pub(super) fn lines(source: &str) -> impl Iterator<Item = &str> {
    let mut remaining = Some(source);
    std::iter::from_fn(move || {
        let source = remaining?;
        let (line, rest) = split_line(source);
        remaining = rest;
        Some(line)
    })
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn layout_position_starts_line(&self, position: u32) -> bool {
        for character in self.ast.source()[..position as usize].chars().rev() {
            match character {
                '\n' | '\r' => return true,
                _ if !is_source_horizontal_whitespace(character) => return false,
                _ => {}
            }
        }
        true
    }

    pub(super) fn layout_line_starts_line(&self, position: u32) -> bool {
        self.ast.source()[..position as usize]
            .trim_end_matches([' ', '\t'])
            .ends_with(['\n', '\r'])
    }

    /// Semantic source ownership includes every ECMAScript line terminator.
    pub(super) fn source_position_starts_line(&self, position: u32) -> bool {
        let before = &self.ast.source()[..position as usize];
        for character in before.chars().rev() {
            match character {
                '\n' | '\r' | '\u{2028}' | '\u{2029}' => return true,
                _ if !is_source_whitespace(character) => return false,
                _ => {}
            }
        }
        true
    }

    pub(super) fn find_comma(&self, span: Span, comments: &[Comment]) -> Option<u32> {
        self.find_token(span, comments, ",")
    }

    pub(super) fn find_token(&self, span: Span, comments: &[Comment], token: &str) -> Option<u32> {
        if let [byte] = token.as_bytes() {
            return self.find_token_in_gaps(span, comments, |source| memchr::memchr(*byte, source));
        }
        if comments.is_empty() {
            return memchr::memmem::find(self.source(span).as_bytes(), token.as_bytes())
                .map(|offset| span.start + offset as u32);
        }
        let token = memchr::memmem::Finder::new(token.as_bytes());
        self.find_token_in_gaps(span, comments, |source| token.find(source))
    }

    fn find_token_in_gaps(
        &self,
        span: Span,
        comments: &[Comment],
        find: impl Fn(&[u8]) -> Option<usize>,
    ) -> Option<u32> {
        let mut start = span.start;
        for comment in comments {
            if comment.span.end <= start {
                continue;
            }
            if comment.span.start >= span.end {
                break;
            }
            if let Some(offset) = find(self.source(Span::new(start, comment.span.start)).as_bytes())
            {
                return Some(start + offset as u32);
            }
            start = comment.span.end;
        }
        find(self.source(Span::new(start, span.end)).as_bytes()).map(|offset| start + offset as u32)
    }

    pub(super) fn source(&self, span: Span) -> &'ast str {
        &self.ast.source()[span.start as usize..span.end as usize]
    }

    pub(super) fn next_line_is_empty(&self, start: u32, end: u32) -> bool {
        let source = &self.ast.source().as_bytes()[start as usize..end as usize];
        let Some(ending) = layout_line_ending(source) else {
            return false;
        };
        source[ending.end..]
            .iter()
            .find(|byte| !matches!(byte, b' ' | b'\t'))
            .is_some_and(|byte| matches!(byte, b'\r' | b'\n'))
    }

    pub(super) fn previous_line_is_empty(&self, start: u32, end: u32) -> bool {
        let gap = self
            .source(Span::new(start, end))
            .trim_end_matches([' ', '\t']);
        let Some(previous) = gap
            .strip_suffix("\r\n")
            .or_else(|| gap.strip_suffix(['\r', '\n']))
        else {
            return false;
        };
        previous
            .trim_end_matches([' ', '\t'])
            .ends_with(['\r', '\n'])
    }

    /// Layout counts CR/LF independently of semantic source line boundaries.
    pub(super) fn line_breaks(&self, start: u32, end: u32) -> u8 {
        let source = &self.ast.source().as_bytes()[start as usize..end as usize];
        let Some(ending) = layout_line_ending(source) else {
            return 0;
        };
        1 + u8::from(memchr::memchr2(b'\n', b'\r', &source[ending.end..]).is_some())
    }

    /// Syntax and tool directives retain every ECMAScript line terminator.
    pub(super) fn source_line_breaks(&self, start: u32, end: u32) -> u8 {
        let mut characters = self.source(Span::new(start, end)).chars().peekable();
        let mut count = 0;
        while let Some(character) = characters.next() {
            match character {
                '\r' => {
                    if characters.peek() == Some(&'\n') {
                        characters.next();
                    }
                }
                '\n' | '\u{2028}' | '\u{2029}' => {}
                _ => continue,
            }
            count += 1;
            if count == 2 {
                break;
            }
        }
        count
    }
}
