//! Shared source-ordered argument slots and comma/comment boundaries.

use super::Formatter;
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{ContentMode, DelimitedFollowing, DelimitedSuppression};
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{Argument, CommentKind, GetSpan, Span};

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
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum FollowingItem {
    End,
    Node(Span),
    Elision { node: Option<Span>, holes: usize },
}

impl FollowingItem {
    fn node(self) -> Option<Span> {
        match self {
            Self::End => None,
            Self::Node(span) => Some(span),
            Self::Elision { node, .. } => node,
        }
    }
}

/// The real terminal item owns its comma before the following comments.
#[derive(Clone, Copy)]
pub(in crate::formatter) enum TrailingCommaPolicy {
    Forbidden,
    IfBreak,
}

pub(in crate::formatter) struct PreparedItem {
    item_span: Span,
    following: FollowingItem,
    span: Span,
    pub(in crate::formatter) mode: ContentMode,
    trailing_comma: TrailingCommaPolicy,
    suppression: Option<DelimitedSuppression>,
}

impl PreparedItem {
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
    Source(usize),
}

#[derive(Clone, Copy)]
pub(in crate::formatter) struct ItemBoundary {
    pub(in crate::formatter) end: u32,
    pub(in crate::formatter) separator: Separator,
    pub(in crate::formatter) following_mode: ContentMode,
    pub(in crate::formatter) elisions: ElisionLayout,
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
    pub(super) fn format_argument_item(
        &mut self,
        slot: ArrayArgumentSlot,
        span: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let ArrayArgumentSlot {
            argument,
            following,
            mode,
            layout,
        } = slot;
        let following = match following {
            FollowingArgument::Argument(argument) => FollowingItem::Node(argument.span(self.ast)),
            FollowingArgument::Elision { argument, holes } => FollowingItem::Elision {
                node: argument.map(|node| node.span(self.ast)),
                holes,
            },
            FollowingArgument::End => FollowingItem::End,
        };
        let mut item = self.prepare_delimited_item(
            argument.span(self.ast),
            following,
            span,
            trailing_comma,
            writer,
        )?;
        item.set_leading_mode(mode);
        let mut write =
            |writer: &mut Writer<'ast>| self.format_argument(argument, item.mode, writer);
        // Ordinary elements own their complete expression scope; their comma
        // and separator still observe the array. Concise fill owns its entries.
        let tail = match layout {
            ArgumentLayout::Lines => writer.group(false, write),
            ArgumentLayout::Fill => write(writer),
        }?;
        self.finish_delimited_item(item, tail, writer)
    }

    pub(in crate::formatter) fn prepare_delimited_item(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        let following_span = following.node();
        let suppression = self.delimited_suppression(
            item_span,
            following_span.map_or(span.end, |span| span.start),
            match following {
                FollowingItem::End => DelimitedFollowing::End,
                FollowingItem::Node(_) => DelimitedFollowing::Node,
                FollowingItem::Elision { node: Some(_), .. } => DelimitedFollowing::Node,
                FollowingItem::Elision { node: None, .. } => DelimitedFollowing::End,
            },
        );
        let leading = self
            .comments
            .between(Span::new(span.start, item_span.start))?;
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
        self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
        if let Some(last) = leading.last() {
            self.comment_gap(last.span.end, item_span.start, writer);
        }
        Ok(PreparedItem {
            item_span,
            following,
            span,
            mode,
            trailing_comma: if trailing_comma {
                TrailingCommaPolicy::IfBreak
            } else {
                TrailingCommaPolicy::Forbidden
            },
            suppression,
        })
    }

