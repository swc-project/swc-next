//! Flat execution storage selected directly by construction and decoding.
//!
//! | Prettier Doc | Storage |
//! | --- | --- |
//! | strings and lines | Direct instructions; common compositions stay fused |
//! | `group(contents)` | Balanced scopes, optionally with cached flat measurements |
//! | group candidates | `CompactGroup` or `SourceGroup`, sharing the group contract |
//! | `ifBreak` | Branch references, or inline text for a single current-mode branch |
//! | `fill(parts)` | Balanced Fill and FillPart scopes |
//! | other `contents` wrappers | Balanced scopes |
//! | shared arrays or flat projections | `Fragment` or `FlatFragment` references |
//!
//! Skip offsets, cached measurements, and construction summaries are native
//! execution metadata. The codec maps these instructions to version-0 transport.

use super::{
    BreakEffect, ConditionTarget, FlatFragment, FlatSummary, Fragment, FragmentSummary, GroupId,
    GroupRequest, LabelId, LineMode, LineOptions,
    group::{CompactGroup, Group, SourceGroup},
    text::{Text, Width},
    thin::{ThinSlice, ThinStr},
};

#[derive(Debug, Clone, Copy)]
pub(super) enum RichIndentKind {
    AlignText,
    Dedent,
    MarkRoot,
    DedentToRoot,
}

