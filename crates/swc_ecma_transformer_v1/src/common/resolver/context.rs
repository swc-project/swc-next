//! Symbol and scope identities for the Next AST transforms.

use std::{collections::HashMap, hash::BuildHasher};

use hashbrown::{HashTable, hash_table::Entry};
use rustc_hash::FxBuildHasher;
use swc_next_allocator::{Idx, IndexVec};
use swc_next_ecma_ast::*;

use super::super::{hygiene, rename};
use super::resolver;

#[derive(Clone, Copy, Debug, Default)]
pub(super) enum NodeContext {
    #[default]
    None,
    Symbol(SymbolId),
    Scope(ScopeId),
}

#[derive(Clone, Copy)]
struct Symbol {
    name: AstUtf8,
    scope: ScopeId,
}

/// Preserve SWC's identity for equal spellings in the same syntax context.
/// Use AST string handles for storage and compare their contents for lookup.
#[derive(Default)]
pub(super) struct Symbols {
    entries: IndexVec<SymbolId, Symbol>,
    by_name: HashTable<SymbolId>,
}

impl Symbols {
    fn hash(scope: ScopeId, name: &str) -> u64 {
        FxBuildHasher.hash_one((scope, name))
    }

    fn find(&self, ast: &Ast<'_>, scope: ScopeId, name: &str) -> Option<SymbolId> {
        self.by_name
            .find(Self::hash(scope, name), |&id| {
                let symbol = self.entries[id];
                symbol.scope == scope && ast.get_utf8(symbol.name) == name
            })
            .copied()
    }

    pub(super) fn intern(&mut self, ast: &Ast<'_>, scope: ScopeId, name: AstUtf8) -> SymbolId {
        let name_text = ast.get_utf8(name);
        let hash = Self::hash(scope, name_text);
        match self.by_name.entry(
            hash,
            |&id| {
                let symbol = self.entries[id];
                symbol.scope == scope && ast.get_utf8(symbol.name) == name_text
            },
            |&id| {
                let symbol = self.entries[id];
                Self::hash(symbol.scope, ast.get_utf8(symbol.name))
            },
        ) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = self.entries.push(Symbol { name, scope });
                entry.insert(id);
                id
            }
        }
    }

    fn rename(&mut self, ast: &Ast<'_>, renames: HashMap<SymbolId, AstUtf8>) {
        for (id, name) in renames {
            let symbol = &mut self.entries[id];
            let scope = symbol.scope;
            let old_hash = Self::hash(scope, ast.get_utf8(symbol.name));
            if let Ok(entry) = self.by_name.find_entry(old_hash, |&other| other == id) {
                entry.remove();
            }
            symbol.name = name;

            // A type-only name erased by TS may still occur in the index.
            // Lookups after renaming must find the surviving binding.
            let text = ast.get_utf8(name);
            let hash = Self::hash(scope, text);
            match self.by_name.entry(
                hash,
                |&other| {
                    let other = self.entries[other];
                    other.scope == scope && ast.get_utf8(other.name) == text
                },
                |&other| {
                    let other = self.entries[other];
                    Self::hash(other.scope, ast.get_utf8(other.name))
                },
            ) {
                Entry::Occupied(mut entry) => *entry.get_mut() = id,
                Entry::Vacant(entry) => {
                    entry.insert(id);
                }
            }
        }
    }
}

/// Tracks the legacy resolver's identities as transforms move, copy and add nodes.
pub(crate) struct Resolver {
    nodes: IndexVec<NodeId, NodeContext>,
    symbols: Symbols,
    top_level: ScopeId,
    unresolved: ScopeId,
    next_scope: usize,
}

impl Resolver {
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/resolver/mod.rs#L136>.
    pub(crate) fn new(ast: &Ast<'_>) -> Self {
        let resolved = resolver(ast);
        Self {
            top_level: resolved.top_level_scope_id,
            unresolved: resolved.unresolved_scope_id,
            next_scope: resolved.next_scope,
            nodes: resolved.nodes,
            symbols: resolved.symbols,
        }
    }

    fn context(&self, node: NodeId) -> NodeContext {
        self.nodes.get(node).copied().unwrap_or_default()
    }

    fn set_context(&mut self, node: NodeId, context: NodeContext) {
        if node.index() >= self.nodes.len() {
            self.nodes.resize(node.index() + 1, NodeContext::None);
        }
        self.nodes[node] = context;
    }

    pub(crate) fn copy_context(&mut self, from: NodeId, to: NodeId) {
        self.set_context(to, self.context(from));
    }

    pub(crate) fn symbol(&self, node: NodeId) -> Option<SymbolId> {
        match self.context(node) {
            NodeContext::Symbol(symbol) => Some(symbol),
            _ => None,
        }
    }

