//! Expression dispatch and immediate grammar and statement-start contexts.

pub(super) mod parentheses;

use swc_next_ecma_ast::{Expr, ExprData, GetSpan};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use super::literals::StringContext;
use super::suppression::ContentMode;
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum ExpressionPosition {
    Statement,
    Nested,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ExpressionStart {
    None,
    StatementPending,
    StatementGuarded,
}

impl ExpressionStart {
    fn needs_asi_guard(self) -> bool {
        matches!(self, Self::StatementPending)
    }
}

#[derive(Clone, Copy)]
pub(super) struct ExpressionContext {
    position: ExpressionPosition,
    expression_start: ExpressionStart,
    content_mode: ContentMode,
}

impl ExpressionContext {
    pub(super) fn at(position: ExpressionPosition) -> Self {
        Self {
            position,
            expression_start: match position {
                ExpressionPosition::Statement => ExpressionStart::StatementPending,
                ExpressionPosition::Nested => ExpressionStart::None,
            },
            content_mode: ContentMode::Formatted,
        }
    }

    pub(super) fn with_expression_start(mut self, start: ExpressionStart) -> Self {
        self.expression_start = start;
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

    pub(super) fn expression_needs_asi_guard(
        &self,
        expression: Expr,
        context: ExpressionContext,
    ) -> bool {
        if context.content_mode == ContentMode::Verbatim {
            let source = self.source(expression.span(self.ast));
            return self.expression_is_parenthesized(self.ast.expr_data(expression), context)
                || source.starts_with(['(', '[', '/', '`', '<'])
                || source.starts_with(['+', '-'])
                    && !(source.starts_with("++") || source.starts_with("--"));
        }
        let data = self.expression_content(expression).data;
        self.expression_is_parenthesized(data, context)
            || matches!(
                data,
                ExprData::ParenthesizedExpression(_) | ExprData::RegExpLiteral(_)
            )
    }

    fn expression_is_parenthesized(&self, data: ExprData, context: ExpressionContext) -> bool {
        self.expression_needs_parentheses(data, context.position) == Some(true)
    }

    fn expression_needs_parentheses(
        &self,
        data: ExprData,
        position: ExpressionPosition,
    ) -> Option<bool> {
        match data {
            ExprData::IdentifierReference(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::RegExpLiteral(_)
            | ExprData::ThisExpression(_)
            | ExprData::ParenthesizedExpression(_) => Some(false),
            ExprData::StringLiteral(_) => Some(matches!(position, ExpressionPosition::Statement)),
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
            && self.expression_needs_asi_guard(expression, context)
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
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
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
            _ => return Err(self.unsupported("expression", expression)),
        }
        Ok(())
    }
}
