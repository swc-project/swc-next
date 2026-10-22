//! Flat instruction streams with borrowed text and explicit layout scopes.
//!
//! Construction owns a growable instruction buffer in the formatting arena.
//! Finishing seals an immutable slice; source text remains borrowed from its owner.
//! Line suffixes retain complete Docs and their insertion mode and indentation.
//! Syntax-specific whitespace and break requests belong to the caller's Doc.

mod printer;
#[cfg(test)]
mod tests;
mod text;
mod writer;

pub(crate) use writer::{FillEntry, Writer};

use std::num::NonZeroUsize;

use crate::FormatOptions;
use text::Text;

#[derive(Debug, Clone, Copy)]
pub(crate) struct GroupId(NonZeroUsize);

impl GroupId {
    fn index(self) -> usize {
        self.0.get() - 1
    }
}

/// Conditions read either the active execution mode or an explicitly named group.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ConditionTarget {
    Current,
    Group(GroupId),
}

/// Explicit layout requests use the same meaning during measurement and printing.
#[derive(Debug, Clone, Copy)]
pub(crate) enum GroupRequest {
    Auto,
    Expanded,
}

impl From<bool> for GroupRequest {
    fn from(expand: bool) -> Self {
        if expand { Self::Expanded } else { Self::Auto }
    }
}

/// Required expansion joins within a group and stops at conditional groups.
#[derive(Debug, Clone, Copy, Default)]
struct BreakEffect(u8);

impl BreakEffect {
    const REQUIRED: Self = Self(1);

    fn from_request(request: GroupRequest) -> Self {
        match request {
            GroupRequest::Auto => Self::default(),
            GroupRequest::Expanded => Self::REQUIRED,
        }
    }

