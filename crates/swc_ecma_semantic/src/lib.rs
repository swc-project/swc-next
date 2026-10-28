//! Scope, symbol, reference, and early-error analysis for the flat ECMAScript AST.
//!
//! TypeScript analysis resolves declaration identities, spaces, and visibility.
//! It does not infer expression types or evaluate conditional types. Member
//! lookup uses declarations and direct property-name syntax. Unresolved computed
//! names preserve possible property value contributions without opening type or
//! namespace lookup, inferring member names, or proving declaration conflicts.
//! References expose this uncertainty and retain every possible binding in the
//! use index. Parentheses are transparent when collecting property-name facts,
//! including where TypeScript's own member binding distinguishes their syntax.
//! Merged class/interface type parameters share declaration identity while
//! retaining occurrence-local visibility. This does not model TypeScript's
//! shared identities between type parameters and same-named instance members.
//!
//! Semantic analysis based on
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).
//! Modified for SWC Next.

#![deny(clippy::all)]
#![deny(unused)]

use swc_next_allocator::{
    Idx, IndexSlice,
    hash_map::{HashMap as ArenaHashMap, Values},
    vec::{ArenaIndexVec, ArenaVec},
};
use swc_next_ecma_ast::{Ast, AstWtf8, Diagnostic, NodeId, ScopeId, SymbolId, VisitWith};

use crate::{
    binder::{Range, SymbolLinks, TsMemberIndex},
    builder::SemanticBuilder,
    scope::ScopeTree,
};

mod binder;
mod builder;
mod ecmascript;
pub mod scope;
mod scope_model;
mod syntax_checks;

pub use binder::{
    Declaration, NameResolution, Reference, ReferenceFlags, ReferenceSpace, Symbol, SymbolFlags,
};
pub use scope_model::ScopeModel;
pub use swc_next_ecma_ast::ReferenceId;
pub use syntax_checks::SyntaxChecks;

swc_next_allocator::define_nonmax_u32_index_type! {
    /// Identifier for a declaration in a completed [`Semantic`] model.
    pub struct DeclarationId;
}

type Symbols = IndexSlice<SymbolId, [Symbol]>;
type Declarations = IndexSlice<DeclarationId, [Declaration]>;
type Scopes = IndexSlice<ScopeId, [scope::Scope]>;

// Internal names survive replacement of their scope's ordinary binding.
type InternalNames<'a> = ArenaHashMap<'a, (ScopeId, AstWtf8), SymbolId>;

type ScopeMap<'a> = ArenaHashMap<'a, &'a [u8], SymbolId>;
type ScopeMaps<'a> = IndexSlice<ScopeId, [ScopeMap<'a>]>;
// Most scopes have no var declarations hoisting through them.
// Keep the var declaration, since its symbol may also contain functions.
type HoistingMap<'a> =
    Option<swc_next_allocator::boxed::Box<'a, ArenaHashMap<'a, &'a [u8], DeclarationId>>>;
type Redeclarations<'a> = ArenaHashMap<'a, SymbolId, ArenaVec<'a, DeclarationId>>;

/// JSX syntax whose implicit factory dependencies can be resolved by a consumer.
/// This records syntax and its lexical environment, not a particular JSX runtime.
#[derive(Debug, Clone, Copy)]
pub struct JsxUsage {
    pub scope: ScopeId,
    pub is_fragment: bool,
}

