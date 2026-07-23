use oxc_index::IndexVec;
use rustc_hash::FxHashMap;
use swc_experimental_ecma_ast::{NodeId, ScopeId, Span, String as AstString, SymbolId};

use crate::scope::{ScopeKind, ScopeTree};

oxc_index::define_index_type! {
    /// Identifier for a [`Reference`].
    pub struct ReferenceId = u32;
}

/// Declaration kinds, modifiers, and namespace membership recorded for a
/// symbol.
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
    pub const ANNEX_B_FUNCTION: Self = Self(1 << 19);
    pub const CAPTURED: Self = Self(1 << 20);
    pub const REASSIGNED: Self = Self(1 << 21);

    /// Declarations visible in JavaScript value space at runtime.
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
    /// Declarations visible in TypeScript type space.
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

    /// Returns the union of two flag sets.
    ///
    /// This is used when compatible declarations merge into one symbol.
    pub const fn merge(self, other: Self) -> Self {
        self.union(other)
    }

    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Returns true when the two sets have at least one flag in common.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns true for a `var` that hoists past intermediate blocks.
    ///
    /// Parameters and catch variables are function-scoped but do not hoist.
    pub const fn is_hoisting_var(self) -> bool {
        self.contains(Self::FUNCTION_SCOPED_VAR)
            && !self.intersects(Self(Self::PARAMETER.0 | Self::CATCH_VAR.0))
    }

    /// Returns true for declarations a hoisting `var` cannot pass through:
    /// block-scoped bindings, classes, and functions.
    pub const fn is_block_scoped_like(self) -> bool {
        self.intersects(Self(
            Self::BLOCK_SCOPED_VAR.0 | Self::CLASS.0 | Self::FUNCTION.0,
        ))
    }

    /// Returns true for a runtime value-space binding.
    pub const fn in_value_space(self) -> bool {
        self.intersects(Self::VALUE_SPACE)
    }

    /// Returns true for a TypeScript type-space binding.
    pub const fn in_type_space(self) -> bool {
        self.intersects(Self::TYPE_SPACE)
    }

    pub const fn value_space() -> Self {
        Self::VALUE_SPACE
    }

    pub const fn type_space() -> Self {
        Self::TYPE_SPACE
    }

    pub const fn block_var_excludes() -> Self {
        Self::value_space()
    }

    pub const fn function_var_excludes() -> Self {
        Self::value_space()
            .without(Self::FUNCTION_SCOPED_VAR)
            .without(Self::FUNCTION)
    }

    pub const fn function_excludes(allow_overload: bool) -> Self {
        if allow_overload {
            Self::value_space()
                .without(Self::FUNCTION_SCOPED_VAR)
                .without(Self::FUNCTION)
                .without(Self::VALUE_MODULE)
                .without(Self::CLASS)
        } else {
            Self::block_var_excludes()
        }
    }

    pub const fn class_excludes() -> Self {
        Self::value_space()
            .merge(Self::type_space())
            .without(Self::VALUE_MODULE)
            .without(Self::INTERFACE)
    }

    pub const fn interface_excludes() -> Self {
        Self::type_space()
            .without(Self::INTERFACE)
            .without(Self::CLASS)
    }

    pub const fn type_alias_excludes() -> Self {
        Self::type_space()
    }

    pub const fn regular_enum_excludes() -> Self {
        Self::value_space()
            .merge(Self::type_space())
            .without(Self::REGULAR_ENUM)
            .without(Self::VALUE_MODULE)
    }

    pub const fn const_enum_excludes() -> Self {
        Self::value_space()
            .merge(Self::type_space())
            .without(Self::CONST_ENUM)
    }

    pub const fn value_module_excludes() -> Self {
        Self::value_space()
            .without(Self::FUNCTION)
            .without(Self::CLASS)
            .without(Self::REGULAR_ENUM)
            .without(Self::VALUE_MODULE)
    }

    pub const fn namespace_module_excludes() -> Self {
        Self(0)
    }

    pub const fn parameter_excludes() -> Self {
        Self::value_space().without(Self::FUNCTION_SCOPED_VAR)
    }

    pub const fn catch_param_excludes() -> Self {
        Self::value_space()
    }

    pub const fn import_binding_excludes() -> Self {
        Self::IMPORT.merge(Self::TYPE_IMPORT)
    }

    pub const fn type_parameter_excludes() -> Self {
        Self::type_space().without(Self::TYPE_PARAMETER)
    }

    /// Returns a human-readable declaration category for diagnostics.
    pub fn to_kind_string(self) -> &'static str {
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

