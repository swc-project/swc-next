//! Line boundaries in source gaps, including ECMAScript line terminators.

use swc_next_ecma_ast::Span;

use super::Formatter;

pub(super) fn is_source_whitespace(character: char) -> bool {
    character.is_whitespace() || character == '\u{feff}'
}

pub(super) fn split_line(source: &str) -> (&str, Option<&str>) {
    let Some(index) = source.find(['\n', '\r']) else {
        return (source, None);
    };
    let is_crlf =
        source.as_bytes()[index] == b'\r' && source.as_bytes().get(index + 1) == Some(&b'\n');
    let end = index + 1 + usize::from(is_crlf);
    (&source[..index], Some(&source[end..]))
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
    pub(super) fn source_position_starts_line(&self, position: u32) -> bool {
        let before = &self.ast.source()[..position as usize];
        let line = match before
            .char_indices()
            .rev()
            .find(|(_, character)| matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
        {
            Some((index, character)) => &before[index + character.len_utf8()..],
            None => before,
        };
        line.chars().all(is_source_whitespace)
    }

    pub(super) fn source(&self, span: Span) -> &'ast str {
        &self.ast.source()[span.start as usize..span.end as usize]
    }

    pub(super) fn next_line_is_empty(&self, start: u32, end: u32) -> bool {
        let mut characters = self.source(Span::new(start, end)).chars().peekable();
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
            // A separator on the next line is content, not an empty source line.
            return characters
                .find(|character| !matches!(character, ' ' | '\t'))
                .is_some_and(|character| {
                    matches!(character, '\r' | '\n' | '\u{2028}' | '\u{2029}')
                });
        }
        false
    }

    pub(super) fn line_breaks(&self, start: u32, end: u32) -> u8 {
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
