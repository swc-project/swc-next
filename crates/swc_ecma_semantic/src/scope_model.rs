//! Scope, binding, declaration, and reference indexes for transforms.
//!
//! The model consumes Semantic's basic indexes without retaining its hoisting
//! conflict tables, module record, or TypeScript member and alias lookup indexes.
//! Index mutations take semantic records and identifiers, not an AST. Transforms
//! read syntax and coordinate subtree edits through Mutation.

use oxc_index::Idx;
use swc_next_allocator::vec::Vec as ArenaVec;
use swc_next_ecma_ast::{AstUtf8, NodeId, ReferenceId, ScopeId, SymbolId};

use crate::{
    Declaration, DeclarationId, DeclarationIdIterator, DeclarationIterator, Redeclarations,
    Reference, ReferenceIterator, ScopeIterator, ScopeMap, Semantic, Symbol, SymbolFlags,
    SymbolIterator,
    binder::find_expression_name,
    scope::{Scope, ScopeFlags, ScopeKind, ScopeTree},
};

/// The basic semantic indexes needed by transforms.
///
/// Scope, symbol, declaration, reference, and node identities are preserved.
/// This model maintains basic indexes without Semantic's TypeScript
/// name-resolution queries or AST traversal.
#[derive(Debug)]
pub struct ScopeModel<'a> {
    scopes: ScopeTree<'a>,
    symbols: ArenaVec<'a, Symbol>,
    references: ArenaVec<'a, Reference>,
    declarations: ArenaVec<'a, Declaration>,
    redeclarations: Redeclarations<'a>,
    /// Independently growable use lists, indexed by SymbolId.
    uses: ArenaVec<'a, ArenaVec<'a, ReferenceId>>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    node_symbols: ArenaVec<'a, Option<SymbolId>>,
    node_references: ArenaVec<'a, Option<ReferenceId>>,
    node_scopes: ArenaVec<'a, Option<ScopeId>>,
}

impl<'a> ScopeModel<'a> {
    /// Consumes Semantic, moving its basic indexes and expanding its compact
    /// use index into one arena vector per symbol.
    pub fn new(semantic: Semantic<'a>) -> Self {
        let allocator = semantic.use_ids.bump();
        let uses = ArenaVec::from_iter_in(
            semantic.use_ranges.iter().map(|range| {
                let start = range.start as usize;
                ArenaVec::from_iter_in(
                    semantic.use_ids[start..start + range.len as usize]
                        .iter()
                        .copied(),
                    allocator,
                )
            }),
            allocator,
        );

        Self {
            scopes: semantic.scopes,
            symbols: semantic.symbols,
            references: semantic.references,
            declarations: semantic.declarations,
            redeclarations: semantic.redeclarations,
            uses,
            scope_maps: semantic.scope_maps,
            node_symbols: semantic.node_symbols,
            node_references: semantic.node_references,
            node_scopes: semantic.node_scopes,
        }
    }

    /// The evaluation scope recorded for a node.
    pub fn scope_at(&self, node: NodeId) -> Option<ScopeId> {
        self.node_scopes.get(node.index()).copied().flatten()
    }

    /// The evaluation scope outside a scope-owning expression.
    pub fn scope_outside(&self, node: NodeId) -> Option<ScopeId> {
        let mut scope = self.scope_at(node)?;
        while self.scope(scope).node == node {
            scope = self.scope(scope).parent?;
        }
        Some(scope)
    }

    pub fn scope(&self, id: ScopeId) -> Scope {
        self.scopes.get(id)
    }

    pub fn symbol(&self, id: SymbolId) -> Symbol {
        self.symbols[id.index()]
    }

    pub fn reference(&self, id: ReferenceId) -> Reference {
        self.references[id.index()]
    }

    pub fn declaration(&self, id: DeclarationId) -> Declaration {
        self.declarations[id.index()]
    }

