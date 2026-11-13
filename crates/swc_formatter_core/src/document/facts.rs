//! Construction facts shared by writers, decoding, and rewritten fragments.
//!
//! Mutable fields keep typed construction cheap; sealing packs them into the
//! immutable fragment summary. Logical queries and identity-sensitive group
//! propagation remain separate analyses with their own traversal policies.

use super::{
    BreakEffect, Fragment, FragmentCapabilities, FragmentSummary, Group, GroupRequest, IfBreak,
    Instruction, Text,
};

pub(super) enum GroupBoundary {
    None,
    Start,
    End,
}

#[derive(Default)]
pub(super) struct Facts {
    effect: BreakEffect,
    conditional_lines: bool,
    intrinsic_lines: bool,
    source_groups: bool,
    rich_indent: bool,
    extended_print: bool,
}

impl Facts {
    /// Native text constructors omit empty strings before recording them.
    #[inline]
    pub(super) fn nonempty_text(&mut self, text: Text<'_>) {
        debug_assert!(!text.source().is_empty());
        if !text.is_single_line() {
            self.include_intrinsic_lines(true);
        }
    }

    #[inline]
    fn text(&mut self, text: Text<'_>) {
        if text.source().is_empty() {
            // Retained empty instructions must not commit deferred line spaces.
            self.require_extended_print();
        } else {
            self.nonempty_text(text);
        }
    }

    #[inline]
    pub(super) fn require_break(&mut self) {
        self.effect = BreakEffect::REQUIRED;
    }

    #[inline]
    pub(super) fn hard_line(&mut self, expand: bool) {
        self.intrinsic_lines = true;
        if expand {
            self.require_break();
        }
    }

    #[inline]
    pub(super) fn conditional_line(&mut self) {
        self.conditional_lines = true;
    }

    #[inline]
    pub(super) fn line_suffix(&mut self) {
        self.intrinsic_lines = true;
    }

    #[inline]
    pub(super) fn require_rich_indent(&mut self) {
        self.rich_indent = true;
    }

    #[inline]
    pub(super) fn require_extended_print(&mut self) {
        self.extended_print = true;
    }

    #[inline(always)]
    fn layout_flags(&mut self, flags: u8) {
        self.source_groups |= flags & FragmentSummary::SOURCE_GROUPS != 0;
        self.rich_indent |= flags & FragmentSummary::RICH_INDENT != 0;
        self.extended_print |= flags & FragmentSummary::EXTENDED_PRINT != 0;
    }

    #[inline(always)]
    fn state_flags(&mut self, flags: u8) {
        self.layout_flags(flags);
        self.conditional_lines |= flags & FragmentSummary::CONDITIONAL_LINES != 0;
    }

    #[inline(always)]
    fn fragment_flags(&mut self, flags: u8) {
        self.state_flags(flags);
        self.effect = self
            .effect
            .join(BreakEffect(flags & FragmentSummary::REQUIRED));
        self.intrinsic_lines |= flags & FragmentSummary::INTRINSIC_LINES != 0;
    }

    /// Requirements retained even in a flat view or a non-primary state.
    #[inline(always)]
    pub(super) fn flat_fragment(&mut self, fragment: Fragment<'_>) {
        self.layout_flags(fragment.summary.0);
    }

    #[inline(always)]
    pub(super) fn fragment(&mut self, fragment: Fragment<'_>) {
        self.fragment_flags(fragment.summary.0);
    }

    #[inline(always)]
    pub(super) fn if_break(&mut self, conditional: &IfBreak<'_>, flat: bool) {
        let flags = conditional.broken.summary.0 | conditional.flat.summary.0;
        if flat {
            self.layout_flags(flags);
        } else {
            self.fragment_flags(flags);
        }
    }

