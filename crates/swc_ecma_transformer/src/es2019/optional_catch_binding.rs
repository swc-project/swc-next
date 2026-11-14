use swc_next_ecma_ast::{BindingPattern, CatchClause};
use swc_next_ecma_semantic::SymbolFlags;

use crate::context::{BoundIdentifier, TransformCtx};

pub(super) struct OptionalCatchBinding;

impl OptionalCatchBinding {
    pub fn enter_catch_clause(&mut self, node: CatchClause, ctx: &mut TransformCtx<'_, '_>) {
        if node.param(ctx.ast()).is_some() {
            return;
        }

        let scope = ctx.current_scope();
        let binding = BoundIdentifier::named(
            ctx,
            scope,
            "_unused",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::CATCH_VAR,
        );
        node.set_param(
            ctx.ast,
            Some(BindingPattern::BindingIdentifier(binding.identifier)),
        );
    }
}
