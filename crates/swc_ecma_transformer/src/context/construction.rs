//! Node allocation and construction that registers bindings, references and scopes.
use super::Mutation;
use swc_next_ecma_ast::{
    Ast, AstUtf8, AstWtf8, DUMMY_SP, ExtraDataValue, IdentifierReference, ScopeId, SubRange,
};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace};

impl<'a> Mutation<'_, '_, 'a> {
    /// Allocate nodes in their final evaluation scope and register only the new
    /// NodeId range. The closure must only create nodes; existing AST edges are
    /// edited through mutation APIs. Bindings, references and scope owners use
    /// their dedicated construction methods.
    pub(crate) fn add_node<N>(
        &mut self,
        scope: ScopeId,
        create: impl FnOnce(&mut Ast<'a>) -> N,
    ) -> N {
        assert!(
            self.state.semantic.scope(scope).active,
            "target scope must be active"
        );
        let start = self.ast.node_count();
        let node = create(self.ast);
        self.state.semantic.register_nodes(self.ast, start, scope);
        node
    }

    pub(crate) fn intern_utf8(&mut self, value: &str) -> AstUtf8 {
        self.ast.add_utf8(value)
    }

    pub(crate) fn create_named_reference(
        &mut self,
        name: AstUtf8,
        scope: ScopeId,
        flags: ReferenceFlags,
    ) -> IdentifierReference {
        assert!(
            self.state.semantic.scope(scope).active,
            "reference scope must be active"
        );
        let symbol = self.state.semantic.lookup(
            scope,
            self.ast.get_utf8(name).as_bytes(),
            ReferenceSpace::Value,
        );
        let node = self.ast.identifier_reference(DUMMY_SP, name);
        self.state
            .semantic
            .register_reference(self.ast, node, scope, symbol, flags);
        node
    }

    pub(crate) fn add_sub_range<T: ExtraDataValue>(
        &mut self,
        nodes: impl IntoIterator<Item = T>,
    ) -> SubRange<T> {
        self.ast.add_sub_range(nodes)
    }

    pub(crate) fn intern_wtf8(&mut self, value: &str) -> AstWtf8 {
        self.ast.add_wtf8(value.into())
    }
}
