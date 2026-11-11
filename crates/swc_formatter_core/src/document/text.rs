//! Cached display widths and source-line classification for borrowed text.

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
    Multiline,
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
        let kind = if memchr::memchr2(b'\r', b'\n', source.as_bytes()).is_some() {
            WidthKind::Multiline
        } else if source.contains(['\u{2028}', '\u{2029}']) {
            WidthKind::UnicodeWithLines
        } else {
            WidthKind::Unicode
        };
        Self {
            kind,
            value: u16::try_from(measure_text_width(source)).unwrap_or(u16::MAX),
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
            WidthKind::Multiline => return None,
        })
    }

    pub(super) fn clean_ascii_width(self) -> Option<usize> {
        match self.width.kind {
            WidthKind::Ascii => Some(self.source().len()),
            _ => None,
        }
    }

    /// A Text is one string for layout, even when its source contains newlines.
    pub(super) fn advance_column(self, column: usize) -> usize {
        column
            + match self.width.kind {
                WidthKind::Ascii => self.source().len(),
                _ => usize::from(self.width.value),
            }
    }
}
