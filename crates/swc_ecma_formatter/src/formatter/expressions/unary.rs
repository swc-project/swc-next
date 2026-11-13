//! Prefix and postfix operators with explicitly owned operand boundaries.

use swc_next_ecma_ast::{
    Comment, CommentKind, ExprData, GetSpan, Span, UnaryExpression, UnaryOperator, UpdateExpression,
};

use super::binary::BinaryLike;
use super::parentheses::{
    ExpressionContent, PreparedAnnotationParentheses, PreparedExpressionPlan,
};
use super::{
    EmittedHeadBinding, ExpressionContext, ExpressionOutput, ExpressionPosition, Formatter,
    OperandIndent, ParenthesesOwner, ResolvedExpressionHead,
};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

struct UnaryOperandTail<'ast> {
    binary: BinaryLike,
    tail: ExpressionTail<'ast>,
}

struct PreparedUnaryOperandHead<'ast> {
    resolved: ResolvedExpressionHead<'ast>,
    released: Option<ResolvedExpressionHead<'ast>>,
}

struct UnaryOperandSource<'ast> {
    content: super::parentheses::ExpressionContent,
    context: ExpressionContext<'ast>,
    argument_span: Span,
    leading: &'ast [Comment],
    trailing: &'ast [Comment],
    leading_end: u32,
}

impl UnaryOperandSource<'_> {
    fn content(&self) -> super::parentheses::ExpressionContent {
        // Reuse the prepared view without selecting source parentheses again.
        super::parentheses::ExpressionContent {
            expression: self.content.expression,
            data: self.content.data,
            parentheses: self.content.parentheses,
        }
    }
}

/// One actual unary owner shares its immutable operand plan with its RHS boundary.
pub(super) struct PreparedUnaryOperand<'ast> {
    owner: UnaryExpression,
    mode: ContentMode,
    head_scope: Option<super::parentheses::HeadScopeOwner>,
    source: UnaryOperandSource<'ast>,
    head: Option<PreparedUnaryOperandHead<'ast>>,
    tail: Option<UnaryOperandTail<'ast>>,
    annotation: Option<PreparedAnnotationParentheses>,
}

