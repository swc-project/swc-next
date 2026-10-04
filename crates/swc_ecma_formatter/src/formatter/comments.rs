//! Bounded, source-order comment consumption and basic comment layout.

use swc_next_ecma_ast::{Comment, CommentKind, Span};

use super::Formatter;
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::source::{lines, split_line};
use crate::{FormatError, document::Writer};

/// A comment can carry more than one tool's annotation.
pub(super) struct CommentSemantics {
    pub(super) suppression: bool,
    pub(super) diagnostic: bool,
    pub(super) file_directive: bool,
    pub(super) optimizer: bool,
    pub(super) type_cast: bool,
}

impl CommentSemantics {
    pub(super) fn is_ordinary(&self) -> bool {
        !self.suppression
            && !self.diagnostic
            && !self.file_directive
            && !self.has_target_annotation()
    }

    pub(super) fn has_target_annotation(&self) -> bool {
        self.optimizer || self.type_cast
    }
}

/// A syntax owner resolves the source target before relocating comments.
/// Unknown annotation targets and diagnostic coverage retain the source boundary.
#[derive(Clone, Copy)]
pub(super) enum CommentBinding {
    Unbound,
    BoundTo(Span),
    SourceBound,
}

impl CommentBinding {
    pub(super) fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unbound, binding) | (binding, Self::Unbound) => binding,
            (Self::BoundTo(left), Self::BoundTo(right)) if left == right => Self::BoundTo(left),
            _ => Self::SourceBound,
        }
    }

    pub(super) fn combine_all(bindings: impl Iterator<Item = Self>) -> Self {
        let mut combined = Self::Unbound;
        for binding in bindings {
            combined = combined.combine(binding);
            if matches!(combined, Self::SourceBound) {
                break;
            }
        }
        combined
    }

    pub(super) fn allows_relocation(&self, target: Option<Span>) -> bool {
        match self {
            Self::Unbound => true,
            Self::BoundTo(source) => target == Some(*source),
            Self::SourceBound => false,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum CommentRunLayout {
    Source,
    SourceAfter(u32),
    Leading(u32),
    GroupedLeading(u32),
    InlineLeading(u32),
    FlatLeading(u32),
    CanonicalDangling,
    Lines,
    OwnLineTrailing,
}

/// A claimed leading run and the source gap owned by its node.
pub(super) struct NodeHead<'ast> {
    pub(super) comments: &'ast [Comment],
    pub(super) content_start: u32,
    pub(super) layout: CommentRunLayout,
    pub(super) gap_end: Option<u32>,
}

/// The cursor stays at a pending tail until its real punctuation owner claims it.
pub(super) struct ExpressionTail<'ast> {
    pub(super) source_end: u32,
    pub(super) comments: &'ast [Comment],
}

/// Source facts remain valid after the pending run has been claimed.
pub(super) struct CommentBoundary<'ast> {
    comments: &'ast [Comment],
    source_end: u32,
    gap: CommentGap,
}

impl<'ast> CommentBoundary<'ast> {
    pub(super) fn comments(&self) -> &'ast [Comment] {
        self.comments
    }

    pub(super) fn end(&self) -> u32 {
        self.comments
            .last()
            .map_or(self.source_end, |comment| comment.span.end)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    pub(super) fn gap(&self) -> CommentGap {
        self.gap
    }
}

#[derive(Clone, Copy)]
pub(super) enum CommentGap {
    LineComment,
    Source(LineConstraint),
}

impl CommentGap {
    pub(super) fn requires_line(self) -> bool {
        matches!(
            self,
            Self::LineComment | Self::Source(LineConstraint::NewLine(_))
        )
    }