/// The semantic model of a tree, with scopes, symbols, references, and their
/// lookup certainty cross-indexed. Opaque TypeScript facts may leave multiple
/// possible reference targets. Backed by the
/// tree's arena and valid for the lifetime of the tree.
///
/// The query methods connect declaration nodes, reference nodes,
/// containing scopes, uses, and name lookup without rebuilding
/// indexes outside the arena-backed model.
#[derive(Debug)]
pub struct Semantic<'a> {
    /// Every scope, indexed by [`ScopeId`].
    scopes: ScopeTree<'a>,
    jsx_usages: ArenaVec<'a, JsxUsage>,
    /// Every symbol, in declaration order, indexed by [`SymbolId`].
    symbols: ArenaIndexVec<'a, SymbolId, Symbol>,
    symbol_links: SymbolLinks<'a>,
    /// Every reference, in source order, indexed by [`ReferenceId`].
    references: ArenaIndexVec<'a, ReferenceId, Reference>,
    reference_candidates: binder::ReferenceCandidates<'a>,
    /// Every declaration in source order, indexed by [`DeclarationId`].
    declarations: ArenaIndexVec<'a, DeclarationId, Declaration>,
    /// Additional declaration ids for redeclared symbols, in source order.
    redeclarations: Redeclarations<'a>,

    use_ids: ArenaVec<'a, ReferenceId>,
    use_ranges: ArenaIndexVec<'a, SymbolId, Range>,
    scope_maps: ArenaIndexVec<'a, ScopeId, ScopeMap<'a>>,
    internal_names: InternalNames<'a>,
    hoisting_variables: ArenaIndexVec<'a, ScopeId, HoistingMap<'a>>,

    /// Each node records one binding, reference, or created scope identity.
    node_semantic: ArenaIndexVec<'a, NodeId, NodeSemanticId>,

    ts_members: TsMemberIndex<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NodeSemanticId {
    None,
    Symbol(SymbolId),
    Reference(ReferenceId),
    Scope(ScopeId),
}

impl NodeSemanticId {
    #[inline]
    fn symbol(self) -> Option<SymbolId> {
        match self {
            Self::Symbol(id) => Some(id),
            _ => None,
        }
    }

    #[inline]
    fn reference(self) -> Option<ReferenceId> {
        match self {
            Self::Reference(id) => Some(id),
            _ => None,
        }
    }

    #[inline]
    fn scope(self) -> Option<ScopeId> {
        match self {
            Self::Scope(id) => Some(id),
            _ => None,
        }
    }
}