impl PreparedUnaryOperand<'_> {
    pub(super) fn matches_owner(
        &self,
        unary: UnaryExpression,
        context: ExpressionContext<'_>,
    ) -> bool {
        self.owner == unary
            && self.mode == context.content_mode
            && self.head_scope == context.head_scope
    }

    pub(super) fn formatted_layout_body(&self) -> Option<&ExpressionContent> {
        if self.source.context.content_mode != ContentMode::Formatted {
            return None;
        }
        if let Some(frame) = &self.annotation {
            return Some(frame.layout_body());
        }
        (self.head.is_some()
            && !matches!(
                self.source.content.data,
                ExprData::ParenthesizedExpression(_)
            ))
        .then_some(&self.source.content)
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn prepare_unary_operand(
        &self,
        unary: UnaryExpression,
        context: ExpressionContext<'ast>,
    ) -> Result<Option<&'ast PreparedUnaryOperand<'ast>>, FormatError> {
        let span = unary.span(self.ast);
        // Sensitive diagnostics keep source enclosure preparation with the renderer.
        if context.content_mode != ContentMode::Formatted
            || self.diagnostic_range_is_sensitive(span)
        {
            return Ok(None);
        }
        let operator_end = span.start + unary.operator(self.ast).as_str().len() as u32;
        let packet = self.comments.in_range(Span::new(operator_end, span.end));
        if packet.is_empty() {
            return Ok(None);
        }
        let source = self.prepare_unary_operand_source(unary);
        let head = self.prepare_unary_operand_head(&source)?;
        let prepared = self.prepare_unary_operand_frame(unary, context, source, head);
        Ok(Some(self.allocator.alloc(prepared)))
    }

    fn prepare_unary_operand_source(&self, unary: UnaryExpression) -> UnaryOperandSource<'ast> {
        let operator = unary.operator(self.ast);
        let source_argument = unary.argument(self.ast);
        let content = self.expression_content(source_argument);
        let span = unary.span(self.ast);
        let argument_span = content.expression.span(self.ast);
        let leading = self.comments.in_range(Span::new(
            span.start + operator.as_str().len() as u32,
            argument_span.start,
        ));
        let mode = self.content_mode(leading);
        let context = self.with_head_scope(
            ExpressionContext::at(ExpressionPosition::UnaryArgument(operator))
                .with_content_mode(mode),
            &content,
        );
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
        UnaryOperandSource {
            content,
            context,
            argument_span,
            leading,
            trailing,
            leading_end,
        }
    }

    fn prepare_unary_operand_frame(
        &self,
        unary: UnaryExpression,
        context: ExpressionContext<'ast>,
        source: UnaryOperandSource<'ast>,
        head: Option<PreparedUnaryOperandHead<'ast>>,
    ) -> PreparedUnaryOperand<'ast> {
        let annotation =
            if let ExprData::ParenthesizedExpression(parenthesized) = source.content.data {
                let body_context = self.unary_operand_body_context(unary, &source, head.is_some());
                self.prepare_annotation_parentheses(parenthesized, &source.content, body_context)
            } else {
                None
            };
        let tail = if head.is_some()
            && source.leading.is_empty()
            && source.context.content_mode == ContentMode::Formatted
        {
            self.prepare_unary_operand_tail(
                source.content.data,
                source.argument_span,
                unary.span(self.ast).end,
                source.trailing,
            )
        } else {
            None
        };
        PreparedUnaryOperand {
            owner: unary,
            mode: context.content_mode,
            head_scope: context.head_scope,
            source,
            head,
            tail,
            annotation,
        }
    }

    fn unary_operand_body_context(
        &self,
        unary: UnaryExpression,
        source: &UnaryOperandSource<'ast>,
        framed: bool,
    ) -> ExpressionContext<'ast> {
        let context = if framed {
            let position = ExpressionPosition::ParenthesizedUnaryArgument(unary.operator(self.ast));
            let mut context = ExpressionContext::at(position)
                .with_content_mode(source.context.content_mode)
                .inside_parentheses();
            if self.expression_needs_parentheses(
                source.content.data,
                position,
                ContentMode::Formatted,
            ) == Some(true)
            {
                context.parentheses_owner = ParenthesesOwner::Expression;
            }
            context
        } else {
            source.context
        };
        self.with_head_scope(context, &source.content)
    }

    pub(super) fn format_unary_output(
        &mut self,
        unary: UnaryExpression,
        context: ExpressionContext<'ast>,
        prepared_cache: Option<&PreparedUnaryOperand<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        let operator = unary.operator(self.ast);
        writer.text(operator.as_str());
        let word_operator = matches!(
            operator,
            UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete
        );
        if word_operator {
            writer.space();
        }
        if let Some(prepared) =
            prepared_cache.filter(|prepared| prepared.matches_owner(unary, context))
        {
            return self.format_prepared_unary_operand(prepared, writer);
        }
        let mut source = self.prepare_unary_operand_source(unary);
        let span = unary.span(self.ast);
        let diagnostic_sensitive = self.diagnostic_range_is_sensitive(span);
        if diagnostic_sensitive
            && let Some(enclosure) =
                self.prepare_unary_enclosure(unary, &source.content, source.leading, source.context)
        {
            return self
                .format_prepared_unary_enclosure(source.content, enclosure, writer)
                .map(ExpressionOutput::closed);
        }
        let source_pair = if diagnostic_sensitive {
            self.protected_grammar_parentheses(&source.content, source.context)
        } else {
            None
        };
        if let Some(boundary) = source_pair {
            source.argument_span = boundary.outer;
            let count = source
                .leading
                .partition_point(|comment| comment.span.end <= boundary.outer.start);
            source.leading = &source.leading[..count];
            source.trailing = self
                .comments
                .in_range(Span::new(source.argument_span.end, span.end));
        }
        // Required frames keep their prepared packet. Otherwise an interior
        // reset belongs to the operator's ordinary optional comment frame.
        let diagnostic_sensitive = diagnostic_sensitive
            && (source_pair.is_some()
                || !self.expression_head_resets_diagnostic_boundary(
                    &source.content,
                    source.leading,
                    span.start,
                ));
        if diagnostic_sensitive {
            let operator_end = span.start + operator.as_str().len() as u32;
            self.comments.claim(source.leading)?;
            let layout = CommentRunLayout::SourceAfter(operator_end);
            if word_operator {
                self.format_comment_run_after_separator(source.leading, layout, writer)?;
            } else {
                self.format_comment_run(source.leading, layout, writer)?;
            }
            let flat = if source.leading.is_empty() {
                FlatSeparator::Empty
            } else {
                FlatSeparator::Space
            };
            self.line_constraint_after_comments(
                source.leading,
                operator_end,
                source.argument_span.start,
            )
            .write(writer, flat, |_| {});
            let operand_indent = match source.content.data {
                ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::ParenthesizedExpression(_) => OperandIndent::Parent,
                _ => OperandIndent::Expression,
            };
            let context = source.context.with_operand_indent(operand_indent);
            let mut output = match source_pair {
                Some(boundary) => self
                    .format_protected_grammar_parentheses(
                        boundary,
                        source.content,
                        context,
                        None,
                        writer,
                    )
                    .map(ExpressionOutput::closed)?,
                None => self.format_expression_prepared_output(
                    source.content.expression,
                    Some(source.content),
                    context,
                    writer,
                )?,
            };
            output.pending = self.capture_expression_tail(
                Span::new(source.argument_span.end, span.end),
                output.pending,
            )?;
            return Ok(output);
        }
        let Some(head) = self.prepare_unary_operand_head(&source)? else {
            // The operand's own renderer keeps any retained source frame.
            return self.format_expression_prepared_output(
                source.content.expression,
                Some(source.content),
                source.context,
                writer,
            );
        };
        let prepared = self.prepare_unary_operand_frame(unary, context, source, Some(head));
        self.format_prepared_unary_operand(&prepared, writer)
    }

    fn format_prepared_unary_operand(
        &mut self,
        prepared: &PreparedUnaryOperand<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        let source = &prepared.source;
        let content = source.content();
        let argument = content.expression;
        let mut body_context =
            self.unary_operand_body_context(prepared.owner, source, prepared.head.is_some());
        let Some(head) = &prepared.head else {
            if let Some(frame) = &prepared.annotation {
                let plan = PreparedExpressionPlan::Annotation(frame);
                return self.format_expression_prepared_with_plan_output(
                    argument,
                    Some(content),
                    body_context,
                    &plan,
                    writer,
                );
            }
            return self.format_expression_prepared_output(
                argument,
                Some(content),
                body_context,
                writer,
            );
        };
        // The operator owns one group around boundary comments and its operand.
        writer.group(false, |writer| {
            writer.text("(");
            writer.indent(|writer| {
                writer.soft_line();
                self.comments.claim(head.resolved.head.comments)?;
                if let Some(owner) = &prepared.tail {
                    // A last operand's tail stays in the binary continuation group,
                    // even after its source parentheses have been removed.
                    let tail = self.format_binary_enclosed(owner.binary, writer)?;
                    self.format_expression_tail(tail, writer)?;
                    self.format_expression_tail(
                        Some(ExpressionTail {
                            source_end: owner.tail.source_end,
                            comments: owner.tail.comments,
                        }),
                        writer,
                    )
                } else {
                    if let Some(released) = &head.released
                        && let Some(first) = released.head.comments.first()
                        && let Some(last) = released.head.comments.last()
                    {
                        body_context.emitted_head_binding = EmittedHeadBinding::Resolved {
                            packet: Span::new(first.span.start, last.span.end),
                            binding: released.binding,
                        };
                    }
                    let resolved = ResolvedExpressionHead {
                        head: NodeHead {
                            ..head.resolved.head
                        },
                        binding: head.resolved.binding,
                    };
                    let tail = if let Some(frame) = &prepared.annotation {
                        let plan = PreparedExpressionPlan::Annotation(frame);
                        self.format_expression_leading_with_plan(
                            argument,
                            Some(content),
                            resolved,
                            body_context,
                            None,
                            &plan,
                            writer,
                        )?
                        .pending
                    } else {
                        self.format_expression_leading_prepared(
                            argument,
                            Some(content),
                            resolved,
                            body_context,
                            None,
                            writer,
                        )?
                    };
                    self.format_expression_tail(tail, writer)?;
                    self.comments.claim(source.trailing)?;
                    self.format_comment_run(
                        source.trailing,
                        CommentRunLayout::SourceAfter(source.argument_span.end),
                        writer,
                    )
                }
            })?;
            if source
                .trailing
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
            {
                writer.hard_line();
            } else {
                writer.soft_line();
            }
            writer.text(")");
            Ok(ExpressionOutput::closed(None))
        })
    }

    fn prepare_unary_operand_head(
        &self,
        source: &UnaryOperandSource<'ast>,
    ) -> Result<Option<PreparedUnaryOperandHead<'ast>>, FormatError> {
        let argument = source.content.expression;
        let exposed = if source.trailing.is_empty()
            && !matches!(source.content.data, ExprData::ParenthesizedExpression(_))
        {
            self.exposed_expression_head(argument, source.context)?
        } else {
            None
        };
        if source.leading.is_empty() && source.trailing.is_empty() && exposed.is_none() {
            return Ok(None);
        }
        Ok(Some(PreparedUnaryOperandHead {
            resolved: ResolvedExpressionHead {
                head: NodeHead {
                    comments: source.leading,
                    content_start: argument.span(self.ast).start,
                    layout: CommentRunLayout::GroupedLeading(source.leading_end),
                    gap_end: None,
                },
                binding: self.expression_head_binding(source.leading, argument),
            },
            released: exposed,
        }))
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
        context: ExpressionContext<'ast>,
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
