//! Shared source-ordered argument slots and comma/comment boundaries.

use super::callbacks::TestCallback;
use super::calls::CallCallee;
use super::typescript::ExpressionTypeArguments;
use super::{
    AccessKind, ExpressionContext, ExpressionPosition, Formatter, JsxCallbackContext, OperandIndent,
};
use crate::formatter::comments::{
    ArrayCommentOwner, CommentBinding, CommentRunLayout, ExpressionTail, NodeHead,
    PreparedTrailingRun, SourceSeparatorTransition,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint, TerminalCloseBoundary};
use crate::formatter::suppression::{
    ContentMode, DelimitedFollowing, DelimitedSuppression, SuppressionPosition,
};
use crate::{
    FormatError, TrailingCommas,
    document::{ConditionTarget, Writer},
};
use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrowFunctionExpression, CallExpression, Comment, CommentKind, Expr,
    ExprData, GetSpan, Span, SubRange, TemplateElement,
};

#[derive(Clone, Copy)]
pub(super) enum FollowingArgument {
    End,
    Argument(Argument),
    Elision {
        argument: Option<Argument>,
        holes: usize,
    },
}

pub(super) struct ArrayArgumentSlot {
    pub(super) argument: Argument,
    pub(super) following: FollowingArgument,
    pub(super) mode: ContentMode,
    pub(super) layout: ArgumentLayout,
    pub(super) transition: Option<SourceSeparatorTransition>,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum FollowingItem {
    End,
    /// The enclosing syntax prepared the real owner beyond this closing delimiter.
    ResolvedTerminal(DelimitedSuppression),
    Node(Span),
    ModuleSpecifier(Span),
    Elision {
        node: Option<Span>,
        holes: usize,
    },
}

impl FollowingItem {
    fn node(self) -> Option<Span> {
        match self {
            Self::End | Self::ResolvedTerminal(_) => None,
            Self::Node(span) | Self::ModuleSpecifier(span) => Some(span),
            Self::Elision { node, .. } => node,
        }
    }
}

/// The real terminal item owns its comma before the following comments.
#[derive(Clone, Copy)]
pub(in crate::formatter) enum TrailingCommaPolicy {
    Forbidden,
    IfBreak(ConditionTarget),
    Required,
}

/// The list's real closing owner also governs its optional terminal comma.
#[derive(Clone, Copy)]
pub(in crate::formatter) enum DelimitedClose {
    Source,
    Terminal(TerminalCloseBoundary),
    Fixed(LineConstraint),
}

impl DelimitedClose {
    pub(in crate::formatter) fn constraint(
        self,
        formatter: &Formatter<'_, '_>,
        end: u32,
        close: u32,
    ) -> LineConstraint {
        match self {
            Self::Source => formatter.line_constraint(end, close),
            Self::Terminal(boundary) => formatter.terminal_close_constraint(boundary, end),
            Self::Fixed(constraint) => constraint,
        }
    }
}

struct ItemComments<'item, 'ast> {
    head: CommentRunLayout,
    array: Option<&'item mut ArrayCommentOwner<'ast>>,
    /// The preceding boundary's certificate for borrowing the leading comment packet.
    separator_transition: Option<SourceSeparatorTransition>,
}

pub(in crate::formatter) struct PreparedItem {
    item_span: Span,
    following: FollowingItem,
    span: Span,
    pub(in crate::formatter) mode: ContentMode,
    has_leading_comments: bool,
    trailing_comma: TrailingCommaPolicy,
    suppression: Option<DelimitedSuppression>,
}

impl PreparedItem {
    pub(in crate::formatter) fn set_trailing_comma(&mut self, policy: TrailingCommaPolicy) {
        self.trailing_comma = policy;
    }

