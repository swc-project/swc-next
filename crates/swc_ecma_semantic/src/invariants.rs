//! Debug checks for semantic data after AST transformation.

use swc_next_ecma_ast::{
    Ast, BindingIdentifier, IdentifierReference, NodeKindData, Visit, VisitWith,
};

use crate::Semantic;

impl Semantic<'_> {
    /// Checks the semantic indexes of the final reachable AST.
    #[cfg(debug_assertions)]
    pub fn assert_invariants(&self, ast: &Ast<'_>) {
        assert_eq!(self.node_symbols.len(), ast.node_count());
        assert_eq!(self.node_references.len(), ast.node_count());
        let mut visitor = Invariants {
            ast,
            semantic: self,
            reachable: vec![false; ast.node_count()],
            path: Vec::new(),
        };
        ast.root_program().visit_with(&mut visitor);
        let reachable = visitor.reachable;
        for (index, scope) in self.node_scopes.iter().enumerate() {
            // Detached structural nodes remain in the arena until compilation
            // ends. Only reachable nodes need a current evaluation scope.
            if reachable[index] {
                let scope = scope.expect("reachable node must have a scope");
                assert!(self.scope(scope).active);
            }
        }

        let scope_count = self.scopes.list.len();
        for entry in self.iter_scopes() {
            assert!(
                reachable[entry.scope.node.index()],
                "scope owner must be reachable"
            );
            let mut parent = entry.scope.parent;
            let mut depth = 0;
            while let Some(id) = parent {
                assert!(self.scope(id).active);
                depth += 1;
                assert!(depth < scope_count, "scope parent chain must be acyclic");
                parent = self.scope(id).parent;
            }
            assert!(self.scope(entry.scope.hoist_target).active);
        }
        for entry in self.iter_symbols() {
            assert!(self.scope(self.scope_of(entry.id)).active);
            let declaration = self.declaration(entry.symbol.declaration);
            assert!(
                reachable[declaration.node.index()],
                "symbol declaration must be reachable"
            );
            assert!(self.symbol_of(declaration.node).is_some());
        }
        for entry in self.iter_references() {
            assert!(reachable[entry.reference.node.index()]);
            assert!(self.scope(entry.reference.scope).active);
            if let Some(symbol) = entry.reference.symbol {
                assert!(self.uses(symbol).contains(&entry.id));
            }
        }
        self.class.assert_invariants(ast, &reachable);
    }
}

#[cfg(debug_assertions)]
struct Invariants<'v, 'a> {
    ast: &'v Ast<'a>,
    semantic: &'v Semantic<'a>,
    reachable: Vec<bool>,
    path: Vec<NodeKindData>,
}

#[cfg(debug_assertions)]
impl<'a> Visit<'a> for Invariants<'_, 'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }
    fn enter_node(&mut self, node: NodeKindData) {
        self.reachable[node.node_id().index()] = true;
        self.path.push(node);
    }
    fn leave_node(&mut self, _: NodeKindData) {
        self.path.pop();
    }
    fn visit_binding_identifier(&mut self, node: BindingIdentifier) {
        self.reachable[node.node_id().index()] = true;
        let Some(symbol) = self.semantic.symbol_of(node.node_id()) else {
            // The binder treats signature-type parameter names as labels,
            // rather than lexical declarations (see enter_signature_type).
            assert!(
                self.path.iter().any(|node| matches!(
                    node,
                    NodeKindData::TsFunctionType(_)
                        | NodeKindData::TsConstructorType(_)
                        | NodeKindData::TsMethodSignature(_)
                        | NodeKindData::TsCallSignatureDeclaration(_)
                        | NodeKindData::TsConstructSignatureDeclaration(_)
                        | NodeKindData::TsIndexSignature(_)
                )),
                "reachable binding must have a symbol"
            );
            node.visit_children_with(self);
            return;
        };
        // Default exports and merged TS declarations can use a canonical
        // symbol name that differs from this binding occurrence.
        assert!(
            self.semantic
                .declarations(symbol)
                .any(|declaration| declaration.node == node.node_id())
        );
        node.visit_children_with(self);
    }
    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        self.reachable[node.node_id().index()] = true;
        let reference = self
            .semantic
            .reference_of(node.node_id())
            .expect("reachable identifier must have a reference");
        let reference = self.semantic.reference(reference);
        assert_eq!(reference.node, node.node_id());
        assert_eq!(
            self.ast.get_utf8(reference.name),
            self.ast.get_utf8(node.name(self.ast))
        );
    }
}
