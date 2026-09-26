//! Source delimiter ownership and semantic boundaries.

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, ParenthesizedExpression, Span,
};

use super::{ExpressionContext, ExpressionPosition, ExpressionStart, Formatter};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, TrailingSuppression};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(in crate::formatter) struct SourceParentheses {
    pub(in crate::formatter) outer: Span,
    pub(in crate::formatter) inner: Span,
    pub(in crate::formatter) inner_target: ParenthesisInner,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ParenthesisInner {
    Expression(Expr),
    SourcePair,
}

pub(in crate::formatter) struct ExpressionContent {
    pub(in crate::formatter) expression: Expr,
    pub(in crate::formatter) data: ExprData,
    pub(super) parentheses: Option<RemovedExpressionParentheses>,
}

/// A removed source envelope keeps the binding resolved by its real owner.
#[derive(Clone, Copy)]
pub(super) struct RemovedExpressionParentheses {
    pub(super) source: SourceParentheses,
    pub(super) head: SourceParenthesesHead,
    pub(super) tail: Option<SourceParenthesesTail>,
}

/// Binding and immutable head presence are computed by the same source owner.
#[derive(Clone, Copy)]
pub(super) struct SourceParenthesesHead {
    pub(super) binding: CommentBinding,
    pub(super) has_comments: bool,
}

/// The removed source pair gives its emitted replacement this trailing boundary.
#[derive(Clone, Copy)]
pub(super) struct SourceParenthesesTail {
    span: Span,
}

struct RemovableSourceParentheses {
    head: SourceParenthesesHead,
    has_tail_comments: bool,
}

#[derive(Clone, Copy)]
enum ParenthesisCastPosition {
    Outer,
    Inner,
}

struct SourceParenthesesComments<'ast> {
    head: &'ast [Comment],
    tail: &'ast [Comment],
}

enum RetainedParentheses {
    Cast,
    Annotation,
    Operand,
}

