//! Assignment and update owners recover identifier delimiters from their grammar gaps.

use super::parentheses::{ParenthesisInner, SourceParentheses};
use super::{ExpressionContext, ExpressionPosition, ExpressionStart, Formatter};
use crate::formatter::bindings::BindingGroup;
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, Semicolons, document::Writer};
use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, AssignmentTargetData, Ast, BindingPattern, Comment,
    Expr, ExprData, GetSpan, SimpleAssignmentTarget, SimpleAssignmentTargetData, Span,
    UpdateExpression, UpdateOperator,
};

#[derive(Clone, Copy)]
enum TargetOperator {
    Assignment(AssignmentOperator),
    Update(UpdateOperator),
}
impl TargetOperator {
    fn as_str(self) -> &'static str {
        match self {
            Self::Assignment(operator) => operator.as_str(),
            Self::Update(operator) => operator.as_str(),
        }
    }
}

#[derive(Clone, Copy)]
enum TargetNode {
    Simple(Expr),
    Pattern {
        pattern: BindingPattern,
        group: Option<BindingGroup>,
    },
}
impl TargetNode {
    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Simple(expression) => expression.span(ast),
            Self::Pattern { pattern, .. } => pattern.span(ast),
        }
    }
}

struct PreparedTarget {
    target: TargetNode,
    span: Span,
    layers: Vec<TargetLayer>,
    tail_end: u32,
}

pub(super) struct SourceTarget {
    envelope: PreparedTarget,
    operator: TargetOperator,
    pub(super) operator_span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetLayerMode {
    Remove,
    Retain,
    Verbatim,
}
struct TargetLayer {
    parentheses: SourceParentheses,
    mode: TargetLayerMode,
}

impl SourceTarget {
    pub(super) fn operator(&self) -> &'static str {
        self.operator.as_str()
    }
    pub(super) fn span(&self) -> Span {
        self.envelope.span
    }
    pub(super) fn binding(&self) -> Option<BindingPattern> {
        match self.envelope.target {
            TargetNode::Pattern { pattern, .. } => Some(pattern),
            TargetNode::Simple(_) => None,
        }
    }
    pub(super) fn set_binding_group(&mut self, group: BindingGroup) {
        if let TargetNode::Pattern {
            group: prepared, ..
        } = &mut self.envelope.target
        {
            *prepared = Some(group);
        }
    }
    pub(super) fn needs_asi_guard(&self) -> bool {
        self.binding().is_some() || self.has_retained_parentheses()
    }
    pub(super) fn left_end(&self) -> u32 {
        self.envelope.tail_end
    }
    pub(super) fn has_retained_parentheses(&self) -> bool {
        self.envelope
            .layers
            .iter()
            .any(|layer| layer.mode != TargetLayerMode::Remove)
    }
}

/// Only walks the owner's gaps; complete target and comment bodies are excluded.
struct TargetGap<'a> {
    source: &'a str,
    comments: &'a [Comment],
    position: u32,
    end: u32,
}

