use oxc_index::IndexVec;
use rustc_hash::FxHashMap;
use swc_experimental_ecma_ast::{Atom, NodeId, ScopeId, Span, SymbolId};

use crate::scope::ScopeTree;

oxc_index::define_index_type! {
    pub struct ReferenceId = u32;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SymbolFlags(u32);

impl SymbolFlags {
    pub const FUNCTION_SCOPED_VAR: Self = Self(1 << 0);
    pub const BLOCK_SCOPED_VAR: Self = Self(1 << 1);
    pub const FUNCTION: Self = Self(1 << 2);
    pub const CLASS: Self = Self(1 << 3);
    pub const REGULAR_ENUM: Self = Self(1 << 4);
    pub const CONST_ENUM: Self = Self(1 << 5);
    pub const VALUE_MODULE: Self = Self(1 << 6);
    pub const INTERFACE: Self = Self(1 << 7);
    pub const TYPE_ALIAS: Self = Self(1 << 8);
    pub const TYPE_PARAMETER: Self = Self(1 << 9);
    pub const NAMESPACE_MODULE: Self = Self(1 << 10);
    pub const IMPORT: Self = Self(1 << 11);
    pub const TYPE_IMPORT: Self = Self(1 << 12);
    pub const CONST_VAR: Self = Self(1 << 13);
    pub const AMBIENT: Self = Self(1 << 14);
    pub const PARAMETER: Self = Self(1 << 15);
    pub const CATCH_VAR: Self = Self(1 << 16);
    pub const EXPORTED: Self = Self(1 << 17);
    pub const DEFAULT_EXPORT: Self = Self(1 << 18);

    pub const VALUE_SPACE: Self = Self(
        Self::FUNCTION_SCOPED_VAR.0
            | Self::BLOCK_SCOPED_VAR.0
            | Self::FUNCTION.0
            | Self::CLASS.0
            | Self::REGULAR_ENUM.0
            | Self::CONST_ENUM.0
            | Self::VALUE_MODULE.0
            | Self::IMPORT.0,
    );
    pub const TYPE_SPACE: Self = Self(
        Self::CLASS.0
            | Self::REGULAR_ENUM.0
            | Self::CONST_ENUM.0
            | Self::INTERFACE.0
            | Self::TYPE_ALIAS.0
            | Self::TYPE_PARAMETER.0
            | Self::TYPE_IMPORT.0,
    );

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn is_hoisting_var(self) -> bool {
        self.contains(Self::FUNCTION_SCOPED_VAR)
            && !self.intersects(Self(Self::PARAMETER.0 | Self::CATCH_VAR.0))
    }

    pub const fn is_block_scoped_like(self) -> bool {
        self.intersects(Self(
            Self::BLOCK_SCOPED_VAR.0 | Self::CLASS.0 | Self::FUNCTION.0,
        ))
    }

    pub const fn in_value_space(self) -> bool {
        self.intersects(Self::VALUE_SPACE)
    }

    pub const fn in_type_space(self) -> bool {
        self.intersects(Self::TYPE_SPACE)
    }

    pub fn description(self) -> &'static str {
        if self.intersects(Self::FUNCTION) {
            "function"
        } else if self.intersects(Self::CLASS) {
            "class"
        } else if self.intersects(Self(Self::REGULAR_ENUM.0 | Self::CONST_ENUM.0)) {
            "enum"
        } else if self.intersects(Self(Self::VALUE_MODULE.0 | Self::NAMESPACE_MODULE.0)) {
            "namespace"
        } else if self.intersects(Self::INTERFACE) {
            "interface"
        } else if self.intersects(Self::TYPE_ALIAS) {
            "type alias"
        } else if self.intersects(Self::TYPE_IMPORT) {
            "type import"
        } else if self.intersects(Self::IMPORT) {
            "import"
        } else if self.intersects(Self::PARAMETER) {
            "parameter"
        } else if self.intersects(Self::CATCH_VAR) {
            "catch parameter"
        } else if self.intersects(Self::TYPE_PARAMETER) {
            "type parameter"
        } else {
            "variable"
        }
    }
}

#[derive(Clone, Debug)]
pub struct Symbol<'a> {
    pub name: Atom<'a>,
    pub flags: SymbolFlags,
    pub scope: ScopeId,
    pub declarations: Vec<NodeId>,
    pub declaration_spans: Vec<Span>,
    pub references: Vec<ReferenceId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceKind {
    Value,
    Type,
}

#[derive(Clone, Debug)]
pub struct Reference<'a> {
    pub name: Atom<'a>,
    pub scope: ScopeId,
    pub node: NodeId,
    pub kind: ReferenceKind,
    pub is_write: bool,
    pub symbol_id: Option<SymbolId>,
}

