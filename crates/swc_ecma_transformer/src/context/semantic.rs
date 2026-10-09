use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingIdentifier, DUMMY_SP, IdentifierReference, NodeId, NodeKindData, ScopeId,
    Span, SymbolId,
};
use swc_next_ecma_semantic::{
    ReferenceFlags, SymbolFlags,
    scope::{ScopeFlags, ScopeKind},
};

use crate::{context::Mutation, used_names::UsedNameState};

/// A generated binding keeps its declaration and symbol together
#[derive(Clone, Copy)]
pub struct BoundIdentifier {
    pub identifier: BindingIdentifier,
    pub symbol: SymbolId,
}

impl BoundIdentifier {
    pub fn named(
        ctx: &mut Mutation<'_, '_, '_>,
        scope: ScopeId,
        base: &str,
        flags: SymbolFlags,
    ) -> BoundIdentifier {
        let name = ctx.generate_name(base, 2);
        let identifier = ctx.ast.binding_identifier(DUMMY_SP, name);
        let symbol = ctx
            .state
            .semantic
            .register_binding(ctx.ast, scope, identifier, flags);
        let binding = BoundIdentifier { identifier, symbol };
        binding
    }

    /// Create a fresh read-only reference to this binding.
    pub fn reference_read(
        &self,
        ctx: &mut Mutation<'_, '_, '_>,
        scope: ScopeId,
        span: Span,
    ) -> IdentifierReference {
        let node = ctx
            .ast
            .identifier_reference(span, self.identifier.name(ctx.ast));
        ctx.state.semantic.register_reference(
            ctx.ast,
            scope,
            node,
            Some(self.symbol),
            ReferenceFlags::default(),
        );
        node
    }

    /// Create a fresh write-only reference for an assignment target.
    pub fn reference_write(
        &self,
        ctx: &mut Mutation<'_, '_, '_>,
        scope: ScopeId,
        span: Span,
    ) -> IdentifierReference {
        let node = ctx
            .ast
            .identifier_reference(span, self.identifier.name(ctx.ast));
        ctx.state.semantic.register_reference(
            ctx.ast,
            scope,
            node,
            Some(self.symbol),
            ReferenceFlags::default().write_only(),
        );
        node
    }
}

impl Mutation<'_, '_, '_> {
    /// Reserves and interns a name in the shared transform collision domain.
    /// Collect existing names once, when a transform first needs a fresh name.
    pub fn generate_name(&mut self, base: &str, start_suffix: u64) -> AstUtf8 {
        let names = self.state.used_names.get_or_insert_with(|| {
            let mut names = UsedNameState::new(self.ast.allocator());
            for entry in self.state.semantic.iter_symbols() {
                names.observe(self.ast.get_wtf8(entry.symbol.name).as_bytes());
            }
            // Merged TypeScript declarations can have different source names
            // while sharing a canonical symbol, notably default exports.
            for entry in self.state.semantic.iter_declarations() {
                if let NodeKindData::BindingIdentifier(identifier) =
                    self.ast.kind_data(entry.declaration.node)
                {
                    names.observe(self.ast.get_utf8(identifier.name(self.ast)).as_bytes());
                }
            }
            for entry in self.state.semantic.iter_references() {
                names.observe(self.ast.get_utf8(entry.reference.name).as_bytes());
            }
            names
        });
        self.ast.add_utf8(names.next_name(base, start_suffix))
    }
}