/// Flat storage for Prettier Doc semantics and specialized standard Doc compositions.
///
/// Prettier's `contents` become balanced start/end instructions; widths, summaries,
/// and scope offsets are internal metadata rather than Prettier Doc fields.
///
/// Common names denote complete builder semantics: `HardLine` and `LiteralLine`
/// include `breakParent`; `Only` omits that expansion request. Literal lines keep
/// trailing whitespace and restore the marked root. `AtCurrentIndent` additionally
/// marks the current indentation as that root. `IfGroupBreaks` reads a named group.
///
/// `TextIfBreak` / `TextIfFlat` specialize `IfBreak` for current-mode string branches.
/// `FlatIfBreak` and `FlatFragment` retain a flat projection without rewriting source
/// storage. Groups keep their inline, measured, compact, or source-preserving forms.
/// These specializations are sibling variants with one execution discriminant.
#[derive(Debug, Clone, Copy)]
pub(super) enum Instruction<'a> {
    /// A Prettier string Doc, with an internal measurement cache.
    Text {
        /// The borrowed string Doc itself.
        source: ThinStr<'a>,
        /// Cached text measurement under our width policy; no Prettier field.
        width: Width,
    },
    /// Prettier `softline`: `{ type: "line", soft: true }`.
    SoftLine,
    /// Fused `ifBreak(softline, "", { groupId })`; the ID selects the tested group.
    SoftLineIfGroupBreaks(GroupId),
    /// Prettier `line`: `{ type: "line" }`, a space in flat mode.
    NormalLine,
    /// Prettier `hardline`: `[hardlineWithoutBreakParent, breakParent]`, fused.
    HardLine,
    /// Prettier `hardlineWithoutBreakParent`: `{ type: "line", hard: true }`.
    HardLineOnly,
    /// Prettier `breakParent`; requests expansion without printing a newline.
    BreakParent,
    /// Prettier `literalline`: `[literallineWithoutBreakParent, breakParent]`, fused.
    LiteralLine,
    /// Prettier `{ type: "line", hard: true, literal: true }`, without `breakParent`.
    LiteralLineOnly,
    /// Fused `markAsRoot(literalline)`, including `breakParent`; resumes at the current indentation.
    LiteralLineAtCurrentIndent,
    /// Fused `ifBreak(text, "")` controlled by the current execution mode.
    TextIfBreak {
        /// The string Doc in Prettier's `breakContents`.
        source: ThinStr<'a>,
        /// Cached text measurement; no Prettier `if-break` field.
        width: Width,
    },
    /// Fused `ifBreak("", text)` controlled by the current execution mode.
    TextIfFlat {
        /// The string Doc in Prettier's `flatContents`.
        source: ThinStr<'a>,
        /// Cached text measurement; no Prettier `if-break` field.
        width: Width,
    },
    /// Prettier `ifBreak(breakContents, flatContents, { groupId })`; payload holds both branches.
    IfBreak(&'a IfBreak<'a>),
    /// Internal flat projection of a named `ifBreak`; the condition still reads its group.
    /// Export projects both branch Docs instead of introducing a new Prettier node type.
    FlatIfBreak(&'a IfBreak<'a>),
    /// Opens Prettier `lineSuffix(contents)`; even empty contents register a pending suffix.
    StartLineSuffix {
        /// Distance to `EndLineSuffix`; an internal skip offset, not a Doc field.
        end_offset: u32,
    },
    EndLineSuffix,
    /// Prettier `lineSuffixBoundary`; inserts a newline only when a suffix is pending.
    LineSuffixBoundary,
    /// Opens Prettier `group(contents, { shouldBreak, id })` without `expandedStates`.
    StartGroup {
        /// Builder `shouldBreak`, stored as `break` on the Prettier Doc.
        request: GroupRequest,
        /// Cached request plus child break effects; Prettier propagates these into `break`.
        effect: BreakEffect,
        /// Prettier group `id`, remapped to a local ID; `None` is anonymous.
        id: Option<GroupId>,
        /// Distance to `EndGroup`; internal scope metadata, not a Doc field.
        end_offset: u32,
    },
    /// Prettier `group(contents)`, anonymous and Auto, with cached flat measurements.
    StartGroupWithFlatWidth {
        /// Context-free ASCII flat fit measurements; omitted from the exported Prettier Doc.
        summary: FlatSummary,
        /// Distance to `EndGroup`; internal scope metadata, not a Doc field.
        end_offset: u32,
    },
    EndGroup,
    /// Opens Prettier `indent(contents)`, adding one configured indentation level.
    StartIndent,
    /// Root-relative indentation, expressed through standard Doc compositions.
    /// Positive widths export via `addAlignmentToDoc(contents, width, tabWidth)`;
    /// zero exports as `dedentToRoot(contents)`.
    StartAbsoluteIndent {
        /// Root-relative indentation width; distinct from relative `align(n, contents)`.
        width: u32,
    },
    /// Opens Prettier `indentIfBreak(contents, { groupId, negate })`.
    StartIndentIfBreak {
        /// Prettier `groupId`; `Current` corresponds to an omitted group ID.
        target: ConditionTarget,
        /// Prettier `negate`: indent when the selected mode is flat instead.
        negate: bool,
        /// Distance to `EndIndent`; internal scope metadata, not a Doc field.
        end_offset: u32,
    },
    EndIndent,
    /// Opens numeric Prettier `align(n, contents)` with nonnegative `n`.
    StartAlign {
        /// Prettier `align`'s numeric `n`, interpreted under the indentation options.
        width: u32,
    },
    /// Opens string Prettier `align(prefix, contents)`; payload holds `n` and its cached width.
    StartAlignText(&'a AlignText<'a>),
    /// Opens Prettier `dedent(contents)`, i.e. `align(-1, contents)`.
    StartDedent,
    /// Opens Prettier `markAsRoot(contents)`, i.e. `align({ type: "root" }, contents)`.
    StartMarkRoot,
    /// Opens Prettier `dedentToRoot(contents)`, i.e. `align(-Infinity, contents)`.
    StartDedentToRoot,
    EndRichIndent(RichIndentKind),
    EndAlign {
        width: u32,
    },
    /// Opens Prettier `fill(parts)`; each part has its own scope below.
    StartFill,
    EndFill,
    /// Opens one Doc in Prettier `fill.parts`, preserving alternating content/separator positions.
    /// Empty entries still occupy a part; this wrapper has no standalone Prettier node type.
    StartFillPart {
        /// Distance to `EndFillPart`; internal scope metadata, not a Doc field.
        end_offset: u32,
    },
    EndFillPart,
    /// A shared complete Doc sequence; exports as an ordinary Doc or array, not a custom node.
    Fragment {
        /// The referenced sequence's standard Doc contents, retaining sharing.
        elements: ThinSlice<'a, Instruction<'a>>,
        /// Cached construction facts and print capabilities; no Prettier field.
        summary: FragmentSummary,
    },
    /// An internal flat projection of a shared scope, exported as ordinary Docs.
    /// It flattens lines, resolves current-mode conditionals, and selects primary group layouts.
    FlatFragment {
        /// Retained source instructions; flat projection does not rewrite this slice.
        elements: ThinSlice<'a, Instruction<'a>>,
        /// Facts of the retained source scope; projection is tracked separately.
        summary: FragmentSummary,
    },
    /// Compact `conditionalGroup([...preferred, fallback])` with `contents = preferred[0]`.
    /// The payload has anonymous Auto metadata and stores the candidates once.
    CompactGroup(&'a CompactGroup<'a>),
    /// Prettier `group` with independent `contents`, `expandedStates`, `break`, and `id`.
    /// Raw fields remain independent; Prettier export puts `contents` first in executable states.
    SourceGroup(&'a SourceGroup<'a>),
    /// Raw Prettier `{ type: "line", hard: false, soft: true, literal: true }`.
    /// Empty in flat mode; preserves whitespace and restores the marked root when broken.
    SoftLiteralLine,
    /// Raw Prettier `{ type: "line", hard: false, soft: false, literal: true }`.
    /// A space in flat mode; preserves whitespace and restores the marked root when broken.
    NormalLiteralLine,
    /// Prettier `trim`; removes trailing spaces and tabs already written.
    Trim,
    /// Prettier `cursor`; marks an output position without emitting text.
    Cursor,
    /// Opens Prettier `label(value, contents)`; the ID represents its `label` field.
    StartLabel(LabelId),
    EndLabel,
}

