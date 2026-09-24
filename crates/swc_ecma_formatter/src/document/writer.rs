//! Direct instruction construction with stable borrowed and generated text.

use std::borrow::Cow;

use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};

use super::{BreakEffect, Doc, FormatElement, GroupRequest, text::Text};

pub(crate) struct Writer<'a> {
    allocator: &'a Allocator,
    elements: ArenaVec<'a, FormatElement<'a>>,
    effect: BreakEffect,
}

impl<'a> Writer<'a> {
    pub(crate) fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            elements: ArenaVec::new_in(allocator),
            effect: BreakEffect::default(),
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
        self.elements.push(FormatElement::Space);
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

    pub(crate) fn align<T>(&mut self, width: usize, write: impl FnOnce(&mut Self) -> T) -> T {
        if width == 0 {
            return write(self);
        }
        self.elements.push(FormatElement::StartAlign { width });
        let result = write(self);
        self.elements.push(FormatElement::EndAlign { width });
        result
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
        let start = self.elements.len();
        let outer_effect = self.effect;
        self.effect = BreakEffect::from_request(request);
        self.elements.push(FormatElement::StartGroup {
            effect: self.effect,
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

    pub(crate) fn soft_line(&mut self) {
        self.elements.push(FormatElement::SoftLine);
    }

    pub(crate) fn line(&mut self) {
        self.elements.push(FormatElement::Line);
    }

    pub(crate) fn finish(self) -> Doc<'a> {
        Doc {
            elements: self.elements.into_bump_slice(),
        }
    }
}