    pub fn symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        self.node_symbols[node.index()]
    }

    pub fn reference_of(&self, node: NodeId) -> Option<ReferenceId> {
        self.node_references[node.index()]
    }

    /// The scope of the symbol's first declaration. Merged TypeScript symbols
    /// may have additional declarations in other scopes.
    pub fn scope_of(&self, id: SymbolId) -> ScopeId {
        self.declaration(self.symbol(id).declaration).scope
    }

    pub fn declaration_ids(&self, id: SymbolId) -> DeclarationIdIterator<'_> {
        let additional = self
            .redeclarations
            .get(&id)
            .map_or(&[][..], |declarations| declarations.as_slice());
        DeclarationIdIterator {
            first: Some(self.symbol(id).declaration),
            additional: additional.iter().copied(),
        }
    }

    pub fn declarations(
        &self,
        id: SymbolId,
    ) -> impl ExactSizeIterator<Item = Declaration> + DoubleEndedIterator + '_ {
        self.declaration_ids(id).map(|id| self.declaration(id))
    }

    /// Every definite or possible use retained from Semantic, in source order.
    /// An uncertain reference may occur in more than one symbol's list.
    pub fn uses(&self, id: SymbolId) -> &[ReferenceId] {
        &self.uses[id.index()]
    }

    /// A binding owned by this lexical scope. Hoisting declarations are only
    /// recorded at their hoist target; TypeScript member views are not queried.
    pub fn own_binding(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.scope_maps[scope.index()].get(name).copied()
    }

    pub fn bindings(&self, scope: ScopeId) -> impl Iterator<Item = SymbolId> + '_ {
        self.scope_maps[scope.index()].values().copied()
    }

    pub fn iter_scopes(&self) -> ScopeIterator<'_> {
        ScopeIterator {
            list: &self.scopes.list,
            index: 0,
        }
    }

    pub fn iter_symbols(&self) -> SymbolIterator<'_> {
        SymbolIterator {
            symbols: &self.symbols,
            index: 0,
        }
    }

    pub fn iter_references(&self) -> ReferenceIterator<'_> {
        ReferenceIterator {
            references: &self.references,
            node_references: &self.node_references,
            index: 0,
        }
    }

    pub fn iter_declarations(&self) -> DeclarationIterator<'_> {
        DeclarationIterator {
            declarations: &self.declarations,
            index: 0,
        }
    }

    /// Extend node indexes for new allocations in an active scope without changing existing entries.
    ///
    /// ```javascript
    /// // Before: run() and its reference are already indexed.
    /// run();
    /// // After: the caller allocates a unary wrapper in the same evaluation scope.
    /// void run();
    /// // Grow through the new NodeId, set its evaluation scope and leave identity slots empty.
    /// // Keep the existing call/reference entries; register new bindings and references separately.
    /// ```
    pub fn grow_to(&mut self, node_count: usize, scope: ScopeId) {
        assert!(self.scope(scope).active, "target scope must be active");
        // Never truncate indexes when an already registered node is reused.
        if node_count > self.node_scopes.len() {
            self.node_scopes.resize(node_count, Some(scope));
            self.node_symbols.resize(node_count, None);
            self.node_references.resize(node_count, None);
        }
    }

    /// Register a declaration and its scope binding using the caller-supplied name, text and flags.
    ///
    /// ```javascript
    /// // Before: the transform has no temporary binding.
    /// run();
    /// // After: the caller allocates and attaches a temporary declaration.
    /// var _temp;
    /// run();
    /// // Register _temp's node, name handle, arena-lived text and var flags in its hoist scope.
    /// // Reuse the symbol for an existing same-name binding and record the new declaration.
    /// // Grow through this node only; the caller registers other allocations separately.
    /// ```
    pub fn register_binding(
        &mut self,
        scope: ScopeId,
        node: NodeId,
        name: AstUtf8,
        text: &'a str,
        flags: SymbolFlags,
    ) -> SymbolId {
        self.grow_to(node.index() + 1, scope);
        if let Some(symbol) = self.symbol_of(node) {
            return symbol;
        }
        let declaration = DeclarationId::from_usize(self.declarations.len());
        let previous = self.own_binding(scope, text.as_bytes());
        let existing = previous.filter(|&symbol| {
            !self
                .symbol(symbol)
                .flags
                .intersects(SymbolFlags::EXPRESSION_NAME)
        });
        let symbol = if let Some(symbol) = existing {
            self.symbols[symbol.index()].flags |= flags;
            self.redeclarations
                .entry(symbol)
                .or_insert_with(|| ArenaVec::new_in(self.declarations.bump()))
                .push(declaration);
            symbol
        } else {
            let symbol = SymbolId::from_usize(self.symbols.len());
            self.symbols.push(Symbol {
                name: previous.map_or(name.into(), |symbol| self.symbol(symbol).name),
                flags,
                declaration,
            });
            self.uses.push(ArenaVec::new_in(self.symbols.bump()));
            self.scope_maps[scope.index()].insert(text.as_bytes(), symbol);
            symbol
        };
        self.declarations.push(Declaration {
            node,
            symbol,
            scope,
            flags,
        });
        self.node_symbols[node.index()] = Some(symbol);
        self.node_scopes[node.index()] = Some(scope);
        symbol
    }

    /// Register a fresh reference record in the node indexes and its target symbol's use list.
    ///
    /// ```javascript
    /// // Before: _temp's declaration already has a SymbolId.
    /// var _temp;
    /// // After: the caller creates a read of that binding.
    /// var _temp;
    /// _temp;
    /// // Register the read's node, scope, target and flags, then append its ReferenceId to uses.
    /// // An unresolved reference has no target/use entry; registering the same node twice is invalid.
    /// ```
    pub fn register_reference(&mut self, reference: Reference) -> ReferenceId {
        let Reference {
            node,
            scope,
            symbol: target,
            ..
        } = reference;
        self.grow_to(node.index() + 1, scope);
        assert!(
            self.reference_of(node).is_none(),
            "reference already registered"
        );
        let id = ReferenceId::from_usize(self.references.len());
        self.references.push(reference);
        self.node_references[node.index()] = Some(id);
        self.node_scopes[node.index()] = Some(scope);
        if let Some(symbol) = target {
            self.uses[symbol.index()].push(id);
        }
        id
    }
}

