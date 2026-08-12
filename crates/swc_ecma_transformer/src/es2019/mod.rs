use swc_next_ecma_ast::CatchClause;

use crate::state::TransformerCtx;

mod optional_catch_binding;

pub struct ES2019 {
    optional_catch_binding: bool,
}

#[derive(Default)]
pub struct ES2019Options {
    pub optional_catch_binding: bool,
}

impl ES2019 {
    pub fn new(options: ES2019Options) -> Self {
        Self {
            optional_catch_binding: options.optional_catch_binding,
        }
    }

    pub(crate) fn enter_catch_clause(
        &mut self,
        node: CatchClause,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.optional_catch_binding {
            optional_catch_binding::OptionalCatchBinding.enter_catch_clause(node, ctx);
        }
    }
}