impl<'a> Semantic<'a> {
    /// JSX usage in traversal order, coalescing adjacent sites in one scope.
    /// Intrinsic tags are included; a fragment also requires an element factory.
    pub fn jsx_usages(&self) -> &[JsxUsage] {
        &self.jsx_usages
    }

    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        self.symbols[id]
    }

    /// Symbols linked by JavaScript binding rules.
    ///
    /// Includes `id` exactly once, along with every transitively linked symbol,
    /// in unspecified order. Ordinary symbols yield only themselves. Catch vars,
    /// split parameter/body vars, and Annex B function bindings can couple names
    /// without sharing a binding identity, scope, or declaration flags.
    /// Their declaration names and references must be renamed together.
    ///
    /// This describes name coupling only. The chosen name must still avoid
    /// capture and preserve dynamic names and external interfaces.
    pub fn linked_symbols(&self, id: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.symbol_links.symbols(id)
    }

    /// The reference with the given id.
    pub fn reference(&self, id: ReferenceId) -> Reference {
        self.references[id]
    }

    /// The declaration with the given id.
    pub fn declaration(&self, id: DeclarationId) -> Declaration {
        self.declarations[id]
    }

    /// The scope with the given id.
    pub fn scope(&self, id: ScopeId) -> scope::Scope {
        self.scopes.get(id)
    }

    /// The symbol declared at `node`, or `None` when the node is not a
    /// binding or member declaration site. Source-module and anonymous default
    /// export symbols are queried through the TypeScript export APIs.
    pub fn symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        self.node_semantic[node].symbol()
    }

    /// The reference recorded at `node`, or `None` when the node is
    /// not a reference site.
    pub fn reference_of(&self, node: NodeId) -> Option<ReferenceId> {
        self.node_semantic[node].reference()
    }

    /// The scope created by the given node, or `None` if the node is not a
    /// scope-creating site. If a node creates multiple scopes, returns the
    /// last one created (for example, the program's source-type wrapper).
    pub fn scope_of(&self, node: NodeId) -> Option<ScopeId> {
        self.node_semantic[node].scope()
    }

    /// The TypeScript source module's symbol, or `None` outside a TypeScript module.
    /// The program node records its scope rather than this export-container symbol.
    pub fn ts_source_module_symbol(&self) -> Option<SymbolId> {
        self.ts_members.source_module_symbol()
    }

    /// The symbol shared by TypeScript default export declarations, including
    /// anonymous classes and functions. Expression exports do not create it.
    pub fn ts_default_export_symbol(&self) -> Option<SymbolId> {
        self.ts_members.default_export_symbol()
    }

    /// The binding scope of a symbol in JavaScript semantic analysis only.
    /// Returns the scope of the symbol's first declaration.
    ///
    /// TypeScript symbols can merge declarations across multiple scopes;
    /// use the relevant [`Declaration::scope`] for TypeScript analysis.
    pub fn scope_of_symbol(&self, id: SymbolId) -> ScopeId {
        let declaration = self.symbol(id).declaration;
        self.declaration(declaration).scope
    }

    /// The definite terminal symbol of an internal TypeScript import alias.
    /// Follows forwarding aliases without changing their binding identities.
    /// Returns `None` for non-aliases, opaque or cyclic targets, property-only
    /// targets, and merged contributions without a unique terminal symbol.
    /// The target retains its declaration kind, including `CONST_ENUM`, regardless
    /// of JavaScript emit options. This does not decide runtime access or emission.
    pub fn ts_import_alias_target(&self, alias: SymbolId) -> Option<SymbolId> {
        self.ts_members.import_alias_target(alias)
    }

    /// The initializer's root reference in an internal `import A = N.Member`.
    /// Its access flags account for the whole alias target, not just `N`:
    /// a known type-only target does not read the root, while opaque targets
    /// conservatively retain a read. Ambient and explicit type-only aliases
    /// also have no runtime access. External `require` targets return `None`.
    ///
    /// This describes initializer access, not whether the alias is used or
    /// whether an emit option preserves its target. Use ordinary reference
    /// queries for declaration dependencies even when this reference is unread.
    /// `ast` must be the tree analyzed to produce this model.
    pub fn import_equals_reference(
        &self,
        ast: &Ast<'_>,
        declaration: swc_next_ecma_ast::TsImportEqualsDeclaration,
    ) -> Option<ReferenceId> {
        let node = ecmascript::import_equals_reference_node(ast, declaration)?;
        self.reference_of(node)
    }

    /// Iterates the ids of every declaration of `id`, in source order.
    pub fn declaration_ids(&self, id: SymbolId) -> DeclarationIdIterator<'_> {
        let additional = self
            .redeclarations
            .get(&id)
            .map_or(&[][..], |declarations| declarations.as_slice());
        DeclarationIdIterator {
            first: Some(self.symbols[id].declaration),
            additional: additional.iter().copied(),
        }
    }

    /// Every declaration of `id`, in source order. Usually a single element.
    /// Symbols whose declarations merge (`var` redeclarations, TS overloads,
    /// `class` + `interface`) have one entry per declaration.
    ///
    /// ## Example
    /// ```ts
    /// var a = 1; var a = 2;
    /// //  ^ declarations[0]
    /// //             ^ declarations[1], same symbol
    /// ```
    ///
    /// A conflicting redeclaration is recorded here too, aliased onto
    /// the existing symbol so tooling on broken code still maps the node.
    pub fn declarations(
        &self,
        id: SymbolId,
    ) -> impl ExactSizeIterator<Item = Declaration> + DoubleEndedIterator + '_ {
        self.declaration_ids(id)
            .map(|declaration_id| self.declaration(declaration_id))
    }

    /// Every definite or possible use site of `id`, once each in source order. Declaration sites are
    /// not uses; those are returned by [`Semantic::declarations`].
    /// An uncertain TypeScript reference is included under every candidate,
    /// even when [`Reference::symbol`] names a different, nearer candidate.
    pub fn uses(&self, id: SymbolId) -> &[ReferenceId] {
        let range = self.use_ranges[id];
        let start = range.start as usize;
        &self.use_ids[start..start + range.len as usize]
    }

    /// Possible bindings for this reference, with the nearest candidate first
    /// and any additional targets in declaration order. Each symbol occurs once.
    /// A candidate is not proof that lookup succeeds: an opaque alias may not
    /// contribute to the reference's space. Inspect [`Reference::resolution`]
    /// before an operation that requires a unique target.
    pub fn reference_candidates(&self, id: ReferenceId) -> impl Iterator<Item = SymbolId> + '_ {
        self.reference(id)
            .symbol
            .into_iter()
            .chain(self.reference_candidates.additional(id).iter().copied())
    }

    /// Value bindings associated with a reference's selected bindings.
    /// Includes the alias contributions of a selected public member and follows
    /// local TypeScript ExportValue links without changing lexical identity.
    /// Alias space information determines whether each target is a definite
    /// value or only a possible one. Results are never Unresolved.
    ///
    /// This queries value meaning, not runtime usage: a `typeof` or explicit
    /// type-only export can still name a value. Access flags and emit options
    /// determine whether it is needed at runtime. Import aliases remain bindings;
    /// this does not replace them with their final property or declaration target.
    /// `ast` must be the tree analyzed to produce this model.
    pub fn reference_value_targets<'s>(
        &'s self,
        ast: &'s Ast<'_>,
        id: ReferenceId,
    ) -> impl Iterator<Item = NameResolution> + 's {
        let reference = self.reference(id);
        let name = ast.get_utf8(reference.name).as_bytes();
        let member_values = self.ts_members.reference_values(id);
        let lexical_values = member_values
            .is_none()
            .then_some(())
            .into_iter()
            .flat_map(move |_| self.reference_candidates(id))
            .filter_map(move |symbol| {
                let resolution = if reference.flags.is_uncertain() {
                    NameResolution::Uncertain { candidate: symbol }
                } else {
                    NameResolution::Resolved(symbol)
                };
                self.ts_members.value_target(
                    &self.symbols,
                    &self.declarations,
                    &self.scope_maps,
                    resolution,
                    name,
                    reference.flags.space.value_target_space(),
                )
            });
        member_values
            .into_iter()
            .flatten()
            .copied()
            .chain(lexical_values)
    }

    /// Members declared by any declaration merged into a TypeScript namespace.
    pub fn ts_namespace_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.ts_members.namespace_members(owner)
    }

    /// Finds a member declared by any declaration merged into a TypeScript namespace.
    /// This returns its identity; its declaration flags do not describe its
    /// resolved visibility in every reference space or member view.
    pub fn ts_namespace_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.ts_members.namespace_member(owner, name)
    }

    /// Whether a namespace has export specifiers in addition to declared members.
    /// This includes type-only and unresolved exports; query their value targets
    /// with [`Self::ts_namespace_value_targets`] before using them at runtime.
    pub fn ts_namespace_has_export_aliases(&self, owner: SymbolId) -> bool {
        self.ts_members.namespace_has_export_aliases(owner)
    }

    /// Value bindings exposed by a namespace or enum's public member name.
    /// Includes resolved export-specifier contributions and ExportValue links,
    /// excludes type-only exports, and preserves uncertainty for opaque aliases.
    /// Import aliases retain their binding identities; use [`Self::ts_import_alias_target`]
    /// when the terminal declaration is needed. Contributions may repeat a symbol.
    /// This does not infer class properties or other expression types.
    pub fn ts_namespace_value_targets<'s>(
        &'s self,
        owner: SymbolId,
        name: &'s [u8],
    ) -> impl Iterator<Item = NameResolution> + 's {
        self.ts_members.namespace_value_targets(
            &self.symbols,
            &self.declarations,
            &self.scope_maps,
            owner,
            name,
        )
    }

    /// Members declared by any declaration merged into a TypeScript enum.
    pub fn ts_enum_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.ts_members.enum_members(owner)
    }

    /// Finds a member declared by any declaration merged into a TypeScript enum.
    /// This returns its identity; a merged symbol can also have declarations
    /// whose visibility belongs to a different member view.
    pub fn ts_enum_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.ts_members.enum_member(owner, name)
    }

    /// The lexical binding of `name` at `scope` alone, including a hoisting
    /// `var` passing through on its way to its hoist target. This does not walk
    /// the scope chain; see [`Semantic::lookup`]. TypeScript container members
    /// are owned by their merged container environment and are therefore
    /// queried through [`Semantic::lookup`] or the container APIs instead.
    /// Source-file named exports are returned when no separate local binding
    /// owns the name; use [`Semantic::lookup`] to select a declaration space.
    /// Source-file default declarations likewise expose their local names
    /// through [`Semantic::lookup`], with separate visibility in each space.
    ///
    /// ## Example
    /// ```ts
    /// function f() { { var a; } }
    /// //             ^ binding here finds the occurrence owned by
    /// //               the function scope
    /// ```
    pub fn binding(&self, scope_id: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.scope_maps[scope_id]
            .get(name)
            .copied()
            .or_else(|| {
                self.hoisting_variables[scope_id]
                    .as_deref()
                    .and_then(|bindings| bindings.get(name).copied())
                    .map(|declaration| self.declarations[declaration].symbol)
            })
            .or_else(|| {
                (scope_id == ScopeId::MODULE)
                    .then(|| self.ts_members.source_binding(name))
                    .flatten()
            })
    }

    /// Every lexical symbol owned directly by `scope`. A hoisting `var`
    /// appears in its hoist target, not in blocks it passes through. Merged
    /// TypeScript container members are returned by the container APIs rather
    /// than projected into every declaration body. Source-file default
    /// declarations are also owned by their shared export identity. Named
    /// source-file exports are included when no separate local owns their name.
    pub fn bindings(&self, scope_id: ScopeId) -> BindingIterator<'_> {
        BindingIterator {
            inner: self.scope_maps[scope_id].values(),
            source: if scope_id == ScopeId::MODULE {
                self.ts_members.source_bindings(&self.scope_maps[scope_id])
            } else {
                None
            },
        }
    }

    /// The nearest name-resolution candidate for `name` visible in `space`
    /// from `scope`. At each lexical layer, body-local binding candidates are
    /// considered before TypeScript container member views. A candidate
    /// outside the requested space does not shadow.
    /// [`ReferenceSpace::Any`] matches by name alone. This does not query
    /// runtime object properties when the path crosses [`scope::ScopeKind::With`].
    ///
    /// Unresolved internal TypeScript aliases remain conservative candidates.
    /// A result is not proof of their target's type or of a computed member's
    /// existence. Unknown property names permit value candidates only; they do
    /// not create type or namespace contributions. Consumers must not interpret
    /// missing member facts as absence.
    /// This query does not distinguish definite bindings from possible candidates.
    ///
    /// ## Example
    /// ```ts
    /// type T = string;
    /// function f() {
    ///   const T = 1;
    ///   // type space finds the outer alias, value space the local const
    /// }
    /// ```
    pub fn lookup(
        &self,
        scope_id: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> Option<SymbolId> {
        self.resolve_name(scope_id, name, space).candidate()
    }

    /// Looks up a name and reports whether its target is known or only possible.
    /// Unknown contributions do not prove that an outer binding is shadowed.
    fn resolve_name(
        &self,
        scope_id: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> NameResolution {
        self.ts_members.resolve_name(self, scope_id, name, space)
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
}

/// A `(id, scope)` pair yielded by [`Semantic::iter_scopes`].
pub struct ScopeEntry {
    pub id: ScopeId,
    pub scope: scope::Scope,
}

/// A `(id, symbol)` pair yielded by [`Semantic::iter_symbols`].
pub struct SymbolEntry {
    pub id: SymbolId,
    pub symbol: Symbol,
}

/// A `(id, reference)` pair yielded by [`Semantic::iter_references`].
pub struct ReferenceEntry {
    pub id: ReferenceId,
    pub reference: Reference,
}

/// A `(id, declaration)` pair yielded by [`Semantic::iter_declarations`].
pub struct DeclarationEntry {
    pub id: DeclarationId,
    pub declaration: Declaration,
}

/// Declaration ids for one symbol, in source order.
#[derive(Clone)]
pub struct DeclarationIdIterator<'a> {
    first: Option<DeclarationId>,
    additional: std::iter::Copied<std::slice::Iter<'a, DeclarationId>>,
}

impl Iterator for DeclarationIdIterator<'_> {
    type Item = DeclarationId;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| self.additional.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for DeclarationIdIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.additional.next_back().or_else(|| self.first.take())
    }
}

