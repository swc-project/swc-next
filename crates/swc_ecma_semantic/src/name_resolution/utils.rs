use swc_next_ecma_ast::{
    Ast, BindingIdentifier, BindingPattern, Expr, PropertyKey, Visit, VisitWith,
};

struct DestructuringFinder<'found, 'ast> {
    ast: &'ast Ast<'ast>,
    found: &'found mut Vec<BindingIdentifier>,
}

pub fn find_binding_ids<'ast>(
    ast: &'ast Ast<'ast>,
    pattern: BindingPattern,
    found: &mut Vec<BindingIdentifier>,
) {
    pattern.visit_with(&mut DestructuringFinder { ast, found });
}

impl<'ast> Visit<'ast> for DestructuringFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_binding_identifier(&mut self, identifier: BindingIdentifier) {
        self.found.push(identifier);
    }

    /// Initializers and computed expressions do not introduce bindings.
    fn visit_expr(&mut self, _: Expr) {}

    /// A non-computed property name does not introduce a binding either.
    fn visit_property_key(&mut self, _: PropertyKey) {}
}
