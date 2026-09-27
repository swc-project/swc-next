//! Expression dispatch and immediate grammar and statement-start contexts.

mod binary;
mod conditional;
pub(super) mod parentheses;
mod targets;
mod unary;

use swc_next_ecma_ast::{Expr, ExprData, GetSpan, UnaryOperator, UpdateOperator};

use self::binary::{BinaryLike, BinaryOperatorKind, OperandSide};
use self::parentheses::{ExpressionContent, ResolvedExpressionHead};
use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use super::literals::StringContext;
use super::suppression::ContentMode;
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum ExpressionPosition {
    Statement,
    Nested,
    UnaryArgument(UnaryOperator),
    ParenthesizedUnaryArgument(UnaryOperator),
    UpdateArgument,
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
}

impl ExpressionStart {
    fn needs_asi_guard(self) -> bool {
        matches!(self, Self::StatementPending | Self::StatementGuard)
    }
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
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TailOwner {
    Expression,
    ParentOperation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum OperandIndent {
    Expression,
    Parent,
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
}

impl ExpressionContext {
    pub(super) fn at(position: ExpressionPosition) -> Self {
        Self {
            position,
            expression_start: match position {
                ExpressionPosition::Statement => ExpressionStart::StatementPending,
                _ => ExpressionStart::None,
            },
            content_mode: ContentMode::Formatted,
            parentheses_owner: ParenthesesOwner::Expression,
            head_comments: HeadComments::Pending,
            tail_owner: TailOwner::Expression,
            operand_indent: OperandIndent::Expression,
            protected_head_start: None,
        }
    }

    pub(super) fn with_expression_start(mut self, start: ExpressionStart) -> Self {
        self.expression_start = start;
        self
    }

    pub(super) fn with_parent_head(mut self, parent: Self) -> Self {
        self.expression_start = parent.expression_start;
        self.protected_head_start = parent.protected_head_start;
        self
    }

    pub(super) fn inside_parentheses(mut self) -> Self {
        self.parentheses_owner = ParenthesesOwner::Parent;
        self.expression_start = ExpressionStart::None;
        self.protected_head_start = None;
        self
    }

    pub(super) fn with_tail_owner(mut self, owner: TailOwner) -> Self {
        self.tail_owner = owner;
        self
    }

    pub(super) fn with_operand_indent(mut self, owner: OperandIndent) -> Self {
        self.operand_indent = owner;
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
        self.format_expression_with_leading_comments(
            expression,
            NodeHead {
                comments: &[],
                content_start: expression.span(self.ast).start,
                layout: CommentRunLayout::Leading(expression.span(self.ast).start),
                gap_end: None,
            },
            context,
            writer,
        )
    }

    fn format_expression_prepared(
        &mut self,
        expression: Expr,
        prepared: Option<ExpressionContent>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let binding = prepared
            .as_ref()
            .and_then(|content| content.parentheses)
            .map_or(super::comments::CommentBinding::Unbound, |removed| {
                removed.head.binding
            });
        self.format_expression_leading_prepared(
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
            writer,
        )
    }

    pub(super) fn binary_grammar_pair_owns_indent(position: ExpressionPosition) -> bool {
        matches!(position, ExpressionPosition::UnaryArgument(_))
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
                    ExprData::ParenthesizedExpression(_) | ExprData::RegExpLiteral(_)
                )
                || matches!(data, ExprData::UnaryExpression(node) if matches!(node.operator(self.ast), UnaryOperator::Positive | UnaryOperator::Negate))
            {
                return Ok(true);
            }
            if let ExprData::UpdateExpression(update) = data
                && !update.prefix(self.ast)
            {
                return Ok(self
                    .update_source_target(update)?
                    .has_retained_parentheses());
            }
            if let Some(binary) = BinaryLike::from_data(data, self.ast) {
                expression = binary.left;
                context.position = ExpressionPosition::BinaryOperand {
                    operator: binary.operator,
                    side: OperandSide::Left,
                };
                continue;
            }
            if let ExprData::ConditionalExpression(conditional) = data {
                expression = conditional.test(self.ast);
                context.position = ExpressionPosition::ConditionalTest;
                continue;
            }
            return Ok(false);
        }
    }

    fn expression_is_parenthesized(&self, data: ExprData, context: ExpressionContext) -> bool {
        context.parentheses_owner == ParenthesesOwner::Expression
            && self.expression_needs_parentheses(data, context.position) == Some(true)
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
    ) -> Option<bool> {
        if self.is_nullish_conditional_operand(data, position) {
            return Some(true);
        }
        if let Some(child) = BinaryOperatorKind::from_data(data, self.ast) {
            return Some(match position {
                ExpressionPosition::UnaryArgument(_) => true,
                ExpressionPosition::BinaryOperand { operator, side } => {
                    operator.needs_parentheses(child, side)
                }
                _ => false,
            });
        }
        match data {
            ExprData::ConditionalExpression(_) => Some(matches!(
                position,
                ExpressionPosition::UnaryArgument(_)
                    | ExpressionPosition::ParenthesizedUnaryArgument(_)
                    | ExpressionPosition::BinaryOperand { .. }
                    | ExpressionPosition::ConditionalTest
            )),
            ExprData::StringLiteral(_) => Some(matches!(position, ExpressionPosition::Statement)),
            ExprData::UnaryExpression(unary) => Some(match position {
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
            ExprData::IdentifierReference(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::RegExpLiteral(_)
            | ExprData::ThisExpression(_)
            | ExprData::ParenthesizedExpression(_) => Some(false),
            _ => None,
        }
    }

    fn format_verbatim_expression(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let parenthesize =
            self.expression_is_parenthesized(self.ast.expr_data(expression), context);
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
            ExprData::ThisExpression(_) => writer.text("this"),
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
}
