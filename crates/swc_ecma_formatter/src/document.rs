//! Flat documents containing borrowed text and explicit line breaks.
//!
//! Construction owns a growable instruction buffer in the formatting arena.
//! Finishing seals an immutable slice; source text remains borrowed from its owner.

mod printer;
#[cfg(test)]
mod tests;
mod text;
mod writer;

pub(crate) use writer::Writer;

use crate::FormatOptions;
use text::Text;

#[derive(Debug, Clone, Copy)]
pub(crate) struct GroupId(usize);

/// Explicit layout requests stay distinct from preserved source layout.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
pub(crate) enum GroupRequest {
    Auto,
    Expanded,
    PreserveSource,
}

impl From<bool> for GroupRequest {
    fn from(expand: bool) -> Self {
        if expand { Self::Expanded } else { Self::Auto }
    }
}

/// Effects join within each complete scope. Source preferences remain effective
/// when printing, while required expansion also constrains canonical measurement.
#[derive(Debug, Clone, Copy, Default)]
struct BreakEffect(u8);

impl BreakEffect {
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    const SOURCE: Self = Self(1);
    const REQUIRED: Self = Self(2);

    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    fn from_request(request: GroupRequest) -> Self {
        match request {
            GroupRequest::Auto => Self::default(),
            GroupRequest::Expanded => Self::REQUIRED,
            GroupRequest::PreserveSource => Self::SOURCE,
        }
    }

