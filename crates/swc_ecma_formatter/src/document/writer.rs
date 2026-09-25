//! Direct instruction construction with stable borrowed and generated text.

use std::borrow::Cow;

use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};

use super::{
    Alternatives, BreakEffect, CanonicalCandidate, Doc, FlatFragment, FormatElement, Fragment,
    FragmentCapabilities, GroupId, GroupRequest, Measurement, NeutralFragment, RecordedGroup,
    SpaceKind, text::Text,
};

pub(crate) struct Writer<'a> {
    allocator: &'a Allocator,
    elements: ArenaVec<'a, FormatElement<'a>>,
    effect: BreakEffect,
    flattening: bool,
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
            flattening: false,
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

    /// Removes optional layout breaks in one complete scope. Required source
    /// breaks and line suffixes keep the original document unchanged.
    pub(crate) fn flat<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        if self.flattening {
            return write(self);
        }
        let start = self.elements.len();
        let effect = self.effect;
        let conditional_lines = self.conditional_lines;
        self.flattening = true;
        let result = write(self);
        self.flattening = false;
        if self.elements[start..]
            .iter()
            .any(|element| !element.permits_flat())
        {
            return result;
        }
        let mut output = start;
        for index in start..self.elements.len() {
            let Some(element) = self.elements[index].flattened() else {
                continue;
            };
            self.elements[output] = element;
            output += 1;
        }
        self.elements.truncate(output);
        self.effect = effect;
        self.conditional_lines = conditional_lines;
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

    /// Syntax is consumed once; only completed documents are retained.
    pub(crate) fn record_with_value<T, E>(
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
        let fragment = Fragment {
            elements: child.elements.into_bump_slice(),
            effect: child.effect,
            capabilities,
            conditional_lines: child.conditional_lines,
        };
        Ok((fragment, value))
    }

    pub(crate) fn record_group_with_value<T, E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<(RecordedGroup<'a>, T), E> {
        let (grouped, value) =
            self.record_with_value(|writer| writer.group_with_request(GroupRequest::Auto, write))?;
        let [
            FormatElement::StartGroup { id: None, .. },
            ..,
            FormatElement::EndGroup,
        ] = grouped.elements
        else {
            unreachable!("recorded group retains its complete anonymous outer scope");
        };
        Ok((RecordedGroup { grouped }, value))
    }

    /// Wraps a completed scope without recording its syntax or allocating a named group.
    pub(crate) fn group_fragment(&self, expand: bool, fragment: Fragment<'a>) -> Fragment<'a> {
        self.group_fragment_with_request(GroupRequest::from(expand), fragment)
    }

    pub(crate) fn group_fragment_with_request(
        &self,
        request: GroupRequest,
        fragment: Fragment<'a>,
    ) -> Fragment<'a> {
        let effect = BreakEffect::from_request(request).join(fragment.effect);
        let capabilities = if effect.preferred_breaks() || fragment.conditional_lines {
            FragmentCapabilities::from_flat(fragment.capabilities.permits_flat())
        } else {
            fragment.capabilities
        };
        let elements = self.allocator.alloc([
            FormatElement::StartGroup { effect, id: None },
            FormatElement::Fragment(fragment),
            FormatElement::EndGroup,
        ]);
        Fragment {
            elements,
            effect,
            capabilities,
            conditional_lines: fragment.conditional_lines,
        }
    }

    /// Joins complete scopes with the same optional deferred separator as `space`.
    pub(crate) fn join_fragments(
        &self,
        left: Fragment<'a>,
        separate: bool,
        right: Fragment<'a>,
    ) -> Fragment<'a> {
        let effect = left.effect.join(right.effect);
        let conditional_lines = left.conditional_lines || right.conditional_lines;
        let capabilities = match (left.capabilities, right.capabilities) {
            (FragmentCapabilities::General, _) | (_, FragmentCapabilities::General) => {
                FragmentCapabilities::General
            }
            (FragmentCapabilities::Neutral, FragmentCapabilities::Neutral)
                if !effect.preferred_breaks() && !conditional_lines =>
            {
                FragmentCapabilities::Neutral
            }
            _ => FragmentCapabilities::FlatEligible,
        };
        let elements: &'a [FormatElement<'a>] = if separate {
            self.allocator.alloc([
                FormatElement::Fragment(left),
                FormatElement::Space(SpaceKind::Deferred),
                FormatElement::Fragment(right),
            ])
        } else {
            self.allocator.alloc([
                FormatElement::Fragment(left),
                FormatElement::Fragment(right),
            ])
        };
        Fragment {
            elements,
            effect,
            capabilities,
            conditional_lines,
        }
    }

    /// Delimiters observe one complete group; the prefix keeps its own outer scope.
    pub(crate) fn conditional_group_fragment(
        &self,
        prefix: Fragment<'a>,
        request: GroupRequest,
        open: &'a str,
        body: Fragment<'a>,
        close: &'a str,
    ) -> Fragment<'a> {
        let group_effect = BreakEffect::from_request(request).join(body.effect);
        let effect = prefix.effect.join(group_effect);
        let conditional_lines = prefix.conditional_lines || body.conditional_lines;
        // Conditional delimiters disappear in a flat projection, and both
        // complete backing scopes retain their own projection eligibility.
        let capabilities = FragmentCapabilities::from_flat(
            prefix.capabilities.permits_flat() && body.capabilities.permits_flat(),
        );
        let elements = self.allocator.alloc([
            FormatElement::Fragment(prefix),
            FormatElement::StartGroup {
                effect: group_effect,
                id: None,
            },
            FormatElement::IfBreak(Text::new(open)),
            FormatElement::StartIndent,
            FormatElement::SoftLine,
            FormatElement::Fragment(body),
            FormatElement::EndIndent,
            FormatElement::SoftLine,
            FormatElement::IfBreak(Text::new(close)),
            FormatElement::EndGroup,
        ]);
        Fragment {
            elements,
            effect,
            capabilities,
            conditional_lines,
        }
    }

    /// Flat views project their backing scopes without propagating backing break flags.
    pub(crate) fn flat_delimited_fragment(
        &self,
        open: &'a str,
        prefix: FlatFragment<'a>,
        body: Fragment<'a>,
        suffix: FlatFragment<'a>,
        close: &'a str,
    ) -> Fragment<'a> {
        let open = Text::new(open);
        let close = Text::new(close);
        let capabilities = if !body.capabilities.permits_flat()
            || !open.is_single_line()
            || !close.is_single_line()
        {
            FragmentCapabilities::General
        } else if !body.effect.preferred_breaks()
            && !body.conditional_lines
            && matches!(prefix.0.capabilities, FragmentCapabilities::Neutral)
            && matches!(body.capabilities, FragmentCapabilities::Neutral)
            && matches!(suffix.0.capabilities, FragmentCapabilities::Neutral)
        {
            FragmentCapabilities::Neutral
        } else {
            FragmentCapabilities::FlatEligible
        };
        let elements = self.allocator.alloc([
            FormatElement::Text(open),
            FormatElement::FlatFragment(prefix),
            FormatElement::Fragment(body),
            FormatElement::FlatFragment(suffix),
            FormatElement::Text(close),
        ]);
        Fragment {
            elements,
            effect: body.effect,
            capabilities,
            conditional_lines: body.conditional_lines,
        }
    }

    /// Only prefix and body gain the outer indent; suffix retains its own complete scopes.
    pub(crate) fn indented_delimited_fragment(
        &self,
        open: &'a str,
        prefix: Fragment<'a>,
        body: Fragment<'a>,
        suffix: Fragment<'a>,
        close: &'a str,
    ) -> Fragment<'a> {
        let open = Text::new(open);
        let close = Text::new(close);
        let effect = prefix.effect.join(body.effect).join(suffix.effect);
        let conditional_lines =
            prefix.conditional_lines || body.conditional_lines || suffix.conditional_lines;
        let capabilities = FragmentCapabilities::from_flat(
            prefix.capabilities.permits_flat()
                && body.capabilities.permits_flat()
                && suffix.capabilities.permits_flat()
                && open.is_single_line()
                && close.is_single_line(),
        );
        let elements = self.allocator.alloc([
            FormatElement::StartGroup { effect, id: None },
            FormatElement::Text(open),
            FormatElement::StartIndent,
            FormatElement::Fragment(prefix),
            FormatElement::Fragment(body),
            FormatElement::EndIndent,
            FormatElement::Fragment(suffix),
            FormatElement::Text(close),
            FormatElement::EndGroup,
        ]);
        Fragment {
            elements,
            effect,
            capabilities,
            conditional_lines,
        }
    }

    pub(crate) fn write_fragment(&mut self, fragment: Fragment<'a>) {
        self.effect = self.effect.join(fragment.effect);
        self.conditional_lines |= fragment.conditional_lines;
        self.elements.push(FormatElement::Fragment(fragment));
    }

    pub(crate) fn write_flat_fragment(&mut self, fragment: FlatFragment<'a>) {
        self.elements.push(FormatElement::FlatFragment(fragment));
    }

    pub(crate) fn write_neutral_fragment(
        &mut self,
        NeutralFragment(fragment): NeutralFragment<'a>,
    ) {
        self.effect = self.effect.join(fragment.effect);
        self.conditional_lines |= fragment.conditional_lines;
        self.elements
            .push(FormatElement::FlatFragment(FlatFragment(fragment)));
    }

    pub(crate) fn write_alternatives(
        &mut self,
        normal: Fragment<'a>,
        preferred: &[Fragment<'a>],
        fallback: Fragment<'a>,
    ) {
        if preferred.is_empty() {
            self.write_fragment(fallback);
            return;
        }
        let mut choices = ArenaVec::with_capacity_in(preferred.len(), self.allocator);
        choices.extend_from_slice(preferred);
        self.write_recorded_alternatives(normal, choices, fallback, Measurement::Display);
    }

    pub(crate) fn write_canonical_alternatives(
        &mut self,
        normal: Fragment<'a>,
        preferred: &[CanonicalCandidate<'a>],
        fallback: Fragment<'a>,
    ) {
        if preferred.is_empty() {
            self.write_fragment(fallback);
            return;
        }
        let mut choices = ArenaVec::with_capacity_in(preferred.len(), self.allocator);
        choices.extend(preferred.iter().map(|candidate| candidate.0));
        self.write_recorded_alternatives(normal, choices, fallback, Measurement::Canonical);
    }

    fn write_recorded_alternatives(
        &mut self,
        normal: Fragment<'a>,
        preferred: ArenaVec<'a, Fragment<'a>>,
        fallback: Fragment<'a>,
        measurement: Measurement,
    ) {
        // The normal layout owns outward expansion. Optional display variants
        // retain their own groups without expanding the parent.
        self.effect = self.effect.join(normal.effect);
        self.conditional_lines |= normal.conditional_lines
            || fallback.conditional_lines
            || preferred.iter().any(|fragment| fragment.conditional_lines);
        let alternatives = self.allocator.alloc(Alternatives {
            preferred: preferred.into_bump_slice(),
            fallback,
        });
        let element = match measurement {
            Measurement::Display => FormatElement::Alternatives(alternatives),
            Measurement::Canonical => FormatElement::CanonicalAlternatives(alternatives),
        };
        self.elements.push(element);
    }

    pub(crate) fn finish(self) -> Doc<'a> {
        Doc {
            elements: self.elements.into_bump_slice(),
            group_count: self.group_count,
            conditional_lines: self.conditional_lines,
        }
    }
}