    /// Candidate layouts share facts but stop outward break propagation.
    #[inline(always)]
    pub(super) fn group<'a>(&mut self, group: impl Group<'a>) {
        // Capabilities are classified separately when sealing the scope.
        self.state_flags(group.layout_flags());
        if group.has_source_fields() {
            self.source_groups = true;
            self.include_request(group.request());
        }
        self.intrinsic_lines |= group.primary_ref().summary.intrinsic_lines();
    }

    /// Dynamic instructions use the same semantic operations as typed writers.
    /// Return inline group boundaries so a rewriting caller can update cached
    /// group measurements without classifying each instruction a second time.
    #[inline(always)]
    pub(super) fn observe(&mut self, element: Instruction<'_>) -> GroupBoundary {
        use Instruction as I;
        match element {
            I::StartGroup { .. } | I::StartGroupWithFlatWidth { .. } => {
                return GroupBoundary::Start;
            }
            I::EndGroup => return GroupBoundary::End,
            I::BreakParent => self.require_break(),
            I::HardLine | I::LiteralLine | I::LiteralLineAtCurrentIndent => self.hard_line(true),
            I::HardLineOnly | I::LiteralLineOnly => self.hard_line(false),
            I::StartLineSuffix { .. } => self.line_suffix(),
            I::Text { source, width }
            | I::TextIfBreak { source, width }
            | I::TextIfFlat { source, width } => {
                self.text(Text::from_parts(source, width));
            }
            I::Trim => self.require_extended_print(),
            I::SoftLineIfGroupBreaks(_) => self.conditional_line(),
            I::StartAlignText(_) | I::StartDedent | I::StartMarkRoot | I::StartDedentToRoot => {
                self.require_rich_indent();
            }
            I::Fragment { elements, summary } => self.fragment(Fragment { elements, summary }),
            I::FlatFragment { elements, summary } => {
                self.flat_fragment(Fragment { elements, summary })
            }
            I::IfBreak(c) => self.if_break(c, false),
            I::FlatIfBreak(c) => self.if_break(c, true),
            I::CompactGroup(group) => self.group(group),
            I::SourceGroup(group) => self.group(group),
            _ => {}
        }
        GroupBoundary::None
    }

    #[inline]
    pub(super) fn effect(&self) -> BreakEffect {
        self.effect
    }

    #[inline]
    pub(super) fn include_request(&mut self, request: GroupRequest) {
        self.effect = self.effect.join(BreakEffect::from_request(request));
    }

    #[inline]
    pub(super) fn has_intrinsic_lines(&self) -> bool {
        self.intrinsic_lines
    }

    #[inline]
    pub(super) fn take_intrinsic_lines(&mut self) -> bool {
        std::mem::take(&mut self.intrinsic_lines)
    }

    #[inline]
    pub(super) fn include_intrinsic_lines(&mut self, intrinsic: bool) {
        self.intrinsic_lines |= intrinsic;
    }

    #[inline]
    pub(super) fn begin_group(&mut self) -> BreakEffect {
        std::mem::take(&mut self.effect)
    }

    #[inline]
    pub(super) fn end_group(&mut self, request: GroupRequest, outer: BreakEffect) -> BreakEffect {
        let effect = BreakEffect::from_request(request).join(self.effect);
        self.effect = outer.join(effect);
        effect
    }

    #[inline(always)]
    pub(super) fn finish(self, elements: &[Instruction<'_>]) -> FragmentSummary {
        let capabilities = if self.effect.preferred_breaks() || self.conditional_lines {
            FragmentCapabilities::from_flat(elements.iter().all(|element| element.permits_flat()))
        } else {
            FragmentCapabilities::classify(elements)
        };
        self.finish_with_capabilities(capabilities)
    }

    #[inline(always)]
    pub(super) fn finish_with_capabilities(
        self,
        capabilities: FragmentCapabilities,
    ) -> FragmentSummary {
        FragmentSummary(
            u8::from(self.effect.preferred_breaks())
                | ((capabilities as u8) << 1)
                | (u8::from(self.conditional_lines) << 3)
                | (u8::from(self.intrinsic_lines) << 4)
                | (u8::from(self.source_groups) << 5)
                | (u8::from(self.rich_indent) << 6)
                | (u8::from(self.extended_print) << 7),
        )
    }
}