    pub(in crate::formatter) fn set_leading_mode(&mut self, mode: ContentMode) {
        self.mode = if mode == ContentMode::Verbatim
            || self
                .suppression
                .as_ref()
                .and_then(|source| source.preceding)
                .is_some_and(|target| target == self.item_span)
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
    }
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ArgumentLayout {
    Lines,
    Fill,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum Separator {
    Line,
    EmptyLine,
    OwnLineComment,
    TrailingLine,
    Source(LineConstraint),
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ElisionLayout {
    Formatted,
    /// The preceding real separator has already been emitted. Comments before
    /// it can still belong to this slot, so retain a separate token frontier.
    Located {
        comma: u32,
        separator: u32,
    },
    Source(usize),
}

struct PreparedElision {
    comma: u32,
    separator: u32,
    trailing: usize,
}

impl PreparedElision {
    fn layout(self) -> ElisionLayout {
        ElisionLayout::Located {
            comma: self.comma,
            separator: self.separator,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::formatter) struct ItemBoundary {
    pub(in crate::formatter) end: u32,
    pub(in crate::formatter) separator: Separator,
    pub(in crate::formatter) following_mode: ContentMode,
    pub(in crate::formatter) elisions: ElisionLayout,
    pub(in crate::formatter) transition: Option<SourceSeparatorTransition>,
}

impl ArgumentLayout {
    pub(super) fn entry<'a, T>(
        self,
        writer: &mut Writer<'a>,
        write: impl FnOnce(&mut Writer<'a>) -> T,
    ) -> T {
        match self {
            Self::Lines => write(writer),
            Self::Fill => writer.fill_entry(write),
        }
    }

    pub(in crate::formatter) fn separator(
        self,
        separator: Separator,
        constraint: LineConstraint,
        writer: &mut Writer<'_>,
    ) {
        self.entry(writer, |writer| {
            if let Separator::Source(constraint) = separator {
                constraint.write(writer, FlatSeparator::Space, Writer::line);
                return;
            }
            if matches!(separator, Separator::TrailingLine) {
                // Fill may choose a flat separator even in an expanded array.
                // A pending owned suffix must flush before the following item.
                writer.hard_line();
                return;
            }
            constraint.write(writer, FlatSeparator::Space, |writer| {
                match (self, separator) {
                    (Self::Fill, Separator::EmptyLine) => {
                        writer.hard_line();
                        writer.hard_line();
                    }
                    (Self::Fill, Separator::OwnLineComment) => writer.hard_line(),
                    (_, Separator::EmptyLine) => {
                        writer.line();
                        writer.soft_line();
                    }
                    _ => writer.line(),
                }
            })
        });
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_argument_item_with_array_comments(
        &mut self,
        slot: ArrayArgumentSlot,
        span: Span,
        trailing_comma: TrailingCommaPolicy,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let ArrayArgumentSlot {
            argument,
            following,
            mode,
            layout,
            transition,
        } = slot;
        let following = match following {
            FollowingArgument::Argument(argument) => FollowingItem::Node(argument.span(self.ast)),
            FollowingArgument::Elision { argument, holes } => FollowingItem::Elision {
                node: argument.map(|node| node.span(self.ast)),
                holes,
            },
            FollowingArgument::End => FollowingItem::End,
        };
        let mut item = self.prepare_delimited_item_with_comments(
            argument.span(self.ast),
            following,
            span,
            !matches!(trailing_comma, TrailingCommaPolicy::Forbidden),
            ItemComments {
                head: CommentRunLayout::Source,
                array: array_comments.as_deref_mut(),
                separator_transition: transition,
            },
            writer,
        )?;
        item.set_leading_mode(mode);
        item.set_trailing_comma(trailing_comma);
        let mut write = |writer: &mut Writer<'ast>| {
            self.format_argument_with_array_comments(
                argument,
                item.mode,
                array_comments.as_deref_mut(),
                writer,
            )
        };
        // Ordinary elements own their complete expression scope; their comma
        // and separator still observe the array. Concise fill owns its entries.
        let tail = match layout {
            ArgumentLayout::Lines => writer.group(false, write),
            ArgumentLayout::Fill => write(writer),
        }?;
        self.finish_delimited_item_with_array_comments(item, tail, array_comments, writer)
    }

    pub(in crate::formatter) fn prepare_delimited_item(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        transition: Option<SourceSeparatorTransition>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        self.prepare_delimited_item_with_comments(
            item_span,
            following,
            span,
            trailing_comma,
            ItemComments {
                head: CommentRunLayout::Source,
                array: None,
                separator_transition: transition,
            },
            writer,
        )
    }

    pub(in crate::formatter) fn prepare_call_argument_item(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        transition: Option<SourceSeparatorTransition>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        self.prepare_delimited_item_with_comments(
            item_span,
            following,
            span,
            trailing_comma,
            ItemComments {
                head: CommentRunLayout::GroupedLeading(item_span.start),
                array: None,
                separator_transition: transition,
            },
            writer,
        )
    }

    fn prepare_delimited_item_with_comments(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        comments: ItemComments<'_, 'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        let following_span = following.node();
        let suppression = if let FollowingItem::ResolvedTerminal(source) = following {
            Some(source)
        } else {
            self.delimited_suppression(
                item_span,
                following_span.map_or(span.end, |span| span.start),
                match following {
                    FollowingItem::End | FollowingItem::ResolvedTerminal(_) => {
                        DelimitedFollowing::End
                    }
                    FollowingItem::Node(_) => DelimitedFollowing::Node,
                    FollowingItem::ModuleSpecifier(_) => DelimitedFollowing::ModuleSpecifier,
                    FollowingItem::Elision { node: Some(_), .. } => DelimitedFollowing::Node,
                    FollowingItem::Elision { node: None, .. } => DelimitedFollowing::End,
                },
            )
        };
        let leading = self
            .comments
            .between(Span::new(span.start, item_span.start))?;
        let has_leading_comments = !leading.is_empty();
        let packet = (has_leading_comments
            && suppression.is_none()
            && self.content_mode(leading) == ContentMode::Formatted)
            .then(|| {
                self.source_separator_packet(leading, item_span, comments.separator_transition)
            })
            .flatten();
        let leading = self.comments.take(leading.len());
        let leading_mode = self.content_mode(leading);
        let mode = if leading_mode == ContentMode::Verbatim
            || suppression
                .as_ref()
                .and_then(|source| source.preceding)
                .is_some_and(|target| target == item_span)
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        let item = PreparedItem {
            item_span,
            following,
            span,
            mode,
            has_leading_comments,
            trailing_comma: if trailing_comma {
                TrailingCommaPolicy::IfBreak(ConditionTarget::Current)
            } else {
                TrailingCommaPolicy::Forbidden
            },
            suppression,
        };
        if !has_leading_comments {
            return Ok(item);
        }
        let source_head = matches!(comments.head, CommentRunLayout::Source)
            || !matches!(
                self.comment_binding(
                    leading,
                    Span::new(span.start, item_span.start),
                    Some(item_span),
                ),
                CommentBinding::Unbound
            );
        let head = NodeHead {
            comments: leading,
            content_start: item_span.start,
            layout: if source_head {
                CommentRunLayout::Source
            } else {
                comments.head
            },
            gap_end: source_head.then_some(item_span.start),
        };
        self.format_comment_run_with_source_packet(
            head.comments,
            head.layout,
            packet.as_ref(),
            comments.array,
            writer,
        )?;
        if let Some(end) = head.gap_end
            && let Some(last) = head.comments.last()
        {
            self.comment_gap_with_packet(last.span.end, end, packet.as_ref(), writer);
        }
        Ok(item)
    }

    fn diagnostic_separator_precedes_uncovered_item(
        &self,
        comma: u32,
        following_start: u32,
    ) -> bool {
        self.diagnostic_line_is_sensitive(comma)
            && !self.diagnostic_line_is_sensitive(following_start)
    }

    pub(in crate::formatter) fn finish_delimited_item(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        self.finish_delimited_item_with_array_comments(item, tail, None, writer)
    }

    pub(in crate::formatter) fn finish_delimited_item_with_closing(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
        closing: DelimitedClose,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        self.finish_delimited_item_with_boundary(item, tail, None, Some(closing), writer)
    }

    fn finish_delimited_item_with_array_comments(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        self.finish_delimited_item_with_boundary(item, tail, array_comments, None, writer)
    }

    fn finish_delimited_item_with_boundary(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        closing: Option<DelimitedClose>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let PreparedItem {
            item_span,
            following,
            span,
            mode,
            trailing_comma,
            suppression,
            ..
        } = item;
        let following_span = following.node();
        let has_more = !matches!(
            following,
            FollowingItem::End | FollowingItem::ResolvedTerminal(_)
        );
        let content_end = tail.as_ref().map_or(item_span.end, |tail| tail.source_end);
        let following_start = following_span.map(|span| span.start);
        let gap = Span::new(content_end, following_start.unwrap_or(span.end));
        let following_mode = suppression
            .as_ref()
            .map_or(ContentMode::Formatted, |source| source.following);
        let preceding = suppression.as_ref().and_then(|source| source.preceding);
        if matches!(following, FollowingItem::ResolvedTerminal(_))
            || preceding.is_some()
                && (following_span.is_some() || matches!(following, FollowingItem::Elision { .. }))
        {
            let gap = Span::new(item_span.end, gap.end);
            let own_line = tail.as_ref().is_some_and(|tail| {
                self.comment_run_has_own_line_tail(tail.comments, tail.source_end)
            });
            self.format_expression_tail_with_array_comments(
                tail,
                array_comments.as_deref_mut(),
                writer,
            )?;
            if own_line {
                writer.hard_line();
            }
            let source = self.format_delimited_source_gap(
                gap,
                following_span.map_or(DelimitedFollowing::End, |_| DelimitedFollowing::Node),
                writer,
            )?;
            return Ok(ItemBoundary {
                end: source.end,
                separator: Separator::Source(source.following),
                following_mode,
                transition: None,
                elisions: match following {
                    FollowingItem::Elision { holes, .. } => ElisionLayout::Source(holes),
                    _ => ElisionLayout::Formatted,
                },
            });
        }
        let comments = self.comments.between(gap)?;
        let trailing_comma = if !has_more
            && matches!(trailing_comma, TrailingCommaPolicy::IfBreak(_))
            && closing.is_some_and(|closing| {
                let end = comments
                    .last()
                    .map_or(content_end, |comment| comment.span.end);
                closing.constraint(self, end, span.end) == LineConstraint::SameLine
            }) {
            TrailingCommaPolicy::Forbidden
        } else {
            trailing_comma
        };
        let comma = self.find_comma(gap, comments);
        // Preserve existing operand coverage. A directive covering only this
        // separator does not constrain ordinary comment attachment, even when
        // moving the comma makes the directive cover the following operand.
        // Elisions and terminal boundaries retain their source ownership.
        let separator_only_coverage = mode == ContentMode::Formatted
            && suppression.is_none()
            && comma.is_some_and(|position| {
                matches!(following, FollowingItem::Node(next)
                    if self.diagnostic_separator_precedes_uncovered_item(position, next.start))
                    && !self.diagnostic_line_is_sensitive(item_span.end)
                    && !self.diagnostic_line_is_sensitive(content_end)
            });
        let constrained = !separator_only_coverage && self.diagnostic_range_is_sensitive(span);
        let elision = if matches!(following, FollowingItem::Elision { .. })
            && mode == ContentMode::Formatted
            && suppression.is_none()
            && !constrained
        {
            comma.and_then(|comma| self.prepare_elision_boundary(comments, gap, comma))
        } else {
            None
        };
        let preserve_comma =
            constrained || matches!(following, FollowingItem::Elision { .. }) && elision.is_none();
        // A directive-covered separator owns the comments on its source side.
        let protected_comma =
            comma.filter(|&position| constrained && self.diagnostic_line_is_sensitive(position));
        let trailing_limit = if let Some(elision) = &elision {
            elision.comma
        } else if matches!(following, FollowingItem::Elision { .. }) {
            comma.unwrap_or(gap.end)
        } else {
            gap.end
        };
        let ordinary_attachment = mode == ContentMode::Formatted
            && !constrained
            && (!matches!(following, FollowingItem::Elision { .. }) || elision.is_some());
        if comments.is_empty() && ordinary_attachment {
            let mut separator = Separator::Line;
            if has_more || matches!(trailing_comma, TrailingCommaPolicy::Required) {
                writer.text(",");
                if let Some(comma) = comma
                    && self.next_line_is_empty(comma + 1, gap.end)
                {
                    separator = Separator::EmptyLine;
                }
            } else if let TrailingCommaPolicy::IfBreak(target) = trailing_comma {
                writer.if_break_for(target, ",");
            }
            return Ok(ItemBoundary {
                end: content_end,
                separator,
                following_mode,
                elisions: elision.map_or(ElisionLayout::Formatted, PreparedElision::layout),
                transition: None,
            });
        }
        let mut transition = match following {
            FollowingItem::Node(next)
                if mode == ContentMode::Formatted
                    && suppression.is_none()
                    && !self.diagnostic_line_is_sensitive(item_span.end)
                    && !self.diagnostic_line_is_sensitive(content_end)
                    && !self.diagnostic_line_is_sensitive(next.start) =>
            {
                comma.map(|comma| SourceSeparatorTransition {
                    source: gap,
                    destination: next,
                    comma,
                    pre_packet_empty: false,
                })
            }
            _ => None,
        };
        let (trailing_count, physical_break, inline_line) = if let Some(elision) = &elision {
            let trailing = &comments[..elision.trailing];
            (
                trailing.len(),
                false,
                trailing
                    .last()
                    .is_some_and(|comment| comment.kind == CommentKind::Line),
            )
        } else if ordinary_attachment {
            let before_separator = comma.map_or(comments.len(), |comma| {
                comments.partition_point(|comment| comment.span.end <= comma)
            });
            let (trailing, leading) = self.split_separator_comments(
                comments,
                before_separator,
                content_end,
                following_start,
            );
            let end = trailing
                .last()
                .map_or(content_end, |comment| comment.span.end);
            let physical_break = leading
                .first()
                .is_some_and(|comment| self.line_breaks(end, comment.span.start) != 0);
            let inline_line = trailing
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line);
            (trailing.len(), physical_break, inline_line)
        } else {
            let mut inline_end = content_end;
            let mut trailing_count = 0;
            let mut physical_break = false;
            let mut inline_line = false;
            for (index, comment) in comments.iter().enumerate() {
                if comment.span.start >= trailing_limit {
                    break;
                }
                if self.line_breaks(inline_end, comment.span.start) != 0 {
                    physical_break = true;
                    break;
                }
                inline_end = comment.span.end;
                if comment.kind == CommentKind::Line {
                    trailing_count = index + 1;
                    inline_line = true;
                    break;
                }
                if following_span.is_none()
                    || comma.is_none_or(|position| comment.span.start < position)
                {
                    trailing_count = index + 1;
                }
            }
            (trailing_count, physical_break, inline_line)
        };
        let prepare_run = (physical_break || inline_line) && ordinary_attachment;
        let optimizer_run = (ordinary_attachment && elision.is_none())
            .then(|| self.prepare_delimited_optimizer_run(comments, tail.as_ref(), comma))
            .flatten();
        let prepared_run = optimizer_run.map(|run| (&comments[..0], run)).or_else(|| {
            if prepare_run {
                self.prepare_delimited_trailing_run(
                    if elision.is_some() {
                        &comments[..trailing_count]
                    } else {
                        comments
                    },
                    content_end,
                    trailing_limit,
                    following_span.is_some(),
                    array_comments.as_deref_mut(),
                )
                .map(PreparedTrailingRun::split_at_line)
            } else {
                None
            }
        });
        let canonical_run = prepared_run.is_some();
        let boundary_count = match prepared_run.as_ref() {
            Some((blocks, run)) => {
                debug_assert!(
                    blocks.len() + run.len() >= trailing_count,
                    "a canonical trailing run must include the complete legacy inline prefix",
                );
                blocks.len() + run.len()
            }
            None => comma
                .filter(|_| preserve_comma)
                .map_or(trailing_count, |comma| {
                    comments
                        .partition_point(|comment| comment.span.end <= comma)
                        .max(trailing_count)
                }),
        };
        let boundary_end = comments[..boundary_count]
            .last()
            .map_or(content_end, |comment| comment.span.end);
        // Resolve the complete leading packet's separator before claiming it.
        // A real post-comma blank belongs here, not in its internal comment gap.
        let before_packet_empty = transition.is_some()
            && ordinary_attachment
            && !canonical_run
            && comments
                .get(boundary_count)
                .is_some_and(|first| self.next_line_is_empty(boundary_end, first.span.start));
        let pre_packet_empty = before_packet_empty
            || comma.is_some_and(|comma| {
                let after_comma = comments.partition_point(|comment| comment.span.start <= comma);
                let next = comments
                    .get(boundary_count.max(after_comma))
                    .map_or(gap.end, |comment| comment.span.start);
                let start = (comma + 1).max(boundary_end);
                start < next && self.next_line_is_empty(start, next)
            });
        if let Some(transition) = transition.as_mut() {
            transition.pre_packet_empty = pre_packet_empty;
        }
        let boundary = self.comments.take(boundary_count);
        let (trailing, before_comma) = if let Some((blocks, _)) = &prepared_run {
            (*blocks, &boundary[..0])
        } else {
            boundary.split_at(trailing_count)
        };
        let trailing_end = trailing
            .last()
            .map_or(content_end, |comment| comment.span.end);
        self.format_inline_trailing_comments_with_array_comments(
            trailing,
            array_comments.as_deref_mut(),
            writer,
        )?;
        if !before_comma.is_empty() {
            self.format_comment_run_with_array_comments(
                before_comma,
                CommentRunLayout::SourceAfter(trailing_end),
                array_comments.as_deref_mut(),
                writer,
            )?;
        }
        // Fill must retain the leading packet's own-line attachment for blocks too.
        let mut separator = if comments[boundary_count..].iter().any(|comment| {
            comment.kind == CommentKind::Line || self.source_line_starts_line(comment.span.start)
        }) {
            Separator::OwnLineComment
        } else {
            Separator::Line
        };
        let required_comma = has_more
            || protected_comma.is_some()
            || matches!(trailing_comma, TrailingCommaPolicy::Required);
        if required_comma {
            if let Some(comma) = comma
                && let Some(last) = boundary.last()
                && last.span.end <= comma
            {
                self.line_constraint(last.span.end, comma).write(
                    writer,
                    FlatSeparator::Empty,
                    |writer| {
                        if before_comma
                            .last()
                            .is_some_and(|comment| comment.kind == CommentKind::Line)
                        {
                            writer.hard_line();
                        }
                    },
                );
            }
            writer.text(",");
            if pre_packet_empty {
                separator = Separator::EmptyLine;
            }
        } else if let TrailingCommaPolicy::IfBreak(target) = trailing_comma {
            writer.if_break_for(target, ",");
        }
        if let Some((_, run)) = prepared_run {
            // The next real separator owns its mandatory suffix termination.
            self.format_prepared_trailing_run(run, array_comments.as_deref_mut(), writer)?;
            if !matches!(separator, Separator::EmptyLine) {
                separator = Separator::TrailingLine;
            }
        }
        let mut end = boundary_end;
        if required_comma
            && let Some(comma) = comma
            && (preserve_comma || separator_only_coverage)
            && self
                .comments
                .first()
                .is_none_or(|comment| comment.span.start >= comma)
        {
            end = end.max(comma + 1);
        }
        if !has_more
            || !preserve_comma
                && following_span.is_none()
                && !matches!(following, FollowingItem::Elision { .. })
        {
            let own_line = self.comments.between(Span::new(end, span.end))?;
            let own_line = self.comments.take(own_line.len());
            if let Some(first) = own_line.first() {
                writer.hard_line();
                if self.previous_line_is_empty(end, first.span.start) {
                    writer.hard_line();
                }
                self.format_comment_run_with_array_comments(
                    own_line,
                    if own_line
                        .iter()
                        .any(|comment| comment.kind == CommentKind::Line)
                    {
                        CommentRunLayout::SourceDangling
                    } else {
                        CommentRunLayout::Lines
                    },
                    array_comments,
                    writer,
                )?;
            }
            if let Some(last) = own_line.last() {
                end = last.span.end;
            }
        }
        Ok(ItemBoundary {
            end,
            separator,
            following_mode,
            elisions: elision.map_or(ElisionLayout::Formatted, PreparedElision::layout),
            transition,
        })
    }

    fn prepare_elision_boundary(
        &self,
        comments: &[Comment],
        gap: Span,
        separator: u32,
    ) -> Option<PreparedElision> {
        let after_separator = comments.partition_point(|comment| comment.span.end <= separator);
        let comma = self.find_comma(
            Span::new(separator + 1, gap.end),
            &comments[after_separator..],
        )?;
        let before_hole = comments.partition_point(|comment| comment.span.end <= comma);
        let packet = &comments[..before_hole];
        if !packet
            .iter()
            .all(|&comment| self.comment_semantics(comment).is_ordinary())
        {
            return None;
        }
        // A hole is not an AST comment owner. An inline source run trails the
        // real element even when its comma anchors a new physical line.
        let mut trailing = 0;
        for comment in packet {
            if self.source_line_starts_line(comment.span.start) {
                break;
            }
            trailing += 1;
            if comment.kind == CommentKind::Line {
                break;
            }
        }
        // Leading comments before the separator and trailing comments after it
        // can have interleaved owners. Keep their physical comma as the boundary.
        if trailing < after_separator && after_separator < before_hole
            || packet
                .get(trailing.max(after_separator))
                .is_some_and(|comment| !self.source_line_starts_line(comment.span.start))
        {
            return None;
        }
        Some(PreparedElision {
            comma,
            separator,
            trailing,
        })
    }

    pub(in crate::formatter) fn format_array_elisions(
        &mut self,
        holes: usize,
        gap: Span,
        elisions: ElisionLayout,
        previous: Option<Separator>,
        layout: ArgumentLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Separator), FormatError> {
        if let ElisionLayout::Source(count) = elisions {
            debug_assert_eq!(
                count, holes,
                "a source elision boundary covers the prepared AST slots"
            );
            return Ok((gap.start, previous.unwrap_or(Separator::Line)));
        }
        let mut comments = self.comments.between(gap)?;
        let constrained = self.diagnostic_range_is_sensitive(gap);
        let mut end = gap.start;
        let mut separator = previous.unwrap_or(Separator::Line);
        let mut has_slot = previous.is_some();
        let mut located = match elisions {
            ElisionLayout::Located { comma, separator } => Some((comma, separator)),
            ElisionLayout::Formatted | ElisionLayout::Source(_) => None,
        };
        // Borrow the packet once and advance its comment and token frontiers
        // together. Dense arrays never enter this path.
        for _ in 0..holes {
            if constrained || !comments.is_empty() || located.is_some() {
                let relocated = located.take();
                let comma = match relocated
                    .map(|(comma, _)| comma)
                    .or_else(|| self.find_comma(Span::new(end, gap.end), comments))
                {
                    Some(comma) => comma,
                    None => {
                        return Err(FormatError::unsupported(
                            "array elision boundary",
                            None,
                            gap,
                        ));
                    }
                };
                let count = comments.partition_point(|comment| comment.span.end <= comma);
                let leading = self.comments.take(count);
                comments = &comments[count..];
                match (leading.first(), has_slot) {
                    (Some(first), _) => {
                        if has_slot {
                            let start = relocated
                                .map(|(_, separator)| separator)
                                .filter(|&separator| {
                                    end <= separator && separator < first.span.start
                                })
                                .map_or(end, |separator| separator + 1);
                            self.comment_gap(start, first.span.start, writer);
                        }
                        if !constrained
                            && leading
                                .iter()
                                .all(|&comment| self.comment_semantics(comment).is_ordinary())
                        {
                            self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
                            let last = leading.last().expect("a nonempty elision comment packet");
                            if let Some((_, source_separator)) = relocated
                                && last.span.end <= source_separator
                            {
                                // This packet preceded the old real separator.
                                // Its following blank belongs to the hole boundary,
                                // rather than to the removed comma inside the packet.
                                self.line_constraint(source_separator + 1, comma).write(
                                    writer,
                                    FlatSeparator::Empty,
                                    |writer| {
                                        if matches!(separator, Separator::EmptyLine) {
                                            writer.hard_line();
                                            writer.hard_line();
                                        } else if last.kind == CommentKind::Line {
                                            writer.hard_line();
                                        }
                                    },
                                );
                            } else {
                                self.prepare_leading_comment_gap(
                                    last.span, last.kind, comma, false,
                                )
                                .gap
                                .write(
                                    writer,
                                    FlatSeparator::Empty,
                                    |_| {},
                                );
                            }
                        } else {
                            self.format_comment_run(
                                leading,
                                CommentRunLayout::Leading(comma),
                                writer,
                            )?;
                        }
                    }
                    (None, true) => {
                        layout.separator(separator, self.line_constraint(end, comma), writer)
                    }
                    // The array opener already owns this physical gap.
                    (None, false) => {}
                }
                writer.text(",");
                end = comma + 1;
            } else {
                if has_slot {
                    layout.separator(separator, LineConstraint::Unconstrained, writer);
                }
                writer.text(",");
            }
            separator = Separator::Line;
            has_slot = true;
        }
        Ok((end, separator))
    }
}

/// An actual source argument list and its borrowed pre-opener comment owner.
pub(super) struct SourceArguments<'ast> {
    arguments: SubRange<Argument>,
    first: Option<SourceFirstArgument>,
    delimiters: Span,
    closing: TerminalCloseBoundary,
    pub(super) callee: Expr,
    callee_span: Span,
    type_arguments: Option<ExpressionTypeArguments<'ast>>,
    source_opening: Option<SourceArgumentOpening>,
    prefix: SourceArgumentPrefix<'ast>,
    pub(super) callee_mode: ContentMode,
    /// An ignore inside an empty list owns the complete call, not an argument.
    pub(super) call_mode: ContentMode,
    pub(super) group: ArgumentGroup,
    pub(super) frame: CallArgumentFrame,
    pub(super) curried_callee: Option<CallExpression>,
    pub(super) allow_composition: bool,
    pub(super) jsx_call: Option<CallExpression>,
    pub(super) binary_indent: OperandIndent,
}

struct ArgumentPunctuation {
    operator: Option<Span>,
    opening: u32,
}

fn argument_opening_gap(prefix_start: u32, first: Option<Span>, end: u32) -> Span {
    Span::new(prefix_start, first.map_or(end - 1, |span| span.start))
}

#[derive(Clone, Copy)]
struct SourceFirstArgument {
    node: Argument,
    mode: ContentMode,
}

struct CallArgumentSlot {
    node: Argument,
    following: Option<Argument>,
}

pub(super) struct ArgumentHead {
    pub(super) first_mode: ContentMode,
    pub(super) source_opening: Option<SourceArgumentOpening>,
    pub(super) ordinary_group: ArgumentGroup,
    pub(super) jsx_call: Option<CallExpression>,
    pub(super) independent_callback: Option<ArrowFunctionExpression>,
    pub(super) binary_indent: OperandIndent,
}

#[derive(Clone, Copy)]
pub(super) struct SourceArgumentOpening {
    pub(super) span: Span,
    pub(super) following: LineConstraint,
}

#[derive(Clone, Copy)]
pub(super) enum CallArgumentFrame {
    Ordinary,
    /// The selected block arrow retains its complete signature's own groups.
    DecoratedFunction(ArrowFunctionExpression),
    Fixed {
        role: CallFrameRole,
        closing: LineConstraint,
    },
}

#[derive(Clone, Copy)]
pub(super) enum CallFrameRole {
    Module,
    TestCallback(TestCallback),
    TestWrapper {
        expression: Expr,
        call: CallExpression,
    },
    AngularWrapper,
    MultilineTemplate,
    CallbackDependencies,
}

impl CallArgumentFrame {
    pub(super) fn fixed(role: CallFrameRole) -> Self {
        Self::Fixed {
            role,
            closing: LineConstraint::Unconstrained,
        }
    }

    fn is_fixed(self) -> bool {
        matches!(self, Self::Fixed { .. })
    }

    fn independent_callback(self) -> Option<ArrowFunctionExpression> {
        match self {
            Self::DecoratedFunction(callback) => Some(callback),
            Self::Ordinary | Self::Fixed { .. } => None,
        }
    }

    fn angular_wrapper(self) -> Option<(Expr, CallExpression)> {
        match self {
            Self::Fixed {
                role: CallFrameRole::TestWrapper { expression, call },
                ..
            } => Some((expression, call)),
            _ => None,
        }
    }

    fn test_callback(self) -> Option<TestCallback> {
        match self {
            Self::Fixed {
                role: CallFrameRole::TestCallback(callback),
                ..
            } => Some(callback),
            _ => None,
        }
    }

    fn separator(self, separator: Separator, constraint: LineConstraint, writer: &mut Writer<'_>) {
        match self {
            Self::Ordinary | Self::DecoratedFunction(_) => {
                ArgumentLayout::Lines.separator(separator, constraint, writer)
            }
            Self::Fixed { .. } => match separator {
                Separator::Source(source) => {
                    source.write(writer, FlatSeparator::Space, Writer::line);
                }
                Separator::OwnLineComment | Separator::TrailingLine => writer.hard_line(),
                _ => constraint.write(writer, FlatSeparator::Space, Writer::space),
            },
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ArgumentGroup {
    Local,
    ParentCall,
}

impl ArgumentGroup {
    pub(super) fn write<'a, T>(
        self,
        expand: bool,
        writer: &mut Writer<'a>,
        write: impl FnOnce(&mut Writer<'a>) -> T,
    ) -> T {
        match (self, expand) {
            (Self::ParentCall, false) => write(writer),
            _ => writer.group(expand, write),
        }
    }
}

impl<'ast> SourceArguments<'ast> {
    pub(super) fn arguments(&self) -> SubRange<Argument> {
        self.arguments
    }

    /// An empty call has no first argument that could own this ordinary packet.
    pub(super) fn empty_call_head(&self) -> Option<NodeHead<'ast>> {
        if !self.arguments.is_empty()
            || self.type_arguments.is_some()
            || self.callee_mode != ContentMode::Formatted
            || self.call_mode != ContentMode::Formatted
            || self.frame.is_fixed()
        {
            return None;
        }
        let comments = match &self.prefix {
            SourceArgumentPrefix::Empty { comments, .. } => *comments,
            SourceArgumentPrefix::Direct(prefix) if prefix.comments().is_empty() => {
                prefix.comments()
            }
            SourceArgumentPrefix::Optional { before, after, .. }
                if before.is_empty() && after.comments().is_empty() =>
            {
                after.comments()
            }
            _ => return None,
        };
        Some(NodeHead {
            comments,
            content_start: self.delimiters.start,
            layout: CommentRunLayout::GroupedLeading(self.delimiters.start),
            gap_end: None,
        })
    }

    /// Move the prepared packet without claiming it; the chain keeps its source order.
    pub(super) fn take_empty_call_head(&mut self) -> Option<NodeHead<'ast>> {
        let head = self.empty_call_head()?;
        let operator = match self.prefix {
            SourceArgumentPrefix::Empty { operator, .. } => operator,
            SourceArgumentPrefix::Optional { operator, .. } => Some(operator),
            SourceArgumentPrefix::Direct(_) => None,
        };
        self.prefix = match operator {
            Some(operator) => SourceArgumentPrefix::Optional {
                operator,
                before: &[],
                after: ArgumentPrefix::Outside(&[]),
            },
            None => SourceArgumentPrefix::Direct(ArgumentPrefix::Outside(&[])),
        };
        Some(head)
    }

    pub(super) fn empty_argument_content(&self) -> Option<Span> {
        (self.arguments.is_empty() && self.call_mode == ContentMode::Formatted)
            .then(|| Span::new(self.delimiters.start + 1, self.delimiters.end - 1))
    }

    pub(super) fn access(&self) -> AccessKind {
        if let Some(types) = &self.type_arguments {
            return types.access;
        }
        match self.prefix {
            SourceArgumentPrefix::Direct(_) => AccessKind::Ordinary,
            SourceArgumentPrefix::Optional { .. } => AccessKind::Optional,
            SourceArgumentPrefix::Empty { operator, .. } => {
                if operator.is_some() {
                    AccessKind::Optional
                } else {
                    AccessKind::Ordinary
                }
            }
        }
    }

    /// The selected argument frame preserves the callee's independent break.
    pub(super) fn has_fixed_argument_frame(&self) -> bool {
        self.frame.is_fixed()
    }

    pub(super) fn has_outside_prefix(&self) -> bool {
        if self
            .type_arguments
            .as_ref()
            .is_some_and(ExpressionTypeArguments::has_comments)
        {
            return true;
        }
        match &self.prefix {
            SourceArgumentPrefix::Direct(prefix) => {
                !prefix.belongs_to_arguments() && !prefix.comments().is_empty()
            }
            SourceArgumentPrefix::Optional { before, after, .. } => {
                !before.is_empty() || !after.belongs_to_arguments() && !after.comments().is_empty()
            }
            SourceArgumentPrefix::Empty { .. } => true,
        }
    }

    pub(super) fn callee_head(&self) -> CallCallee {
        CallCallee {
            expression: self.callee,
            access: self.access(),
            curried_call: self.curried_callee,
            mode: if self.call_mode == ContentMode::Verbatim {
                self.call_mode
            } else {
                self.callee_mode
            },
        }
    }

    pub(super) fn callee_context(
        &self,
        context: ExpressionContext<'ast>,
    ) -> ExpressionContext<'ast> {
        self.callee_head().context(context)
    }
}

/// Facts needed only while recording fluent-chain arguments. Source-dependent
/// collection policy stays in the language builder, outside document fitting.
#[derive(Default)]
pub(super) struct ChainArgumentFacts {
    pub(super) has_function: bool,
    /// An argument contains source-expanded objects without intrinsic newlines.
    pub(super) source_expanded_object: bool,
}

/// The actual prefix owner reports its emitted boundary after receiving a tail.
/// Its ordinary caller only needs the tail; a chain also consumes the layout fact.
pub(super) struct FormattedArguments<'ast> {
    pub(super) tail: Option<ExpressionTail<'ast>>,
    pub(super) has_outside_prefix: bool,
}

enum ArgumentPrefix<'ast> {
    Outside(&'ast [Comment]),
    Arguments(&'ast [Comment]),
}

impl<'ast> ArgumentPrefix<'ast> {
    fn comments(&self) -> &'ast [Comment] {
        match self {
            Self::Outside(comments) | Self::Arguments(comments) => comments,
        }
    }

    fn belongs_to_arguments(&self) -> bool {
        matches!(self, Self::Arguments(_))
    }
}

enum SourceArgumentPrefix<'ast> {
    Direct(ArgumentPrefix<'ast>),
    Optional {
        operator: Span,
        before: &'ast [Comment],
        after: ArgumentPrefix<'ast>,
    },
    Empty {
        operator: Option<Span>,
        comments: &'ast [Comment],
    },
}

impl SourceArgumentPrefix<'_> {
    fn has_comments(&self) -> bool {
        match self {
            Self::Direct(prefix) => !prefix.comments().is_empty(),
            Self::Optional { before, after, .. } => {
                !before.is_empty() || !after.comments().is_empty()
            }
            Self::Empty { .. } => true,
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn source_arguments(
        &self,
        arguments: SubRange<Argument>,
        callee: Expr,
        end: u32,
        access: AccessKind,
    ) -> Option<SourceArguments<'ast>> {
        self.prepare_source_arguments(arguments, callee, end, access, None)
    }

    pub(super) fn source_arguments_after_types(
        &self,
        arguments: SubRange<Argument>,
        callee: Expr,
        end: u32,
        types: ExpressionTypeArguments<'ast>,
    ) -> Option<SourceArguments<'ast>> {
        self.prepare_source_arguments(arguments, callee, end, AccessKind::Ordinary, Some(types))
    }

    /// The enclosing chain has already proved that this argument boundary is
    /// comment- and diagnostic-clear. Preserve the actual source token geometry.
    pub(super) fn source_clear_argument_opening(
        &self,
        arguments: SubRange<Argument>,
        callee: Expr,
        end: u32,
        access: AccessKind,
    ) -> bool {
        let ast = self.ast;
        let first = arguments.first().map(|id| {
            let argument: Argument = ast.get_node_in_sub_range(id);
            argument.span(ast)
        });
        let gap = argument_opening_gap(callee.span(ast).end, first, end);
        self.source_argument_punctuation(gap, access, &[]).is_some()
    }

    /// A packet moves into an empty call only if that prefix can own it again.
    pub(super) fn empty_call_prefix_can_relocate(
        &self,
        callee: Expr,
        opening: u32,
        comments: &[Comment],
    ) -> bool {
        let callee_start = callee.span(self.ast).start;
        let start = comments
            .first()
            .map_or(callee_start, |comment| callee_start.min(comment.span.start));
        self.comment_binding(comments, Span::new(start, opening + 1), None)
            .allows_relocation(None)
            && self.source_cast_head(callee).is_none()
    }

    fn source_argument_punctuation(
        &self,
        gap: Span,
        access: AccessKind,
        comments: &[Comment],
    ) -> Option<ArgumentPunctuation> {
        let operator = match access {
            AccessKind::Ordinary => None,
            AccessKind::Optional => {
                let start = self.find_token(gap, comments, "?.")?;
                Some(Span::new(start, start + 2))
            }
        };
        let opening_start = operator.map_or(gap.start, |operator| operator.end);
        let opening_gap = Span::new(opening_start, gap.end);
        let opening = self.find_token(opening_gap, comments, "(")?;
        Some(ArgumentPunctuation { operator, opening })
    }

    fn prepare_source_arguments(
        &self,
        arguments: SubRange<Argument>,
        callee: Expr,
        end: u32,
        access: AccessKind,
        mut type_arguments: Option<ExpressionTypeArguments<'ast>>,
    ) -> Option<SourceArguments<'ast>> {
        let ast = self.ast;
        let callee_span = callee.span(ast);
        let prefix_start = type_arguments
            .as_ref()
            .map_or(callee_span.end, |types| types.node.span(ast).end);
        let mut first = arguments.iter().next().map(|id| SourceFirstArgument {
            node: ast.get_node_in_sub_range(id),
            mode: ContentMode::Formatted,
        });
        let first_span = first.map(|argument| argument.node.span(ast));
        let gap = argument_opening_gap(prefix_start, first_span, end);
        let boundary_comments = self.comments.in_range(gap);
        let ArgumentPunctuation { operator, opening } =
            self.source_argument_punctuation(gap, access, boundary_comments)?;
        let prefix_count = boundary_comments.partition_point(|comment| comment.span.end <= opening);
        let prefix_comments = &boundary_comments[..prefix_count];
        let (prefix, mut callee_mode) = match operator {
            None => {
                let span = Span::new(prefix_start, opening);
                let (prefix, mode) = self.argument_prefix(first.as_mut(), span, prefix_comments);
                (SourceArgumentPrefix::Direct(prefix), mode)
            }
            Some(operator) => {
                let before_count =
                    prefix_comments.partition_point(|comment| comment.span.end <= operator.start);
                let (before, after) = prefix_comments.split_at(before_count);
                let before_span = Span::new(prefix_start, operator.start);
                self.prepare_first_argument_suppression(first.as_mut(), before, operator.start);
                let before_mode = self.trailing_content_mode(before_span, false);
                let after_span = Span::new(operator.end, opening);
                let (after, after_mode) = self.argument_prefix(first.as_mut(), after_span, after);
                if !after.belongs_to_arguments()
                    && self.content_mode(after.comments()) == ContentMode::Verbatim
                    && (first.is_some() || after_mode != ContentMode::Verbatim)
                {
                    return None;
                }
                let mode = if before_mode == ContentMode::Verbatim
                    || after_mode == ContentMode::Verbatim
                {
                    ContentMode::Verbatim
                } else {
                    ContentMode::Formatted
                };
                let prefix = SourceArgumentPrefix::Optional {
                    operator,
                    before,
                    after,
                };
                (prefix, mode)
            }
        };
        let prefix =
            if first.is_none() && prefix.has_comments() && callee_mode == ContentMode::Formatted {
                let comments = prefix_comments;
                if !comments.is_empty()
                    && self.empty_call_prefix_can_relocate(
                        callee,
                        opening,
                        self.comments
                            .in_range(Span::new(callee_span.start, opening)),
                    )
                {
                    SourceArgumentPrefix::Empty { operator, comments }
                } else {
                    prefix
                }
            } else {
                prefix
            };
        let mut source_opening = None;
        if let Some(types) = &mut type_arguments {
            let empty_prefix_mode = if first.is_none() && self.has_suppression {
                self.content_mode(prefix_comments)
            } else {
                ContentMode::Formatted
            };
            if callee_mode == ContentMode::Verbatim || empty_prefix_mode == ContentMode::Verbatim {
                types.mode = ContentMode::Verbatim;
            }
            if self.has_suppression
                && let Some(first) = &mut first
            {
                let inside = &boundary_comments[prefix_count..];
                let mut trailing_types = false;
                for position in self.suppression_positions(inside, gap.end) {
                    match position {
                        SuppressionPosition::EndOfLine => {
                            types.mode = ContentMode::Verbatim;
                            trailing_types = true;
                        }
                        SuppressionPosition::OwnLine | SuppressionPosition::Inline => {
                            first.mode = ContentMode::Verbatim;
                        }
                    }
                }
                if trailing_types && let Some(last) = inside.last() {
                    // The actual angle owner precedes this end-of-line packet.
                    // Carry only the opener and leading comments; argument values
                    // and their selected layouts keep their independent modes.
                    source_opening = Some(SourceArgumentOpening {
                        span: Span::new(opening, last.span.end),
                        following: self.source_line_constraint(last.span.end, gap.end),
                    });
                }
            }
            callee_mode = types.callee_mode;
        }
        let content = Span::new(opening + 1, end - 1);
        let closing = self.terminal_close_boundary(content, || {
            let mut arguments = arguments.iter().rev();
            let last: Argument = ast.get_node_in_sub_range(arguments.next()?);
            let head_start = arguments.next().map_or(content.start, |id| {
                let argument: Argument = ast.get_node_in_sub_range(id);
                argument.span(ast).end
            });
            let last = match ast.argument_data(last) {
                ArgumentData::Expr(expression) => self.terminal_argument_content_span(expression),
                ArgumentData::SpreadElement(_) => last.span(ast),
            };
            Some((last, head_start))
        });
        let mut list = SourceArguments {
            arguments,
            first,
            delimiters: Span::new(opening, end),
            closing,
            callee,
            callee_span,
            type_arguments,
            source_opening,
            prefix,
            callee_mode,
            call_mode: if first.is_none() {
                self.content_mode(&boundary_comments[prefix_count..])
            } else {
                ContentMode::Formatted
            },
            group: ArgumentGroup::Local,
            frame: CallArgumentFrame::Ordinary,
            allow_composition: true,
            curried_callee: None,
            jsx_call: None,
            binary_indent: OperandIndent::Expression,
        };
        list.frame = self.structural_argument_frame(&list);
        Some(list)
    }

