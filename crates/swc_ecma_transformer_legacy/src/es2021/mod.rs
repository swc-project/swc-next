//! ES2021 transform options.

use swc_next_ecma_ast::TraverseMutCtx;

use crate::common::{TraverseCtx, var_declarations::hook_pass_with_var_declarations};

use self::logical_assignment_operators::LogicalAssignmentOperatorsPass;

/// Selects ES2021 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2021Options {
    /// Lower `&&=`, `||=`, and `??=` using SWC's `logical_assignments` pass.
    /// Nullish coalescing produced by `??=` is selected separately through
    /// [`ES2020Options::nullish_coalescing`](crate::es2020::ES2020Options::nullish_coalescing).
    pub logical_assignments: bool,
}

mod logical_assignment_operators;

pub(crate) fn transform(ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>, options: ES2021Options) {
    if options.logical_assignments {
        hook_pass_with_var_declarations(LogicalAssignmentOperatorsPass, ctx);
    }
}
