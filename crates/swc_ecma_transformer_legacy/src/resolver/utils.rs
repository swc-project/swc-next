use swc_next_ecma_ast::{
    Ast, BindingIdentifier, BindingPattern, Expr, PropertyKey, Visit, VisitWith,
};

struct BindingIdVisitor<'op, 'ast, F> {
    ast: &'ast Ast<'ast>,
    op: &'op mut F,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1724>.
pub(super) fn for_each_binding_id<'ast>(
    ast: &'ast Ast<'ast>,
    pattern: BindingPattern,
    mut op: impl FnMut(BindingIdentifier),
) {
    pattern.visit_with(&mut BindingIdVisitor { ast, op: &mut op });
}

impl<'ast, F: FnMut(BindingIdentifier)> Visit<'ast> for BindingIdVisitor<'_, 'ast, F> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1724>.
    fn visit_binding_identifier(&mut self, identifier: BindingIdentifier) {
        (self.op)(identifier);
    }

    /// Initializers and computed expressions do not introduce bindings.
    ///
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1724>.
    fn visit_expr(&mut self, _: Expr) {}

    /// A non-computed property name does not introduce a binding either.
    ///
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1724>.
    fn visit_property_key(&mut self, _: PropertyKey) {}
}