    pub(super) fn format_argument_with_chain_facts(
        &mut self,
        argument: Argument,
        mode: ContentMode,
        binary_indent: OperandIndent,
        jsx_call: Option<CallExpression>,
        chain_facts: &mut Option<&mut ChainArgumentFacts>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if binary_indent == OperandIndent::Expression
            && chain_facts.is_none()
            && (jsx_call.is_none() || mode == ContentMode::Verbatim)
        {
            return self.format_argument(argument, mode, writer);
        }
        let data = self.ast.argument_data(argument);
        let content = match data {
            ArgumentData::Expr(expression) => Some(self.expression_content(expression)),
            ArgumentData::SpreadElement(_) => None,
        };
        let has_function = matches!(
            content.as_ref().map(|content| content.data),
            Some(ExprData::Function(_) | ExprData::ArrowFunctionExpression(_))
        );
        if let Some(facts) = chain_facts.as_deref_mut() {
            facts.has_function |= has_function;
        }
        let jsx_callback = content
            .as_ref()
            .map_or(JsxCallbackContext::Ordinary, |content| {
                JsxCallbackContext::argument(
                    jsx_call.filter(|_| mode == ContentMode::Formatted),
                    content.data,
                )
            });
        let operand_indent = match content.as_ref().map(|content| content.data) {
            Some(
                ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::ParenthesizedExpression(_),
            ) => binary_indent,
            _ => OperandIndent::Expression,
        };
        let context = ExpressionContext::at(ExpressionPosition::Nested)
            .with_operand_indent(operand_indent)
            .with_content_mode(mode)
            .with_arrow_layout(super::arrows::ArrowLayout::Argument)
            .with_jsx_callback(jsx_callback);
        let source_generation = self.source_object_layout.generation;
        let write = |writer: &mut Writer<'ast>| match data {
            ArgumentData::Expr(expression) => self.format_expression_prepared_with_array_comments(
                expression, content, context, None, writer,
            ),
            ArgumentData::SpreadElement(_) => {
                self.format_argument_data_with_array_comments(data, mode, None, writer)
            }
        };
        if let Some(facts) = chain_facts.as_deref_mut() {
            let (tail, intrinsic_lines) = writer.write_with_intrinsic_lines(write)?;
            // Required body/comment breaks must not become source-only facts
            // when the formatted argument is parsed for a second pass.
            facts.source_expanded_object |=
                !intrinsic_lines && source_generation != self.source_object_layout.generation;
            Ok(tail)
        } else {
            write(writer)
        }
    }