impl<'a> Mutation<'_, '_, 'a> {
    /// Allocate nodes in their final evaluation scope and register only the new
    /// NodeId range. The closure must only create nodes; existing AST edges are
    /// edited through mutation APIs. Bindings, references and scope owners use
    /// their dedicated construction methods.
    pub(crate) fn add_node<N>(&mut self, scope: ScopeId, f: impl FnOnce(&mut Ast<'a>) -> N) -> N {
        debug_assert!(
            self.state.semantic.scope(scope).active,
            "target scope must be active"
        );
        let node = f(self.ast);
        self.state.semantic.grow_to(self.ast.node_count(), scope);
        node
    }

    /// Register a scope for an already allocated owner below an active parent.
    ///
    /// Register the owner's node mapping, inherit strictness, and initialize the
    /// hoist target and flags according to `kind`. The owner must not already own
    /// a scope. Existing children are moved separately with `move_subtree_to_scope`;
    /// newly allocated parameters/body nodes use `add_node` in the returned scope.
    pub fn create_child_scope(
        &mut self,
        parent: ScopeId,
        owner: NodeId,
        kind: ScopeKind,
        flags: ScopeFlags,
    ) -> ScopeId {
        todo!("Register {kind:?} scope for {owner:?} below {parent:?} with {flags:?}")
    }

    /// Transfer a scope to a replacement owner and change its kind/flags.
    ///
    /// Preserve ScopeId, parent, bindings, declaration/reference identities and
    /// child scopes. Replace the old owner's mapping with the new owner's mapping;
    /// retained content stays in this scope. Refresh affected hoist targets,
    /// function-body linkage and flags instead of copying the old scope unchanged.
    /// Example: a StaticBlock scope becomes an arrow's Function scope.
    pub fn convert_scope(
        &mut self,
        scope: ScopeId,
        owner: NodeId,
        kind: ScopeKind,
        flags: ScopeFlags,
    ) {
        todo!("Convert {scope:?} to {kind:?} owned by {owner:?} with {flags:?}")
    }

    /// Move the evaluation of a retained subtree between active scopes.
    ///
    /// Update node mappings and Reference.scope only for nodes directly in `from`.
    /// Reparent direct child scopes contained in the subtree to `to`, preserving
    /// their internal mappings and bindings. Preserve reference IDs, targets and
    /// use lists; maintain affected direct-eval flags along the new ancestry.
    /// This operation does not move bindings: callers use `move_binding` for local
    /// declarations whose ownership changes. It prepares semantic information;
    /// the caller attaches the subtree through a container, setter or action.
    pub fn move_subtree_to_scope(&mut self, root: NodeId, from: ScopeId, to: ScopeId) {
        todo!("Move evaluation of subtree {root:?} from {from:?} to {to:?}")
    }

    /// Move an existing binding between scopes without allocating a new symbol.
    ///
    /// Move the binding-table entry and declarations belonging to `from`; preserve
    /// merged declarations in other scopes, declaration IDs, reference targets and
    /// use lists. Require that `to` has no conflicting binding. Node evaluation
    /// scopes are updated separately by `move_subtree_to_scope`, since declaration
    /// ownership (e.g. a var hoist target) can differ from its evaluation scope.
    pub fn move_binding(&mut self, from: ScopeId, to: ScopeId, symbol: SymbolId) {
        todo!("Move binding {symbol:?} from {from:?} to {to:?}")
    }

    /// Change an active non-root scope's parent to another active scope.
    ///
    /// Reject cycles. Preserve owner, bindings and internal node/reference mappings;
    /// refresh affected hoist targets and inherited flags in descendants, and
    /// direct-eval flags along the old/new ancestry. Function and static-block
    /// scopes remain their own hoist targets.
    pub fn reparent_scope(&mut self, scope: ScopeId, parent: ScopeId) {
        todo!("Reparent {scope:?} below {parent:?}")
    }

    /// Retire an empty non-root scope after its retained contents have moved.
    ///
    /// Require no remaining bindings, declarations, references, child scopes or
    /// retained node mappings. Clear the detached owner's mapping and mark the
    /// scope inactive without recycling its ID. Reused scopes use `convert_scope`
    /// instead; this operation does not recursively delete a subtree.
    pub fn remove_scope(&mut self, scope: ScopeId) {
        todo!("Retire empty scope {scope:?}")
    }

    /// Clear the evaluation-scope mapping of a detached syntax shell only.
    ///
    /// Require that the node owns no active scope, binding or reference. Do not
    /// visit its children: they may already be retained elsewhere in the AST.
    /// Example: the discarded ExpressionStatement after extracting its expression.
    pub fn clear_node_scope(&mut self, node: NodeId) {
        todo!("Clear scope mapping for detached shell {node:?}")
    }
}
