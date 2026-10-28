//! Flat instruction streams with borrowed text and explicit layout scopes.
//!
//! Construction owns a growable instruction buffer in the formatting arena.
//! Finishing seals an immutable slice; source text remains borrowed from its owner.
//! Compact instructions retain immutable borrows with checked lengths.
//! Line suffixes retain complete Docs and their insertion mode and indentation.
//! Syntax-specific whitespace and break requests belong to the caller's Doc.

mod encode;
mod printer;
#[cfg(test)]
mod tests;
mod text;
mod thin;
mod writer;

pub use writer::{FillEntry, FillSeparatorPolicy, Writer};

use std::num::NonZeroU32;

use crate::PrintOptions;
use text::{Text, Width};
use thin::{ThinSlice, ThinStr};

#[derive(Debug, Clone, Copy)]
pub struct GroupId(NonZeroU32);

impl GroupId {
    fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

/// Conditions read either the active execution mode or an explicitly named group.
#[derive(Debug, Clone, Copy)]
pub enum ConditionTarget {
    Current,
    Group(GroupId),
}

/// Explicit layout requests use the same meaning during measurement and printing.
#[derive(Debug, Clone, Copy)]
pub enum GroupRequest {
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
    Text {
        source: ThinStr<'a>,
        width: Width,
    },
    SoftLine,
    SoftLineIfBreak(GroupId),
    Line,
    HardLine,
    BreakParent,
    LiteralLine,
    /// The standard composition `markAsRoot(literalline)`.
    LiteralLineWithIndent,
    IfBreak {
        source: ThinStr<'a>,
        width: Width,
    },
    IfFlat {
        source: ThinStr<'a>,
        width: Width,
    },
    Conditional(&'a Conditional<'a>),
    FlatConditional(&'a Conditional<'a>),
    /// A deferred Doc scope; its contents own their spacing and break requests.
    StartLineSuffix {
        end_offset: u32,
    },
    EndLineSuffix,
    /// Flushes pending line suffixes before continuing on a new line.
    LineSuffixBoundary,
    StartGroup {
        effect: BreakEffect,
        id: Option<GroupId>,
    },
    /// An anonymous Auto group with exact context-free flat measurements and
    /// the distance from this opening instruction to its matching EndGroup.
    StartGroupWithFlatWidth {
        summary: FlatSummary,
        end_offset: u32,
    },
    EndGroup,
    StartIndent,
    StartAbsoluteIndent {
        width: u32,
    },
    StartIndentIfBreak {
        id: GroupId,
        end_offset: u32,
    },
    EndIndent,
    StartAlign {
        width: u32,
    },
    EndAlign {
        width: u32,
    },
    StartFill,
    EndFill,
    /// Distance from this opening instruction to its matching EndEntry.
    StartEntry {
        end_offset: u32,
    },
    EndEntry,
    Fragment {
        elements: ThinSlice<'a, FormatElement<'a>>,
        summary: FragmentSummary,
    },
    FlatFragment {
        elements: ThinSlice<'a, FormatElement<'a>>,
        summary: FragmentSummary,
    },
    Alternatives(&'a Alternatives<'a>),
}

const _: () = assert!(std::mem::size_of::<FormatElement<'_>>() <= 16);

/// A complete immutable scope, owned by the current formatting call.
#[derive(Debug, Clone, Copy)]
pub struct Fragment<'a> {
    elements: ThinSlice<'a, FormatElement<'a>>,
    summary: FragmentSummary,
}

/// Construction facts travel with every borrowed scope, including flat views.
/// They describe independent properties; projecting a scope does not erase them.
#[derive(Debug, Clone, Copy)]
struct FragmentSummary(u8);

impl FragmentSummary {
    const REQUIRED: u8 = 1;
    const CAPABILITIES: u8 = 0b110;
    const CONDITIONAL_LINES: u8 = 1 << 3;
    const INTRINSIC_LINES: u8 = 1 << 4;

