use swc_next_ecma_ast::{
    AssignmentOperator, AssignmentTarget, Ast, AstUtf8, BindingIdentifier, DUMMY_SP, Expr,
    IdentifierReference, NodeId, NodeKindData, ReferenceId, ScopeId, SimpleAssignmentTarget, Span,
    SymbolId,
};
use swc_next_ecma_semantic::{
    Reference, ReferenceFlags, SymbolFlags,
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
        let symbol = ctx.register_binding(scope, identifier, flags);
        BoundIdentifier { identifier, symbol }
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
        ctx.register_reference(scope, node, Some(self.symbol), ReferenceFlags::default());
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
        ctx.register_reference(
            scope,
            node,
            Some(self.symbol),
            ReferenceFlags::default().write_only(),
        );
        node
    }

    /// Build an assignment using a fresh write-only reference to this binding.
    pub fn assign(
        &self,
        ctx: &mut Mutation<'_, '_, '_>,
        scope: ScopeId,
        span: Span,
        value: Expr,
    ) -> Expr {
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(self.reference_write(ctx, scope, span)),
        );
        Expr::AssignmentExpression(ctx.add_node(scope, |ast| {
            ast.assignment_expression(span, target, value, AssignmentOperator::Assign)
        }))
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
    /// Read a binding's AST name and register all pending allocations before
    /// passing the declaration data to ScopeModel's index-only operation.
    pub fn register_binding(
        &mut self,
        scope: ScopeId,
        binding: BindingIdentifier,
        flags: SymbolFlags,
    ) -> SymbolId {
        let name = binding.name(self.ast);
        let text = self.ast.get_utf8(name);
        self.state.semantic.grow_to(self.ast.node_count(), scope);
        self.state
            .semantic
            .register_binding(scope, binding.node_id(), name, text, flags)
    }

    /// Read a reference's AST name and register allocations before passing its
    /// semantic record to ScopeModel. The caller supplies its resolved target.
    pub fn register_reference(
        &mut self,
        scope: ScopeId,
        reference: IdentifierReference,
        target: Option<SymbolId>,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        let reference = Reference {
            name: reference.name(self.ast),
            node: reference.node_id(),
            scope,
            symbol: target,
            flags,
        };
        self.state.semantic.grow_to(self.ast.node_count(), scope);
        self.state.semantic.register_reference(reference)
    }

    /// Allocate nodes in their final evaluation scope and register only the new
    /// NodeId range. The closure must only create nodes; existing AST edges are
    /// edited through mutation APIs. Bindings, references and scope owners use
    /// their dedicated construction methods.
    pub fn add_node<N>(&mut self, scope: ScopeId, f: impl FnOnce(&mut Ast<'a>) -> N) -> N {
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
        self.state.semantic.grow_to(self.ast.node_count(), parent);
        self.state
            .semantic
            .create_child_scope(parent, owner, kind, flags)
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
        self.state.semantic.grow_to(self.ast.node_count(), scope);
        self.state.semantic.convert_scope(scope, owner, kind, flags);
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
    #[expect(dead_code, reason = "Planned binding relocation API")]
    pub fn move_binding(&mut self, from: ScopeId, to: ScopeId, symbol: SymbolId) {
        self.state.semantic.move_binding(from, to, symbol);
    }

    /// Change an active non-root scope's parent to another active scope.
    ///
    /// Reject cycles. Preserve owner, bindings and internal node/reference mappings;
    /// delegate tree/hoist-target changes to ScopeModel and coordinate any
    /// syntax-dependent strictness/direct-eval changes here. Function and
    /// static-block scopes remain their own hoist targets.
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
        self.state.semantic.remove_scope(scope);
    }

    /// Clear the evaluation-scope mapping of a detached syntax shell only.
    ///
    /// Require that the node owns no active scope, binding or reference. Do not
    /// visit its children: they may already be retained elsewhere in the AST.
    /// Example: the discarded ExpressionStatement after extracting its expression.
    pub fn clear_node_scope(&mut self, node: NodeId) {
        self.state.semantic.clear_node_scope(node);
    }
}