    fn argument_prefix(
        &self,
        first: Option<&mut SourceFirstArgument>,
        span: Span,
        comments: &'ast [Comment],
    ) -> (ArgumentPrefix<'ast>, ContentMode) {
        let Some(first) = first else {
            return (
                ArgumentPrefix::Outside(comments),
                self.trailing_content_mode(span, false),
            );
        };
        let first_span = first.node.span(self.ast);
        let last_line_break = if self.has_suppression && !comments.is_empty() {
            self.source(Span::new(span.start, first_span.start))
                .rfind(['\n', '\r', '\u{2028}', '\u{2029}'])
                .map(|offset| span.start + offset as u32)
        } else {
            None
        };
        let mut callee_mode = ContentMode::Formatted;
        let mut first_mode = ContentMode::Formatted;
        for (run, position) in self.suppressed_comment_runs(comments, span.end) {
            // The complete opening gap decides the target before relocation.
            // A same-line marker followed by a source line break trails the callee,
            // including when the break is inside the argument opening packet.
            if !matches!(position, SuppressionPosition::OwnLine)
                && last_line_break.is_some_and(|end| run[run.len() - 1].span.end <= end)
            {
                callee_mode = ContentMode::Verbatim;
            } else {
                first_mode = ContentMode::Verbatim;
            }
        }
        if first_mode == ContentMode::Verbatim {
            first.mode = first_mode;
        }
        let target = self.has_suppression.then_some(first_span);
        let boundary = Span::new(span.start, span.end + 1);
        let binding = if first_mode == ContentMode::Verbatim {
            // An opaque argument carries the complete annotation packet with it.
            // Line directives still retain their source boundary.
            let binding = self.comment_run_binding(comments.iter().copied(), |comment| {
                let semantics = self.comment_semantics(comment);
                if semantics.diagnostic || semantics.file_directive {
                    CommentBinding::SourceBound
                } else if semantics.is_ordinary() {
                    CommentBinding::Unbound
                } else {
                    CommentBinding::BoundTo(first_span)
                }
            });
            if self.diagnostic_range_is_sensitive(boundary) {
                CommentBinding::SourceBound
            } else {
                binding
            }
        } else {
            self.comment_binding(comments, boundary, target)
        };
        let outside_line = comments.iter().any(|comment| {
            comment.kind == CommentKind::Line
                && (!self.source_line_starts_line(comment.span.start)
                    || !matches!(binding, CommentBinding::Unbound))
        });
        let prefix = if comments.is_empty()
            || callee_mode == ContentMode::Verbatim
            || outside_line
            || !binding.allows_relocation(target)
        {
            ArgumentPrefix::Outside(comments)
        } else {
            ArgumentPrefix::Arguments(comments)
        };
        (prefix, callee_mode)
    }

