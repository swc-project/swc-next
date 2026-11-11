//! Flat instruction streams with borrowed text and explicit layout scopes.
//!
//! Construction owns a growable instruction buffer in the formatting arena.
//! Finishing seals an immutable slice; source text remains borrowed from its owner.
//! Compact instructions retain immutable borrows with checked lengths.
//! Line suffixes retain complete Docs and their insertion mode and indentation.
//! Syntax-specific whitespace and break requests belong to the caller's Doc.
//!
//! [`Writer`] selects flat [`Instruction`] storage directly from semantic builders.
//! [`Fragment`] seals a complete scope with its construction facts. Fit probes and
//! printing execute the same instructions; each graph analysis chooses its own
//! traversal policy. Encoding and decoding handle transport and Prettier interop.

mod breaks;
mod decode;
mod encode;
mod facts;
mod group;
mod indent;
mod instruction;
mod layout;
mod line;
mod printer;
mod query;
#[cfg(test)]
mod tests;
mod text;
mod thin;
mod transform;
mod wire;
mod writer;

pub use decode::DocumentBufferError;
pub use line::{LineMode, LineOptions};
pub use printer::{CursorError, PrintResult};
pub use writer::{FillEntry, FillSeparatorPolicy, Writer};

use std::num::NonZeroU32;

use crate::PrintOptions;
use group::Group;
use instruction::{AlignText, IfBreak, Instruction, RichIndentKind};
use text::Text;
use thin::ThinSlice;

/// Prettier group `id` / conditional `groupId`, remapped to a document-local identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupId(NonZeroU32);

