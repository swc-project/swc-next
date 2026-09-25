//! Line boundaries in source gaps, including ECMAScript line terminators.

use swc_next_ecma_ast::Span;

use super::Formatter;

pub(super) fn is_source_whitespace(character: char) -> bool {
    character.is_whitespace() || character == '\u{feff}'
}

impl<'ast, 'source> Formatter<'ast, 'source> {
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