    pub(crate) fn symbol_of(&self, node: NodeId) -> SymbolId {
        self.symbol(node).expect("identifier has a symbol")
    }

    pub(crate) fn same_symbol(&self, node: NodeId, other: NodeId) -> bool {
        self.symbol(node)
            .is_some_and(|symbol| self.symbol(other) == Some(symbol))
    }

    pub(crate) fn scope_of(&self, node: NodeId) -> Option<ScopeId> {
        match self.context(node) {
            NodeContext::None => None,
            NodeContext::Scope(scope) => Some(scope),
            NodeContext::Symbol(symbol) => Some(self.scope_of_symbol(symbol)),
        }
    }

    pub(crate) fn is_unresolved(&self, node: NodeId) -> bool {
        self.scope_of(node).unwrap_or(self.unresolved) == self.unresolved
    }

    pub(crate) fn scope_of_symbol(&self, symbol: SymbolId) -> ScopeId {
        self.symbols.entries[symbol].scope
    }

    pub(crate) fn symbol_count(&self) -> usize {
        self.symbols.entries.len()
    }

    pub(crate) fn symbol_name(&self, symbol: SymbolId) -> AstUtf8 {
        self.symbols.entries[symbol].name
    }

    pub(crate) fn set_symbol(&mut self, node: NodeId, symbol: SymbolId) {
        self.set_context(node, NodeContext::Symbol(symbol));
    }

    pub(crate) fn lookup(&self, ast: &Ast<'_>, scope: ScopeId, name: &str) -> Option<SymbolId> {
        self.symbols.find(ast, scope, name)
    }

    pub(crate) fn scoped_symbol(
        &mut self,
        ast: &mut Ast<'_>,
        scope: ScopeId,
        name: &str,
    ) -> SymbolId {
        if let Some(symbol) = self.lookup(ast, scope, name) {
            return symbol;
        }
        let name = ast.add_utf8(name);
        self.symbols.intern(ast, scope, name)
    }

    pub(crate) fn binding(&mut self, ast: &mut Ast<'_>, symbol: SymbolId) -> BindingIdentifier {
        let binding = ast.binding_identifier(DUMMY_SP, self.symbol_name(symbol), None);
        self.set_symbol(binding.node_id(), symbol);
        binding
    }

    pub(crate) fn top_level(&self) -> ScopeId {
        self.top_level
    }

    pub(crate) fn fresh_scope(&mut self) -> ScopeId {
        let scope = ScopeId::from_usize(self.next_scope);
        self.next_scope += 1;
        scope
    }

    /// SWC `private_ident!`: each generated binding receives a fresh context.
    pub(crate) fn private_binding(&mut self, ast: &mut Ast<'_>, name: &str) -> BindingIdentifier {
        let scope = self.fresh_scope();
        let symbol = self.scoped_symbol(ast, scope, name);
        self.binding(ast, symbol)
    }

    pub(crate) fn symbol_reference(&mut self, ast: &mut Ast<'_>, symbol: SymbolId) -> Expr {
        let reference = ast.identifier_reference(DUMMY_SP, self.symbol_name(symbol));
        self.set_symbol(reference.node_id(), symbol);
        Expr::IdentifierReference(reference)
    }

    pub(crate) fn reference(&mut self, ast: &mut Ast<'_>, binding: BindingIdentifier) -> Expr {
        self.symbol_reference(ast, self.symbol_of(binding.node_id()))
    }

    /// Copy the resolver identity of the original JSX component root.
    pub(crate) fn source_reference(
        &mut self,
        ast: &mut Ast<'_>,
        identifier: JsxIdentifier,
    ) -> Expr {
        let reference = ast.identifier_reference(identifier.span(ast), identifier.name(ast));
        self.copy_context(identifier.node_id(), reference.node_id());
        Expr::IdentifierReference(reference)
    }

    /// SWC marks pragma roots with the top-level mark and `require` with the
    /// unresolved mark.
    pub(crate) fn marked_reference(
        &mut self,
        ast: &mut Ast<'_>,
        name: &str,
        unresolved: bool,
    ) -> Expr {
        let scope = if unresolved {
            self.unresolved
        } else {
            self.top_level
        };
        let symbol = self.scoped_symbol(ast, scope, name);
        self.symbol_reference(ast, symbol)
    }

    pub(crate) fn rename(&mut self, ast: &mut Ast<'_>, renames: HashMap<SymbolId, AstUtf8>) {
        rename::rename(ast, self, &renames);
        self.symbols.rename(ast, renames);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/hygiene/mod.rs#L68>.
    pub(crate) fn hygiene(&mut self, ast: &mut Ast<'_>) {
        hygiene::hygiene(ast, self);
    }
}
