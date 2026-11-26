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
mod normalize;
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
pub use writer::{FillEntry, FillSeparatorPolicy, GroupScope, IndentScope, Writer};

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
struct FragmentSummary(u16);

impl FragmentSummary {
    const EMPTY: Self = Self((FragmentCapabilities::Neutral as u16) << 1);

    const REQUIRED: u16 = 1;
    const CAPABILITIES: u16 = 0b110;
    const CONDITIONAL_LINES: u16 = 1 << 3;
    const INTRINSIC_LINES: u16 = 1 << 4;
    const SOURCE_GROUPS: u16 = 1 << 5;
    const RICH_INDENT: u16 = 1 << 6;
    const EXTENDED_PRINT: u16 = 1 << 7;
    const HAS_LABELS: u16 = 1 << 8;

    fn effect(self) -> BreakEffect {
        BreakEffect((self.0 & Self::REQUIRED) as u8)
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

    fn has_labels(self) -> bool {
        self.0 & Self::HAS_LABELS != 0
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

    /// Whether the shared contents put their final text at the current indent
    /// when their enclosing group breaks.
    fn breaks_before_final_text(self) -> bool {
        matches!(
            self.contents().elements(),
            [
                ..,
                Instruction::SoftLine | Instruction::NormalLine,
                Instruction::Text { .. }
            ]
        )
    }
}

/// Known ASCII widths surrounding a projected layout line.
struct FlatLineBounds {
    prefix: usize,
    suffix: usize,
    indent: usize,
    indent_width: usize,
}

/// Context-free ASCII measurements with a width representable as `u16`.
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

/// Combines a completed group's top-level instructions. Cached inner groups are
/// skipped, so closing a chain of nested groups does not visit the same text again.
#[inline]
fn flat_summary(mut elements: &[Instruction<'_>]) -> Option<FlatSummary> {
    let mut summary = FlatSummary::default();
    let mut total_width = 0u64;
    while let Some((element, rest)) = elements.split_first() {
        elements = rest;
        let (width, starts_with_line, ends_with_line) = match element {
            Instruction::Text { source, width } | Instruction::TextIfFlat { source, width } => {
                let width = Text::from_parts(*source, *width).clean_ascii_width()?;
                (width as u64, false, false)
            }
            Instruction::NormalLine => {
                summary.ends_with_line = true;
                continue;
            }
            Instruction::StartGroupWithFlatWidth {
                summary: child,
                end_offset,
            } => {
                elements = &elements[*end_offset as usize..];
                if !child.has_text {
                    summary.ends_with_line |= child.ends_with_line;
                    continue;
                }
                (
                    u64::from(child.width),
                    child.starts_with_line,
                    child.ends_with_line,
                )
            }
            Instruction::SoftLine
            | Instruction::TextIfBreak { .. }
            | Instruction::StartIndent
            | Instruction::StartAbsoluteIndent { .. }
            | Instruction::EndIndent
            | Instruction::StartAlign { .. }
            | Instruction::EndAlign { .. } => continue,
            _ => return None,
        };
        let separator = summary.ends_with_line || starts_with_line;
        total_width += width + u64::from(summary.has_text && separator);
        summary.starts_with_line |= !summary.has_text && separator;
        summary.has_text = true;
        summary.ends_with_line = ends_with_line;
    }
    // The group length and every text length are bounded by u32. Narrow
    // the total only at the end, including at most one separator per item.
    summary.width = u16::try_from(total_width).ok()?;
    Some(summary)
}

impl<'a> Fragment<'a> {
    fn empty() -> Self {
        Self::new(&[], FragmentSummary::EMPTY)
    }

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

impl FlatFragment<'_> {
    /// Whether the flat projection exceeds the width even at column zero.
    /// Layout dependencies outside the fragment leave the result conservative.
    pub fn exceeds_line_width(self, line_width: u16) -> bool {
        !printer::standalone_flat_fits(self.0, line_width)
    }
}

pub struct Doc<'a> {
    /// Arena owning instruction storage; no Prettier field.
    allocator: &'a swc_next_allocator::Allocator,
    /// The root Prettier Doc represented as a complete flat scope.
    root: Fragment<'a>,
    /// Capacity for local group decisions; no Prettier field.
    group_count: u32,
    /// An intact, validated transport trailer borrowed from a decoded buffer.
    label_data: &'a [u8],
}

impl<'a> Doc<'a> {
    /// Whether the root has no layout instructions.
    pub fn is_empty(&self) -> bool {
        self.root.elements().is_empty()
    }

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

    /// Replaces ordinary lines with spaces and removes soft lines.
    /// Selects the flat branch of IfBreak and retains hard lines and groups.
    pub fn remove_lines(mut self) -> Self {
        self.root = transform::remove_lines(self.allocator, self.root);
        self
    }

    /// Replaces LF within Text with a literal line and a parent break request.
    pub fn replace_end_of_line(self) -> Self {
        transform::replace_end_of_line(self, None)
    }

    /// Replaces LF within Text with a separate Doc. The replacement's named
    /// groups are scoped independently and its label IDs are preserved.
    /// A single labeled input or a shared transport table carries its records
    /// into the result. For conflicting label namespaces, supply a combined
    /// table when encoding the result.
    pub fn replace_end_of_line_with(self, replacement: Self) -> Self {
        transform::replace_end_of_line(self, Some(replacement))
    }

    /// Coalesces adjacent Text and removes inert empty wrappers in logical sequences.
    /// Empty scopes with layout or metadata effects are retained.
    pub fn clean(self) -> Self {
        normalize::clean(self)
    }

    /// Cleans the Doc, then removes trailing hardline pairs and terminal
    /// newlines from the final logical part of each eligible branch.
    pub fn strip_trailing_hardline(self) -> Self {
        normalize::strip_trailing_hardline(self)
    }

    /// Decodes a versioned document, borrowing text and opaque label data from the buffer.
    ///
    /// Supports version 0's complete balanced fragment scopes, including
    /// shared fragments. Line suffixes must not depend on a fill outside the
    /// suffix. This is not a decoder for arbitrary Prettier documents.
    ///
    /// Returns an error for unsupported versions or malformed documents. An
    /// optional label table must contain a record for every encoded label ID.
    /// Distinct text ranges may consume at most eight times the byte length
    /// through the end of the string pool in decoding work. Exceeding this
    /// limit returns a resource error. This limit applies to decoding.
    pub fn decode(
        allocator: &'a swc_next_allocator::Allocator,
        buffer: &'a [u8],
    ) -> Result<Self, DocumentBufferError> {
        decode::decode(allocator, buffer)
    }

    /// Encodes the logical document into an owned version-0 buffer.
    ///
    /// Text instructions retain their complete strings, including source newlines.
    /// A decoded document retains its label records when re-encoded.
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

    /// Encodes a Prettier Doc buffer with opaque records indexed by [`LabelId`].
    ///
    /// Every exported label ID must index the supplied records. Several scopes
    /// can use the same ID, and records may be empty or unused. An empty slice
    /// emits the ordinary version-0 layout. The native printer renders the label contents.
    ///
    /// Returns an error for out-of-range IDs and tables beyond the transport limits.
    pub fn encode_for_prettier_with_label_data(
        self,
        records: &[impl AsRef<[u8]>],
    ) -> Result<Vec<u8>, DocumentBufferError> {
        encode::encode_with_label_data(self, records)
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