#[derive(Debug)]
pub struct SymbolTable<'a> {
    symbols: IndexVec<SymbolId, Symbol<'a>>,
    references: IndexVec<ReferenceId, Reference<'a>>,
    scope_bindings: IndexVec<ScopeId, FxHashMap<Atom<'a>, SymbolId>>,
    hoisted_bindings: IndexVec<ScopeId, FxHashMap<Atom<'a>, SymbolId>>,
    unresolved: Vec<ReferenceId>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DeclareResult {
    pub symbol_id: SymbolId,
    pub conflict: Option<Span>,
}

impl<'a> SymbolTable<'a> {
    pub(crate) fn new(scope_count: usize) -> Self {
        let mut scope_bindings = IndexVec::with_capacity(scope_count);
        let mut hoisted_bindings = IndexVec::with_capacity(scope_count);
        for _ in 0..scope_count {
            scope_bindings.push(FxHashMap::default());
            hoisted_bindings.push(FxHashMap::default());
        }
        Self {
            symbols: IndexVec::new(),
            references: IndexVec::new(),
            scope_bindings,
            hoisted_bindings,
            unresolved: Vec::new(),
        }
    }

    pub(crate) fn add_scope(&mut self) {
        self.scope_bindings.push(FxHashMap::default());
        self.hoisted_bindings.push(FxHashMap::default());
    }

    pub fn symbols(&self) -> impl Iterator<Item = (SymbolId, &Symbol<'a>)> {
        self.symbols.iter_enumerated()
    }

    pub fn references(&self) -> impl Iterator<Item = (ReferenceId, &Reference<'a>)> {
        self.references.iter_enumerated()
    }

    pub fn unresolved_references(&self) -> impl Iterator<Item = (ReferenceId, &Reference<'a>)> {
        self.unresolved
            .iter()
            .copied()
            .map(|id| (id, &self.references[id]))
    }

    pub fn symbol(&self, id: SymbolId) -> &Symbol<'a> {
        &self.symbols[id]
    }

    pub fn reference(&self, id: ReferenceId) -> &Reference<'a> {
        &self.references[id]
    }

    pub fn find_in_scope(&self, scope: ScopeId, name: &str) -> Option<SymbolId> {
        self.scope_bindings[scope].get(name).copied()
    }

    pub fn find_in_scope_or_hoisted(&self, scope: ScopeId, name: &str) -> Option<SymbolId> {
        self.find_in_scope(scope, name)
            .or_else(|| self.hoisted_bindings[scope].get(name).copied())
    }

    pub fn resolve(&self, scopes: &ScopeTree, scope: ScopeId, name: &str) -> Option<SymbolId> {
        scopes
            .ancestors(scope)
            .find_map(|ancestor| self.find_in_scope_or_hoisted(ancestor, name))
    }

    pub(crate) fn declare(
        &mut self,
        name: Atom<'a>,
        flags: SymbolFlags,
        excludes: SymbolFlags,
        target: ScopeId,
        node: NodeId,
        span: Span,
    ) -> DeclareResult {
        if let Some(existing) = self.find_in_scope_or_hoisted(target, name.as_str()) {
            let symbol = &mut self.symbols[existing];
            let conflict = symbol
                .flags
                .intersects(excludes)
                .then(|| symbol.declaration_spans[0]);
            if conflict.is_none() {
                symbol.flags = symbol.flags.union(flags);
            }
            symbol.declarations.push(node);
            symbol.declaration_spans.push(span);
            return DeclareResult {
                symbol_id: existing,
                conflict,
            };
        }

        let symbol_id = self.symbols.push(Symbol {
            name,
            flags,
            scope: target,
            declarations: vec![node],
            declaration_spans: vec![span],
            references: Vec::new(),
        });
        self.scope_bindings[target].insert(name, symbol_id);
        DeclareResult {
            symbol_id,
            conflict: None,
        }
    }

    pub(crate) fn register_hoist(
        &mut self,
        scopes: &ScopeTree,
        current: ScopeId,
        target: ScopeId,
        name: Atom<'a>,
        symbol_id: SymbolId,
    ) {
        for scope in scopes.ancestors(current) {
            if scope == target {
                break;
            }
            self.hoisted_bindings[scope]
                .entry(name)
                .or_insert(symbol_id);
        }
    }

    pub(crate) fn add_reference(
        &mut self,
        name: Atom<'a>,
        scope: ScopeId,
        node: NodeId,
        kind: ReferenceKind,
        is_write: bool,
    ) -> ReferenceId {
        self.references.push(Reference {
            name,
            scope,
            node,
            kind,
            is_write,
            symbol_id: None,
        })
    }

    pub(crate) fn resolve_reference(
        &mut self,
        scopes: &ScopeTree,
        reference_id: ReferenceId,
    ) -> Option<SymbolId> {
        let reference = &self.references[reference_id];
        let symbol_id = self.resolve(scopes, reference.scope, reference.name.as_str());
        self.references[reference_id].symbol_id = symbol_id;
        if let Some(symbol_id) = symbol_id {
            self.symbols[symbol_id].references.push(reference_id);
        } else {
            self.unresolved.push(reference_id);
        }
        symbol_id
    }
}
