use swc_next_ecma_ast::{
    Ast, BindingIdentifier, BindingPattern, Expr, PropertyKey, Visit, VisitWith,
};

struct BindingIdVisitor<'op, 'ast, F> {
    ast: &'ast Ast<'ast>,
    op: &'op mut F,
}

pub fn for_each_binding_id<'ast>(
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

    fn visit_binding_identifier(&mut self, identifier: BindingIdentifier) {
        (self.op)(identifier);
    }

    /// Initializers and computed expressions do not introduce bindings.
    fn visit_expr(&mut self, _: Expr) {}

    /// A non-computed property name does not introduce a binding either.
    fn visit_property_key(&mut self, _: PropertyKey) {}
}