enum SourceParenthesesChoice {
    Verbatim,
    Retained {
        kind: RetainedParentheses,
        mode: ContentMode,
    },
    Content {
        mode: ContentMode,
        ignored_child: Option<Expr>,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn expression_content(&self, expression: Expr) -> ExpressionContent {
        let mut inner = expression;
        let mut head = SourceParenthesesHead {
            binding: CommentBinding::Unbound,
            has_comments: false,
        };
        let mut has_tail_comments = false;
        let data = loop {
            let data = self.ast.expr_data(inner);
            let ExprData::ParenthesizedExpression(parenthesized) = data else {
                break data;
            };
            let child = parenthesized.expression(self.ast);
            if self
                .expression_needs_parentheses(self.ast.expr_data(child), ExpressionPosition::Nested)
                .is_none()
            {
                break data;
            }
            let Some(next) = self.removable_parentheses_head(SourceParentheses {
                outer: parenthesized.span(self.ast),
                inner: child.span(self.ast),
                inner_target: ParenthesisInner::Expression(child),
            }) else {
                break data;
            };
            let binding = head.binding.combine(next.head.binding);
            if matches!(binding, CommentBinding::SourceBound) {
                break data;
            }
            head = SourceParenthesesHead {
                binding,
                has_comments: head.has_comments || next.head.has_comments,
            };
            has_tail_comments |= next.has_tail_comments;
            inner = child;
        };
        let parentheses = (inner != expression).then(|| RemovedExpressionParentheses {
            source: SourceParentheses {
                outer: expression.span(self.ast),
                inner: inner.span(self.ast),
                inner_target: ParenthesisInner::Expression(inner),
            },
            head,
            tail: has_tail_comments.then_some(SourceParenthesesTail {
                span: Span::new(inner.span(self.ast).end, expression.span(self.ast).end),
            }),
        });
        ExpressionContent {
            expression: inner,
            data,
            parentheses,
        }
    }

    fn verbatim_parentheses_are_removable(
        &self,
        parenthesized: ParenthesizedExpression,
        position: ExpressionPosition,
    ) -> bool {
        if matches!(position, ExpressionPosition::Statement) {
            return false;
        }
        let child = parenthesized.expression(self.ast);
        self.parentheses_are_removable(SourceParentheses {
            outer: parenthesized.span(self.ast),
            inner: child.span(self.ast),
            inner_target: ParenthesisInner::Expression(child),
        }) && self.expression_needs_parentheses(self.ast.expr_data(child), position) == Some(false)
    }

    pub(in crate::formatter) fn parentheses_are_removable(
        &self,
        boundary: SourceParentheses,
    ) -> bool {
        self.removable_parentheses_head(boundary).is_some()
    }

    fn removable_parentheses_head(
        &self,
        boundary: SourceParentheses,
    ) -> Option<RemovableSourceParentheses> {
        self.removable_parentheses_head_with_casts(boundary, |position| {
            let start = match position {
                ParenthesisCastPosition::Outer => boundary.outer.start,
                ParenthesisCastPosition::Inner => boundary.inner.start,
            };
            self.source_type_cast(start)
        })
    }

    fn removable_parentheses_head_with_casts(
        &self,
        boundary: SourceParentheses,
        mut cast: impl FnMut(ParenthesisCastPosition) -> Option<Comment>,
    ) -> Option<RemovableSourceParentheses> {
        let comments = self.ast.comments();
        if comments.is_empty() {
            return Some(RemovableSourceParentheses {
                head: SourceParenthesesHead {
                    binding: CommentBinding::Unbound,
                    has_comments: false,
                },
                has_tail_comments: false,
            });
        }
        if self.parentheses_separate_diagnostic_lines(boundary) {
            return None;
        }
        let SourceParenthesesComments { head, tail } = self.source_parentheses_comments(boundary);
        let leading_binding = self.source_parentheses_head_binding(head, boundary, || {
            cast(ParenthesisCastPosition::Inner)
        });
        let inner_target = match leading_binding {
            CommentBinding::BoundTo(target) => Some(target),
            _ => None,
        };
        if !leading_binding.allows_relocation(inner_target) {
            return None;
        }
        let trailing_binding = self.comment_run_binding(tail.iter().copied(), |comment| {
            if self.comment_semantics(comment).is_ordinary() {
                CommentBinding::Unbound
            } else {
                CommentBinding::SourceBound
            }
        });
        if !trailing_binding.allows_relocation(None)
            || cast(ParenthesisCastPosition::Outer).is_some()
        {
            return None;
        }
        Some(RemovableSourceParentheses {
            head: SourceParenthesesHead {
                binding: leading_binding,
                has_comments: !head.is_empty(),
            },
            has_tail_comments: !tail.is_empty(),
        })
    }

    fn source_parentheses_comments(
        &self,
        boundary: SourceParentheses,
    ) -> SourceParenthesesComments<'ast> {
        let comments = self.ast.comments();
        let range = |span: Span| {
            let first = comments.partition_point(|comment| comment.span.start < span.start);
            let inside = &comments[first..];
            let count = inside.partition_point(|comment| comment.span.end <= span.end);
            &inside[..count]
        };
        SourceParenthesesComments {
            head: range(Span::new(boundary.outer.start, boundary.inner.start)),
            tail: range(Span::new(boundary.inner.end, boundary.outer.end)),
        }
    }

    fn parentheses_separate_diagnostic_lines(&self, boundary: SourceParentheses) -> bool {
        if !self.diagnostic_range_is_sensitive(boundary.outer) {
            return false;
        }
        // Delimiters can separate covered tokens from the next source line.
        let opening = (self.diagnostic_line_is_sensitive(boundary.outer.start)
            || self.diagnostic_line_is_sensitive(boundary.inner.start))
            && self.line_breaks(boundary.outer.start, boundary.inner.start) != 0;
        if opening {
            return true;
        }
        self.diagnostic_line_is_sensitive(boundary.inner.end)
            && self.line_breaks(boundary.inner.end, boundary.outer.end) != 0
    }

