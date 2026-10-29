//! Bounded, source-order comment consumption and basic comment layout.

use swc_next_ecma_ast::{Comment, CommentKind, Span};

use super::Formatter;
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::source::{is_source_whitespace, lines, split_line};
use crate::{FormatError, document::Writer};

pub(super) enum PreparedBoundaryComments<'ast> {
    Preserved(CommentGap),
    Head {
        gap: CommentGap,
    },
    Trailing {
        run: PreparedTrailingRun<'ast>,
        after: CommentBoundary<'ast>,
        gap: CommentGap,
    },
}

#[derive(Default)]
struct JsDocBindingRoles {
    cast: bool,
    non_cast: bool,
}

impl JsDocBindingRoles {
    fn is_empty(&self) -> bool {
        !self.cast && !self.non_cast
    }
}

enum JsDocBindingRole {
    Cast,
    NonCast,
}

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
    /// Keep a terminal packet's Blocks in place and its ordinary Line suffix inline.
    SourceDangling,
    SourceAfter(u32),
    /// Preserve source order when a released expression tail joins a statement suffix.
    StatementTail(u32),
    SourceLeading {
        next: u32,
        placement: HeadGapPlacement,
    },
    Leading(u32),
    GroupedLeading(u32),
    InlineLeading(u32),
    FlatLeading(u32),
    CanonicalDangling,
    Lines,
    OwnLineTrailing,
}

/// A caller's separator covers only the first horizontal comment gap.
#[derive(Clone, Copy)]
pub(super) enum CommentOpening {
    Needed,
    Provided,
}

impl CommentOpening {
    fn separator(self) -> FlatSeparator {
        match self {
            Self::Needed => FlatSeparator::Space,
            Self::Provided => FlatSeparator::Empty,
        }
    }

    fn write(self, writer: &mut Writer<'_>) {
        if matches!(self, Self::Needed) {
            writer.space();
        }
    }

    pub(super) fn line_suffix<'a>(
        self,
        writer: &mut Writer<'a>,
        contents: impl FnOnce(&mut Writer<'a>),
    ) {
        writer.line_suffix(|writer| {
            self.write(writer);
            contents(writer);
        });
        writer.break_parent();
    }
}

/// A real-node transition certified before comments or punctuation are claimed.
#[derive(Clone, Copy)]
pub(super) struct SourceSeparatorTransition {
    pub(super) source: Span,
    pub(super) destination: Span,
    pub(super) comma: u32,
    pub(super) pre_packet_empty: bool,
    pub(super) following_gap: Option<PreparedSeparatorGap>,
}

/// Following layout belongs to its real owner, not the consumed comma frontier.
#[derive(Clone, Copy)]
pub(super) struct PreparedSeparatorGap {
    pub(super) owner_start: u32,
    pub(super) layout: LineConstraint,
}

#[derive(Clone, Copy)]
enum SourceSeparatorGap {
    BeforePacket,
    WithinPacket,
}

/// The certified transition reborrows its still-unclaimed comment packet.
#[derive(Clone, Copy)]
pub(super) struct SourceSeparatorPacket<'ast> {
    pub(super) comments: &'ast [Comment],
    transition: SourceSeparatorTransition,
}

/// Placement belongs to the emitted owner, independently of a removed opener.
#[derive(Clone, Copy)]
pub(super) enum HeadGapPlacement {
    Inline,
    OwnLine,
}

/// A layout fact for one complete packet; it neither claims nor rebinds comments.
#[derive(Clone, Copy)]
pub(super) struct PreparedHeadGap {
    packet: Span,
    count: usize,
    placement: HeadGapPlacement,
    owner: HeadGapOwner,
}

#[derive(Clone, Copy)]
enum HeadGapOwner {
    Relocatable,
    SourceAdjacent(u32),
}

impl PreparedHeadGap {
    pub(super) fn new(comments: &[Comment], placement: HeadGapPlacement) -> Option<Self> {
        Some(Self {
            packet: Span::new(comments.first()?.span.start, comments.last()?.span.end),
            count: comments.len(),
            placement,
            owner: HeadGapOwner::Relocatable,
        })
    }

    /// The preparer proved whitespace-only adjacency to this unchanged token.
    pub(super) fn source_adjacent(
        comments: &[Comment],
        placement: HeadGapPlacement,
        token: u32,
    ) -> Option<Self> {
        let mut gap = Self::new(comments, placement)?;
        gap.owner = HeadGapOwner::SourceAdjacent(token);
        Some(gap)
    }

    fn apply(
        self,
        mut head: NodeHead<'_>,
        binding: CommentBinding,
        source_adjacent: bool,
    ) -> NodeHead<'_> {
        let same_owner = matches!(binding, CommentBinding::Unbound)
            || match self.owner {
                HeadGapOwner::Relocatable => false,
                HeadGapOwner::SourceAdjacent(token) => {
                    source_adjacent && head.content_start == token && head.gap_end == Some(token)
                }
            };
        if same_owner
            && head.comments.len() == self.count
            && head
                .comments
                .first()
                .is_some_and(|first| first.span.start == self.packet.start)
            && head
                .comments
                .last()
                .is_some_and(|last| last.span.end == self.packet.end)
        {
            head.layout = CommentRunLayout::SourceLeading {
                next: head.gap_end.unwrap_or(head.content_start),
                placement: self.placement,
            };
            // The shared run now emits this final gap exactly once.
            head.gap_end = None;
        }
        head
    }
}

/// At most one exterior packet and one normalized released packet share a head.
pub(super) struct PreparedHeadGaps {
    prefix: Option<PreparedHeadGap>,
    head: Option<PreparedHeadGap>,
    grouped: bool,
}