    fn prepare_first_argument_suppression(
        &self,
        first: Option<&mut SourceFirstArgument>,
        comments: &[Comment],
        following_start: u32,
    ) {
        if !self.has_suppression {
            return;
        }
        let Some(first) = first else {
            return;
        };
        // Relocation and consumption are separate: an own-line marker before
        // the opener still targets this actual argument when it stays outside.
        if self
            .suppression_positions(comments, following_start)
            .any(|position| matches!(position, SuppressionPosition::OwnLine))
        {
            first.mode = ContentMode::Verbatim;
        }
    }

    fn format_empty_argument_prefix(
        &self,
        comments: &'ast [Comment],
        operator: Option<Span>,
        source_end: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous_end = source_end;
        let mut pending_line = false;
        for comment in comments {
            let own_line = self.source_position_starts_line(comment.span.start);
            let gap_start = operator
                .filter(|operator| {
                    previous_end <= operator.start && operator.end <= comment.span.start
                })
                .map_or(previous_end, |operator| operator.end);
            if pending_line || own_line {
                writer.hard_line();
                if self.next_line_is_empty(gap_start, comment.span.start) {
                    writer.hard_line();
                }
            } else {
                writer.space();
            }
            self.format_comment_run(
                std::slice::from_ref(comment),
                CommentRunLayout::Source,
                writer,
            )?;
            pending_line = own_line || comment.kind == CommentKind::Line;
            previous_end = comment.span.end;
        }
        if pending_line {
            writer.hard_line();
        }
        Ok(())
    }

