//! Debug checks for semantic data after AST transformation.

use swc_next_ecma_ast::{
    Ast, BindingIdentifier, IdentifierReference, NodeId, NodeKindData, Visit, VisitWith,
};

use swc_next_ecma_semantic::Semantic;

/// Checks the semantic indexes of the final reachable AST.
pub(super) fn assert_semantic_invariants(semantic: &Semantic<'_>, ast: &Ast<'_>) {
    let mut visitor = Invariants {
        ast,
        semantic,
        reachable: vec![false; ast.node_count()],
        path: Vec::new(),
    };
    ast.root_program().visit_with(&mut visitor);
    let reachable = visitor.reachable;
    for (index, &is_reachable) in reachable.iter().enumerate() {
        // Detached structural nodes remain in the arena until compilation
        // ends. Only reachable nodes need a current evaluation scope.
        if is_reachable {
            let scope = semantic
                .scope_at(NodeId::from_usize(index))
                .expect("reachable node must have a scope");
            assert!(semantic.scope(scope).active);
        }
    }

    let scope_count = semantic.iter_scopes().count();
    for entry in semantic.iter_scopes() {
        assert!(
            reachable[entry.scope.node.index()],
            "scope owner must be reachable"
        );
        let mut parent = entry.scope.parent;
        let mut depth = 0;
        while let Some(id) = parent {
            assert!(semantic.scope(id).active);
            depth += 1;
            assert!(depth < scope_count, "scope parent chain must be acyclic");
            parent = semantic.scope(id).parent;
        }
        assert!(semantic.scope(entry.scope.hoist_target).active);
    }
    for entry in semantic.iter_symbols() {
        assert!(semantic.scope(semantic.scope_of(entry.id)).active);
        let declaration = semantic.declaration(entry.symbol.declaration);
        assert!(
            reachable[declaration.node.index()],
            "symbol declaration must be reachable"
        );
        assert!(semantic.symbol_of(declaration.node).is_some());
    }
    for entry in semantic.iter_references() {
        assert!(reachable[entry.reference.node.index()]);
        assert!(semantic.scope(entry.reference.scope).active);
        if let Some(symbol) = entry.reference.symbol {
            assert!(semantic.uses(symbol).contains(&entry.id));
        }
    }
}

struct Invariants<'v, 'a> {
    ast: &'v Ast<'a>,
    semantic: &'v Semantic<'a>,
    reachable: Vec<bool>,
    path: Vec<NodeKindData>,
}

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
