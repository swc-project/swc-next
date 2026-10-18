//! Prefix and postfix operators with explicitly owned operand boundaries.

use swc_next_ecma_ast::{
    Comment, CommentKind, ExprData, GetSpan, Span, UnaryExpression, UnaryOperator, UpdateExpression,
};

use super::binary::BinaryLike;
use super::{ExpressionContext, ExpressionPosition, Formatter, OperandIndent, ParenthesesOwner};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

struct UnaryOperandTail<'ast> {
    binary: BinaryLike,
    tail: ExpressionTail<'ast>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn format_unary(
        &mut self,
        unary: UnaryExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let operator = unary.operator(self.ast);
        writer.text(operator.as_str());
        if matches!(
            operator,
            UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete
        ) {
            writer.space();
        }
        let position = ExpressionPosition::UnaryArgument(operator);
        let source_argument = unary.argument(self.ast);
        let content = self.expression_content(source_argument);
        let span = unary.span(self.ast);
        let diagnostic_sensitive = self.diagnostic_range_is_sensitive(span);
        let argument = content.expression;
        let mut argument_span = argument.span(self.ast);
        let mut leading = self.comments.between(Span::new(
            span.start + operator.as_str().len() as u32,
            argument_span.start,
        ))?;
        let mode = self.content_mode(leading);
        let source_pair = if diagnostic_sensitive {
            self.protected_grammar_parentheses(
                &content,
                ExpressionContext::at(position).with_content_mode(mode),
            )
        } else {
            None
        };
        if let Some(boundary) = source_pair {
            argument_span = boundary.outer;
            let count = leading.partition_point(|comment| comment.span.end <= boundary.outer.start);
            leading = &leading[..count];
        }
        let trailing = self
            .comments
            .in_range(Span::new(argument_span.end, span.end));
        let leading_end = match content.parentheses.map(|removed| removed.source) {
            Some(boundary)
                if mode == ContentMode::Verbatim
                    && leading
                        .last()
                        .is_some_and(|comment| comment.span.end <= boundary.outer.start) =>
            {
                boundary.outer.start
            }
            _ => argument_span.start,
        };
        if diagnostic_sensitive {
            let operator_end = span.start + operator.as_str().len() as u32;
            self.comments.claim(leading)?;
            self.format_comment_run(leading, CommentRunLayout::SourceAfter(operator_end), writer)?;
            let flat = if leading.is_empty() {
                FlatSeparator::Empty
            } else {
                FlatSeparator::Space
            };
            self.line_constraint_after_comments(leading, operator_end, argument_span.start)
                .write(writer, flat, |_| {});
            let operand_indent = match content.data {
                ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::ParenthesizedExpression(_) => OperandIndent::Parent,
                _ => OperandIndent::Expression,
            };
            let context = ExpressionContext::at(position)
                .with_content_mode(mode)
                .with_operand_indent(operand_indent);
            let tail = match source_pair {
                Some(boundary) => self.format_protected_grammar_parentheses(
                    boundary, content, context, None, writer,
                )?,
                None => {
                    self.format_expression_prepared(argument, Some(content), context, writer)?
                }
            };
            return self.capture_expression_tail(Span::new(argument_span.end, span.end), tail);
        }
        if leading.is_empty() && trailing.is_empty() {
            return self.format_expression(argument, ExpressionContext::at(position), writer);
        }
        let operand_tail = if leading.is_empty() && mode == ContentMode::Formatted {
            self.prepare_unary_operand_tail(content.data, argument_span, span.end, trailing)
        } else {
            None
        };
        // The operator owns one group around boundary comments and its operand.
        writer.group(false, |writer| {
            writer.text("(");
            writer.indent(|writer| {
                writer.soft_line();
                self.comments.claim(leading)?;
                if let Some(owner) = operand_tail {
                    // A last operand's tail stays in the binary continuation group,
                    // even after its source parentheses have been removed.
                    let tail = self.format_binary_enclosed(owner.binary, writer)?;
                    self.format_expression_tail(tail, writer)?;
                    self.format_expression_tail(Some(owner.tail), writer)
                } else {
                    let position = ExpressionPosition::ParenthesizedUnaryArgument(operator);
                    let mut context = ExpressionContext::at(position)
                        .with_content_mode(mode)
                        .inside_parentheses();
                    // A distinct operand pair remains owned by that expression;
                    // otherwise this physical unary pair supplies its frame.
                    if !matches!(
                        content.data,
                        ExprData::UnaryExpression(_) | ExprData::UpdateExpression(_)
                    ) && self.expression_needs_parentheses(
                        content.data,
                        position,
                        ContentMode::Formatted,
                    ) == Some(true)
                    {
                        context.parentheses_owner = ParenthesesOwner::Expression;
                    }
                    let tail = self.format_expression_with_leading_comments(
                        argument,
                        NodeHead {
                            comments: leading,
                            content_start: argument_span.start,
                            layout: CommentRunLayout::GroupedLeading(leading_end),
                            gap_end: None,
                        },
                        context,
                        None,
                        writer,
                    )?;
                    self.format_expression_tail(tail, writer)?;
                    self.comments.claim(trailing)?;
                    self.format_comment_run(
                        trailing,
                        CommentRunLayout::SourceAfter(argument_span.end),
                        writer,
                    )
                }
            })?;
            if trailing
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
            {
                writer.hard_line();
            } else {
                writer.soft_line();
            }
            writer.text(")");
            Ok(None)
        })
    }

    fn prepare_unary_operand_tail(
        &self,
        argument: ExprData,
        argument_span: Span,
        unary_end: u32,
        comments: &'ast [Comment],
    ) -> Option<UnaryOperandTail<'ast>> {
        let last = comments.last()?;
        let binary = BinaryLike::from_data(argument, self.ast)?;
        let right = binary.right.span(self.ast);
        if self.line_breaks(argument_span.start, right.start) == 0
            || self.line_breaks(right.start, last.span.end) != 0
            || !matches!(
                self.comment_binding(comments, Span::new(argument_span.end, unary_end), None),
                CommentBinding::Unbound
            )
        {
            return None;
        }
        Some(UnaryOperandTail {
            binary,
            tail: ExpressionTail {
                source_end: right.end,
                comments,
            },
        })
    }

    pub(in crate::formatter) fn format_update(
        &mut self,
        update: UpdateExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let target = self.update_source_target(update)?;
        let span = update.span(self.ast);
        let target_span = target.span();
        let operator = target.operator();
        if update.prefix(self.ast) {
            writer.text(operator);
            let comments = self
                .comments
                .between(Span::new(target.operator_span.end, target_span.start))?;
            let mode = self.content_mode(comments);
            self.comments.claim(comments)?;
            let next = comments
                .first()
                .map_or(target_span.start, |comment| comment.span.start);
            self.line_constraint(target.operator_span.end, next).write(
                writer,
                FlatSeparator::Empty,
                |_| {},
            );
            self.format_comment_run(
                comments,
                CommentRunLayout::FlatLeading(target_span.start),
                writer,
            )?;
            let tail = self.format_source_target(
                &target,
                context
                    .with_expression_start(super::ExpressionStart::None)
                    .with_content_mode(mode),
                writer,
            )?;
            self.capture_expression_tail(Span::new(target_span.end, span.end), tail)
        } else {
            let leading = self
                .comments
                .between(Span::new(span.start, target_span.start))?;
            let leading = self.comments.take(leading.len());
            self.format_comment_run(
                leading,
                CommentRunLayout::Leading(target_span.start),
                writer,
            )?;
            let boundary = Span::new(target_span.end, target.operator_span.start);
            let mode = self.content_mode(self.comments.in_range(boundary));
            let mode = if self.content_mode(leading) == ContentMode::Verbatim {
                ContentMode::Verbatim
            } else {
                mode
            };
            let tail =
                self.format_source_target(&target, context.with_content_mode(mode), writer)?;
            let source_end = tail
                .as_ref()
                .map_or(target_span.end, |tail| tail.source_end);
            let boundary = Span::new(source_end, target.operator_span.start);
            let comments = self.comments.between(boundary)?;
            let count = comments
                .iter()
                .take_while(|comment| {
                    comment.kind == CommentKind::Block
                        && self.line_breaks(source_end, comment.span.start) == 0
                        && self.line_breaks(comment.span.start, comment.span.end) == 0
                })
                .count();
            let inline = self.comments.take(count);
            self.format_inline_trailing_comments(inline, writer)?;
            let source_end = inline.last().map_or(source_end, |comment| comment.span.end);
            writer.text(operator);
            self.capture_expression_tail(Span::new(source_end, boundary.end), None)
        }
    }
}
