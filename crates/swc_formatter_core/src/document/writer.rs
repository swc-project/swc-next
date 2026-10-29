//! Direct instruction construction with stable borrowed and generated text.

use std::borrow::Cow;

use swc_next_allocator::{Allocator, vec::ArenaVec};

use super::{
    Alternatives, BreakEffect, ConditionTarget, Conditional, Doc, FlatFragment, FormatElement,
    Fragment, FragmentCapabilities, GroupId, GroupRequest, GroupWithStates, NeutralFragment,
    RecordedGroup, text::Text, thin::ThinSlice,
};

pub struct Writer<'a> {
    allocator: &'a Allocator,
    elements: ArenaVec<'a, FormatElement<'a>>,
    effect: BreakEffect,
    flattening: bool,
    group_count: u32,
    group_states: bool,
    conditional_lines: bool,
    intrinsic_lines: bool,
}

/// A completed entry can retain a suffix while it is still the final scope.
pub struct FillEntry {
    start: usize,
    end: usize,
}

/// Treatment of empty separator lines after a complete Fill has been built.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FillSeparatorPolicy {
    Preserve,
    CollapseEmptyLines,
}

fn instruction_offset(value: usize) -> u32 {
    u32::try_from(value).expect("document instruction distance exceeds u32::MAX")
}

fn indentation_width(value: usize) -> u32 {
    u32::try_from(value).expect("document indentation exceeds u32::MAX")
}

