//! Additional TypeScript relationships between the existing semantic IDs.
//!
//! Symbols, declarations, references, lexical bindings, and declaration flags
//! continue to use `SymbolTracker`'s existing storage. These sparse side tables
//! in `TsBinder` hold only the relationships that storage does not express.
//! Binding scratch data need not become part of `Semantic`.
//!
//! The additional tables support shared namespace/enum members, class/interface
//! members, local/export identities, UMD exports, internal aliases, and module
//! instantiation. Class static members and function expandos use the export
//! tables when resolving alias targets.
//!
//! Go's `Locals` already has a counterpart in `symbol_tables`. Per-declaration
//! visibility uses the existing `Declaration.flags`, rather than aggregate
//! symbol flags. Implicit export context (`NodeFlagsExportContext`) can be
//! computed from each body's syntax and the binder's ambient context while
//! binding its declarations. A selected `ValueDeclaration` can be derived from
//! the existing declaration list and AST using `SetValueDeclaration`'s precedence
//! rules; it need not be the first declaration.
//!
//! Cross-file module resolution, expression type inference, enum constant
//! evaluation, and emit-policy-dependent alias usage are separate extensions.
//! Their checker caches are not part of these binding data definitions.
//!
//! - [`ast/symbol.go`]: symbol ownership, exports, and local/export links.
//! - [`binder/binder.go`]: export context and deferred expando binding.
//! - [`checker/types.go`]: alias targets and type-only provenance.
//! - [`ast/utilities.go`]: module-instance states and their traversal cache.
//!
//! [`ast/symbol.go`]: https://github.com/microsoft/TypeScript/blob/7502ae1a2d6a8252b3a2fe43332d2342239703d0/tsc/internal/ast/symbol.go
//! [`binder/binder.go`]: https://github.com/microsoft/TypeScript/blob/7502ae1a2d6a8252b3a2fe43332d2342239703d0/tsc/internal/binder/binder.go
//! [`checker/types.go`]: https://github.com/microsoft/TypeScript/blob/7502ae1a2d6a8252b3a2fe43332d2342239703d0/tsc/internal/checker/types.go
//! [`ast/utilities.go`]: https://github.com/microsoft/TypeScript/blob/7502ae1a2d6a8252b3a2fe43332d2342239703d0/tsc/internal/ast/utilities.go

#![expect(
    dead_code,
    reason = "side-table definitions precede TypeScript analysis integration"
)]

use swc_next_allocator::{Allocator, hash_map::HashMap, hash_set::HashSet, vec::ArenaVec};
use swc_next_ecma_ast::{NodeId, ScopeId, SymbolId};

use crate::SymbolTable;

