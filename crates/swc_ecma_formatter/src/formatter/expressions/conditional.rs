//! Conditional positions, aligned branches, and prepared operator boundaries.

use swc_next_ecma_ast::{Comment, ConditionalExpression, Expr, ExprData, GetSpan, Span};

use super::parentheses::{
    EmittedHeadAnnotation, ExpressionContent, PreparedGeneratedEnclosure, ResolvedExpressionHead,
};
use super::{ExpressionContext, ExpressionPosition, Formatter, OperandIndent, TailOwner};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, IndentStyle, document::Writer};

#[derive(Clone, Copy)]
enum ConditionalLayout {
    Root,
    Test,
    Consequent,
    Alternate,
}

struct ConditionalOperand {
    original: Expr,
    content: ExpressionContent,
    source_end: u32,
    content_start: u32,
}

struct PreparedConditional<'ast> {
    test: ConditionalOperand,
    consequent: ConditionalOperand,
    alternate: ConditionalOperand,
    question: ConditionalBoundary<'ast>,
    colon: ConditionalBoundary<'ast>,
}

struct ConditionalBranch<'ast> {
    operand: ConditionalOperand,
    operand_indent: OperandIndent,
    position: ExpressionPosition,
    boundary: ResolvedConditionalBoundary<'ast>,
    following_mode: ContentMode,
}

/// Source effects are prepared before either operand is printed. Ordinary
/// attachment waits for the preceding operand's complete pending packet.
struct ConditionalBoundary<'ast> {
    operator: &'static str,
    source_range: Span,
    comments: &'ast [Comment],
    next_binding: CommentBinding,
    source: Option<ConditionalSourceBoundary>,
}

struct ConditionalSourceBoundary {
    operator_start: u32,
    binding: CommentBinding,
    previous_mode: ContentMode,
    next_mode: ContentMode,
    before_operator: LineConstraint,
    after_operator: LineConstraint,
}

impl ConditionalBoundary<'_> {
    fn previous_mode(&self) -> ContentMode {
        self.source
            .as_ref()
            .map_or(ContentMode::Formatted, |source| source.previous_mode)
    }
}