impl<'a> Writer<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            elements: ArenaVec::new_in(allocator),
            effect: BreakEffect::default(),
            flattening: false,
            group_count: 0,
            group_states: false,
            conditional_lines: false,
            intrinsic_lines: false,
        }
    }

    /// Appends one string whose complete source width participates in layout.
    /// Internal newlines do not reset the layout column or expand groups; LF is
    /// converted to the selected output line ending. Use [`Self::literal_text`]
    /// when source lines should instead behave as explicit literal lines.
    pub fn text<'text>(&mut self, text: impl Into<Cow<'text, str>>)
    where
        'text: 'a,
    {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        let text = match text {
            Cow::Borrowed(text) => text,
            Cow::Owned(text) => self.allocator.alloc_str(&text),
        };
        let text = Text::new(text);
        if !text.is_single_line() {
            self.intrinsic_lines = true;
        }
        self.elements.push(FormatElement::text(text));
    }

    /// Copies transient text into storage owned by this formatting call.
    pub fn text_copied(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let text = self.allocator.alloc_str(text);
        let text = Text::new(text);
        if !text.is_single_line() {
            self.intrinsic_lines = true;
        }
        self.elements.push(FormatElement::text(text));
    }

    /// Splits source lines with explicit literal lines, expanding enclosing groups.
    /// Source CR, LF, and CRLF become the selected output line ending, and each
    /// line resets the layout column without structural indentation.
    pub fn literal_text<'text>(&mut self, text: impl Into<Cow<'text, str>>)
    where
        'text: 'a,
    {
        let mut text = match text.into() {
            Cow::Borrowed(text) => text,
            Cow::Owned(text) => self.allocator.alloc_str(&text),
        };
        while let Some(index) = memchr::memchr2(b'\r', b'\n', text.as_bytes()) {
            self.text(&text[..index]);
            self.literal_line();
            let crlf =
                text.as_bytes()[index] == b'\r' && text.as_bytes().get(index + 1) == Some(&b'\n');
            text = &text[index + 1 + usize::from(crlf)..];
        }
        self.text(text);
    }

    /// Writes a hard line and requests expansion of its enclosing groups.
    pub fn hard_line(&mut self) {
        self.intrinsic_lines = true;
        self.elements.push(FormatElement::HardLine);
        self.effect = self.effect.join(BreakEffect::REQUIRED);
    }

    /// Writes a hard line without requesting expansion of its enclosing groups.
    pub fn hard_line_without_break_parent(&mut self) {
        self.intrinsic_lines = true;
        self.elements
            .push(FormatElement::HardLineWithoutBreakParent);
    }

    /// Preserves trailing whitespace, resets indentation, and expands enclosing groups.
    pub fn literal_line(&mut self) {
        self.intrinsic_lines = true;
        self.elements.push(FormatElement::LiteralLine);
        self.effect = self.effect.join(BreakEffect::REQUIRED);
    }

    /// Preserves trailing whitespace and resets indentation without expanding parents.
    pub fn literal_line_without_break_parent(&mut self) {
        self.intrinsic_lines = true;
        self.elements
            .push(FormatElement::LiteralLineWithoutBreakParent);
    }

    /// Preserves source whitespace while returning to the current indentation.
    /// This is the standard Doc composition `markAsRoot(literalline)`.
    pub fn literal_line_with_indent(&mut self) {
        self.intrinsic_lines = true;
        self.elements.push(FormatElement::LiteralLineWithIndent);
        self.effect = self.effect.join(BreakEffect::REQUIRED);
    }

    /// Writes one literal separator; the syntax owner decides whether it is needed.
    pub fn space(&mut self) {
        self.elements.push(FormatElement::text(Text::new(" ")));
    }

    pub fn soft_line(&mut self) {
        self.elements.push(FormatElement::SoftLine);
    }

    pub fn soft_line_if_break(&mut self, group: GroupId) {
        self.conditional_lines = true;
        self.elements.push(FormatElement::SoftLineIfBreak(group));
    }

    pub fn line(&mut self) {
        self.elements.push(FormatElement::Line);
    }

    pub fn if_break(&mut self, text: &'a str) {
        self.if_break_for(ConditionTarget::Current, text);
    }

    pub fn if_flat(&mut self, text: &'a str) {
        self.if_flat_for(ConditionTarget::Current, text);
    }

    pub fn if_break_for(&mut self, target: ConditionTarget, text: &'a str) {
        if !text.is_empty() {
            match target {
                ConditionTarget::Current => {
                    let text = Text::new(text);
                    self.intrinsic_lines |= !text.is_single_line();
                    self.elements.push(FormatElement::if_break(text));
                }
                ConditionTarget::Group(_) => {
                    let broken = self.text_fragment(text);
                    let flat = self.text_fragment("");
                    self.if_break_doc(target, broken, flat);
                }
            }
        }
    }

    pub fn if_flat_for(&mut self, target: ConditionTarget, text: &'a str) {
        if !text.is_empty() {
            match target {
                ConditionTarget::Current => {
                    let text = Text::new(text);
                    self.intrinsic_lines |= !text.is_single_line();
                    self.elements.push(FormatElement::if_flat(text));
                }
                ConditionTarget::Group(_) => {
                    let broken = self.text_fragment("");
                    let flat = self.text_fragment(text);
                    self.if_break_doc(target, broken, flat);
                }
            }
        }
    }

    fn text_fragment(&self, text: &'a str) -> Fragment<'a> {
        let text = Text::new(text);
        let elements: &[FormatElement<'a>] = if text.source().is_empty() {
            &[]
        } else {
            self.allocator.alloc([FormatElement::text(text)])
        };
        Fragment::new(
            elements,
            BreakEffect::default(),
            FragmentCapabilities::classify(elements),
            false,
            !text.is_single_line(),
            false,
        )
    }

    /// Both branches contribute break effects, including the unselected branch.
    pub fn if_break_doc(
        &mut self,
        target: ConditionTarget,
        broken: Fragment<'a>,
        flat: Fragment<'a>,
    ) {
        self.group_states |= broken.group_states() || flat.group_states();
        self.effect = self.effect.join(broken.effect()).join(flat.effect());
        self.conditional_lines |= broken.conditional_lines() || flat.conditional_lines();
        self.intrinsic_lines |= broken.has_intrinsic_lines() || flat.has_intrinsic_lines();
        let conditional = self.allocator.alloc(Conditional {
            target,
            broken,
            flat,
        });
        self.elements.push(FormatElement::Conditional(conditional));
    }

    pub fn line_suffix<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.intrinsic_lines = true;
        let start = self.elements.len();
        self.elements
            .push(FormatElement::StartLineSuffix { end_offset: 0 });
        let result = write(self);
        let end_offset = instruction_offset(self.elements.len() - start);
        self.elements[start] = FormatElement::StartLineSuffix { end_offset };
        self.elements.push(FormatElement::EndLineSuffix);
        result
    }

    pub fn line_suffix_boundary(&mut self) {
        self.elements.push(FormatElement::LineSuffixBoundary);
    }

    pub fn break_parent(&mut self) {
        self.elements.push(FormatElement::BreakParent);
        self.effect = self.effect.join(BreakEffect::REQUIRED);
    }

    pub fn group<T>(&mut self, expand: bool, write: impl FnOnce(&mut Self) -> T) -> T {
        self.group_with_request(GroupRequest::from(expand), write)
    }

    pub fn group_with_request<T>(
        &mut self,
        request: GroupRequest,
        write: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.group_with_id(request, None, write)
    }

    /// Retains exact flat measurements and the boundary of a complete group.
    /// Unsupported instructions keep the ordinary Auto group unchanged.
    pub fn group_with_flat_width<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        let start = self.elements.len();
        let result = self.group(false, write);
        let FormatElement::StartGroup { effect, .. } = self.elements[start] else {
            unreachable!("group scope retains its opening instruction");
        };
        if !effect.preferred_breaks()
            && let Some(summary) =
                super::flat_summary(&self.elements[start + 1..self.elements.len() - 1])
        {
            self.elements[start] = FormatElement::StartGroupWithFlatWidth {
                summary,
                end_offset: instruction_offset(self.elements.len() - start - 1),
            };
        }
        result
    }

    /// The returned handle refers to a group already written in this document.
    pub fn named_group(&mut self, expand: bool, write: impl FnOnce(&mut Self)) -> GroupId {
        self.named_group_with_request(GroupRequest::from(expand), write)
    }

    pub fn named_group_with_request(
        &mut self,
        request: GroupRequest,
        write: impl FnOnce(&mut Self),
    ) -> GroupId {
        let id = self.next_group_id();
        self.group_with_id(request, Some(id), write);
        id
    }

    /// Reserves an identity before recording contents shared by layout alternatives.
    pub fn next_group_id(&mut self) -> GroupId {
        self.group_count = self
            .group_count
            .checked_add(1)
            .expect("document group count exceeds u32::MAX");
        GroupId(super::NonZeroU32::new(self.group_count).expect("group identity is nonzero"))
    }

    pub fn group_with_id<T>(
        &mut self,
        request: GroupRequest,
        id: Option<GroupId>,
        write: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let start = self.elements.len();
        let outer_effect = self.effect;
        self.effect = BreakEffect::default();
        self.elements.push(FormatElement::StartGroup {
            request,
            effect: self.effect,
            id,
            end_offset: 0,
        });
        let result = write(self);
        let effect = BreakEffect::from_request(request).join(self.effect);
        self.effect = outer_effect.join(effect);
        let distance = instruction_offset(self.elements.len() - start);
        let FormatElement::StartGroup {
            effect: group_effect,
            end_offset,
            ..
        } = &mut self.elements[start]
        else {
            unreachable!("group scope retains its opening instruction");
        };
        *group_effect = effect;
        *end_offset = distance;
        self.elements.push(FormatElement::EndGroup);
        result
    }

    /// Removes optional layout breaks in one complete scope. Required source
    /// breaks and line suffixes keep the original document unchanged.
    pub fn flat<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        if self.flattening {
            return write(self);
        }
        let (fragment, result) = self
            .record_with_value(|writer| {
                writer.flattening = true;
                Ok::<_, std::convert::Infallible>(write(writer))
            })
            .unwrap_or_else(|never| match never {});
        if let Some(flat) = fragment.try_flat() {
            self.write_flat_fragment(flat);
        } else {
            self.write_fragment(fragment);
        }
        result
    }

    pub fn indent<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.elements.push(FormatElement::StartIndent);
        let result = write(self);
        self.elements.push(FormatElement::EndIndent);
        result
    }

    /// Gives a complete scope its own root-relative indentation.
    pub fn absolute_indent<T>(&mut self, width: usize, write: impl FnOnce(&mut Self) -> T) -> T {
        let width = indentation_width(width);
        self.elements
            .push(FormatElement::StartAbsoluteIndent { width });
        let result = write(self);
        self.elements.push(FormatElement::EndIndent);
        result
    }

    pub fn indent_if_break<T>(&mut self, group: GroupId, write: impl FnOnce(&mut Self) -> T) -> T {
        let start = self.elements.len();
        self.elements.push(FormatElement::StartIndentIfBreak {
            id: group,
            end_offset: 0,
        });
        let result = write(self);
        let end_offset = instruction_offset(self.elements.len() - start);
        self.elements[start] = FormatElement::StartIndentIfBreak {
            id: group,
            end_offset,
        };
        self.elements.push(FormatElement::EndIndent);
        result
    }

    pub fn align<T>(&mut self, width: usize, write: impl FnOnce(&mut Self) -> T) -> T {
        if width == 0 {
            return write(self);
        }
        let width = indentation_width(width);
        self.elements.push(FormatElement::StartAlign { width });
        let result = write(self);
        self.elements.push(FormatElement::EndAlign { width });
        result
    }

    pub fn fill<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.fill_with_separator_policy(|writer| (write(writer), FillSeparatorPolicy::Preserve))
    }

    /// Builds a Fill and applies the caller's separator policy to that same scope.
    /// Content and separator entries alternate; only separators containing exactly
    /// two hard lines are collapsed. Required parent breaks are retained.
    pub fn fill_with_separator_policy<T>(
        &mut self,
        write: impl FnOnce(&mut Self) -> (T, FillSeparatorPolicy),
    ) -> T {
        let start = self.elements.len();
        self.elements.push(FormatElement::StartFill);
        let (result, policy) = write(self);
        let end = self.elements.len();
        self.elements.push(FormatElement::EndFill);
        if policy == FillSeparatorPolicy::CollapseEmptyLines {
            self.collapse_empty_fill_separators(start + 1, end);
        }
        result
    }

    fn collapse_empty_fill_separators(&mut self, mut index: usize, scope_end: usize) {
        let mut separator = false;
        while index < scope_end {
            let FormatElement::StartEntry { end_offset } = self.elements[index] else {
                unreachable!("fill cleanup visits complete owned entries");
            };
            let end = index + end_offset as usize;
            debug_assert!(
                end < scope_end,
                "fill entry stays inside its completed scope"
            );
            if separator
                && matches!(
                    self.elements[index + 1..end],
                    [FormatElement::HardLine, FormatElement::HardLine]
                )
            {
                self.elements[end - 1] = FormatElement::BreakParent;
            }
            separator = !separator;
            index = end + 1;
        }
    }

    /// Fill entries alternate between content and its following separator.
    pub fn fill_entry<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.fill_entry_with_handle(write).1
    }

    pub fn fill_entry_with_handle<T>(
        &mut self,
        write: impl FnOnce(&mut Self) -> T,
    ) -> (FillEntry, T) {
        let start = self.elements.len();
        self.elements
            .push(FormatElement::StartEntry { end_offset: 0 });
        let result = write(self);
        let end_offset = instruction_offset(self.elements.len() - start);
        self.elements[start] = FormatElement::StartEntry { end_offset };
        self.elements.push(FormatElement::EndEntry);
        let end = self.elements.len();
        (FillEntry { start, end }, result)
    }

    pub fn extend_fill_entry(&mut self, entry: FillEntry, write: impl FnOnce(&mut Self)) {
        assert_eq!(
            self.elements.len(),
            entry.end,
            "a fill suffix must extend the writer's final completed entry"
        );
        self.elements.pop();
        write(self);
        let end_offset = instruction_offset(self.elements.len() - entry.start);
        self.elements[entry.start] = FormatElement::StartEntry { end_offset };
        self.elements.push(FormatElement::EndEntry);
    }

    /// Writes once and measures only appended instructions; the slice never escapes.
    pub fn write_with_clean_text_width<E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        let start = self.elements.len();
        write(self)?;
        Ok(super::clean_text_width(&self.elements[start..]))
    }

    /// Observes this construction scope without scanning its instruction stream.
    pub fn write_with_intrinsic_lines<T, E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<(T, bool), E> {
        let outer = self.intrinsic_lines;
        self.intrinsic_lines = false;
        let result = write(self);
        let intrinsic_lines = self.intrinsic_lines;
        self.intrinsic_lines |= outer;
        result.map(|value| (value, intrinsic_lines))
    }

    /// Syntax is consumed once; only completed documents are retained.
    pub fn record_with_value<T, E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<(Fragment<'a>, T), E> {
        let mut child = Self::new(self.allocator);
        child.group_count = self.group_count;
        let value = write(&mut child)?;
        self.group_count = child.group_count;
        let capabilities = if child.effect.preferred_breaks() || child.conditional_lines {
            FragmentCapabilities::from_flat(
                child.elements.iter().all(|element| element.permits_flat()),
            )
        } else {
            FragmentCapabilities::classify(&child.elements)
        };
        let fragment = Fragment::new(
            child.elements.into_bump_slice(),
            child.effect,
            capabilities,
            child.conditional_lines,
            child.intrinsic_lines,
            child.group_states,
        );
        Ok((fragment, value))
    }

    pub fn record_group_with_value<T, E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<(RecordedGroup<'a>, T), E> {
        let (grouped, value) =
            self.record_with_value(|writer| writer.group_with_request(GroupRequest::Auto, write))?;
        let [
            FormatElement::StartGroup { id: None, .. },
            ..,
            FormatElement::EndGroup,
        ] = grouped.elements()
        else {
            unreachable!("recorded group retains its complete anonymous outer scope");
        };
        Ok((RecordedGroup { grouped }, value))
    }

    /// Wraps a completed scope without recording its syntax or allocating a named group.
    pub fn group_fragment(&self, expand: bool, fragment: Fragment<'a>) -> Fragment<'a> {
        self.group_fragment_with_request(GroupRequest::from(expand), fragment)
    }

    pub fn group_fragment_with_request(
        &self,
        request: GroupRequest,
        fragment: Fragment<'a>,
    ) -> Fragment<'a> {
        self.group_fragment_with_id(request, None, fragment)
    }

    pub fn group_fragment_with_id(
        &self,
        request: GroupRequest,
        id: Option<GroupId>,
        fragment: Fragment<'a>,
    ) -> Fragment<'a> {
        let effect = BreakEffect::from_request(request).join(fragment.effect());
        let capabilities = if id.is_some() {
            FragmentCapabilities::General
        } else if effect.preferred_breaks() || fragment.conditional_lines() {
            FragmentCapabilities::from_flat(fragment.capabilities().permits_flat())
        } else {
            fragment.capabilities()
        };
        let elements = self.allocator.alloc([
            FormatElement::StartGroup {
                request,
                effect,
                id,
                end_offset: 2,
            },
            FormatElement::fragment(fragment),
            FormatElement::EndGroup,
        ]);
        Fragment::new(
            elements,
            effect,
            capabilities,
            fragment.conditional_lines(),
            fragment.has_intrinsic_lines(),
            fragment.group_states(),
        )
    }

    /// Writes a group with independent contents and at least one expanded state.
    pub fn write_group_with_states(
        &mut self,
        request: GroupRequest,
        id: Option<GroupId>,
        contents: Fragment<'a>,
        first: Fragment<'a>,
        rest: &[Fragment<'a>],
    ) {
        let mut states = ArenaVec::with_capacity_in(rest.len() + 1, self.allocator);
        states.push(first);
        states.extend_from_slice(rest);
        self.group_states = true;
        self.effect = self.effect.join(BreakEffect::from_request(request));
        self.conditional_lines |= contents.conditional_lines()
            || first.conditional_lines()
            || rest.iter().any(|state| state.conditional_lines());
        self.intrinsic_lines |= contents.has_intrinsic_lines();
        let group = self.allocator.alloc(GroupWithStates {
            contents,
            states: states.into_bump_slice(),
            request,
            id,
        });
        self.elements.push(FormatElement::GroupWithStates(group));
    }

    pub fn write_fragment(&mut self, fragment: Fragment<'a>) {
        self.group_states |= fragment.group_states();
        self.effect = self.effect.join(fragment.effect());
        self.conditional_lines |= fragment.conditional_lines();
        self.intrinsic_lines |= fragment.has_intrinsic_lines();
        self.elements.push(FormatElement::fragment(fragment));
    }

    pub fn write_flat_fragment(&mut self, fragment: FlatFragment<'a>) {
        self.group_states |= fragment.0.group_states();
        self.elements.push(FormatElement::flat_fragment(fragment));
    }

    pub fn write_neutral_fragment(&mut self, NeutralFragment(fragment): NeutralFragment<'a>) {
        self.group_states |= fragment.group_states();
        self.effect = self.effect.join(fragment.effect());
        self.conditional_lines |= fragment.conditional_lines();
        self.elements
            .push(FormatElement::flat_fragment(FlatFragment(fragment)));
    }

    /// Writes a standard conditional group. Branch-local break requests do not
    /// force an enclosing group; the selected branch still retains its own groups.
    pub fn write_alternatives(&mut self, preferred: &[Fragment<'a>], fallback: Fragment<'a>) {
        self.group_states |=
            fallback.group_states() || preferred.iter().any(|fragment| fragment.group_states());
        let mut choices = ArenaVec::with_capacity_in(preferred.len().max(1), self.allocator);
        if preferred.is_empty() {
            choices.push(fallback);
        } else {
            choices.extend_from_slice(preferred);
        }
        self.conditional_lines |= fallback.conditional_lines()
            || preferred
                .iter()
                .any(|fragment| fragment.conditional_lines());
        let alternatives = self.allocator.alloc(Alternatives {
            preferred: choices.into_bump_slice(),
            fallback,
        });
        // Only the first candidate participates in the intrinsic-line summary. Later
        // choices can introduce layout lines without making them intrinsic.
        self.intrinsic_lines |= alternatives.first().has_intrinsic_lines();
        self.elements
            .push(FormatElement::Alternatives(alternatives));
    }

    pub fn finish(self) -> Doc<'a> {
        Doc {
            allocator: self.allocator,
            elements: ThinSlice::new(self.elements.into_bump_slice()).get(),
            group_count: self.group_count,
            group_states: self.group_states,
        }
    }
}
