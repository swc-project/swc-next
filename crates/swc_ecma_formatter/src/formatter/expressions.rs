//! Expression dispatch and immediate grammar and statement-start contexts.

pub(super) mod arguments;
mod arrays;
pub(super) mod arrows;
pub(super) mod assignment;
mod binary;
mod callbacks;
mod calls;
pub(super) mod chains;
mod conditional;
mod constructors;
mod for_initializer;
mod imports;
mod members;
pub(super) mod objects;
pub(super) mod parentheses;
mod sequence;
mod suspension;
pub(super) mod targets;
mod typescript;
mod unary;

use self::chains::FluentChainEligibility;
use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrayExpression, CallExpression, Expr, ExprData, GetSpan,
    IdentifierReference, ObjectExpression, SequenceExpression, Span, UnaryOperator, UpdateOperator,
};

pub(in crate::formatter) use self::for_initializer::ForInitializerGrammar;

use self::arrays::{ArrayArgumentArity, RecordedArray, RecordedArrayArgument};
use self::assignment::AssignmentParent;
use self::binary::{BinaryLike, BinaryOperatorKind, OperandSide};
use self::conditional::{ConditionalClosing, ConditionalEnvelope};
use self::objects::RecordedObject;
use self::parentheses::{ExpressionContent, ResolvedExpressionHead};
use super::Formatter;
use super::comments::{ArrayCommentOwner, CommentRunLayout, ExpressionTail, NodeHead};
use super::literals::StringContext;
use super::suppression::ContentMode;
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AccessKind {
    Ordinary,
    Optional,
}

