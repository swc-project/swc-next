//! Expression token emission for primitive values.

use swc_next_ecma_ast::{Expr, ExprData};

use super::Formatter;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_expression(
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
            ExprData::NumericLiteral(node) => self.format_numeric_literal(node, writer),
            ExprData::BigIntLiteral(node) => self.format_bigint_literal(node, writer),
            ExprData::RegExpLiteral(node) => self.format_regexp_literal(node, writer),
            ExprData::ThisExpression(_) => writer.text("this"),
            _ => return Err(self.unsupported("expression", expression)),
        }
        Ok(())
    }
}
