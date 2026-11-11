//! Flat instruction storage for borrowed layout documents.

use super::{
    BreakEffect, CompactGroup, FlatSummary, FragmentSummary, GroupId, IfBreak, ThinSlice, ThinStr,
    Width,
};

#[derive(Debug, Clone, Copy)]
pub(super) enum Instruction<'a> {
    Text {
        source: ThinStr<'a>,
        width: Width,
    },
    SoftLine,
    SoftLineIfGroupBreaks(GroupId),
    NormalLine,
    HardLine,
    BreakParent,
    LiteralLine,
    /// The standard composition `markAsRoot(literalline)`.
    LiteralLineAtCurrentIndent,
    TextIfBreak {
        source: ThinStr<'a>,
        width: Width,
    },
    TextIfFlat {
        source: ThinStr<'a>,
        width: Width,
    },
    IfBreak(&'a IfBreak<'a>),
    FlatConditional(&'a IfBreak<'a>),
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
    StartFillPart {
        end_offset: u32,
    },
    EndFillPart,
    Fragment {
        elements: ThinSlice<'a, Instruction<'a>>,
        summary: FragmentSummary,
    },
    FlatFragment {
        elements: ThinSlice<'a, Instruction<'a>>,
        summary: FragmentSummary,
    },
    CompactGroup(&'a CompactGroup<'a>),
}