    fn join(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    fn preferred_breaks(self) -> bool {
        self.0 != 0
    }

    fn required(self) -> bool {
        self.0 & Self::REQUIRED.0 != 0
    }
}

#[derive(Debug, Clone, Copy)]
enum Measurement {
    Display,
    Canonical,
}

#[derive(Debug, Clone, Copy)]
enum SpaceKind {
    Deferred,
    /// One canonical gap reserves fit width before the next text commits it.
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    Reserved,
}

#[derive(Debug, Clone, Copy)]
enum FormatElement<'a> {
    Text(Text<'a>),
    Space(SpaceKind),
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    SoftLine,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    SoftLineIfBreak(GroupId),
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    Line,
    HardLine,
    BreakParent,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    LiteralLine,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    IfBreak(Text<'a>),
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    IfFlat(Text<'a>),
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    IfCurrentBreak(Text<'a>),
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    IfCurrentFlat(Text<'a>),
    LineSuffix(Text<'a>),
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    StartGroup {
        effect: BreakEffect,
        id: Option<GroupId>,
    },
    /// An anonymous Auto group with a lower bound for its own flat contents.
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    StartGroupWithFlatWidth {
        width: usize,
    },
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    EndGroup,
    FlatGroup(GroupId),
    StartIndent,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    StartAbsoluteIndent {
        width: usize,
    },
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    StartIndentIfBreak(GroupId),
    EndIndent,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    StartAlign {
        width: usize,
    },
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    EndAlign {
        width: usize,
    },
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    StartFill,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    EndFill,
    /// Distance from this opening instruction to its matching EndEntry.
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    StartEntry {
        end_offset: usize,
    },
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    EndEntry,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    Fragment(Fragment<'a>),
    FlatFragment(FlatFragment<'a>),
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    Alternatives(&'a Alternatives<'a>),
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    CanonicalAlternatives(&'a Alternatives<'a>),
}

/// A complete immutable scope, owned by the current formatting call.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Fragment<'a> {
    elements: &'a [FormatElement<'a>],
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    effect: BreakEffect,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    capabilities: FragmentCapabilities,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    conditional_lines: bool,
}

/// Both views retain the same complete scope; only the anonymous outer group differs.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
pub(crate) struct RecordedGroup<'a> {
    grouped: Fragment<'a>,
}

impl<'a> RecordedGroup<'a> {
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(crate) fn grouped(self) -> Fragment<'a> {
        self.grouped
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(crate) fn contents(self) -> Fragment<'a> {
        // Construction retains one complete anonymous Auto group. Its wrapper
        // inherits the contents' effects and capabilities without adding any.
        let elements = self.grouped.elements;
        let elements = &elements[1..elements.len() - 1];
        Fragment {
            elements,
            ..self.grouped
        }
    }
}

#[derive(Default)]
#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
struct CleanTextWidth {
    width: usize,
    ascii_width: Option<usize>,
    unicode: Option<CleanUnicodeText>,
    pending_space: bool,
}

#[derive(Clone, Copy)]
#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
enum CleanUnicodeText {
    Single(usize),
    Joined(usize),
}

#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
fn clean_text_width(elements: &[FormatElement<'_>]) -> Option<usize> {
    let mut text = CleanTextWidth {
        ascii_width: Some(0),
        ..CleanTextWidth::default()
    };
    add_clean_text_width(elements, &mut text)?;
    let trailing_space = usize::from(text.pending_space);
    if let Some(width) = text.ascii_width {
        // This shortcut applies to the complete joined text, including DEL.
        return Some(width.saturating_add(trailing_space));
    }
    let Some(CleanUnicodeText::Joined(bytes)) = text.unicode else {
        return Some(text.width.saturating_add(trailing_space));
    };
    // Clean, non-ASCII text can compose emoji across instruction boundaries.
    // The synthesized spelling belongs to this measurement, not to the Doc.
    let mut joined = String::with_capacity(bytes.saturating_add(trailing_space));
    let mut pending_space = false;
    append_clean_text(elements, &mut joined, &mut pending_space)?;
    if pending_space {
        joined.push(' ');
    }
    Some(crate::text_width::width(&joined))
}

#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
fn add_clean_text_width(elements: &[FormatElement<'_>], text: &mut CleanTextWidth) -> Option<()> {
    for element in elements {
        match *element {
            FormatElement::Text(part) => {
                let part_width = part.clean_width()?;
                let ascii_width = part.clean_ascii_width();
                let gap = usize::from(text.pending_space);
                if let Some(unicode) = text.unicode {
                    let bytes = match unicode {
                        CleanUnicodeText::Single(bytes) | CleanUnicodeText::Joined(bytes) => bytes,
                    };
                    text.unicode = Some(CleanUnicodeText::Joined(
                        bytes.saturating_add(gap).saturating_add(part.source.len()),
                    ));
                } else if ascii_width.is_none() {
                    // Until this first Unicode part, width also equals byte length.
                    let bytes = text
                        .width
                        .saturating_add(gap)
                        .saturating_add(part.source.len());
                    text.unicode = Some(if text.width == 0 {
                        CleanUnicodeText::Single(bytes)
                    } else {
                        CleanUnicodeText::Joined(bytes)
                    });
                }
                text.width = text.width.saturating_add(gap).saturating_add(part_width);
                text.ascii_width = match (text.ascii_width, ascii_width) {
                    (Some(width), Some(part_width)) => {
                        Some(width.saturating_add(gap).saturating_add(part_width))
                    }
                    _ => None,
                };
                text.pending_space = false;
            }
            FormatElement::Space(_) => text.pending_space = true,
            FormatElement::Fragment(fragment)
            | FormatElement::FlatFragment(FlatFragment(fragment)) => {
                if fragment.effect.preferred_breaks() || fragment.conditional_lines {
                    return None;
                }
                add_clean_text_width(fragment.elements, text)?;
            }
            _ => return None,
        }
    }
    Some(())
}

#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
fn append_clean_text(
    elements: &[FormatElement<'_>],
    output: &mut String,
    pending_space: &mut bool,
) -> Option<()> {
    for element in elements {
        match *element {
            FormatElement::Text(text) => {
                if *pending_space {
                    output.push(' ');
                    *pending_space = false;
                }
                output.push_str(text.source);
            }
            FormatElement::Space(_) => *pending_space = true,
            FormatElement::Fragment(fragment)
            | FormatElement::FlatFragment(FlatFragment(fragment)) => {
                append_clean_text(fragment.elements, output, pending_space)?;
            }
            _ => return None,
        }
    }
    Some(())
}

/// Measures only direct instructions whose flat mode cannot change before the
/// scope ends. Leading and trailing gaps are excluded so the result remains a
/// lower bound regardless of the caller's pending gap.
#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
fn flat_width(elements: &[FormatElement<'_>]) -> Option<usize> {
    let mut width = 0usize;
    let mut has_text = false;
    let mut pending_space = false;
    for element in elements {
        match element {
            FormatElement::Text(text) | FormatElement::IfFlat(text) => {
                let text_width = text.clean_ascii_width()?;
                width = width
                    .saturating_add(usize::from(has_text && pending_space))
                    .saturating_add(text_width);
                has_text = true;
                pending_space = false;
            }
            FormatElement::Space(_) | FormatElement::Line => pending_space = true,
            FormatElement::SoftLine
            | FormatElement::IfBreak(_)
            | FormatElement::StartGroup {
                effect: BreakEffect(0),
                id: None,
            }
            | FormatElement::EndGroup
            | FormatElement::StartIndent
            | FormatElement::StartAbsoluteIndent { .. }
            | FormatElement::EndIndent
            | FormatElement::StartAlign { .. }
            | FormatElement::EndAlign { .. } => {}
            _ => return None,
        }
    }
    Some(width)
}

impl<'a> Fragment<'a> {
    /// Whether the normal render path contains a line-breaking opportunity.
    /// Alternatives follow their first candidate, while flat views inspect
    /// only the instructions that survive their existing projection.
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(crate) fn can_break(self) -> bool {
        if matches!(self.capabilities, FragmentCapabilities::Neutral) {
            return false;
        }
        let mut pending = vec![(self.elements, false)];
        while let Some((elements, flat)) = pending.pop() {
            for &element in elements {
                let element = if flat {
                    let Some(element) = element.flattened() else {
                        continue;
                    };
                    element
                } else {
                    element
                };
                match element {
                    FormatElement::SoftLine
                    | FormatElement::SoftLineIfBreak(_)
                    | FormatElement::Line
                    | FormatElement::HardLine
                    | FormatElement::LiteralLine => return true,
                    FormatElement::Text(text)
                    | FormatElement::IfBreak(text)
                    | FormatElement::IfFlat(text)
                    | FormatElement::IfCurrentBreak(text)
                    | FormatElement::IfCurrentFlat(text)
                    | FormatElement::LineSuffix(text) => {
                        if !text.is_single_line() {
                            return true;
                        }
                    }
                    FormatElement::Fragment(fragment) => pending.push((fragment.elements, false)),
                    FormatElement::FlatFragment(FlatFragment(fragment)) => {
                        pending.push((fragment.elements, true));
                    }
                    FormatElement::Alternatives(alternatives)
                    | FormatElement::CanonicalAlternatives(alternatives) => {
                        pending.push((alternatives.first().elements, false));
                    }
                    _ => {}
                }
            }
        }
        false
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(crate) fn preferred_breaks(self) -> bool {
        self.effect.preferred_breaks()
    }

    /// Width of a document that reduces to text without observing layout scopes.
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(crate) fn clean_text_width(self) -> Option<usize> {
        if self.effect.preferred_breaks() || self.conditional_lines {
            return None;
        }
        clean_text_width(self.elements)
    }

    /// Measures this complete scope without changing its retained render view.
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(crate) fn canonical_candidate(self) -> CanonicalCandidate<'a> {
        CanonicalCandidate(self)
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(crate) fn try_flat(self) -> Option<FlatFragment<'a>> {
        self.capabilities
            .permits_flat()
            .then_some(FlatFragment(self))
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    pub(crate) fn try_neutral(self) -> Option<NeutralFragment<'a>> {
        matches!(self.capabilities, FragmentCapabilities::Neutral).then_some(NeutralFragment(self))
    }
}

/// A flat view whose complete scope cannot observe layout mode.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
pub(crate) struct NeutralFragment<'a>(Fragment<'a>);

/// A zero-copy measurement request, distinct from a printed Flat projection.
#[derive(Clone, Copy)]
#[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
pub(crate) struct CanonicalCandidate<'a>(Fragment<'a>);

#[derive(Debug, Clone, Copy)]
enum FragmentCapabilities {
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    General,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    FlatEligible,
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    Neutral,
}

impl FragmentCapabilities {
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    fn from_flat(flat: bool) -> Self {
        if flat {
            Self::FlatEligible
        } else {
            Self::General
        }
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    fn permits_flat(self) -> bool {
        !matches!(self, Self::General)
    }

    /// Fresh Writer records contain complete, balanced scopes. Once neutrality
    /// is ruled out, the same iterator resumes the existing flat-only scan.
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    fn classify(elements: &[FormatElement<'_>]) -> Self {
        let mut elements = elements.iter().copied();
        while let Some(element) = elements.next() {
            let flat = match element {
                FormatElement::Text(text) => {
                    if !text.is_single_line() {
                        return Self::General;
                    }
                    continue;
                }
                FormatElement::Space(_)
                | FormatElement::StartGroup {
                    effect: BreakEffect(0),
                    id: None,
                }
                | FormatElement::StartGroupWithFlatWidth { .. }
                | FormatElement::EndGroup => continue,
                FormatElement::Fragment(fragment) => match fragment.capabilities {
                    Self::Neutral => continue,
                    Self::General => return Self::General,
                    Self::FlatEligible => true,
                },
                FormatElement::FlatFragment(fragment)
                    if matches!(fragment.0.capabilities, Self::Neutral) =>
                {
                    continue;
                }
                FormatElement::FlatFragment(_) => true,
                element => element.permits_flat(),
            };
            return Self::from_flat(flat && elements.all(FormatElement::permits_flat));
        }
        Self::Neutral
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FlatFragment<'a>(Fragment<'a>);

#[derive(Debug)]
struct Alternatives<'a> {
    preferred: &'a [Fragment<'a>],
    fallback: Fragment<'a>,
}

impl<'a> Alternatives<'a> {
    /// Flat views and their eligibility always follow this exact path.
    fn first(&self) -> Fragment<'a> {
        self.preferred.first().copied().unwrap_or(self.fallback)
    }
}

impl FormatElement<'_> {
    #[cfg_attr(not(test), expect(dead_code, reason = "Pending syntax consumers."))]
    fn permits_flat(self) -> bool {
        match self {
            Self::HardLine | Self::LiteralLine | Self::LineSuffix(_) | Self::BreakParent => false,
            Self::Text(text) | Self::IfCurrentFlat(text) => text.is_single_line(),
            Self::Fragment(fragment) => fragment.capabilities.permits_flat(),
            Self::Alternatives(alternatives) | Self::CanonicalAlternatives(alternatives) => {
                alternatives.first().capabilities.permits_flat()
            }
            _ => true,
        }
    }

    /// Applies a flat view without changing any retained instructions.
    fn flattened(self) -> Option<Self> {
        match self {
            Self::Line => Some(Self::Space(SpaceKind::Deferred)),
            Self::IfFlat(text) | Self::IfCurrentFlat(text) => Some(Self::Text(text)),
            Self::StartGroup { id: Some(id), .. } => Some(Self::FlatGroup(id)),
            Self::Fragment(fragment) => Some(Self::FlatFragment(FlatFragment(fragment))),
            Self::Alternatives(alternatives) | Self::CanonicalAlternatives(alternatives) => {
                Some(Self::FlatFragment(FlatFragment(alternatives.first())))
            }
            element @ (Self::Text(_)
            | Self::Space(_)
            | Self::FlatGroup(_)
            | Self::FlatFragment(_)) => Some(element),
            _ => None,
        }
    }
}

pub(crate) struct Doc<'a> {
    elements: &'a [FormatElement<'a>],
    group_count: usize,
    conditional_lines: bool,
}

impl Doc<'_> {
    pub(crate) fn print(self, options: FormatOptions) -> String {
        printer::print(
            self.elements,
            self.group_count,
            self.conditional_lines,
            options,
        )
    }
}