    fn structural_argument_frame(&self, list: &SourceArguments<'ast>) -> CallArgumentFrame {
        let Some(first) = list.first else {
            return CallArgumentFrame::Ordinary;
        };
        if list.source_opening.is_some()
            || list.callee_mode != ContentMode::Formatted
            || first.mode != ContentMode::Formatted
        {
            return CallArgumentFrame::Ordinary;
        }
        let ast = self.ast;
        if list.arguments.len() == 1 {
            let ArgumentData::Expr(expression) = ast.argument_data(first.node) else {
                return CallArgumentFrame::Ordinary;
            };
            let content = Span::new(list.delimiters.start + 1, list.delimiters.end - 1);
            if self.sole_multiline_template(expression, content) {
                return CallArgumentFrame::fixed(CallFrameRole::MultilineTemplate);
            }
        } else if self.callback_dependency_arguments(list.arguments)
            && self.argument_boundaries_are_clear(list)
        {
            return CallArgumentFrame::fixed(CallFrameRole::CallbackDependencies);
        }
        CallArgumentFrame::Ordinary
    }

    /// Raw template quasis retain their source lines; interpolation documents
    /// keep their independent groups inside the fixed argument frame.
    pub(super) fn sole_multiline_template(&self, expression: Expr, content: Span) -> bool {
        let ast = self.ast;
        let prepared = self.expression_content(expression);
        let template = match prepared.data {
            ExprData::TemplateLiteral(template) => template,
            ExprData::TaggedTemplateExpression(tagged) => tagged.quasi(ast),
            _ => return false,
        };
        if self.source_position_starts_line(prepared.expression.span(ast).start) {
            return false;
        }
        let multiline = template.quasis(ast).iter().any(|id| {
            let quasi: TemplateElement = ast.get_node_in_sub_range(id);
            ast.get_utf8(quasi.raw(ast))
                .bytes()
                .any(|byte| matches!(byte, b'\n' | b'\r'))
        });
        if !multiline || self.diagnostic_range_is_sensitive(content) {
            return false;
        }
        let span = prepared.expression.span(ast);
        self.comments
            .in_range(Span::new(content.start, span.start))
            .is_empty()
            && self
                .comments
                .in_range(Span::new(span.end, content.end))
                .is_empty()
    }

