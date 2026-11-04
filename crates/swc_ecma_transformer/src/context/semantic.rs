use swc_next_ecma_ast::{
    AssignmentOperator, AssignmentTarget, Ast, AstUtf8, BindingIdentifier, DUMMY_SP, Expr,
    IdentifierReference, NodeId, NodeKindData, ReferenceId, ScopeId, SimpleAssignmentTarget, Span,
    SymbolId, Traverse, TraverseCtx, TraverseWith,
};
use swc_next_ecma_semantic::{
    Reference, ReferenceFlags, ScopeModel, SymbolFlags,
    scope::{ScopeFlags, ScopeKind},
};

use crate::{context::TransformCtx, used_names::UsedNameState};

/// A generated binding keeps its declaration and symbol together
#[derive(Clone, Copy)]
pub struct BoundIdentifier {
    pub identifier: BindingIdentifier,
    pub symbol: SymbolId,
}

impl BoundIdentifier {
    /// Allocate a collision-free binding identifier and register its declaration and symbol.
    ///
    /// ```javascript
    /// // Before: a generated temporary would conflict with _temp.
    /// var _temp;
    /// // After: the caller can attach a declaration using the returned binding identity.
    /// var _temp;
    /// var _temp2;
    /// // Reserve _temp2, allocate its BindingIdentifier and register it in the requested scope.
    /// // Return the identifier and SymbolId together; declaration attachment happens separately.
    /// ```
    pub fn named(
        ctx: &mut TransformCtx<'_, '_>,
        scope: ScopeId,
        base: &str,
        flags: SymbolFlags,
    ) -> BoundIdentifier {
        let name = ctx.generate_name(base, 2);
        let identifier = ctx.ast.binding_identifier(DUMMY_SP, name);
        let symbol = ctx.register_binding(scope, identifier, flags);
        BoundIdentifier { identifier, symbol }
    }

    /// Allocate a fresh read reference to this binding in the requested evaluation scope.
    ///
    /// ```javascript
    /// // Before: one generated binding exists.
    /// var _temp;
    /// // After: each use needs a distinct reference node, not a cloned declaration.
    /// var _temp;
    /// use(_temp, _temp);
    /// // Each call allocates a fresh IdentifierReference and ReferenceId targeting the same SymbolId.
    /// ```
    pub fn reference_read(
        &self,
        ctx: &mut TransformCtx<'_, '_>,
        scope: ScopeId,
        span: Span,
    ) -> IdentifierReference {
        let node = ctx
            .ast
            .identifier_reference(span, self.identifier.name(ctx.ast));
        ctx.register_reference(scope, node, Some(self.symbol), ReferenceFlags::default());
        node
    }

    /// Allocate a fresh write-only reference to this binding for an assignment target.
    ///
    /// ```javascript
    /// // Before: the generated binding has no assignment reference.
    /// var _temp;
    /// // After: the assignment builder needs a write target, not a read or declaration.
    /// var _temp;
    /// _temp = 1;
    /// // Allocate the target's IdentifierReference with write-only flags and the existing SymbolId.
    /// // The caller attaches that reference to the assignment separately.
    /// ```
    pub fn reference_write(
        &self,
        ctx: &mut TransformCtx<'_, '_>,
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

    /// Build an assignment to this binding with a fresh write-only target reference.
    ///
    /// ```javascript
    /// // Before: initialize() is an existing expression in the destination scope.
    /// initialize();
    /// // After: the caller retains the expression as the assignment's right-hand side.
    /// _temp = initialize();
    /// // Create a fresh write reference to the registered _temp binding and allocate the assignment.
    /// // Retain initialize() and its semantic identities; attach the prepared expression separately.
    /// ```
    pub fn assign(
        &self,
        ctx: &mut TransformCtx<'_, '_>,
        scope: ScopeId,
        span: Span,
        value: Expr,
    ) -> Expr {
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(self.reference_write(ctx, scope, span)),
        );
        Expr::AssignmentExpression(ctx.add_node(|ast| {
            ast.assignment_expression(span, target, value, AssignmentOperator::Assign)
        }))
    }
}