/// One claimed packet supplies the previous tail and the following node head.
/// This contains placement only; no already printed operand can change mode.
struct ResolvedConditionalBoundary<'ast> {
    operator: &'static str,
    leading: &'ast [Comment],
    annotation: Option<EmittedHeadAnnotation>,
    next_binding: CommentBinding,
    next_mode: ContentMode,
    before_operator: LineConstraint,
    after_operator: LineConstraint,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_conditional(
        &mut self,
        conditional: ConditionalExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let prepared = self.prepare_conditional(conditional)?;
        self.format_prepared_conditional(prepared, context, writer)
    }

    fn format_prepared_conditional(
        &mut self,
        prepared: PreparedConditional<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let layout = match context.position {
            ExpressionPosition::ConditionalTest => ConditionalLayout::Test,
            ExpressionPosition::ConditionalConsequent => ConditionalLayout::Consequent,
            ExpressionPosition::ConditionalAlternate => ConditionalLayout::Alternate,
            _ => ConditionalLayout::Root,
        };
        let write = |writer: &mut Writer<'ast>| {
            self.format_conditional_contents(prepared, context, layout, writer)
        };
        match layout {
            ConditionalLayout::Root | ConditionalLayout::Test => writer.group(false, write),
            ConditionalLayout::Consequent | ConditionalLayout::Alternate => write(writer),
        }
    }

    fn prepare_conditional(
        &self,
        conditional: ConditionalExpression,
    ) -> Result<PreparedConditional<'ast>, FormatError> {
        let test = self.conditional_operand(conditional.test(self.ast));
        let consequent = self.conditional_operand(conditional.consequent(self.ast));
        let alternate = self.conditional_operand(conditional.alternate(self.ast));
        let question = self.conditional_boundary(&test, &consequent, "?")?;
        let colon = self.conditional_boundary(&consequent, &alternate, ":")?;
        Ok(PreparedConditional {
            test,
            consequent,
            alternate,
            question,
            colon,
        })
    }

    fn conditional_operand(&self, original: Expr) -> ConditionalOperand {
        let content = self.expression_content(original);
        ConditionalOperand {
            original,
            source_end: original.span(self.ast).end,
            content_start: content.expression.span(self.ast).start,
            content,
        }
    }

    fn format_conditional_contents(
        &mut self,
        prepared: PreparedConditional<'ast>,
        context: ExpressionContext,
        layout: ConditionalLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let PreparedConditional {
            test,
            consequent,
            alternate,
            question,
            colon,
        } = prepared;
        let operand_indent = match context.position {
            ExpressionPosition::Nested => context.operand_indent,
            _ => OperandIndent::Parent,
        };
        let write_test = |writer: &mut Writer<'ast>| {
            self.format_prepared_expression_with_leading_comments(
                test.original,
                test.content,
                NodeHead {
                    comments: &[],
                    content_start: test.content_start,
                    layout: CommentRunLayout::Source,
                    gap_end: None,
                },
                ExpressionContext::at(ExpressionPosition::ConditionalTest)
                    .with_parent_head(context)
                    .with_operand_indent(operand_indent)
                    .with_content_mode(question.previous_mode())
                    .with_tail_owner(TailOwner::ParentOperation),
                writer,
            )
        };
        let tail = if matches!(layout, ConditionalLayout::Alternate) {
            writer.align(2, write_test)?
        } else {
            write_test(writer)?
        };
        let question = self.resolve_conditional_boundary(question, tail, writer)?;
        let options = self.options;
        let write_branches = |writer: &mut Writer<'ast>| {
            let colon = self.format_conditional_branch(
                ConditionalBranch {
                    operand: consequent,
                    operand_indent,
                    position: ExpressionPosition::ConditionalConsequent,
                    boundary: question,
                    following_mode: colon.previous_mode(),
                },
                writer,
                |formatter, tail, writer| {
                    formatter.resolve_conditional_boundary(colon, tail, writer)
                },
            )?;
            self.format_conditional_branch(
                ConditionalBranch {
                    operand: alternate,
                    operand_indent,
                    position: ExpressionPosition::ConditionalAlternate,
                    boundary: colon,
                    following_mode: ContentMode::Formatted,
                },
                writer,
                |_, tail, _| Ok(tail),
            )
        };
        match layout {
            ConditionalLayout::Root | ConditionalLayout::Test => writer.indent(write_branches),
            ConditionalLayout::Consequent if options.indent_style == IndentStyle::Space => {
                let width = usize::from(options.indent_width.value()).saturating_sub(2);
                writer.align(width, write_branches)
            }
            ConditionalLayout::Consequent | ConditionalLayout::Alternate => write_branches(writer),
        }
    }

    fn conditional_boundary(
        &self,
        previous: &ConditionalOperand,
        next: &ConditionalOperand,
        operator: &'static str,
    ) -> Result<ConditionalBoundary<'ast>, FormatError> {
        let span = Span::new(previous.source_end, next.content_start);
        let comments = self.comments.in_range(span);
        let sensitive = self.diagnostic_range_is_sensitive(span);
        if comments.is_empty() && !sensitive {
            return Ok(ConditionalBoundary {
                operator,
                source_range: span,
                comments,
                next_binding: next
                    .content
                    .parentheses
                    .map_or(CommentBinding::Unbound, |removed| removed.head.binding),
                source: None,
            });
        }
        let Some(operator_start) = self.find_token(span, comments, operator) else {
            return Err(FormatError::unsupported(
                "conditional operator boundary",
                None,
                span,
            ));
        };
        let before_count = comments.partition_point(|comment| comment.span.end <= operator_start);
        let (before, after) = comments.split_at(before_count);
        let previous_binding = self.comment_run_binding(before.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.is_ordinary() {
                CommentBinding::Unbound
            } else if semantics.suppression {
                CommentBinding::BoundTo(previous.original.span(self.ast))
            } else if semantics.type_cast
                && !semantics.diagnostic
                && !semantics.file_directive
                && !semantics.optimizer
            {
                // The following source token is the actual ? or : operator,
                // so this annotation has no source cast-pair target.
                CommentBinding::Unbound
            } else {
                CommentBinding::SourceBound
            }
        });
        let next_binding = self.expression_head_binding(after, next.original);
        let binding = previous_binding.combine(next_binding);
        let binding = if sensitive {
            CommentBinding::SourceBound
        } else {
            binding
        };
        let next_binding = next.content.parentheses.map_or(next_binding, |removed| {
            next_binding.combine(removed.head.binding)
        });
        let previous_end = before.last().map_or(span.start, |comment| comment.span.end);
        let next_start = after.first().map_or(span.end, |comment| comment.span.start);
        Ok(ConditionalBoundary {
            operator,
            source_range: span,
            comments,
            next_binding,
            source: Some(ConditionalSourceBoundary {
                operator_start,
                binding,
                previous_mode: self.content_mode(before),
                next_mode: self.content_mode(after),
                before_operator: self.line_constraint(previous_end, operator_start),
                after_operator: self.line_constraint(operator_start + 1, next_start),
            }),
        })
    }

    fn resolve_conditional_boundary(
        &mut self,
        boundary: ConditionalBoundary<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ResolvedConditionalBoundary<'ast>, FormatError> {
        let source_end = tail
            .as_ref()
            .map_or(boundary.source_range.start, |tail| tail.source_end);
        let comments = match tail {
            Some(tail) => {
                let comments = self
                    .comments
                    .between(Span::new(source_end, boundary.source_range.end))?;
                debug_assert!(
                    comments.starts_with(tail.comments),
                    "conditional packet must begin with the preceding operand's pending lease",
                );
                comments
            }
            None => boundary.comments,
        };
        let (trailing, leading, next_mode, before_operator, after_operator) = match boundary.source
        {
            Some(source) => {
                let before_count =
                    comments.partition_point(|comment| comment.span.end <= source.operator_start);
                let (trailing, leading) = if source.binding.allows_relocation(None) {
                    self.split_separator_comments(
                        comments,
                        before_count,
                        source_end,
                        Some(boundary.source_range.end),
                    )
                } else {
                    comments.split_at(before_count)
                };
                (
                    trailing,
                    leading,
                    source.next_mode,
                    source.before_operator,
                    source.after_operator,
                )
            }
            None if comments.is_empty() => (
                &[][..],
                &[][..],
                ContentMode::Formatted,
                LineConstraint::Unconstrained,
                LineConstraint::Unconstrained,
            ),
            None => {
                // An ordinary source-envelope tail can arrive on the direct
                // path. Locate only this original inter-operand operator.
                let Some(operator_start) =
                    self.find_token(boundary.source_range, boundary.comments, boundary.operator)
                else {
                    return Err(FormatError::unsupported(
                        "conditional operator boundary",
                        None,
                        boundary.source_range,
                    ));
                };
                let before_count =
                    comments.partition_point(|comment| comment.span.end <= operator_start);
                let (trailing, leading) = self.split_separator_comments(
                    comments,
                    before_count,
                    source_end,
                    Some(boundary.source_range.end),
                );
                (
                    trailing,
                    leading,
                    ContentMode::Formatted,
                    LineConstraint::Unconstrained,
                    LineConstraint::Unconstrained,
                )
            }
        };
        let annotation = if matches!(boundary.next_binding, CommentBinding::Unbound) {
            self.emitted_head_annotation(leading)
        } else {
            None
        };
        self.comments.claim(comments)?;
        self.format_comment_run(trailing, CommentRunLayout::SourceAfter(source_end), writer)?;
        Ok(ResolvedConditionalBoundary {
            operator: boundary.operator,
            leading,
            annotation,
            next_binding: boundary.next_binding,
            next_mode,
            before_operator,
            after_operator,
        })
    }

    fn format_conditional_branch<T>(
        &mut self,
        branch: ConditionalBranch<'ast>,
        writer: &mut Writer<'ast>,
        finish: impl FnOnce(
            &mut Self,
            Option<ExpressionTail<'ast>>,
            &mut Writer<'ast>,
        ) -> Result<T, FormatError>,
    ) -> Result<T, FormatError> {
        let ConditionalBranch {
            operand,
            operand_indent,
            position,
            boundary,
            following_mode,
        } = branch;
        boundary
            .before_operator
            .write(writer, FlatSeparator::Space, |writer| {
                writer.line();
            });
        writer.text(boundary.operator);
        writer.space();
        let ConditionalOperand {
            original,
            content,
            content_start,
            ..
        } = operand;
        let mode = if following_mode == ContentMode::Verbatim {
            ContentMode::Verbatim
        } else {
            boundary.next_mode
        };
        let nested_consequent = matches!(position, ExpressionPosition::ConditionalConsequent)
            && matches!(content.data, ExprData::ConditionalExpression(_))
            && mode == ContentMode::Formatted;
        if nested_consequent {
            writer.if_flat("(");
        }
        let indent_style = self.options.indent_style;
        let write = |writer: &mut Writer<'ast>| {
            boundary
                .after_operator
                .write(writer, FlatSeparator::Empty, |_| {});
            let source_head = mode == ContentMode::Verbatim
                || !matches!(boundary.next_binding, CommentBinding::Unbound)
                || boundary.leading.iter().any(|comment| {
                    comment.kind != swc_next_ecma_ast::CommentKind::Block
                        || self.line_breaks(comment.span.start, comment.span.end) != 0
                });
            let head = NodeHead {
                comments: boundary.leading,
                content_start,
                layout: if source_head {
                    CommentRunLayout::Source
                } else {
                    // This head follows the emitted operator, so source-owned
                    // opening lines become breakable inline comment gaps.
                    CommentRunLayout::InlineLeading(content_start)
                },
                gap_end: source_head.then_some(content_start),
            };
            let operand_indent = if self.is_nullish_conditional_operand(content.data, position) {
                operand_indent
            } else {
                OperandIndent::Parent
            };
            let context = ExpressionContext::at(position)
                .with_operand_indent(operand_indent)
                .with_content_mode(mode)
                .with_tail_owner(TailOwner::ParentOperation);
            let generated = boundary.annotation.and_then(|annotation| {
                self.emitted_head_enclosure_owner(
                    original,
                    &content,
                    annotation,
                    boundary.next_binding,
                    context,
                )
            });
            let tail = match generated {
                Some(owner) => self.format_prepared_generated_enclosure_with_head(
                    PreparedGeneratedEnclosure { owner, content },
                    head,
                    context,
                    writer,
                ),
                None => self.format_prepared_expression_with_resolved_head(
                    original,
                    content,
                    ResolvedExpressionHead {
                        head,
                        binding: boundary.next_binding,
                    },
                    context,
                    writer,
                ),
            }?;
            if nested_consequent {
                writer.if_flat(")");
            }
            finish(self, tail, writer)
        };
        match indent_style {
            IndentStyle::Space => writer.align(2, write),
            IndentStyle::Tab => writer.indent(write),
        }
    }
}
