//! Assignment layouts and borrowed operator boundaries prepared before printing.

use super::binary::BinaryLike;
use super::chains::RhsCallChainOwnership;
use super::parentheses::{ExpressionContent, ResolvedExpressionHead};
use super::{ExpressionContext, ExpressionPosition, Formatter, SourceTailOwner};
use crate::formatter::bindings::BindingInitializerOwner;
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::literals::StringContext;
use crate::formatter::suppression::{
    ContentMode, DelimitedFollowing, SuppressionPosition, is_suppression_comment,
};
use crate::{
    FormatError,
    document::{Fragment, Writer},
};
use swc_next_ecma_ast::{
    ArgumentData, AssignmentExpression, AssignmentTarget, AssignmentTargetData, CallExpression,
    Comment, CommentKind, Expr, ExprData, GetSpan, PropertyKeyData, SimpleAssignmentTargetData,
    Span, TemplateElement,
};

#[derive(Clone, Copy)]
pub(in crate::formatter) enum AssignmentParent {
    Statement,
    Nested,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum RhsLayout {
    Fluid,
    BreakAfterOperator,
    NeverBreakAfterOperator,
}

impl RhsLayout {
    pub(in crate::formatter) fn operand_indent(self) -> super::OperandIndent {
        match self {
            Self::Fluid | Self::BreakAfterOperator => super::OperandIndent::Parent,
            Self::NeverBreakAfterOperator => super::OperandIndent::Expression,
        }
    }

