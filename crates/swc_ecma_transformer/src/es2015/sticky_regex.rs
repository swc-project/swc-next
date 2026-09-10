use swc_next_ecma_ast::{Argument, Expr, GetSpan, RegExpLiteral};

use crate::context::Mutation;

pub(super) struct StickyRegex;

impl StickyRegex {
    pub fn exit_reg_exp_literal(&mut self, literal: RegExpLiteral, ctx: &mut Mutation<'_, '_, '_>) {
        let flags = literal.flags(ctx.ast());
        if !ctx.ast().get_utf8(flags).contains('y') {
            return;
        }
        let span = literal.span(ctx.ast());
        let pattern = literal.pattern(ctx.ast());
        let pattern = Expr::StringLiteral(ctx.add_node(ctx.current_scope(), |ast| {
            ast.string_literal(span, pattern.into(), ast.empty_utf8())
        }));
        let flags = Expr::StringLiteral(ctx.add_node(ctx.current_scope(), |ast| {
            ast.string_literal(span, flags.into(), ast.empty_utf8())
        }));
        let arguments = ctx.add_sub_range([Argument::Expr(pattern), Argument::Expr(flags)]);
        let scope = ctx.current_scope();
        let callee = Expr::IdentifierReference(ctx.create_reference("RegExp", scope));
        let replacement = Expr::NewExpression(ctx.add_node(ctx.current_scope(), |ast| {
            ast.new_expression(span, callee, None, arguments)
        }));
        ctx.replace_current_expr(replacement);
    }
}
