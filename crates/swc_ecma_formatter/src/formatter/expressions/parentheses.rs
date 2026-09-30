//! Source delimiter ownership and semantic boundaries.

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, ParenthesizedExpression, Span,
};

use super::{
    ExpressionContext, ExpressionPosition, ExpressionStart, Formatter, OperandIndent,
    ParenthesesOwner, SourceTailOwner, TailOwner,
};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
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
    Unresolved,
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

pub(in crate::formatter) struct ResolvedExpressionHead<'ast> {
    pub(in crate::formatter) head: NodeHead<'ast>,
    pub(in crate::formatter) binding: CommentBinding,
}

#[derive(Clone, Copy)]
pub(super) struct EmittedHeadAnnotation {
    comment: Comment,
}

#[derive(Clone, Copy)]
pub(super) struct GeneratedEnclosureOwner {
    expression: Expr,
    annotation: Comment,
}

pub(super) struct PreparedGeneratedEnclosure {
    pub(super) owner: GeneratedEnclosureOwner,
    pub(super) content: ExpressionContent,
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
    FunctionReceiver,
}

struct RetainedSourceParentheses {
    boundary: SourceParentheses,
    kind: RetainedParentheses,
    hug: bool,
    leading_mode: Option<ContentMode>,
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
                .expression_needs_parentheses(
                    self.ast.expr_data(child),
                    ExpressionPosition::Nested,
                    ContentMode::Formatted,
                )
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
        }) && self.expression_needs_parentheses(
            self.ast.expr_data(child),
            position,
            ContentMode::Verbatim,
        ) == Some(false)
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

    pub(super) fn source_cast_head(&self, mut expression: Expr) -> Option<ParenthesizedExpression> {
        if !self.has_type_casts {
            return None;
        }
        loop {
            expression = match self.ast.expr_data(expression) {
                ExprData::ParenthesizedExpression(parenthesized) => {
                    if self.parenthesis_has_type_cast(parenthesized.span(self.ast).start) {
                        return Some(parenthesized);
                    }
                    parenthesized.expression(self.ast)
                }
                ExprData::MemberExpression(member) => member.object(self.ast),
                ExprData::CallExpression(call) => call.callee(self.ast),
                _ => return None,
            };
        }
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
        let function_target = match boundary.inner_target {
            ParenthesisInner::Expression(expression)
                if matches!(
                    self.ast.expr_data(expression),
                    ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
                ) =>
            {
                Some(expression.span(self.ast))
            }
            _ => None,
        };
        let function_binding = |comment: Comment| {
            function_target
                .filter(|_| {
                    let raw = self.source(comment.span);
                    raw.starts_with("/**") && !raw.starts_with("/***")
                })
                .map_or(CommentBinding::Unbound, CommentBinding::BoundTo)
        };
        let mut cast_binding = None;
        let resolve = |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.is_ordinary() {
                return function_binding(comment);
            }
            if !semantics.type_cast
                || semantics.suppression
                || semantics.diagnostic
                || semantics.file_directive
                || semantics.optimizer
            {
                return CommentBinding::SourceBound;
            }
            let binding = *cast_binding.get_or_insert_with(|| {
                if cast().is_none() {
                    return CommentBinding::Unbound;
                }
                let mut expression = match boundary.inner_target {
                    ParenthesisInner::Expression(expression) => expression,
                    ParenthesisInner::SourcePair => return CommentBinding::BoundTo(boundary.inner),
                    ParenthesisInner::Unresolved => return CommentBinding::SourceBound,
                };
                loop {
                    if expression.span(self.ast).start != boundary.inner.start {
                        return CommentBinding::SourceBound;
                    }
                    let data = self.ast.expr_data(expression);
                    if let ExprData::ParenthesizedExpression(node) = data {
                        return CommentBinding::BoundTo(node.span(self.ast));
                    }
                    if let ExprData::CallExpression(node) = data {
                        expression = node.callee(self.ast);
                        continue;
                    }
                    if let ExprData::MemberExpression(node) = data {
                        expression = node.object(self.ast);
                        continue;
                    }
                    if let ExprData::ConditionalExpression(node) = data {
                        expression = node.test(self.ast);
                        continue;
                    }
                    if let ExprData::SequenceExpression(node) = data {
                        let Some(first) = node.expressions(self.ast).first() else {
                            return CommentBinding::SourceBound;
                        };
                        expression = self.ast.get_node_in_sub_range(first);
                        continue;
                    }
                    match super::binary::BinaryLike::from_data(data, self.ast) {
                        Some(binary) => expression = binary.left,
                        None => return CommentBinding::Unbound,
                    }
                }
            });
            match binding {
                CommentBinding::Unbound => function_binding(comment),
                _ => binding,
            }
        };
        if function_target.is_none() {
            return self.comment_run_binding(comments.iter().copied(), resolve);
        }
        CommentBinding::combine_all(comments.iter().copied().map(resolve))
    }

    pub(super) fn property_rhs_content(
        &self,
        content: &ExpressionContent,
        mode: ContentMode,
    ) -> ExprData {
        self.source_parentheses_rhs_content(
            content.data,
            ExpressionContext::at(ExpressionPosition::PropertyValue).with_content_mode(mode),
        )
    }

    pub(super) fn source_parentheses_rhs_content(
        &self,
        mut data: ExprData,
        mut context: ExpressionContext,
    ) -> ExprData {
        let original = data;
        let mut ignored = false;
        if !self.has_suppression {
            return data;
        }
        while let ExprData::ParenthesizedExpression(parenthesized) = data {
            if context.content_mode == ContentMode::Verbatim
                && !self.verbatim_parentheses_are_removable(parenthesized, context.position)
            {
                break;
            }
            let (mode, child) = match self.source_parentheses_choice(parenthesized, context) {
                SourceParenthesesChoice::Content {
                    mode,
                    ignored_child: Some(child),
                } => (mode, child),
                _ => break,
            };
            context.content_mode = mode;
            ignored = true;
            data = self.ast.expr_data(child);
        }
        if ignored { data } else { original }
    }

    /// Only the printer that emits this pair can claim its removed source tail.
    pub(super) fn format_parentheses_tail(
        &mut self,
        owner: Option<SourceParenthesesTail>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let Some(owner) = owner else {
            return Ok(tail);
        };
        let tail = self.capture_expression_tail(owner.span, tail)?;
        let line_comment = tail.as_ref().is_some_and(|tail| {
            tail.comments
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
        });
        self.format_expression_tail(tail, writer)?;
        if line_comment {
            writer.hard_line();
        }
        Ok(None)
    }

    pub(super) fn ignored_parentheses_child(
        &self,
        parenthesized: ParenthesizedExpression,
        mode: ContentMode,
    ) -> Option<Expr> {
        if !self.has_suppression {
            return None;
        }
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let child_span = child.span(self.ast);
        let mode = self.parentheses_content_mode(
            SourceParentheses {
                outer: span,
                inner: child_span,
                inner_target: ParenthesisInner::Expression(child),
            },
            mode,
        );
        if mode != ContentMode::Verbatim {
            return None;
        }
        let head = self
            .comments
            .in_range(Span::new(span.start, child_span.start));
        let trailing = Span::new(child_span.end, span.end - 1);
        let tail = self.comments.in_range(trailing);
        self.ignored_parentheses_child_with_comments(
            child,
            mode,
            SourceParenthesesComments { head, tail },
        )
    }

    fn ignored_parentheses_child_with_comments(
        &self,
        child: Expr,
        mode: ContentMode,
        comments: SourceParenthesesComments<'ast>,
    ) -> Option<Expr> {
        if !self.has_suppression || mode != ContentMode::Verbatim {
            return None;
        }
        comments
            .head
            .iter()
            .chain(comments.tail)
            .all(|&comment| {
                let semantics = self.comment_semantics(comment);
                semantics.is_ordinary() || semantics.suppression
            })
            .then_some(child)
    }

    fn parentheses_content_mode(
        &self,
        boundary: SourceParentheses,
        mode: ContentMode,
    ) -> ContentMode {
        if mode == ContentMode::Verbatim || !self.has_suppression {
            return mode;
        }
        let leading = self
            .comments
            .in_range(Span::new(boundary.outer.start, boundary.inner.start));
        if self.content_mode(leading) == ContentMode::Verbatim {
            return ContentMode::Verbatim;
        }
        let trailing = Span::new(boundary.inner.end, boundary.outer.end - 1);
        self.trailing_content_mode(trailing, true)
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
            && (matches!(context.position, ExpressionPosition::Statement)
                || context.tail_owner == TailOwner::ParentOperation
                || self.expression_needs_parentheses(
                    self.ast.expr_data(child),
                    context.position,
                    ContentMode::Verbatim,
                ) == Some(true))
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
        if context.parentheses_owner == ParenthesesOwner::Expression
            && matches!(context.position, ExpressionPosition::CallCallee)
        {
            let receiver = if mode == ContentMode::Formatted {
                self.expression_content(child).expression
            } else {
                child
            };
            if matches!(
                self.ast.expr_data(receiver),
                ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
            ) {
                let packet = self.source_parentheses_comments(SourceParentheses {
                    outer: span,
                    inner: receiver.span(self.ast),
                    inner_target: ParenthesisInner::Expression(receiver),
                });
                if !packet.head.is_empty() || !packet.tail.is_empty() {
                    return SourceParenthesesChoice::Retained {
                        kind: RetainedParentheses::FunctionReceiver,
                        mode,
                    };
                }
            }
        }
        if matches!(context.position, ExpressionPosition::ConditionalTest)
            && matches!(
                self.ast.expr_data(child),
                ExprData::ConditionalExpression(_)
            )
        {
            return SourceParenthesesChoice::Retained {
                kind: RetainedParentheses::Operand,
                mode: context.content_mode,
            };
        }
        if mode == ContentMode::Verbatim
            && (matches!(context.position, ExpressionPosition::Statement)
                || trailing_suppression.is_some()
                    && context.tail_owner == TailOwner::ParentOperation
                || matches!(context.position, ExpressionPosition::BinaryOperand { operator, side: super::binary::OperandSide::Left } if super::binary::BinaryLike::from_data(self.ast.expr_data(child), self.ast).is_none_or(|binary| !operator.flattens(binary.operator)))
                || matches!(
                    context.position,
                    ExpressionPosition::UnaryArgument(_)
                        | ExpressionPosition::ConditionalTest
                        | ExpressionPosition::ArrowBody
                        | ExpressionPosition::RestrictedArgument
                        | ExpressionPosition::MemberObject
                        | ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::CallCallee
                        | ExpressionPosition::NewCallee
                )
                || self.parentheses_separate_diagnostic_lines(boundary)
                || self
                    .expression_needs_parentheses(
                        self.ast.expr_data(child),
                        context.position,
                        ContentMode::Verbatim,
                    )
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
        self.format_expression_leading_prepared(
            expression,
            None,
            ResolvedExpressionHead { head, binding },
            context,
            writer,
        )
    }

    pub(super) fn format_expression_leading_prepared(
        &mut self,
        expression: Expr,
        prepared: Option<ExpressionContent>,
        resolved: ResolvedExpressionHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression_head_and_content(expression, prepared, resolved, context, writer)
    }

    fn format_expression_head_and_content(
        &mut self,
        expression: Expr,
        mut prepared: Option<ExpressionContent>,
        resolved: ResolvedExpressionHead<'ast>,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ResolvedExpressionHead { head, binding } = resolved;
        if !head.comments.is_empty() {
            context.arrow_layout = context.arrow_layout.with_leading_comments();
        }
        // Added readability pairs have their own placement proof; the source
        // binding still owns ASI and diagnostic line constraints.
        let pair_binding = self.optional_nullish_head_binding(
            head.comments,
            || {
                let span = expression.span(self.ast);
                SourceParentheses {
                    outer: span,
                    inner: span,
                    inner_target: ParenthesisInner::Expression(expression),
                }
            },
            binding,
        );
        let protected_head = context
            .protected_head_start
            .is_some_and(|start| start == expression.span(self.ast).start);
        let nullish_operand = context.parentheses_owner == ParenthesesOwner::Expression
            && (!head.comments.is_empty()
                || matches!(pair_binding, CommentBinding::SourceBound)
                || protected_head)
            && self.is_nullish_conditional_operand(
                prepared
                    .get_or_insert_with(|| self.expression_content(expression))
                    .data,
                context.position,
            );
        if nullish_operand
            && (matches!(pair_binding, CommentBinding::SourceBound) || protected_head)
        {
            // The enclosing owner withholds this optional pair to preserve its
            // complete source packet's adjacency to the actual lexical head.
            context.parentheses_owner = ParenthesesOwner::Parent;
        }
        let enclosure_binding = if nullish_operand {
            pair_binding
        } else {
            binding
        };
        let parenthesize_head = !head.comments.is_empty()
            && matches!(enclosure_binding, CommentBinding::BoundTo(_))
            && context.parentheses_owner == ParenthesesOwner::Expression
            && self.expression_needs_parentheses(
                prepared
                    .get_or_insert_with(|| self.expression_content(expression))
                    .data,
                context.position,
                ContentMode::Formatted,
            ) == Some(true);
        if parenthesize_head {
            return self
                .format_parenthesized_expression(expression, prepared, head, context, writer);
        }
        if !head.comments.is_empty() && !matches!(pair_binding, CommentBinding::Unbound) {
            context.protected_head_start = Some(expression.span(self.ast).start);
        }
        let generated = if self.has_type_casts
            && !head.comments.is_empty()
            && matches!(binding, CommentBinding::Unbound)
            && context.content_mode == ContentMode::Formatted
            && context.parentheses_owner == ParenthesesOwner::Expression
        {
            let content = prepared.get_or_insert_with(|| self.expression_content(expression));
            self.generated_enclosure_owner(expression, content, &head, binding, context)
        } else {
            None
        };
        context.expression_start =
            self.format_expression_head(expression, head, binding, context, writer)?;
        if let Some(owner) = generated {
            return self.format_generated_enclosure(owner, prepared, context, writer);
        }
        self.expression_layout(expression).write(writer, |writer| {
            self.format_expression_content_prepared(expression, prepared, context, writer)
        })
    }

    fn format_expression_content_prepared(
        &mut self,
        expression: Expr,
        prepared: Option<ExpressionContent>,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(expression) {
            if context.content_mode == ContentMode::Verbatim
                && !self.verbatim_parentheses_are_removable(parenthesized, context.position)
            {
                self.format_verbatim_expression(expression, context, writer)?;
                return Ok(None);
            }
            return self.format_source_parentheses(parenthesized, prepared, context, writer);
        }
        if context.content_mode == ContentMode::Verbatim {
            self.format_verbatim_expression(expression, context, writer)?;
            return Ok(None);
        }
        let expression = prepared
            .as_ref()
            .map_or(expression, |content| content.expression);
        let data = self.ast.expr_data(expression);
        if let ExprData::SequenceExpression(sequence) = data {
            return self.format_sequence(sequence, context, writer);
        }
        let parenthesize = self.expression_is_parenthesized(data, context);
        let binary = super::binary::BinaryLike::from_data(data, self.ast);
        if context.expression_start == ExpressionStart::StatementPending
            && self.options.semicolons == Semicolons::AsNeeded
        {
            let mut asi_guard = parenthesize
                || match data {
                    ExprData::ArrayExpression(_) | ExprData::RegExpLiteral(_) => true,
                    ExprData::UnaryExpression(unary) => matches!(
                        unary.operator(self.ast),
                        swc_next_ecma_ast::UnaryOperator::Positive
                            | swc_next_ecma_ast::UnaryOperator::Negate
                    ),
                    _ => false,
                };
            if let Some(binary) = binary {
                asi_guard |= self.expression_needs_parentheses(
                    self.expression_content(binary.left).data,
                    ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: super::binary::OperandSide::Left,
                    },
                    ContentMode::Formatted,
                ) == Some(true);
            }
            if asi_guard {
                context.expression_start = ExpressionStart::StatementGuard;
            }
        }
        let conditional = matches!(data, ExprData::ConditionalExpression(_));
        if context.expression_start == ExpressionStart::StatementGuard
            && (binary.is_none() && !conditional || parenthesize)
        {
            writer.text(";");
            context.expression_start = ExpressionStart::None;
        }
        if let Some(binary) = binary {
            if parenthesize && Self::binary_grammar_pair_owns_indent(context.position) {
                return writer.group(false, |writer| {
                    writer.text("(");
                    writer.indent(|writer| {
                        writer.soft_line();
                        let tail = self.format_binary_enclosed(binary, writer)?;
                        self.format_expression_tail(tail, writer)
                    })?;
                    writer.soft_line();
                    writer.text(")");
                    Ok(None)
                });
            }
            if parenthesize {
                writer.text("(");
                context.expression_start = ExpressionStart::None;
            }
            let tail = self.format_binary(binary, context, writer)?;
            if parenthesize {
                writer.text(")");
            }
            return Ok(tail);
        }
        if let ExprData::ConditionalExpression(conditional) = data
            && parenthesize
            && matches!(
                context.position,
                ExpressionPosition::CallCallee
                    | ExpressionPosition::NewCallee
                    | ExpressionPosition::MemberObject
                    | ExpressionPosition::MemberAssignmentObject
            )
        {
            return self.format_conditional_enclosed(conditional, context, writer);
        }
        if let ExprData::ConditionalExpression(conditional) = data
            && matches!(context.position, ExpressionPosition::ConditionalTest)
        {
            return writer.group(false, |writer| {
                writer.text("(");
                writer.indent(|writer| {
                    writer.soft_line();
                    let tail = self.format_conditional(
                        conditional,
                        context
                            .with_expression_start(ExpressionStart::None)
                            .with_operand_indent(OperandIndent::Parent),
                        writer,
                    )?;
                    self.format_expression_tail(tail, writer)
                })?;
                writer.soft_line();
                writer.text(")");
                Ok(None)
            });
        }
        if parenthesize {
            writer.text("(");
            context.expression_start = ExpressionStart::None;
        }
        let tail = self.format_expression_data(expression, context, writer)?;
        let tail = if parenthesize {
            self.format_parentheses_tail(context.source_parentheses_tail, tail, writer)?
        } else {
            tail
        };
        if parenthesize {
            writer.text(")");
        }
        Ok(tail)
    }

    pub(super) fn format_expression_head(
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
                ExpressionStart::None,
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
        self.format_node_head(head, context.expression_start, writer)
    }

    pub(super) fn format_node_head(
        &self,
        head: NodeHead<'ast>,
        mut start: ExpressionStart,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        if start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && !head.comments.is_empty()
            && self.parenthesis_has_type_cast(head.content_start)
        {
            writer.text(";");
            start = ExpressionStart::None;
        }
        self.format_comment_run(head.comments, head.layout, writer)?;
        if let Some(end) = head.gap_end
            && let Some(last) = head.comments.last()
        {
            self.comment_gap(last.span.end, end, writer);
        }
        Ok(start)
    }

    fn format_source_parentheses(
        &mut self,
        parenthesized: ParenthesizedExpression,
        prepared: Option<ExpressionContent>,
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
        let content = prepared.unwrap_or_else(|| {
            self.expression_content(Expr::ParenthesizedExpression(parenthesized))
        });
        if ignored_child.is_none()
            && let Some(boundary) = self
                .protected_grammar_parentheses(&content, context)
                .or_else(|| self.sequence_member_parentheses(&content, context))
        {
            return self.format_protected_grammar_parentheses(boundary, content, context, writer);
        }
        if ignored_child.is_none() && content.parentheses.is_none() {
            return self.format_retained_parentheses(
                parenthesized,
                context,
                RetainedParentheses::Annotation,
                writer,
            );
        }
        if context.protected_head_start == Some(span.start) {
            context.protected_head_start = ignored_child
                .is_none()
                .then_some(content.expression.span(self.ast).start);
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
        } else if matches!(content.data, ExprData::SequenceExpression(_))
            && context.content_mode == ContentMode::Formatted
            && ignored_child.is_none()
        {
            CommentRunLayout::GroupedLeading(inner_span.start)
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
        let owns_source_tail = context.source_tail_owner == SourceTailOwner::Enclosure
            && matches!(
                content.data,
                ExprData::SequenceExpression(_) | ExprData::AssignmentExpression(_)
            );
        if ignored_child.is_none()
            && owns_source_tail
            && self.expression_is_parenthesized(content.data, context)
        {
            context.source_parentheses_tail = content.parentheses.and_then(|removed| removed.tail);
        }
        let tail = self.format_expression_leading_prepared(
            inner,
            None,
            ResolvedExpressionHead {
                head: NodeHead {
                    comments: leading,
                    content_start: inner_span.start,
                    layout,
                    gap_end: matches!(context.position, ExpressionPosition::Statement)
                        .then_some(inner_span.start),
                },
                binding,
            },
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
        let (inner, data) = if context.content_mode == ContentMode::Verbatim {
            (child, self.ast.expr_data(child))
        } else {
            let content = self.expression_content(child);
            (content.expression, content.data)
        };
        let inner_span = inner.span(self.ast);
        let leading = self
            .comments
            .between(Span::new(span.start, inner_span.start))?;
        let trailing = self.comments.in_range(Span::new(inner_span.end, span.end));
        let hug = match data {
            ExprData::ArrayExpression(array) => {
                leading.is_empty()
                    && trailing.is_empty()
                    && (!array.elements(self.ast).is_empty()
                        || self.comments.in_range(inner_span).is_empty())
            }
            ExprData::ObjectExpression(object) => {
                leading.is_empty()
                    && trailing.is_empty()
                    && (!object.properties(self.ast).is_empty()
                        || self.comments.in_range(inner_span).is_empty())
            }
            _ => false,
        };
        self.format_retained_source_parentheses(
            RetainedSourceParentheses {
                boundary: SourceParentheses {
                    outer: span,
                    inner: inner_span,
                    inner_target: ParenthesisInner::Expression(inner),
                },
                kind,
                hug,
                leading_mode: None,
            },
            context,
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
        self.format_retained_source_parentheses(
            RetainedSourceParentheses {
                boundary,
                kind,
                hug: false,
                leading_mode: None,
            },
            context,
            writer,
            content,
        )
    }

    fn format_retained_source_parentheses(
        &mut self,
        source: RetainedSourceParentheses,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(
            &mut Self,
            ExpressionContext,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let RetainedSourceParentheses {
            boundary,
            kind,
            hug,
            leading_mode,
        } = source;
        let span = boundary.outer;
        let inner_span = boundary.inner;
        let source_comments = self.source_parentheses_comments(boundary);
        let mode = if context.content_mode == ContentMode::Verbatim {
            ContentMode::Verbatim
        } else {
            leading_mode.unwrap_or_else(|| self.content_mode(source_comments.head))
        };
        let sequence_head = if mode == ContentMode::Formatted
            && let ParenthesisInner::Expression(expression) = boundary.inner_target
            && let ExprData::SequenceExpression(sequence) = self.ast.expr_data(expression)
        {
            self.enclosed_sequence_head_start(sequence)
                .map(|start| (sequence, start))
        } else {
            None
        };
        let head_end = sequence_head.map_or(inner_span.start, |(_, start)| start);
        let source_leading = if sequence_head.is_some() {
            self.comments.in_range(Span::new(span.start, head_end))
        } else {
            source_comments.head
        };
        let leading = self.comments.between(Span::new(span.start, head_end))?;
        let trailing = source_comments.tail;
        let first = source_leading
            .first()
            .map_or(inner_span.start, |comment| comment.span.start);
        let binding = self.source_parentheses_head_binding(source_leading, boundary, || {
            self.source_type_cast(inner_span.start)
        });
        let protected_head_start = (!source_leading.is_empty()
            && !matches!(
                self.optional_nullish_head_binding(source_leading, || boundary, binding),
                CommentBinding::Unbound
            ))
        .then_some(inner_span.start);
        let opening = if !matches!(kind, RetainedParentheses::FunctionReceiver)
            && (leading_mode.is_some() || matches!(binding, CommentBinding::SourceBound))
        {
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
            CommentRunLayout::FlatLeading(head_end)
        } else {
            CommentRunLayout::GroupedLeading(head_end)
        };
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
        }
        let hug = hug
            && !matches!(opening, LineConstraint::NewLine(_))
            && !matches!(closing, LineConstraint::NewLine(_));
        let operand_indent = if context.operand_indent == OperandIndent::Parent
            && !matches!(kind, RetainedParentheses::Cast)
            && matches!(boundary.inner_target, ParenthesisInner::Expression(expression) if matches!(self.ast.expr_data(expression), ExprData::BinaryExpression(_) | ExprData::LogicalExpression(_) | ExprData::ParenthesizedExpression(_)))
        {
            OperandIndent::Parent
        } else {
            OperandIndent::Expression
        };
        let write_contents = |writer: &mut Writer<'ast>, operand_indent| {
            self.comments.claim(leading)?;
            self.format_comment_run(leading, leading_layout, writer)?;
            let position =
                if let (ContentMode::Verbatim, ExpressionPosition::UnaryArgument(operator)) =
                    (mode, context.position)
                {
                    ExpressionPosition::ParenthesizedUnaryArgument(operator)
                } else {
                    ExpressionPosition::Nested
                };
            let mut inner_context = if matches!(kind, RetainedParentheses::FunctionReceiver) {
                context.inside_parentheses()
            } else {
                ExpressionContext::at(position)
            }
            .with_content_mode(mode)
            .with_operand_indent(operand_indent)
            .with_tail_owner(context.tail_owner);
            if !matches!(
                (mode, context.position),
                (ContentMode::Verbatim, ExpressionPosition::UnaryArgument(_))
            ) {
                inner_context = inner_context.inside_parentheses();
            }
            inner_context.member_chain_layout = context.member_chain_layout;
            if context.member_chain_layout == super::MemberChainLayout::Local {
                inner_context.member_layout = context.member_layout;
            }
            inner_context.protected_head_start = protected_head_start;
            if let Some((sequence, _)) = sequence_head {
                inner_context.head_comments = super::HeadComments::ClaimedSequence(sequence);
            }
            let tail = content(self, inner_context, writer)?;
            let tail = self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)?;
            self.format_expression_tail(tail, writer)
        };
        let write = |writer: &mut Writer<'ast>| {
            writer.text("(");
            if hug {
                write_contents(writer, OperandIndent::Expression)?;
            } else {
                writer.indent(|writer| {
                    opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                    write_contents(writer, operand_indent)
                })?;
                if trailing
                    .last()
                    .is_some_and(|comment| comment.kind == CommentKind::Line)
                {
                    writer.hard_line();
                } else {
                    closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
                }
            }
            writer.text(")");
            Ok(None)
        };
        if matches!(kind, RetainedParentheses::FunctionReceiver) {
            write(writer)
        } else {
            writer.group(expand, write)
        }
    }
    pub(super) fn generated_enclosure_owner(
        &self,
        expression: Expr,
        content: &ExpressionContent,
        head: &NodeHead<'ast>,
        binding: CommentBinding,
        context: ExpressionContext,
    ) -> Option<GeneratedEnclosureOwner> {
        if !self.has_type_casts
            || head.comments.is_empty()
            || !matches!(binding, CommentBinding::Unbound)
            || context.content_mode != ContentMode::Formatted
            || context.parentheses_owner != ParenthesesOwner::Expression
            || self.expression_needs_parentheses(
                content.data,
                context.position,
                context.content_mode,
            ) != Some(true)
        {
            return None;
        }
        // This complete exterior run is the emitted adjacency owner. A binary
        // hoist may cross removed source openers before reaching its lexical head.
        let annotation = self.source_cast_before(head.content_start)?;
        if !head.comments.contains(&annotation)
            || self.diagnostic_range_is_sensitive(Span::new(
                annotation.span.start,
                expression.span(self.ast).end,
            ))
        {
            return None;
        }
        Some(GeneratedEnclosureOwner {
            expression,
            annotation,
        })
    }

    pub(super) fn optional_nullish_head_binding(
        &self,
        comments: &[Comment],
        boundary: impl FnOnce() -> SourceParentheses,
        binding: CommentBinding,
    ) -> CommentBinding {
        if !matches!(binding, CommentBinding::SourceBound) || !self.has_target_annotations {
            return binding;
        }
        let mut optimizer = false;
        for &comment in comments {
            let semantics = self.comment_semantics(comment);
            if semantics.type_cast
                || semantics.suppression
                || semantics.diagnostic
                || semantics.file_directive
            {
                return binding;
            }
            optimizer |= semantics.optimizer;
        }
        if optimizer {
            self.optimizer_head_binding(boundary())
        } else {
            binding
        }
    }

    /// Resolve only an exterior packet's actual lexical target. Retained source
    /// pairs and prefix syntax keep the source owner's existing decision.
    fn optimizer_head_binding(&self, boundary: SourceParentheses) -> CommentBinding {
        let ParenthesisInner::Expression(mut expression) = boundary.inner_target else {
            return CommentBinding::SourceBound;
        };
        let mut head_start = boundary.inner.start;
        let mut removed_binding = CommentBinding::Unbound;
        loop {
            if expression.span(self.ast).start != head_start {
                return CommentBinding::SourceBound;
            }
            let data = self.ast.expr_data(expression);
            expression = match data {
                ExprData::ParenthesizedExpression(_) => {
                    let content = self.expression_content(expression);
                    let Some(removed) = content.parentheses else {
                        return CommentBinding::SourceBound;
                    };
                    removed_binding = removed_binding.combine(removed.head.binding);
                    if matches!(removed_binding, CommentBinding::SourceBound) {
                        return CommentBinding::SourceBound;
                    }
                    // Reuse the selected removal and require its known target to
                    // agree with the complete lexical target exposed below.
                    head_start = content.expression.span(self.ast).start;
                    content.expression
                }
                ExprData::CallExpression(_)
                | ExprData::NewExpression(_)
                | ExprData::Function(_)
                | ExprData::ArrowFunctionExpression(_) => {
                    return removed_binding
                        .combine(CommentBinding::BoundTo(expression.span(self.ast)));
                }
                ExprData::MemberExpression(node) => node.object(self.ast),
                ExprData::ConditionalExpression(node) => node.test(self.ast),
                ExprData::SequenceExpression(node) => {
                    let Some(first) = node.expressions(self.ast).first() else {
                        return CommentBinding::SourceBound;
                    };
                    self.ast.get_node_in_sub_range(first)
                }
                ExprData::IdentifierReference(_)
                | ExprData::StringLiteral(_)
                | ExprData::BooleanLiteral(_)
                | ExprData::NullLiteral(_)
                | ExprData::NumericLiteral(_)
                | ExprData::BigIntLiteral(_)
                | ExprData::RegExpLiteral(_)
                | ExprData::ThisExpression(_) => {
                    return if removed_binding.allows_relocation(Some(expression.span(self.ast))) {
                        CommentBinding::Unbound
                    } else {
                        CommentBinding::SourceBound
                    };
                }
                _ => match super::binary::BinaryLike::from_data(data, self.ast) {
                    Some(binary) => binary.left,
                    None => return CommentBinding::SourceBound,
                },
            };
        }
    }

    pub(in crate::formatter) fn expression_head_binding(
        &self,
        comments: &[Comment],
        expression: Expr,
    ) -> CommentBinding {
        let span = expression.span(self.ast);
        self.source_parentheses_head_binding(
            comments,
            SourceParentheses {
                outer: span,
                inner: span,
                inner_target: ParenthesisInner::Expression(expression),
            },
            || self.source_type_cast(span.start),
        )
    }

    pub(super) fn emitted_head_annotation(
        &self,
        comments: &[Comment],
    ) -> Option<EmittedHeadAnnotation> {
        if !self.has_type_casts {
            return None;
        }
        let comment = comments
            .iter()
            .rev()
            .find(|comment| self.source(comment.span).starts_with("/**"))?;
        // Ordinary trivia is transparent; a newer JSDoc replaces the older one.
        self.comment_is_type_cast(*comment)
            .then_some(EmittedHeadAnnotation { comment: *comment })
    }

    pub(super) fn emitted_head_enclosure_owner(
        &self,
        expression: Expr,
        content: &ExpressionContent,
        annotation: EmittedHeadAnnotation,
        binding: CommentBinding,
        context: ExpressionContext,
    ) -> Option<GeneratedEnclosureOwner> {
        if !matches!(binding, CommentBinding::Unbound)
            || context.content_mode != ContentMode::Formatted
            || context.parentheses_owner != ParenthesesOwner::Expression
            || self.expression_needs_parentheses(
                content.data,
                context.position,
                context.content_mode,
            ) != Some(true)
            || self.diagnostic_range_is_sensitive(Span::new(
                annotation.comment.span.start,
                expression.span(self.ast).end,
            ))
        {
            return None;
        }
        Some(GeneratedEnclosureOwner {
            expression,
            annotation: annotation.comment,
        })
    }

    pub(super) fn format_prepared_expression_with_resolved_head(
        &mut self,
        expression: Expr,
        content: ExpressionContent,
        resolved: ResolvedExpressionHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression_leading_prepared(
            expression,
            Some(content),
            resolved,
            context,
            writer,
        )
    }

    pub(in crate::formatter) fn format_prepared_expression_with_leading_comments(
        &mut self,
        expression: Expr,
        content: ExpressionContent,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let binding = self.expression_head_binding(head.comments, expression);
        let binding = content
            .parentheses
            .map_or(binding, |removed| binding.combine(removed.head.binding));
        self.format_prepared_expression_with_resolved_head(
            expression,
            content,
            ResolvedExpressionHead { head, binding },
            context,
            writer,
        )
    }

    pub(super) fn format_prepared_generated_enclosure_with_head(
        &mut self,
        enclosure: PreparedGeneratedEnclosure,
        head: NodeHead<'ast>,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        context.expression_start = self.format_expression_head(
            enclosure.owner.expression,
            head,
            CommentBinding::Unbound,
            context,
            writer,
        )?;
        self.format_prepared_generated_enclosure(enclosure, context, writer)
    }

    pub(super) fn format_prepared_generated_enclosure(
        &mut self,
        enclosure: PreparedGeneratedEnclosure,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_generated_enclosure(enclosure.owner, Some(enclosure.content), context, writer)
    }

    fn format_generated_enclosure(
        &mut self,
        owner: GeneratedEnclosureOwner,
        prepared: Option<ExpressionContent>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let GeneratedEnclosureOwner {
            expression,
            annotation: _annotation,
        } = owner;
        let start = prepared
            .as_ref()
            .map_or(expression.span(self.ast).start, |content| {
                content.expression.span(self.ast).start
            });
        self.format_parenthesized_expression(
            expression,
            prepared,
            NodeHead {
                comments: &[],
                content_start: start,
                layout: CommentRunLayout::FlatLeading(start),
                gap_end: None,
            },
            context,
            writer,
        )
    }

    fn format_parenthesized_expression(
        &mut self,
        expression: Expr,
        mut prepared: Option<ExpressionContent>,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
        }
        let operand_indent = if self.is_nullish_conditional_operand(
            prepared
                .get_or_insert_with(|| self.expression_content(expression))
                .data,
            context.position,
        ) {
            OperandIndent::Parent
        } else {
            OperandIndent::Expression
        };
        writer.group(false, |writer| {
            writer.text("(");
            let tail = writer.indent(|writer| {
                writer.soft_line();
                self.format_comment_run(
                    head.comments,
                    CommentRunLayout::FlatLeading(head.content_start),
                    writer,
                )?;
                let tail = self.format_expression_prepared(
                    expression,
                    prepared,
                    ExpressionContext::at(ExpressionPosition::Nested)
                        .with_operand_indent(operand_indent)
                        .with_content_mode(context.content_mode)
                        .with_tail_owner(context.tail_owner)
                        .inside_parentheses(),
                    writer,
                )?;
                let tail =
                    self.format_parentheses_tail(context.source_parentheses_tail, tail, writer)?;
                if context.tail_owner == TailOwner::ParentOperation {
                    Ok(tail)
                } else {
                    self.format_expression_tail(tail, writer)?;
                    Ok(None)
                }
            })?;
            writer.soft_line();
            writer.text(")");
            Ok(tail)
        })
    }

    pub(super) fn protected_grammar_parentheses(
        &self,
        content: &ExpressionContent,
        context: ExpressionContext,
    ) -> Option<SourceParentheses> {
        if context.content_mode != ContentMode::Formatted {
            return None;
        }
        let boundary = content.parentheses?.source;
        let breakable = match content.data {
            ExprData::BinaryExpression(_) | ExprData::LogicalExpression(_) => {
                Self::binary_grammar_pair_owns_indent(context.position)
            }
            ExprData::ConditionalExpression(_) => {
                matches!(context.position, ExpressionPosition::ConditionalTest)
                    || matches!(
                        context.position,
                        ExpressionPosition::MemberObject
                            | ExpressionPosition::MemberAssignmentObject
                    ) && (context.conditional_closing == super::ConditionalClosing::MemberLine
                        || context.conditional_envelope
                            == super::ConditionalEnvelope::OperationHead)
            }
            _ => false,
        };
        if !breakable
            || self.line_constraint(boundary.inner.end, boundary.outer.end - 1)
                != LineConstraint::SameLine
            || !self.expression_is_parenthesized(content.data, context)
        {
            return None;
        }
        Some(boundary)
    }

    pub(super) fn format_protected_grammar_parentheses(
        &mut self,
        boundary: SourceParentheses,
        content: ExpressionContent,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let context = if Self::binary_grammar_pair_owns_indent(context.position) {
            context.with_operand_indent(OperandIndent::Parent)
        } else {
            context
        };
        self.format_retained_source_parentheses(
            RetainedSourceParentheses {
                boundary,
                kind: RetainedParentheses::Annotation,
                hug: false,
                leading_mode: None,
            },
            context,
            writer,
            |formatter, inner, writer| {
                formatter.format_expression_prepared(
                    content.expression,
                    Some(content),
                    inner.with_tail_owner(context.tail_owner),
                    writer,
                )
            },
        )
    }
}
