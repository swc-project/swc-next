//! Typed comma-expression traversal and borrowed separator boundaries.

use swc_next_ecma_ast::{Comment, Expr, ExprData, GetSpan, SequenceExpression, Span};

use super::parentheses::{ExpressionContent, ResolvedExpressionHead, SourceParentheses};
use super::{ExpressionContext, ExpressionPosition, Formatter, HeadComments, TailOwner};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{ContentMode, DelimitedSuppression};
use crate::{FormatError, Semicolons, document::Writer};

struct SequenceBoundary<'ast> {
    next: Expr,
    next_start: u32,
    source_end: u32,
    comments: &'ast [Comment],
    comma: Option<u32>,
    trailing: &'ast [Comment],
    leading: &'ast [Comment],
    suppression: Option<DelimitedSuppression>,
    before_comma: LineConstraint,
    separator: LineConstraint,
}

struct SequenceItem<'ast> {
    expression: Expr,
    leading: &'ast [Comment],
    start: u32,
    mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn sequence_member_pair_is_enclosed(
        &self,
        content: &ExpressionContent,
        position: ExpressionPosition,
    ) -> bool {
        if content.parentheses.is_none()
            && !matches!(content.data, ExprData::ParenthesizedExpression(_))
        {
            return false;
        }
        let context = ExpressionContext::at(position);
        content.parentheses.is_some() && self.expression_is_parenthesized(content.data, context)
    }

    pub(super) fn sequence_member_parentheses(
        &self,
        content: &ExpressionContent,
        context: ExpressionContext,
    ) -> Option<SourceParentheses> {
        if context.content_mode != ContentMode::Formatted
            || !matches!(context.position, ExpressionPosition::SequenceMember)
        {
            return None;
        }
        let removed = content.parentheses?;
        (removed.head.has_comments && self.expression_is_parenthesized(content.data, context))
            .then_some(removed.source)
    }

    /// A real enclosing pair can own the ordinary head exposed by removing the
    /// first member's source envelope. Bound packets stay with that member.
    pub(super) fn enclosed_sequence_head_start(&self, sequence: SequenceExpression) -> Option<u32> {
        let first = self
            .ast
            .get_node_in_sub_range(sequence.expressions(self.ast).first()?);
        let content = self.expression_content(first);
        let removed = content.parentheses?;
        if !removed.head.has_comments
            || !matches!(removed.head.binding, CommentBinding::Unbound)
            || self.sequence_member_pair_is_enclosed(&content, ExpressionPosition::SequenceMember)
        {
            return None;
        }
        let start = content.expression.span(self.ast).start;
        let comments = self.ast.comments();
        let first =
            comments.partition_point(|comment| comment.span.start < sequence.span(self.ast).start);
        let comments = &comments[first..];
        let count = comments.partition_point(|comment| comment.span.end <= start);
        matches!(
            self.expression_head_binding(&comments[..count], content.expression),
            CommentBinding::Unbound
        )
        .then_some(start)
    }

    pub(super) fn format_sequence(
        &mut self,
        sequence: SequenceExpression,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let mut expressions = sequence
            .expressions(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id));
        let Some(first) = expressions.next() else {
            return Err(self.unsupported("empty sequence expression", sequence));
        };
        let item_position = ExpressionPosition::SequenceMember;
        let first_content = self.expression_content(first);
        let first_start = if self.sequence_member_pair_is_enclosed(&first_content, item_position) {
            first.span(ast).start
        } else {
            first_content.expression.span(ast).start
        };
        let enclosed_head = matches!(context.head_comments,
            HeadComments::ClaimedSequence(owner) if owner == sequence);
        if enclosed_head {
            context.head_comments = HeadComments::Pending;
        }
        let leading = if enclosed_head {
            &[]
        } else {
            self.comments
                .between(Span::new(sequence.span(ast).start, first_start))?
        };
        self.comments.claim(leading)?;
        let head = NodeHead {
            comments: leading,
            content_start: first_start,
            layout: CommentRunLayout::Leading(first_start),
            gap_end: None,
        };
        let parenthesize =
            self.expression_is_parenthesized(ExprData::SequenceExpression(sequence), context);
        if parenthesize && !leading.is_empty() {
            let binding = self.expression_head_binding(leading, first);
            let binding = first_content
                .parentheses
                .map_or(binding, |removed| binding.combine(removed.head.binding));
            if !matches!(binding, CommentBinding::Unbound) {
                return self.format_prepared_expression_with_resolved_head(
                    Expr::SequenceExpression(sequence),
                    ExpressionContent {
                        expression: Expr::SequenceExpression(sequence),
                        data: ExprData::SequenceExpression(sequence),
                        parentheses: None,
                    },
                    ResolvedExpressionHead { head, binding },
                    context,
                    writer,
                );
            }
        }
        let generated = if parenthesize && self.has_type_casts && !leading.is_empty() {
            let content = ExpressionContent {
                expression: Expr::SequenceExpression(sequence),
                data: ExprData::SequenceExpression(sequence),
                parentheses: None,
            };
            self.generated_enclosure_owner(
                Expr::SequenceExpression(sequence),
                &content,
                &head,
                CommentBinding::Unbound,
                context,
            )
            .map(|owner| super::parentheses::PreparedGeneratedEnclosure { owner, content })
        } else {
            None
        };
        let start = if enclosed_head {
            context.expression_start
        } else {
            self.format_node_head(head, context.expression_start, writer)?
        };
        if let Some(enclosure) = generated {
            return self.format_prepared_generated_enclosure(
                enclosure,
                context.with_expression_start(start),
                writer,
            );
        }
        if self.options.semicolons == Semicolons::AsNeeded && start.needs_asi_guard() {
            writer.text(";");
        }
        if parenthesize {
            writer.text("(");
        }
        let tail = writer.group(false, |writer| {
            let mut write_items = |writer: &mut Writer<'ast>| {
                let boundary = expressions
                    .next()
                    .map(|next| self.sequence_boundary(first, next, item_position))
                    .transpose()?;
                let tail = self.format_sequence_item(
                    SequenceItem {
                        expression: first,
                        leading: &[],
                        start: first_start,
                        mode: ContentMode::Formatted,
                    },
                    boundary.as_ref(),
                    ExpressionContext::at(item_position)
                        .with_parent_head(context)
                        .with_expression_start(if parenthesize {
                            super::ExpressionStart::None
                        } else {
                            context.expression_start
                        })
                        .with_tail_owner(context.tail_owner),
                    writer,
                )?;
                let Some(mut boundary) = boundary else {
                    return Ok(tail);
                };
                self.format_sequence_trailing(&mut boundary, tail, writer)?;
                let mut write_rest = |writer: &mut Writer<'ast>| loop {
                    boundary
                        .separator
                        .write(writer, FlatSeparator::Space, Writer::line);
                    let following = expressions
                        .next()
                        .map(|next| self.sequence_boundary(boundary.next, next, item_position))
                        .transpose()?;
                    let tail = self.format_sequence_item(
                        SequenceItem {
                            expression: boundary.next,
                            leading: boundary.leading,
                            start: boundary.next_start,
                            mode: boundary.suppression.as_ref().map_or_else(
                                || self.content_mode(boundary.leading),
                                |source| source.following,
                            ),
                        },
                        following.as_ref(),
                        ExpressionContext::at(item_position).with_tail_owner(context.tail_owner),
                        writer,
                    )?;
                    let Some(mut following) = following else {
                        break Ok(tail);
                    };
                    self.format_sequence_trailing(&mut following, tail, writer)?;
                    boundary = following;
                };
                if matches!(context.position, ExpressionPosition::Statement) {
                    writer.indent(write_rest)
                } else {
                    write_rest(writer)
                }
            };
            let tail = write_items(writer)?;
            let tail = if parenthesize {
                self.format_parentheses_tail(context.source_parentheses_tail, tail, writer)?
            } else {
                tail
            };
            Ok(tail)
        })?;
        if parenthesize {
            writer.text(")");
        }
        Ok(tail)
    }

    fn format_sequence_item(
        &mut self,
        item: SequenceItem<'ast>,
        following: Option<&SequenceBoundary<'ast>>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let SequenceItem {
            expression,
            leading,
            start,
            mode,
        } = item;
        let mode = if mode == ContentMode::Verbatim {
            mode
        } else {
            following.map_or(ContentMode::Formatted, |boundary| {
                if boundary
                    .suppression
                    .as_ref()
                    .and_then(|source| source.preceding)
                    .is_some_and(|target| target == expression.span(self.ast))
                {
                    ContentMode::Verbatim
                } else {
                    ContentMode::Formatted
                }
            })
        };
        let tail_owner = if following.is_some() {
            TailOwner::ParentOperation
        } else {
            context.tail_owner
        };
        self.comments.claim(leading)?;
        self.format_expression_with_leading_comments(
            expression,
            NodeHead {
                comments: leading,
                content_start: start,
                layout: CommentRunLayout::Leading(start),
                gap_end: None,
            },
            context.with_content_mode(mode).with_tail_owner(tail_owner),
            writer,
        )
    }

    fn sequence_boundary(
        &self,
        previous: Expr,
        next: Expr,
        position: ExpressionPosition,
    ) -> Result<SequenceBoundary<'ast>, FormatError> {
        let start = previous.span(self.ast).end;
        let content = self.expression_content(next);
        let end = if self.sequence_member_pair_is_enclosed(&content, position) {
            next.span(self.ast).start
        } else {
            content.expression.span(self.ast).start
        };
        let span = Span::new(start, end);
        // Retain the borrowed run while the previous operand consumes its own
        // comments. Its returned tail determines the remaining suffix.
        let comments = self
            .comments
            .in_range(Span::new(previous.span(self.ast).start, end));
        let comma = if comments.is_empty() && !self.diagnostic_range_is_sensitive(span) {
            None
        } else {
            let outside = comments.partition_point(|comment| comment.span.end <= start);
            let Some(comma) = self.find_token(span, &comments[outside..], ",") else {
                return Err(FormatError::unsupported(
                    "sequence separator boundary",
                    None,
                    span,
                ));
            };
            Some(comma)
        };
        Ok(SequenceBoundary {
            next,
            next_start: end,
            source_end: start,
            comments,
            comma,
            trailing: &[],
            leading: &[],
            suppression: self.delimited_suppression(previous.span(self.ast), end),
            before_comma: LineConstraint::Unconstrained,
            separator: LineConstraint::Unconstrained,
        })
    }

    fn prepare_sequence_comments(
        &self,
        boundary: &mut SequenceBoundary<'ast>,
        span: Span,
    ) -> Result<(), FormatError> {
        let count = boundary
            .comments
            .partition_point(|comment| comment.span.end <= span.start);
        let comments = &boundary.comments[count..];
        let start = span.start;
        let end = span.end;
        let sensitive = self.diagnostic_range_is_sensitive(span);
        if comments.is_empty() && !sensitive {
            return Ok(());
        }
        let Some(comma) = boundary.comma else {
            return Err(FormatError::unsupported(
                "sequence separator boundary",
                None,
                span,
            ));
        };
        let before_count = comments.partition_point(|comment| comment.span.end <= comma);
        (boundary.trailing, boundary.leading) = if sensitive {
            comments.split_at(before_count)
        } else {
            self.split_separator_comments(comments, before_count, start, Some(end))
        };
        if sensitive {
            let last = boundary
                .trailing
                .last()
                .map_or(start, |comment| comment.span.end);
            let next = boundary
                .leading
                .first()
                .map_or(end, |comment| comment.span.start);
            boundary.before_comma = self.line_constraint(last, comma);
            boundary.separator = self.line_constraint(comma + 1, next);
        }
        Ok(())
    }

    fn format_sequence_trailing(
        &mut self,
        boundary: &mut SequenceBoundary<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some(tail) = tail {
            boundary.source_end = tail.source_end;
        }
        let gap = Span::new(boundary.source_end, boundary.next_start);
        if boundary
            .suppression
            .as_ref()
            .is_some_and(|source| source.preceding.is_some())
        {
            boundary.separator = self.format_delimited_source_gap(gap, writer)?;
            return Ok(());
        }
        self.prepare_sequence_comments(boundary, gap)?;
        self.comments.claim(boundary.trailing)?;
        self.format_comment_run(
            boundary.trailing,
            CommentRunLayout::SourceAfter(boundary.source_end),
            writer,
        )?;
        boundary
            .before_comma
            .write(writer, FlatSeparator::Empty, |_| {});
        writer.text(",");
        Ok(())
    }
}
