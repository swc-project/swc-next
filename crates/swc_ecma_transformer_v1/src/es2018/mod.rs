//! ES2018 transform options. RegExp features are configured separately.

use swc_next_ecma_ast::TraverseMutCtx;

use crate::common::{TraverseCtx, var_declarations::hook_pass_with_var_declarations};

use self::object_rest_spread::ObjectRestSpreadPass;

mod object_rest_spread;

pub use object_rest_spread::Config as ObjectRestSpreadOptions;

/// Selects ES2018 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2018Options {
    /// Lower object rest/spread. `None` disables the pass.
    pub object_rest_spread: Option<ObjectRestSpreadOptions>,
}

pub(crate) fn transform(ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>, options: ES2018Options) {
    if let Some(config) = options.object_rest_spread {
        hook_pass_with_var_declarations(ObjectRestSpreadPass::new(config), ctx);
    }
}