    pub(super) fn write<'a>(
        self,
        writer: &mut Writer<'a>,
        flat: FlatSeparator,
        default: impl FnOnce(&mut Writer<'a>),
    ) {
        match self {
            Self::LineComment => writer.hard_line(),
            Self::Source(constraint) => constraint.write(writer, flat, default),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum LineAttachment {
    Leading,
    Trailing,
    Dangling,
}

/// Placement facts belong to the actual source run, not its eventual Doc.
#[derive(Clone, Copy)]
pub(super) struct PreparedLineFact {
    comment: Comment,
    run_start: u32,
    current_starts_line: bool,
    run_starts_line: bool,
    ordinary: bool,
}

impl PreparedLineFact {
    pub(super) fn attachment(self, has_preceding: bool, has_following: bool) -> LineAttachment {
        if has_following && (self.run_starts_line || !has_preceding) {
            LineAttachment::Leading
        } else if has_preceding {
            LineAttachment::Trailing
        } else {
            LineAttachment::Dangling
        }
    }

    fn movable(self) -> bool {
        self.ordinary && !self.current_starts_line
    }
}

struct SourceCommentRun {
    previous: Comment,
    start: u32,
    ordinary: bool,
}

/// One selected array owns this borrowed source stream through its real children.
/// A boundary can prepare a Line before the following leading owner emits it.
pub(super) struct ArrayCommentOwner<'ast> {
    remaining: &'ast [Comment],
    first_end: u32,
    last_start: u32,
    run: Option<SourceCommentRun>,
    pending: Option<PreparedLineFact>,
    reference_inline_trailing: bool,
}

impl<'ast> ArrayCommentOwner<'ast> {
    pub(super) fn new(comments: &'ast [Comment], first_end: u32, last_start: u32) -> Self {
        Self {
            remaining: comments,
            first_end,
            last_start,
            run: None,
            pending: None,
            reference_inline_trailing: false,
        }
    }

    fn advance(
        &mut self,
        formatter: &Formatter<'ast, '_>,
        comment: Comment,
    ) -> Option<PreparedLineFact> {
        let fact = formatter.prepare_source_comment(comment, &mut self.run);
        if let Some(fact) = fact {
            let has_preceding = comment.span.start >= self.first_end;
            let has_following = comment.span.end <= self.last_start;
            self.reference_inline_trailing |= !fact.current_starts_line
                && fact.attachment(has_preceding, has_following) == LineAttachment::Trailing;
        }
        fact
    }

    fn prepare(
        &mut self,
        formatter: &Formatter<'ast, '_>,
        comment: Comment,
    ) -> Option<PreparedLineFact> {
        if let Some(fact) = self.pending
            && fact.comment.span == comment.span
        {
            return Some(fact);
        }
        let mut result = None;
        while let Some((&next, rest)) = self.remaining.split_first()
            && next.span.end <= comment.span.end
        {
            self.remaining = rest;
            let fact = self.advance(formatter, next);
            if next.span == comment.span {
                result = fact;
            }
        }
        if let Some(fact) = result {
            debug_assert!(
                self.pending.is_none(),
                "an array's prepared Line must reach its real owner before a later Line",
            );
            self.pending = Some(fact);
        }
        result
    }

    fn consume(
        &mut self,
        formatter: &Formatter<'ast, '_>,
        comment: Comment,
    ) -> Option<PreparedLineFact> {
        let fact = self.prepare(formatter, comment);
        debug_assert!(
            comment.kind != CommentKind::Line || fact.is_some(),
            "an array-owned Line must reach its renderer with its prepared source fact",
        );
        if fact.is_some() {
            self.pending = None;
        }
        fact
    }

    /// Raw children and completed source owners can consume trivia without a
    /// comment renderer. Retain its source facts, without changing that policy.
    pub(super) fn finish_source(&mut self, formatter: &Formatter<'ast, '_>, end: u32) {
        if self
            .pending
            .is_some_and(|fact| fact.comment.span.end <= end)
        {
            self.pending = None;
        }
        while let Some((&comment, rest)) = self.remaining.split_first()
            && comment.span.end <= end
        {
            self.remaining = rest;
            self.advance(formatter, comment);
        }
    }

    pub(super) fn is_concise(&self) -> bool {
        !self.reference_inline_trailing
    }
}

/// One complete ordinary prefix owned by a real delimiter boundary.
/// Its final Line and preceding Blocks are emitted together, in source order.
pub(super) struct PreparedTrailingRun<'ast> {
    comments: &'ast [Comment],
    source_end: u32,
    fact: PreparedLineFact,
}

impl PreparedTrailingRun<'_> {
    pub(super) fn len(&self) -> usize {
        self.comments.len()
    }
}

struct BlockCommentPacket {
    end: u32,
    multiline: bool,
}

enum CommentText<'ast> {
    Inline(&'ast str),
    Literal {
        first: &'ast str,
        rest: &'ast str,
    },
    Aligned {
        first: &'ast str,
        rest: &'ast str,
        jsdoc: bool,
    },
}

struct PreparedComment<'ast> {
    span: Span,
    kind: CommentKind,
    text: CommentText<'ast>,
}

impl PreparedComment<'_> {
    fn is_alignable(&self) -> bool {
        matches!(self.text, CommentText::Aligned { .. })
    }
}