    pub(in crate::formatter) fn write<'ast, T>(
        self,
        writer: &mut Writer<'ast>,
        separator: impl FnOnce(&mut Writer<'ast>),
        right: impl FnOnce(&mut Writer<'ast>) -> T,
    ) -> T {
        match self {
            Self::Fluid => {
                let group = writer.named_group(false, |writer| writer.indent(separator));
                writer.indent_if_break(group, right)
            }
            Self::BreakAfterOperator => writer.group(false, |writer| {
                writer.indent(|writer| {
                    separator(writer);
                    right(writer)
                })
            }),
            Self::NeverBreakAfterOperator => {
                separator(writer);
                right(writer)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum AssignmentLayout {
    Ordinary(RhsLayout),
    OperatorLine,
    Chain,
    ChainTail,
}

struct RhsContent {
    data: ExprData,
    unary_operand: Option<Expr>,
}

struct AssignmentSuppression {
    gap: Span,
    left: ContentMode,
    right: ContentMode,
}

#[derive(Clone, Copy)]
enum CastLeadingLayout {
    Flat,
    Break,
}

pub(in crate::formatter) struct AssignmentBoundary<'ast> {
    pub(in crate::formatter) before: &'ast [Comment],
    pub(in crate::formatter) moved: &'ast [Comment],
    pub(in crate::formatter) operator_line: &'ast [Comment],
    pub(in crate::formatter) leading: &'ast [Comment],
    pub(in crate::formatter) trailing: &'ast [Comment],
    pub(in crate::formatter) left_end: u32,
    pub(in crate::formatter) operator_span: Span,
    pub(in crate::formatter) right_start: u32,
    pub(in crate::formatter) layout_start: u32,
    pub(in crate::formatter) before_operator: LineConstraint,
    pub(in crate::formatter) after_operator: LineConstraint,
    pub(in crate::formatter) own_line: bool,
    pub(in crate::formatter) mode: ContentMode,
    cast_leading: Option<CastLeadingLayout>,
    enclosed_head_binding: Option<CommentBinding>,
}

impl<'ast> AssignmentBoundary<'ast> {
    pub(in crate::formatter) fn rhs_requires_line(&self, layout: RhsLayout) -> bool {
        self.rhs_starts_on_own_line()
            || !self.operator_line.is_empty()
            // Fluid RHS groups must start after the pending left-side suffix.
            || matches!(layout, RhsLayout::Fluid)
                && self
                    .before
                    .last()
                    .is_some_and(|comment| comment.kind == CommentKind::Line)
    }

    fn rhs_starts_on_own_line(&self) -> bool {
        self.own_line
            && !(self.mode == ContentMode::Formatted
                && self.moved.is_empty()
                && matches!(self.enclosed_head_binding, Some(CommentBinding::Unbound)))
    }

    pub(in crate::formatter) fn resolved_right_head(&self) -> Option<ResolvedExpressionHead<'ast>> {
        self.enclosed_head_binding
            .map(|binding| ResolvedExpressionHead {
                head: self.right_head(),
                binding,
            })
    }

    pub(in crate::formatter) fn right_head(&self) -> NodeHead<'ast> {
        let source_gap = self.own_line
            || self
                .leading
                .iter()
                .any(|comment| comment.kind == CommentKind::Line);
        NodeHead {
            comments: self.leading,
            content_start: self.right_start,
            layout: if source_gap {
                CommentRunLayout::Source
            } else if self.cast_leading.is_some() {
                CommentRunLayout::GroupedLeading(self.layout_start)
            } else {
                CommentRunLayout::FlatLeading(self.layout_start)
            },
            gap_end: source_gap.then_some(self.layout_start),
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_assignment(
        &mut self,
        assignment: AssignmentExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let mut target = self.assignment_source_target(assignment)?;
        let right = assignment.right(self.ast);
        let decorated_function = self
            .assignment_target_is_module_exports(assignment.left(self.ast))
            .then(|| self.decorated_function_call(right))
            .flatten();
        let suppression = if self.has_suppression {
            self.prepare_assignment_suppression(
                Span::new(target.span().end, right.span(self.ast).start),
                target.operator_span,
                right,
            )
        } else {
            None
        };
        if let Some(source) = suppression {
            if source.left == ContentMode::Formatted
                && let Some(pattern) = target.binding()
            {
                target.set_binding_group(self.bound_initializer_binding_group(pattern));
            }
            return writer.group(false, |writer| {
                let leading = self.comments.between(Span::new(
                    assignment.span(self.ast).start,
                    target.span().start,
                ))?;
                self.comments.claim(leading)?;
                self.format_comment_run(
                    leading,
                    CommentRunLayout::Leading(target.span().start),
                    writer,
                )?;
                let tail = self.format_source_target(
                    &target,
                    ExpressionContext::at(ExpressionPosition::MemberAssignmentObject)
                        .with_parent_head(context)
                        .with_content_mode(source.left),
                    writer,
                )?;
                self.format_expression_tail(tail, writer)?;
                let gap =
                    self.format_delimited_source_gap(source.gap, DelimitedFollowing::Node, writer)?;
                gap.following
                    .write(writer, FlatSeparator::Space, Writer::space);
                let parent = match context.position {
                    ExpressionPosition::Statement => AssignmentParent::Statement,
                    _ => AssignmentParent::Nested,
                };
                self.format_expression(
                    right,
                    ExpressionContext::at(ExpressionPosition::AssignmentRight(parent))
                        .with_source_tail_owner(SourceTailOwner::Enclosure)
                        .with_content_mode(source.right)
                        .with_tail_owner(context.tail_owner)
                        .with_tail_boundary(context.tail_boundary)
                        .with_decorated_function_call(decorated_function),
                    writer,
                )
            });
        }
        let content = self.expression_content(right);
        let data = content.data;
        let head = self.assignment_expression_head(&content);
        let operator = target.operator();
        let boundary = self.assignment_boundary(
            target.left_end(),
            head.as_ref().unwrap_or(&content),
            target.operator_span,
            right.span(self.ast).end,
        )?;
        let parent = match context.position {
            ExpressionPosition::Statement => AssignmentParent::Statement,
            _ => AssignmentParent::Nested,
        };
        let right_context = ExpressionContext::at(ExpressionPosition::AssignmentRight(parent))
            .with_source_tail_owner(SourceTailOwner::Enclosure)
            .with_content_mode(boundary.mode)
            .with_tail_owner(context.tail_owner)
            .with_tail_boundary(context.tail_boundary)
            .with_decorated_function_call(decorated_function);
        let mut layout = self.assignment_layout(data, context.position, right_context, &boundary);
        let binding = target.binding();
        if let Some(pattern) = binding {
            let group = match layout {
                AssignmentLayout::Ordinary(rhs_layout) => {
                    let (group, rhs_layout) = self.binding_initializer_layout(
                        pattern,
                        BindingInitializerOwner::Assignment,
                        data,
                        &boundary,
                        rhs_layout,
                    );
                    layout = AssignmentLayout::Ordinary(rhs_layout);
                    group
                }
                _ => self.declaration_binding_group(pattern),
            };
            target.set_binding_group(group);
        }
        let write_left = |formatter: &mut Self, writer: &mut Writer<'ast>| {
            let leading = formatter.comments.between(Span::new(
                assignment.span(formatter.ast).start,
                target.span().start,
            ))?;
            formatter.comments.claim(leading)?;
            formatter.format_comment_run(
                leading,
                CommentRunLayout::Leading(target.span().start),
                writer,
            )?;
            let tail = formatter.format_source_target(
                &target,
                ExpressionContext::at(ExpressionPosition::MemberAssignmentObject)
                    .with_parent_head(context),
                writer,
            )?;
            formatter.format_expression_tail(tail, writer)?;
            formatter.comments.claim(boundary.before)?;
            formatter.format_comment_run(
                boundary.before,
                CommentRunLayout::SourceAfter(boundary.left_end),
                writer,
            )
        };
        let left = if matches!(layout, AssignmentLayout::Ordinary(RhsLayout::Fluid))
            && boundary.mode == ContentMode::Formatted
            && matches!(data, ExprData::Class(class) if class.decorators(self.ast).is_empty())
        {
            let (left, ()) = writer.record_with_value(|writer| write_left(self, writer))?;
            if !left.can_break() {
                layout = AssignmentLayout::Ordinary(RhsLayout::NeverBreakAfterOperator);
            }
            Some(left)
        } else {
            None
        };
        let write = |writer: &mut Writer<'ast>| {
            if let Some(left) = left {
                writer.write_fragment(left);
            } else {
                write_left(self, writer)?;
            }
            boundary
                .before_operator
                .write(writer, FlatSeparator::Space, Writer::space);
            writer.text(operator);
            self.comments.claim(boundary.operator_line)?;
            self.format_comment_run(
                boundary.operator_line,
                CommentRunLayout::SourceAfter(boundary.operator_span.end),
                writer,
            )?;
            let mut right_context = right_context;
            if matches!(layout, AssignmentLayout::ChainTail) {
                right_context = right_context
                    .with_arrow_layout(super::arrows::ArrowLayout::AssignmentContinuation);
            }
            if right_context.is_formatted_conditional_rhs(data) {
                let operand_indent = match layout {
                    AssignmentLayout::Ordinary(rhs_layout) => rhs_layout.operand_indent(),
                    AssignmentLayout::OperatorLine | AssignmentLayout::ChainTail => {
                        super::OperandIndent::Parent
                    }
                    AssignmentLayout::Chain => super::OperandIndent::Expression,
                };
                right_context = right_context.with_operand_indent(operand_indent);
            }
            let write_right = |writer: &mut Writer<'ast>| {
                self.comments.claim(boundary.moved)?;
                self.format_comment_run(
                    boundary.moved,
                    if boundary.own_line {
                        CommentRunLayout::Source
                    } else {
                        CommentRunLayout::FlatLeading(boundary.layout_start)
                    },
                    writer,
                )?;
                if boundary.own_line
                    && let Some(last) = boundary.moved.last()
                {
                    let next = boundary
                        .leading
                        .first()
                        .map_or(boundary.layout_start, |comment| comment.span.start);
                    let before = self.line_breaks(last.span.end, boundary.operator_span.start);
                    let after = self.line_breaks(boundary.operator_span.end, next);
                    // The displaced operator occupied a source line, not an empty line.
                    match before.max(after) {
                        0 => writer.space(),
                        1 => writer.hard_line(),
                        _ => {
                            writer.hard_line();
                            writer.hard_line();
                        }
                    }
                }
                self.comments.claim(boundary.leading)?;
                self.comments.claim(boundary.trailing)?;
                if let Some(head) = boundary.resolved_right_head() {
                    return self.format_prepared_expression_with_resolved_head(
                        right,
                        content,
                        head,
                        right_context,
                        writer,
                    );
                }
                self.format_prepared_expression_with_leading_comments(
                    right,
                    content,
                    boundary.right_head(),
                    right_context,
                    writer,
                )
            };
            let separator = |writer: &mut Writer<'ast>| {
                boundary
                    .after_operator
                    .write(writer, FlatSeparator::Space, |writer| {
                        let rhs_layout = match layout {
                            AssignmentLayout::Ordinary(rhs_layout) => rhs_layout,
                            _ => RhsLayout::BreakAfterOperator,
                        };
                        if boundary.rhs_requires_line(rhs_layout) {
                            writer.hard_line();
                        } else if matches!(
                            layout,
                            AssignmentLayout::Ordinary(RhsLayout::NeverBreakAfterOperator)
                        ) {
                            writer.space();
                        } else {
                            writer.line();
                        }
                    });
            };
            if !boundary.trailing.is_empty() {
                let (right, tail) = writer.record_with_value(write_right)?;
                let rhs_layout = match layout {
                    AssignmentLayout::Ordinary(rhs_layout) => rhs_layout,
                    AssignmentLayout::OperatorLine | AssignmentLayout::ChainTail => {
                        RhsLayout::BreakAfterOperator
                    }
                    AssignmentLayout::Chain => RhsLayout::Fluid,
                };
                self.format_operator_comment_tail(&boundary, rhs_layout, right, writer)?;
                return Ok(tail);
            }
            let tail = match layout {
                AssignmentLayout::Ordinary(rhs_layout)
                    if binding.is_some()
                        && matches!(rhs_layout, RhsLayout::NeverBreakAfterOperator) =>
                {
                    writer.group(false, |writer| {
                        rhs_layout.write(writer, separator, write_right)
                    })
                }
                AssignmentLayout::Ordinary(rhs_layout) => {
                    rhs_layout.write(writer, separator, write_right)
                }
                AssignmentLayout::OperatorLine | AssignmentLayout::ChainTail => {
                    writer.indent(|writer| {
                        separator(writer);
                        write_right(writer)
                    })
                }
                AssignmentLayout::Chain => {
                    separator(writer);
                    write_right(writer)
                }
            }?;
            Ok(tail)
        };
        match layout {
            AssignmentLayout::Chain | AssignmentLayout::ChainTail => write(writer),
            _ => writer.group(false, write),
        }
    }

