//! Cached display widths for single-line borrowed text.

use crate::text_width;

use super::thin::ThinStr;

#[derive(Debug, Clone, Copy)]
pub(super) struct Text<'a> {
    source: &'a str,
    width: Width,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum WidthKind {
    Ascii,
    Unicode,
    Dynamic,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub(super) struct Width {
    kind: WidthKind,
    value: u16,
}

impl<'a> Text<'a> {
    #[inline]
    pub(super) fn new(source: &'a str) -> Self {
        let width = if text_width::ascii_width(source).is_some() {
            Width {
                kind: WidthKind::Ascii,
                value: 0,
            }
        } else if source.chars().any(char::is_control) {
            debug_assert!(
                !source.contains(['\r', '\n']),
                "physical lines must use literal_text or a line instruction"
            );
            Width {
                kind: WidthKind::Dynamic,
                value: 0,
            }
        } else {
            Width {
                kind: WidthKind::Unicode,
                value: u16::try_from(text_width::unicode_width(source)).unwrap_or(u16::MAX),
            }
        };
        Self { source, width }
    }

    pub(super) fn from_parts(source: ThinStr<'a>, width: Width) -> Self {
        Self {
            source: source.as_str(),
            width,
        }
    }

    pub(super) const fn into_parts(self) -> (ThinStr<'a>, Width) {
        (ThinStr::new(self.source), self.width)
    }

    pub(super) fn source(self) -> &'a str {
        self.source
    }

    pub(super) const fn from_source_parts(source: &'a str, width: Width) -> Self {
        Self { source, width }
    }

    pub(super) const fn width(self) -> Width {
        self.width
    }

    pub(super) fn is_single_line(self) -> bool {
        match self.width.kind {
            WidthKind::Ascii => true,
            _ => !self.source().contains(['\n', '\r', '\u{2028}', '\u{2029}']),
        }
    }

    /// Textual width has no column context and does not prove a flat render view.
    pub(super) fn clean_width(self) -> Option<usize> {
        Some(match self.width.kind {
            WidthKind::Ascii => self.source().len(),
            WidthKind::Unicode => usize::from(self.width.value),
            WidthKind::Dynamic => {
                if self.source().contains(['\r', '\n']) {
                    return None;
                }
                text_width::width(self.source())
            }
        })
    }

    pub(super) fn clean_ascii_width(self) -> Option<usize> {
        match self.width.kind {
            WidthKind::Ascii => Some(self.source().len()),
            _ => None,
        }
    }

    pub(super) fn first_line(self, column: usize) -> (usize, bool) {
        match self.width.kind {
            WidthKind::Ascii => (column + self.source().len(), false),
            WidthKind::Unicode => (column + usize::from(self.width.value), false),
            WidthKind::Dynamic => advance(self.source(), column),
        }
    }

    /// Returns the final column and whether the text crossed a physical line break.
    pub(super) fn last_line(self, column: usize) -> (usize, bool) {
        if matches!(self.width.kind, WidthKind::Dynamic)
            && let Some((index, character)) = self
                .source()
                .char_indices()
                .rfind(|(_, character)| matches!(character, '\r' | '\n'))
        {
            return (
                advance(&self.source()[index + character.len_utf8()..], 0).0,
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
