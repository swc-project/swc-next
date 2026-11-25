//! ES2016 transform options.

use swc_next_ecma_ast::TraverseMutCtx;

use crate::common::{TraverseCtx, var_declarations::hook_pass_with_var_declarations};

use self::exponentiation_operator::ExponentiationOperatorPass;

/// Selects ES2016 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2016Options {
    /// Lower `**` and `**=` using SWC's `exponentiation` pass.
    pub exponentiation: bool,
}

mod exponentiation_operator;

pub(crate) fn transform(ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>, options: ES2016Options) {
    if options.exponentiation {
        hook_pass_with_var_declarations(ExponentiationOperatorPass, ctx);
    }
}