    /// Only the list boundaries belong to the call frame. Comments inside an
    /// argument (including a callback body) remain that expression's concern.
    pub(super) fn argument_boundaries_are_clear(&self, list: &SourceArguments<'ast>) -> bool {
        if list.source_opening.is_some()
            || list.callee_mode != ContentMode::Formatted
            || list
                .first
                .is_some_and(|first| first.mode != ContentMode::Formatted)
            || list.has_outside_prefix()
        {
            return false;
        }
        let content = Span::new(list.callee_span.end, list.delimiters.end - 1);
        if self.diagnostic_range_is_sensitive(content) {
            return false;
        }
        let mut previous = content.start;
        for id in list.arguments.iter() {
            let argument: Argument = self.ast.get_node_in_sub_range(id);
            let span = argument.span(self.ast);
            if !self
                .comments
                .in_range(Span::new(previous, span.start))
                .is_empty()
            {
                return false;
            }
            previous = span.end;
        }
        self.comments
            .in_range(Span::new(previous, content.end))
            .is_empty()
    }

    /// The module frame owns delimiters; each argument retains its document
    /// and source/comment boundaries, including an AMD dependency array.
    pub(super) fn module_argument_frame(
        &self,
        list: &SourceArguments<'ast>,
        commented_first: bool,
    ) -> CallArgumentFrame {
        let Some(first) = list.first else {
            return CallArgumentFrame::Ordinary;
        };
        if list.source_opening.is_some()
            || list.callee_mode == ContentMode::Verbatim
            || first.mode == ContentMode::Verbatim
        {
            return CallArgumentFrame::Ordinary;
        }
        let ast = self.ast;
        if !commented_first {
            if matches!(&list.prefix, SourceArgumentPrefix::Direct(prefix)
            if prefix.belongs_to_arguments() && !prefix.comments().is_empty())
            {
                return CallArgumentFrame::Ordinary;
            }
            let span = first.node.span(ast);
            if !self
                .comments
                .in_range(Span::new(list.delimiters.start + 1, span.start))
                .is_empty()
            {
                return CallArgumentFrame::Ordinary;
            }
            let mut remaining = list.arguments;
            let following = remaining
                .split_off(1)
                .first()
                .map(|id| ast.get_node_in_sub_range(id));
            let following_start = following.map(|argument: Argument| argument.span(ast).start);
            let gap = Span::new(span.end, following_start.unwrap_or(list.delimiters.end - 1));
            let trailing = self.comments.in_range(gap);
            if !trailing.is_empty() {
                let comma = self.find_comma(gap, trailing);
                let before_separator = comma.map_or(trailing.len(), |comma| {
                    trailing.partition_point(|comment| comment.span.end <= comma)
                });
                let (owned, _) = self.split_separator_comments(
                    trailing,
                    before_separator,
                    span.end,
                    following_start,
                );
                if following.is_none() || !owned.is_empty() {
                    return CallArgumentFrame::Ordinary;
                }
            }
        }
        let mut closing = LineConstraint::Unconstrained;
        if !self.ast.comments().is_empty()
            && let Some(last) = list.arguments.last()
        {
            let last: Argument = ast.get_node_in_sub_range(last);
            let trailing = self
                .comments
                .in_range(Span::new(last.span(ast).end, list.delimiters.end - 1));
            if let Some(comment) = trailing.last()
                && comment.kind == CommentKind::Line
            {
                closing = self.source_line_constraint(comment.span.end, list.delimiters.end - 1);
            }
        }
        CallArgumentFrame::Fixed {
            role: CallFrameRole::Module,
            closing,
        }
    }

    pub(super) fn format_source_arguments(
        &mut self,
        list: SourceArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let output = self.format_source_arguments_with_chain_facts(list, tail, None, writer)?;
        Ok(output.tail)
    }