    fn new(
        effect: BreakEffect,
        capabilities: FragmentCapabilities,
        conditional_lines: bool,
        intrinsic_lines: bool,
    ) -> Self {
        Self(
            u8::from(effect.preferred_breaks())
                | ((capabilities as u8) << 1)
                | (u8::from(conditional_lines) << 3)
                | (u8::from(intrinsic_lines) << 4),
        )
    }

    fn effect(self) -> BreakEffect {
        BreakEffect(self.0 & Self::REQUIRED)
    }

    fn capabilities(self) -> FragmentCapabilities {
        match (self.0 & Self::CAPABILITIES) >> 1 {
            0 => FragmentCapabilities::General,
            1 => FragmentCapabilities::FlatEligible,
            2 => FragmentCapabilities::Neutral,
            _ => unreachable!("fragment capabilities are set during construction"),
        }
    }

    fn conditional_lines(self) -> bool {
        self.0 & Self::CONDITIONAL_LINES != 0
    }

    fn intrinsic_lines(self) -> bool {
        self.0 & Self::INTRINSIC_LINES != 0
    }
}

/// Both views retain the same complete scope; only the anonymous outer group differs.
#[derive(Debug, Clone, Copy)]
pub struct RecordedGroup<'a> {
    grouped: Fragment<'a>,
}

impl<'a> RecordedGroup<'a> {
    pub fn grouped(self) -> Fragment<'a> {
        self.grouped
    }

    pub fn contents(self) -> Fragment<'a> {
        // Construction retains one complete anonymous Auto group. Its wrapper
        // inherits the contents' effects and capabilities without adding any.
        let elements = self.grouped.elements();
        let elements = &elements[1..elements.len() - 1];
        Fragment {
            elements: ThinSlice::new(elements),
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
            FormatElement::Text { source, width } => {
                let part = Text::from_parts(source, width);
                let part_width = part.clean_width()?;
                let ascii_width = part.clean_ascii_width();
                if let Some(unicode) = text.unicode {
                    let bytes = match unicode {
                        CleanUnicodeText::Single(bytes) | CleanUnicodeText::Joined(bytes) => bytes,
                    };
                    text.unicode = Some(CleanUnicodeText::Joined(
                        bytes.saturating_add(part.source().len()),
                    ));
                } else if ascii_width.is_none() {
                    // Until this first Unicode part, width also equals byte length.
                    let bytes = text.width.saturating_add(part.source().len());
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
            FormatElement::Fragment { elements, summary }
            | FormatElement::FlatFragment { elements, summary } => {
                if summary.effect().preferred_breaks() || summary.conditional_lines() {
                    return None;
                }
                add_clean_text_width(elements.get(), text)?;
            }
            _ => return None,
        }
    }
    Some(())
}

