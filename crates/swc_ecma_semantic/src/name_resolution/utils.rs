use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingIdentifier, BindingPattern, Expr, FormalParameters, IdentifierReference,
    NodeId, PropertyKey, Visit, VisitWith,
};

#[derive(Clone, Copy)]
pub struct Binding {
    pub node_id: NodeId,
    pub name: AstUtf8,
}

struct DestructuringFinder<'found, 'ast> {
    ast: &'ast Ast<'ast>,
    found: &'found mut Vec<Binding>,
}

pub fn find_binding_ids<'ast>(
    ast: &'ast Ast<'ast>,
    pattern: BindingPattern,
    found: &mut Vec<Binding>,
) {
    pattern.visit_with(&mut DestructuringFinder { ast, found });
}

pub fn find_parameter_ids<'ast>(
    ast: &'ast Ast<'ast>,
    parameters: FormalParameters,
    found: &mut Vec<Binding>,
) {
    parameters.visit_with(&mut DestructuringFinder { ast, found });
}

impl<'ast> Visit<'ast> for DestructuringFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_binding_identifier(&mut self, identifier: BindingIdentifier) {
        self.found.push(Binding {
            node_id: identifier.node_id(),
            name: identifier.name(self.ast),
        });
    }

    fn visit_identifier_reference(&mut self, identifier: IdentifierReference) {
        // The flat AST represents identifiers nested inside some binding
        // patterns as simple assignment targets. They are bindings here even
        // though the shared node kind is `IdentifierReference`.
        self.found.push(Binding {
            node_id: identifier.node_id(),
            name: identifier.name(self.ast),
        });
    }

    /// Initializers and computed expressions do not introduce bindings.
    fn visit_expr(&mut self, _: Expr) {}

    /// A non-computed property name does not introduce a binding either.
    fn visit_property_key(&mut self, _: PropertyKey) {}
}
