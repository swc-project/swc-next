use swc_experimental_ecma_ast::{GetSpan, RegExpLiteral};

use crate::state::TransformerCtx;

pub(super) struct StickyRegex;

impl StickyRegex {
    pub fn exit_reg_exp_literal(
        &mut self,
        literal: RegExpLiteral,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let (span, pattern, flags, is_sticky) = {
            let ast = &*ctx.ast;
            let flags = literal.flags(ast);
            (
                literal.span(ast),
                literal.pattern(ast),
                flags,
                ast.string(flags).contains(&b'y'),
            )
        };
        if !is_sticky {
            return;
        }

        let ast = &mut *ctx.ast;
        let pattern = ast.argument_expr_string_literal(span, pattern, ast.empty_string());
        let flags = ast.argument_expr_string_literal(span, flags, ast.empty_string());
        let arguments = ast.add_typed_sub_range([pattern, flags]);
        let reg_exp_name = ast.add_string(b"RegExp");
        let callee = ast.expr_identifier_reference(span, reg_exp_name, None);
        let replacement = ast.new_expression(span, callee, arguments, None);
        ast.replace_data(literal, replacement);
    }
}