impl TransformCtx<'_, '_> {
    /// Reserve and intern an available name in the lazily initialized shared transform collision domain.
    ///
    /// ```javascript
    /// // Before: both a declaration and an unresolved reference reserve names.
    /// var _temp;
    /// use(_temp2);
    /// // After: a generated declaration can use the first available suffix.
    /// var _temp;
    /// var _temp3;
    /// use(_temp2);
    /// // On first use, observe symbols, binding declaration spellings and all reference names.
    /// // Reserve _temp3 immediately so another allocation cannot reuse it; no binding is registered yet.
    /// ```
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

impl<'a> TransformCtx<'_, 'a> {
    /// Read a binding's AST name and register pending allocations before registering its declaration and symbol.
    ///
    /// ```javascript
    /// // Before: lowering needs a temporary but its declaration is not indexed.
    /// run();
    /// // After: the caller attaches a prepared var declaration.
    /// var _temp;
    /// run();
    /// // Intern _temp's name, then grow indexes through all AST allocations.
    /// // Register the binding in its destination scope with var flags; retain its returned SymbolId.
    /// ```
    pub fn register_binding(
        &mut self,
        scope: ScopeId,
        binding: BindingIdentifier,
        flags: SymbolFlags,
    ) -> SymbolId {
        let name = self.ast.intern_utf8(binding.name(self.ast));
        self.state.semantic.grow_to(self.ast.node_count());
        self.state
            .semantic
            .register_binding(scope, binding.node_id(), name, flags)
    }

    /// Read an identifier's AST name and register a caller-resolved reference after registering pending allocations.
    ///
    /// ```javascript
    /// // Before: the temporary binding is registered.
    /// var _temp;
    /// // After: lowering prepares a write reference for an assignment.
    /// var _temp;
    /// _temp = initialize();
    /// // Read the new target node's name, grow pending allocations and build its Reference record.
    /// // Supply the destination scope, existing _temp SymbolId and write-only flags; attach it separately.
    /// ```
    pub fn register_reference(
        &mut self,
        scope: ScopeId,
        reference: IdentifierReference,
        target: Option<SymbolId>,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        let reference = Reference {
            name: self.ast.intern_utf8(reference.name(self.ast)),
            node: reference.node_id(),
            scope,
            symbol: target,
            flags,
        };
        self.state.semantic.grow_to(self.ast.node_count());
        self.state.semantic.register_reference(reference)
    }