#[derive(Clone, Copy)]
pub(super) enum ExpressionPosition {
    Statement,
    ExportDefault,
    Nested,
    ArrowBody,
    RestrictedArgument,
    YieldArgument,
    AwaitArgument,
    PropertyValue,
    TypePropertyKey,
    Superclass,
    SpreadArgument,
    MemberObject(AccessKind),
    CallCallee {
        access: AccessKind,
        curried_call: Option<CallExpression>,
    },
    NewCallee,
    MemberAssignmentObject,
    SequenceMember,
    ForSequenceMember,
    UnaryArgument(UnaryOperator),
    ParenthesizedUnaryArgument(UnaryOperator),
    UpdateArgument,
    AssignmentTarget,
    AssignmentRight(AssignmentParent),
    Initializer(AssignmentParent),
    ControlCondition,
    ForHead,
    ForTest,
    ForInitializer {
        grammar: ForInitializerGrammar,
        parentheses: bool,
    },
    ForInInitializer,
    ConditionalTest,
    ConditionalConsequent,
    ConditionalAlternate,
    BinaryOperand {
        operator: BinaryOperatorKind,
        side: OperandSide,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ExpressionStart {
    None,
    StatementPending,
    StatementGuard,
    StatementGuarded,
    /// A controlled body cannot insert a leading empty statement.
    StatementBody,
    ArrowBody,
    ForInitializer,
}

impl ExpressionStart {
    fn is_statement(self) -> bool {
        matches!(
            self,
            Self::StatementPending
                | Self::StatementGuard
                | Self::StatementGuarded
                | Self::StatementBody
        )
    }

    fn needs_asi_guard(self) -> bool {
        matches!(self, Self::StatementPending | Self::StatementGuard)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MemberChainLayout {
    Fluent,
    Local,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MemberLayout {
    Normal,
    Tight,
    Rhs,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExpressionHead {
    None,
    Statement,
    ForInitializer,
    Iteration(IdentifierReference),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParenthesesOwner {
    Expression,
    Parent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum HeadComments {
    Pending,
    Claimed,
    ClaimedSequence(SequenceExpression),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TailOwner {
    Expression,
    ParentOperation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SourceTailOwner {
    Consumer,
    Enclosure,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum OperandIndent {
    Expression,
    Parent,
}

/// Optional breaks for a complete expression belong to its actual source line.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ExpressionLayout {
    Normal,
    Flat,
}

impl ExpressionLayout {
    pub(super) fn write<'a, T>(
        self,
        writer: &mut Writer<'a>,
        write: impl FnOnce(&mut Writer<'a>) -> T,
    ) -> T {
        match self {
            Self::Normal => write(writer),
            Self::Flat => writer.flat(write),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ExpressionContext {
    position: ExpressionPosition,
    expression_start: ExpressionStart,
    content_mode: ContentMode,
    parentheses_owner: ParenthesesOwner,
    head_comments: HeadComments,
    tail_owner: TailOwner,
    operand_indent: OperandIndent,
    protected_head_start: Option<u32>,
    expression_head: ExpressionHead,
    computed_receiver: Option<Expr>,
    conditional_closing: ConditionalClosing,
    conditional_envelope: ConditionalEnvelope,
    member_layout: MemberLayout,
    member_chain_layout: MemberChainLayout,
    arrow_layout: arrows::ArrowLayout,
    source_parentheses_tail: Option<parentheses::SourceParenthesesTail>,
    source_tail_owner: SourceTailOwner,
}

impl ExpressionContext {
    fn is_call_or_member_receiver(self) -> bool {
        matches!(
            self.position,
            ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::CallCallee { .. }
        )
    }

    pub(super) fn is_verbatim(self) -> bool {
        self.content_mode == ContentMode::Verbatim
    }

    pub(super) fn variable_initializer() -> Self {
        Self::at(ExpressionPosition::Initializer(AssignmentParent::Statement))
            .with_source_tail_owner(SourceTailOwner::Enclosure)
            .with_arrow_layout(arrows::ArrowLayout::Assignment(
                arrows::ArrowHeadGap::Expression,
            ))
    }

    /// The real variable declarator owns its complete call/member RHS scope.
    pub(super) fn with_variable_initializer_owner(mut self) -> Self {
        self.member_layout = MemberLayout::Rhs;
        self
    }

    pub(super) fn binding_initializer() -> Self {
        let mut context = Self::at(ExpressionPosition::Initializer(AssignmentParent::Nested))
            .with_source_tail_owner(SourceTailOwner::Enclosure);
        context.conditional_envelope = ConditionalEnvelope::Local;
        context.arrow_layout = arrows::ArrowLayout::Ordinary;
        context
    }

    pub(super) fn at(position: ExpressionPosition) -> Self {
        Self {
            position,
            arrow_layout: match position {
                ExpressionPosition::CallCallee { .. } | ExpressionPosition::NewCallee => {
                    arrows::ArrowLayout::Callee
                }
                ExpressionPosition::AssignmentRight(_) | ExpressionPosition::Initializer(_) => {
                    arrows::ArrowLayout::Assignment(arrows::ArrowHeadGap::Expression)
                }
                ExpressionPosition::BinaryOperand { .. } => arrows::ArrowLayout::Argument,
                _ => arrows::ArrowLayout::Ordinary,
            },
            expression_start: match position {
                ExpressionPosition::Statement => ExpressionStart::StatementPending,
                ExpressionPosition::ArrowBody => ExpressionStart::ArrowBody,
                ExpressionPosition::ForInitializer { .. }
                | ExpressionPosition::ForInInitializer => ExpressionStart::ForInitializer,
                _ => ExpressionStart::None,
            },
            content_mode: ContentMode::Formatted,
            parentheses_owner: ParenthesesOwner::Expression,
            head_comments: HeadComments::Pending,
            tail_owner: TailOwner::Expression,
            operand_indent: OperandIndent::Expression,
            protected_head_start: None,
            expression_head: match position {
                ExpressionPosition::Statement => ExpressionHead::Statement,
                ExpressionPosition::ForInitializer {
                    grammar: ForInitializerGrammar::Expression,
                    ..
                } => ExpressionHead::ForInitializer,
                _ => ExpressionHead::None,
            },
            computed_receiver: None,
            member_layout: if matches!(position, ExpressionPosition::AssignmentRight(_)) {
                MemberLayout::Rhs
            } else {
                MemberLayout::Normal
            },
            member_chain_layout: MemberChainLayout::Fluent,
            conditional_closing: ConditionalClosing::Inline,
            conditional_envelope: match position {
                ExpressionPosition::AssignmentRight(_)
                | ExpressionPosition::Initializer(_)
                | ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::RestrictedArgument
                | ExpressionPosition::AwaitArgument
                | ExpressionPosition::YieldArgument => ConditionalEnvelope::ContinuationRoot,
                _ => ConditionalEnvelope::Local,
            },
            source_parentheses_tail: None,
            source_tail_owner: SourceTailOwner::Consumer,
        }
    }

    pub(super) fn with_expression_start(mut self, start: ExpressionStart) -> Self {
        self.expression_start = start;
        self
    }

    pub(in crate::formatter) fn with_lexical_head(
        mut self,
        identifier: Option<IdentifierReference>,
    ) -> Self {
        self.expression_head = identifier.map_or(ExpressionHead::None, ExpressionHead::Iteration);
        self
    }

    pub(super) fn with_parent_head(mut self, parent: Self) -> Self {
        self.expression_start = parent.expression_start;
        self.expression_head = parent.expression_head;
        self.protected_head_start = parent.protected_head_start;
        self
    }

    pub(super) fn inside_parentheses(mut self) -> Self {
        self.parentheses_owner = ParenthesesOwner::Parent;
        self.source_parentheses_tail = None;
        self.expression_start = ExpressionStart::None;
        self.protected_head_start = None;
        self
    }

    pub(super) fn with_source_tail_owner(mut self, owner: SourceTailOwner) -> Self {
        self.source_tail_owner = owner;
        self
    }

    pub(super) fn with_tail_owner(mut self, owner: TailOwner) -> Self {
        self.tail_owner = owner;
        self
    }

    pub(super) fn is_formatted_conditional_rhs(&self, data: ExprData) -> bool {
        self.content_mode == ContentMode::Formatted
            && matches!(data, ExprData::ConditionalExpression(_))
    }

    pub(super) fn with_operand_indent(mut self, owner: OperandIndent) -> Self {
        self.operand_indent = owner;
        self
    }

    fn with_arrow_layout(mut self, layout: arrows::ArrowLayout) -> Self {
        self.arrow_layout = layout;
        self
    }

    pub(super) fn with_content_mode(mut self, mode: ContentMode) -> Self {
        self.content_mode = mode;
        self
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_expression(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression_with_array_comments(expression, context, None, writer)
    }

    pub(super) fn format_expression_with_array_comments(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let tail = self.format_expression_with_leading_comments(
            expression,
            NodeHead {
                comments: &[],
                content_start: expression.span(self.ast).start,
                layout: CommentRunLayout::Leading(expression.span(self.ast).start),
                gap_end: None,
            },
            context,
            array_comments.as_deref_mut(),
            writer,
        )?;
        if let Some(owner) = array_comments {
            let end = tail
                .as_ref()
                .map_or(expression.span(self.ast).end, |tail| tail.source_end);
            owner.finish_source(self, end);
        }
        Ok(tail)
    }

    fn format_expression_prepared(
        &mut self,
        expression: Expr,
        prepared: Option<ExpressionContent>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression_prepared_with_array_comments(
            expression, prepared, context, None, writer,
        )
    }

    fn format_expression_prepared_with_array_comments(
        &mut self,
        expression: Expr,
        prepared: Option<ExpressionContent>,
        context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let binding = prepared
            .as_ref()
            .and_then(|content| content.parentheses)
            .map_or(super::comments::CommentBinding::Unbound, |removed| {
                removed.head.binding
            });
        let tail = self.format_expression_leading_prepared(
            expression,
            prepared,
            ResolvedExpressionHead {
                head: NodeHead {
                    comments: &[],
                    content_start: expression.span(self.ast).start,
                    layout: CommentRunLayout::Source,
                    gap_end: None,
                },
                binding,
            },
            context,
            array_comments.as_deref_mut(),
            writer,
        )?;
        if let Some(owner) = array_comments {
            let end = tail
                .as_ref()
                .map_or(expression.span(self.ast).end, |tail| tail.source_end);
            owner.finish_source(self, end);
        }
        Ok(tail)
    }

    pub(super) fn for_head_grammar_pair_is_enclosed(
        data: ExprData,
        position: ExpressionPosition,
    ) -> bool {
        matches!(
            position,
            ExpressionPosition::ForInitializer {
                parentheses: true,
                ..
            }
        ) && !matches!(
            data,
            ExprData::ParenthesizedExpression(_) | ExprData::SequenceExpression(_)
        )
    }

    pub(super) fn binary_grammar_pair_owns_indent(position: ExpressionPosition) -> bool {
        matches!(
            position,
            ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::NewCallee
                | ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
        )
    }

    pub(super) fn expression_needs_asi_guard(
        &self,
        mut expression: Expr,
        mut context: ExpressionContext,
    ) -> Result<bool, FormatError> {
        loop {
            if context.content_mode == ContentMode::Verbatim {
                let source = self.source(expression.span(self.ast));
                return Ok(self
                    .expression_is_parenthesized(self.ast.expr_data(expression), context)
                    || source.starts_with(['(', '[', '/', '`', '<'])
                    || source.starts_with(['+', '-'])
                        && !(source.starts_with("++") || source.starts_with("--")));
            }
            let data = self.expression_content(expression).data;
            if self.expression_is_parenthesized(data, context)
                || matches!(
                    data,
                    ExprData::ParenthesizedExpression(_)
                        | ExprData::RegExpLiteral(_)
                        | ExprData::SequenceExpression(_)
                        | ExprData::ArrayExpression(_)
                )
                || matches!(data, ExprData::UnaryExpression(node) if matches!(node.operator(self.ast), UnaryOperator::Positive | UnaryOperator::Negate))
            {
                return Ok(true);
            }
            if let ExprData::ArrowFunctionExpression(arrow) = data {
                return self.arrow_needs_asi_guard(arrow);
            }
            if let ExprData::AssignmentExpression(assignment) = data {
                return self
                    .source_target_needs_asi_guard(&self.assignment_source_target(assignment)?);
            }
            if let ExprData::UpdateExpression(update) = data
                && !update.prefix(self.ast)
            {
                return self.source_target_needs_asi_guard(&self.update_source_target(update)?);
            }
            if let Some((child, child_context)) = self.expression_head_operand(data, context)? {
                expression = child;
                context = child_context;
                continue;
            }
            return Ok(false);
        }
    }

    /// One printed-head edge shared by ASI and restricted lexical envelopes.
    pub(in crate::formatter) fn expression_head_operand(
        &self,
        data: ExprData,
        context: ExpressionContext,
    ) -> Result<Option<(Expr, ExpressionContext)>, FormatError> {
        let operand = if let Some(binary) = BinaryLike::from_data(data, self.ast) {
            let mut child_context = context;
            child_context.position = ExpressionPosition::BinaryOperand {
                operator: binary.operator,
                side: OperandSide::Left,
            };
            (binary.left, child_context)
        } else {
            match data {
                ExprData::CallExpression(call) => {
                    let callee = self.source_call_callee(call, context)?;
                    (callee.expression, callee.context(context))
                }
                ExprData::MemberExpression(member) => {
                    self.source_member_receiver(member, context)?
                }
                ExprData::ChainExpression(chain) => (chain.expression(self.ast), context),
                ExprData::ConditionalExpression(conditional) => {
                    let mut child_context = context;
                    child_context.position = ExpressionPosition::ConditionalTest;
                    (conditional.test(self.ast), child_context)
                }
                _ => return Ok(None),
            }
        };
        Ok(Some(operand))
    }

    pub(in crate::formatter) fn expression_is_parenthesized(
        &self,
        data: ExprData,
        context: ExpressionContext,
    ) -> bool {
        context.parentheses_owner == ParenthesesOwner::Expression
            && (self.expression_needs_parentheses(data, context.position, context.content_mode)
                == Some(true)
                || self.lexical_identifier_head_needs_parentheses(data, context)
                || matches!(data, ExprData::ObjectExpression(_))
                    && (context.expression_start.is_statement()
                        || matches!(context.expression_start, ExpressionStart::ArrowBody)
                        || matches!(context.expression_head, ExpressionHead::Iteration(_)))
                || matches!(data, ExprData::Function(_) | ExprData::Class(_))
                    && context.expression_start.is_statement()
                || matches!((data, context.expression_head), (ExprData::IdentifierReference(node), ExpressionHead::Iteration(head)) if node == head))
    }

    /// The actual computed receiver owns the lexical pair, not its whole member.
    fn lexical_identifier_head_needs_parentheses(
        &self,
        data: ExprData,
        context: ExpressionContext,
    ) -> bool {
        let Some(receiver) = context.computed_receiver else {
            return false;
        };
        let ExprData::IdentifierReference(identifier) = data else {
            return false;
        };
        self.ast.get_utf8(identifier.name(self.ast)) == "let"
            && matches!(self.expression_content(receiver).data,
                ExprData::IdentifierReference(node) if node == identifier)
    }

    fn default_export_head_needs_parentheses(&self, mut data: ExprData, mode: ContentMode) -> bool {
        loop {
            let (expression, position) = if let Some(binary) = BinaryLike::from_data(data, self.ast)
            {
                (
                    binary.left,
                    ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Left,
                    },
                )
            } else {
                match data {
                    ExprData::Function(_) | ExprData::Class(_) => return true,
                    ExprData::CallExpression(call) => (
                        call.callee(self.ast),
                        ExpressionPosition::CallCallee {
                            access: if call.optional(self.ast) {
                                AccessKind::Optional
                            } else {
                                AccessKind::Ordinary
                            },
                            curried_call: None,
                        },
                    ),
                    ExprData::MemberExpression(member) => (
                        member.object(self.ast),
                        ExpressionPosition::MemberObject(if member.optional(self.ast) {
                            AccessKind::Optional
                        } else {
                            AccessKind::Ordinary
                        }),
                    ),
                    ExprData::ConditionalExpression(conditional) => (
                        conditional.test(self.ast),
                        ExpressionPosition::ConditionalTest,
                    ),
                    ExprData::ChainExpression(chain) => {
                        (chain.expression(self.ast), ExpressionPosition::Nested)
                    }
                    ExprData::AssignmentExpression(assignment) => {
                        let swc_next_ecma_ast::AssignmentTargetData::SimpleAssignmentTarget(target) =
                            self.ast.assignment_target_data(assignment.left(self.ast))
                        else {
                            return false;
                        };
                        let Ok(target) = self.simple_target_expression(target) else {
                            return true;
                        };
                        (target, ExpressionPosition::AssignmentTarget)
                    }
                    ExprData::UpdateExpression(update) if !update.prefix(self.ast) => {
                        let Ok(target) = self.simple_target_expression(update.argument(self.ast))
                        else {
                            return true;
                        };
                        (target, ExpressionPosition::UpdateArgument)
                    }
                    _ => return false,
                }
            };
            data = self.expression_content(expression).data;
            if self.expression_needs_parentheses(data, position, mode) == Some(true) {
                return false;
            }
        }
    }

    fn is_nullish_conditional_operand(&self, data: ExprData, position: ExpressionPosition) -> bool {
        matches!(
            position,
            ExpressionPosition::ConditionalTest
                | ExpressionPosition::ConditionalConsequent
                | ExpressionPosition::ConditionalAlternate
        ) && matches!(data, ExprData::LogicalExpression(node)
            if node.operator(self.ast) == swc_next_ecma_ast::LogicalOperator::NullishCoalescing)
    }

    fn expression_needs_parentheses(
        &self,
        data: ExprData,
        position: ExpressionPosition,
        mode: ContentMode,
    ) -> Option<bool> {
        if let Some(scope) = self.for_initializer_span {
            if mode == ContentMode::Verbatim && self.opaque_initializer_needs_enclosure(data, scope)
            {
                return Some(true);
            }
            if let ExprData::BinaryExpression(node) = data
                && node.operator(self.ast) == swc_next_ecma_ast::BinaryOperator::In
            {
                let span = node.span(self.ast);
                if scope.start <= span.start && span.end <= scope.end {
                    return Some(true);
                }
            }
        }
        if mode == ContentMode::Formatted
            && matches!(
                position,
                ExpressionPosition::ForInitializer {
                    parentheses: true,
                    ..
                } | ExpressionPosition::ForInInitializer
            )
        {
            // An opaque source pair already supplies this header's grammar boundary.
            return Some(match data {
                ExprData::ParenthesizedExpression(_) => false,
                ExprData::IdentifierReference(_) => {
                    !matches!(position, ExpressionPosition::ForInInitializer)
                }
                _ => true,
            });
        }
        if matches!(position, ExpressionPosition::Superclass) {
            let mut operand = data;
            loop {
                let expression = match operand {
                    ExprData::ChainExpression(node) => node.expression(self.ast),
                    ExprData::ParenthesizedExpression(node) if mode == ContentMode::Formatted => {
                        node.expression(self.ast)
                    }
                    _ => break,
                };
                operand = self.ast.expr_data(expression);
            }
            self.expression_needs_parentheses(operand, ExpressionPosition::Nested, mode)?;
            return Some(matches!(
                operand,
                ExprData::NewExpression(_)
                    | ExprData::ArrowFunctionExpression(_)
                    | ExprData::ObjectExpression(_)
                    | ExprData::SequenceExpression(_)
                    | ExprData::AssignmentExpression(_)
                    | ExprData::ConditionalExpression(_)
                    | ExprData::UnaryExpression(_)
                    | ExprData::UpdateExpression(_)
                    | ExprData::BinaryExpression(_)
                    | ExprData::LogicalExpression(_)
                    | ExprData::YieldExpression(_)
                    | ExprData::AwaitExpression(_)
            ));
        }
        let position = if matches!(position, ExpressionPosition::ForSequenceMember)
            && !matches!(data, ExprData::AssignmentExpression(_))
        {
            ExpressionPosition::SequenceMember
        } else {
            position
        };
        if matches!(position, ExpressionPosition::ExportDefault)
            && self.default_export_head_needs_parentheses(data, mode)
        {
            return Some(true);
        }
        if matches!(position, ExpressionPosition::NewCallee)
            && self.new_callee_contains_call(data, mode)
        {
            return Some(true);
        }
        if self.is_nullish_conditional_operand(data, position) {
            return Some(true);
        }
        if let Some(child) = BinaryOperatorKind::from_data(data, self.ast) {
            return Some(match position {
                ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::AwaitArgument
                | ExpressionPosition::SpreadArgument
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::NewCallee
                | ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject => true,
                ExpressionPosition::BinaryOperand { operator, side } => {
                    operator.needs_parentheses(child, side)
                }
                _ => false,
            });
        }
        match data {
            ExprData::NewExpression(constructor) => {
                if mode == ContentMode::Formatted
                    || !matches!(
                        position,
                        ExpressionPosition::CallCallee { .. }
                            | ExpressionPosition::MemberObject(_)
                            | ExpressionPosition::MemberAssignmentObject
                            | ExpressionPosition::NewCallee
                    )
                {
                    return Some(false);
                }
                let suffix_end = constructor.type_arguments(self.ast).map_or_else(
                    || constructor.callee(self.ast).span(self.ast).end,
                    |arguments| arguments.span(self.ast).end,
                );
                Some(constructor.span(self.ast).end == suffix_end)
            }
            ExprData::ArrowFunctionExpression(_) => Some(matches!(
                position,
                ExpressionPosition::UnaryArgument(_)
                    | ExpressionPosition::ParenthesizedUnaryArgument(_)
                    | ExpressionPosition::AwaitArgument
                    | ExpressionPosition::BinaryOperand { .. }
                    | ExpressionPosition::MemberObject(_)
                    | ExpressionPosition::MemberAssignmentObject
                    | ExpressionPosition::CallCallee { .. }
                    | ExpressionPosition::NewCallee
                    | ExpressionPosition::ConditionalTest
            )),
            ExprData::AwaitExpression(_) | ExprData::YieldExpression(_) => Some(
                matches!(
                    position,
                    ExpressionPosition::UnaryArgument(_)
                        | ExpressionPosition::ParenthesizedUnaryArgument(_)
                        | ExpressionPosition::BinaryOperand { .. }
                        | ExpressionPosition::SpreadArgument
                        | ExpressionPosition::MemberObject(_)
                        | ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::CallCallee { .. }
                        | ExpressionPosition::NewCallee
                        | ExpressionPosition::ConditionalTest
                ) || matches!(data, ExprData::YieldExpression(_))
                    && matches!(position, ExpressionPosition::AwaitArgument),
            ),
            ExprData::ChainExpression(_) => Some(matches!(
                position,
                ExpressionPosition::MemberObject(AccessKind::Ordinary)
                    | ExpressionPosition::CallCallee {
                        access: AccessKind::Ordinary,
                        ..
                    }
                    | ExpressionPosition::MemberAssignmentObject
                    | ExpressionPosition::NewCallee
            )),
            ExprData::MetaProperty(_) | ExprData::ImportExpression(_) => Some(false),
            ExprData::Class(_) => Some(matches!(
                position,
                ExpressionPosition::Statement | ExpressionPosition::NewCallee
            )),
            ExprData::Function(_) => Some(matches!(
                position,
                ExpressionPosition::CallCallee { .. } | ExpressionPosition::NewCallee
            )),
            ExprData::ArrayExpression(_) => Some(false),
            ExprData::ObjectExpression(_) => {
                Some(matches!(position, ExpressionPosition::Statement))
            }
            ExprData::SequenceExpression(_) => Some(!matches!(
                position,
                ExpressionPosition::ForHead
                    | ExpressionPosition::ForInitializer {
                        grammar: ForInitializerGrammar::Expression,
                        parentheses: false
                    }
            )),
            ExprData::ConditionalExpression(_) => Some(matches!(
                position,
                ExpressionPosition::UnaryArgument(_)
                    | ExpressionPosition::ParenthesizedUnaryArgument(_)
                    | ExpressionPosition::AwaitArgument
                    | ExpressionPosition::SpreadArgument
                    | ExpressionPosition::BinaryOperand { .. }
                    | ExpressionPosition::ConditionalTest
                    | ExpressionPosition::CallCallee { .. }
                    | ExpressionPosition::NewCallee
                    | ExpressionPosition::MemberObject(_)
                    | ExpressionPosition::MemberAssignmentObject
            )),
            ExprData::AssignmentExpression(assignment) => {
                Some(if matches!(position, ExpressionPosition::Statement) {
                    matches!(
                        self.ast.assignment_target_data(assignment.left(self.ast)),
                        swc_next_ecma_ast::AssignmentTargetData::ObjectAssignmentTarget(_)
                    )
                } else {
                    !matches!(
                        position,
                        ExpressionPosition::AssignmentRight(_)
                            | ExpressionPosition::ExportDefault
                            | ExpressionPosition::TypePropertyKey
                            | ExpressionPosition::ForHead
                            | ExpressionPosition::ForSequenceMember
                            | ExpressionPosition::ForInitializer { .. }
                    )
                })
            }
            ExprData::StringLiteral(_) => Some(matches!(position, ExpressionPosition::Statement)),
            ExprData::UnaryExpression(unary) => Some(match position {
                ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::NewCallee => true,
                ExpressionPosition::UnaryArgument(parent)
                | ExpressionPosition::ParenthesizedUnaryArgument(parent) => {
                    matches!(parent, UnaryOperator::Positive | UnaryOperator::Negate)
                        && unary.operator(self.ast) == parent
                }
                ExpressionPosition::BinaryOperand { operator, side } => matches!(
                    (side, operator),
                    (
                        OperandSide::Left,
                        BinaryOperatorKind::Binary(
                            swc_next_ecma_ast::BinaryOperator::Exponent
                                | swc_next_ecma_ast::BinaryOperator::In
                                | swc_next_ecma_ast::BinaryOperator::Instanceof
                        )
                    )
                ),
                _ => false,
            }),
            ExprData::UpdateExpression(update) => Some(match position {
                ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::NewCallee => true,
                ExpressionPosition::UnaryArgument(parent)
                | ExpressionPosition::ParenthesizedUnaryArgument(parent) => {
                    update.prefix(self.ast)
                        && matches!(
                            (parent, update.operator(self.ast)),
                            (UnaryOperator::Positive, UpdateOperator::Increment)
                                | (UnaryOperator::Negate, UpdateOperator::Decrement)
                        )
                }
                ExpressionPosition::BinaryOperand { operator, side } => matches!(
                    (side, operator),
                    (
                        OperandSide::Left,
                        BinaryOperatorKind::Binary(swc_next_ecma_ast::BinaryOperator::Exponent)
                    )
                ),
                _ => false,
            }),
            ExprData::NumericLiteral(_) => Some(matches!(
                position,
                ExpressionPosition::MemberObject(_) | ExpressionPosition::MemberAssignmentObject
            )),
            ExprData::Super(_)
            | ExprData::PrivateIdentifier(_)
            | ExprData::IdentifierReference(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::RegExpLiteral(_)
            | ExprData::ThisExpression(_)
            | ExprData::ParenthesizedExpression(_)
            | ExprData::MemberExpression(_)
            | ExprData::CallExpression(_) => Some(false),
            _ => None,
        }
    }

    pub(super) fn chain_element_data(&self, mut data: ExprData) -> ExprData {
        loop {
            match data {
                ExprData::ChainExpression(chain) => {
                    data = self.expression_content(chain.expression(self.ast)).data
                }
                _ => return data,
            }
        }
    }
    fn expression_layout(&self, expression: Expr) -> ExpressionLayout {
        if self.diagnostic_lines.is_empty() {
            return ExpressionLayout::Normal;
        }
        let span = expression.span(self.ast);
        if self.diagnostic_line_is_sensitive(span.start)
            && self.line_breaks(span.start, span.end) == 0
        {
            ExpressionLayout::Flat
        } else {
            ExpressionLayout::Normal
        }
    }

    fn format_verbatim_expression(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let parenthesize = self
            .expression_is_parenthesized(self.ast.expr_data(expression), context)
            || matches!(context.expression_head, ExpressionHead::Iteration(_));
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && self.expression_needs_asi_guard(expression, context)?
        {
            writer.text(";");
        }
        if parenthesize {
            writer.text("(");
        }
        self.format_verbatim(expression.span(self.ast), writer)?;
        if parenthesize {
            writer.text(")");
        }
        Ok(())
    }

    fn format_expression_data(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(node) => {
                writer.text(self.ast.get_utf8(node.name(self.ast)))
            }
            ExprData::BooleanLiteral(node) => writer.text(if node.value(self.ast) {
                "true"
            } else {
                "false"
            }),
            ExprData::NullLiteral(_) => writer.text("null"),
            ExprData::StringLiteral(node) => {
                self.format_string_literal(node, StringContext::Expression, writer)
            }
            ExprData::NumericLiteral(node) => self.format_numeric_literal(node, writer),
            ExprData::BigIntLiteral(node) => self.format_bigint_literal(node, writer),
            ExprData::RegExpLiteral(node) => self.format_regexp_literal(node, writer),
            ExprData::Super(_) => writer.text("super"),
            ExprData::PrivateIdentifier(identifier) => {
                writer.text("#");
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
            }
            ExprData::Class(class) => self.format_class(class, context.position, writer)?,
            ExprData::ThisExpression(_) => writer.text("this"),
            ExprData::ChainExpression(chain) => {
                let child_context = if self
                    .expression_is_parenthesized(ExprData::ChainExpression(chain), context)
                {
                    context.inside_parentheses()
                } else {
                    context
                };
                return self.format_expression(chain.expression(self.ast), child_context, writer);
            }
            ExprData::AwaitExpression(node) => return self.format_await(node, context, writer),
            ExprData::YieldExpression(node) => return self.format_yield(node, writer),
            ExprData::ImportExpression(import) => {
                return self.format_import_expression(import, writer);
            }
            ExprData::MetaProperty(node) => {
                self.format_meta_property(node, writer)?;
            }
            ExprData::Function(function) => {
                self.format_function(function, writer)?;
            }
            ExprData::ArrowFunctionExpression(arrow) => {
                return self.format_arrow(arrow, context, writer);
            }
            ExprData::ArrayExpression(array) => self.format_array(array, writer)?,
            ExprData::ObjectExpression(object) => self.format_object(object, writer)?,
            ExprData::NewExpression(constructor) => {
                return self.format_new(constructor, context, writer);
            }
            ExprData::MemberExpression(member) => {
                let eligibility = if context.member_chain_layout == MemberChainLayout::Local {
                    FluentChainEligibility::Unqualified
                } else {
                    self.fluent_chain_eligibility(
                        self.ast.expr_data(expression),
                        context,
                        self.expression_layout(expression),
                    )
                };
                match eligibility {
                    FluentChainEligibility::Fluent => {
                        return self.format_fluent_chain(expression, context, writer);
                    }
                    FluentChainEligibility::MemberOnly => {
                        return self.format_member(
                            member,
                            {
                                let mut local = context;
                                local.member_chain_layout = MemberChainLayout::Local;
                                local
                            },
                            writer,
                        );
                    }
                    FluentChainEligibility::Unqualified => {
                        return self.format_member(member, context, writer);
                    }
                }
            }
            ExprData::CallExpression(call) => {
                if !chains::atomic_chain_head(self.ast.expr_data(call.callee(self.ast)))
                    && matches!(
                        self.fluent_chain_eligibility(
                            self.ast.expr_data(expression),
                            context,
                            self.expression_layout(expression)
                        ),
                        FluentChainEligibility::Fluent
                    )
                {
                    return self.format_fluent_chain(expression, context, writer);
                } else {
                    return self.format_call(call, context, writer);
                }
            }
            ExprData::BinaryExpression(node) => {
                return self.format_binary(
                    BinaryLike {
                        left: node.left(self.ast),
                        right: node.right(self.ast),
                        operator: BinaryOperatorKind::Binary(node.operator(self.ast)),
                    },
                    context,
                    writer,
                );
            }
            ExprData::LogicalExpression(node) => {
                return self.format_binary(
                    BinaryLike {
                        left: node.left(self.ast),
                        right: node.right(self.ast),
                        operator: BinaryOperatorKind::Logical(node.operator(self.ast)),
                    },
                    context,
                    writer,
                );
            }
            ExprData::AssignmentExpression(assignment) => {
                return self.format_assignment(assignment, context, writer);
            }
            ExprData::ConditionalExpression(conditional) => {
                return self.format_conditional(conditional, context, writer);
            }
            ExprData::UnaryExpression(unary) => return self.format_unary(unary, writer),
            ExprData::UpdateExpression(update) => {
                return self.format_update(update, context, writer);
            }
            _ => return Err(self.unsupported("expression", expression)),
        }
        Ok(None)
    }
    fn record_object_argument(
        &mut self,
        expression: Expr,
        object: ObjectExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedObject<'ast>, FormatError> {
        let layout = self.expression_layout(expression);
        match layout {
            ExpressionLayout::Flat => {
                let (normal, ()) = writer.record_with_value(|writer| {
                    layout.write(writer, |writer| self.format_object(object, writer))
                })?;
                Ok(RecordedObject {
                    normal,
                    forced: None,
                })
            }
            ExpressionLayout::Normal => self.record_object(object, writer),
        }
    }

    fn record_array_argument(
        &mut self,
        expression: Expr,
        array: ArrayExpression,
        arity: ArrayArgumentArity,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedArrayArgument<'ast>, FormatError> {
        let layout = self.expression_layout(expression);
        match layout {
            ExpressionLayout::Flat => {
                let (normal, ()) = writer.record_with_value(|writer| {
                    layout.write(writer, |writer| self.format_array(array, writer))
                })?;
                Ok(RecordedArrayArgument::Selected(RecordedArray {
                    normal,
                    forced: None,
                }))
            }
            ExpressionLayout::Normal => self.record_array(array, arity, writer),
        }
    }

    fn format_argument(
        &mut self,
        argument: Argument,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_argument_with_array_comments(argument, mode, None, writer)
    }

    fn format_argument_with_array_comments(
        &mut self,
        argument: Argument,
        mode: ContentMode,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_argument_data_with_array_comments(
            self.ast.argument_data(argument),
            mode,
            array_comments,
            writer,
        )
    }

    fn format_argument_data_with_array_comments(
        &mut self,
        argument: ArgumentData,
        mode: ContentMode,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match argument {
            ArgumentData::Expr(expression) => self.format_expression_with_array_comments(
                expression,
                ExpressionContext::at(ExpressionPosition::Nested)
                    .with_content_mode(mode)
                    .with_arrow_layout(arrows::ArrowLayout::Argument),
                array_comments,
                writer,
            ),
            ArgumentData::SpreadElement(spread) => {
                if mode == ContentMode::Verbatim {
                    self.format_verbatim(spread.span(self.ast), writer)?;
                    return Ok(None);
                }
                let argument = spread.argument(self.ast);
                writer.text("...");
                let comments = self.comments.between(Span::new(
                    spread.span(self.ast).start + 3,
                    argument.span(self.ast).start,
                ))?;
                let comments = self.comments.take(comments.len());
                let mode = self.content_mode(comments);
                self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
                if let Some(last) = comments.last() {
                    self.comment_gap(last.span.end, argument.span(self.ast).start, writer);
                }
                self.format_expression(
                    argument,
                    ExpressionContext::at(ExpressionPosition::SpreadArgument)
                        .with_content_mode(mode),
                    writer,
                )
            }
        }
    }
}