impl PreparedHeadGaps {
    pub(super) fn new(
        prefix: Option<PreparedHeadGap>,
        head: Option<PreparedHeadGap>,
        grouped: bool,
    ) -> Option<Self> {
        (prefix.is_some() || head.is_some()).then_some(Self {
            prefix,
            head,
            grouped,
        })
    }

    pub(super) fn requires_group(&self) -> bool {
        self.grouped
    }

    pub(super) fn apply<'ast>(
        &self,
        mut head: NodeHead<'ast>,
        binding: CommentBinding,
        source_adjacent: bool,
    ) -> NodeHead<'ast> {
        if let Some(prefix) = self.prefix {
            head = prefix.apply(head, binding, source_adjacent);
        }
        if let Some(gap) = self.head {
            head = gap.apply(head, binding, source_adjacent);
        }
        head
    }
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

/// Source statement extent and the emitted terminator's position relative to a leased tail.
#[derive(Clone, Copy)]
pub(super) struct StatementCommentBoundary {
    pub(super) source_end: u32,
    pub(super) terminator: StatementTerminatorPlacement,
}

#[derive(Clone, Copy)]
pub(super) enum StatementTerminatorPlacement {
    BeforeTail,
    Adjacent,
}

/// The removed source closer lies between the leased tail and its suffix.
#[derive(Clone, Copy)]
struct ReleasedTailGap {
    tail_end: u32,
    source_end: u32,
}

/// Source facts remain valid after the pending run has been claimed.
pub(super) struct CommentBoundary<'ast> {
    comments: &'ast [Comment],
    source_end: u32,
    gap: CommentGap,
}

impl<'ast> CommentBoundary<'ast> {
    /// Adjacent tokens have no intervening comments or source line break.
    pub(super) fn adjacent(position: u32) -> Self {
        Self {
            comments: &[],
            source_end: position,
            gap: CommentGap::Source(LineConstraint::SameLine),
        }
    }

    pub(super) fn first_comment_start(&self) -> Option<u32> {
        self.comments.first().map(|comment| comment.span.start)
    }

    pub(super) fn comments(&self) -> &'ast [Comment] {
        self.comments
    }

    /// Source end supplied by the actual preceding syntax owner.
    pub(super) fn source_end(&self) -> u32 {
        self.source_end
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

    /// Resolve a token boundary to a required line break or one literal space.
    pub(super) fn write_separator(self, writer: &mut Writer<'_>) {
        match self {
            Self::Source(LineConstraint::Unconstrained | LineConstraint::SameLine) => {
                writer.text(" ");
            }
            _ => self.write(writer, FlatSeparator::Empty, |_| {}),
        }
    }
}

pub(super) struct PreparedLeadingCommentGap {
    pub(super) source_breaks: u8,
    pub(super) gap: CommentGap,
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

/// A real array item resets source protection; commas and whitespace do not.
struct SourceCommentPrefix {
    previous: Comment,
    ordinary: bool,
}

/// One selected array owns this borrowed source stream through its real children.
/// A boundary can prepare a Line before the following leading owner emits it.
pub(super) struct ArrayCommentOwner<'ast> {
    remaining: &'ast [Comment],
    first_end: u32,
    last_start: u32,
    run: Option<SourceCommentRun>,
    prefix: Option<SourceCommentPrefix>,
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
            prefix: None,
            pending: None,
            reference_inline_trailing: false,
        }
    }

    fn advance(
        &mut self,
        formatter: &Formatter<'ast, '_>,
        comment: Comment,
    ) -> Option<PreparedLineFact> {
        let fact = formatter.prepare_array_comment(comment, &mut self.run, &mut self.prefix);
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

/// One complete prefix owned by a real delimiter boundary.
/// Its source facts survive a split between Blocks and the final Line suffix.
pub(super) struct PreparedTrailingRun<'ast> {
    comments: &'ast [Comment],
    source_end: u32,
    fact: PreparedLineFact,
}

impl<'ast> PreparedTrailingRun<'ast> {
    pub(super) fn len(&self) -> usize {
        self.comments.len()
    }