    /// Allocate new AST nodes and grow their identity indexes without changing existing edges.
    ///
    /// ```javascript
    /// // Before: initialize() is already registered.
    /// initialize();
    /// // After: lowering prepares a unary wrapper in the same scope.
    /// void initialize();
    /// // Allocate only the UnaryExpression and grow indexes through its new NodeId.
    /// // Retain the existing call; attach the wrapper separately and register new semantic identities explicitly.
    /// ```
    pub fn add_node<N>(&mut self, f: impl FnOnce(&mut Ast<'a>) -> N) -> N {
        let node = f(self.ast);
        self.state.semantic.grow_to(self.ast.node_count());
        node
    }

    /// Register a fresh scope owner below an active parent with the supplied kind and flags.
    ///
    /// The caller must compute strictness from the parent context and the new scope's own directives.
    ///
    /// ```javascript
    /// // Before: run() executes in the surrounding scope.
    /// run();
    /// // After: the caller adds an arrow, which needs its own Function scope.
    /// (() => run())();
    /// // Register pending AST allocations, then register the arrow owner with the supplied flags.
    /// // Move existing children into it separately; the owner must not own a scope yet.
    /// ```
    pub fn create_child_scope(
        &mut self,
        parent: ScopeId,
        owner: NodeId,
        kind: ScopeKind,
        flags: ScopeFlags,
    ) -> ScopeId {
        self.state.semantic.grow_to(self.ast.node_count());
        self.state
            .semantic
            .create_child_scope(parent, owner, kind, flags)
    }

    /// Transfer a scope to a new owner and kind while preserving its contents and identities.
    ///
    /// The caller must maintain strictness inheritance and update affected descendant flags separately.
    /// The current owner must own only this scope; a replacement owner must not own another scope.
    ///
    /// ```javascript
    /// // Before: value and its uses belong to the static-block scope.
    /// class C { static { let value = 1; use(value); } }
    /// // After: reuse that scope for the generated arrow instead of rebuilding bindings.
    /// class C { static #_ = (() => { let value = 1; use(value); })(); }
    /// // Register pending AST allocations, replace the StaticBlock owner and change kind to Function.
    /// // Keep ScopeId, parent, bindings, references and child scopes; refresh hoist/body links.
    /// // The caller supplies arrow flags, including arguments_barrier = false.
    /// ```
    pub fn convert_scope(
        &mut self,
        scope: ScopeId,
        owner: NodeId,
        kind: ScopeKind,
        flags: ScopeFlags,
    ) {
        self.state.semantic.grow_to(self.ast.node_count());
        self.state.semantic.convert_scope(scope, owner, kind, flags);
    }

    /// Move a retained subtree's evaluations between scopes and reparent child scopes while preserving identities.
    ///
    /// Scope flags stay unchanged; the caller must maintain strictness inheritance in affected scopes.
    ///
    /// ```javascript
    /// // Before: use and value execute in a static-block scope; the arrow owns a nested scope.
    /// class C { static { use(value, () => value); } }
    /// // After: the caller reuses the whole call in the class scope.
    /// class C { static #_ = use(value, () => value); }
    /// // Move references directly in the block scope to the class scope, preserving their identities.
    /// // Reparent the nested arrow scope to the class scope without moving its internal nodes/bindings.
    /// // Move owned bindings separately, then attach through a setter or container.
    /// // Pass the typed expression handle to reuse Traverse; NodeId alone has no traversal implementation.
    /// ```
    pub fn move_subtree_to_scope(
        &mut self,
        root: impl TraverseWith<'a>,
        from: ScopeId,
        to: ScopeId,
    ) {
        debug_assert!(self.semantic().scope(from).active && self.semantic().scope(to).active);
        if from == to {
            return;
        }
        let mut mover = ScopeMover {
            semantic: &mut self.state.semantic,
            from,
            to,
        };
        root.traverse_with(&mut mover, &mut TraverseCtx::new(self.ast, ()));
    }

    /// Change an active non-root scope's parent while preserving its owner and internal identities.
    ///
    /// Scope flags stay unchanged; the caller must maintain strictness inheritance in affected scopes.
    ///
    /// ```javascript
    /// // Before: the inner block is a child of f's Function scope.
    /// function f() { { let value = 1; use(value); } }
    /// // After: a generated arrow encloses the same inner block.
    /// function f() { (() => { { let value = 1; use(value); } })(); }
    /// // Reparent the block scope from f to the arrow without changing its contents.
    /// // Reject cycles and refresh descendant hoist targets; functions keep their own targets.
    /// ```
    pub fn reparent_scope(&mut self, scope: ScopeId, parent: ScopeId) {
        self.state.semantic.reparent_scope(scope, parent);
    }

    /// Retire an empty non-root scope and clear its detached owner's mapping without recycling its ID.
    ///
    /// ```javascript
    /// // Before: an empty static block still owns a scope.
    /// class C { static {} }
    /// // After: the caller removes the block from the class body.
    /// class C {}
    /// // Require no remaining bindings, declarations, references or child scopes.
    /// // Clear the block owner's mapping and mark its scope inactive; do not delete descendants.
    /// // For a reused block body, convert_scope keeps the scope instead.
    /// ```
    pub fn remove_scope(&mut self, scope: ScopeId) {
        self.state.semantic.remove_scope(scope);
    }
}

struct ScopeMover<'c, 'a> {
    semantic: &'c mut ScopeModel<'a>,
    from: ScopeId,
    to: ScopeId,
}

impl<'a> Traverse<'a> for ScopeMover<'_, 'a> {
    fn enter_node(&mut self, node: NodeId, _ctx: &mut TraverseCtx<'_, 'a>) {
        if let Some(scope) = self.semantic.scope_of(node) {
            // Program scopes keep their fixed outer wrapper.
            if self.semantic.scope(scope).parent == Some(self.from)
                && self.semantic.scope(self.from).node != node
            {
                self.semantic.reparent_scope(scope, self.to);
            }
        }
        if let Some(reference) = self.semantic.reference_of(node)
            && self.semantic.reference(reference).scope == self.from
        {
            self.semantic
                .move_reference_to_scope(reference, self.from, self.to);
        }
    }
}