fn append_clean_text(elements: &[FormatElement<'_>], output: &mut String) -> Option<()> {
    for element in elements {
        match *element {
            FormatElement::Text { source, .. } => output.push_str(source.as_str()),
            FormatElement::Fragment { elements, .. }
            | FormatElement::FlatFragment { elements, .. } => {
                append_clean_text(elements.get(), output)?;
            }
            _ => return None,
        }
    }
    Some(())
}

/// Context-free ASCII measurements, saturated at the supported width limit.
/// Literal text spaces remain part of `width`;
/// only spaces contributed by `line` remain pending at the scope boundaries.
#[derive(Debug, Clone, Copy, Default)]
struct FlatSummary {
    width: u16,
    has_text: bool,
    starts_with_line: bool,
    ends_with_line: bool,
}

impl FlatSummary {
    /// Advances a flat fit probe without consuming pending line spaces twice.
    fn advance(self, column: usize, pending_line: &mut bool) -> usize {
        if self.has_text {
            let column = column
                .saturating_add(usize::from(self.width))
                .saturating_add(usize::from(*pending_line || self.starts_with_line));
            *pending_line = self.ends_with_line;
            column
        } else {
            *pending_line |= self.ends_with_line;
            column
        }
    }
}

/// Measures a completed group once, retaining only context-free ASCII results.
/// Literal text spaces count directly; flat line spaces stay pending until text.
fn flat_summary(elements: &[FormatElement<'_>]) -> Option<FlatSummary> {
    let mut summary = FlatSummary::default();
    for element in elements {
        match element {
            FormatElement::Text { source, width } | FormatElement::IfFlat { source, width } => {
                let width = Text::from_parts(*source, *width).clean_ascii_width()?;
                let width = u16::try_from(width).unwrap_or(u16::MAX);
                if summary.has_text {
                    summary.width = summary
                        .width
                        .saturating_add(u16::from(summary.ends_with_line))
                        .saturating_add(width);
                } else {
                    summary.starts_with_line = summary.ends_with_line;
                    summary.width = width;
                    summary.has_text = true;
                }
                summary.ends_with_line = false;
            }
            FormatElement::Line => summary.ends_with_line = true,
            FormatElement::SoftLine
            | FormatElement::IfBreak { .. }
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
    Some(summary)
}

impl<'a> Fragment<'a> {
    fn new(
        elements: &'a [FormatElement<'a>],
        effect: BreakEffect,
        capabilities: FragmentCapabilities,
        conditional_lines: bool,
        intrinsic_lines: bool,
    ) -> Self {
        Self {
            elements: ThinSlice::new(elements),
            summary: FragmentSummary::new(effect, capabilities, conditional_lines, intrinsic_lines),
        }
    }

    fn elements(self) -> &'a [FormatElement<'a>] {
        self.elements.get()
    }

    fn effect(self) -> BreakEffect {
        self.summary.effect()
    }

    fn capabilities(self) -> FragmentCapabilities {
        self.summary.capabilities()
    }

    fn conditional_lines(self) -> bool {
        self.summary.conditional_lines()
    }

    /// Whether the normal render path contains a line-breaking opportunity.
    /// Alternatives follow their first candidate, while flat views inspect
    /// only the instructions that survive their existing projection.
    pub fn can_break(self) -> bool {
        if matches!(self.capabilities(), FragmentCapabilities::Neutral) {
            return false;
        }
        let mut pending = vec![(self.elements(), false)];
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
                    FormatElement::Text { source, width }
                    | FormatElement::IfBreak { source, width }
                    | FormatElement::IfFlat { source, width } => {
                        if !Text::from_parts(source, width).is_single_line() {
                            return true;
                        }
                    }
                    FormatElement::Conditional(conditional)
                    | FormatElement::FlatConditional(conditional) => {
                        let flat = matches!(element, FormatElement::FlatConditional(_));
                        pending.push((conditional.broken.elements(), flat));
                        pending.push((conditional.flat.elements(), flat));
                    }
                    FormatElement::Fragment { elements, .. } => {
                        pending.push((elements.get(), false))
                    }
                    FormatElement::FlatFragment { elements, .. } => {
                        pending.push((elements.get(), true));
                    }
                    FormatElement::Alternatives(alternatives) => {
                        pending.push((alternatives.first().elements(), false));
                    }
                    _ => {}
                }
            }
        }
        false
    }

    pub fn preferred_breaks(self) -> bool {
        self.effect().preferred_breaks()
    }

    /// Construction summary for language layout policy, independent of flat
    /// projection eligibility. Group decisions and breakParent have no intrinsic
    /// newline; literal lines, multiline text, and suffix scopes do.
    pub fn has_intrinsic_lines(self) -> bool {
        self.summary.intrinsic_lines()
    }

    pub fn try_flat(self) -> Option<FlatFragment<'a>> {
        self.capabilities()
            .permits_flat()
            .then_some(FlatFragment(self))
    }

    pub fn try_neutral(self) -> Option<NeutralFragment<'a>> {
        matches!(self.capabilities(), FragmentCapabilities::Neutral)
            .then_some(NeutralFragment(self))
    }
}