    /// Blocks trail the operand; only the final Line follows its separator.
    pub(super) fn split_at_line(mut self) -> (&'ast [Comment], Self) {
        let (blocks, line) = self.comments.split_at(self.comments.len() - 1);
        self.comments = line;
        (blocks, self)
    }
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
                let mut preserve_trailing = false;
                for line in lines(rest) {
                    if preserve_trailing {
                        writer.literal_line_with_indent();
                    } else {
                        writer.hard_line();
                    }
                    writer.space();
                    let trimmed = line.trim();
                    writer.text(trimmed);
                    preserve_trailing = jsdoc && trimmed != "*" && line.ends_with("  ");
                    if preserve_trailing {
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
        if self
            .remaining
            .first()
            .is_none_or(|comment| comment.span.end > span.end)
        {
            return &self.remaining[..0];
        }
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
    /// Source-bound punctuation keeps its geometry without making its comments
    /// part of a protected node's verbatim contents.
    pub(super) fn format_source_gap(
        &mut self,
        span: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comments = self.comments.between(span)?;
        self.comments.claim(comments)?;
        let mut start = span.start;
        for &comment in comments {
            let prepared = self.prepare_comment(comment)?;
            if !prepared.is_alignable() {
                continue;
            }
            writer.literal_text(self.source(Span::new(start, comment.span.start)));
            prepared.text.write(writer);
            start = comment.span.end;
        }
        writer.literal_text(self.source(Span::new(start, span.end)));
        Ok(())
    }

    fn format_comment_line_suffix_with_opening(
        &self,
        comment: Comment,
        opening: CommentOpening,
        writer: &mut Writer<'ast>,
    ) {
        opening.line_suffix(writer, |writer| {
            writer.text(self.source(comment.span).trim_end());
        });
    }

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

    pub(super) fn format_comment_boundary_after_separator(
        &mut self,
        boundary: &CommentBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(boundary.comments)?;
        self.format_comment_run_with_opening(
            boundary.comments,
            CommentRunLayout::SourceAfter(boundary.source_end),
            CommentOpening::Provided,
            None,
            None,
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
            .all(super::source::is_source_horizontal_whitespace)
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

    fn prepare_array_comment(
        &self,
        comment: Comment,
        run: &mut Option<SourceCommentRun>,
        prefix: &mut Option<SourceCommentPrefix>,
    ) -> Option<PreparedLineFact> {
        let mut ordinary = self.comment_semantics(comment).is_ordinary();
        if let Some(previous) = prefix
            && self
                .source(Span::new(previous.previous.span.end, comment.span.start))
                .chars()
                .all(|character| is_source_whitespace(character) || character == ',')
        {
            ordinary &= previous.ordinary;
        }
        *prefix = Some(SourceCommentPrefix {
            previous: comment,
            ordinary,
        });
        let mut fact = self.prepare_source_comment(comment, run)?;
        if !ordinary {
            // Source-owned packets retain physical gaps instead of releasing
            // a Line across indentation that the renderer will normalize.
            fact.current_starts_line = self.source_position_starts_line(comment.span.start);
            fact.run_starts_line = self.source_position_starts_line(fact.run_start);
        }
        Some(fact)
    }

    pub(super) fn array_has_inline_line_comments(
        &self,
        comments: &[Comment],
        first_end: u32,
    ) -> bool {
        let mut run = None;
        let mut prefix = None;
        comments.iter().any(|&comment| {
            self.prepare_array_comment(comment, &mut run, &mut prefix)
                .is_some_and(|fact| comment.span.start >= first_end && !fact.current_starts_line)
        })
    }

    /// Prepare the complete prefix after the old physical boundary decision.
    /// Canonicalizing only its Line would discard the preceding run's owner.
    /// A rejected prefix keeps its leading or dangling owner.
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
        let mut inline_prefix = true;
        for (index, &comment) in comments.iter().enumerate() {
            if comment.span.start >= trailing_limit {
                break;
            }
            inline_prefix &= !self.source_line_starts_line(comment.span.start);
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
                // End boundaries keep own-line Blocks dangling. The final
                // source run cannot lift that complete prefix onto the item.
                return (ordinary_prefix
                    && (has_following || inline_prefix)
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

    /// Keep a complete source-inline optimizer suffix on its emitted comma.
    /// The same owner is recovered after a released source pair disappears.
    pub(super) fn prepare_delimited_optimizer_run(
        &self,
        comments: &'ast [Comment],
        tail: Option<&ExpressionTail<'ast>>,
        comma: Option<u32>,
    ) -> Option<PreparedTrailingRun<'ast>> {
        let first = comments.first()?;
        let (source_end, limit) = if let Some(comma) = comma
            && comma < first.span.start
        {
            (comma + 1, None)
        } else {
            let tail = tail?;
            if tail.comments.first()?.span != first.span {
                return None;
            }
            (tail.source_end, Some(tail.comments.last()?.span.end))
        };
        let mut previous_end = source_end;
        let mut optimizer = false;
        let mut run = None;
        for (index, &comment) in comments.iter().enumerate() {
            if limit.is_some_and(|limit| comment.span.end > limit)
                || comment.span.start < previous_end
                || !self
                    .source(Span::new(previous_end, comment.span.start))
                    .chars()
                    .all(|character| {
                        is_source_whitespace(character)
                            && !matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                    })
            {
                return None;
            }
            let semantics = self.comment_semantics(comment);
            if semantics.suppression
                || semantics.diagnostic
                || semantics.file_directive
                || semantics.type_cast
            {
                return None;
            }
            optimizer |= semantics.optimizer;
            let fact = self.prepare_source_comment(comment, &mut run);
            if let Some(fact) = fact {
                return optimizer.then_some(PreparedTrailingRun {
                    comments: &comments[..index + 1],
                    source_end,
                    fact,
                });
            }
            previous_end = comment.span.end;
        }
        None
    }

    fn optimizer_only_packet(&self, comments: &[Comment]) -> bool {
        let mut optimizer = false;
        for &comment in comments {
            let semantics = self.comment_semantics(comment);
            if semantics.suppression
                || semantics.diagnostic
                || semantics.file_directive
                || semantics.type_cast
            {
                return false;
            }
            optimizer |= semantics.optimizer;
        }
        optimizer
    }

    pub(super) fn prepare_bound_comments(
        &self,
        boundary: &CommentBoundary<'ast>,
        target: Span,
    ) -> PreparedBoundaryComments<'ast> {
        let gap = if boundary
            .comments
            .last()
            .is_some_and(|comment| self.source_line_starts_line(comment.span.start))
        {
            CommentGap::Source(self.source_line_constraint(boundary.end(), target.start))
        } else {
            boundary.gap
        };
        if boundary.is_empty()
            || !matches!(
                self.comment_boundary_binding(boundary, target),
                CommentBinding::Unbound
            )
        {
            return PreparedBoundaryComments::Preserved(gap);
        }
        let mut source_run = None;
        let mut blocks = true;
        for (index, &comment) in boundary.comments.iter().enumerate() {
            let fact = self.prepare_source_comment(comment, &mut source_run);
            blocks &= comment.kind == CommentKind::Block;
            if let Some(fact) = fact {
                if !fact.movable() || fact.attachment(true, true) != LineAttachment::Trailing {
                    return PreparedBoundaryComments::Preserved(gap);
                }
                let run = PreparedTrailingRun {
                    comments: &boundary.comments[..index + 1],
                    source_end: boundary.source_end,
                    fact,
                };
                let after = CommentBoundary {
                    comments: &boundary.comments[index + 1..],
                    source_end: fact.comment.span.end,
                    gap: boundary.gap,
                };
                return PreparedBoundaryComments::Trailing { run, after, gap };
            }
        }
        if blocks {
            PreparedBoundaryComments::Head { gap }
        } else {
            PreparedBoundaryComments::Preserved(gap)
        }
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

    pub(super) fn separator_comment_starts_following(
        &self,
        comment: Comment,
        index: usize,
        before_separator: usize,
    ) -> bool {
        index >= before_separator
            && self.has_target_annotations
            && self.comment_has_target_annotation(comment)
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
            if self.separator_comment_starts_following(*comment, index, before_separator) {
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

    fn line_comment_directive_marker(&self, comment: Comment) -> Option<&'ast str> {
        if comment.kind != CommentKind::Line {
            return None;
        }
        let raw = self.source(comment.span);
        let whitespace = |character| is_source_whitespace(character) && character != '\u{85}';
        let marker = raw
            .strip_prefix("///")
            .or_else(|| raw.strip_prefix("//"))?
            .trim_start_matches(whitespace);
        Some(marker)
    }

    pub(super) fn comment_checking_mode(&self, comment: Comment) -> Option<bool> {
        Self::checking_mode_tag(self.line_comment_directive_marker(comment)?)
    }

    fn checking_mode_tag(marker: &str) -> Option<bool> {
        let tag = marker
            .split(|character| {
                character == ':' || is_source_whitespace(character) && character != '\u{85}'
            })
            .next()
            .unwrap_or(marker);
        if tag.eq_ignore_ascii_case("@ts-check") {
            Some(true)
        } else if tag.eq_ignore_ascii_case("@ts-nocheck") {
            Some(false)
        } else {
            None
        }
    }

    pub(super) fn comment_is_file_directive(&self, comment: Comment) -> bool {
        let raw = self.source(comment.span);
        if !raw.contains('@') && !raw.starts_with("///") {
            return false;
        }
        match comment.kind {
            CommentKind::Line => {
                let marker = self.line_comment_directive_marker(comment).unwrap_or("");
                if Self::checking_mode_tag(marker).is_some() {
                    return true;
                }
                raw.starts_with("///")
                    && ["<reference", "<amd-module", "<amd-dependency"]
                        .iter()
                        .any(|tag| {
                            marker
                                .strip_prefix(tag)
                                .is_some_and(|rest| rest.starts_with([' ', '\t', '/', '>']))
                        })
            }
            CommentKind::Block => memchr::memchr_iter(b'@', raw.as_bytes()).any(|offset| {
                let tag = &raw[offset + 1..];
                ["jsx", "jsxfrag", "jsximportsource", "jsxruntime"]
                    .iter()
                    .any(|name| {
                        tag.get(..name.len())
                            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(name))
                            && tag[name.len()..]
                                .chars()
                                .next()
                                .is_none_or(super::source::is_source_whitespace)
                    })
            }),
        }
    }

    pub(super) fn comment_is_optimizer_annotation(&self, comment: Comment) -> bool {
        let raw = self.source(comment.span);
        memchr::memchr2_iter(b'@', b'#', raw.as_bytes()).any(|offset| {
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

    pub(super) fn has_jsdoc_enclosures(&self) -> bool {
        *self.has_jsdoc_enclosures.get_or_init(|| {
            self.ast
                .comments()
                .iter()
                .any(|&comment| self.comment_needs_jsdoc_enclosure(comment))
        })
    }

    /// Compiler-recognized JSDoc roles retain their original syntax owner.
    /// Optimizer markers and unknown documentation tags have separate policies.
    pub(super) fn comment_is_jsdoc_binding(&self, comment: Comment) -> bool {
        !self.comment_jsdoc_binding_roles(comment).is_empty()
    }

    /// Preserve roles that have no existing Closure-cast enclosure policy.
    pub(super) fn comment_needs_jsdoc_enclosure(&self, comment: Comment) -> bool {
        let roles = self.comment_jsdoc_binding_roles(comment);
        roles.non_cast || roles.cast && !self.comment_is_type_cast(comment)
    }

    fn comment_jsdoc_binding_roles(&self, comment: Comment) -> JsDocBindingRoles {
        let raw = self.source(comment.span);
        let mut roles = JsDocBindingRoles::default();
        if raw.starts_with("/**") {
            for (offset, _) in raw.match_indices('@') {
                match Self::jsdoc_binding_tag(&raw[offset + 1..]) {
                    Some(JsDocBindingRole::Cast) => roles.cast = true,
                    Some(JsDocBindingRole::NonCast) => roles.non_cast = true,
                    None => {}
                }
            }
        }
        roles
    }

    fn jsdoc_binding_tag(mut tag: &str) -> Option<JsDocBindingRole> {
        // Every supported role is ASCII; constructor is the longest name.
        let mut name = [0; 11];
        let mut length = 0;
        while let Some(mut character) = tag.chars().next() {
            tag = &tag[character.len_utf8()..];
            let escaped = character == '\\';
            if escaped {
                let rest = tag.strip_prefix('u')?;
                let (digits, rest) = if let Some(braced) = rest.strip_prefix('{') {
                    let end = braced.bytes().take_while(u8::is_ascii_hexdigit).count();
                    let (digits, rest) = braced.split_at(end);
                    let rest = rest.strip_prefix('}')?;
                    (digits, rest)
                } else {
                    let (Some(digits), Some(rest)) = (rest.get(..4), rest.get(4..)) else {
                        return None;
                    };
                    (digits, rest)
                };
                if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return None;
                }
                let decoded = u32::from_str_radix(digits, 16)
                    .ok()
                    .and_then(char::from_u32)?;
                character = decoded;
                tag = rest;
            }
            let identifier_part = if character.is_ascii() {
                swc_next_ecma_ast::is_ident_continue_ascii(character as u8)
            } else {
                let mut identifier = [0; 5];
                identifier[0] = b'x';
                let end = 1 + character.encode_utf8(&mut identifier[1..]).len();
                swc_next_ecma_ast::is_identifier(&identifier[..end])
            };
            if !identifier_part {
                if escaped {
                    return None;
                }
                break;
            }
            if !character.is_ascii() || length == name.len() {
                return None;
            }
            name[length] = character as u8;
            length += 1;
        }
        match &name[..length] {
            b"type" | b"satisfies" => Some(JsDocBindingRole::Cast),
            b"typedef" | b"callback" | b"overload" | b"template" | b"param" | b"arg"
            | b"argument" | b"return" | b"returns" | b"extends" | b"augments" | b"implements"
            | b"class" | b"constructor" | b"enum" | b"this" | b"import" | b"deprecated"
            | b"public" | b"private" | b"protected" | b"readonly" | b"override" => {
                Some(JsDocBindingRole::NonCast)
            }
            _ => None,
        }
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

    pub(super) fn source_separator_packet(
        &self,
        comments: &'ast [Comment],
        destination: Span,
        transition: Option<SourceSeparatorTransition>,
    ) -> Option<SourceSeparatorPacket<'ast>> {
        let transition = transition?;
        if transition.destination != destination
            || comments
                .first()
                .is_some_and(|comment| comment.span.start < transition.source.start)
            || comments
                .last()
                .is_some_and(|comment| comment.span.end > transition.source.end)
        {
            return None;
        }
        Some(SourceSeparatorPacket {
            comments,
            transition,
        })
    }

    pub(super) fn source_separator_constraint(
        &self,
        start: u32,
        end: u32,
        transition: Option<&SourceSeparatorTransition>,
    ) -> LineConstraint {
        let Some(transition) = transition else {
            return self.line_constraint(start, end);
        };
        if let Some(gap) = transition.following_gap
            && gap.owner_start == end
            && transition.source.start <= start
            && start <= end
            && end <= transition.source.end
        {
            return gap.layout;
        }
        self.source_transition_constraint(
            start,
            end,
            Some(transition),
            SourceSeparatorGap::BeforePacket,
        )
    }

    fn source_transition_constraint(
        &self,
        start: u32,
        end: u32,
        transition: Option<&SourceSeparatorTransition>,
        owner: SourceSeparatorGap,
    ) -> LineConstraint {
        let constraint = self.line_constraint(start, end);
        if matches!(constraint, LineConstraint::Unconstrained) {
            return constraint;
        }
        if let Some(transition) = transition
            && start <= transition.comma
            && transition.comma < end
            && start >= transition.source.start
            && end <= transition.source.end
        {
            let (breaks, empty) = self.transition_gap_lines(start, end, Some(transition), owner);
            return if breaks == 0 {
                LineConstraint::SameLine
            } else {
                LineConstraint::NewLine(if empty { 2 } else { 1 })
            };
        }
        constraint
    }

    pub(super) fn packet_gap_lines(
        &self,
        start: u32,
        end: u32,
        packet: Option<&SourceSeparatorPacket<'ast>>,
    ) -> (u8, bool) {
        self.transition_gap_lines(
            start,
            end,
            packet.map(|packet| &packet.transition),
            SourceSeparatorGap::WithinPacket,
        )
    }

    fn transition_gap_lines(
        &self,
        start: u32,
        end: u32,
        transition: Option<&SourceSeparatorTransition>,
        owner: SourceSeparatorGap,
    ) -> (u8, bool) {
        if let Some(transition) = transition
            && start <= transition.comma
            && transition.comma < end
        {
            let before = self.line_breaks(start, transition.comma);
            let after = self.line_breaks(transition.comma + 1, end);
            return match owner {
                SourceSeparatorGap::BeforePacket => {
                    (before.max(after), transition.pre_packet_empty)
                }
                // The source gap after the comma belongs before the complete
                // leading packet. Only its pre-comma blank stays inside it.
                SourceSeparatorGap::WithinPacket => (
                    before.max(after.min(1)),
                    self.next_line_is_empty(start, transition.comma),
                ),
            };
        }
        let breaks = self.line_breaks(start, end);
        (breaks, breaks > 1 && self.next_line_is_empty(start, end))
    }

    pub(super) fn comment_gap(&self, start: u32, end: u32, writer: &mut Writer<'ast>) {
        self.comment_gap_with_opening(start, end, CommentOpening::Needed, writer);
    }

    fn comment_gap_with_opening(
        &self,
        start: u32,
        end: u32,
        opening: CommentOpening,
        writer: &mut Writer<'ast>,
    ) {
        self.comment_gap_constraint(start, end)
            .write(writer, opening.separator(), |_| {});
    }

    /// Resolve the final source gap after an already-emitted comment packet.
    /// Its own geometry remains independent of an earlier operator's layout.
    pub(super) fn comment_gap_constraint(&self, start: u32, end: u32) -> LineConstraint {
        match self.line_constraint(start, end) {
            LineConstraint::Unconstrained => match self.line_breaks(start, end) {
                0 => LineConstraint::SameLine,
                1 => LineConstraint::NewLine(1),
                _ => LineConstraint::NewLine(if self.next_line_is_empty(start, end) {
                    2
                } else {
                    1
                }),
            },
            constraint => constraint,
        }
    }

    pub(super) fn comment_gap_with_packet(
        &self,
        start: u32,
        end: u32,
        packet: Option<&SourceSeparatorPacket<'ast>>,
        writer: &mut Writer<'ast>,
    ) {
        self.comment_gap_with_packet_and_opening(
            start,
            end,
            packet,
            CommentOpening::Needed,
            writer,
        );
    }

    fn comment_gap_with_packet_and_opening(
        &self,
        start: u32,
        end: u32,
        packet: Option<&SourceSeparatorPacket<'ast>>,
        opening: CommentOpening,
        writer: &mut Writer<'ast>,
    ) {
        if packet.is_none() {
            self.comment_gap_with_opening(start, end, opening, writer);
            return;
        }
        let (breaks, empty) = self.packet_gap_lines(start, end, packet);
        self.source_transition_constraint(
            start,
            end,
            packet.map(|packet| &packet.transition),
            SourceSeparatorGap::WithinPacket,
        )
        .write(writer, opening.separator(), |writer| match breaks {
            0 => opening.write(writer),
            1 => writer.hard_line(),
            _ => {
                writer.hard_line();
                if empty {
                    writer.hard_line();
                }
            }
        });
    }

    /// The source gap and its effective leading-comment layout share one lookup.
    pub(super) fn prepare_leading_comment_gap(
        &self,
        span: Span,
        kind: CommentKind,
        next_start: u32,
        inline: bool,
    ) -> PreparedLeadingCommentGap {
        let constraint = self.line_constraint(span.end, next_start);
        let breaks = match constraint {
            LineConstraint::SameLine => 0,
            LineConstraint::NewLine(count) => count,
            LineConstraint::Unconstrained => self.line_breaks(span.end, next_start),
        };
        let constraint = match constraint {
            LineConstraint::Unconstrained => match breaks {
                0 if kind == CommentKind::Block => LineConstraint::SameLine,
                1 if kind == CommentKind::Block
                    && (inline || !self.source_position_starts_line(span.start)) =>
                {
                    LineConstraint::Unconstrained
                }
                0 | 1 => LineConstraint::NewLine(1),
                _ => LineConstraint::NewLine(if self.next_line_is_empty(span.end, next_start) {
                    2
                } else {
                    1
                }),
            },
            constraint => constraint,
        };
        PreparedLeadingCommentGap {
            source_breaks: breaks,
            gap: CommentGap::Source(constraint),
        }
    }

    fn leading_comment_gap(
        &self,
        comment: &PreparedComment<'ast>,
        next_start: u32,
        grouped: bool,
        inline: bool,
        writer: &mut Writer<'ast>,
    ) {
        self.leading_comment_gap_with_packet(comment, next_start, grouped, inline, None, writer);
    }

    fn leading_comment_gap_with_placement(
        &self,
        comment: &PreparedComment<'ast>,
        next_start: u32,
        grouped: bool,
        placement: Option<HeadGapPlacement>,
        writer: &mut Writer<'ast>,
    ) {
        self.leading_comment_gap_with_prepared_placement(
            comment, next_start, grouped, placement, None, writer,
        );
    }

    fn leading_comment_gap_with_packet(
        &self,
        comment: &PreparedComment<'ast>,
        next_start: u32,
        grouped: bool,
        inline: bool,
        packet: Option<&SourceSeparatorPacket<'ast>>,
        writer: &mut Writer<'ast>,
    ) {
        self.leading_comment_gap_with_prepared_placement(
            comment,
            next_start,
            grouped,
            inline.then_some(HeadGapPlacement::Inline),
            packet,
            writer,
        );
    }

    fn leading_comment_gap_with_prepared_placement(
        &self,
        comment: &PreparedComment<'ast>,
        next_start: u32,
        grouped: bool,
        placement: Option<HeadGapPlacement>,
        packet: Option<&SourceSeparatorPacket<'ast>>,
        writer: &mut Writer<'ast>,
    ) {
        if packet.is_none() {
            let inline = matches!(placement, Some(HeadGapPlacement::Inline));
            self.prepare_leading_comment_gap(comment.span, comment.kind, next_start, inline)
                .gap
                .write(writer, FlatSeparator::Space, |writer| {
                    if grouped {
                        writer.line();
                    } else {
                        writer.space();
                    }
                });
            return;
        }
        let (breaks, empty) = self.packet_gap_lines(comment.span.end, next_start, packet);
        self.source_transition_constraint(
            comment.span.end,
            next_start,
            packet.map(|packet| &packet.transition),
            SourceSeparatorGap::WithinPacket,
        )
        .write(writer, FlatSeparator::Space, |writer| match breaks {
            0 if comment.kind == CommentKind::Block => writer.space(),
            1 if comment.kind == CommentKind::Block
                && match placement {
                    Some(HeadGapPlacement::Inline) => true,
                    Some(HeadGapPlacement::OwnLine) => false,
                    None => !self.source_position_starts_line(comment.span.start),
                } =>
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
                if empty {
                    writer.hard_line();
                }
            }
        });
    }

    fn relocated_type_head_gap(
        &self,
        comment: &PreparedComment<'ast>,
        next_start: u32,
        inline: &mut bool,
        writer: &mut Writer<'ast>,
    ) {
        let gap = self.source(Span::new(comment.span.end, next_start));
        let boundary = gap
            .find(|character| !is_source_whitespace(character))
            .filter(|&offset| matches!(gap.as_bytes()[offset], b'&' | b'|'))
            .map_or(next_start, |offset| comment.span.end + offset as u32);
        let breaks = self.line_breaks(comment.span.end, boundary);
        self.leading_comment_gap(comment, boundary, true, *inline, writer);
        if comment.kind == CommentKind::Line || breaks != 0 {
            *inline = false;
        } else if boundary != next_start {
            // The removed operator joins the following comment to this packet.
            *inline |= !self.source_position_starts_line(comment.span.start);
        }
    }

    pub(super) fn format_relocated_type_head(
        &self,
        comments: &[Comment],
        next_start: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous: Option<PreparedComment<'ast>> = None;
        let mut inline = false;
        for &comment in comments {
            let prepared = self.prepare_comment(comment)?;
            if let Some(previous) = previous.as_ref() {
                let nestled = previous.span.end == prepared.span.start
                    && previous.is_alignable()
                    && prepared.is_alignable();
                if !nestled {
                    self.relocated_type_head_gap(
                        previous,
                        prepared.span.start,
                        &mut inline,
                        writer,
                    );
                }
            }
            if comment.kind == CommentKind::Line {
                writer.line_suffix_boundary();
            }
            prepared.text.write(writer);
            previous = Some(prepared);
        }
        if let Some(previous) = previous {
            self.relocated_type_head_gap(&previous, next_start, &mut inline, writer);
        }
        Ok(())
    }

    pub(super) fn format_comment_run(
        &self,
        comments: &[Comment],
        layout: CommentRunLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_comment_run_with_array_comments(comments, layout, None, writer)
    }

    pub(super) fn format_comment_run_after_separator(
        &self,
        comments: &[Comment],
        layout: CommentRunLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_comment_run_with_opening(
            comments,
            layout,
            CommentOpening::Provided,
            None,
            None,
            writer,
        )
    }
    pub(super) fn format_comment_run_with_array_comments(
        &self,
        comments: &[Comment],
        layout: CommentRunLayout,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if comments.is_empty() {
            return Ok(());
        }
        self.format_comment_run_with_source_packet(comments, layout, None, array_comments, writer)
    }

    pub(super) fn format_comment_run_with_source_packet(
        &self,
        comments: &[Comment],
        layout: CommentRunLayout,
        packet: Option<&SourceSeparatorPacket<'ast>>,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_comment_run_with_opening(
            comments,
            layout,
            CommentOpening::Needed,
            packet,
            array_comments,
            writer,
        )
    }

    fn format_comment_run_with_opening(
        &self,
        comments: &[Comment],
        layout: CommentRunLayout,
        opening: CommentOpening,
        packet: Option<&SourceSeparatorPacket<'ast>>,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comments = packet.map_or(comments, |packet| packet.comments);
        let mut previous: Option<PreparedComment<'ast>> = None;
        let mut source_run = None;
        let mut ordinary_prefix = true;
        for &comment in comments {
            if matches!(layout, CommentRunLayout::SourceDangling) {
                ordinary_prefix &= self.comment_semantics(comment).is_ordinary();
            }
            let fact = match array_comments.as_deref_mut() {
                Some(owner) => owner.consume(self, comment),
                None if matches!(layout, CommentRunLayout::SourceDangling) => {
                    self.prepare_source_comment(comment, &mut source_run)
                }
                None => None,
            };
            let prepared = self.prepare_comment(comment)?;
            let line_suffix = match (layout, comment.kind) {
                (
                    CommentRunLayout::SourceAfter(start) | CommentRunLayout::StatementTail(start),
                    CommentKind::Line,
                ) => {
                    let end = previous
                        .as_ref()
                        .map_or(start, |previous| previous.span.end);
                    self.line_breaks(end, comment.span.start) == 0
                }
                (CommentRunLayout::SourceDangling, CommentKind::Line) => {
                    previous.is_some()
                        && ordinary_prefix
                        && fact.is_some_and(PreparedLineFact::movable)
                }
                _ => false,
            };
            if let Some(previous) = previous.as_ref() {
                let nestled = previous.span.end == prepared.span.start
                    && previous.is_alignable()
                    && prepared.is_alignable();
                if matches!(layout, CommentRunLayout::StatementTail(_))
                    && previous.kind == CommentKind::Line
                {
                    // Flush the suffix and preserve the remaining source gap.
                    writer.hard_line();
                    if self.previous_line_is_empty(previous.span.end, prepared.span.start) {
                        writer.hard_line();
                    }
                } else if !nestled && !line_suffix {
                    match layout {
                        CommentRunLayout::Source
                        | CommentRunLayout::SourceDangling
                        | CommentRunLayout::SourceAfter(_)
                        | CommentRunLayout::StatementTail(_)
                        | CommentRunLayout::SourceLeading { .. } => self.comment_gap_with_packet(
                            previous.span.end,
                            prepared.span.start,
                            packet,
                            writer,
                        ),
                        CommentRunLayout::OwnLineTrailing
                            if previous.kind == CommentKind::Block
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
                        | CommentRunLayout::InlineLeading(_) => self
                            .leading_comment_gap_with_packet(
                                previous,
                                prepared.span.start,
                                !matches!(layout, CommentRunLayout::Leading(_)),
                                matches!(layout, CommentRunLayout::InlineLeading(_))
                                    && comments
                                        .first()
                                        .is_some_and(|first| first.span == previous.span),
                                packet,
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
            } else if let CommentRunLayout::SourceAfter(start)
            | CommentRunLayout::StatementTail(start) = layout
                && !line_suffix
            {
                self.comment_gap_with_packet_and_opening(
                    start,
                    prepared.span.start,
                    packet,
                    opening,
                    writer,
                );
            }
            if comment.kind == CommentKind::Line {
                writer.line_suffix_boundary();
            }
            if line_suffix {
                let opening = if previous.is_none() {
                    opening
                } else {
                    CommentOpening::Needed
                };
                self.format_comment_line_suffix_with_opening(comment, opening, writer);
            } else {
                prepared.text.write(writer);
            }
            previous = Some(prepared);
        }
        match (layout, previous) {
            (CommentRunLayout::SourceLeading { next, placement }, Some(previous)) => self
                .leading_comment_gap_with_placement(&previous, next, true, Some(placement), writer),
            (CommentRunLayout::Leading(next), Some(previous)) => {
                self.leading_comment_gap_with_packet(&previous, next, false, false, packet, writer)
            }
            (
                CommentRunLayout::GroupedLeading(next) | CommentRunLayout::InlineLeading(next),
                Some(previous),
            ) => self.leading_comment_gap_with_packet(
                &previous,
                next,
                true,
                matches!(layout, CommentRunLayout::InlineLeading(_))
                    && comments
                        .first()
                        .is_some_and(|first| first.span == previous.span),
                packet,
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

    /// `SourceAfter` defers inline line comments, but prints own-line comments directly.
    pub(super) fn comment_run_has_own_line_tail(&self, comments: &[Comment], start: u32) -> bool {
        let Some((last, previous)) = comments.split_last() else {
            return false;
        };
        last.kind == CommentKind::Line
            && self.line_breaks(
                previous.last().map_or(start, |comment| comment.span.end),
                last.span.start,
            ) != 0
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

    pub(super) fn line_suffix_following_gap(&self, end: u32) -> LineConstraint {
        let empty = self
            .comments
            .first()
            .is_some_and(|next| self.next_line_is_empty(end, next.span.start));
        LineConstraint::NewLine(if empty { 2 } else { 1 })
    }

    pub(super) fn format_trailing_comments(
        &mut self,
        span: Span,
        boundary: StatementCommentBoundary,
        has_following: bool,
        pending: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Option<LineConstraint>), FormatError> {
        if has_following
            && let Some(tail) = pending.as_ref()
            && self.optimizer_only_packet(tail.comments)
        {
            // A released marker follows the same next-head owner as a source
            // post-semicolon packet. That owner also places any required ASI guard.
            return Ok((tail.source_end, None));
        }
        let source_end = pending.as_ref().map_or(span.start, |tail| tail.source_end);
        let comments = self.comments.between(Span::new(source_end, span.end))?;
        let available = comments.len();
        let (comments, source_end) = if let Some(tail) = pending {
            // Keep the leased child tail and the parent's suffix in one packet:
            // a later block must not overtake a pending line suffix.
            if matches!(
                boundary.terminator,
                StatementTerminatorPlacement::BeforeTail
            ) && matches!(
                self.comment_binding(
                    tail.comments,
                    Span::new(tail.source_end, boundary.source_end),
                    None,
                ),
                CommentBinding::SourceBound
            ) {
                // The leased tail and the source suffix now share the same
                // post-terminator boundary as a reparsed statement packet.
                let prefix = self.trailing_comment_prefix_projected(
                    comments,
                    Span::new(tail.source_end, span.end),
                    tail.source_end,
                    has_following,
                    tail.comments.last().map(|last| ReleasedTailGap {
                        tail_end: last.span.end,
                        source_end: boundary.source_end,
                    }),
                );
                if prefix.is_empty() && has_following {
                    return Ok((tail.source_end, None));
                }
                let count = prefix.len().max(tail.comments.len());
                (&comments[..count], Some(tail.source_end))
            } else {
                let (inherited, following) = comments.split_at(tail.comments.len());
                let start = inherited
                    .last()
                    .map_or(span.start, |comment| span.start.max(comment.span.end));
                let following = self.trailing_comment_prefix(
                    following,
                    Span::new(start, span.end),
                    boundary.source_end,
                    has_following,
                );
                (
                    &comments[..inherited.len() + following.len()],
                    Some(tail.source_end),
                )
            }
        } else {
            (
                self.trailing_comment_prefix(comments, span, boundary.source_end, has_following),
                None,
            )
        };
        self.comments.claim(comments)?;
        if let Some(last) = comments.last() {
            if let Some(source_end) = source_end {
                self.format_comment_run(
                    comments,
                    CommentRunLayout::StatementTail(source_end),
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
            let following_gap = (source_end.is_some()
                && last.kind == CommentKind::Line
                && comments.len() < available)
                .then(|| self.line_suffix_following_gap(last.span.end));
            Ok((span.start.max(last.span.end), following_gap))
        } else {
            Ok((span.start, None))
        }
    }

    pub(super) fn format_inline_trailing_comments(
        &self,
        comments: &[Comment],
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_inline_trailing_comments_with_array_comments(comments, None, writer)
    }

    pub(super) fn format_inline_trailing_comments_with_opening(
        &self,
        comments: &[Comment],
        opening: CommentOpening,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_trailing_comment_run_with_opening(comments, None, opening, None, writer)
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
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_trailing_comment_run_with_opening(
            comments,
            previous,
            CommentOpening::Needed,
            array_comments,
            writer,
        )
    }

    fn format_trailing_comment_run_with_opening(
        &self,
        comments: &[Comment],
        previous: Option<Comment>,
        opening: CommentOpening,
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
                    CommentKind::Line => {
                        writer.line_suffix_boundary();
                        self.format_comment_line_suffix_with_opening(
                            comment,
                            if previous.is_none() {
                                opening
                            } else {
                                CommentOpening::Needed
                            },
                            writer,
                        );
                    }
                    CommentKind::Block => {
                        let nestled = previous.as_ref().is_some_and(|previous| {
                            previous.span.end == prepared.span.start
                                && previous.is_alignable()
                                && prepared.is_alignable()
                        });
                        if !nestled {
                            if previous.is_none() {
                                opening.write(writer);
                            } else {
                                writer.space();
                            }
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
        self.trailing_comment_prefix_projected(comments, span, statement_end, has_following, None)
    }

    fn trailing_comment_prefix_projected(
        &self,
        comments: &'ast [Comment],
        span: Span,
        statement_end: u32,
        has_following: bool,
        released_closer: Option<ReleasedTailGap>,
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
            let gap_start = released_closer
                .filter(|gap| gap.tail_end == comment.span.end && gap.source_end <= following)
                .map_or(comment.span.end, |gap| gap.source_end);
            if comment.kind == CommentKind::Line || self.line_breaks(gap_start, following) != 0 {
                count = index + 1;
                break;
            }
            end = gap_start;
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