/// A declared binding.
///
/// Declarations that legally share a name merge into one symbol, including
/// `var` redeclarations, TypeScript overloads, and `class` + `interface`
/// merging. [`Symbol::declarations`] contains every declaration site, while
/// [`Symbol::flags`] describes the declaration kinds, modifiers, and spaces
/// occupied by the symbol.
///
/// ## Example
///
/// ```ts
/// interface Shape { kind: string }
/// class Shape { kind = "circle" }
/// //    ^^^^^ one symbol with two declarations in value and type space
/// ```
#[derive(Clone, Debug)]
pub struct Symbol {
    /// Interned declaration name.
    pub name: AstString,
    /// Merged declaration kinds, modifiers, and namespace membership.
    pub flags: SymbolFlags,
    /// Scope where the symbol is declared.
    ///
    /// For a hoisting `var`, this is the hoist target instead of the block
    /// where the declaration was written.
    pub scope: ScopeId,
    /// Declaration nodes in source order.
    pub declarations: Vec<NodeId>,
    /// Spans parallel to [`Symbol::declarations`].
    pub declaration_spans: Vec<Span>,
    /// References resolved to this symbol, in source order.
    pub references: Vec<ReferenceId>,
}

/// Declaration space in which a reference is resolved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceKind {
    /// A runtime value use.
    Value,
    /// A type-position use.
    Type,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReferenceFlags {
    /// The reference reads the current value.
    pub read: bool,
    /// The reference reassigns its binding.
    ///
    /// This includes assignment targets, `++`/`--` operands, for-in/for-of
    /// iteration variables, and destructuring-assignment leaves. Declaration
    /// initializers are not writes.
    pub write: bool,
    /// The reference crosses a function boundary.
    pub captured: bool,
    /// Resolution can be intercepted by a surrounding `with` statement.
    pub dynamic: bool,
    /// Access may throw because the binding is still in its temporal dead zone.
    pub may_throw_tdz: bool,
}

/// A use of a name.
///
/// One reference is recorded for each identifier reference, JSX component tag
/// name, and TypeScript type-predicate parameter. Declaration sites are stored
/// on [`Symbol`] instead.
///
/// ## Example
///
/// ```ts
/// let a = b; a = 2;
/// //      ^ read of `b`
/// //         ^ write of `a`
/// ```
#[derive(Clone, Debug)]
pub struct Reference {
    /// Interned referenced name.
    pub name: AstString,
    /// Scope containing the reference.
    pub scope: ScopeId,
    /// AST node that references the name.
    pub node: NodeId,
    /// Source span of the reference.
    pub span: Span,
    /// Declaration space in which the name is resolved.
    pub kind: ReferenceKind,
    /// Convenience mirror of [`ReferenceFlags::write`].
    pub is_write: bool,
    /// Read/write and resolution metadata.
    pub flags: ReferenceFlags,
    /// Declaring symbol, or `None` when no visible binding resolves the name.
    pub symbol_id: Option<SymbolId>,
}

/// Symbol and reference storage with per-scope name indexes.
#[derive(Debug)]
pub struct SymbolTable {
    symbols: IndexVec<SymbolId, Symbol>,
    references: IndexVec<ReferenceId, Reference>,
    scope_bindings: IndexVec<ScopeId, FxHashMap<Vec<u8>, SymbolId>>,
    hoisted_bindings: IndexVec<ScopeId, FxHashMap<Vec<u8>, SymbolId>>,
    unresolved: Vec<ReferenceId>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DeclareResult {
    pub symbol_id: SymbolId,
    pub conflict: Option<Span>,
}

impl SymbolTable {
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

    /// Iterates symbols in declaration order.
    pub fn symbols(&self) -> impl Iterator<Item = (SymbolId, &Symbol)> {
        self.symbols.iter_enumerated()
    }

    /// Returns the number of symbols.
    pub fn symbol_len(&self) -> usize {
        self.symbols.len()
    }

    /// Iterates references in source order.
    pub fn references(&self) -> impl Iterator<Item = (ReferenceId, &Reference)> {
        self.references.iter_enumerated()
    }

    /// Returns the number of references.
    pub fn reference_len(&self) -> usize {
        self.references.len()
    }

