use swc_next_ecma_ast::{GetSpan, RegExpLiteral};

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
                ast.get_utf8(flags).contains('y'),
            )
        };
        if !is_sticky {
            return;
        }

        let ast = &mut *ctx.ast;
        let pattern = ast.argument_expr_string_literal(span, pattern.into(), ast.empty_utf8());
        let flags = ast.argument_expr_string_literal(span, flags.into(), ast.empty_utf8());
        let arguments = ast.add_typed_sub_range([pattern, flags]);
        let reg_exp_name = ast.add_utf8("RegExp");
        let callee = ast.expr_identifier_reference(span, reg_exp_name);
        let replacement = ast.new_expression(span, callee, None, arguments);
        // SAFETY: `replacement` is the unattached last node, and this exit hook
        // does not access `literal` after replacing it.
        unsafe { ast.replace_data(literal, replacement) };
    }
}