    fn join(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    fn preferred_breaks(self) -> bool {
        self.0 != 0
    }
}

#[derive(Debug, Clone, Copy)]
enum FormatElement<'a> {
    Text(Text<'a>),
    SoftLine,
    SoftLineIfBreak(GroupId),
    Line,
    HardLine,
    BreakParent,
    LiteralLine,
    /// The standard composition `markAsRoot(literalline)`.
    LiteralLineWithIndent,
    IfBreak(Text<'a>),
    IfFlat(Text<'a>),
    Conditional(&'a Conditional<'a>),
    FlatConditional(&'a Conditional<'a>),
    /// A deferred Doc scope; its contents own their spacing and break requests.
    StartLineSuffix {
        end_offset: usize,
    },
    EndLineSuffix,
    /// Flushes pending line suffixes before continuing on a new line.
    LineSuffixBoundary,
    StartGroup {
        effect: BreakEffect,
        id: Option<GroupId>,
    },
    /// An anonymous Auto group with a lower bound for its own flat contents.
    StartGroupWithFlatWidth {
        width: usize,
    },
    EndGroup,
    StartIndent,
    StartAbsoluteIndent {
        width: usize,
    },
    StartIndentIfBreak {
        id: GroupId,
        end_offset: usize,
    },
    EndIndent,
    StartAlign {
        width: usize,
    },
    EndAlign {
        width: usize,
    },
    StartFill,
    EndFill,
    /// Distance from this opening instruction to its matching EndEntry.
    StartEntry {
        end_offset: usize,
    },
    EndEntry,
    Fragment(Fragment<'a>),
    FlatFragment(FlatFragment<'a>),
    Alternatives(&'a Alternatives<'a>),
}

/// A complete immutable scope, owned by the current formatting call.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Fragment<'a> {
    elements: &'a [FormatElement<'a>],
    effect: BreakEffect,
    capabilities: FragmentCapabilities,
    conditional_lines: bool,
    intrinsic_lines: bool,
}

/// Both views retain the same complete scope; only the anonymous outer group differs.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RecordedGroup<'a> {
    grouped: Fragment<'a>,
}

impl<'a> RecordedGroup<'a> {
    pub(crate) fn grouped(self) -> Fragment<'a> {
        self.grouped
    }

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
struct CleanTextWidth {
    width: usize,
    ascii_width: Option<usize>,
    unicode: Option<CleanUnicodeText>,
}

#[derive(Clone, Copy)]
enum CleanUnicodeText {
    Single(usize),
    Joined(usize),
}

fn clean_text_width(elements: &[FormatElement<'_>]) -> Option<usize> {
    let mut text = CleanTextWidth {
        ascii_width: Some(0),
        ..CleanTextWidth::default()
    };
    add_clean_text_width(elements, &mut text)?;
    if let Some(width) = text.ascii_width {
        // This shortcut applies to the complete joined text, including DEL.
        return Some(width);
    }
    let Some(CleanUnicodeText::Joined(bytes)) = text.unicode else {
        return Some(text.width);
    };
    // Clean, non-ASCII text can compose emoji across instruction boundaries.
    // The synthesized spelling belongs to this measurement, not to the Doc.
    let mut joined = String::with_capacity(bytes);
    append_clean_text(elements, &mut joined)?;
    Some(crate::text_width::width(&joined))
}

fn add_clean_text_width(elements: &[FormatElement<'_>], text: &mut CleanTextWidth) -> Option<()> {
    for element in elements {
        match *element {
            FormatElement::Text(part) => {
                let part_width = part.clean_width()?;
                let ascii_width = part.clean_ascii_width();
                if let Some(unicode) = text.unicode {
                    let bytes = match unicode {
                        CleanUnicodeText::Single(bytes) | CleanUnicodeText::Joined(bytes) => bytes,
                    };
                    text.unicode = Some(CleanUnicodeText::Joined(
                        bytes.saturating_add(part.source.len()),
                    ));
                } else if ascii_width.is_none() {
                    // Until this first Unicode part, width also equals byte length.
                    let bytes = text.width.saturating_add(part.source.len());
                    text.unicode = Some(if text.width == 0 {
                        CleanUnicodeText::Single(bytes)
                    } else {
                        CleanUnicodeText::Joined(bytes)
                    });
                }
                text.width = text.width.saturating_add(part_width);
                text.ascii_width = match (text.ascii_width, ascii_width) {
                    (Some(width), Some(part_width)) => Some(width.saturating_add(part_width)),
                    _ => None,
                };
            }
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

fn append_clean_text(elements: &[FormatElement<'_>], output: &mut String) -> Option<()> {
    for element in elements {
        match *element {
            FormatElement::Text(text) => output.push_str(text.source),
            FormatElement::Fragment(fragment)
            | FormatElement::FlatFragment(FlatFragment(fragment)) => {
                append_clean_text(fragment.elements, output)?;
            }
            _ => return None,
        }
    }
    Some(())
}

/// Measures only direct instructions whose flat mode cannot change before the
/// scope ends. As in a fit probe, line spaces count only before later text.
fn flat_width(elements: &[FormatElement<'_>]) -> Option<usize> {
    let mut width = 0usize;
    let mut pending_space = false;
    for element in elements {
        match element {
            FormatElement::Text(text) | FormatElement::IfFlat(text) => {
                let text_width = text.clean_ascii_width()?;
                width = width
                    .saturating_add(usize::from(pending_space))
                    .saturating_add(text_width);
                pending_space = false;
            }
            FormatElement::Line => {
                pending_space = true;
            }
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
                    | FormatElement::LiteralLine
                    | FormatElement::LiteralLineWithIndent => return true,
                    FormatElement::Text(text)
                    | FormatElement::IfBreak(text)
                    | FormatElement::IfFlat(text) => {
                        if !text.is_single_line() {
                            return true;
                        }
                    }
                    FormatElement::Conditional(conditional)
                    | FormatElement::FlatConditional(conditional) => {
                        let flat = matches!(element, FormatElement::FlatConditional(_));
                        pending.push((conditional.broken.elements, flat));
                        pending.push((conditional.flat.elements, flat));
                    }
                    FormatElement::Fragment(fragment) => pending.push((fragment.elements, false)),
                    FormatElement::FlatFragment(FlatFragment(fragment)) => {
                        pending.push((fragment.elements, true));
                    }
                    FormatElement::Alternatives(alternatives) => {
                        pending.push((alternatives.first().elements, false));
                    }
                    _ => {}
                }
            }
        }
        false
    }

    pub(crate) fn preferred_breaks(self) -> bool {
        self.effect.preferred_breaks()
    }

    /// Construction summary for language layout policy, independent of flat
    /// projection eligibility. Group decisions and breakParent have no intrinsic
    /// newline; literal lines, multiline text, and suffix scopes do.
    pub(crate) fn has_intrinsic_lines(self) -> bool {
        self.intrinsic_lines
    }

    pub(crate) fn try_flat(self) -> Option<FlatFragment<'a>> {
        self.capabilities
            .permits_flat()
            .then_some(FlatFragment(self))
    }

    pub(crate) fn try_neutral(self) -> Option<NeutralFragment<'a>> {
        matches!(self.capabilities, FragmentCapabilities::Neutral).then_some(NeutralFragment(self))
    }
}

/// A flat view whose complete scope cannot observe layout mode.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NeutralFragment<'a>(Fragment<'a>);

#[derive(Debug, Clone, Copy)]
enum FragmentCapabilities {
    General,
    FlatEligible,
    Neutral,
}

impl FragmentCapabilities {
    fn from_flat(flat: bool) -> Self {
        if flat {
            Self::FlatEligible
        } else {
            Self::General
        }
    }

    fn permits_flat(self) -> bool {
        !matches!(self, Self::General)
    }

    /// Fresh Writer records contain complete, balanced scopes. Once neutrality
    /// is ruled out, the same iterator resumes the existing flat-only scan.
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
                FormatElement::StartGroup {
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

#[derive(Debug)]
struct Conditional<'a> {
    target: ConditionTarget,
    broken: Fragment<'a>,
    flat: Fragment<'a>,
}

impl<'a> Alternatives<'a> {
    /// Flat views and their eligibility always follow this exact path.
    fn first(&self) -> Fragment<'a> {
        self.preferred.first().copied().unwrap_or(self.fallback)
    }
}

impl FormatElement<'_> {
    fn permits_flat(self) -> bool {
        match self {
            Self::HardLine
            | Self::LiteralLine
            | Self::LiteralLineWithIndent
            | Self::StartLineSuffix { .. }
            | Self::EndLineSuffix
            | Self::BreakParent
            // Named groups commit an observable layout decision. Removing that
            // decision cannot be expressed by a flat view of standard Doc.
            | Self::StartGroup { id: Some(_), .. } => false,
            Self::Conditional(conditional) | Self::FlatConditional(conditional) => {
                conditional.broken.capabilities.permits_flat()
                    && conditional.flat.capabilities.permits_flat()
            }
            Self::Text(text) | Self::IfFlat(text) => text.is_single_line(),
            Self::Fragment(fragment) => fragment.capabilities.permits_flat(),
            Self::Alternatives(alternatives) => alternatives.first().capabilities.permits_flat(),
            _ => true,
        }
    }

    /// Applies a flat view without changing any retained instructions.
    fn flattened(self) -> Option<Self> {
        match self {
            Self::Line => Some(Self::Text(Text::new(" "))),
            Self::IfFlat(text) => Some(Self::Text(text)),
            Self::Conditional(conditional) | Self::FlatConditional(conditional) => {
                Some(match conditional.target {
                    ConditionTarget::Current => Self::FlatFragment(FlatFragment(conditional.flat)),
                    ConditionTarget::Group(_) => Self::FlatConditional(conditional),
                })
            }
            Self::Fragment(fragment) => Some(Self::FlatFragment(FlatFragment(fragment))),
            Self::Alternatives(alternatives) => {
                Some(Self::FlatFragment(FlatFragment(alternatives.first())))
            }
            element @ (Self::Text(_)
            | Self::LineSuffixBoundary
            | Self::StartIndentIfBreak { .. }
            | Self::FlatFragment(_)) => Some(element),
            _ => None,
        }
    }
}

pub(crate) struct Doc<'a> {
    allocator: &'a swc_next_allocator::Allocator,
    elements: &'a [FormatElement<'a>],
    group_count: usize,
}

impl Doc<'_> {
    pub(crate) fn print(self, options: FormatOptions) -> String {
        printer::print(self.elements, self.allocator, self.group_count, options)
    }
}
