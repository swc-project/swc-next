//! ES2020 transform options.

use swc_next_ecma_ast::TraverseMutCtx;

use crate::{
    common::{TraverseCtx, var_declarations::hook_pass_with_var_declarations},
    es2022::optional_chaining_impl::optional_chaining_impl,
};

use self::nullish_coalescing::NullishCoalescingPass;

mod nullish_coalescing;

pub use crate::es2022::optional_chaining_impl::Config as OptionalChainingOptions;
pub use nullish_coalescing::Config as NullishCoalescingOptions;

/// Selects ES2020 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2020Options {
    /// Lower `export * as namespace from "module"`.
    pub export_namespace_from: bool,
    /// Lower `??`. `None` disables the pass.
    pub nullish_coalescing: Option<NullishCoalescingOptions>,
    /// Lower optional chaining. `None` disables the pass.
    pub optional_chaining: Option<OptionalChainingOptions>,
}

mod export_namespace_from;

pub(crate) fn transform(ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>, options: ES2020Options) {
    if let Some(config) = options.optional_chaining {
        optional_chaining_impl(ctx.ast, ctx.state.resolver, config);
    }
    if options.export_namespace_from && ctx.ast.is_module() {
        export_namespace_from::pass(ctx.ast, ctx.state.resolver);
    }
    if let Some(config) = options.nullish_coalescing {
        hook_pass_with_var_declarations(NullishCoalescingPass(config), ctx);
    }
}
