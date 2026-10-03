//! The for initializer carries its NoIn grammar decision into expression layout.

use swc_next_ecma_ast::{ArrowFunctionBodyData, BinaryOperator, Expr, ExprData, GetSpan, Span};

use super::parentheses::IgnoredSourceParentheses;
use super::{ExpressionContext, ExpressionPosition, Formatter, SourceTailOwner, TailOwner};
use crate::FormatError;

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ForInitializerGrammar {
    Expression,
    Assignment,
}

impl ExpressionContext {
    pub(in crate::formatter) fn is_for_initializer(&self) -> bool {
        matches!(self.position, ExpressionPosition::ForInitializer { .. })
    }

    pub(in crate::formatter) fn for_initializer_requires_enclosure(&self) -> bool {
        matches!(
            self.position,
            ExpressionPosition::ForInitializer {
                parentheses: true,
                ..
            }
        )
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn for_initializer_context(
        &self,
        expression: Expr,
        grammar: ForInitializerGrammar,
    ) -> ExpressionContext {
        self.prepare_for_initializer_context(expression, grammar, TailOwner::Expression)
            .0
    }

    /// The header consumes this actual opaque owner without repeating its head query.
    pub(in crate::formatter) fn prepare_for_initializer_context(
        &self,
        expression: Expr,
        grammar: ForInitializerGrammar,
        tail_owner: TailOwner,
    ) -> (ExpressionContext, Option<IgnoredSourceParentheses>) {
        let mut context = ExpressionContext::at(ExpressionPosition::ForInitializer {
            grammar,
            parentheses: false,
        })
        .with_tail_owner(tail_owner);
        let mut ignored = None;
        if self.has_suppression && expression.as_parenthesized_expression(self.ast).is_some() {
            // Ordinary outer pairs cannot hide the actual opaque source owner.
            let content = self.expression_content(expression);
            let head = self.source_parentheses_head_content(content.data, context);
            if let Some(mut child) = head.ignored {
                context.position = ExpressionPosition::ForInitializer {
                    grammar,
                    parentheses: self.for_initializer_contains_in(child.expression),
                };
                child.leading.start = expression.span(self.ast).start;
                ignored = Some(child);
            }
        }
        if matches!(grammar, ForInitializerGrammar::Assignment) {
            context = context.with_source_tail_owner(SourceTailOwner::Enclosure);
        }
        (context, ignored)
    }

    /// Install the real initializer owner only while that header item prints.
    /// Restoring before returning also covers errors and nested for statements.
    pub(in crate::formatter) fn with_for_initializer<T>(
        &mut self,
        scope: Option<Span>,
        format: impl FnOnce(&mut Self) -> Result<T, FormatError>,
    ) -> Result<T, FormatError> {
        let previous = std::mem::replace(&mut self.for_initializer_span, scope);
        let result = format(self);
        self.for_initializer_span = previous;
        result
    }

    /// Only an actual ignored opaque child needs a complete NoIn source enclosure.
    fn for_initializer_contains_in(&self, expression: Expr) -> bool {
        self.opaque_initializer_contains_in(
            self.ast.expr_data(expression),
            expression.span(self.ast),
        )
    }

    /// A real source pair already supplies the opaque node's NoIn enclosure.
    pub(super) fn opaque_initializer_needs_enclosure(&self, data: ExprData, scope: Span) -> bool {
        !matches!(data, ExprData::ParenthesizedExpression(_))
            && self.opaque_initializer_contains_in(data, scope)
    }

    fn opaque_initializer_contains_in(&self, data: ExprData, scope: Span) -> bool {
        let ast = self.ast;
        let within = |span: Span| scope.start <= span.start && span.end <= scope.end;
        let contains =
            |expression| self.opaque_initializer_contains_in(ast.expr_data(expression), scope);
        match data {
            ExprData::BinaryExpression(node) => {
                within(node.span(ast))
                    && (node.operator(ast) == BinaryOperator::In
                        || contains(node.left(ast))
                        || contains(node.right(ast)))
            }
            ExprData::LogicalExpression(node) => {
                within(node.span(ast)) && (contains(node.left(ast)) || contains(node.right(ast)))
            }
            ExprData::ParenthesizedExpression(node) => {
                within(node.span(ast)) && contains(node.expression(ast))
            }
            ExprData::TsAsExpression(node) => {
                within(node.span(ast)) && contains(node.expression(ast))
            }
            ExprData::TsSatisfiesExpression(node) => {
                within(node.span(ast)) && contains(node.expression(ast))
            }
            ExprData::AssignmentExpression(node) => {
                within(node.span(ast)) && contains(node.right(ast))
            }
            ExprData::SequenceExpression(node) => {
                within(node.span(ast))
                    && node
                        .expressions(ast)
                        .iter()
                        .any(|id| contains(ast.get_node_in_sub_range(id)))
            }
            ExprData::ConditionalExpression(node) => {
                within(node.span(ast))
                    && (contains(node.test(ast)) || contains(node.alternate(ast)))
            }
            ExprData::UnaryExpression(node) => {
                within(node.span(ast)) && contains(node.argument(ast))
            }
            ExprData::AwaitExpression(node) => {
                within(node.span(ast)) && contains(node.argument(ast))
            }
            ExprData::YieldExpression(node) => {
                within(node.span(ast)) && node.argument(ast).is_some_and(contains)
            }
            ExprData::ArrowFunctionExpression(node) => {
                within(node.span(ast)) && {
                    match ast.arrow_function_body_data(node.body(ast)) {
                        ArrowFunctionBodyData::Expr(body) => contains(body),
                        ArrowFunctionBodyData::FunctionBody(_) => false,
                    }
                }
            }
            _ => false,
        }
    }
}