    /// Iterates references that did not resolve to a visible symbol.
    pub fn unresolved_references(&self) -> impl Iterator<Item = (ReferenceId, &Reference)> {
        self.unresolved
            .iter()
            .copied()
            .map(|id| (id, &self.references[id]))
    }

    /// Returns the symbol with the given ID.
    pub fn symbol(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id]
    }

    /// Returns the reference with the given ID.
    pub fn reference(&self, id: ReferenceId) -> &Reference {
        &self.references[id]
    }

    /// Returns the binding declared directly in `scope`, excluding hoisted
    /// variables passing through it.
    pub fn find_in_scope(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.scope_bindings[scope].get(name).copied()
    }

    /// Returns the binding at `scope`, including a hoisting `var` passing
    /// through on the way to its target.
    pub fn find_in_scope_or_hoisted(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.find_in_scope(scope, name)
            .or_else(|| self.hoisted_bindings[scope].get(name).copied())
    }

    /// Finds the nearest binding visible from `scope` by walking outward.
    pub fn resolve(&self, scopes: &ScopeTree, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        scopes
            .ancestors(scope)
            .find_map(|ancestor| self.find_in_scope_or_hoisted(ancestor, name))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn declare(
        &mut self,
        name: AstString,
        name_bytes: &[u8],
        flags: SymbolFlags,
        excludes: SymbolFlags,
        target: ScopeId,
        node: NodeId,
        span: Span,
    ) -> DeclareResult {
        if let Some(existing) = self.find_in_scope_or_hoisted(target, name_bytes) {
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
        self.scope_bindings[target].insert(name_bytes.to_vec(), symbol_id);
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
        name: &[u8],
        symbol_id: SymbolId,
    ) {
        for scope in scopes.ancestors(current) {
            if scope == target {
                break;
            }
            self.hoisted_bindings[scope]
                .entry(name.to_vec())
                .or_insert(symbol_id);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_reference(
        &mut self,
        name: AstString,
        scope: ScopeId,
        node: NodeId,
        span: Span,
        kind: ReferenceKind,
        is_read: bool,
        is_write: bool,
        dynamic: bool,
    ) -> ReferenceId {
        self.references.push(Reference {
            name,
            scope,
            node,
            span,
            kind,
            is_write,
            flags: ReferenceFlags {
                read: is_read,
                write: is_write,
                dynamic,
                ..ReferenceFlags::default()
            },
            symbol_id: None,
        })
    }

    pub(crate) fn resolve_reference(
        &mut self,
        scopes: &ScopeTree,
        reference_id: ReferenceId,
        name: &[u8],
    ) -> Option<SymbolId> {
        let reference_scope = self.references[reference_id].scope;
        let reference_span = self.references[reference_id].span;
        let symbol_id = self.resolve(scopes, reference_scope, name);
        self.references[reference_id].symbol_id = symbol_id;
        if let Some(symbol_id) = symbol_id {
            let captured = self.crosses_function_boundary(scopes, reference_scope, symbol_id);
            self.references[reference_id].flags.captured = captured;
            if captured {
                self.symbols[symbol_id].flags =
                    self.symbols[symbol_id].flags.union(SymbolFlags::CAPTURED);
            }
            if self.references[reference_id].flags.write {
                self.symbols[symbol_id].flags =
                    self.symbols[symbol_id].flags.union(SymbolFlags::REASSIGNED);
            }
            let symbol = &self.symbols[symbol_id];
            let lexical = symbol.flags.intersects(
                SymbolFlags::BLOCK_SCOPED_VAR
                    .union(SymbolFlags::CLASS)
                    .union(SymbolFlags::IMPORT)
                    .union(SymbolFlags::TYPE_IMPORT),
            );
            let before_initialization = symbol
                .declaration_spans
                .first()
                .is_some_and(|declaration| reference_span.start < declaration.start);
            self.references[reference_id].flags.may_throw_tdz =
                lexical && (captured || before_initialization);
            self.symbols[symbol_id].references.push(reference_id);
        } else {
            self.unresolved.push(reference_id);
        }
        symbol_id
    }

    fn crosses_function_boundary(
        &self,
        scopes: &ScopeTree,
        reference_scope: ScopeId,
        symbol_id: SymbolId,
    ) -> bool {
        let symbol_scope = self.symbols[symbol_id].scope;
        for scope in scopes.ancestors(reference_scope) {
            if scope == symbol_scope {
                return false;
            }
            if scopes.get(scope).kind == ScopeKind::Function {
                return true;
            }
        }
        false
    }
}