impl GroupId {
    fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

/// Prettier `label.label`, represented by a caller-owned metadata ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelId(pub u32);

/// Conditions read either the active execution mode or an explicitly named group.
#[derive(Debug, Clone, Copy)]
pub enum ConditionTarget {
    /// An omitted Prettier `groupId`, reading the active execution mode.
    Current,
    /// Prettier `groupId`, reading the resolved mode of that named group.
    Group(GroupId),
}

/// Selects the group mode that controls conditional indentation.
#[derive(Debug, Clone, Copy)]
pub struct IndentIfBreakOptions {
    /// Prettier `groupId`; `Current` corresponds to an omitted group ID.
    pub target: ConditionTarget,
    /// Prettier `negate`: indent in flat mode instead of break mode.
    pub negate: bool,
}

impl Default for IndentIfBreakOptions {
    fn default() -> Self {
        Self {
            target: ConditionTarget::Current,
            negate: false,
        }
    }
}

/// Explicit layout requests use the same meaning during measurement and printing.
#[derive(Debug, Clone, Copy)]
pub enum GroupRequest {
    /// Prettier `group.break = false`, leaving layout to fitting and child effects.
    Auto,
    /// Prettier `group.break = true`, requesting its expanded layout.
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

/// A complete immutable scope, owned by the current formatting call.
#[derive(Debug, Clone, Copy)]
pub struct Fragment<'a> {
    /// A complete Doc scope stored as flat instructions; no separate Prettier node type.
    elements: ThinSlice<'a, Instruction<'a>>,
    /// Cached construction facts; no Prettier field.
    summary: FragmentSummary,
}

const _: () = assert!(std::mem::size_of::<Fragment<'_>>() <= 16);

/// Construction facts travel with every borrowed scope, including flat views.
/// They describe independent properties; projecting a scope does not erase them.
#[derive(Debug, Clone, Copy)]
struct FragmentSummary(u8);

impl FragmentSummary {
    const REQUIRED: u8 = 1;
    const CAPABILITIES: u8 = 0b110;
    const CONDITIONAL_LINES: u8 = 1 << 3;
    const INTRINSIC_LINES: u8 = 1 << 4;
    const SOURCE_GROUPS: u8 = 1 << 5;
    const RICH_INDENT: u8 = 1 << 6;
    const EXTENDED_PRINT: u8 = 1 << 7;

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

    fn rich_indent(self) -> bool {
        self.0 & Self::RICH_INDENT != 0
    }

    fn has_source_groups(self) -> bool {
        self.0 & Self::SOURCE_GROUPS != 0
    }

    fn intrinsic_lines(self) -> bool {
        self.0 & Self::INTRINSIC_LINES != 0
    }
}

/// Both views retain the same complete scope; only the anonymous outer group differs.
#[derive(Debug, Clone, Copy)]
pub struct RecordedGroup<'a> {
    /// One complete Prettier `group(contents)` scope, retained for both views.
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

/// Context-free ASCII measurements, saturated at the supported width limit.
/// Literal text spaces remain part of `width`;
/// only spaces contributed by `line` remain pending at the scope boundaries.
#[derive(Debug, Clone, Copy, Default)]
struct FlatSummary {
    /// Cached ASCII flat width; no Prettier field.
    width: u16,
    /// Whether flat contents include non-line text; no Prettier field.
    has_text: bool,
    /// Whether a leading `line` space is pending; internal measurement metadata.
    starts_with_line: bool,
    /// Whether a trailing `line` space is pending; internal measurement metadata.
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
fn flat_summary(elements: &[Instruction<'_>]) -> Option<FlatSummary> {
    let mut summary = FlatSummary::default();
    for element in elements {
        match element {
            Instruction::Text { source, width } | Instruction::TextIfFlat { source, width } => {
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
            Instruction::NormalLine => summary.ends_with_line = true,
            Instruction::SoftLine
            | Instruction::TextIfBreak { .. }
            | Instruction::StartGroup {
                effect: BreakEffect(0),
                id: None,
                ..
            }
            | Instruction::EndGroup
            | Instruction::StartIndent
            | Instruction::StartAbsoluteIndent { .. }
            | Instruction::EndIndent
            | Instruction::StartAlign { .. }
            | Instruction::EndAlign { .. } => {}
            _ => return None,
        }
    }
    Some(summary)
}

impl<'a> Fragment<'a> {
    fn new(elements: &'a [Instruction<'a>], summary: FragmentSummary) -> Self {
        Self {
            elements: ThinSlice::new(elements),
            summary,
        }
    }

    fn elements(self) -> &'a [Instruction<'a>] {
        self.elements.get()
    }

    fn rich_indent(self) -> bool {
        self.summary.rich_indent()
    }

    fn has_source_groups(self) -> bool {
        self.summary.has_source_groups()
    }

    fn requires_group_analysis(self) -> bool {
        self.has_source_groups() && group::requires_analysis(self)
    }

    fn extended_print(self) -> bool {
        self.summary.0 & FragmentSummary::EXTENDED_PRINT != 0
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

    /// Whether the primary contents contain an explicit line instruction.
    /// Visits both conditional branches and line suffixes, respects flat views,
    /// and ignores newlines in Text and requests to expand a group.
    pub fn can_break(self) -> bool {
        query::can_break(self)
    }

    /// Whether the primary contents contain a line instruction or multiline Text.
    /// Uses the same traversal and flat-view rules as [`Self::can_break`].
    pub fn has_line_breaks(self) -> bool {
        query::has_line_breaks(self)
    }

    /// Whether primary group contents request a break or contain a hard line.
    /// This query is independent of print width and distinct from propagation
    /// through a conditional group's candidate layouts.
    pub fn will_break(self) -> bool {
        query::will_break(self)
    }

    /// Whether construction requests expansion of enclosing groups. Candidate
    /// layouts form propagation barriers; explicit hard lines without a parent
    /// break request do not contribute. This is not [`Self::will_break`].
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
    fn classify(elements: &[Instruction<'_>]) -> Self {
        let mut elements = elements.iter().copied();
        while let Some(element) = elements.next() {
            let flat = match element {
                Instruction::Text { source, width } => {
                    if !Text::from_parts(source, width).is_single_line() {
                        return Self::General;
                    }
                    continue;
                }
                Instruction::StartGroup {
                    effect: BreakEffect(0),
                    id: None,
                    ..
                }
                | Instruction::StartGroupWithFlatWidth { .. }
                | Instruction::EndGroup => continue,
                Instruction::Fragment { summary, .. } => match summary.capabilities() {
                    Self::Neutral => continue,
                    Self::General => return Self::General,
                    Self::FlatEligible => true,
                },
                Instruction::FlatFragment { summary, .. }
                    if matches!(summary.capabilities(), Self::Neutral) =>
                {
                    continue;
                }
                Instruction::FlatFragment { .. } => true,
                element => element.permits_flat(),
            };
            return Self::from_flat(flat && elements.all(Instruction::permits_flat));
        }
        Self::Neutral
    }
}

/// A flat-layout view of retained instructions, exported through standard Prettier
/// Docs. This is a layout projection, independent of the flat storage format.
#[derive(Debug, Clone, Copy)]
pub struct FlatFragment<'a>(Fragment<'a>);

pub struct Doc<'a> {
    /// Arena owning instruction storage; no Prettier field.
    allocator: &'a swc_next_allocator::Allocator,
    /// The root Prettier Doc represented as a complete flat scope.
    root: Fragment<'a>,
    /// Capacity for local group decisions; no Prettier field.
    group_count: u32,
}

impl<'a> Doc<'a> {
    /// Whether the Doc contains an explicit line instruction, independent of
    /// print width. Visits both IfBreak branches and line suffixes, follows
    /// only the primary contents of conditional groups, and respects flat
    /// projections. Ignores newlines in Text and requests to expand a group.
    pub fn can_break(&self) -> bool {
        self.root.can_break()
    }

    /// Whether primary contents contain a line instruction or multiline Text.
    /// See [`Fragment::has_line_breaks`] for traversal and projection rules.
    pub fn has_line_breaks(&self) -> bool {
        self.root.has_line_breaks()
    }

    /// Whether the Doc requests a break or contains a hard line in its primary
    /// group contents. This query is independent of the print width.
    pub fn will_break(&self) -> bool {
        self.root.will_break()
    }

    /// Replaces ordinary lines with spaces and soft lines with empty text.
    /// Selects the flat branch of IfBreak and retains hard lines and groups.
    pub fn remove_lines(self) -> Self {
        transform::remove_lines(self)
    }

    /// Replaces LF within Text with a literal line and a parent break request.
    pub fn replace_end_of_line(self) -> Self {
        transform::replace_end_of_line(self, None)
    }

    /// Replaces LF within Text with a separate Doc. The replacement's named
    /// groups are scoped independently and its label IDs are preserved.
    pub fn replace_end_of_line_with(self, replacement: Self) -> Self {
        transform::replace_end_of_line(self, Some(replacement))
    }

    /// Decodes a versioned document, borrowing its text from the input buffer.
    ///
    /// Supports version 0's complete balanced fragment scopes, including
    /// shared fragments. Line suffixes must not depend on a fill outside the
    /// suffix. This is not a decoder for arbitrary Prettier documents.
    ///
    /// Returns an error for unsupported versions or malformed documents.
    /// Distinct text ranges may consume at most eight times the buffer length
    /// in decoding work; exceeding this limit returns a resource error. The
    /// returned document's later printing work is not bounded by this limit.
    pub fn decode(
        allocator: &'a swc_next_allocator::Allocator,
        buffer: &'a [u8],
    ) -> Result<Self, DocumentBufferError> {
        decode::decode(allocator, buffer)
    }

    /// Encodes the logical document into an owned version-0 buffer.
    ///
    /// Text instructions retain their complete strings, including source newlines.
    /// This transport is still evolving and does not yet cover every document that
    /// the native printer can render.
    pub fn encode(self) -> Vec<u8> {
        encode::encode::<false>(self)
    }

    /// Encodes a version-0 buffer for execution by a Prettier Doc printer.
    ///
    /// Expanded groups export their primary contents as the first state,
    /// followed by their executable alternatives and fallback. This preserves
    /// group decisions and shared scopes while exposing all executable layouts
    /// to Prettier's break propagation. Use [`Self::encode`] to retain the
    /// original contents and expanded-state fields for structural round trips.
    pub fn encode_for_prettier(self) -> Vec<u8> {
        encode::encode::<true>(self)
    }

    pub fn print(self, options: PrintOptions) -> String {
        printer::print(self, options, false)
            .expect("cursor collection is disabled")
            .text
    }

    /// Prints the Doc and returns the UTF-8 byte range between two emitted cursors.
    /// Zero or one cursor produces no range. More than two produces an error.
    pub fn print_with_cursor(self, options: PrintOptions) -> Result<PrintResult, CursorError> {
        printer::print(self, options, true)
    }
}