impl TargetGap<'_> {
    fn skip_trivia(&mut self) {
        loop {
            if let Some((comment, rest)) = self.comments.split_first()
                && comment.span.start == self.position
            {
                self.position = comment.span.end;
                self.comments = rest;
                continue;
            }
            let remaining = &self.source[self.position as usize..self.end as usize];
            let Some(character) = remaining.chars().next() else {
                break;
            };
            if !is_source_whitespace(character) {
                break;
            }
            self.position += character.len_utf8() as u32;
        }
    }

    fn consume(&mut self, token: &str) -> Option<Span> {
        self.skip_trivia();
        let remaining = &self.source[self.position as usize..self.end as usize];
        if !remaining.starts_with(token) {
            return None;
        }
        let start = self.position;
        self.position += token.len() as u32;
        Some(Span::new(start, self.position))
    }

    fn finished(&mut self) -> bool {
        self.skip_trivia();
        self.position == self.end
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn simple_target_expression(
        &self,
        target: SimpleAssignmentTarget,
    ) -> Result<Expr, FormatError> {
        match self.ast.simple_assignment_target_data(target) {
            SimpleAssignmentTargetData::IdentifierReference(identifier) => {
                Ok(Expr::IdentifierReference(identifier))
            }
            _ => Err(self.unsupported("assignment target", target)),
        }
    }

    pub(in crate::formatter) fn format_simple_assignment_target(
        &mut self,
        target: SimpleAssignmentTarget,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression(self.simple_target_expression(target)?, context, writer)
    }

    pub(super) fn assignment_source_target(
        &self,
        assignment: AssignmentExpression,
    ) -> Result<SourceTarget, FormatError> {
        let left = assignment.left(self.ast);
        let target = match self.ast.assignment_target_data(left) {
            AssignmentTargetData::SimpleAssignmentTarget(node) => {
                TargetNode::Simple(self.simple_target_expression(node)?)
            }
            AssignmentTargetData::ArrayAssignmentTarget(node) => TargetNode::Pattern {
                pattern: BindingPattern::ArrayPattern(node),
                group: None,
            },
            AssignmentTargetData::ObjectAssignmentTarget(node) => TargetNode::Pattern {
                pattern: BindingPattern::ObjectPattern(node),
                group: None,
            },
        };
        if matches!(target, TargetNode::Pattern { .. })
            && assignment.operator(self.ast) != AssignmentOperator::Assign
        {
            return Err(self.unsupported("destructuring compound assignment", assignment));
        }
        let span = target.span(self.ast);
        let mut source = self.prepare_source_target(
            target,
            TargetOperator::Assignment(assignment.operator(self.ast)),
            Span::new(assignment.span(self.ast).start, span.start),
            Span::new(span.end, assignment.right(self.ast).span(self.ast).start),
            None,
        )?;
        if let Some(outer) = source.envelope.layers.first_mut()
            && outer.mode == TargetLayerMode::Remove
            && matches!(target, TargetNode::Simple(_))
            && self.target_pair_is_semantic_boundary(assignment)
        {
            outer.mode = TargetLayerMode::Retain;
        }
        source.envelope.tail_end = source
            .envelope
            .layers
            .iter()
            .find(|layer| layer.mode != TargetLayerMode::Remove)
            .map_or(span.end, |layer| layer.parentheses.outer.end);
        Ok(source)
    }

    fn target_pair_is_semantic_boundary(&self, assignment: AssignmentExpression) -> bool {
        if !matches!(
            assignment.operator(self.ast),
            AssignmentOperator::Assign
                | AssignmentOperator::LogicalAndAssign
                | AssignmentOperator::LogicalOrAssign
                | AssignmentOperator::NullishAssign
        ) {
            return false;
        }
        // Suppressed right operands can still observe anonymous identifier naming.
        let mut expression = assignment.right(self.ast);
        loop {
            expression = match self.ast.expr_data(expression) {
                ExprData::ParenthesizedExpression(node) => node.expression(self.ast),
                ExprData::Function(node) => return node.id(self.ast).is_none(),
                ExprData::Class(node) => return node.id(self.ast).is_none(),
                ExprData::ArrowFunctionExpression(_) => return true,
                _ => return false,
            };
        }
    }

    pub(super) fn update_source_target(
        &self,
        update: UpdateExpression,
    ) -> Result<SourceTarget, FormatError> {
        let argument = update.argument(self.ast);
        let target = TargetNode::Simple(self.simple_target_expression(argument)?);
        let span = update.span(self.ast);
        let target_span = target.span(self.ast);
        let operator = update.operator(self.ast);
        let length = operator.as_str().len() as u32;
        let (prefix, suffix, operator_span) = if update.prefix(self.ast) {
            (
                Span::new(span.start + length, target_span.start),
                Span::new(target_span.end, span.end),
                Span::new(span.start, span.start + length),
            )
        } else {
            (
                Span::new(span.start, target_span.start),
                Span::new(target_span.end, span.end - length),
                Span::new(span.end - length, span.end),
            )
        };
        self.prepare_source_target(
            target,
            TargetOperator::Update(operator),
            prefix,
            suffix,
            Some(operator_span),
        )
    }

    fn prepare_source_target(
        &self,
        target: TargetNode,
        operator: TargetOperator,
        prefix: Span,
        suffix: Span,
        known_operator: Option<Span>,
    ) -> Result<SourceTarget, FormatError> {
        let mut trailing = self.target_gap(suffix);
        let envelope = self.prepare_target_envelope(target, prefix, &mut trailing)?;
        let invalid = || self.unsupported_target_boundary(Span::new(prefix.start, suffix.end));
        let operator_span = match known_operator {
            Some(span) if self.source(span) == operator.as_str() => span,
            Some(_) => return Err(invalid()),
            None => trailing.consume(operator.as_str()).ok_or_else(invalid)?,
        };
        if !trailing.finished() {
            return Err(invalid());
        }
        Ok(SourceTarget {
            envelope,
            operator,
            operator_span,
        })
    }

    fn target_gap(&self, span: Span) -> TargetGap<'_> {
        TargetGap {
            source: self.ast.source(),
            comments: self.comments.in_range(span),
            position: span.start,
            end: span.end,
        }
    }

    fn unsupported_target_boundary(&self, span: Span) -> FormatError {
        FormatError::unsupported("target source boundary", None, span)
    }

    fn prepare_target_envelope(
        &self,
        target: TargetNode,
        prefix: Span,
        trailing: &mut TargetGap<'_>,
    ) -> Result<PreparedTarget, FormatError> {
        let owner = Span::new(prefix.start, trailing.end);
        let invalid = || self.unsupported_target_boundary(owner);
        let mut leading = self.target_gap(prefix);
        let mut layers = Vec::new();
        while let Some(opening) = leading.consume("(") {
            layers.push(TargetLayer {
                parentheses: SourceParentheses {
                    outer: opening,
                    inner: target.span(self.ast),
                    inner_target: ParenthesisInner::Unresolved,
                },
                mode: TargetLayerMode::Remove,
            });
        }
        if !leading.finished() {
            return Err(invalid());
        }
        let mut inner = target.span(self.ast);
        let mut inner_target = match target {
            TargetNode::Simple(expression) => ParenthesisInner::Expression(expression),
            TargetNode::Pattern { .. } => ParenthesisInner::Unresolved,
        };
        for layer in layers.iter_mut().rev() {
            let closing = trailing.consume(")").ok_or_else(invalid)?;
            layer.parentheses.outer.end = closing.end;
            layer.parentheses.inner = inner;
            layer.parentheses.inner_target = inner_target;
            layer.mode = self.target_layer_mode(layer.parentheses);
            inner = layer.parentheses.outer;
            inner_target = ParenthesisInner::SourcePair;
        }
        if let Some(outer) = layers.first_mut()
            && outer.mode == TargetLayerMode::Remove
            && matches!(target, TargetNode::Pattern { .. })
        {
            outer.mode = TargetLayerMode::Retain;
        }
        Ok(PreparedTarget {
            target,
            span: inner,
            layers,
            tail_end: inner.end,
        })
    }

    fn target_layer_mode(&self, boundary: SourceParentheses) -> TargetLayerMode {
        if self.has_suppression {
            let leading = self
                .comments
                .in_range(Span::new(boundary.outer.start, boundary.inner.start));
            let trailing = Span::new(boundary.inner.end, boundary.outer.end - 1);
            if self.content_mode(leading) == ContentMode::Verbatim
                || self
                    .trailing_suppression_from_comments(
                        trailing,
                        self.comments.in_range(trailing),
                        true,
                    )
                    .is_some()
            {
                return TargetLayerMode::Verbatim;
            }
        }
        if self.parentheses_are_removable(boundary) {
            TargetLayerMode::Remove
        } else {
            TargetLayerMode::Retain
        }
    }

    pub(super) fn format_source_target(
        &mut self,
        target: &SourceTarget,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        context.position = match target.operator {
            TargetOperator::Assignment(_) => ExpressionPosition::AssignmentTarget,
            TargetOperator::Update(_) => ExpressionPosition::UpdateArgument,
        };
        if target.binding().is_some()
            && context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
            context.expression_start = ExpressionStart::None;
        }
        if context.content_mode == ContentMode::Verbatim && !target.envelope.layers.is_empty() {
            if context.expression_start.needs_asi_guard()
                && self.options.semicolons == Semicolons::AsNeeded
            {
                writer.text(";");
            }
            self.format_verbatim(target.envelope.span, writer)?;
            return Ok(None);
        }
        self.format_target_layers(
            target.envelope.target,
            &target.envelope.layers,
            target.envelope.tail_end,
            context,
            writer,
        )
    }

    fn format_target_layers(
        &mut self,
        target: TargetNode,
        mut layers: &[TargetLayer],
        tail_end: u32,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let target_end = target.span(self.ast).end;
        let (content_end, tail) = loop {
            let Some((layer, remaining)) = layers.split_first() else {
                break (
                    target_end,
                    match target {
                        TargetNode::Simple(expression) => {
                            self.format_expression(expression, context, writer)?
                        }
                        TargetNode::Pattern { pattern, group } => self.format_binding_with_group(
                            pattern,
                            group.unwrap_or(BindingGroup::Auto),
                            context.content_mode,
                            writer,
                        )?,
                    },
                );
            };
            let boundary = layer.parentheses;
            if layer.mode != TargetLayerMode::Remove
                && context.expression_start.needs_asi_guard()
                && self.options.semicolons == Semicolons::AsNeeded
            {
                writer.text(";");
                context.expression_start = ExpressionStart::None;
            }
            match layer.mode {
                TargetLayerMode::Verbatim => {
                    self.format_verbatim(boundary.outer, writer)?;
                    break (boundary.outer.end, None);
                }
                TargetLayerMode::Retain => {
                    let tail = self.format_retained_target_parentheses(
                        boundary,
                        context,
                        writer,
                        |formatter, context, writer| {
                            formatter.format_target_layers(
                                target,
                                remaining,
                                boundary.inner.end,
                                context,
                                writer,
                            )
                        },
                    )?;
                    break (boundary.outer.end, tail);
                }
                TargetLayerMode::Remove => {
                    let leading = self
                        .comments
                        .between(Span::new(boundary.outer.start, boundary.inner.start))?;
                    self.comments.claim(leading)?;
                    self.format_comment_run(
                        leading,
                        CommentRunLayout::FlatLeading(boundary.inner.start),
                        writer,
                    )?;
                    layers = remaining;
                }
            }
        };
        if content_end == tail_end {
            return Ok(tail);
        }
        // One deferred tail belongs to the whole run of removed wrappers.
        self.capture_expression_tail(Span::new(content_end, tail_end), tail)
    }
}
