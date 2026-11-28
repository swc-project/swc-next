//! Parentheses at the boundary between an expression's type arguments and its parent.

use swc_next_ecma_ast::{
    Ast, BinaryOperator, Expr, ExprData, GetSpan, Span, TsInstantiationExpression,
};

use super::super::binary::{BinaryLike, BinaryOperatorKind, OperandSide};
use super::super::{AccessKind, ExpressionPosition, Formatter};
use crate::formatter::suppression::ContentMode;

impl Formatter<'_, '_> {
    #[inline]
    pub(in crate::formatter::expressions) fn ts_binary_operand_needs_parentheses(
        &self,
        data: ExprData,
        operator: BinaryOperatorKind,
        side: OperandSide,
        mode: ContentMode,
    ) -> bool {
        self.ast.is_ts() && self.instantiation_operand_needs_parentheses(data, operator, side, mode)
    }

    fn instantiation_operand_needs_parentheses(
        &self,
        data: ExprData,
        operator: BinaryOperatorKind,
        side: OperandSide,
        mode: ContentMode,
    ) -> bool {
        let BinaryOperatorKind::Binary(operator) = operator else {
            return false;
        };
        if side == OperandSide::Right && operator == BinaryOperator::LeftShift {
            // TypeScript can read `x << (f<T>)` as the start of a type argument list.
            return false;
        }
        // Keep the formatter's grouping on the operand that owns the angle list.
        // This includes prefix operands even when the following token is unambiguous.
        if mode == ContentMode::Formatted && self.is_instantiation_operand(data) {
            return !matches!(
                (side, operator, data),
                (
                    OperandSide::Right,
                    BinaryOperator::In | BinaryOperator::Instanceof,
                    ExprData::TsInstantiationExpression(_)
                )
            );
        }
        if side != OperandSide::Left {
            return false;
        }
        match operator {
            // A break after `>` can make TypeScript read `a < b >` as `a<b>`.
            BinaryOperator::GreaterThan
                if matches!(data, ExprData::BinaryExpression(left)
                if left.operator(self.ast) == BinaryOperator::LessThan) =>
            {
                true
            }
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::RightShift
            | BinaryOperator::UnsignedRightShift => {
                self.right_edge_exposes_type_arguments(data, mode)
            }
            _ => false,
        }
    }

    fn right_edge_exposes_type_arguments(&self, mut data: ExprData, mut mode: ContentMode) -> bool {
        // Follow the emitted right edge. Once a subtree is copied, only its
        // source pairs can protect the generic suffix from the next operator.
        loop {
            let check_suppression = mode == ContentMode::Formatted && self.has_suppression;
            let mut child_mode = mode;
            let (operand, position) = match data {
                ExprData::TsInstantiationExpression(_) => return true,
                ExprData::UnaryExpression(node) => {
                    if mode == ContentMode::Formatted
                        && !self
                            .source_comments_in_range(node.span(self.ast))
                            .is_empty()
                    {
                        let Some(operand_mode) = self.unary_tail_operand_mode(node) else {
                            return false;
                        };
                        child_mode = operand_mode;
                    }
                    (
                        node.argument(self.ast),
                        ExpressionPosition::UnaryArgument(node.operator(self.ast)),
                    )
                }
                ExprData::AwaitExpression(node) => {
                    let argument = node.argument(self.ast);
                    if check_suppression {
                        let gap =
                            Span::new(node.span(self.ast).start + 5, argument.span(self.ast).start);
                        child_mode = self.source_content_mode(gap);
                    }
                    (argument, ExpressionPosition::AwaitArgument)
                }
                ExprData::TsTypeAssertion(node) => {
                    if check_suppression {
                        let Ok(closing) = self.type_assertion_closing(node) else {
                            return true;
                        };
                        child_mode = self.content_mode(closing.operand_head);
                    }
                    (
                        node.expression(self.ast),
                        ExpressionPosition::TypeAssertionOperand,
                    )
                }
                ExprData::ChainExpression(node) => {
                    (node.expression(self.ast), ExpressionPosition::Nested)
                }
                _ => {
                    let Some(binary) = BinaryLike::from_data(data, self.ast) else {
                        return false;
                    };
                    if check_suppression {
                        let Ok(operand_mode) = self.binary_right_mode(binary) else {
                            return true;
                        };
                        child_mode = operand_mode;
                    }
                    (
                        binary.right,
                        ExpressionPosition::BinaryOperand {
                            operator: binary.operator,
                            side: OperandSide::Right,
                        },
                    )
                }
            };
            let Some(edge) = self.exposed_right_operand(operand, position, mode, child_mode) else {
                return false;
            };
            (data, mode) = edge;
        }
    }