    pub(in crate::formatter) fn capture_expression_tail(
        &self,
        span: Span,
        tail: Option<ExpressionTail<'ast>>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let source_end = tail.as_ref().map_or(span.start, |tail| tail.source_end);
        let comments = self.comments.between(Span::new(source_end, span.end))?;
        Ok((!comments.is_empty()).then_some(ExpressionTail {
            source_end,
            comments,
        }))
    }

    pub(in crate::formatter) fn parenthesis_has_type_cast(&self, start: u32) -> bool {
        self.source_type_cast(start).is_some()
    }

    fn source_type_cast(&self, start: u32) -> Option<Comment> {
        if !self.has_type_casts || !self.ast.source()[start as usize..].starts_with('(') {
            return None;
        }
        self.source_cast_before(start)
    }

    pub(in crate::formatter) fn source_cast_before(&self, start: u32) -> Option<Comment> {
        if !self.has_type_casts {
            return None;
        }
        let comments = self.ast.comments();
        if comments.is_empty() {
            return None;
        }
        let count = comments.partition_point(|comment| comment.span.end <= start);
        self.source_type_cast_before(start, &comments[..count])
    }

    fn source_type_cast_before(&self, start: u32, comments: &[Comment]) -> Option<Comment> {
        let mut next_start = start;
        for &comment in comments.iter().rev() {
            if !self
                .source(Span::new(comment.span.end, next_start))
                .chars()
                .all(is_source_whitespace)
            {
                break;
            }
            // Ordinary trivia is transparent; a newer JSDoc replaces the older one.
            if self.source(comment.span).starts_with("/**") {
                return self.comment_is_type_cast(comment).then_some(comment);
            }
            next_start = comment.span.start;
        }
        None
    }

