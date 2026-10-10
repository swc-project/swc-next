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
    scope::{Scope, ScopeFlags, ScopeKind, ScopeTree},
};

/// The basic semantic indexes needed by Babel-style transforms.
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

    pub fn grow_to(&mut self, node_count: usize, scope: ScopeId) {
        assert!(self.scope(scope).active, "target scope must be active");
        // Never truncate indexes when an already registered node is reused.
        if node_count > self.node_scopes.len() {
            self.node_scopes.resize(node_count, Some(scope));
            self.node_symbols.resize(node_count, None);
            self.node_references.resize(node_count, None);
        }
    }

    /// Register a binding from data read by the caller.
    ///
    /// `text` is the arena-lived UTF-8 spelling represented by `name`. It becomes
    /// the binding-table key; `name` remains the symbol's AST string handle.
    /// Grow indexes through this node only. The AST-facing caller registers any
    /// other new allocations with `grow_to` in their evaluation scope.
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
        let symbol = if let Some(symbol) = self.own_binding(scope, text.as_bytes()) {
            self.symbols[symbol.index()].flags |= flags;
            self.redeclarations
                .entry(symbol)
                .or_insert_with(|| ArenaVec::new_in(self.declarations.bump()))
                .push(declaration);
            symbol
        } else {
            let symbol = SymbolId::from_usize(self.symbols.len());
            self.symbols.push(Symbol {
                name: name.into(),
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

    /// Register a reference record and its node/use indexes without reading AST.
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
    /// Register an allocated scope owner below an active parent.
    ///
    /// Set the owner's node mapping, inherit strictness, and initialize the hoist
    /// target and flags according to `kind`. The owner must not already own a
    /// scope. The caller registers AST allocations and supplies syntax-dependent
    /// flags; existing children are moved separately.
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
    /// retained content stays in this scope. Refresh affected hoist targets and
    /// function-body linkage. The caller supplies syntax-dependent flags.
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

    /// Move one node's evaluation mapping and its Reference.scope, if present.
    ///
    /// Require its current evaluation scope to be `from` and both scopes active.
    /// Scope owners use `convert_scope` or `reparent_scope`. Preserve reference
    /// identity, target, flags and use lists. Declaration ownership is separate:
    /// a binding node may execute in a scope different from its var hoist target.
    /// This method does not traverse syntax or move declarations/bindings.
    pub fn move_node_to_scope(&mut self, node: NodeId, from: ScopeId, to: ScopeId) {
        todo!("Move node {node:?} evaluation from {from:?} to {to:?}")
    }

    /// Move an existing binding between scopes without allocating a new symbol.
    ///
    /// Move the binding-table entry and declarations belonging to `from`; preserve
    /// merged declarations in other scopes, declaration IDs, reference targets and
    /// use lists. Require that `to` has no conflicting binding. Node evaluation
    /// scopes are updated separately with `move_node_to_scope`.
    pub fn move_binding(&mut self, from: ScopeId, to: ScopeId, symbol: SymbolId) {
        todo!("Move binding {symbol:?} from {from:?} to {to:?}")
    }

    /// Change an active non-root scope's parent to another active scope.
    ///
    /// Reject cycles. Preserve owner, bindings and internal node/reference mappings;
    /// refresh affected descendant hoist targets. Function and static-block scopes
    /// remain their own hoist targets. Syntax-dependent strictness and direct-eval
    /// changes are coordinated by Mutation, not inferred by traversing AST here.
    pub fn reparent_scope(&mut self, scope: ScopeId, parent: ScopeId) {
        todo!("Reparent {scope:?} below {parent:?}")
    }

    /// Apply scope flags computed by the AST-facing caller.
    ///
    /// Preserve owner, parent, kind, bindings and node/reference mappings. Mutation
    /// determines syntax-dependent strictness, arguments barriers and direct-eval
    /// summaries, including any affected ancestors/descendants.
    pub fn set_scope_flags(&mut self, scope: ScopeId, flags: ScopeFlags) {
        todo!("Set flags of {scope:?} to {flags:?}")
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

    /// Clear a detached syntax shell's evaluation mapping without visiting children.
    ///
    /// Require that the node owns no active scope, binding or reference. Its
    /// children may already be retained elsewhere in the AST.
    pub fn clear_node_scope(&mut self, node: NodeId) {
        todo!("Clear scope mapping for detached shell {node:?}")
    }
}
