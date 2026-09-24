//! Direct instruction construction with stable borrowed and generated text.

use std::borrow::Cow;

use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};

use super::{BreakEffect, Doc, FormatElement, GroupId, GroupRequest, SpaceKind, text::Text};

pub(crate) struct Writer<'a> {
    allocator: &'a Allocator,
    elements: ArenaVec<'a, FormatElement<'a>>,
    effect: BreakEffect,
    group_count: usize,
    conditional_lines: bool,
}

/// A completed entry can retain a suffix while it is still the final scope.
pub(crate) struct FillEntry {
    start: usize,
    end: usize,
}

impl<'a> Writer<'a> {
    pub(crate) fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            elements: ArenaVec::new_in(allocator),
            effect: BreakEffect::default(),
            group_count: 0,
            conditional_lines: false,
        }
    }

    pub(crate) fn text<'text>(&mut self, text: impl Into<Cow<'text, str>>)
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
        self.elements.push(FormatElement::Text(Text::new(text)));
    }

    /// Literal source lines retain their contents without structural indentation.
    pub(crate) fn literal_text<'text>(&mut self, text: impl Into<Cow<'text, str>>)
    where
        'text: 'a,
    {
        let mut text = match text.into() {
            Cow::Borrowed(text) => text,
            Cow::Owned(text) => self.allocator.alloc_str(&text),
        };
        while let Some(index) = text.find(['\r', '\n']) {
            self.text(&text[..index]);
            self.literal_line();
            let crlf =
                text.as_bytes()[index] == b'\r' && text.as_bytes().get(index + 1) == Some(&b'\n');
            text = &text[index + 1 + usize::from(crlf)..];
        }
        self.text(text);
    }

    pub(crate) fn hard_line(&mut self) {
        self.elements.push(FormatElement::HardLine);
        self.effect = self.effect.join(BreakEffect::REQUIRED);
    }

    pub(crate) fn literal_line(&mut self) {
        self.elements.push(FormatElement::LiteralLine);
        self.effect = self.effect.join(BreakEffect::REQUIRED);
    }

    pub(crate) fn space(&mut self) {
        self.elements
            .push(FormatElement::Space(SpaceKind::Deferred));
    }

    /// A canonical separator counts during fit probing and disappears at a break.
    pub(crate) fn reserved_space(&mut self) {
        self.elements
            .push(FormatElement::Space(SpaceKind::Reserved));
    }

    pub(crate) fn soft_line(&mut self) {
        self.elements.push(FormatElement::SoftLine);
    }

    pub(crate) fn soft_line_if_break(&mut self, group: GroupId) {
        self.conditional_lines = true;
        self.elements.push(FormatElement::SoftLineIfBreak(group));
    }

    pub(crate) fn line(&mut self) {
        self.elements.push(FormatElement::Line);
    }

    pub(crate) fn if_break(&mut self, text: &'a str) {
        self.elements.push(FormatElement::IfBreak(Text::new(text)));
    }

    pub(crate) fn if_flat(&mut self, text: &'a str) {
        self.elements.push(FormatElement::IfFlat(Text::new(text)));
    }

    /// Unlike group-bound conditionals, fill entries observe their local mode.
    pub(crate) fn if_current_break(&mut self, text: &'a str) {
        self.elements
            .push(FormatElement::IfCurrentBreak(Text::new(text)));
    }

    pub(crate) fn if_current_flat(&mut self, text: &'a str) {
        self.elements
            .push(FormatElement::IfCurrentFlat(Text::new(text)));
    }

    pub(crate) fn line_suffix(&mut self, text: &'a str) {
        self.elements
            .push(FormatElement::LineSuffix(Text::new(text)));
        self.break_parent();
    }

    pub(crate) fn break_parent(&mut self) {
        self.elements.push(FormatElement::BreakParent);
        self.effect = self.effect.join(BreakEffect::REQUIRED);
    }

    pub(crate) fn group<T>(&mut self, expand: bool, write: impl FnOnce(&mut Self) -> T) -> T {
        self.group_with_request(GroupRequest::from(expand), write)
    }

    pub(crate) fn group_with_request<T>(
        &mut self,
        request: GroupRequest,
        write: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.group_with_id(request, None, write)
    }

    /// The returned handle refers to a group already written in this document.
    pub(crate) fn named_group(&mut self, expand: bool, write: impl FnOnce(&mut Self)) -> GroupId {
        self.named_group_with_request(GroupRequest::from(expand), write)
    }

    pub(crate) fn named_group_with_request(
        &mut self,
        request: GroupRequest,
        write: impl FnOnce(&mut Self),
    ) -> GroupId {
        let id = GroupId(self.group_count);
        self.group_count += 1;
        self.group_with_id(request, Some(id), write);
        id
    }

    fn group_with_id<T>(
        &mut self,
        request: GroupRequest,
        id: Option<GroupId>,
        write: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let start = self.elements.len();
        let outer_effect = self.effect;
        self.effect = BreakEffect::from_request(request);
        self.elements.push(FormatElement::StartGroup {
            effect: self.effect,
            id,
        });
        let result = write(self);
        let effect = self.effect;
        self.effect = outer_effect.join(effect);
        let FormatElement::StartGroup {
            effect: group_effect,
            ..
        } = &mut self.elements[start]
        else {
            unreachable!("group scope retains its opening instruction");
        };
        *group_effect = effect;
        self.elements.push(FormatElement::EndGroup);
        result
    }

    pub(crate) fn indent<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.elements.push(FormatElement::StartIndent);
        let result = write(self);
        self.elements.push(FormatElement::EndIndent);
        result
    }

    /// Gives a complete scope its own root-relative indentation.
    pub(crate) fn absolute_indent<T>(
        &mut self,
        width: usize,
        write: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.elements
            .push(FormatElement::StartAbsoluteIndent { width });
        let result = write(self);
        self.elements.push(FormatElement::EndIndent);
        result
    }

    pub(crate) fn indent_if_break<T>(
        &mut self,
        group: GroupId,
        write: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.elements.push(FormatElement::StartIndentIfBreak(group));
        let result = write(self);
        self.elements.push(FormatElement::EndIndent);
        result
    }

    pub(crate) fn align<T>(&mut self, width: usize, write: impl FnOnce(&mut Self) -> T) -> T {
        if width == 0 {
            return write(self);
        }
        self.elements.push(FormatElement::StartAlign { width });
        let result = write(self);
        self.elements.push(FormatElement::EndAlign { width });
        result
    }

    pub(crate) fn fill<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.elements.push(FormatElement::StartFill);
        let result = write(self);
        self.elements.push(FormatElement::EndFill);
        result
    }

    /// The completed syntax scope decides whether its empty separator lines collapse.
    pub(crate) fn fill_with_empty_line_policy<T, E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<(T, bool), E>,
    ) -> Result<T, E> {
        let start = self.elements.len();
        self.elements.push(FormatElement::StartFill);
        let result = write(self);
        if matches!(result.as_ref(), Ok((_, true))) {
            let mut index = start + 1;
            let mut separator = false;
            while index < self.elements.len() {
                let FormatElement::StartEntry { end_offset } = self.elements[index] else {
                    unreachable!("fill cleanup visits complete owned entries");
                };
                let end = index + end_offset;
                if separator
                    && matches!(
                        self.elements[index + 1..end],
                        [FormatElement::HardLine, FormatElement::HardLine]
                    )
                {
                    // Keep entry distances and the required parent break stable.
                    self.elements[end - 1] = FormatElement::BreakParent;
                }
                separator = !separator;
                index = end + 1;
            }
        }
        self.elements.push(FormatElement::EndFill);
        result.map(|(value, _)| value)
    }

    /// Fill entries alternate between content and its following separator.
    pub(crate) fn fill_entry<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.fill_entry_with_handle(write).1
    }

    pub(crate) fn fill_entry_with_handle<T>(
        &mut self,
        write: impl FnOnce(&mut Self) -> T,
    ) -> (FillEntry, T) {
        let start = self.elements.len();
        self.elements
            .push(FormatElement::StartEntry { end_offset: 0 });
        let result = write(self);
        let end_offset = self.elements.len() - start;
        self.elements[start] = FormatElement::StartEntry { end_offset };
        self.elements.push(FormatElement::EndEntry);
        let end = self.elements.len();
        (FillEntry { start, end }, result)
    }

    pub(crate) fn extend_fill_entry(&mut self, entry: FillEntry, write: impl FnOnce(&mut Self)) {
        assert_eq!(
            self.elements.len(),
            entry.end,
            "a fill suffix must extend the writer's final completed entry"
        );
        self.elements.pop();
        write(self);
        let end_offset = self.elements.len() - entry.start;
        self.elements[entry.start] = FormatElement::StartEntry { end_offset };
        self.elements.push(FormatElement::EndEntry);
    }

    pub(crate) fn finish(self) -> Doc<'a> {
        Doc {
            elements: self.elements.into_bump_slice(),
            group_count: self.group_count,
            conditional_lines: self.conditional_lines,
        }
    }
}
