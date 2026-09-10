//! Scope ownership and lexical environments for constructed or relocated nodes.
use super::Mutation;
use swc_next_ecma_ast::{Ast, AstNode, FormalParameterKind, FormalParameters, NodeId, ScopeId};
use swc_next_ecma_semantic::{ClassArrowInfo, scope::ScopeKind};

pub(crate) struct ScopeOptions {
    pub parent: ScopeId,
    pub kind: ScopeKind,
    /// Transfer this existing scope to the new owner instead of allocating one.
    pub reuse: Option<ScopeId>,
    pub environment: Option<ClassArrowInfo>,
}

impl<'a> Mutation<'_, '_, 'a> {
    pub(crate) fn scope_for_node(&self, owner: NodeId) -> Option<ScopeId> {
        self.state.semantic.scope_at(owner).filter(|&id| {
            let scope = self.state.semantic.scope(id);
            scope.active && scope.node == owner
        })
    }

    /// Construct a scope owner and register or transfer its scope before returning.
    /// The caller chooses the node shape and any inherited class environment.
    pub(crate) fn add_scope_node<N: AstNode + Copy>(
        &mut self,
        options: ScopeOptions,
        create: impl FnOnce(&mut Ast<'a>) -> N,
    ) -> (N, ScopeId) {
        assert!(
            self.state.semantic.scope(options.parent).active,
            "parent scope must be active"
        );
        if let Some(scope) = options.reuse {
            let mut ancestor = Some(options.parent);
            while let Some(id) = ancestor {
                assert_ne!(
                    scope, id,
                    "scope destination cannot be inside the reused scope"
                );
                ancestor = self.state.semantic.scope(id).parent;
            }
            assert!(
                self.state.semantic.scope(scope).active,
                "reused scope must be active"
            );
        }
        let start = self.ast.node_count();
        let node = create(self.ast);
        self.state.semantic.grow_node_indexes(self.ast.node_count());
        let scope = if let Some(scope) = options.reuse {
            let old = self.state.semantic.scope(scope).node;
            self.state
                .semantic
                .replace_scope_owner(self.ast, old, node.node_id(), options.kind);
            self.state.semantic.reparent_scope(scope, options.parent);
            scope
        } else {
            self.state.semantic.create_child_scope(
                self.ast,
                node.node_id(),
                options.parent,
                options.kind,
            )
        };
        self.state.semantic.register_nodes(self.ast, start, scope);
        if let Some(environment) = options.environment {
            self.state
                .semantic
                .record_arrow_environment(node.node_id(), environment);
        }
        (node, scope)
    }

    /// Merge a binding-free scope into the target while retaining reference IDs
    /// and updating arrows that inherit its evaluation environment.
    pub(crate) fn merge_scope(
        &mut self,
        owner: NodeId,
        parent: ScopeId,
        environment: ClassArrowInfo,
    ) {
        let scope = self
            .scope_for_node(owner)
            .expect("source owner must have an active scope");
        self.state.semantic.merge_scope(scope, parent, environment);
    }

    pub(crate) fn set_parameter_kind(
        &mut self,
        params: FormalParameters,
        kind: FormalParameterKind,
    ) {
        params.set_kind(self.ast, kind);
    }
}