    pub(in crate::formatter) fn finish_delimited_item(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
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
        let has_more = !matches!(following, FollowingItem::End);
        let content_end = tail.as_ref().map_or(item_span.end, |tail| tail.source_end);
        let following_start = following_span.map(|span| span.start);
        let gap = Span::new(content_end, following_start.unwrap_or(span.end));
        let following_mode = suppression
            .as_ref()
            .map_or(ContentMode::Formatted, |source| source.following);
        let preceding = suppression.as_ref().and_then(|source| source.preceding);
        if preceding.is_some()
            && (following_span.is_some() || matches!(following, FollowingItem::Elision { .. }))
        {
            let source = self.format_delimited_source_gap(
                gap,
                following_span.map_or(DelimitedFollowing::End, |_| DelimitedFollowing::Node),
                writer,
            )?;
            return Ok(ItemBoundary {
                end: source.end,
                separator: Separator::Source(source.following),
                following_mode,
                elisions: match following {
                    FollowingItem::Elision { holes, .. } => ElisionLayout::Source(holes),
                    _ => ElisionLayout::Formatted,
                },
            });
        }
        let comments = self.comments.between(gap)?;
        let comma = self.find_token(gap, comments, ",");
        let constrained = self.diagnostic_range_is_sensitive(span);
        let preserve_comma = constrained || matches!(following, FollowingItem::Elision { .. });
        // A directive-covered separator owns the comments on its source side.
        let protected_comma =
            comma.filter(|&position| constrained && self.diagnostic_line_is_sensitive(position));
        let trailing_limit = if matches!(following, FollowingItem::Elision { .. }) {
            // The next elision owns the comments after this item's separator.
            comma.unwrap_or(gap.end)
        } else {
            gap.end
        };
        let ordinary_attachment = mode == ContentMode::Formatted
            && !constrained
            && !matches!(following, FollowingItem::Elision { .. });
        let (trailing_count, physical_break, inline_line) = if ordinary_attachment {
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
        let prepared_run = if prepare_run {
            self.prepare_delimited_trailing_run(
                comments,
                content_end,
                trailing_limit,
                following_span.is_some(),
            )
        } else {
            None
        };
        let canonical_run = prepared_run.is_some();
        let boundary_count = match prepared_run.as_ref() {
            Some(run) => {
                debug_assert!(
                    run.len() >= trailing_count,
                    "a canonical trailing run must include the complete legacy inline prefix",
                );
                run.len()
            }
            None => comma
                .filter(|_| preserve_comma)
                .map_or(trailing_count, |comma| {
                    comments
                        .partition_point(|comment| comment.span.end <= comma)
                        .max(trailing_count)
                }),
        };
        let boundary = self.comments.take(boundary_count);
        let (trailing, before_comma) = if canonical_run {
            (&boundary[..0], &boundary[..0])
        } else {
            boundary.split_at(trailing_count)
        };
        let trailing_end = trailing
            .last()
            .map_or(content_end, |comment| comment.span.end);
        self.format_trailing_comment_run(trailing, writer)?;
        if !before_comma.is_empty() {
            self.format_comment_run(
                before_comma,
                CommentRunLayout::SourceAfter(trailing_end),
                writer,
            )?;
        }
        let boundary_end = boundary
            .last()
            .map_or(content_end, |comment| comment.span.end);
        let mut separator = if comments[boundary_count..]
            .iter()
            .any(|comment| comment.kind == CommentKind::Line)
        {
            Separator::OwnLineComment
        } else {
            Separator::Line
        };
        let required_comma = has_more || protected_comma.is_some();
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
            if let Some(comma) = comma {
                let after_separator = Span::new(comma + 1, gap.end);
                // The source gap after the comma is independent of comment ownership.
                let after_comma = comments.partition_point(|comment| comment.span.start <= comma);
                let next_start = comments
                    .get(boundary_count.max(after_comma))
                    .map_or(gap.end, |comment| comment.span.start);
                let start = after_separator.start.max(boundary_end);
                if start < next_start && self.next_line_is_empty(start, next_start) {
                    separator = Separator::EmptyLine;
                }
            }
        } else if matches!(trailing_comma, TrailingCommaPolicy::IfBreak) {
            writer.if_break(",");
        }
        if let Some(run) = prepared_run {
            // Keep comma and the entire ordinary Block/Line packet together.
            // The next real separator owns its mandatory suffix termination.
            self.format_prepared_trailing_run(run, writer)?;
            if !matches!(separator, Separator::EmptyLine) {
                separator = Separator::TrailingLine;
            }
        }
        let mut end = boundary_end;
        if required_comma
            && let Some(comma) = comma
            && preserve_comma
            && self
                .comments
                .first()
                .is_none_or(|comment| comment.span.start >= comma)
        {
            end = end.max(comma + 1);
        }
        if !has_more || !preserve_comma && following_span.is_none() {
            let own_line = self.comments.between(Span::new(end, span.end))?;
            let own_line = self.comments.take(own_line.len());
            if let Some(first) = own_line.first() {
                writer.hard_line();
                if self.previous_line_is_empty(end, first.span.start) {
                    writer.hard_line();
                }
                self.format_comment_run(own_line, CommentRunLayout::Lines, writer)?;
            }
            if let Some(last) = own_line.last() {
                end = last.span.end;
            }
        }
        Ok(ItemBoundary {
            end,
            separator,
            following_mode,
            elisions: ElisionLayout::Formatted,
        })
    }
}
