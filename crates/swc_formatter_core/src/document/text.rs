//! Cached display widths for single-line borrowed text.

use crate::measure_text_width;

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
    UnicodeWithLines,
    Dynamic,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub(super) struct Width {
    kind: WidthKind,
    value: u16,
}

impl Width {
    #[cold]
    fn measure_slow(source: &str) -> Self {
        if memchr::memchr2(b'\r', b'\n', source.as_bytes()).is_some() {
            debug_assert!(
                !source.contains(['\r', '\n']),
                "physical lines must use literal_text or a line instruction"
            );
            Self {
                kind: WidthKind::Dynamic,
                value: 0,
            }
        } else {
            Self {
                kind: if source.contains(['\u{2028}', '\u{2029}']) {
                    WidthKind::UnicodeWithLines
                } else {
                    WidthKind::Unicode
                },
                value: u16::try_from(measure_text_width(source)).unwrap_or(u16::MAX),
            }
        }
    }
}

impl<'a> Text<'a> {
    #[inline]
    pub(super) fn new(source: &'a str) -> Self {
        let width = if source.bytes().all(|byte| (0x20..=0x7f).contains(&byte)) {
            Width {
                kind: WidthKind::Ascii,
                value: 0,
            }
        } else {
            Width::measure_slow(source)
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
        matches!(self.width.kind, WidthKind::Ascii | WidthKind::Unicode)
    }

    /// Textual width has no column context and does not prove a flat render view.
    pub(super) fn clean_width(self) -> Option<usize> {
        Some(match self.width.kind {
            WidthKind::Ascii => self.source().len(),
            WidthKind::Unicode | WidthKind::UnicodeWithLines => usize::from(self.width.value),
            WidthKind::Dynamic => {
                if self.source().contains(['\r', '\n']) {
                    return None;
                }
                measure_text_width(self.source())
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
            WidthKind::Unicode | WidthKind::UnicodeWithLines => {
                (column + usize::from(self.width.value), false)
            }
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

fn advance(source: &str, column: usize) -> (usize, bool) {
    if let Some(index) = source.find(['\r', '\n']) {
        (column + measure_text_width(&source[..index]), true)
    } else {
        (column + measure_text_width(source), false)
    }
}