    /// The ordinary operator packet and its complete operand share one layout decision.
    fn assignment_target_is_module_exports(&self, target: AssignmentTarget) -> bool {
        let ast = self.ast;
        let AssignmentTargetData::SimpleAssignmentTarget(target) =
            ast.assignment_target_data(target)
        else {
            return false;
        };
        let SimpleAssignmentTargetData::MemberExpression(member) =
            ast.simple_assignment_target_data(target)
        else {
            return false;
        };
        if member.computed(ast) || member.optional(ast) {
            return false;
        }
        matches!(
            (ast.expr_data(member.object(ast)), ast.property_key_data(member.property(ast))),
            (ExprData::IdentifierReference(object), PropertyKeyData::IdentifierName(property))
                if ast.get_utf8(object.name(ast)) == "module"
                    && ast.get_utf8(property.name(ast)) == "exports"
        )
    }

    pub(in crate::formatter) fn format_operator_comment_tail(
        &self,
        boundary: &AssignmentBoundary<'ast>,
        layout: RhsLayout,
        right: Fragment<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let Some((line, prefix)) = boundary.trailing.split_last() else {
            writer.write_fragment(right);
            return Ok(());
        };
        let (prefix, ()) = writer.record_with_value(|writer| {
            self.format_comment_run(
                prefix,
                CommentRunLayout::FlatLeading(boundary.layout_start),
                writer,
            )
        })?;
        let (suffix, ()) = writer.record_with_value(|writer| {
            writer.group(false, |writer| {
                self.format_inline_trailing_comments(std::slice::from_ref(line), writer)
            })
        })?;
        if matches!(layout, RhsLayout::NeverBreakAfterOperator) {
            writer.space();
            writer.write_fragment(prefix);
            writer.write_fragment(right);
            writer.write_fragment(suffix);
            return Ok(());
        }
        let (fallback, ()) = writer.record_with_value(|writer| {
            writer.space();
            writer.write_fragment(prefix);
            writer.write_fragment(suffix);
            writer.indent(|writer| {
                writer.hard_line();
                writer.write_fragment(right);
            });
            Ok::<_, FormatError>(())
        })?;
        if matches!(layout, RhsLayout::Fluid) {
            writer.write_fragment(fallback);
            return Ok(());
        }
        let (Some(prefix), Some(right)) = (prefix.try_flat(), right.try_flat()) else {
            writer.write_fragment(fallback);
            return Ok(());
        };
        let (flat, ()) = writer.record_with_value(|writer| {
            writer.space();
            writer.write_flat_fragment(prefix);
            writer.write_flat_fragment(right);
            writer.write_fragment(suffix);
            Ok::<_, FormatError>(())
        })?;
        writer.write_canonical_alternatives(fallback, &[flat.canonical_candidate()], fallback);
        Ok(())
    }

