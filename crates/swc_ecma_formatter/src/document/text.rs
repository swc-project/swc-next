//! Cached display widths for borrowed text; physical breaks retain column context.

use crate::text_width;

#[derive(Debug, Clone, Copy)]
pub(super) struct Text<'a> {
    pub(super) source: &'a str,
    width: Width,
}

#[derive(Debug, Clone, Copy)]
enum Width {
    Ascii,
    Unicode(u32),
    Dynamic,
}

impl<'a> Text<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        let width = if text_width::ascii_width(source).is_some() {
            Width::Ascii
        } else if source.chars().any(char::is_control) {
            Width::Dynamic
        } else {
            Width::Unicode(u32::try_from(text_width::unicode_width(source)).unwrap_or(u32::MAX))
        };
        Self { source, width }
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(super) fn is_single_line(self) -> bool {
        match self.width {
            Width::Ascii => true,
            _ => !self.source.contains(['\n', '\r', '\u{2028}', '\u{2029}']),
        }
    }

    /// Textual width has no column context and does not prove a flat render view.
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(super) fn clean_width(self) -> Option<usize> {
        Some(match self.width {
            Width::Ascii => self.source.len(),
            Width::Unicode(width) => width as usize,
            Width::Dynamic => {
                if self.source.contains(['\r', '\n']) {
                    return None;
                }
                text_width::width(self.source)
            }
        })
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(super) fn clean_ascii_width(self) -> Option<usize> {
        match self.width {
            Width::Ascii => Some(self.source.len()),
            _ => None,
        }
    }

    pub(super) fn first_line(self, column: usize) -> (usize, bool) {
        match self.width {
            Width::Ascii => (column + self.source.len(), false),
            Width::Unicode(width) => (column + width as usize, false),
            Width::Dynamic => advance(self.source, column),
        }
    }

    /// Returns the final column and whether the text crossed a physical line break.
    pub(super) fn last_line(self, column: usize) -> (usize, bool) {
        if matches!(self.width, Width::Dynamic)
            && let Some((index, character)) = self
                .source
                .char_indices()
                .rfind(|(_, character)| matches!(character, '\r' | '\n'))
        {
            return (
                advance(&self.source[index + character.len_utf8()..], 0).0,
                true,
            );
        }
        self.first_line(column)
    }
}

fn advance(mut source: &str, mut column: usize) -> (usize, bool) {
    while let Some((index, character)) = source
        .char_indices()
        .find(|(_, character)| character.is_control())
    {
        column += text_width::unicode_width(&source[..index]);
        if matches!(character, '\r' | '\n') {
            return (column, true);
        }
        source = &source[index + character.len_utf8()..];
    }
    (column + text_width::unicode_width(source), false)
}
