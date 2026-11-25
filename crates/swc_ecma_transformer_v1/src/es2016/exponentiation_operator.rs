use crate::common::{TraverseCtx, build};
use swc_next_ecma_ast::*;

pub(crate) struct ExponentiationOperatorPass;

impl ExponentiationOperatorPass {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2016/exponentiation_operator.rs#L91>.
    fn transform_exp_assign(
        &mut self,
        assignment: AssignmentExpression,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    ) -> Option<Expr> {
        let target = assignment
            .left(ctx.ast)
            .as_simple_assignment_target(ctx.ast)?;
        let right = assignment.right(ctx.ast);
        if let Some(id) = target.as_identifier_reference(ctx.ast) {
            let left = Expr::IdentifierReference(id);
            let read = build::clone_expr(ctx.ast, ctx.state.resolver, left);
            let pow = create_math_pow(ctx, read, right);
            return Some(build::assign(ctx.ast, left, pow));
        }
        let member = target.as_member_expression(ctx.ast)?;
        if member.object(ctx.ast).is_super(ctx.ast) {
            return None;
        }
        let mut object = member.object(ctx.ast);
        let mut property = member.property(ctx.ast);
        let mut expressions = Vec::new();
        if !object.is_identifier_reference(ctx.ast) {
            let name = temp_name(ctx.ast, object);
            let binding = ctx.state.insert_var(ctx.ast, &name);
            let reference = ctx.state.resolver.reference(ctx.ast, binding);
            expressions.push(build::assign(ctx.ast, reference, object));
            object = ctx.state.resolver.reference(ctx.ast, binding);
        }
        if member.computed(ctx.ast)
            && let Some(expression) = property.as_expr(ctx.ast)
            && !is_literal_expr(ctx.ast, expression)
        {
            let name = expression
                .as_identifier_reference(ctx.ast)
                .map(|id| format!("_{}", ctx.ast.get_utf8(id.name(ctx.ast))))
                .unwrap_or_else(|| "_prop".into());
            let binding = ctx.state.insert_var(ctx.ast, &name);
            let reference = ctx.state.resolver.reference(ctx.ast, binding);
            expressions.push(build::assign(ctx.ast, reference, expression));
            property = PropertyKey::Expr(ctx.state.resolver.reference(ctx.ast, binding));
        }
        let left = ctx.ast.expr_member_expression(
            DUMMY_SP,
            object,
            property,
            member.computed(ctx.ast),
            false,
        );
        let read = build::clone_expr(ctx.ast, ctx.state.resolver, left);
        let pow = create_math_pow(ctx, read, right);
        expressions.push(build::assign(ctx.ast, left, pow));
        Some(build::sequence(ctx.ast, expressions))
    }
}

impl<'ast> TraverseMut<'ast, TraverseCtx<'_>> for ExponentiationOperatorPass {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2016/exponentiation_operator.rs#L48>.
    fn enter_expr(
        &mut self,
        expression: Expr,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Expr> {
        let replacement = match ctx.ast.expr_data(expression) {
            ExprData::BinaryExpression(binary)
                if binary.operator(ctx.ast) == BinaryOperator::Exponent
                    && !binary.left(ctx.ast).is_big_int_literal(ctx.ast)
                    && !binary.right(ctx.ast).is_big_int_literal(ctx.ast) =>
            {
                Some(create_math_pow(
                    ctx,
                    binary.left(ctx.ast),
                    binary.right(ctx.ast),
                ))
            }
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(ctx.ast) == AssignmentOperator::ExponentAssign
                    && !assignment.right(ctx.ast).is_big_int_literal(ctx.ast) =>
            {
                self.transform_exp_assign(assignment, ctx)
            }
            _ => None,
        };
        replacement.map_or(TraverseAction::Continue, TraverseAction::ReplaceContinue)
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2016/exponentiation_operator.rs#L42>.
fn temp_name(ast: &Ast<'_>, expression: Expr) -> String {
    if let Some(member) = expression.as_member_expression(ast) {
        let object = member
            .object(ast)
            .as_identifier_reference(ast)
            .map(|id| ast.get_utf8(id.name(ast)))
            .unwrap_or("ref");
        if let Some(property) = member.property(ast).as_identifier_name(ast) {
            format!("_{object}_{}", ast.get_utf8(property.name(ast)))
        } else {
            format!("_{object}")
        }
    } else if let Some(id) = expression.as_identifier_reference(ast) {
        format!("_{}", ast.get_utf8(id.name(ast)))
    } else {
        "_ref".into()
    }
}

fn is_literal_expr(ast: &Ast<'_>, expression: Expr) -> bool {
    matches!(
        ast.expr_data(expression),
        ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::RegExpLiteral(_)
            | ExprData::IdentifierReference(_)
            | ExprData::ThisExpression(_)
            | ExprData::PrivateIdentifier(_)
    )
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2016/exponentiation_operator.rs#L276>.
fn create_math_pow(
    ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    left: Expr,
    right: Expr,
) -> Expr {
    let math = ctx.state.resolver.marked_reference(ctx.ast, "Math", true);
    let callee = build::member(ctx.ast, math, "pow");
    build::call(ctx.ast, callee, [left, right], false)
}