impl ExactSizeIterator for DeclarationIdIterator<'_> {
    fn len(&self) -> usize {
        usize::from(self.first.is_some()) + self.additional.len()
    }
}

impl std::iter::FusedIterator for DeclarationIdIterator<'_> {}

pub struct ScopeIterator<'a> {
    list: &'a Scopes,
    index: u32,
}

impl Iterator for ScopeIterator<'_> {
    type Item = ScopeEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let index = self.index as usize;
            let scope = *self.list.raw.get(index)?;
            self.index += 1;
            if scope.active {
                return Some(ScopeEntry {
                    id: ScopeId::from_usize(index),
                    scope,
                });
            }
        }
    }
}

pub struct BindingIterator<'a> {
    inner: Values<'a, &'a [u8], SymbolId>,
    source: Option<binder::TsSourceBindingIterator<'a>>,
}

impl Iterator for BindingIterator<'_> {
    type Item = SymbolId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .copied()
            .or_else(|| self.source.as_mut()?.next())
    }
}

pub struct SymbolIterator<'a> {
    symbols: &'a Symbols,
    index: u32,
}

impl Iterator for SymbolIterator<'_> {
    type Item = SymbolEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let symbol = *self.symbols.raw.get(index)?;
        self.index += 1;
        Some(SymbolEntry {
            id: SymbolId::from_usize(index),
            symbol,
        })
    }
}