    fn prepare_assignment_suppression(
        &self,
        gap: Span,
        operator: Span,
        right: Expr,
    ) -> Option<AssignmentSuppression> {
        let comments = self.comments.in_range(gap);
        if self.content_mode(comments) != ContentMode::Verbatim {
            return None;
        }
        let mut inner = right;
        while let ExprData::ParenthesizedExpression(node) = self.ast.expr_data(inner) {
            inner = node.expression(self.ast);
        }
        let complex = matches!(
            self.ast.expr_data(inner),
            ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_)
        );
        let mut source = AssignmentSuppression {
            gap,
            left: ContentMode::Formatted,
            right: ContentMode::Formatted,
        };
        for (run, position) in self.suppressed_comment_runs(comments, gap.end) {
            // An own-line RHS marker belongs to the operand's prepared node head.
            if matches!(position, SuppressionPosition::OwnLine) {
                continue;
            }
            for &comment in run {
                if !is_suppression_comment(self.ast, comment) {
                    continue;
                }
                let left = match position {
                    SuppressionPosition::OwnLine => false,
                    SuppressionPosition::EndOfLine => comment.kind == CommentKind::Line && !complex,
                    SuppressionPosition::Inline => comment.span.end <= operator.start,
                };
                if left {
                    source.left = ContentMode::Verbatim;
                } else {
                    source.right = ContentMode::Verbatim;
                }
            }
        }
        (source.left == ContentMode::Verbatim || source.right == ContentMode::Verbatim)
            .then_some(source)
    }