    pub(super) fn format_source_arguments_with_chain_facts(
        &mut self,
        list: SourceArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        chain_facts: Option<&mut ChainArgumentFacts>,
        writer: &mut Writer<'ast>,
    ) -> Result<FormattedArguments<'ast>, FormatError> {
        let observing = self.source_object_layout.observing;
        self.source_object_layout.observing |= chain_facts.is_some();
        let result = self.format_observed_source_arguments(list, tail, chain_facts, writer);
        // Retain nested discoveries while restoring the caller's observation scope,
        // including when construction returns an error.
        self.source_object_layout.observing = observing;
        result
    }

    fn format_observed_source_arguments(
        &mut self,
        list: SourceArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        mut chain_facts: Option<&mut ChainArgumentFacts>,
        writer: &mut Writer<'ast>,
    ) -> Result<FormattedArguments<'ast>, FormatError> {
        if let Some(facts) = chain_facts.as_deref_mut() {
            facts.source_expanded_object = false;
        }
        let ast = self.ast;
        let SourceArguments {
            arguments,
            first,
            delimiters,
            closing,
            callee_span,
            callee_mode,
            type_arguments,
            source_opening,
            prefix,
            group,
            frame,
            jsx_call,
            binary_indent,
            allow_composition,
            ..
        } = list;
        let first_mode = first.map_or(ContentMode::Formatted, |argument| argument.mode);
        let opening = delimiters.start;
        let mut has_outside_prefix = false;
        let (prefix_start, tail) = if let Some(types) = type_arguments {
            // The angle owner prints this pending tail before its opener.
            has_outside_prefix =
                types.has_comments() || tail.as_ref().is_some_and(|tail| !tail.comments.is_empty());
            let end = types.node.span(ast).end;
            self.format_expression_type_arguments(types, tail, writer)?;
            (end, None)
        } else {
            (callee_span.end, tail)
        };
        let source_end = tail.as_ref().map_or(prefix_start, |tail| tail.source_end);
        let content_start = match prefix {
            SourceArgumentPrefix::Empty { operator, comments } => {
                let comments = if tail.is_some() {
                    self.comments.between(Span::new(source_end, opening))?
                } else {
                    comments
                };
                has_outside_prefix |= !comments.is_empty();
                self.comments.claim(comments)?;
                self.format_empty_argument_prefix(comments, operator, source_end, writer)?;
                if operator.is_some() {
                    writer.text("?.");
                }
                opening + 1
            }
            SourceArgumentPrefix::Direct(prefix) => {
                let before = self.comments.between(Span::new(source_end, opening))?;
                let prefix_belongs_to_arguments = prefix.belongs_to_arguments();
                let move_tail = first.is_some()
                    && (prefix.comments().is_empty() || prefix_belongs_to_arguments)
                    && tail.as_ref().is_some_and(|tail| {
                        tail.comments
                            .iter()
                            .all(|comment| comment.kind == CommentKind::Block)
                            && self
                                .comment_binding(
                                    tail.comments,
                                    Span::new(source_end, opening + 1),
                                    None,
                                )
                                .allows_relocation(None)
                    });
                let outside_count = if move_tail {
                    0
                } else if prefix_belongs_to_arguments {
                    tail.as_ref().map_or(0, |tail| tail.comments.len())
                } else {
                    before.len()
                };
                let (outside, inside) = before.split_at(outside_count);
                has_outside_prefix |= !outside.is_empty();
                let boundary = self.prepare_comment_boundary(outside, source_end, opening);
                self.format_comment_boundary(&boundary, writer)?;
                if callee_mode == ContentMode::Verbatim
                    && self.content_mode(outside) == ContentMode::Verbatim
                {
                    // A same-line opening would reattach this marker to the first
                    // argument after reparsing, even when its text stayed outside.
                    writer.hard_line();
                } else {
                    boundary.gap().write(writer, FlatSeparator::Empty, |_| {});
                }
                if inside.is_empty() {
                    opening + 1
                } else if move_tail {
                    source_end
                } else {
                    prefix_start
                }
            }
            SourceArgumentPrefix::Optional {
                operator,
                before,
                after,
            } => {
                let before = if tail.is_some() {
                    self.comments
                        .between(Span::new(source_end, operator.start))?
                } else {
                    before
                };
                has_outside_prefix |= !before.is_empty();
                let boundary = self.prepare_comment_boundary(before, source_end, operator.start);
                self.format_comment_boundary(&boundary, writer)?;
                boundary.gap().write(writer, FlatSeparator::Empty, |_| {});
                writer.text("?.");
                if after.belongs_to_arguments() {
                    operator.end
                } else {
                    let after = after.comments();
                    has_outside_prefix |= !after.is_empty();
                    let boundary = self.prepare_comment_boundary(after, operator.end, opening);
                    self.format_comment_boundary(&boundary, writer)?;
                    boundary.gap().write(writer, FlatSeparator::Empty, |_| {});
                    opening + 1
                }
            }
        };
        let content_start = source_opening.map_or(content_start, |opening| opening.span.end);
        let content = Span::new(content_start, delimiters.end - 1);
        if first.is_none() {
            let comments = self.comments.in_range(content);
            if self.content_mode(comments) == ContentMode::Verbatim {
                self.format_verbatim(Span::new(opening, delimiters.end), writer)?;
                return Ok(FormattedArguments {
                    tail: None,
                    has_outside_prefix,
                });
            }
            if comments.is_empty() {
                writer.text("(");
                self.format_argument_dangling(content, writer)?;
                writer.text(")");
                return Ok(FormattedArguments {
                    tail: None,
                    has_outside_prefix,
                });
            }
        }
        let fixed_frame = frame.is_fixed();
        let hugged = fixed_frame
            || source_opening.is_none()
                && arguments.len() == 1
                && self.comments.in_range(content).is_empty()
                && !self.diagnostic_range_is_sensitive(content)
                && first.is_some_and(|argument| match ast.argument_data(argument.node) {
                    ArgumentData::Expr(expression) => self.call_argument_is_collection(expression),
                    ArgumentData::SpreadElement(_) => false,
                });
        let expand = !fixed_frame
            && (arguments
                .iter()
                .map(|id| ast.get_node_in_sub_range(id))
                .scan(None, |previous, argument| {
                    let span = argument.span(ast);
                    let empty = previous.is_some_and(|previous| {
                        let gap = Span::new(previous, span.start);
                        self.find_token(gap, self.comments.in_range(gap), ",")
                            .is_some_and(|comma| self.next_line_is_empty(comma + 1, span.start))
                    });
                    *previous = Some(span.end);
                    Some(empty)
                })
                .any(|empty| empty)
                || allow_composition && self.arguments_are_function_composition(arguments));
        if !fixed_frame
            && !expand
            && let Some(selected) = self.select_argument_layout(arguments, !hugged)
        {
            let tail = self.format_selected_arguments(
                arguments,
                content,
                ArgumentHead {
                    first_mode,
                    source_opening,
                    ordinary_group: group,
                    jsx_call,
                    independent_callback: frame.independent_callback(),
                    binary_indent,
                },
                selected,
                chain_facts,
                writer,
            )?;
            return Ok(FormattedArguments {
                tail,
                has_outside_prefix,
            });
        }
        let closing =
            if fixed_frame || source_opening.is_some() || callee_mode != ContentMode::Formatted {
                DelimitedClose::Source
            } else {
                DelimitedClose::Terminal(closing)
            };
        let tail = group.write(expand, writer, |writer| {
            if let Some(opening) = source_opening {
                self.format_verbatim(opening.span, writer)?;
            } else {
                writer.text("(");
            }
            let mut write_items = |writer: &mut Writer<'ast>| {
                if !hugged && (first.is_some() || !self.comments.in_range(content).is_empty()) {
                    if let Some(opening) = source_opening {
                        opening
                            .following
                            .write(writer, FlatSeparator::Empty, Writer::soft_line);
                    } else {
                        self.opening_line_constraint(content).write(
                            writer,
                            FlatSeparator::Empty,
                            Writer::soft_line,
                        );
                    }
                }
                let mut arguments = arguments
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .peekable();
                let mut previous = None;
                while let Some(argument) = arguments.next() {
                    let slot = CallArgumentSlot {
                        node: argument,
                        following: arguments.peek().copied(),
                    };
                    let mut item = self.prepare_argument_slot_with_frame(
                        slot,
                        previous,
                        content,
                        !hugged && self.options.trailing_commas == TrailingCommas::All,
                        frame,
                        writer,
                    )?;
                    if previous.is_none() && first_mode == ContentMode::Verbatim {
                        item.set_leading_mode(first_mode);
                    }
                    let tail = if let Some(callback) = frame.test_callback()
                        && callback.argument == argument && item.mode == ContentMode::Formatted
                    {
                        self.format_test_callback(callback, jsx_call, &mut chain_facts, writer)?
                    } else if let Some((wrapper_expression, wrapper)) = frame.angular_wrapper()
                        && matches!(ast.argument_data(argument), ArgumentData::Expr(expression) if expression == wrapper_expression)
                    {
                        self.format_expression(
                            wrapper_expression,
                            ExpressionContext::at(ExpressionPosition::Nested)
                                .with_content_mode(item.mode)
                                .with_angular_test_wrapper(wrapper),
                            writer,
                        )?
                    } else {
                        self.format_argument_with_chain_facts(
                            argument,
                            item.mode,
                            binary_indent,
                            jsx_call,
                            &mut chain_facts,
                            writer,
                        )?
                    };
                    previous = Some(self.finish_delimited_item_with_closing(
                        item,
                        tail,
                        closing,
                        writer,
                    )?);
                }
                let end = previous.map_or(content.start, |boundary| boundary.end);
                self.format_argument_dangling(Span::new(end, content.end), writer)
            };
            let end = if hugged {
                write_items(writer)?
            } else {
                writer.indent(write_items)?
            };
            if !hugged && (first.is_some() || end != content.start) {
                closing.constraint(self, end, content.end)
                    .write(writer, FlatSeparator::Empty, Writer::soft_line);
            }
            if let CallArgumentFrame::Fixed { closing, .. } = frame {
                closing.write(writer, FlatSeparator::Empty, |_| {});
            }
            writer.text(")");
            Ok(None)
        })?;
        if fixed_frame && let Some(facts) = chain_facts {
            facts.source_expanded_object = false;
        }
        Ok(FormattedArguments {
            tail,
            has_outside_prefix,
        })
    }
    pub(super) fn prepare_argument_slot(
        &mut self,
        argument: Argument,
        following: Option<Argument>,
        previous: Option<ItemBoundary>,
        content: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        let slot = CallArgumentSlot {
            node: argument,
            following,
        };
        self.prepare_argument_slot_with_frame(
            slot,
            previous,
            content,
            trailing_comma,
            CallArgumentFrame::Ordinary,
            writer,
        )
    }

    fn prepare_argument_slot_with_frame(
        &mut self,
        slot: CallArgumentSlot,
        previous: Option<ItemBoundary>,
        content: Span,
        trailing_comma: bool,
        frame: CallArgumentFrame,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        let CallArgumentSlot { node, following } = slot;
        let span = node.span(self.ast);
        let end = previous.map_or(content.start, |boundary| boundary.end);
        if let Some(previous) = previous {
            let leading = self.comments.between(Span::new(end, span.start))?;
            let next = leading
                .first()
                .map_or(span.start, |comment| comment.span.start);
            let separator = if matches!(frame, CallArgumentFrame::Fixed { .. })
                && (matches!(previous.separator, Separator::EmptyLine)
                    && leading
                        .iter()
                        .any(|comment| comment.kind == CommentKind::Line)
                    || leading.first().is_some_and(|comment| {
                        self.source_position_starts_line(comment.span.start)
                    }) && leading
                        .last()
                        .is_some_and(|comment| self.line_breaks(comment.span.end, span.start) != 0))
            {
                Separator::OwnLineComment
            } else {
                previous.separator
            };
            frame.separator(
                separator,
                self.source_separator_constraint(end, next, previous.transition.as_ref()),
                writer,
            );
        }
        let following = following.map_or(FollowingItem::End, |argument| {
            FollowingItem::Node(argument.span(self.ast))
        });
        let mut item = self.prepare_call_argument_item(
            span,
            following,
            Span::new(end, content.end),
            trailing_comma,
            previous.and_then(|boundary| boundary.transition),
            writer,
        )?;
        if let Some(previous) = previous {
            item.set_leading_mode(previous.following_mode);
        }
        Ok(item)
    }

    /// Reads the unconsumed outside boundary of this actual argument slot.
    /// Object-internal comments remain owned by the value formatter.
    pub(super) fn argument_has_boundary_comments(&self, item: &PreparedItem) -> bool {
        if item.has_leading_comments {
            return true;
        }
        let end = item
            .following
            .node()
            .map_or(item.span.end, |span| span.start);
        !self
            .comments
            .in_range(Span::new(item.item_span.end, end))
            .is_empty()
    }

    /// Inline blocks after the comma may stay with the following argument.
    /// Effective trailing ignore targets are already excluded by the item's mode;
    /// mandatory line breaks are checked on the recorded fragments.
    pub(super) fn first_callback_has_boundary_comments(&self, item: &PreparedItem) -> bool {
        if item.has_leading_comments {
            return true;
        }
        let end = item
            .following
            .node()
            .map_or(item.span.end, |span| span.start);
        let gap = Span::new(item.item_span.end, end);
        let comments = self.comments.in_range(gap);
        let Some(comment) = comments.first() else {
            return false;
        };
        let comma = self.find_comma(gap, comments);
        comma.is_none_or(|comma| comment.span.start < comma)
    }

    pub(super) fn format_argument_dangling(
        &mut self,
        span: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let dangling = self.comments.between(span)?;
        self.comments.claim(dangling)?;
        if dangling
            .iter()
            .any(|comment| comment.kind == CommentKind::Line)
        {
            writer.break_parent();
        }
        self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
        Ok(dangling
            .last()
            .map_or(span.start, |comment| comment.span.end))
    }
}