/// A flat view whose complete scope cannot observe layout mode.
#[derive(Debug, Clone, Copy)]
pub struct NeutralFragment<'a>(Fragment<'a>);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
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
                FormatElement::Text { source, width } => {
                    if !Text::from_parts(source, width).is_single_line() {
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
                FormatElement::Fragment { summary, .. } => match summary.capabilities() {
                    Self::Neutral => continue,
                    Self::General => return Self::General,
                    Self::FlatEligible => true,
                },
                FormatElement::FlatFragment { summary, .. }
                    if matches!(summary.capabilities(), Self::Neutral) =>
                {
                    continue;
                }
                FormatElement::FlatFragment { .. } => true,
                element => element.permits_flat(),
            };
            return Self::from_flat(flat && elements.all(FormatElement::permits_flat));
        }
        Self::Neutral
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlatFragment<'a>(Fragment<'a>);

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

impl<'a> FormatElement<'a> {
    fn text(text: Text<'a>) -> Self {
        let (source, width) = text.into_parts();
        Self::Text { source, width }
    }

    fn if_break(text: Text<'a>) -> Self {
        let (source, width) = text.into_parts();
        Self::IfBreak { source, width }
    }

    fn if_flat(text: Text<'a>) -> Self {
        let (source, width) = text.into_parts();
        Self::IfFlat { source, width }
    }

    fn fragment(fragment: Fragment<'a>) -> Self {
        Self::Fragment {
            elements: fragment.elements,
            summary: fragment.summary,
        }
    }

    fn flat_fragment(FlatFragment(fragment): FlatFragment<'a>) -> Self {
        Self::FlatFragment {
            elements: fragment.elements,
            summary: fragment.summary,
        }
    }

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
                conditional.broken.capabilities().permits_flat()
                    && conditional.flat.capabilities().permits_flat()
            }
            Self::Text { source, width } | Self::IfFlat { source, width } => Text::from_parts(source, width).is_single_line(),
            Self::Fragment { summary, .. } => summary.capabilities().permits_flat(),
            Self::Alternatives(alternatives) => alternatives.first().capabilities().permits_flat(),
            _ => true,
        }
    }

    /// Applies a flat view without changing any retained instructions.
    // Keep projection state out of the ordinary printer and fit dispatch loops.
    #[inline(never)]
    fn flattened(self) -> Option<Self> {
        match self {
            Self::Line => Some(Self::text(Text::new(" "))),
            Self::IfFlat { source, width } => Some(Self::Text { source, width }),
            Self::Conditional(conditional) | Self::FlatConditional(conditional) => {
                Some(match conditional.target {
                    ConditionTarget::Current => Self::flat_fragment(FlatFragment(conditional.flat)),
                    ConditionTarget::Group(_) => Self::FlatConditional(conditional),
                })
            }
            Self::Fragment { elements, summary } => Some(Self::FlatFragment { elements, summary }),
            Self::Alternatives(alternatives) => {
                Some(Self::flat_fragment(FlatFragment(alternatives.first())))
            }
            element @ (Self::Text { .. }
            | Self::LineSuffixBoundary
            | Self::StartIndentIfBreak { .. }
            | Self::FlatFragment { .. }) => Some(element),
            _ => None,
        }
    }
}

pub struct Doc<'a> {
    allocator: &'a swc_next_allocator::Allocator,
    elements: &'a [FormatElement<'a>],
    group_count: u32,
}

impl Doc<'_> {
    /// Encodes the logical document into an owned version-0 buffer.
    ///
    /// The current JavaScript decoder requires text instructions to be single-line.
    /// Use [`Writer::literal_text`] for source text containing physical line breaks.
    /// This transport is still evolving and does not yet cover every document that
    /// the native printer can render.
    pub fn encode(self) -> Vec<u8> {
        encode::encode(self)
    }

    pub fn print(self, options: PrintOptions) -> String {
        printer::print(self.elements, self.allocator, self.group_count, options)
    }
}
