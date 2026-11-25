/// SWC's `nullish_coalescing::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Assume values cannot be the special `document.all` object, allowing
    /// `value != null` instead of separate null and undefined comparisons.
    pub no_document_all: bool,
}

use crate::common::{TraverseCtx, build};
use swc_next_ecma_ast::*;

pub(crate) struct NullishCoalescingPass(pub(crate) Config);

impl<'ast> TraverseMut<'ast, TraverseCtx<'_>> for NullishCoalescingPass {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2020/nullish_coalescing.rs#L60>.
    fn enter_expr(
        &mut self,
        expression: Expr,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Expr> {
        let replacement = match ctx.ast.expr_data(expression) {
            ExprData::LogicalExpression(logical)
                if logical.operator(ctx.ast) == LogicalOperator::NullishCoalescing =>
            {
                Some(self.transform_nullish_coalescing(ctx, logical))
            }
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(ctx.ast) == AssignmentOperator::NullishAssign =>
            {
                self.transform_nullish_coalescing_assign(ctx, assignment)
            }
            _ => None,
        };
        replacement.map_or(TraverseAction::Continue, TraverseAction::ReplaceContinue)
    }
}

impl NullishCoalescingPass {
    fn transform_nullish_coalescing(
        &self,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
        logical: LogicalExpression,
    ) -> Expr {
        let left = logical.left(ctx.ast);
        let (first, consequent, rest) = if left.is_identifier_reference(ctx.ast) {
            let first = build::clone_expr(ctx.ast, ctx.state.resolver, left);
            let rest = build::clone_expr(ctx.ast, ctx.state.resolver, left);
            (first, left, rest)
        } else {
            let name = build::alias_ident_for(ctx.ast, left, "ref");
            let binding = ctx.state.insert_var(ctx.ast, &name);
            let reference = ctx.state.resolver.reference(ctx.ast, binding);
            let first = build::assign(ctx.ast, reference, left);
            (
                first,
                ctx.state.resolver.reference(ctx.ast, binding),
                ctx.state.resolver.reference(ctx.ast, binding),
            )
        };
        let test = create_nullish_test(ctx.ast, first, rest, self.0.no_document_all);
        ctx.ast
            .expr_conditional_expression(DUMMY_SP, test, consequent, logical.right(ctx.ast))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2020/nullish_coalescing.rs#L139>.
    fn transform_nullish_coalescing_assign(
        &mut self,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
        assignment: AssignmentExpression,
    ) -> Option<Expr> {
        let target = assignment
            .left(ctx.ast)
            .as_simple_assignment_target(ctx.ast)?;
        let left = build::target_expr(ctx.ast, target);
        let right = assignment.right(ctx.ast);
        if target.is_identifier_reference(ctx.ast) {
            let first = build::clone_expr(ctx.ast, ctx.state.resolver, left);
            let rest = build::clone_expr(ctx.ast, ctx.state.resolver, left);
            let consequent = build::clone_expr(ctx.ast, ctx.state.resolver, left);
            let test = create_nullish_test(ctx.ast, first, rest, self.0.no_document_all);
            let value = ctx
                .ast
                .expr_conditional_expression(DUMMY_SP, test, consequent, right);
            Some(build::assign(ctx.ast, left, value))
        } else {
            let name = build::alias_ident_for(ctx.ast, left, "refs");
            let binding = ctx.state.insert_var(ctx.ast, &name);
            let reference = ctx.state.resolver.reference(ctx.ast, binding);
            let read = build::clone_expr(ctx.ast, ctx.state.resolver, left);
            let first = build::assign(ctx.ast, reference, read);
            let rest = ctx.state.resolver.reference(ctx.ast, binding);
            let test = create_nullish_test(ctx.ast, first, rest, self.0.no_document_all);
            let consequent = ctx.state.resolver.reference(ctx.ast, binding);
            let alternative = build::assign(ctx.ast, left, right);
            let value =
                ctx.ast
                    .expr_conditional_expression(DUMMY_SP, test, consequent, alternative);
            let left = ctx.state.resolver.reference(ctx.ast, binding);
            Some(build::assign(ctx.ast, left, value))
        }
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2020/nullish_coalescing.rs#L223>.
fn create_nullish_test(ast: &mut Ast<'_>, first: Expr, rest: Expr, no_document_all: bool) -> Expr {
    let null = ast.expr_null_literal(DUMMY_SP);
    let check = ast.expr_binary_expression(
        DUMMY_SP,
        first,
        null,
        if no_document_all {
            BinaryOperator::NotEqual
        } else {
            BinaryOperator::StrictNotEqual
        },
    );
    if no_document_all {
        check
    } else {
        let undefined = build::undefined(ast);
        let check_undefined =
            ast.expr_binary_expression(DUMMY_SP, rest, undefined, BinaryOperator::StrictNotEqual);
        ast.expr_logical_expression(DUMMY_SP, check, check_undefined, LogicalOperator::And)
    }
}