impl ScopeModel<'_> {
    /// Register a fresh scope owner below an active parent with the supplied kind and flags.
    ///
    /// The caller must compute strictness from the parent context and the new scope's own directives.
    ///
    /// ```javascript
    /// // Before: run() executes in the surrounding scope.
    /// run();
    /// // After: the caller adds an arrow, which needs its own Function scope.
    /// (() => run())();
    /// // Register the arrow owner below the surrounding scope with the supplied flags.
    /// // Move existing children into it separately; the owner must not own a scope yet.
    /// ```
    pub fn create_child_scope(
        &mut self,
        parent: ScopeId,
        owner: NodeId,
        kind: ScopeKind,
        flags: ScopeFlags,
    ) -> ScopeId {
        let parent_scope = self.scope(parent);
        assert!(parent_scope.active);
        assert!(!self.iter_scopes().any(|entry| entry.scope.node == owner));
        assert!(!matches!(kind, ScopeKind::Global | ScopeKind::Module));
        if kind == ScopeKind::FunctionBody {
            assert_eq!(parent_scope.kind, ScopeKind::Function);
            assert!(parent_scope.function_body.is_none());
        }
        self.grow_to(owner.index() + 1, parent);
        let id = ScopeId::from_usize(self.scopes.list.len());
        self.scopes.list.push(Scope {
            active: true,
            node: owner,
            parent: Some(parent),
            hoist_target: if kind.is_hoist_target() {
                id
            } else {
                parent_scope.hoist_target
            },
            function_body: None,
            kind,
            flags,
        });
        self.scope_maps
            .push(ScopeMap::new_in(self.scopes.list.bump()));
        self.node_scopes[owner.index()] = Some(id);
        if kind == ScopeKind::FunctionBody {
            self.scopes.list[parent.index()].function_body = Some(id);
        }
        id
    }

    /// Transfer a scope to a new owner and kind while preserving its contents and identities.
    ///
    /// The caller must maintain strictness inheritance and update affected descendant flags separately.
    ///
    /// ```javascript
    /// // Before: value and its uses belong to the static-block scope.
    /// class C { static { let value = 1; use(value); } }
    /// // After: reuse that scope for the generated arrow instead of rebuilding bindings.
    /// class C { static #_ = (() => { let value = 1; use(value); })(); }
    /// // Replace the StaticBlock owner with the arrow owner and change kind to Function.
    /// // Keep ScopeId, parent, bindings, references and child scopes; refresh hoist/body links.
    /// // The caller supplies arrow flags, including arguments_barrier = false.
    /// // Owners shared by multiple active scopes, such as Annex B if functions, remain TODO.
    /// ```
    pub fn convert_scope(
        &mut self,
        scope: ScopeId,
        owner: NodeId,
        kind: ScopeKind,
        flags: ScopeFlags,
    ) {
        let original = self.scope(scope);
        assert!(original.active);
        let parent = original.parent.expect("cannot convert a root scope");
        assert!(!matches!(kind, ScopeKind::Global | ScopeKind::Module));
        assert!(kind == ScopeKind::Function || original.function_body.is_none());
        if self
            .iter_scopes()
            .any(|entry| entry.id != scope && entry.scope.node == original.node)
        {
            todo!("Convert an owner shared by multiple active scopes");
        }
        assert!(
            !self
                .iter_scopes()
                .any(|entry| entry.id != scope && entry.scope.node == owner)
        );
        if kind == ScopeKind::FunctionBody {
            assert_eq!(self.scope(parent).kind, ScopeKind::Function);
            assert!(
                self.scope(parent)
                    .function_body
                    .is_none_or(|body| body == scope)
            );
        }
        self.grow_to(owner.index() + 1, scope);
        if original.node != owner {
            assert_eq!(self.scope_at(original.node), Some(scope));
            self.node_scopes[original.node.index()] = None;
        }
        if original.kind == ScopeKind::FunctionBody && kind != ScopeKind::FunctionBody {
            self.scopes.list[parent.index()].function_body = None;
        } else if kind == ScopeKind::FunctionBody {
            self.scopes.list[parent.index()].function_body = Some(scope);
        }
        let current = &mut self.scopes.list[scope.index()];
        current.node = owner;
        current.kind = kind;
        current.flags = flags;
        self.node_scopes[owner.index()] = Some(scope);
        let target = if kind.is_hoist_target() {
            scope
        } else {
            self.scope(parent).hoist_target
        };
        self.update_hoist_target(scope, target);
    }

    /// Move one node's evaluation mapping and reference scope without changing reference identity or target.
    ///
    /// Scope flags stay unchanged; the caller must maintain strictness in the destination context.
    ///
    /// ```javascript
    /// // Before: value's reference executes in the static-block scope.
    /// class C { static { use(value); } }
    /// // After: the caller reuses the call in the class scope.
    /// class C { static #_ = use(value); }
    /// // Move value's node mapping and Reference.scope from the active block scope to the class scope.
    /// // Keep ReferenceId, target, flags and uses; move other nodes separately, without traversing AST.
    /// // Binding ownership is separate, and scope owners use convert_scope or reparent_scope.
    /// ```
    pub fn move_node_to_scope(&mut self, node: NodeId, from: ScopeId, to: ScopeId) {
        assert!(self.scope(from).active && self.scope(to).active);
        assert_eq!(self.scope_at(node), Some(from));
        assert_ne!(
            self.scope(from).node,
            node,
            "scope owners must be moved separately"
        );
        self.node_scopes[node.index()] = Some(to);
        if let Some(reference) = self.reference_of(node) {
            self.references[reference.index()].scope = to;
        }
    }

    /// Move a binding and its source-owned declarations to another scope without changing its symbol.
    ///
    /// ```javascript
    /// // Before: the block owns value's lexical binding.
    /// { let value = 1; use(value); }
    /// // After: the caller moves the declaration and use into an arrow.
    /// (() => { let value = 1; use(value); })();
    /// // Move value's binding-table entry and source-owned declarations to the arrow scope.
    /// // Keep declaration IDs, SymbolId, reference targets, uses and declarations elsewhere.
    /// // The destination must have no conflicting value; move node evaluation scopes separately.
    /// ```
    pub fn move_binding(&mut self, from: ScopeId, to: ScopeId, symbol: SymbolId) {
        assert!(self.scope(from).active && self.scope(to).active);
        if from == to {
            return;
        }
        let names: Vec<_> = self.scope_maps[from.index()]
            .iter()
            .filter_map(|(&name, &binding)| (binding == symbol).then_some(name))
            .collect();
        assert!(!names.is_empty(), "source must own the binding");
        for &name in &names {
            assert!(
                self.own_binding(to, name)
                    .is_none_or(|binding| binding == symbol)
            );
        }
        for name in names {
            if let Some(expression_name) =
                find_expression_name(&self.symbols, &self.declarations, from, symbol)
            {
                self.scope_maps[from.index()].insert(name, expression_name);
            } else {
                self.scope_maps[from.index()].remove(name);
            }
            self.scope_maps[to.index()].insert(name, symbol);
        }
        let first = self.symbol(symbol).declaration;
        for id in std::iter::once(first).chain(
            self.redeclarations
                .get(&symbol)
                .into_iter()
                .flatten()
                .copied(),
        ) {
            let declaration = &mut self.declarations[id.index()];
            if declaration.scope == from {
                declaration.scope = to;
            }
        }
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
        let original = self.scope(scope);
        assert!(original.active && self.scope(parent).active);
        let previous = original.parent.expect("cannot reparent a root scope");
        if previous == parent {
            return;
        }
        let mut ancestor = Some(parent);
        while let Some(id) = ancestor {
            assert_ne!(id, scope, "scope parent must not create a cycle");
            ancestor = self.scope(id).parent;
        }
        if original.kind == ScopeKind::FunctionBody {
            assert_eq!(self.scope(parent).kind, ScopeKind::Function);
            assert!(self.scope(parent).function_body.is_none());
            self.scopes.list[previous.index()].function_body = None;
            self.scopes.list[parent.index()].function_body = Some(scope);
        }
        self.scopes.list[scope.index()].parent = Some(parent);
        let target = if original.kind.is_hoist_target() {
            scope
        } else {
            self.scope(parent).hoist_target
        };
        self.update_hoist_target(scope, target);
    }

    /// Apply caller-computed scope flags without changing the scope's structure or identities.
    ///
    /// Only this scope's flags change; the caller must maintain strictness inheritance in affected scopes.
    ///
    /// ```javascript
    /// // Before: f's body is not explicitly strict.
    /// function f() { run(); }
    /// // After: the caller inserts a directive and computes the new strictness flag.
    /// function f() { "use strict"; run(); }
    /// // Apply the flags while keeping owner, parent, kind, bindings and node/reference mappings.
    /// ```
    pub fn set_scope_flags(&mut self, scope: ScopeId, flags: ScopeFlags) {
        assert!(self.scope(scope).active);
        self.scopes.list[scope.index()].flags = flags;
    }

    /// Retire an empty non-root scope and clear its detached owner's mapping without recycling its ID.
    ///
    /// ```javascript
    /// // Before: an empty static block still owns a scope.
    /// class C { static {} }
    /// // After: the caller removes the block from the class body.
    /// class C {}
    /// // Require no remaining bindings, declarations, references, child scopes or retained nodes.
    /// // Clear the block owner's mapping and mark its scope inactive; do not delete descendants.
    /// // For a reused block body, convert_scope keeps the scope instead.
    /// ```
    pub fn remove_scope(&mut self, scope: ScopeId) {
        let original = self.scope(scope);
        assert!(original.active);
        let parent = original.parent.expect("cannot remove a root scope");
        assert!(self.scope_maps[scope.index()].is_empty());
        assert!(
            !self
                .declarations
                .iter()
                .any(|declaration| declaration.scope == scope)
        );
        assert!(
            !self
                .iter_references()
                .any(|entry| entry.reference.scope == scope)
        );
        assert!(
            !self
                .iter_scopes()
                .any(|entry| entry.scope.parent == Some(scope))
        );
        assert!(
            self.node_scopes
                .iter()
                .enumerate()
                .all(|(index, &mapped)| mapped != Some(scope) || index == original.node.index())
        );
        assert!(original.function_body.is_none());
        if original.kind == ScopeKind::FunctionBody {
            self.scopes.list[parent.index()].function_body = None;
        }
        self.scopes.list[scope.index()].active = false;
        self.clear_node_scope(original.node);
    }

    /// Clear a detached node's evaluation mapping without changing mappings of its retained children.
    ///
    /// ```javascript
    /// // Before: use(value) is attached through an ExpressionStatement.
    /// class C { static { use(value); } }
    /// // After: its expression is reused directly as a field initializer.
    /// class C { static #_ = use(value); }
    /// // Move the expression first, then clear only the discarded ExpressionStatement mapping.
    /// // Its call and references remain registered; the shell must own no scope, binding or reference.
    /// ```
    pub fn clear_node_scope(&mut self, node: NodeId) {
        assert!(self.symbol_of(node).is_none() && self.reference_of(node).is_none());
        assert!(!self.iter_scopes().any(|entry| entry.scope.node == node));
        self.node_scopes[node.index()] = None;
    }

    /// Refresh inherited hoist targets in a changed scope's branch without changing bindings.
    ///
    /// ```javascript
    /// // Before: the inner block inherits f's var hoist target.
    /// function f() { { use(); } }
    /// // After: the caller moves that block below an arrow Function scope.
    /// function f() { (() => { { use(); } })(); }
    /// // Redirect inherited hoist targets in the moved branch to the arrow.
    /// // Independent function/static-block targets and binding ownership stay unchanged.
    /// ```
    fn update_hoist_target(&mut self, scope: ScopeId, target: ScopeId) {
        let previous = self.scope(scope).hoist_target;
        if previous == target {
            return;
        }
        self.scopes.list[scope.index()].hoist_target = target;
        for index in 0..self.scopes.list.len() {
            let current = self.scopes.list[index];
            if !current.active || current.kind.is_hoist_target() || current.hoist_target != previous
            {
                continue;
            }
            let mut ancestor = current.parent;
            while let Some(id) = ancestor {
                if id == scope {
                    self.scopes.list[index].hoist_target = target;
                    break;
                }
                ancestor = self.scope(id).parent;
            }
        }
    }
}
