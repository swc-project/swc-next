//! Grammar right edges of syntax copied verbatim.

use swc_next_ecma_ast::{ArrowFunctionBodyData, Ast, Expr, ExprData};

use super::super::types::{RawTypeTerminal, raw_type_terminal};
use super::targets::SimpleTarget;

/// Whether this exact raw expression needs a following grammar fence.
pub(in crate::formatter) fn raw_expression_has_open_type_end(node: Expr, ast: &Ast<'_>) -> bool {
    raw_expression_terminal(node, ast) != RawTypeTerminal::Closed
}

pub(in crate::formatter) fn raw_expression_terminal(
    mut node: Expr,
    ast: &Ast<'_>,
) -> RawTypeTerminal {
    loop {
        node = match ast.expr_data(node) {
            ExprData::TsAsExpression(node) => {
                return raw_type_terminal(node.type_annotation(ast), ast);
            }
            ExprData::TsSatisfiesExpression(node) => {
                return raw_type_terminal(node.type_annotation(ast), ast);
            }
            ExprData::AssignmentExpression(node) => node.right(ast),
            ExprData::BinaryExpression(node) => node.right(ast),
            ExprData::LogicalExpression(node) => node.right(ast),
            ExprData::ConditionalExpression(node) => node.alternate(ast),
            ExprData::SequenceExpression(node) => {
                let Some(last) = node.expressions(ast).last() else {
                    return RawTypeTerminal::Closed;
                };
                ast.get_node_in_sub_range(last)
            }
            ExprData::ArrowFunctionExpression(node) => {
                match ast.arrow_function_body_data(node.body(ast)) {
                    ArrowFunctionBodyData::Expr(body) => body,
                    ArrowFunctionBodyData::FunctionBody(_) => return RawTypeTerminal::Closed,
                }
            }
            ExprData::UnaryExpression(node) => node.argument(ast),
            ExprData::AwaitExpression(node) => node.argument(ast),
            ExprData::YieldExpression(node) => {
                let Some(argument) = node.argument(ast) else {
                    return RawTypeTerminal::Closed;
                };
                argument
            }
            ExprData::TsTypeAssertion(node) => node.expression(ast),
            ExprData::ChainExpression(node) => node.expression(ast),
            ExprData::UpdateExpression(node) if node.prefix(ast) => {
                SimpleTarget::new(ast, node.argument(ast)).expression()
            }
            // The copied pair, postfix token, closing delimiter, or atom is the real end.
            ExprData::IdentifierReference(_)
            | ExprData::ThisExpression(_)
            | ExprData::Super(_)
            | ExprData::MetaProperty(_)
            | ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::RegExpLiteral(_)
            | ExprData::TemplateLiteral(_)
            | ExprData::UpdateExpression(_)
            | ExprData::MemberExpression(_)
            | ExprData::CallExpression(_)
            | ExprData::NewExpression(_)
            | ExprData::TaggedTemplateExpression(_)
            | ExprData::ArrayExpression(_)
            | ExprData::ObjectExpression(_)
            | ExprData::ParenthesizedExpression(_)
            | ExprData::Function(_)
            | ExprData::Class(_)
            | ExprData::ImportExpression(_)
            | ExprData::PrivateIdentifier(_)
            | ExprData::TsNonNullExpression(_)
            | ExprData::TsInstantiationExpression(_)
            | ExprData::JsxElement(_)
            | ExprData::JsxFragment(_) => return RawTypeTerminal::Closed,
        };
    }
}
