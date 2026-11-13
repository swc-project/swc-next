//! Cached display widths and source-line classification for borrowed text.

use crate::measure_text_width;

use rustc_hash::FxHashMap;

use super::{Instruction, thin::ThinStr, transform::key};

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

#[derive(Clone, Copy)]
struct CleanTextWidth {
    width: usize,
    ascii_width: Option<usize>,
    unicode: Option<CleanUnicodeText>,
    has_text: bool,
}

impl Default for CleanTextWidth {
    fn default() -> Self {
        Self {
            width: 0,
            ascii_width: Some(0),
            unicode: None,
            has_text: false,
        }
    }
}

#[derive(Clone, Copy)]
enum CleanUnicodeText {
    Single(usize),
    Joined(usize),
}

impl CleanUnicodeText {
    fn bytes(self) -> usize {
        match self {
            Self::Single(bytes) | Self::Joined(bytes) => bytes,
        }
    }
}

impl CleanTextWidth {
    #[inline]
    fn text(&mut self, part: Text<'_>) -> Option<()> {
        let part_width = part.clean_width()?;
        let ascii_width = part.clean_ascii_width();
        if let Some(unicode) = self.unicode {
            self.unicode = Some(CleanUnicodeText::Joined(
                unicode.bytes().saturating_add(part.source().len()),
            ));
        } else if ascii_width.is_none() {
            // Until the first Unicode part, width also equals byte length.
            let bytes = self.width.saturating_add(part.source().len());
            self.unicode = Some(if self.width == 0 {
                CleanUnicodeText::Single(bytes)
            } else {
                CleanUnicodeText::Joined(bytes)
            });
        }
        self.width = self.width.saturating_add(part_width);
        self.ascii_width = match (self.ascii_width, ascii_width) {
            (Some(width), Some(part_width)) => Some(width.saturating_add(part_width)),
            _ => None,
        };
        self.has_text = true;
        Some(())
    }

    fn append(&mut self, part: &Self) {
        if !part.has_text {
            return;
        }
        if let Some(unicode) = self.unicode {
            let bytes = part.unicode.map_or(part.width, CleanUnicodeText::bytes);
            self.unicode = Some(CleanUnicodeText::Joined(
                unicode.bytes().saturating_add(bytes),
            ));
        } else if let Some(unicode) = part.unicode {
            self.unicode = Some(if self.width == 0 {
                unicode
            } else {
                CleanUnicodeText::Joined(self.width.saturating_add(unicode.bytes()))
            });
        }
        self.width = self.width.saturating_add(part.width);
        self.ascii_width = match (self.ascii_width, part.ascii_width) {
            (Some(width), Some(part_width)) => Some(width.saturating_add(part_width)),
            _ => None,
        };
        self.has_text = true;
    }
}

pub(super) fn clean_text_width(elements: &[Instruction<'_>]) -> Option<usize> {
    let mut text = CleanTextWidth::default();
    add_clean_text_width(elements, &mut text)?;
    if let Some(width) = text.ascii_width {
        // ASCII widths compose without expanding shared text.
        return Some(width);
    }
    let Some(CleanUnicodeText::Joined(bytes)) = text.unicode else {
        return Some(text.width);
    };
    // Unicode can compose across fragment boundaries. Materialize only for this
    // measurement, retaining the Doc's original instruction and sharing layout.
    let mut joined = String::with_capacity(bytes);
    append_clean_text(elements, &mut joined)?;
    Some(measure_text_width(&joined))
}

fn add_clean_text_width(mut elements: &[Instruction<'_>], text: &mut CleanTextWidth) -> Option<()> {
    while let Some((element, rest)) = elements.split_first() {
        match *element {
            Instruction::Text { source, width } => text.text(Text::from_parts(source, width))?,
            Instruction::Fragment {
                elements: child,
                summary,
            }
            | Instruction::FlatFragment {
                elements: child,
                summary,
            } => {
                if summary.effect().preferred_breaks() || summary.conditional_lines() {
                    return None;
                }
                if rest.is_empty() {
                    elements = child.get();
                    continue;
                }
                text.append(&graph_text_width(elements)?);
                return Some(());
            }
            _ => return None,
        }
        elements = rest;
    }
    Some(())
}

// Branching references use a postorder fold. Each immutable slice contributes
// its summary once; repeated references compose that summary in constant time.
fn graph_text_width(root: &[Instruction<'_>]) -> Option<CleanTextWidth> {
    let mut summaries = FxHashMap::default();
    let mut pending = vec![(root, false)];
    while let Some((elements, ready)) = pending.pop() {
        let id = key(elements);
        if summaries.contains_key(&id) {
            continue;
        }
        if !ready {
            pending.push((elements, true));
            for element in elements {
                match *element {
                    Instruction::Text { .. } => {}
                    Instruction::Fragment { elements, summary }
                    | Instruction::FlatFragment { elements, summary } => {
                        if summary.effect().preferred_breaks() || summary.conditional_lines() {
                            return None;
                        }
                        pending.push((elements.get(), false));
                    }
                    _ => return None,
                }
            }
            continue;
        }
        let mut text = CleanTextWidth::default();
        for element in elements {
            match *element {
                Instruction::Text { source, width } => {
                    text.text(Text::from_parts(source, width))?
                }
                Instruction::Fragment { elements, .. }
                | Instruction::FlatFragment { elements, .. } => {
                    text.append(&summaries[&key(elements.get())])
                }
                _ => unreachable!("text-only instructions were checked before visiting children"),
            }
        }
        summaries.insert(id, text);
    }
    summaries.remove(&key(root))
}

fn append_clean_text(mut elements: &[Instruction<'_>], output: &mut String) -> Option<()> {
    let mut pending = Vec::new();
    loop {
        while let Some((element, rest)) = elements.split_first() {
            match *element {
                Instruction::Text { source, .. } => output.push_str(source.as_str()),
                Instruction::Fragment {
                    elements: child, ..
                }
                | Instruction::FlatFragment {
                    elements: child, ..
                } => {
                    if !rest.is_empty() {
                        pending.push(rest);
                    }
                    elements = child.get();
                    continue;
                }
                _ => return None,
            }
            elements = rest;
        }
        let Some(next) = pending.pop() else {
            return Some(());
        };
        elements = next;
    }
}
