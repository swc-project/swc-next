//! Direct instruction construction with stable borrowed and generated text.

use std::borrow::Cow;

use swc_next_allocator::{Allocator, vec::ArenaVec};

use super::{
    AlignText, BreakEffect, ConditionTarget, Doc, FlatFragment, Fragment, FragmentCapabilities,
    Group, GroupId, GroupRequest, IfBreak, IndentIfBreakOptions, Instruction, LabelId, LineOptions,
    NeutralFragment, RecordedGroup, RichIndentKind, facts::Facts, text::Text, transform,
};

pub struct Writer<'a> {
    allocator: &'a Allocator,
    elements: ArenaVec<'a, Instruction<'a>>,
    facts: Facts,
    flattening: bool,
    group_count: u32,
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
            facts: Facts::default(),
            flattening: false,
            group_count: 0,
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
        self.facts.nonempty_text(text);
        self.elements.push(Instruction::text(text));
    }

    /// Copies transient text into storage owned by this formatting call.
    pub fn text_copied(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let text = self.allocator.alloc_str(text);
        let text = Text::new(text);
        self.facts.nonempty_text(text);
        self.elements.push(Instruction::text(text));
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
        self.facts.hard_line(true);
        self.elements.push(Instruction::HardLine);
    }

    /// Writes a hard line without requesting expansion of its enclosing groups.
    pub fn hard_line_without_break_parent(&mut self) {
        self.facts.hard_line(false);
        self.elements.push(Instruction::HardLineOnly);
    }

    /// Preserves trailing whitespace, resets indentation, and expands enclosing groups.
    pub fn literal_line(&mut self) {
        self.facts.hard_line(true);
        self.elements.push(Instruction::LiteralLine);
    }

    /// Preserves trailing whitespace and resets indentation without expanding parents.
    pub fn literal_line_without_break_parent(&mut self) {
        self.facts.hard_line(false);
        self.elements.push(Instruction::LiteralLineOnly);
    }

    /// Preserves source whitespace while returning to the current indentation.
    /// This is the standard Doc composition `markAsRoot(literalline)`.
    pub fn literal_line_with_indent(&mut self) {
        self.facts.hard_line(true);
        self.elements.push(Instruction::LiteralLineAtCurrentIndent);
    }

    /// Writes one literal separator; the syntax owner decides whether it is needed.
    pub fn space(&mut self) {
        self.elements.push(Instruction::text(Text::new(" ")));
    }

    pub fn soft_line(&mut self) {
        self.elements.push(Instruction::SoftLine);
    }

    pub fn soft_line_if_break(&mut self, group: GroupId) {
        self.facts.conditional_line();
        self.elements
            .push(Instruction::SoftLineIfGroupBreaks(group));
    }

    pub fn line(&mut self) {
        self.elements.push(Instruction::NormalLine);
    }

    /// Writes one raw Prettier `line` Doc without an added `breakParent`.
    /// Use [`Self::hard_line`] for Prettier's complete `hardline` composition.
    pub fn raw_line(&mut self, options: LineOptions) {
        let element = Instruction::raw_line(options);
        self.facts.observe(element);
        self.elements.push(element);
    }

    /// Removes the spaces and tabs trailing the output at this point.
    pub fn trim(&mut self) {
        self.facts.require_extended_print();
        self.elements.push(Instruction::Trim);
    }

    /// Marks a position for printing with cursor tracking.
    pub fn cursor(&mut self) {
        self.elements.push(Instruction::Cursor);
    }

    /// Associates caller-owned metadata with a scope without affecting layout.
    pub fn label<T>(&mut self, id: LabelId, write: impl FnOnce(&mut Self) -> T) -> T {
        self.elements.push(Instruction::StartLabel(id));
        let result = write(self);
        self.elements.push(Instruction::EndLabel);
        result
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
                    self.facts.nonempty_text(text);
                    self.elements.push(Instruction::text_if_break(text));
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
                    self.facts.nonempty_text(text);
                    self.elements.push(Instruction::text_if_flat(text));
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
        let mut facts = Facts::default();
        let elements: &[Instruction<'a>] = if text.source().is_empty() {
            &[]
        } else {
            facts.nonempty_text(text);
            self.allocator.alloc([Instruction::text(text)])
        };
        Fragment::new(elements, facts.finish(elements))
    }

    /// Both branches contribute break effects, including the unselected branch.
    pub fn if_break_doc(
        &mut self,
        target: ConditionTarget,
        broken: Fragment<'a>,
        flat: Fragment<'a>,
    ) {
        let conditional = self.allocator.alloc(IfBreak {
            target,
            broken,
            flat,
        });
        self.facts.if_break(conditional, false);
        self.elements.push(Instruction::IfBreak(conditional));
    }

    /// Defers content until the next line break, suffix boundary, or end of the Doc.
    /// Even empty content registers a pending suffix and affects boundaries.
    pub fn line_suffix<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        let start = self.elements.len();
        self.facts.line_suffix();
        self.elements
            .push(Instruction::StartLineSuffix { end_offset: 0 });
        let result = write(self);
        let end_offset = instruction_offset(self.elements.len() - start);
        self.elements[start] = Instruction::StartLineSuffix { end_offset };
        self.elements.push(Instruction::EndLineSuffix);
        result
    }

    pub fn line_suffix_boundary(&mut self) {
        self.elements.push(Instruction::LineSuffixBoundary);
    }

    pub fn break_parent(&mut self) {
        self.facts.require_break();
        self.elements.push(Instruction::BreakParent);
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
        let Instruction::StartGroup { effect, .. } = self.elements[start] else {
            unreachable!("group scope retains its opening instruction");
        };
        if !effect.preferred_breaks()
            && let Some(summary) =
                super::flat_summary(&self.elements[start + 1..self.elements.len() - 1])
        {
            self.elements[start] = Instruction::StartGroupWithFlatWidth {
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
        let outer_effect = self.facts.begin_group();
        self.elements.push(Instruction::StartGroup {
            request,
            effect: self.facts.effect(),
            id,
            end_offset: 0,
        });
        let result = write(self);
        let effect = self.facts.end_group(request, outer_effect);
        let distance = instruction_offset(self.elements.len() - start);
        let Instruction::StartGroup {
            effect: group_effect,
            end_offset,
            ..
        } = &mut self.elements[start]
        else {
            unreachable!("group scope retains its opening instruction");
        };
        *group_effect = effect;
        *end_offset = distance;
        self.elements.push(Instruction::EndGroup);
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
        self.elements.push(Instruction::StartIndent);
        let result = write(self);
        self.elements.push(Instruction::EndIndent);
        result
    }

    /// Gives a complete scope its own root-relative indentation.
    pub fn absolute_indent<T>(&mut self, width: usize, write: impl FnOnce(&mut Self) -> T) -> T {
        let width = indentation_width(width);
        self.elements
            .push(Instruction::StartAbsoluteIndent { width });
        let result = write(self);
        self.elements.push(Instruction::EndIndent);
        result
    }

    pub fn indent_if_break<T>(&mut self, group: GroupId, write: impl FnOnce(&mut Self) -> T) -> T {
        self.indent_if_break_with_options(
            IndentIfBreakOptions {
                target: ConditionTarget::Group(group),
                negate: false,
            },
            write,
        )
    }

    /// Indents the scope when the selected group mode matches the options.
    pub fn indent_if_break_with_options<T>(
        &mut self,
        options: IndentIfBreakOptions,
        write: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let start = self.elements.len();
        self.elements.push(Instruction::StartIndentIfBreak {
            target: options.target,
            negate: options.negate,
            end_offset: 0,
        });
        let result = write(self);
        let end_offset = instruction_offset(self.elements.len() - start);
        self.elements[start] = Instruction::StartIndentIfBreak {
            target: options.target,
            negate: options.negate,
            end_offset,
        };
        self.elements.push(Instruction::EndIndent);
        result
    }

    pub fn align<T>(&mut self, width: usize, write: impl FnOnce(&mut Self) -> T) -> T {
        if width == 0 {
            return write(self);
        }
        let width = indentation_width(width);
        self.elements.push(Instruction::StartAlign { width });
        let result = write(self);
        self.elements.push(Instruction::EndAlign { width });
        result
    }

    /// Aligns the scope with an unchanged string prefix.
    pub fn align_text<T>(&mut self, prefix: &'a str, write: impl FnOnce(&mut Self) -> T) -> T {
        if prefix.is_empty() {
            return write(self);
        }
        let text = self.allocator.alloc(AlignText {
            source: prefix,
            width: crate::measure_text_width(prefix),
        });
        self.facts.require_rich_indent();
        self.elements.push(Instruction::StartAlignText(text));
        let result = write(self);
        self.elements
            .push(Instruction::EndRichIndent(RichIndentKind::AlignText));
        result
    }

    pub fn dedent<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.facts.require_rich_indent();
        self.elements.push(Instruction::StartDedent);
        let result = write(self);
        self.elements
            .push(Instruction::EndRichIndent(RichIndentKind::Dedent));
        result
    }

    pub fn mark_as_root<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.facts.require_rich_indent();
        self.elements.push(Instruction::StartMarkRoot);
        let result = write(self);
        self.elements
            .push(Instruction::EndRichIndent(RichIndentKind::MarkRoot));
        result
    }

    pub fn dedent_to_root<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.facts.require_rich_indent();
        self.elements.push(Instruction::StartDedentToRoot);
        let result = write(self);
        self.elements
            .push(Instruction::EndRichIndent(RichIndentKind::DedentToRoot));
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
        self.elements.push(Instruction::StartFill);
        let (result, policy) = write(self);
        let end = self.elements.len();
        self.elements.push(Instruction::EndFill);
        if policy == FillSeparatorPolicy::CollapseEmptyLines {
            self.collapse_empty_fill_separators(start + 1, end);
        }
        result
    }

    fn collapse_empty_fill_separators(&mut self, mut index: usize, scope_end: usize) {
        let mut separator = false;
        while index < scope_end {
            let Instruction::StartFillPart { end_offset } = self.elements[index] else {
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
                    [Instruction::HardLine, Instruction::HardLine]
                )
            {
                self.elements[end - 1] = Instruction::BreakParent;
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
            .push(Instruction::StartFillPart { end_offset: 0 });
        let result = write(self);
        let end_offset = instruction_offset(self.elements.len() - start);
        self.elements[start] = Instruction::StartFillPart { end_offset };
        self.elements.push(Instruction::EndFillPart);
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
        self.elements[entry.start] = Instruction::StartFillPart { end_offset };
        self.elements.push(Instruction::EndFillPart);
    }

    /// Writes once and measures only appended instructions; the slice never escapes.
    pub fn write_with_clean_text_width<E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        let start = self.elements.len();
        write(self)?;
        Ok(super::text::clean_text_width(&self.elements[start..]))
    }

    /// Observes this construction scope without scanning its instruction stream.
    pub fn write_with_intrinsic_lines<T, E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<(T, bool), E> {
        let outer = self.facts.take_intrinsic_lines();
        let result = write(self);
        let intrinsic_lines = self.facts.has_intrinsic_lines();
        self.facts.include_intrinsic_lines(outer);
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
        Ok((child.finish_fragment(), value))
    }

    fn finish_fragment(self) -> Fragment<'a> {
        let summary = self.facts.finish(&self.elements);
        Fragment::new(self.elements.into_bump_slice(), summary)
    }

    pub fn record_group_with_value<T, E>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<(RecordedGroup<'a>, T), E> {
        let (grouped, value) =
            self.record_with_value(|writer| writer.group_with_request(GroupRequest::Auto, write))?;
        let [
            Instruction::StartGroup { id: None, .. },
            ..,
            Instruction::EndGroup,
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
            Instruction::StartGroup {
                request,
                effect,
                id,
                end_offset: 2,
            },
            Instruction::fragment(fragment),
            Instruction::EndGroup,
        ]);
        let mut facts = Facts::default();
        facts.fragment(fragment);
        facts.include_request(request);
        Fragment::new(elements, facts.finish_with_capabilities(capabilities))
    }

    /// Writes a group with independent contents and at least one expanded state.
    ///
    /// `contents` is the primary layout. `first` retains Prettier's original
    /// `expandedStates[0]`; when `rest` is nonempty it is not an executable
    /// candidate. The intermediate entries of `rest` are tried after `contents`,
    /// and the final state is the fallback. With no `rest`, `first` is the fallback.
    /// An inherited flat layout selects `contents`; otherwise `Expanded` selects
    /// the fallback without fitting. Use [`Self::write_alternatives`] when every
    /// supplied preferred layout should participate in candidate selection.
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
        let group = super::group::with_states(
            self.allocator,
            request,
            id,
            contents,
            states.into_bump_slice(),
        );
        self.facts.group(group);
        self.elements.push(group.element());
    }

    /// Imports an independent document as a fragment in this writer's group namespace.
    /// Each call reserves fresh named group IDs, including their conditional references.
    /// Reusing the returned fragment shares those IDs within this formatting call.
    /// Current-mode conditions and caller-owned label IDs retain their meaning.
    ///
    /// The document's arena and any borrowed source or encoded buffer must remain
    /// alive for `'a`. Shared scopes are retained while changed instructions are
    /// allocated in this writer's arena.
    pub fn import_doc(&mut self, doc: &Doc<'a>) -> Fragment<'a> {
        transform::import_doc(self.allocator, doc, &mut self.group_count)
    }

    /// Imports and appends an independent document. See [`Self::import_doc`] for
    /// group scoping and borrowed storage requirements.
    pub fn write_doc(&mut self, doc: &Doc<'a>) {
        let fragment = self.import_doc(doc);
        self.write_fragment(fragment);
    }

    /// Appends a fragment created or imported in this writer's group namespace.
    pub fn write_fragment(&mut self, fragment: Fragment<'a>) {
        self.facts.fragment(fragment);
        self.elements.push(Instruction::fragment(fragment));
    }

    pub fn write_flat_fragment(&mut self, fragment: FlatFragment<'a>) {
        self.facts.flat_fragment(fragment.0);
        self.elements.push(Instruction::flat_fragment(fragment));
    }

    pub fn write_neutral_fragment(&mut self, NeutralFragment(fragment): NeutralFragment<'a>) {
        self.facts.flat_fragment(fragment);
        self.elements
            .push(Instruction::flat_fragment(FlatFragment(fragment)));
    }

    /// Writes a standard conditional group. Branch-local break requests do not
    /// force an enclosing group; the selected branch still retains its own groups.
    pub fn write_alternatives(&mut self, preferred: &[Fragment<'a>], fallback: Fragment<'a>) {
        let mut choices = ArenaVec::with_capacity_in(preferred.len().max(1), self.allocator);
        if preferred.is_empty() {
            choices.push(fallback);
        } else {
            choices.extend_from_slice(preferred);
        }
        let group = super::group::compact(self.allocator, choices.into_bump_slice(), fallback);
        self.facts.group(group);
        self.elements.push(group.element());
    }

    pub fn finish(self) -> Doc<'a> {
        Doc {
            allocator: self.allocator,
            group_count: self.group_count,
            root: self.finish_fragment(),
        }
    }
}