impl<'ast> CommentText<'ast> {
    fn write(&self, writer: &mut Writer<'ast>) {
        match *self {
            Self::Inline(text) => writer.text(text),
            Self::Literal { first, rest } => {
                writer.text(first);
                for line in lines(rest) {
                    writer.literal_line();
                    writer.text(line);
                }
            }
            Self::Aligned { first, rest, jsdoc } => {
                writer.text(first.trim_end());
                for line in lines(rest) {
                    writer.hard_line();
                    writer.space();
                    let trimmed = line.trim();
                    writer.text(trimmed);
                    if jsdoc && trimmed != "*" && line.ends_with("  ") {
                        // Markdown uses two trailing spaces as a hard line break.
                        writer.text("  ");
                    }
                }
            }
        }
    }
}

pub(super) struct CommentCursor<'ast> {
    remaining: &'ast [Comment],
}

impl<'ast> CommentCursor<'ast> {
    /// A prepared boundary becomes claimable after its preceding child is printed.
    pub(super) fn claim(&mut self, comments: &'ast [Comment]) -> Result<(), FormatError> {
        match (self.first(), comments.first()) {
            (_, None) => Ok(()),
            (Some(pending), Some(first)) if pending.span == first.span => {
                self.take(comments.len());
                Ok(())
            }
            (Some(pending), Some(_)) => Err(FormatError::unsupported(
                "comment placement",
                None,
                pending.span,
            )),
            (None, Some(_)) => {
                unreachable!("prepared comment boundary must remain unconsumed")
            }
        }
    }