const _: () = assert!(std::mem::size_of::<Instruction<'_>>() <= 16);

#[derive(Debug)]
pub(super) struct AlignText<'a> {
    /// The string `n` in Prettier `align(n, contents)`.
    pub(super) source: &'a str,
    /// Cached prefix measurement under our width policy; no Prettier field.
    pub(super) width: usize,
}

#[derive(Debug)]
pub(super) struct IfBreak<'a> {
    /// Prettier `groupId`; `Current` corresponds to an omitted group ID.
    pub(super) target: ConditionTarget,
    /// Prettier `if-break.breakContents`.
    pub(super) broken: Fragment<'a>,
    /// Prettier `if-break.flatContents`.
    pub(super) flat: Fragment<'a>,
}

impl<'a> Instruction<'a> {
    /// Selects one raw line instruction; expansion requests are separate instructions.
    #[inline]
    pub(super) fn raw_line(options: LineOptions) -> Self {
        match (options.mode, options.literal) {
            (LineMode::Hard, true) => Self::LiteralLineOnly,
            (LineMode::Hard, false) => Self::HardLineOnly,
            (LineMode::Soft, true) => Self::SoftLiteralLine,
            (LineMode::Normal, true) => Self::NormalLiteralLine,
            (LineMode::Soft, false) => Self::SoftLine,
            (LineMode::Normal, false) => Self::NormalLine,
        }
    }

    #[inline]
    pub(super) fn group_key(&self) -> Option<usize> {
        match self {
            Self::CompactGroup(group) => Some(group.key()),
            Self::SourceGroup(group) => Some(group.key()),
            _ => None,
        }
    }

    pub(super) fn text(text: Text<'a>) -> Self {
        let (source, width) = text.into_parts();
        Self::Text { source, width }
    }

    pub(super) fn text_if_break(text: Text<'a>) -> Self {
        let (source, width) = text.into_parts();
        Self::TextIfBreak { source, width }
    }

    pub(super) fn text_if_flat(text: Text<'a>) -> Self {
        let (source, width) = text.into_parts();
        Self::TextIfFlat { source, width }
    }

    pub(super) fn fragment(fragment: Fragment<'a>) -> Self {
        Self::Fragment {
            elements: fragment.elements,
            summary: fragment.summary,
        }
    }

    pub(super) fn flat_fragment(FlatFragment(fragment): FlatFragment<'a>) -> Self {
        Self::FlatFragment {
            elements: fragment.elements,
            summary: fragment.summary,
        }
    }

    pub(super) fn permits_flat(self) -> bool {
        match self {
            Self::HardLine
            | Self::HardLineOnly
            | Self::LiteralLine
            | Self::LiteralLineOnly
            | Self::LiteralLineAtCurrentIndent
            | Self::StartLineSuffix { .. }
            | Self::EndLineSuffix
            | Self::BreakParent
            // Named groups commit an observable layout decision. Removing that
            // decision cannot be expressed by a flat view of standard Doc.
            | Self::StartGroup { id: Some(_), .. } => false,
            Self::IfBreak(conditional) | Self::FlatIfBreak(conditional) => {
                conditional.broken.capabilities().permits_flat()
                    && conditional.flat.capabilities().permits_flat()
            }
            Self::Text { source, width } | Self::TextIfFlat { source, width } => Text::from_parts(source, width).is_single_line(),
            Self::Fragment { summary, .. } => summary.capabilities().permits_flat(),
            Self::CompactGroup(group) => group.primary().capabilities().permits_flat(),
            Self::SourceGroup(group) => {
                group.id().is_none() && group.primary().capabilities().permits_flat()
            }
            _ => true,
        }
    }

    /// Applies a flat view without changing any retained instructions.
    // Keep projection state out of the ordinary printer and fit dispatch loops.
    #[inline(never)]
    pub(super) fn flattened(self) -> Option<Self> {
        match self {
            Self::NormalLine | Self::NormalLiteralLine => Some(Self::text(Text::new(" "))),
            Self::TextIfFlat { source, width } => Some(Self::Text { source, width }),
            Self::IfBreak(conditional) | Self::FlatIfBreak(conditional) => {
                Some(match conditional.target {
                    ConditionTarget::Current => Self::flat_fragment(FlatFragment(conditional.flat)),
                    ConditionTarget::Group(_) => Self::FlatIfBreak(conditional),
                })
            }
            Self::Fragment { elements, summary } => Some(Self::FlatFragment { elements, summary }),
            Self::CompactGroup(group) => Some(Self::flat_fragment(FlatFragment(group.primary()))),
            Self::SourceGroup(group) => Some(Self::flat_fragment(FlatFragment(group.primary()))),
            Self::StartIndentIfBreak {
                target: ConditionTarget::Current,
                ..
            } => None,
            element @ (Self::Text { .. }
            | Self::Trim
            | Self::Cursor
            | Self::StartLabel(_)
            | Self::EndLabel
            | Self::LineSuffixBoundary
            | Self::StartIndentIfBreak { .. }
            | Self::FlatFragment { .. }) => Some(element),
            _ => None,
        }
    }
}
