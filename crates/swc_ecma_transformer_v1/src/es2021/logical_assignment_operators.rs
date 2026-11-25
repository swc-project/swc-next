use crate::common::{TraverseCtx, build};
use swc_next_ecma_ast::*;

pub(crate) struct LogicalAssignmentOperatorsPass;

impl<'ast> TraverseMut<'ast, TraverseCtx<'_>> for LogicalAssignmentOperatorsPass {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2021/logical_assignment_operators.rs#L48>.
    fn enter_expr(
        &mut self,
        expression: Expr,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Expr> {
        let Some(assignment) = expression.as_assignment_expression(ctx.ast) else {
            return TraverseAction::Continue;
        };
        let operator = match assignment.operator(ctx.ast) {
            AssignmentOperator::LogicalAndAssign => LogicalOperator::And,
            AssignmentOperator::LogicalOrAssign => LogicalOperator::Or,
            AssignmentOperator::NullishAssign => LogicalOperator::NullishCoalescing,
            _ => return TraverseAction::Continue,
        };
        let Some(target) = assignment
            .left(ctx.ast)
            .as_simple_assignment_target(ctx.ast)
        else {
            return TraverseAction::Continue;
        };
        let (read, write) = if let Some(member) = target.as_member_expression(ctx.ast) {
            transform_member_expression(ctx, member)
        } else {
            let read = build::target_expr(ctx.ast, target);
            let read = build::clone_expr(ctx.ast, ctx.state.resolver, read);
            (read, build::target_expr(ctx.ast, target))
        };
        let value = build::assign(ctx.ast, write, assignment.right(ctx.ast));
        TraverseAction::ReplaceContinue(ctx.ast.expr_logical_expression(
            assignment.span(ctx.ast),
            read,
            value,
            operator,
        ))
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2021/logical_assignment_operators.rs#L127>.
fn transform_member_expression(
    ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    member: MemberExpression,
) -> (Expr, Expr) {
    let object = member.object(ctx.ast);
    let (read_object, write_object) =
        if object.is_this_expression(ctx.ast) || object.is_super(ctx.ast) {
            (
                build::clone_expr(ctx.ast, ctx.state.resolver, object),
                object,
            )
        } else {
            let name = build::alias_ident_for(ctx.ast, object, "_ref");
            let binding = ctx.state.insert_var(ctx.ast, &name);
            let left = ctx.state.resolver.reference(ctx.ast, binding);
            (
                build::assign(ctx.ast, left, object),
                ctx.state.resolver.reference(ctx.ast, binding),
            )
        };
    let property = member.property(ctx.ast);
    let (read_property, write_property) = if member.computed(ctx.ast) {
        let expression = property.as_expr(ctx.ast).unwrap();
        let name = build::alias_ident_for(ctx.ast, expression, "_ref");
        let binding = ctx.state.insert_var(ctx.ast, &name);
        let left = ctx.state.resolver.reference(ctx.ast, binding);
        (
            PropertyKey::Expr(build::assign(ctx.ast, left, expression)),
            PropertyKey::Expr(ctx.state.resolver.reference(ctx.ast, binding)),
        )
    } else {
        (property, property)
    };
    let read = ctx.ast.expr_member_expression(
        member.span(ctx.ast),
        read_object,
        read_property,
        member.computed(ctx.ast),
        false,
    );
    let write = ctx.ast.expr_member_expression(
        member.span(ctx.ast),
        write_object,
        write_property,
        member.computed(ctx.ast),
        false,
    );
    (read, write)
}