/// Additional TypeScript binding data associated with the existing semantic IDs.
///
/// Each field documents its corresponding tsc-go relationship. This groups the
/// sparse tables for `SymbolTracker`; it does not replace its symbols, scopes,
/// declarations, or references with another binder model.
#[derive(Debug)]
pub(super) struct TsBinder<'a> {
    /// Members owned by a symbol: tsc-go `ast.Symbol.Members()`.
    ///
    /// Class instance and interface members, including their type parameters,
    /// share this name map across merged declarations. Lexical type-parameter
    /// bindings remain in the existing symbol tables for occurrence-local lookup.
    /// Member declarations use the existing symbol and declaration arenas.
    /// This table records declaration identities, not inferred member types.
    /// Type/object literals can use the same table when their members are bound;
    /// namespace exports, enum members, and class statics use `symbol_exports`.
    pub symbol_members: HashMap<'a, SymbolId, SymbolTable<'a>>,

    /// Exported names owned by a symbol: tsc-go `ast.Symbol.Exports()`.
    ///
    /// Each value reuses the existing name-to-symbol map used for lexical bindings.
    /// Namespace declarations sharing an owner share this table, while their local
    /// bindings remain in their separate `symbol_tables`. Enum members and class static
    /// members also belong to exports in Go; lookup applies the appropriate meaning.
    /// Export specifiers point to ordinary alias symbols from the existing symbol
    /// arena. `default` and `export=` are entries in the source module's export map.
    pub symbol_exports: HashMap<'a, SymbolId, SymbolTable<'a>>,

    /// Local symbol to its export symbol: tsc-go `ast.Symbol.ExportSymbol()`.
    ///
    /// Existing scope bindings and declaration records retain the local identity;
    /// this relationship supplies its shared export identity. It is independent of
    /// an import/export alias's resolved target. No second per-node `LocalSymbol`
    /// record is needed alongside the existing binding and declaration indexes.
    pub export_symbols: HashMap<'a, SymbolId, SymbolId>,

    /// A declaration body's scope to its owning symbol.
    ///
    /// Corresponds to reading `container.Symbol()` on a module, enum, class, or
    /// source-file container in Go's binder/name resolver. Here lookup already walks
    /// `ScopeId`s, so this table associates the existing scopes with those owners.
    /// Reopened namespace/enum bodies have distinct scopes but the same owner.
    /// An anonymous default class or a source module can have an owner even though
    /// its scope-creating node has no binding-name entry in `node_semantic`.
    pub container_symbols: HashMap<'a, ScopeId, SymbolId>,

    /// Symbol to owning symbol: tsc-go `ast.Symbol.Parent()`.
    ///
    /// This is declaration ownership, not lexical nesting. For example, `B` in
    /// `namespace A.B` belongs to `A` even when that syntax creates no separate
    /// lexical body scope for `A`. Scope parents continue to describe local lookup.
    pub symbol_parents: HashMap<'a, SymbolId, SymbolId>,

    /// UMD global export names: tsc-go `ast.SourceFile.GlobalExports`.
    ///
    /// A semantic model analyzes one AST, so a single existing name map suffices.
    /// `export as namespace` names are separate from that file's lexical bindings
    /// and ordinary module exports.
    pub global_exports: SymbolTable<'a>,

    /// Cached alias targets: tsc-go `checker.AliasSymbolLinks.aliasTarget`.
    ///
    /// Keys and targets use the existing symbol arena; references keep their alias
    /// binding identity. A missing key means uncomputed. `Some(target)` is the result
    /// of `resolveAlias`; `None` represents Go's `unknownSymbol` result, including
    /// unresolved targets and cycles, without allocating a synthetic symbol here.
    /// A target can retain alias meaning when it also has another declaration kind.
    pub alias_targets: HashMap<'a, SymbolId, Option<SymbolId>>,

    /// Type-only provenance of an alias: tsc-go
    /// `checker.AliasSymbolLinks.typeOnlyDeclaration`.
    ///
    /// Stores the first import/export syntax node that restricts this alias path to
    /// type uses. A target that merely lacks value meaning does not create an entry.
    pub type_only_alias_declarations: HashMap<'a, SymbolId, NodeId>,

    /// Active alias-target resolutions, corresponding to the symbol targets of
    /// `AliasTarget` entries in tsc-go `Checker.typeResolutions`.
    ///
    /// Only alias resolution needs this scratch stack here; the general Go
    /// `TypeResolution` and `TypeSystemPropertyName` objects are unnecessary.
    /// A repeated symbol identifies a cycle; its cached result belongs in
    /// [`Self::alias_targets`]. This stack is not retained in the completed model.
    pub alias_resolution_stack: ArenaVec<'a, SymbolId>,

    /// Traversal-local cache corresponding to the `visited` map in tsc-go
    /// `ast.getModuleInstanceStateCached`.
    ///
    /// Keys are declaration/body nodes, not merged symbols: each declaration can
    /// contribute a different state. A missing entry is unvisited. Clear the map
    /// for each root invocation, as Go's `GetModuleInstanceState` does.
    pub module_instance_states: HashMap<'a, NodeId, ModuleInstanceState>,

    /// Symbols with tsc-go `ast.SymbolFlagsConstEnumOnlyModule`.
    ///
    /// Set membership keeps this cold property outside the existing `SymbolFlags`.
    /// It describes the merged owner, rather than a declaration's traversal
    /// state in [`Self::module_instance_states`]. An instantiated namespace,
    /// function, class, or regular enum contribution prevents const-enum-only
    /// classification. The set does not decide whether an emit option preserves
    /// const enums.
    pub const_enum_only_modules: HashSet<'a, SymbolId>,

    /// Deferred expando declarations: tsc-go `Binder.expandoAssignments`,
    /// processed after the existing scope bindings have been collected.
    pub expando_assignments: ArenaVec<'a, ExpandoAssignmentInfo>,
}

impl<'a> TsBinder<'a> {
    /// Creates empty side tables without allocating their backing storage.
    pub fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            symbol_members: HashMap::new_in(allocator),
            symbol_exports: HashMap::new_in(allocator),
            export_symbols: HashMap::new_in(allocator),
            container_symbols: HashMap::new_in(allocator),
            symbol_parents: HashMap::new_in(allocator),
            global_exports: SymbolTable::new_in(allocator),
            alias_targets: HashMap::new_in(allocator),
            type_only_alias_declarations: HashMap::new_in(allocator),
            alias_resolution_stack: ArenaVec::new_in(allocator),
            module_instance_states: HashMap::new_in(allocator),
            const_enum_only_modules: HashSet::new_in(allocator),
            expando_assignments: ArenaVec::new_in(allocator),
        }
    }
}

/// tsc-go `ast.ModuleInstanceState` in `ast/utilities.go`.
///
/// This cannot be reduced to the existing `VALUE_MODULE` flag: `ConstEnumOnly`
/// has value meaning during binding but does not by itself decide emission.
/// `Unknown` is an active traversal marker, not a completed module state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModuleInstanceState {
    Unknown,
    NonInstantiated,
    Instantiated,
    ConstEnumOnly,
}

/// A deferred expando declaration: tsc-go `binder.ExpandoAssignmentInfo`.
///
/// Only the syntax and lexical environments required for binding are retained;
/// the receiver's symbol is found using the existing symbol tables after traversal.
#[derive(Debug, Clone, Copy)]
pub(super) struct ExpandoAssignmentInfo {
    /// Go's `node`: the assignment or `Object.defineProperty` call.
    pub node: NodeId,
    /// Go's `container.Locals`, represented by its existing lexical scope.
    /// `None` preserves a container without locals, such as an object literal;
    /// it must not silently become the nearest enclosing declaration scope.
    pub container: Option<ScopeId>,
    /// Go's `blockScopeContainer.Locals`, represented by its existing scope.
    pub block_scope_container: ScopeId,
}
