// TODO: Support inline helpers and provide swc-next's own helper library.

mod builder;

use crate::context::Mutation;
use swc_next_ecma_ast::{Argument, AstUtf8, DUMMY_SP, Expr, ScopeId, Stmt};

/// Prepared external helper imports, allocated on first use and attached once.
#[derive(Default)]
pub(crate) struct Helpers {
    set_function_name: Option<PreparedHelper>,
    to_property_key: Option<PreparedHelper>,
}

impl Helpers {
    pub(crate) fn name(&self, helper: HelperKind) -> Option<AstUtf8> {
        let entry = match helper {
            HelperKind::SetFunctionName => &self.set_function_name,
            HelperKind::ToPropertyKey => &self.to_property_key,
        };
        entry.as_ref().map(|entry| entry.name)
    }

    pub(crate) fn insert(&mut self, helper: HelperKind, prepared: PreparedHelper) {
        let entry = match helper {
            HelperKind::SetFunctionName => &mut self.set_function_name,
            HelperKind::ToPropertyKey => &mut self.to_property_key,
        };
        debug_assert!(entry.is_none());
        *entry = Some(prepared);
    }

    pub(crate) fn take_statements(&mut self) -> [Option<Stmt>; 2] {
        // Keep import order stable regardless of the order of first use.
        [self.to_property_key.take(), self.set_function_name.take()]
            .map(|helper| helper.map(|helper| helper.statement))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.set_function_name.is_none() && self.to_property_key.is_none()
    }
}

pub(crate) struct PreparedHelper {
    pub(crate) name: AstUtf8,
    statement: Stmt,
}

#[derive(Clone, Copy)]
pub(crate) enum HelperKind {
    SetFunctionName,
    ToPropertyKey,
}

pub(crate) fn prepare(helper: HelperKind, ctx: &mut Mutation<'_, '_, '_>) -> PreparedHelper {
    use swc_next_ecma_semantic::SymbolFlags;
    let base = match helper {
        HelperKind::SetFunctionName => "_set_function_name",
        HelperKind::ToPropertyKey => "_swc_helpers",
    };
    let flags = if ctx.ast().is_module() {
        SymbolFlags::IMPORT
    } else {
        SymbolFlags::FUNCTION_SCOPED_VAR
    };
    let binding = ctx.generate_binding(base, ctx.program_scope(), flags);
    let name = binding.declaration.name(ctx.ast());
    let statement = builder::external_import(name, helper, ctx);
    PreparedHelper { name, statement }
}

pub(crate) fn callee(
    name: AstUtf8,
    helper: HelperKind,
    scope: ScopeId,
    ctx: &mut Mutation<'_, '_, '_>,
) -> Expr {
    match helper {
        HelperKind::ToPropertyKey => builder::external_member(name, "_to_property_key", scope, ctx),
        HelperKind::SetFunctionName => {
            Expr::IdentifierReference(ctx.generated_reference_in_scope(name, DUMMY_SP, scope))
        }
    }
}

pub(crate) fn call<const N: usize>(
    callee: Expr,
    arguments: [Argument; N],
    ctx: &mut Mutation<'_, '_, '_>,
) -> Expr {
    builder::call(callee, arguments, ctx)
}

pub(crate) fn call_set_function_name(
    target: Expr,
    name: Expr,
    ctx: &mut Mutation<'_, '_, '_>,
) -> Expr {
    let scope = ctx
        .semantic()
        .scope_outside(target.node_id())
        .expect("helper target is registered");
    let callee = ctx.helper_callee(HelperKind::SetFunctionName, scope);
    call(callee, [Argument::Expr(target), Argument::Expr(name)], ctx)
}

pub(crate) fn call_to_property_key(argument: Expr, ctx: &mut Mutation<'_, '_, '_>) -> Expr {
    let scope = ctx
        .semantic()
        .scope_outside(argument.node_id())
        .expect("helper argument is registered");
    let callee = ctx.helper_callee(HelperKind::ToPropertyKey, scope);
    call(callee, [Argument::Expr(argument)], ctx)
}

#[cfg(test)]
#[path = "../tests/helpers.rs"]
mod tests;
