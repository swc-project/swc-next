//! Scope, binding, declaration, and reference indexes for transforms.
//!
//! The model consumes Semantic's basic indexes without retaining its hoisting
//! conflict tables, module record, or TypeScript member and alias lookup indexes.
//! Mutation operations will be designed separately.

use oxc_index::Idx;
use swc_next_allocator::vec::Vec as ArenaVec;
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, IdentifierReference, NodeId, ReferenceId, ScopeId, SymbolId,
};

use crate::{
    Declaration, DeclarationId, DeclarationIdIterator, DeclarationIterator, Redeclarations,
    Reference, ReferenceFlags, ReferenceIterator, ScopeIterator, ScopeMap, Semantic, Symbol,
    SymbolFlags, SymbolIterator,
    scope::{Scope, ScopeTree},
};

/// The basic semantic indexes needed by Babel-style transforms.
///
/// Scope, symbol, declaration, reference, and node identities are preserved.
/// Existing scope kinds and declaration/reference flags remain input facts;
/// this model does not provide Semantic's TypeScript name-resolution queries.
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

    pub fn register_binding(
        &mut self,
        ast: &Ast<'a>,
        scope: ScopeId,
        binding: BindingIdentifier,
        flags: SymbolFlags,
    ) -> SymbolId {
        self.grow_to(ast.node_count(), scope);
        let node = binding.node_id();
        if let Some(symbol) = self.symbol_of(node) {
            return symbol;
        }
        let text = ast.get_utf8(binding.name(ast));
        let name = binding.name(ast).into();
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
                name,
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

    pub fn register_reference(
        &mut self,
        ast: &Ast,
        scope: ScopeId,
        reference: IdentifierReference,
        target: Option<SymbolId>,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        self.grow_to(ast.node_count(), scope);
        let node = reference.node_id();
        assert!(
            self.reference_of(node).is_none(),
            "reference already registered"
        );
        let id = ReferenceId::from_usize(self.references.len());
        self.references.push(Reference {
            name: reference.name(ast),
            scope,
            node,
            symbol: target,
            flags,
        });
        self.node_references[node.index()] = Some(id);
        self.node_scopes[node.index()] = Some(scope);
        if let Some(symbol) = target {
            self.uses[symbol.index()].push(id);
        }
        id
    }
}