pub struct ReferenceIterator<'a> {
    references: &'a IndexSlice<ReferenceId, [Reference]>,
    node_semantic: &'a IndexSlice<NodeId, [NodeSemanticId]>,
    index: u32,
}

pub struct DeclarationIterator<'a> {
    declarations: &'a Declarations,
    index: u32,
}

impl Iterator for ReferenceIterator<'_> {
    type Item = ReferenceEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let index = self.index as usize;
            let reference = *self.references.raw.get(index)?;
            self.index += 1;
            let id = ReferenceId::from_usize(index);
            if self.node_semantic[reference.node].reference().is_some() {
                return Some(ReferenceEntry { id, reference });
            }
        }
    }
}

impl Iterator for DeclarationIterator<'_> {
    type Item = DeclarationEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let declaration = *self.declarations.raw.get(index)?;
        self.index += 1;
        Some(DeclarationEntry {
            id: DeclarationId::from_usize(index),
            declaration,
        })
    }
}

/// Semantic analysis output and any requested scope-dependent early errors.
pub struct SemanticReturn<'a> {
    pub semantic: Semantic<'a>,
    pub diagnostics: Vec<Diagnostic<'a>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AnalyzeOptions {
    /// Which scope-dependent syntax errors to check and report.
    ///
    /// Disabled by default. Use [`SyntaxChecks::all`] to enable every check.
    pub check_syntax: SyntaxChecks,
}