    fn assignment_expression_head(&self, content: &ExpressionContent) -> Option<ExpressionContent> {
        let mut expression = match content.data {
            ExprData::ChainExpression(chain) => chain.expression(self.ast),
            ExprData::MemberExpression(member) => member.object(self.ast),
            ExprData::CallExpression(call) => call.callee(self.ast),
            ExprData::TsInstantiationExpression(node) => node.expression(self.ast),
            _ => return None,
        };
        loop {
            let mut head = self.expression_content(expression);
            if head
                .parentheses
                .is_some_and(|removed| matches!(removed.head.binding, CommentBinding::BoundTo(_)))
            {
                // This run belongs to the enclosed head, not the complete RHS.
                // Its source owner prints any required pair with that target.
                head.expression = expression;
                head.data = self.ast.expr_data(expression);
                head.parentheses = None;
                return Some(head);
            }
            expression = match head.data {
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
                ExprData::MemberExpression(member) => member.object(self.ast),
                ExprData::CallExpression(call) => call.callee(self.ast),
                ExprData::TsInstantiationExpression(node) => node.expression(self.ast),
                _ => return Some(head),
            };
        }
    }

    pub(in crate::formatter) fn assignment_boundary(
        &self,
        left_end: u32,
        content: &ExpressionContent,
        operator_span: Span,
        source_right_end: u32,
    ) -> Result<AssignmentBoundary<'ast>, FormatError> {
        let right_start = content.expression.span(self.ast).start;
        let span = Span::new(left_end, right_start);
        let source_comments = self.ast.comments();
        let first = source_comments.partition_point(|comment| comment.span.start < span.start);
        let comments = &source_comments[first..];
        let count = comments.partition_point(|comment| comment.span.end <= span.end);
        let comments = &comments[..count];
        let mut boundary = AssignmentBoundary {
            before: &[],
            moved: &[],
            operator_line: &[],
            leading: &[],
            trailing: &[],
            left_end,
            operator_span,
            right_start,
            layout_start: right_start,
            before_operator: LineConstraint::Unconstrained,
            after_operator: LineConstraint::Unconstrained,
            own_line: false,
            mode: ContentMode::Formatted,
            cast_leading: None,
            enclosed_head_binding: None,
        };
        if let Some(head) = self.assignment_expression_head(content)
            && let Some(removed) = head.parentheses
            && removed.head.has_comments
        {
            boundary.own_line = self
                .comments
                .in_range(Span::new(
                    removed.source.outer.start,
                    head.expression.span(self.ast).start,
                ))
                .iter()
                .any(|comment| comment.kind == CommentKind::Line);
        }
        if let ExprData::ParenthesizedExpression(parenthesized) = content.data
            && let Some(child) =
                self.ignored_parentheses_child(parenthesized, ContentMode::Formatted)
            && self
                .expression_needs_parentheses(
                    self.ast.expr_data(child),
                    ExpressionPosition::Nested,
                    ContentMode::Verbatim,
                )
                .is_some()
        {
            let start = parenthesized.span(self.ast).start;
            let end = child.span(self.ast).start;
            let first = source_comments.partition_point(|comment| comment.span.start < start);
            let head = &source_comments[first..];
            let count = head.partition_point(|comment| comment.span.end <= end);
            let head = &head[..count];
            boundary.own_line = head.iter().any(|comment| comment.kind == CommentKind::Line);
        }
        let sensitive = self.diagnostic_range_is_sensitive(span);
        if comments.is_empty() && !sensitive {
            return Ok(boundary);
        }
        let operator_start = operator_span.start;
        let before_count = comments.partition_point(|comment| comment.span.end <= operator_start);
        let (before, after) = comments.split_at(before_count);
        let hugs_comments = matches!(
            content.data,
            ExprData::ArrayExpression(_)
                | ExprData::ObjectExpression(_)
                | ExprData::TemplateLiteral(_)
                | ExprData::TaggedTemplateExpression(_)
        );
        let before_line = before
            .last()
            .is_some_and(|last| last.kind == CommentKind::Line);
        let promoted = !sensitive
            && matches!(
                self.comment_binding(before, span, None),
                CommentBinding::Unbound
            )
            && before.first().is_some_and(|first| {
                hugs_comments && before_line
                    || self.line_breaks(left_end, first.span.start) != 0
                    || before.last().is_some_and(|last| {
                        last.kind == CommentKind::Block
                            && self.line_breaks(last.span.end, operator_start) != 0
                    })
            });
        if promoted {
            boundary.moved = before;
        } else {
            boundary.before = before;
        }
        let jsx_head = matches!(
            content.data,
            ExprData::JsxElement(_) | ExprData::JsxFragment(_)
        );
        let mut line_count = 0;
        if !promoted && !before_line && !hugs_comments {
            let operator_tail_end = content
                .parentheses
                .filter(|_| jsx_head)
                .map_or(right_start, |removed| removed.source.outer.start);
            let mut end = boundary.operator_span.end;
            for (index, comment) in after.iter().enumerate() {
                if comment.span.start >= operator_tail_end
                    || self.line_breaks(end, comment.span.start) != 0
                {
                    break;
                }
                if comment.kind == CommentKind::Line {
                    line_count = index + 1;
                    break;
                }
                end = comment.span.end;
            }
        }
        let (operator_line, leading) = after.split_at(line_count);
        let rhs_comments = &source_comments[first + count..];
        let source_clear_right = rhs_comments
            .first()
            .is_none_or(|comment| comment.span.start >= source_right_end);
        let left_tail = !sensitive
            && !promoted
            && before_line
            && after.is_empty()
            && self
                .prepare_delimited_trailing_run(before, left_end, operator_start, true, None)
                .is_some_and(|run| run.len() == before.len());
        // A movable left suffix and an operator-line suffix have the same
        // emitted owner. Resolve both before choosing the complete RHS layout.
        let tail = if left_tail { before } else { operator_line };
        let ordinary_tail = !tail.is_empty()
            && leading.is_empty()
            && source_clear_right
            && !matches!(
                content.data,
                ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
            )
            && matches!(
                self.comment_binding(tail, span, None),
                CommentBinding::Unbound
            )
            && content
                .parentheses
                .is_none_or(|removed| matches!(removed.head.binding, CommentBinding::Unbound));
        let leading = if ordinary_tail {
            if left_tail {
                boundary.before = &[];
            }
            boundary.trailing = tail;
            leading
        } else {
            boundary.operator_line = operator_line;
            leading
        };
        boundary.leading = leading;
        if let Some(parentheses) = content.parentheses.map(|removed| removed.source)
            && leading
                .last()
                .is_none_or(|comment| comment.span.end <= parentheses.outer.start)
        {
            // A delimiter before the head owns its inner source line breaks.
            boundary.layout_start = parentheses.outer.start;
        }
        boundary.mode = if promoted && self.content_mode(before) == ContentMode::Verbatim {
            ContentMode::Verbatim
        } else {
            self.content_mode(after)
        };
        boundary.own_line |= promoted
            && before.iter().any(|comment| {
                comment.kind == CommentKind::Line
                    || self.source_line_starts_line(comment.span.start)
                    || self.line_breaks(comment.span.start, comment.span.end) != 0
            });
        if !leading.is_empty()
            && (jsx_head
                || matches!(content.data, ExprData::SequenceExpression(sequence)
                if self.sequence_head_has_type_cast(sequence, leading)
                    && source_right_end == content.parentheses.map_or(
                        content.expression.span(self.ast).end,
                        |removed| removed.source.outer.end,
                    )))
        {
            let binding = self.expression_head_binding(leading, content.expression);
            boundary.enclosed_head_binding = Some(
                content
                    .parentheses
                    .map_or(binding, |removed| binding.combine(removed.head.binding)),
            );
        }
        if !leading.is_empty() && self.parenthesis_has_type_cast(right_start) {
            let breaks = leading.iter().enumerate().any(|(index, comment)| {
                let next = leading
                    .get(index + 1)
                    .map_or(right_start, |next| next.span.start);
                comment.kind == CommentKind::Line || self.line_breaks(comment.span.end, next) != 0
            });
            boundary.cast_leading = Some(if breaks {
                CastLeadingLayout::Break
            } else {
                CastLeadingLayout::Flat
            });
        }
        for &comment in leading {
            boundary.own_line |= before_line
                || comment.kind == CommentKind::Line
                || self.source_line_starts_line(comment.span.start)
                    && !matches!(boundary.cast_leading, Some(CastLeadingLayout::Flat))
                || self.comment_is_alignable(comment)?;
        }
        if sensitive {
            let previous = before.last().map_or(left_end, |last| last.span.end);
            let next = after.first().map_or(right_start, |first| first.span.start);
            boundary.before_operator = self.line_constraint(previous, operator_start);
            boundary.after_operator = self.line_constraint(boundary.operator_span.end, next);
        }
        if let Some((last, previous)) = boundary.before.split_last()
            && last.kind == CommentKind::Line
            && self.line_breaks(
                previous.last().map_or(left_end, |comment| comment.span.end),
                last.span.start,
            ) != 0
            && boundary.before_operator == LineConstraint::Unconstrained
        {
            // Own-line comments are emitted directly, unlike deferred line suffixes.
            boundary.before_operator = LineConstraint::NewLine(1);
        }
        Ok(boundary)
    }

    fn assignment_layout(
        &self,
        data: ExprData,
        position: ExpressionPosition,
        right_context: ExpressionContext,
        boundary: &AssignmentBoundary<'ast>,
    ) -> AssignmentLayout {
        let tail = !matches!(data, ExprData::AssignmentExpression(_));
        if let ExpressionPosition::AssignmentRight(parent) | ExpressionPosition::Initializer(parent) =
            position
            && (!tail || matches!(parent, AssignmentParent::Nested))
        {
            return if tail {
                AssignmentLayout::ChainTail
            } else {
                AssignmentLayout::Chain
            };
        }
        if !boundary.operator_line.is_empty()
            || boundary
                .before
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
        {
            return AssignmentLayout::OperatorLine;
        }
        AssignmentLayout::Ordinary(self.ordinary_rhs_layout(data, right_context, boundary))
    }

    pub(in crate::formatter) fn ordinary_rhs_layout(
        &self,
        data: ExprData,
        context: ExpressionContext,
        boundary: &AssignmentBoundary<'ast>,
    ) -> RhsLayout {
        if boundary.rhs_starts_on_own_line()
            || !boundary.operator_line.is_empty()
            || matches!(boundary.cast_leading, Some(CastLeadingLayout::Break))
        {
            return RhsLayout::BreakAfterOperator;
        }
        // A removable ignored source pair prints its child in this RHS scope.
        let data = self
            .source_parentheses_head_content(data, context.with_content_mode(boundary.mode))
            .data;
        if matches!(data, ExprData::CallExpression(call)
            if !call.optional(self.ast) && self.is_require_callee(call.callee(self.ast)))
            || matches!(
                data,
                ExprData::BooleanLiteral(_) | ExprData::NumericLiteral(_)
            )
        {
            return RhsLayout::NeverBreakAfterOperator;
        }
        let prefix_start = boundary
            .leading
            .first()
            .map_or(boundary.right_start, |comment| comment.span.start);
        self.property_rhs_layout(data, prefix_start, false)
    }

    pub(in crate::formatter) fn property_rhs_layout(
        &self,
        data: ExprData,
        prefix_start: u32,
        has_short_key: bool,
    ) -> RhsLayout {
        let Some(content) = self.assignment_right_fluid_content(data) else {
            return RhsLayout::BreakAfterOperator;
        };
        if has_short_key {
            return RhsLayout::NeverBreakAfterOperator;
        }
        if self.assignment_right_prefers_operator_break(content, prefix_start) {
            return RhsLayout::BreakAfterOperator;
        }
        RhsLayout::Fluid
    }

    /// Return the inspected operand unless the existing RHS rule requires an outer break.
    /// Callers can continue classifying it without walking the unary prefix again.
    fn assignment_right_fluid_content(&self, mut data: ExprData) -> Option<RhsContent> {
        let mut content = RhsContent {
            data,
            unary_operand: None,
        };
        match data {
            ExprData::Class(class) if !class.decorators(self.ast).is_empty() => return None,
            ExprData::AssignmentExpression(assignment) => {
                return (!matches!(
                    self.expression_content(assignment.right(self.ast)).data,
                    ExprData::AssignmentExpression(_)
                ))
                .then_some(content);
            }
            ExprData::ConditionalExpression(conditional) => {
                let test = self.expression_content(conditional.test(self.ast)).data;
                let break_after = BinaryLike::from_data(test, self.ast).is_some_and(|binary| {
                    !self.inline_logical_operand(binary.operator, binary.right)
                });
                return (!break_after).then_some(content);
            }
            _ => {}
        }
        if let Some(binary) = BinaryLike::from_data(data, self.ast) {
            return self
                .inline_logical_operand(binary.operator, binary.right)
                .then_some(content);
        }
        while let ExprData::UnaryExpression(unary) = data {
            let operand = self.expression_content(unary.argument(self.ast));
            content.unary_operand = Some(operand.expression);
            data = operand.data;
        }
        content.data = data;
        (!matches!(data, ExprData::SequenceExpression(_))).then_some(content)
    }

    /// Strings and compact member/call spines favor the enclosing operator break.
    /// Fluent chains retain their own operation and argument groups.
    fn assignment_right_prefers_operator_break(
        &self,
        content: RhsContent,
        prefix_start: u32,
    ) -> bool {
        if let Some(operand) = content.unary_operand
            && !self.rhs_head_is_inspectable(prefix_start, operand)
        {
            return false;
        }
        let mut data = content.data;
        let mut chain = false;
        let mut outer_span = match data {
            ExprData::MemberExpression(member) => Some(member.span(self.ast)),
            ExprData::CallExpression(call) => Some(call.span(self.ast)),
            _ => None,
        };
        loop {
            let receiver = match data {
                ExprData::UnaryExpression(unary) if !chain => {
                    let operand = self.expression_content(unary.argument(self.ast));
                    if !self.rhs_head_is_inspectable(unary.span(self.ast).start, operand.expression)
                    {
                        return false;
                    }
                    data = operand.data;
                    continue;
                }
                ExprData::AwaitExpression(awaited) if !chain => awaited.argument(self.ast),
                ExprData::YieldExpression(yielded) if !chain => {
                    let Some(argument) = yielded.argument(self.ast) else {
                        return false;
                    };
                    argument
                }
                ExprData::MemberExpression(access) => {
                    outer_span.get_or_insert(access.span(self.ast));
                    chain = true;
                    access.object(self.ast)
                }
                ExprData::CallExpression(call) => {
                    let covered_span = *outer_span.get_or_insert(call.span(self.ast));
                    if content.unary_operand.is_none()
                        && !self.rhs_head_is_inspectable(prefix_start, Expr::CallExpression(call))
                    {
                        return false;
                    }
                    return self.rhs_call_chain_ownership(call, covered_span)
                        == Some(RhsCallChainOwnership::Compact);
                }
                ExprData::StringLiteral(_) if !chain => return true,
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
                ExprData::TsNonNullExpression(assertion) => assertion.expression(self.ast),
                _ => {
                    if !chain
                        || !matches!(
                            data,
                            ExprData::IdentifierReference(_) | ExprData::ThisExpression(_)
                        )
                    {
                        return false;
                    }
                    return true;
                }
            };
            data = self.expression_content(receiver).data;
        }
    }

    /// Classify values after the chain owner has checked diagnostic coverage.
    /// It separately validates each frame's immutable comment window.
    pub(super) fn assignment_call_is_short(&self, call: CallExpression) -> bool {
        let ast = self.ast;
        let arguments = call.arguments(ast);
        let short = match arguments.len() {
            0 => true,
            1 => {
                let Some(first) = arguments.first() else {
                    return false;
                };
                let argument = ast.get_node_in_sub_range(first);
                let ArgumentData::Expr(expression) = ast.argument_data(argument) else {
                    return false;
                };
                self.assignment_call_argument_is_short(expression)
            }
            _ => false,
        };
        short
            && call
                .type_arguments(ast)
                .is_none_or(|types| self.call_type_arguments_are_compact(types))
    }

    fn assignment_call_argument_is_short(&self, mut expression: Expr) -> bool {
        let ast = self.ast;
        let threshold = usize::from(self.options.line_width) / 4;
        loop {
            let data = self.expression_content(expression).data;
            return match data {
                ExprData::IdentifierReference(identifier) => {
                    rhs_text_is_short(ast.get_utf8(identifier.name(ast)), threshold)
                }
                ExprData::UnaryExpression(unary) => {
                    expression = unary.argument(ast);
                    continue;
                }
                ExprData::RegExpLiteral(literal) => {
                    rhs_text_is_short(ast.get_utf8(literal.pattern(ast)), threshold)
                }
                ExprData::StringLiteral(literal) => {
                    let text = self.string_literal_text(literal, StringContext::Expression);
                    rhs_text_is_short(&text, threshold)
                }
                ExprData::TemplateLiteral(template) => {
                    if !template.expressions(ast).is_empty() || template.quasis(ast).len() != 1 {
                        return false;
                    }
                    let Some(first) = template.quasis(ast).first() else {
                        return false;
                    };
                    let quasi: TemplateElement = ast.get_node_in_sub_range(first);
                    let raw = ast.get_utf8(quasi.raw(ast));
                    !raw.contains('\n') && rhs_text_is_short(raw, threshold)
                }
                ExprData::CallExpression(call) => {
                    call.arguments(ast).is_empty()
                        && threshold >= 2
                        && matches!(
                            ast.expr_data(call.callee(ast)),
                            ExprData::IdentifierReference(identifier)
                                if rhs_text_is_short(ast.get_utf8(identifier.name(ast)), threshold - 2)
                        )
                }
                ExprData::ThisExpression(_)
                | ExprData::NullLiteral(_)
                | ExprData::BigIntLiteral(_)
                | ExprData::BooleanLiteral(_)
                | ExprData::NumericLiteral(_) => true,
                _ => false,
            };
        }
    }

    fn rhs_head_is_inspectable(&self, start: u32, operand: Expr) -> bool {
        if !self.has_suppression
            && !self.has_target_annotations
            && !self.has_file_directives
            && self.diagnostic_lines.is_empty()
        {
            return true;
        }
        // Keep the actual head's binding opaque before width-driven printing.
        // Unary may generate a source pair around these comments on the next pass.
        let span = operand.span(self.ast);
        let comments = self.ast.comments();
        let first = comments.partition_point(|comment| comment.span.start < start);
        let comments = &comments[first..];
        let count = comments.partition_point(|comment| comment.span.end <= span.start);
        self.expression_head_binding(&comments[..count], operand)
            .allows_relocation(Some(span))
    }
}

/// The lone-short policy counts JavaScript string length, not display columns.
fn rhs_text_is_short(text: &str, threshold: usize) -> bool {
    text.len() <= threshold || text.encode_utf16().take(threshold + 1).count() <= threshold
}
