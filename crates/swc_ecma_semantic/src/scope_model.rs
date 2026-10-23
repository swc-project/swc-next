//! Scope, binding, declaration, and reference indexes for transforms.
//!
//! The model consumes Semantic's basic indexes without retaining its hoisting
//! conflict tables or TypeScript member and alias lookup indexes.
//! Index mutations take semantic records and identifiers, not an AST. Transforms
//! read syntax and coordinate subtree edits through TransformCtx.

use oxc_index::Idx;
use swc_next_allocator::vec::Vec as ArenaVec;
use swc_next_ecma_ast::{AstUtf8, NodeId, ReferenceId, ScopeId, SymbolId};

use crate::{
    BindingIterator, Declaration, DeclarationId, DeclarationIdIterator, DeclarationIterator,
    NodeSemanticId, Redeclarations, Reference, ReferenceIterator, ScopeIterator, ScopeMap,
    Semantic, Symbol, SymbolFlags, SymbolIterator,
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
    node_semantic: ArenaVec<'a, NodeSemanticId>,
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
            node_semantic: semantic.node_semantic,
        }
    }

    /// The scope with the given id.
    pub fn scope(&self, id: ScopeId) -> Scope {
        self.scopes.get(id)
    }

    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        self.symbols[id.index()]
    }

    /// The reference with the given id.
    pub fn reference(&self, id: ReferenceId) -> Reference {
        self.references[id.index()]
    }

    /// The declaration with the given id.
    pub fn declaration(&self, id: DeclarationId) -> Declaration {
        self.declarations[id.index()]
    }

    /// The symbol declared at `node`, or `None` when the node is not a
    /// binding or member declaration site. Source-module and anonymous default
    /// export symbols do not have a node mapping.
    pub fn symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        self.node_semantic[node.index()].symbol()
    }

    /// The reference recorded at `node`, or `None` when the node is
    /// not a reference site.
    pub fn reference_of(&self, node: NodeId) -> Option<ReferenceId> {
        self.node_semantic[node.index()].reference()
    }

    /// The scope created by the given node, or `None` if the node is not a
    /// scope-creating site. If a node creates multiple scopes, returns the
    /// last one created (for example, the program's source-type wrapper).
    pub fn scope_of(&self, node: NodeId) -> Option<ScopeId> {
        self.node_semantic[node.index()].scope()
    }

    /// The scope of the symbol's first declaration. Merged TypeScript symbols
    /// may have additional declarations in other scopes.
    pub fn scope_of_symbol(&self, id: SymbolId) -> ScopeId {
        self.declaration(self.symbol(id).declaration).scope
    }

    /// Iterates the ids of every declaration of `id`, in source order.
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

    /// Every declaration of `id`, in source order. Usually a single element.
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
    /// recorded at their hoist target; passing-through hoisting occurrences
    /// and TypeScript source-export and member views are not retained by this model.
    pub fn binding(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.scope_maps[scope.index()].get(name).copied()
    }

    /// Every lexical symbol owned directly by `scope`. A hoisting `var`
    /// appears in its hoist target, not in blocks it passes through.
    pub fn bindings(&self, scope: ScopeId) -> BindingIterator<'_> {
        BindingIterator {
            inner: self.scope_maps[scope.index()].values(),
            source: None,
        }
    }

    /// Iterates every `(id, scope)` pair in creation order.
    pub fn iter_scopes(&self) -> ScopeIterator<'_> {
        ScopeIterator {
            list: &self.scopes.list,
            index: 0,
        }
    }

    /// Iterates every `(id, symbol)` pair in declaration order.
    pub fn iter_symbols(&self) -> SymbolIterator<'_> {
        SymbolIterator {
            symbols: &self.symbols,
            index: 0,
        }
    }

    /// Iterates every `(id, reference)` pair in source order.
    pub fn iter_references(&self) -> ReferenceIterator<'_> {
        ReferenceIterator {
            references: &self.references,
            node_semantic: &self.node_semantic,
            index: 0,
        }
    }

    /// Iterates every `(id, declaration)` pair in source order.
    pub fn iter_declarations(&self) -> DeclarationIterator<'_> {
        DeclarationIterator {
            declarations: &self.declarations,
            index: 0,
        }
    }

    /// Extend node indexes for new allocations without changing existing entries.
    ///
    /// ```javascript
    /// // Before: run() and its reference are already indexed.
    /// run();
    /// // After: the caller allocates a unary wrapper in the same evaluation scope.
    /// void run();
    /// // Grow through the new NodeId and leave scope, symbol and reference slots empty.
    /// // Keep the existing call/reference entries; register new bindings and references separately.
    /// ```
    pub fn grow_to(&mut self, node_count: usize) {
        // Never truncate indexes when an already registered node is reused.
        if node_count > self.node_semantic.len() {
            self.node_semantic.resize(node_count, NodeSemanticId::None);
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
        debug_assert!(self.scope(scope).active);
        self.grow_to(node.index() + 1);
        if let Some(symbol) = self.symbol_of(node) {
            return symbol;
        }
        let declaration = DeclarationId::from_usize(self.declarations.len());
        let previous = self.binding(scope, text.as_bytes());
        let existing = previous.filter(|&symbol| {
            !self
                .symbol(symbol)
                .flags
                .intersects(SymbolFlags::INTERNAL_NAME)
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
        debug_assert_eq!(self.node_semantic[node.index()], NodeSemanticId::None);
        self.node_semantic[node.index()] = NodeSemanticId::Symbol(symbol);
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
        debug_assert!(self.scope(scope).active);
        self.grow_to(node.index() + 1);
        debug_assert!(
            self.reference_of(node).is_none(),
            "reference already registered"
        );
        let id = ReferenceId::from_usize(self.references.len());
        self.references.push(reference);
        debug_assert_eq!(self.node_semantic[node.index()], NodeSemanticId::None);
        self.node_semantic[node.index()] = NodeSemanticId::Reference(id);
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
        debug_assert!(parent_scope.active);
        debug_assert!(!self.iter_scopes().any(|entry| entry.scope.node == owner));
        debug_assert!(!matches!(kind, ScopeKind::Global | ScopeKind::Module));
        if kind == ScopeKind::FunctionBody {
            debug_assert_eq!(parent_scope.kind, ScopeKind::Function);
            debug_assert!(parent_scope.function_body.is_none());
        }
        self.grow_to(owner.index() + 1);
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
        debug_assert!(matches!(
            self.node_semantic[owner.index()],
            NodeSemanticId::None | NodeSemanticId::Scope(_)
        ));
        self.node_semantic[owner.index()] = NodeSemanticId::Scope(id);
        if kind == ScopeKind::FunctionBody {
            self.scopes.list[parent.index()].function_body = Some(id);
        }
        id
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
    /// // Replace the StaticBlock owner with the arrow owner and change kind to Function.
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
        let original = self.scope(scope);
        debug_assert!(original.active);
        let parent = original.parent.expect("cannot convert a root scope");
        debug_assert!(!matches!(kind, ScopeKind::Global | ScopeKind::Module));
        debug_assert!(kind == ScopeKind::Function || original.function_body.is_none());
        debug_assert!(!self.iter_scopes().any(|entry| {
            entry.id != scope && (entry.scope.node == original.node || entry.scope.node == owner)
        }));
        if kind == ScopeKind::FunctionBody {
            debug_assert_eq!(self.scope(parent).kind, ScopeKind::Function);
            debug_assert!(
                self.scope(parent)
                    .function_body
                    .is_none_or(|body| body == scope)
            );
        }
        self.grow_to(owner.index() + 1);
        if original.node != owner {
            debug_assert_eq!(self.scope_of(original.node), Some(scope));
            self.node_semantic[original.node.index()] = NodeSemanticId::None;
        }
        debug_assert!(matches!(
            self.node_semantic[owner.index()],
            NodeSemanticId::None | NodeSemanticId::Scope(_)
        ));
        self.node_semantic[owner.index()] = NodeSemanticId::Scope(scope);
        if original.kind == ScopeKind::FunctionBody && kind != ScopeKind::FunctionBody {
            self.scopes.list[parent.index()].function_body = None;
        } else if kind == ScopeKind::FunctionBody {
            self.scopes.list[parent.index()].function_body = Some(scope);
        }
        let current = &mut self.scopes.list[scope.index()];
        current.node = owner;
        current.kind = kind;
        current.flags = flags;
        let target = if kind.is_hoist_target() {
            scope
        } else {
            self.scope(parent).hoist_target
        };
        self.update_hoist_target(scope, target);
    }

    /// Move a reference between active scopes without changing its identity,
    /// target, flags, or use lists. Scope ownership and declarations are separate.
    pub fn move_reference_to_scope(&mut self, reference: ReferenceId, from: ScopeId, to: ScopeId) {
        debug_assert!(self.scope(from).active && self.scope(to).active);
        debug_assert_eq!(self.reference(reference).scope, from);
        self.references[reference.index()].scope = to;
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
        debug_assert!(original.active && self.scope(parent).active);
        let previous = original.parent.expect("cannot reparent a root scope");
        if previous == parent {
            return;
        }
        debug_assert!(
            self.scopes.ancestors(Some(parent)).all(|id| id != scope),
            "scope parent must not create a cycle"
        );
        if original.kind == ScopeKind::FunctionBody {
            debug_assert_eq!(self.scope(parent).kind, ScopeKind::Function);
            debug_assert!(self.scope(parent).function_body.is_none());
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
        debug_assert!(self.scope(scope).active);
        self.scopes.list[scope.index()].flags = flags;
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
        let original = self.scope(scope);
        debug_assert!(original.active);
        let parent = original.parent.expect("cannot remove a root scope");
        debug_assert_eq!(self.scope_of(original.node), Some(scope));
        debug_assert!(
            !self
                .iter_scopes()
                .any(|entry| { entry.id != scope && entry.scope.node == original.node })
        );
        debug_assert!(self.scope_maps[scope.index()].is_empty());
        debug_assert!(
            !self
                .declarations
                .iter()
                .any(|declaration| declaration.scope == scope)
        );
        debug_assert!(
            !self
                .iter_references()
                .any(|entry| entry.reference.scope == scope)
        );
        debug_assert!(
            !self
                .iter_scopes()
                .any(|entry| entry.scope.parent == Some(scope))
        );
        debug_assert!(original.function_body.is_none());
        if original.kind == ScopeKind::FunctionBody {
            self.scopes.list[parent.index()].function_body = None;
        }
        self.scopes.list[scope.index()].active = false;
        self.node_semantic[original.node.index()] = NodeSemanticId::None;
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
