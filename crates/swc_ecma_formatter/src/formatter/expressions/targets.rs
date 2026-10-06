//! Assignment and update owners recover target delimiters from their grammar gaps.

use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, AssignmentTargetData, Ast, BindingPattern, Comment,
    Expr, ExprData, GetSpan, SimpleAssignmentTarget, SimpleAssignmentTargetData, Span,
    UpdateExpression, UpdateOperator,
};

use super::parentheses::{ParenthesisInner, SourceParentheses};
use super::{
    ExpressionContext, ExpressionPosition, ExpressionStart, Formatter, MemberChainLayout,
    MemberLayout, TailOwner,
};
use crate::formatter::bindings::BindingGroup;
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
enum TargetOperator {
    Assignment(AssignmentOperator),
    Update(UpdateOperator),
    Iteration { of: bool },
}

impl TargetOperator {
    fn as_str(self) -> &'static str {
        match self {
            Self::Assignment(operator) => operator.as_str(),
            Self::Update(operator) => operator.as_str(),
            Self::Iteration { of } => {
                if of {
                    "of"
                } else {
                    "in"
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum TargetNode {
    Simple(SimpleTarget),
    Pattern {
        pattern: BindingPattern,
        group: Option<BindingGroup>,
    },
}

impl TargetNode {
    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Simple(node) => node.span(ast),
            Self::Pattern { pattern, .. } => pattern.span(ast),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SimpleTargetKind {
    Value,
    Cast,
    NonNull,
}

/// The original thin expression handle, converted once for all target consumers.
#[derive(Clone, Copy)]
pub(in crate::formatter) struct SimpleTarget {
    expression: Expr,
    kind: SimpleTargetKind,
}

impl SimpleTarget {
    pub(in crate::formatter) fn new(ast: &Ast<'_>, target: SimpleAssignmentTarget) -> Self {
        let (expression, kind) = match ast.simple_assignment_target_data(target) {
            SimpleAssignmentTargetData::IdentifierReference(node) => {
                (Expr::IdentifierReference(node), SimpleTargetKind::Value)
            }
            SimpleAssignmentTargetData::MemberExpression(node) => {
                (Expr::MemberExpression(node), SimpleTargetKind::Value)
            }
            SimpleAssignmentTargetData::TsAsExpression(node) => {
                (Expr::TsAsExpression(node), SimpleTargetKind::Cast)
            }
            SimpleAssignmentTargetData::TsSatisfiesExpression(node) => {
                (Expr::TsSatisfiesExpression(node), SimpleTargetKind::Cast)
            }
            SimpleAssignmentTargetData::TsTypeAssertion(node) => {
                (Expr::TsTypeAssertion(node), SimpleTargetKind::Cast)
            }
            SimpleAssignmentTargetData::TsNonNullExpression(node) => {
                (Expr::TsNonNullExpression(node), SimpleTargetKind::NonNull)
            }
        };
        Self { expression, kind }
    }

    pub(in crate::formatter) fn expression(self) -> Expr {
        self.expression
    }

    pub(in crate::formatter) fn is_typescript(self) -> bool {
        self.kind != SimpleTargetKind::Value
    }

    fn span(self, ast: &Ast<'_>) -> Span {
        self.expression.span(ast)
    }
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum TargetUse {
    AssignmentLeft,
    DefaultLeft,
    UpdateOperand,
    PatternElement,
    IterationLeft,
}

impl TargetUse {
    fn requires_cast_parentheses(self) -> bool {
        matches!(
            self,
            Self::AssignmentLeft | Self::DefaultLeft | Self::UpdateOperand
        )
    }

    fn context(self, mut context: ExpressionContext) -> ExpressionContext {
        context.position = match self {
            Self::AssignmentLeft | Self::DefaultLeft => {
                context.member_layout = MemberLayout::Tight;
                ExpressionPosition::AssignmentTarget
            }
            Self::UpdateOperand => ExpressionPosition::UpdateArgument,
            Self::PatternElement => ExpressionPosition::Nested,
            Self::IterationLeft => context.position,
        };
        context.with_tail_owner(TailOwner::ParentOperation)
    }
}

/// Real source delimiter ownership, independent of an operation's punctuation.
pub(in crate::formatter) struct PreparedTarget {
    target: TargetNode,
    use_context: TargetUse,
    pub(in crate::formatter) span: Span,
    tail_end: u32,
    layers: Vec<TargetLayer>,
}

pub(in crate::formatter) struct SourceTarget {
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

    pub(in crate::formatter) fn span(&self) -> Span {
        self.envelope.span
    }

    pub(super) fn left_end(&self) -> u32 {
        self.envelope.tail_end
    }

    pub(in crate::formatter) fn simple(&self) -> Option<SimpleTarget> {
        match self.envelope.target {
            TargetNode::Simple(target) => Some(target),
            TargetNode::Pattern { .. } => None,
        }
    }

    pub(super) fn binding(&self) -> Option<BindingPattern> {
        match self.envelope.target {
            TargetNode::Pattern { pattern, .. } => Some(pattern),
            TargetNode::Simple(_) => None,
        }
    }

    pub(super) fn set_binding_group(&mut self, prepared: BindingGroup) {
        if let TargetNode::Pattern { group, .. } = &mut self.envelope.target {
            *group = Some(prepared);
        }
    }

    pub(in crate::formatter) fn has_retained_parentheses(&self) -> bool {
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
    pub(super) fn assignment_source_target(
        &self,
        assignment: AssignmentExpression,
    ) -> Result<SourceTarget, FormatError> {
        let left = assignment.left(self.ast);
        let target = match self.ast.assignment_target_data(left) {
            AssignmentTargetData::SimpleAssignmentTarget(node) => {
                TargetNode::Simple(SimpleTarget::new(self.ast, node))
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
            && self.target_pair_is_semantic_boundary(assignment, target)
        {
            outer.mode = TargetLayerMode::Retain;
        }
        // The operator owner prepares released trailing trivia before either side prints.
        source.envelope.tail_end = source
            .envelope
            .layers
            .iter()
            .find(|layer| layer.mode != TargetLayerMode::Remove)
            .map_or(span.end, |layer| layer.parentheses.outer.end);
        Ok(source)
    }

    /// Source pairs suppress anonymous identifier naming and JS property initialization.
    fn target_pair_is_semantic_boundary(
        &self,
        assignment: AssignmentExpression,
        target: TargetNode,
    ) -> bool {
        if !matches!(
            assignment.operator(self.ast),
            AssignmentOperator::Assign
                | AssignmentOperator::LogicalAndAssign
                | AssignmentOperator::LogicalOrAssign
                | AssignmentOperator::NullishAssign
        ) {
            return false;
        }
        let TargetNode::Simple(target) = target else {
            return false;
        };
        match self.assignment_runtime_operand(target.expression) {
            ExprData::MemberExpression(_) => {
                return !self.ast.lang().is_typescript()
                    && assignment.operator(self.ast) == AssignmentOperator::Assign;
            }
            ExprData::IdentifierReference(_) => {}
            _ => return false,
        }
        match self.assignment_runtime_operand(assignment.right(self.ast)) {
            ExprData::Function(node) => node.id(self.ast).is_none(),
            ExprData::Class(node) => node.id(self.ast).is_none(),
            ExprData::ArrowFunctionExpression(_) => true,
            _ => false,
        }
    }

    /// Named evaluation observes the runtime operand through erased TS wrappers.
    fn assignment_runtime_operand(&self, mut expression: Expr) -> ExprData {
        loop {
            expression = match self.ast.expr_data(expression) {
                ExprData::ParenthesizedExpression(node) => node.expression(self.ast),
                ExprData::TsAsExpression(node) => node.expression(self.ast),
                ExprData::TsSatisfiesExpression(node) => node.expression(self.ast),
                ExprData::TsTypeAssertion(node) => node.expression(self.ast),
                ExprData::TsNonNullExpression(node) => node.expression(self.ast),
                data => return data,
            };
        }
    }

    pub(super) fn update_source_target(
        &self,
        update: UpdateExpression,
    ) -> Result<SourceTarget, FormatError> {
        let target = update.argument(self.ast);
        let span = update.span(self.ast);
        let target_span = target.span(self.ast);
        let operator = TargetOperator::Update(update.operator(self.ast));
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
            TargetNode::Simple(SimpleTarget::new(self.ast, target)),
            operator,
            prefix,
            suffix,
            Some(operator_span),
        )
    }

    pub(in crate::formatter) fn iteration_source_target(
        &self,
        target: SimpleAssignmentTarget,
        prefix: Span,
        operator: Span,
        of: bool,
    ) -> Result<SourceTarget, FormatError> {
        self.prepare_source_target(
            TargetNode::Simple(SimpleTarget::new(self.ast, target)),
            TargetOperator::Iteration { of },
            prefix,
            Span::new(target.span(self.ast).end, operator.start),
            Some(operator),
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
        let use_context = match operator {
            TargetOperator::Assignment(_) => TargetUse::AssignmentLeft,
            TargetOperator::Update(_) => TargetUse::UpdateOperand,
            TargetOperator::Iteration { .. } => TargetUse::IterationLeft,
        };
        let envelope =
            self.prepare_target_envelope(target, use_context, prefix, &mut trailing, None)?;
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

    pub(in crate::formatter) fn prepare_default_target(
        &self,
        target: SimpleTarget,
        prefix: Span,
        suffix: Span,
    ) -> Result<(PreparedTarget, bool), FormatError> {
        let mut trailing = self.target_gap(suffix);
        let mut whole = false;
        let envelope = self.prepare_target_envelope(
            TargetNode::Simple(target),
            TargetUse::DefaultLeft,
            prefix,
            &mut trailing,
            Some(&mut whole),
        )?;
        if !trailing.finished() {
            return Err(self.unsupported_target_boundary(Span::new(prefix.start, suffix.end)));
        }
        Ok((envelope, whole))
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
        use_context: TargetUse,
        prefix: Span,
        trailing: &mut TargetGap<'_>,
        mut whole_default: Option<&mut bool>,
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
            TargetNode::Pattern { .. } => ParenthesisInner::Unresolved,
            TargetNode::Simple(target) => ParenthesisInner::Expression(target.expression),
        };
        for layer in layers.iter_mut().rev() {
            let closing = trailing.consume(")").ok_or_else(invalid)?;
            layer.parentheses.outer.end = closing.end;
            layer.parentheses.inner = inner;
            layer.parentheses.inner_target = inner_target;
            layer.mode = self.target_layer_mode(layer.parentheses, whole_default.as_deref_mut());
            inner = layer.parentheses.outer;
            inner_target = ParenthesisInner::SourcePair;
        }
        if let Some(outer) = layers.first_mut()
            && outer.mode == TargetLayerMode::Remove
            && (matches!(target, TargetNode::Pattern { .. })
                || matches!(use_context, TargetUse::DefaultLeft)
                || matches!(target, TargetNode::Simple(simple) if simple.kind == SimpleTargetKind::Cast)
                    && use_context.requires_cast_parentheses())
        {
            outer.mode = TargetLayerMode::Retain;
        }
        Ok(PreparedTarget {
            target,
            use_context,
            span: inner,
            tail_end: inner.end,
            layers,
        })
    }

    fn target_layer_mode(
        &self,
        boundary: SourceParentheses,
        whole_default: Option<&mut bool>,
    ) -> TargetLayerMode {
        if self.has_suppression {
            let leading = self
                .comments
                .in_range(Span::new(boundary.outer.start, boundary.inner.start));
            let trailing = Span::new(boundary.inner.end, boundary.outer.end - 1);
            let verbatim = match whole_default {
                Some(whole) => {
                    let mut verbatim = false;
                    let positions =
                        self.suppression_positions(leading, boundary.inner.start)
                            .chain(self.suppression_positions(
                                self.comments.in_range(trailing),
                                trailing.end,
                            ));
                    for position in positions {
                        verbatim = true;
                        *whole |= matches!(position, SuppressionPosition::OwnLine);
                    }
                    verbatim
                }
                None => {
                    self.content_mode(leading) == ContentMode::Verbatim
                        || self.trailing_content_mode(trailing, true) == ContentMode::Verbatim
                }
            };
            if verbatim {
                return TargetLayerMode::Verbatim;
            }
        }
        if self.parentheses_are_removable(boundary) {
            TargetLayerMode::Remove
        } else {
            TargetLayerMode::Retain
        }
    }

    pub(in crate::formatter) fn format_source_target(
        &mut self,
        target: &SourceTarget,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_prepared_target(&target.envelope, context, writer)
    }

    pub(in crate::formatter) fn format_prepared_target(
        &mut self,
        target: &PreparedTarget,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let use_context = target.use_context;
        let mut context = use_context.context(context);
        let verbatim = context.content_mode == ContentMode::Verbatim && !target.layers.is_empty();
        if (matches!(target.target, TargetNode::Pattern { .. }) || verbatim)
            && context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
            context.expression_start = ExpressionStart::None;
        }
        if verbatim {
            self.format_verbatim(target.span, writer)?;
            return Ok(None);
        }
        let context = context.with_member_chain_layout(MemberChainLayout::Local);
        if target.layers.is_empty()
            && let TargetNode::Simple(simple) = target.target
            && simple.kind == SimpleTargetKind::Cast
            && use_context.requires_cast_parentheses()
        {
            return self.format_parenthesized_expression(
                simple.expression,
                None,
                NodeHead {
                    comments: &[],
                    content_start: target.span.start,
                    layout: CommentRunLayout::Source,
                    gap_end: None,
                },
                context,
                None,
                writer,
            );
        }
        self.format_target_layers(
            target.target,
            &target.layers,
            target.tail_end,
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
                        TargetNode::Simple(node) => {
                            self.format_simple_target(node, context, writer)?
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
                    context.expression_start = self.format_node_head(
                        NodeHead {
                            comments: leading,
                            content_start: boundary.inner.start,
                            layout: CommentRunLayout::FlatLeading(boundary.inner.start),
                            gap_end: None,
                        },
                        context.expression_start,
                        None,
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

    pub(super) fn source_target_needs_asi_guard(
        &self,
        target: &SourceTarget,
    ) -> Result<bool, FormatError> {
        if target.has_retained_parentheses() {
            return Ok(true);
        }
        let Some(node) = target.simple() else {
            return Ok(true);
        };
        self.expression_needs_asi_guard(
            node.expression,
            target
                .envelope
                .use_context
                .context(ExpressionContext::at(ExpressionPosition::Statement)),
        )
    }

    pub(in crate::formatter) fn format_simple_assignment_target(
        &mut self,
        target: SimpleAssignmentTarget,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_pattern_target(SimpleTarget::new(self.ast, target), context, writer)
    }

    pub(in crate::formatter) fn format_pattern_target(
        &mut self,
        target: SimpleTarget,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_simple_target(target, TargetUse::PatternElement.context(context), writer)
    }

    fn format_simple_target(
        &mut self,
        target: SimpleTarget,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression(target.expression, context, writer)
    }
}