/// Runs semantic analysis on a tree and builds the requested [`Semantic`] model.
///
/// The checks selected by [`AnalyzeOptions::check_syntax`] report scope-dependent early
/// errors separately from parser diagnostics. Their messages use
/// the tree's arena, so the result is valid as long as the tree is alive.
pub fn analyze<'a>(ast: &Ast<'a>, options: AnalyzeOptions) -> SemanticReturn<'a> {
    let root = ast.root_program();
    let mut builder = SemanticBuilder::new(ast, options.check_syntax);
    root.visit_with(&mut builder);
    builder.build()
}

/// Checks scope-dependent syntax errors without building a [`Semantic`] model.
///
/// Returns the same diagnostics as [`analyze`] with
/// [`AnalyzeOptions::check_syntax`] set to `checks`. Use [`SyntaxChecks::all`]
/// to include every check, including TypeScript aliases.
/// Reference resolution and the model's use indexes are not needed for these
/// checks and are skipped. Messages borrow the tree's arena.
pub fn check_syntax<'a>(ast: &Ast<'a>, checks: SyntaxChecks) -> Vec<Diagnostic<'a>> {
    if checks.is_empty() {
        return Vec::new();
    }
    let mut builder = SemanticBuilder::new(ast, checks);
    ast.root_program().visit_with(&mut builder);
    builder.into_diagnostics()
}