    fn is_instantiation_operand(&self, mut data: ExprData) -> bool {
        loop {
            let operand = match data {
                ExprData::TsInstantiationExpression(_) => return true,
                ExprData::UnaryExpression(node) => node.argument(self.ast),
                ExprData::AwaitExpression(node) => node.argument(self.ast),
                ExprData::YieldExpression(node) => {
                    let Some(argument) = node.argument(self.ast) else {
                        return false;
                    };
                    argument
                }
                ExprData::TsTypeAssertion(node) => node.expression(self.ast),
                _ => return false,
            };
            data = self.ast.expr_data(operand);
            while let ExprData::ParenthesizedExpression(pair) = data {
                data = self.ast.expr_data(pair.expression(self.ast));
            }
        }
    }

    pub(in crate::formatter::expressions) fn shift_suspension_needs_grammar_barrier(
        &self,
        data: ExprData,
        position: ExpressionPosition,
    ) -> bool {
        // The ordinary suspension pair alone lets TypeScript misread `x << (yield f<T>)`.
        is_left_shift_right_operand(position)
            && matches!(
                data,
                ExprData::AwaitExpression(_) | ExprData::YieldExpression(_)
            )
            && self.ast.is_ts()
            && self.is_instantiation_operand(data)
    }
}

pub(super) fn is_left_shift_right_operand(position: ExpressionPosition) -> bool {
    matches!(
        position,
        ExpressionPosition::BinaryOperand {
            operator: BinaryOperatorKind::Binary(BinaryOperator::LeftShift),
            side: OperandSide::Right,
        }
    )
}

pub(super) fn needs_parentheses(
    node: TsInstantiationExpression,
    position: ExpressionPosition,
    ast: &Ast<'_>,
) -> bool {
    match position {
        ExpressionPosition::MemberObject(_)
        | ExpressionPosition::MemberAssignmentObject
        | ExpressionPosition::TemplateTag
        | ExpressionPosition::NonNullOperand
        | ExpressionPosition::InstantiationOperand => true,
        ExpressionPosition::CallCallee {
            access: AccessKind::Ordinary,
            has_type_arguments,
            ..
        }
        | ExpressionPosition::NewCallee { has_type_arguments }
        | ExpressionPosition::Superclass { has_type_arguments } => {
            has_type_arguments || has_nested_instantiation(node.expression(ast), ast)
        }
        _ => false,
    }
}

// An outer call, new, or heritage may absorb the last angle list. For
// nested instantiations that would turn two separate instantiations into one.
fn has_nested_instantiation(mut expression: Expr, ast: &Ast<'_>) -> bool {
    loop {
        match ast.expr_data(expression) {
            ExprData::ParenthesizedExpression(pair) => expression = pair.expression(ast),
            ExprData::ChainExpression(chain) => expression = chain.expression(ast),
            ExprData::TsInstantiationExpression(_) => return true,
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use swc_next_ecma_ast::{ExprData, StmtData};

    use super::{ContentMode, Formatter};
    use crate::formatter::with_first_statement;

    #[test]
    fn binary_operand_comments() {
        with_first_statement(
            "(await /* prettier-ignore */ (fn<T>)) - b;",
            |formatter, statement| {
                let ast = formatter.ast;
                let StmtData::ExpressionStatement(statement) = ast.stmt_data(statement) else {
                    unreachable!()
                };
                let ExprData::BinaryExpression(binary) = ast.expr_data(statement.expression(ast))
                else {
                    unreachable!()
                };
                let left = formatter.expression_data(binary.left(ast));
                let exposed = |f: &Formatter<'_, '_>| {
                    f.right_edge_exposes_type_arguments(left, ContentMode::Formatted)
                };

                assert!(!exposed(formatter));
                formatter.comments.claim(ast.comments()).unwrap();
                assert!(!exposed(formatter));
            },
        );
    }
}