    fn source_parentheses_head_binding(
        &self,
        comments: &[Comment],
        boundary: SourceParentheses,
        mut cast: impl FnMut() -> Option<Comment>,
    ) -> CommentBinding {
        let mut cast_binding = None;
        self.comment_run_binding(comments.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.is_ordinary() {
                return CommentBinding::Unbound;
            }
            if !semantics.type_cast
                || semantics.suppression
                || semantics.diagnostic
                || semantics.file_directive
                || semantics.optimizer
            {
                return CommentBinding::SourceBound;
            }
            *cast_binding.get_or_insert_with(|| {
                if cast().is_none() {
                    return CommentBinding::Unbound;
                }
                let expression = match boundary.inner_target {
                    ParenthesisInner::Expression(expression) => expression,
                    ParenthesisInner::SourcePair => return CommentBinding::BoundTo(boundary.inner),
                };
                match self.ast.expr_data(expression) {
                    ExprData::ParenthesizedExpression(node) => {
                        CommentBinding::BoundTo(node.span(self.ast))
                    }
                    ExprData::IdentifierReference(_)
                    | ExprData::BooleanLiteral(_)
                    | ExprData::NullLiteral(_)
                    | ExprData::StringLiteral(_)
                    | ExprData::NumericLiteral(_)
                    | ExprData::BigIntLiteral(_)
                    | ExprData::RegExpLiteral(_)
                    | ExprData::ThisExpression(_)
                    | ExprData::UnaryExpression(_)
                    | ExprData::UpdateExpression(_) => CommentBinding::Unbound,
                    _ => CommentBinding::SourceBound,
                }
            })
        })
    }

    fn source_parentheses_choice(
        &self,
        parenthesized: ParenthesizedExpression,
        context: ExpressionContext,
    ) -> SourceParenthesesChoice {
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let child_span = child.span(self.ast);
        let boundary = SourceParentheses {
            outer: span,
            inner: child_span,
            inner_target: ParenthesisInner::Expression(child),
        };
        let comments = self.source_parentheses_comments(boundary);
        let trailing = Span::new(child_span.end, span.end - 1);
        let trailing_suppression =
            self.trailing_suppression_from_comments(trailing, comments.tail, true);
        if matches!(trailing_suppression, Some(TrailingSuppression::Inline))
            && matches!(context.position, ExpressionPosition::Statement)
        {
            return SourceParenthesesChoice::Verbatim;
        }
        let mode =
            if context.content_mode == ContentMode::Verbatim || trailing_suppression.is_some() {
                ContentMode::Verbatim
            } else {
                self.content_mode(comments.head)
            };
        if self.parenthesis_has_type_cast(span.start) {
            return SourceParenthesesChoice::Retained {
                kind: RetainedParentheses::Cast,
                mode,
            };
        }
        if mode == ContentMode::Verbatim
            && (matches!(context.position, ExpressionPosition::Statement)
                || matches!(context.position, ExpressionPosition::UnaryArgument(_))
                || self.parentheses_separate_diagnostic_lines(boundary)
                || self
                    .expression_needs_parentheses(self.ast.expr_data(child), context.position)
                    .is_none())
        {
            return SourceParenthesesChoice::Retained {
                kind: RetainedParentheses::Operand,
                mode,
            };
        }
        let ignored_child = (mode == ContentMode::Verbatim
            && comments.head.iter().chain(comments.tail).all(|&comment| {
                let semantics = self.comment_semantics(comment);
                semantics.is_ordinary() || semantics.suppression
            }))
        .then_some(child);
        SourceParenthesesChoice::Content {
            mode,
            ignored_child,
        }
    }

    pub(in crate::formatter) fn format_expression_with_leading_comments(
        &mut self,
        expression: Expr,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = expression.span(self.ast);
        let binding = self.source_parentheses_head_binding(
            head.comments,
            SourceParentheses {
                outer: span,
                inner: span,
                inner_target: ParenthesisInner::Expression(expression),
            },
            || self.source_type_cast(span.start),
        );
        self.format_expression_leading_prepared(expression, head, binding, context, writer)
    }

    fn format_expression_leading_prepared(
        &mut self,
        expression: Expr,
        head: NodeHead<'ast>,
        binding: CommentBinding,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = expression.span(self.ast);
        if self.diagnostic_line_is_sensitive(span.start)
            && self.line_breaks(span.start, span.end) == 0
        {
            writer.flat(|writer| {
                self.format_expression_head_and_content(expression, head, binding, context, writer)
            })
        } else {
            self.format_expression_head_and_content(expression, head, binding, context, writer)
        }
    }

    fn format_expression_head_and_content(
        &mut self,
        expression: Expr,
        head: NodeHead<'ast>,
        binding: CommentBinding,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(expression) {
            if context.content_mode == ContentMode::Verbatim
                && !self.verbatim_parentheses_are_removable(parenthesized, context.position)
            {
                context.expression_start =
                    self.format_expression_head(expression, head, binding, context, writer)?;
                self.format_verbatim_expression(expression, context, writer)?;
                return Ok(None);
            }
            context.expression_start =
                self.format_expression_head(expression, head, binding, context, writer)?;
            return self.format_source_parentheses(parenthesized, context, writer);
        }
        let data = self.ast.expr_data(expression);
        if context.content_mode == ContentMode::Verbatim {
            context.expression_start =
                self.format_expression_head(expression, head, binding, context, writer)?;
            self.format_verbatim_expression(expression, context, writer)?;
            return Ok(None);
        }
        if self.expression_is_parenthesized(data, context) {
            let generated = matches!(binding, CommentBinding::Unbound)
                && self
                    .source_cast_before(head.content_start)
                    .is_some_and(|annotation| {
                        head.comments.contains(&annotation)
                            && !self.diagnostic_range_is_sensitive(Span::new(
                                annotation.span.start,
                                expression.span(self.ast).end,
                            ))
                    });
            context.expression_start =
                self.format_expression_head(expression, head, binding, context, writer)?;
            if context.expression_start.needs_asi_guard()
                && self.options.semicolons == Semicolons::AsNeeded
            {
                writer.text(";");
            }
            if generated {
                return writer.group(false, |writer| {
                    writer.text("(");
                    writer.indent(|writer| {
                        writer.soft_line();
                        let tail = self.format_expression_data(
                            expression,
                            context.inside_parentheses(),
                            writer,
                        )?;
                        self.format_expression_tail(tail, writer)
                    })?;
                    writer.soft_line();
                    writer.text(")");
                    Ok(None)
                });
            }
            writer.text("(");
            let tail = self.format_expression_data(
                expression,
                context.with_expression_start(ExpressionStart::None),
                writer,
            )?;
            writer.text(")");
            return Ok(tail);
        }
        context.expression_start =
            self.format_expression_head(expression, head, binding, context, writer)?;
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && self.expression_needs_asi_guard(expression, context)?
        {
            writer.text(";");
            context.expression_start = ExpressionStart::StatementGuarded;
        }
        self.format_expression_data(expression, context, writer)
    }

    fn format_expression_head(
        &self,
        expression: Expr,
        mut head: NodeHead<'ast>,
        binding: CommentBinding,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && let Some(index) = head
                .comments
                .iter()
                .position(|&comment| self.comment_has_target_annotation(comment))
            && self.expression_needs_asi_guard(expression, context)?
        {
            let source_bound = matches!(binding, CommentBinding::SourceBound);
            let index = if source_bound { 0 } else { index };
            let (prefix, protected) = head.comments.split_at(index);
            self.format_node_head(
                NodeHead {
                    comments: prefix,
                    content_start: protected[0].span.start,
                    layout: head.layout,
                    gap_end: Some(protected[0].span.start),
                },
                writer,
            )?;
            writer.text(";");
            if source_bound {
                writer.hard_line();
            } else if protected
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
                && !self.comment_is_type_cast(protected[0])
            {
                writer.space();
            }
            context.expression_start = ExpressionStart::StatementGuarded;
            head.comments = protected;
        }
        self.format_node_head(head, writer)?;
        Ok(context.expression_start)
    }

    fn format_node_head(
        &self,
        head: NodeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_comment_run(head.comments, head.layout, writer)?;
        if let Some(end) = head.gap_end
            && let Some(last) = head.comments.last()
        {
            self.comment_gap(last.span.end, end, writer);
        }
        Ok(())
    }

    fn format_source_parentheses(
        &mut self,
        parenthesized: ParenthesizedExpression,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = parenthesized.span(self.ast);
        let ignored_child = match self.source_parentheses_choice(parenthesized, context) {
            SourceParenthesesChoice::Verbatim => {
                self.format_verbatim_expression(
                    Expr::ParenthesizedExpression(parenthesized),
                    context,
                    writer,
                )?;
                return Ok(None);
            }
            SourceParenthesesChoice::Retained { kind, mode } => {
                return self.format_retained_parentheses(
                    parenthesized,
                    context.with_content_mode(mode),
                    kind,
                    writer,
                );
            }
            SourceParenthesesChoice::Content {
                mode,
                ignored_child,
            } => {
                context.content_mode = mode;
                ignored_child
            }
        };
        let content = self.expression_content(Expr::ParenthesizedExpression(parenthesized));
        if ignored_child.is_none() && content.parentheses.is_none() {
            return self.format_retained_parentheses(
                parenthesized,
                context,
                RetainedParentheses::Annotation,
                writer,
            );
        }
        let inner = ignored_child.unwrap_or(content.expression);
        let inner_span = inner.span(self.ast);
        let leading = if ignored_child.is_some()
            || content
                .parentheses
                .is_some_and(|removed| removed.head.has_comments)
        {
            self.comments
                .between(Span::new(span.start, inner_span.start))?
        } else {
            &[]
        };
        self.comments.claim(leading)?;
        let layout = if matches!(context.position, ExpressionPosition::Statement) {
            CommentRunLayout::Source
        } else {
            CommentRunLayout::Leading(inner_span.start)
        };
        let mode = if ignored_child.is_some() {
            ContentMode::Verbatim
        } else {
            self.content_mode(leading)
        };
        let binding = content
            .parentheses
            .map_or(CommentBinding::SourceBound, |removed| removed.head.binding);
        let tail = self.format_expression_leading_prepared(
            inner,
            NodeHead {
                comments: leading,
                content_start: inner_span.start,
                layout,
                gap_end: matches!(context.position, ExpressionPosition::Statement)
                    .then_some(inner_span.start),
            },
            binding,
            context.with_content_mode(mode),
            writer,
        )?;
        let source_tail = content.parentheses.and_then(|removed| removed.tail);
        if ignored_child.is_some() {
            self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)
        } else if let Some(source_tail) = source_tail {
            self.capture_expression_tail(source_tail.span, tail)
        } else {
            Ok(tail)
        }
    }

    fn format_retained_parentheses(
        &mut self,
        parenthesized: ParenthesizedExpression,
        context: ExpressionContext,
        kind: RetainedParentheses,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let inner = if context.content_mode == ContentMode::Verbatim {
            child
        } else {
            self.expression_content(child).expression
        };
        let inner_span = inner.span(self.ast);
        let boundary = SourceParentheses {
            outer: span,
            inner: inner_span,
            inner_target: ParenthesisInner::Expression(inner),
        };
        self.format_retained_source_parentheses(
            boundary,
            context,
            kind,
            writer,
            |formatter, context, writer| formatter.format_expression(inner, context, writer),
        )
    }

    pub(super) fn format_retained_target_parentheses(
        &mut self,
        boundary: SourceParentheses,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(
            &mut Self,
            ExpressionContext,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let kind = if self.parenthesis_has_type_cast(boundary.outer.start) {
            RetainedParentheses::Cast
        } else {
            RetainedParentheses::Annotation
        };
        self.format_retained_source_parentheses(boundary, context, kind, writer, content)
    }

    fn format_retained_source_parentheses(
        &mut self,
        boundary: SourceParentheses,
        context: ExpressionContext,
        kind: RetainedParentheses,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(
            &mut Self,
            ExpressionContext,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = boundary.outer;
        let inner_span = boundary.inner;
        let source_comments = self.source_parentheses_comments(boundary);
        let leading = self
            .comments
            .between(Span::new(span.start, inner_span.start))?;
        let trailing = source_comments.tail;
        let first = source_comments
            .head
            .first()
            .map_or(inner_span.start, |comment| comment.span.start);
        let binding = self.source_parentheses_head_binding(source_comments.head, boundary, || {
            self.source_type_cast(inner_span.start)
        });
        let opening = if matches!(binding, CommentBinding::SourceBound) {
            self.source_line_constraint(span.start + 1, first)
        } else {
            self.line_constraint(span.start + 1, first)
        };
        let last = trailing
            .last()
            .map_or(inner_span.end, |comment| comment.span.end);
        let closing = self.line_constraint(last, span.end - 1);
        let expand = matches!(kind, RetainedParentheses::Cast)
            && (source_comments
                .head
                .first()
                .is_some_and(|comment| self.line_breaks(span.start, comment.span.start) != 0)
                || trailing.first().is_some_and(|comment| {
                    self.line_breaks(inner_span.end, comment.span.start) != 0
                }));
        let leading_layout = if matches!(kind, RetainedParentheses::Annotation) {
            CommentRunLayout::FlatLeading(inner_span.start)
        } else {
            CommentRunLayout::GroupedLeading(inner_span.start)
        };
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
        }
        writer.group(expand, |writer| {
            writer.text("(");
            writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                self.comments.claim(leading)?;
                self.format_comment_run(leading, leading_layout, writer)?;
                let mode = if context.content_mode == ContentMode::Verbatim {
                    ContentMode::Verbatim
                } else {
                    self.content_mode(source_comments.head)
                };
                let position =
                    if let (ContentMode::Verbatim, ExpressionPosition::UnaryArgument(operator)) =
                        (mode, context.position)
                    {
                        ExpressionPosition::ParenthesizedUnaryArgument(operator)
                    } else {
                        ExpressionPosition::Nested
                    };
                let tail = content(
                    self,
                    ExpressionContext::at(position)
                        .with_content_mode(mode)
                        .inside_parentheses(),
                    writer,
                )?;
                let tail =
                    self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)?;
                self.format_expression_tail(tail, writer)
            })?;
            if trailing
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
            {
                writer.hard_line();
            } else {
                closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
            }
            writer.text(")");
            Ok(None)
        })
    }
}