    /// Prepares a later boundary without consuming comments inside its child.
    pub(super) fn in_range(&self, span: Span) -> &'ast [Comment] {
        let first = self
            .remaining
            .partition_point(|comment| comment.span.start < span.start);
        let comments = &self.remaining[first..];
        let count = comments.partition_point(|comment| comment.span.end <= span.end);
        &comments[..count]
    }

    pub(super) fn new(comments: &'ast [Comment]) -> Self {
        debug_assert!(
            comments
                .windows(2)
                .all(|pair| pair[0].span.end <= pair[1].span.start),
            "AST comments must remain in source order",
        );
        Self {
            remaining: comments,
        }
    }

    pub(super) fn first(&self) -> Option<Comment> {
        self.remaining.first().copied()
    }

    pub(super) fn between(&self, span: Span) -> Result<&'ast [Comment], FormatError> {
        if let Some(comment) = self.first()
            && comment.span.start < span.start
        {
            return Err(FormatError::unsupported(
                "comment placement",
                None,
                comment.span,
            ));
        }
        let count = self
            .remaining
            .iter()
            .take_while(|comment| comment.span.end <= span.end)
            .count();
        Ok(&self.remaining[..count])
    }

    pub(super) fn take(&mut self, count: usize) -> &'ast [Comment] {
        let (claimed, remaining) = self.remaining.split_at(count);
        self.remaining = remaining;
        claimed
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn prepare_comment_boundary(
        &self,
        comments: &'ast [Comment],
        source_end: u32,
        next_start: u32,
    ) -> CommentBoundary<'ast> {
        let last = comments.last();
        let end = last.map_or(source_end, |comment| comment.span.end);
        let gap = if last.is_some_and(|comment| comment.kind == CommentKind::Line) {
            CommentGap::LineComment
        } else {
            CommentGap::Source(self.line_constraint(end, next_start))
        };
        CommentBoundary {
            comments,
            source_end,
            gap,
        }
    }

    pub(super) fn comment_boundary_binding(
        &self,
        boundary: &CommentBoundary<'ast>,
        target: Span,
    ) -> CommentBinding {
        self.comment_binding(
            boundary.comments,
            Span::new(boundary.source_end, target.start),
            Some(target),
        )
    }

    pub(super) fn format_comment_boundary(
        &mut self,
        boundary: &CommentBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_comment_boundary_with_layout(
            boundary,
            CommentRunLayout::SourceAfter(boundary.source_end),
            writer,
        )
    }

    pub(super) fn format_comment_boundary_with_layout(
        &mut self,
        boundary: &CommentBoundary<'ast>,
        layout: CommentRunLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(boundary.comments)?;
        self.format_comment_run(boundary.comments, layout, writer)
    }

    pub(super) fn source_comment_gap_is_run(&self, previous: Comment, next: Comment) -> bool {
        self.source(Span::new(previous.span.end, next.span.start))
            .chars()
            .all(|character| {
                matches!(
                    character,
                    '\u{0009}'
                        | '\u{000b}'
                        | '\u{000c}'
                        | ' '
                        | '\u{00a0}'
                        | '\u{1680}'
                        | '\u{2000}'
                        ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
                )
            })
    }

    fn prepare_source_comment(
        &self,
        comment: Comment,
        run: &mut Option<SourceCommentRun>,
    ) -> Option<PreparedLineFact> {
        let ordinary = self.comment_semantics(comment).is_ordinary();
        let (start, ordinary) = match run {
            Some(previous) if self.source_comment_gap_is_run(previous.previous, comment) => {
                (previous.start, previous.ordinary && ordinary)
            }
            _ => (comment.span.start, ordinary),
        };
        *run = Some(SourceCommentRun {
            previous: comment,
            start,
            ordinary,
        });
        (comment.kind == CommentKind::Line).then(|| PreparedLineFact {
            comment,
            run_start: start,
            current_starts_line: self.source_line_starts_line(comment.span.start),
            run_starts_line: self.source_line_starts_line(start),
            ordinary,
        })
    }

    /// Prepare the complete prefix after the old physical boundary decision.
    /// Canonicalizing only its Line would discard the preceding run's owner.
    /// A negative fact remains with the array's next leading owner.
    pub(super) fn prepare_delimited_trailing_run(
        &self,
        comments: &'ast [Comment],
        source_end: u32,
        trailing_limit: u32,
        has_following: bool,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
    ) -> Option<PreparedTrailingRun<'ast>> {
        let mut run = None;
        let mut ordinary_prefix = true;
        for (index, &comment) in comments.iter().enumerate() {
            if comment.span.start >= trailing_limit {
                break;
            }
            let fact = match array_comments.as_deref_mut() {
                Some(owner) => {
                    let fact = owner.prepare(self, comment);
                    ordinary_prefix &= owner.run.as_ref().is_none_or(|run| run.ordinary);
                    fact
                }
                None => {
                    let fact = self.prepare_source_comment(comment, &mut run);
                    ordinary_prefix &= run.as_ref().is_none_or(|run| run.ordinary);
                    fact
                }
            };
            if let Some(fact) = fact {
                return (ordinary_prefix
                    && fact.movable()
                    && fact.attachment(true, has_following) == LineAttachment::Trailing)
                    .then_some(PreparedTrailingRun {
                        comments: &comments[..index + 1],
                        source_end,
                        fact,
                    });
            }
        }
        None
    }

    pub(super) fn claim_prepared_trailing_run(
        &mut self,
        run: PreparedTrailingRun<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(run.comments)?;
        self.format_prepared_trailing_run(run, None, writer)
    }

    pub(super) fn format_prepared_trailing_run(
        &self,
        run: PreparedTrailingRun<'ast>,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        debug_assert!(
            run.comments
                .first()
                .is_some_and(|comment| run.source_end <= comment.span.start)
                && run.comments.last().is_some_and(|comment| {
                    comment.kind == CommentKind::Line && comment.span == run.fact.comment.span
                })
                && run.fact.run_start <= run.fact.comment.span.start,
            "a prepared trailing run must retain its actual final Line and delimiter owner",
        );
        self.format_inline_trailing_comments_with_array_comments(
            run.comments,
            array_comments,
            writer,
        )
    }

    pub(super) fn split_separator_comments(
        &self,
        comments: &'ast [Comment],
        before_separator: usize,
        source_end: u32,
        following_start: Option<u32>,
    ) -> (&'ast [Comment], &'ast [Comment]) {
        let mut trailing_count = 0;
        let mut previous_end = source_end;
        for (index, comment) in comments.iter().enumerate() {
            if index >= before_separator
                && self.has_target_annotations
                && self.comment_has_target_annotation(*comment)
            {
                break;
            }
            if self.line_breaks(previous_end, comment.span.start) != 0 {
                break;
            }
            previous_end = comment.span.end;
            let next_start = comments
                .get(index + 1)
                .map(|next| next.span.start)
                .or(following_start);
            if index < before_separator {
                trailing_count = index + 1;
            }
            if comment.kind == CommentKind::Line
                || next_start.is_none_or(|start| self.line_breaks(comment.span.end, start) != 0)
            {
                trailing_count = index + 1;
                break;
            }
        }
        comments.split_at(trailing_count)
    }

    pub(super) fn comment_semantics(&self, comment: Comment) -> CommentSemantics {
        CommentSemantics {
            suppression: self.has_suppression
                && super::suppression::is_suppression_comment(self.ast, comment),
            diagnostic: self.comment_is_diagnostic_directive(comment),
            file_directive: self.has_file_directives && self.comment_is_file_directive(comment),
            optimizer: self.has_target_annotations && self.comment_is_optimizer_annotation(comment),
            type_cast: self.has_type_casts && self.comment_is_type_cast(comment),
        }
    }

    pub(super) fn comment_is_file_directive(&self, comment: Comment) -> bool {
        let raw = self.source(comment.span);
        if !raw.contains('@') && !raw.starts_with("///") {
            return false;
        }
        match comment.kind {
            CommentKind::Line => {
                let triple = raw.strip_prefix("///");
                let marker = triple
                    .or_else(|| raw.strip_prefix("//"))
                    .unwrap_or(raw)
                    .trim_start_matches(super::source::is_source_whitespace);
                let tag = marker
                    .split(|character| {
                        character == ':' || super::source::is_source_whitespace(character)
                    })
                    .next()
                    .unwrap_or(marker);
                if tag.eq_ignore_ascii_case("@ts-check") || tag.eq_ignore_ascii_case("@ts-nocheck")
                {
                    return true;
                }
                triple.is_some()
                    && ["<reference", "<amd-module", "<amd-dependency"]
                        .iter()
                        .any(|tag| {
                            marker
                                .strip_prefix(tag)
                                .is_some_and(|rest| rest.starts_with([' ', '\t', '/', '>']))
                        })
            }
            CommentKind::Block => {
                raw.contains('@')
                    && raw.char_indices().any(|(offset, character)| {
                        if character != '@' {
                            return false;
                        }
                        let tag = raw[offset + 1..]
                            .split(super::source::is_source_whitespace)
                            .next()
                            .unwrap_or("");
                        ["jsx", "jsxfrag", "jsximportsource", "jsxruntime"]
                            .iter()
                            .any(|name| tag.eq_ignore_ascii_case(name))
                    })
            }
        }
    }

    pub(super) fn comment_is_optimizer_annotation(&self, comment: Comment) -> bool {
        let raw = self.source(comment.span);
        raw.match_indices('@')
            .chain(raw.match_indices('#'))
            .any(|(offset, _)| {
                let tag = &raw[offset + 1..];
                tag.starts_with("__PURE__") || tag.starts_with("__NO_SIDE_EFFECTS__")
            })
    }

    pub(super) fn comment_has_target_annotation(&self, comment: Comment) -> bool {
        self.comment_is_optimizer_annotation(comment) || self.comment_is_type_cast(comment)
    }

    /// Reference input normalizes CR/CRLF before attachment. Its backwards
    /// newline check skips SPACE/TAB only; run gaps use ECMA whitespace instead.
    pub(super) fn source_line_starts_line(&self, position: u32) -> bool {
        self.ast.source()[..position as usize]
            .trim_end_matches([' ', '\t'])
            .chars()
            .next_back()
            .is_some_and(|character| matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
    }

    pub(super) fn comment_is_alignable(&self, comment: Comment) -> Result<bool, FormatError> {
        Ok(self.prepare_comment(comment)?.is_alignable())
    }

    pub(super) fn comment_binding(
        &self,
        comments: &[Comment],
        boundary: Span,
        target: Option<Span>,
    ) -> CommentBinding {
        let binding = self.comment_run_binding(comments.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.is_ordinary() {
                CommentBinding::Unbound
            } else if semantics.suppression {
                target.map_or(CommentBinding::SourceBound, CommentBinding::BoundTo)
            } else {
                CommentBinding::SourceBound
            }
        });
        if matches!(binding, CommentBinding::SourceBound)
            || self.diagnostic_range_is_sensitive(boundary)
        {
            CommentBinding::SourceBound
        } else {
            binding
        }
    }

    /// Owners distinguish absent and unknown bindings; a run moves only with one shared target.
    pub(super) fn comment_run_binding(
        &self,
        comments: impl Iterator<Item = Comment>,
        resolve: impl FnMut(Comment) -> CommentBinding,
    ) -> CommentBinding {
        if !self.has_target_annotations
            && !self.has_file_directives
            && !self.has_suppression
            && self.diagnostic_lines.is_empty()
        {
            return CommentBinding::Unbound;
        }
        CommentBinding::combine_all(comments.map(resolve))
    }

    pub(super) fn comment_is_type_cast(&self, comment: Comment) -> bool {
        let raw = self.source(comment.span);
        if !raw.starts_with("/**") {
            return false;
        }
        for (offset, _) in raw.match_indices('@') {
            let tag = &raw[offset..];
            let after = tag
                .strip_prefix("@type")
                .or_else(|| tag.strip_prefix("@satisfies"));
            if after.is_some_and(|after| {
                after
                    .as_bytes()
                    .first()
                    .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
            }) {
                return true;
            }
        }
        false
    }

    fn prepare_comment(&self, comment: Comment) -> Result<PreparedComment<'ast>, FormatError> {
        let source = self.source(comment.span);
        let (first, rest) = split_line(source);
        let text = match rest {
            None => CommentText::Inline(source.trim_end()),
            Some(rest) if lines(rest).all(|line| line.trim_start().starts_with('*')) => {
                CommentText::Aligned {
                    first,
                    rest,
                    jsdoc: source.starts_with("/**") && !source.starts_with("/***"),
                }
            }
            Some(rest) => CommentText::Literal { first, rest },
        };
        Ok(PreparedComment {
            span: comment.span,
            kind: comment.kind,
            text,
        })
    }

    pub(super) fn comment_gap(&self, start: u32, end: u32, writer: &mut Writer<'ast>) {
        self.line_constraint(start, end)
            .write(writer, FlatSeparator::Space, |writer| {
                match self.line_breaks(start, end) {
                    0 => writer.space(),
                    1 => writer.hard_line(),
                    _ => {
                        writer.hard_line();
                        if self.next_line_is_empty(start, end) {
                            writer.hard_line();
                        }
                    }
                }
            });
    }

    fn leading_comment_gap(
        &self,
        comment: &PreparedComment<'ast>,
        next_start: u32,
        grouped: bool,
        inline: bool,
        writer: &mut Writer<'ast>,
    ) {
        self.line_constraint(comment.span.end, next_start).write(
            writer,
            FlatSeparator::Space,
            |writer| match self.line_breaks(comment.span.end, next_start) {
                0 if comment.kind == CommentKind::Block => writer.space(),
                1 if comment.kind == CommentKind::Block
                    && (inline || !self.source_position_starts_line(comment.span.start)) =>
                {
                    if grouped {
                        writer.line();
                    } else {
                        writer.space();
                    }
                }
                0 | 1 => writer.hard_line(),
                _ => {
                    writer.hard_line();
                    if self.next_line_is_empty(comment.span.end, next_start) {
                        writer.hard_line();
                    }
                }
            },
        );
    }

    fn block_comment_packet(&self, comments: &[Comment]) -> Option<BlockCommentPacket> {
        let first = comments.first()?;
        if first.kind != CommentKind::Block {
            return None;
        }
        let mut packet = BlockCommentPacket {
            end: first.span.start,
            multiline: false,
        };
        for comment in comments {
            if comment.kind != CommentKind::Block
                || self.line_breaks(packet.end, comment.span.start) != 0
            {
                break;
            }
            packet.end = comment.span.end;
            packet.multiline |= self.line_breaks(comment.span.start, comment.span.end) != 0;
        }
        Some(packet)
    }

    pub(super) fn format_comment_run(
        &self,
        comments: &[Comment],
        layout: CommentRunLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_comment_run_with_array_comments(comments, layout, None, writer)
    }
    pub(super) fn format_comment_run_with_array_comments(
        &self,
        comments: &[Comment],
        layout: CommentRunLayout,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous: Option<PreparedComment<'ast>> = None;
        let mut packet: Option<BlockCommentPacket> = None;
        for (index, &comment) in comments.iter().enumerate() {
            if matches!(layout, CommentRunLayout::OwnLineTrailing)
                && packet
                    .as_ref()
                    .is_none_or(|packet| comment.span.start >= packet.end)
            {
                packet = self.block_comment_packet(&comments[index..]);
            }
            if let Some(owner) = array_comments.as_deref_mut() {
                owner.consume(self, comment);
            }
            let prepared = self.prepare_comment(comment)?;
            let line_suffix = if let (CommentRunLayout::SourceAfter(start), CommentKind::Line) =
                (layout, comment.kind)
            {
                let end = previous
                    .as_ref()
                    .map_or(start, |previous| previous.span.end);
                self.line_breaks(end, comment.span.start) == 0
            } else {
                false
            };
            if let Some(previous) = previous.as_ref() {
                let nestled = previous.span.end == prepared.span.start
                    && previous.is_alignable()
                    && prepared.is_alignable();
                if !nestled && !line_suffix {
                    match layout {
                        CommentRunLayout::Source | CommentRunLayout::SourceAfter(_) => {
                            self.comment_gap(previous.span.end, prepared.span.start, writer)
                        }
                        CommentRunLayout::OwnLineTrailing
                            if previous.kind == CommentKind::Block
                                && prepared.kind == CommentKind::Block
                                && packet.as_ref().is_some_and(|packet| packet.multiline)
                                && self.line_breaks(previous.span.end, prepared.span.start)
                                    == 0 =>
                        {
                            writer.space();
                        }
                        CommentRunLayout::Lines | CommentRunLayout::OwnLineTrailing => self
                            .line_constraint(previous.span.end, prepared.span.start)
                            .write(writer, FlatSeparator::Space, |writer| {
                                writer.hard_line();
                                if self
                                    .previous_line_is_empty(previous.span.end, prepared.span.start)
                                {
                                    writer.hard_line();
                                }
                            }),
                        CommentRunLayout::CanonicalDangling => self
                            .line_constraint(previous.span.end, prepared.span.start)
                            .write(writer, FlatSeparator::Space, Writer::hard_line),
                        CommentRunLayout::Leading(_)
                        | CommentRunLayout::GroupedLeading(_)
                        | CommentRunLayout::InlineLeading(_) => self.leading_comment_gap(
                            previous,
                            prepared.span.start,
                            !matches!(layout, CommentRunLayout::Leading(_)),
                            matches!(layout, CommentRunLayout::InlineLeading(_))
                                && comments
                                    .first()
                                    .is_some_and(|first| first.span == previous.span),
                            writer,
                        ),
                        CommentRunLayout::FlatLeading(_) => self
                            .line_constraint(previous.span.end, prepared.span.start)
                            .write(writer, FlatSeparator::Space, |writer| match previous.kind {
                                CommentKind::Line => writer.hard_line(),
                                CommentKind::Block => writer.space(),
                            }),
                    }
                }
            } else if let CommentRunLayout::SourceAfter(start) = layout
                && !line_suffix
            {
                self.comment_gap(start, prepared.span.start, writer);
            }
            if line_suffix {
                writer.line_suffix(self.source(comment.span).trim_end());
            } else {
                prepared.text.write(writer);
            }
            previous = Some(prepared);
        }
        match (layout, previous) {
            (CommentRunLayout::Leading(next), Some(previous)) => {
                self.leading_comment_gap(&previous, next, false, false, writer)
            }
            (
                CommentRunLayout::GroupedLeading(next) | CommentRunLayout::InlineLeading(next),
                Some(previous),
            ) => self.leading_comment_gap(
                &previous,
                next,
                true,
                matches!(layout, CommentRunLayout::InlineLeading(_))
                    && comments
                        .first()
                        .is_some_and(|first| first.span == previous.span),
                writer,
            ),
            (CommentRunLayout::FlatLeading(next), Some(previous)) => self
                .line_constraint(previous.span.end, next)
                .write(writer, FlatSeparator::Space, |writer| match previous.kind {
                    CommentKind::Line => writer.hard_line(),
                    CommentKind::Block => writer.space(),
                }),
            _ => {}
        }
        Ok(())
    }

    pub(super) fn format_expression_tail(
        &mut self,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_expression_tail_with_array_comments(tail, None, writer)
    }
    pub(super) fn format_expression_tail_with_array_comments(
        &mut self,
        tail: Option<ExpressionTail<'ast>>,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some(tail) = tail {
            self.comments.claim(tail.comments)?;
            self.format_comment_run_with_array_comments(
                tail.comments,
                CommentRunLayout::SourceAfter(tail.source_end),
                array_comments,
                writer,
            )?;
        }
        Ok(())
    }

    pub(super) fn format_trailing_comments(
        &mut self,
        span: Span,
        statement_end: u32,
        has_following: bool,
        pending: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let source_end = pending.as_ref().map_or(span.start, |tail| tail.source_end);
        let comments = self.comments.between(Span::new(source_end, span.end))?;
        let (comments, source_end) = if let Some(tail) = pending {
            // Keep the leased child tail and the parent's suffix in one packet:
            // a later block must not overtake a pending line suffix.
            let (inherited, following) = comments.split_at(tail.comments.len());
            let start = inherited
                .last()
                .map_or(span.start, |comment| span.start.max(comment.span.end));
            let following = self.trailing_comment_prefix(
                following,
                Span::new(start, span.end),
                statement_end,
                has_following,
            );
            (
                &comments[..inherited.len() + following.len()],
                Some(tail.source_end),
            )
        } else {
            (
                self.trailing_comment_prefix(comments, span, statement_end, has_following),
                None,
            )
        };
        self.comments.claim(comments)?;
        if let Some(last) = comments.last() {
            if let Some(source_end) = source_end {
                self.format_comment_run(
                    comments,
                    CommentRunLayout::SourceAfter(source_end),
                    writer,
                )?;
            } else {
                if self
                    .comment_binding(comments, Span::new(span.start, last.span.end), None)
                    .allows_relocation(None)
                {
                    self.format_inline_trailing_comments(comments, writer)?;
                } else {
                    self.format_comment_run(
                        comments,
                        CommentRunLayout::SourceAfter(span.start),
                        writer,
                    )?;
                }
            }
            Ok(span.start.max(last.span.end))
        } else {
            Ok(span.start)
        }
    }

    pub(super) fn format_inline_trailing_comments(
        &self,
        comments: &[Comment],
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_inline_trailing_comments_with_array_comments(comments, None, writer)
    }
    pub(super) fn format_inline_trailing_comments_with_array_comments(
        &self,
        comments: &[Comment],
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_trailing_comment_run(comments, None, array_comments, writer)
    }

    pub(super) fn format_trailing_comment_run(
        &self,
        comments: &[Comment],
        previous: Option<Comment>,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous = previous
            .map(|comment| self.prepare_comment(comment))
            .transpose()?;
        for &comment in comments {
            if let Some(owner) = array_comments.as_deref_mut() {
                owner.consume(self, comment);
            }
            let prepared = self.prepare_comment(comment)?;
            if previous
                .as_ref()
                .is_some_and(|previous| previous.kind == CommentKind::Line)
            {
                writer.hard_line();
                prepared.text.write(writer);
            } else {
                match comment.kind {
                    CommentKind::Line => writer.line_suffix(self.source(comment.span).trim_end()),
                    CommentKind::Block => {
                        let nestled = previous.as_ref().is_some_and(|previous| {
                            previous.span.end == prepared.span.start
                                && previous.is_alignable()
                                && prepared.is_alignable()
                        });
                        if !nestled {
                            writer.space();
                        }
                        prepared.text.write(writer);
                    }
                }
            }
            previous = Some(prepared);
        }
        Ok(())
    }

    pub(super) fn trailing_comment_prefix(
        &self,
        comments: &'ast [Comment],
        span: Span,
        statement_end: u32,
        has_following: bool,
    ) -> &'ast [Comment] {
        let mut end = span.start;
        let mut count = 0;
        for (index, comment) in comments.iter().enumerate() {
            if has_following
                && self.has_target_annotations
                && self.comment_has_target_annotation(*comment)
            {
                break;
            }
            if self.line_breaks(end, comment.span.start) != 0 {
                break;
            }
            let following = comments
                .get(index + 1)
                .map_or(span.end, |next| next.span.start);
            if comment.kind == CommentKind::Line
                || self.line_breaks(comment.span.end, following) != 0
            {
                count = index + 1;
                break;
            }
            end = comment.span.end;
            if !has_following || comment.span.end < statement_end {
                count = index + 1;
            }
        }
        &comments[..count]
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_source_token_gap(
        &mut self,
        start: u32,
        end: u32,
        space: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comments = self.comments.between(Span::new(start, end))?;
        self.comments.claim(comments)?;
        if space {
            let first = comments.first().map_or(end, |comment| comment.span.start);
            self.line_constraint(start, first)
                .write(writer, FlatSeparator::Space, Writer::space);
            self.format_comment_run(comments, CommentRunLayout::FlatLeading(end), writer)
        } else {
            self.format_comment_run(comments, CommentRunLayout::SourceAfter(start), writer)?;
            let last = comments.last().map_or(start, |comment| comment.span.end);
            self.line_constraint(last, end)
                .write(writer, FlatSeparator::Empty, |writer| {
                    if comments
                        .last()
                        .is_some_and(|comment| comment.kind == swc_next_ecma_ast::CommentKind::Line)
                    {
                        writer.hard_line();
                    }
                });
            Ok(())
        }
    }
}
