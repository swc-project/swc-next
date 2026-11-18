//! Scope, symbol, reference, and early-error analysis for the flat ECMAScript AST.
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
use swc_next_ecma_ast::{Ast, AstCtx, Diagnostic, NodeId, ScopeId, SymbolId, VisitWith};

use crate::{
    binder::{Range, SymbolLinks, ts::LexicalLookup},
    builder::SemanticBuilder,
    scope::ScopeTree,
    ts::TsBindings,
};

mod binder;
mod builder;
mod ecmascript;
mod scope;
mod scope_model;
mod syntax_checks;
mod ts;

pub use binder::{Declaration, Reference, ReferenceFlags, ReferenceSpace, Symbol, SymbolFlags};
pub use scope::{Scope, ScopeFlags, ScopeKind};
pub use scope_model::ScopeModel;
pub use swc_next_ecma_ast::ReferenceId;
pub use syntax_checks::SyntaxChecks;
pub use ts::{TsImportAlias, TsImportAliasSpace, TsMember};

swc_next_allocator::define_nonmax_u32_index_type! {
    /// Identifier for a declaration in a completed [`Semantic`] model.
    pub struct DeclarationId;
}

type Symbols = IndexSlice<SymbolId, [Symbol]>;
type Declarations = IndexSlice<DeclarationId, [Declaration]>;
type Scopes = IndexSlice<ScopeId, [scope::Scope]>;

type SymbolTable<'a> = ArenaHashMap<'a, &'a [u8], SymbolId>;
type SymbolTables<'a> = IndexSlice<ScopeId, [SymbolTable<'a>]>;
// Maps a replacement binding to the internal name it hides in the same scope.
type ShadowedInternalNames<'a> = ArenaHashMap<'a, SymbolId, SymbolId>;
// Keep the var declaration, since its symbol may also contain functions.
type HoistingMap<'a> = ArenaHashMap<'a, &'a [u8], DeclarationId>;
type Redeclarations<'a> = ArenaHashMap<'a, SymbolId, ArenaVec<'a, DeclarationId>>;

/// The semantic model of a tree, with scopes, symbols, declarations, and
/// references cross-indexed. Backed by the tree's arena and valid for the
/// lifetime of the tree.
///
/// The query methods connect declaration nodes, reference nodes,
/// containing scopes, uses, and name lookup without rebuilding
/// indexes outside the arena-backed model.
#[derive(Debug)]
pub struct Semantic<'a> {
    /// Every scope, indexed by [`ScopeId`].
    scopes: ScopeTree<'a>,
    /// Every symbol, in declaration order, indexed by [`SymbolId`].
    symbols: ArenaIndexVec<'a, SymbolId, Symbol>,
    symbol_links: SymbolLinks<'a>,
    /// Every reference, in source order, indexed by [`ReferenceId`].
    references: ArenaIndexVec<'a, ReferenceId, Reference>,
    /// Every declaration in source order, indexed by [`DeclarationId`].
    declarations: ArenaIndexVec<'a, DeclarationId, Declaration>,
    /// Additional declaration ids for redeclared symbols, in source order.
    redeclarations: Redeclarations<'a>,

    use_ids: ArenaVec<'a, ReferenceId>,
    use_ranges: ArenaIndexVec<'a, SymbolId, Range>,
    symbol_tables: ArenaIndexVec<'a, ScopeId, SymbolTable<'a>>,
    shadowed_internal_names: ShadowedInternalNames<'a>,
    hoisting_variables: ArenaIndexVec<'a, ScopeId, HoistingMap<'a>>,

    /// Each node records one binding, reference, or created scope identity.
    node_semantic: ArenaIndexVec<'a, NodeId, NodeSemanticId>,

    ts_bindings: TsBindings<'a>,
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
    /// binding declaration site.
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

    /// The binding scope of the symbol's first declaration.
    pub fn scope_of_symbol(&self, id: SymbolId) -> ScopeId {
        let declaration = self.symbol(id).declaration;
        self.declaration(declaration).scope
    }

    /// Iterates the ids of every declaration of `id`, in source order.
    pub fn declaration_ids(&self, id: SymbolId) -> DeclarationIdIterator<'_> {
        let additional = match self.redeclarations.get(&id) {
            Some(declarations) => declarations.as_slice(),
            None => &[],
        };
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
    /// A TypeScript export and its local binding share declaration IDs.
    /// The declaration's symbol is the canonical export.
    pub fn declarations(
        &self,
        id: SymbolId,
    ) -> impl ExactSizeIterator<Item = Declaration> + DoubleEndedIterator + '_ {
        self.declaration_ids(id)
            .map(|declaration_id| self.declaration(declaration_id))
    }

    /// Every definite or possible use of `id`, in source order. Declaration
    /// sites are returned by [`Semantic::declarations`].
    pub fn uses(&self, id: SymbolId) -> &[ReferenceId] {
        let range = self.use_ranges[id];
        let start = range.start as usize;
        &self.use_ids[start..start + range.len as usize]
    }

    /// The binding of `name` at `scope` alone, including a hoisting
    /// `var` passing through on its way to its hoist target. This does
    /// not walk the scope chain; see [`Semantic::lookup`].
    ///
    /// ## Example
    /// ```ts
    /// function f() { { var a; } }
    /// //             ^ binding here finds `a`, whose symbol
    /// //               lives in the function scope
    /// ```
    pub fn binding(&self, scope_id: ScopeId, name: &[u8]) -> Option<SymbolId> {
        if let Some(&symbol) = self.symbol_tables[scope_id].get(name) {
            return Some(symbol);
        }
        let declaration = self.hoisting_variables[scope_id].get(name)?;
        Some(self.declarations[*declaration].symbol)
    }

    /// Every symbol declared directly in `scope`. A hoisting `var`
    /// appears in its hoist target, not in blocks it passes through.
    pub fn bindings(&self, scope_id: ScopeId) -> BindingIterator<'_> {
        BindingIterator {
            inner: self.symbol_tables[scope_id].values(),
        }
    }

    /// The nearest lexical fallback binding of `name` visible in `space`
    /// from `scope`. A binding outside the space does not shadow.
    /// [`ReferenceSpace::Any`] matches by name alone. This does not query
    /// runtime object properties when the path crosses [`scope::ScopeKind::With`].
    /// Import bindings use their declared flags. For a reference in the AST, use
    /// [`Semantic::reference`] and [`Semantic::reference_candidates`] to account
    /// for the meanings and restrictions of its alias path.
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
        let lookup = LexicalLookup {
            scopes: &self.scopes.list,
            symbols: &self.symbols,
            symbol_tables: &self.symbol_tables,
            shadowed_internal_names: &self.shadowed_internal_names,
            ts: &self.ts_bindings,
        };
        lookup.lookup(scope_id, name, space)
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
        match self.first.take() {
            Some(first) => Some(first),
            None => self.additional.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for DeclarationIdIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.additional.next_back() {
            Some(last) => Some(last),
            None => self.first.take(),
        }
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
}

impl Iterator for BindingIterator<'_> {
    type Item = SymbolId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
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

mod sealed {
    use swc_next_ecma_ast::AstCtx;

    pub trait Sealed {}

    impl Sealed for AstCtx<'_> {}
    impl Sealed for &AstCtx<'_> {}
    impl Sealed for &mut AstCtx<'_> {}
}

/// An owned context or a shared or exclusive borrow accepted by [`crate::analyze`].
///
/// Implementations are restricted to `AstCtx`, `&AstCtx`, and `&mut AstCtx`, so analysis
/// retains exactly the context and access permission supplied by its caller.
pub trait AstBorrow<'a>: sealed::Sealed {
    /// Returns the borrowed context without permitting mutation.
    fn as_ast_ctx(&self) -> &AstCtx<'a>;
}

impl<'a> AstBorrow<'a> for AstCtx<'a> {
    fn as_ast_ctx(&self) -> &AstCtx<'a> {
        self
    }
}

impl<'a> AstBorrow<'a> for &AstCtx<'a> {
    fn as_ast_ctx(&self) -> &AstCtx<'a> {
        self
    }
}

impl<'a> AstBorrow<'a> for &mut AstCtx<'a> {
    fn as_ast_ctx(&self) -> &AstCtx<'a> {
        self
    }
}

/// An owned or borrowed AST together with its current semantic model.
///
/// Only [`analyze`] can create this binding. Owned contexts and both forms of
/// borrow permit queries while keeping the AST unchanged. Only a binding with
/// an exclusive borrow can currently be consumed by an in-place transform.
pub struct SemanticAst<'a, A: AstBorrow<'a>> {
    ast: A,
    semantic: Semantic<'a>,
}

impl<'a, A: AstBorrow<'a>> SemanticAst<'a, A> {
    /// Returns the analyzed AST without permitting mutation.
    pub fn ctx(&self) -> &AstCtx<'a> {
        self.ast.as_ast_ctx()
    }

    /// Returns the current semantic model for queries.
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    /// Consumes the binding and returns its original context or borrow and semantic model.
    ///
    /// Subsequent AST mutations may invalidate the detached model. A new
    /// binding must be obtained by analyzing the context again.
    pub fn into_parts(self) -> (A, Semantic<'a>) {
        (self.ast, self.semantic)
    }
}

/// Semantic analysis output and any requested scope-dependent early errors.
pub struct SemanticReturn<'a, A: AstBorrow<'a>> {
    pub semantic: SemanticAst<'a, A>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AnalyzeOptions {
    /// Which scope-dependent syntax errors to check and report.
    ///
    /// Disabled by default. Use [`SyntaxChecks::all`] to enable every check.
    pub check_syntax: SyntaxChecks,
}

/// Analyzes a tree and binds its current [`Semantic`] model to the supplied context or borrow.
///
/// Pass `AstCtx` to transfer ownership, `&AstCtx` for queries, or `&mut AstCtx` to
/// reserve exclusive access for a subsequent in-place transform. Analysis itself
/// only reads the AST.
/// Reborrow an existing mutable reference with `&mut *ast` if it will be used
/// again after consuming the analysis binding.
///
/// The checks selected by [`AnalyzeOptions::check_syntax`] report scope-dependent early
/// errors separately from parser diagnostics. Diagnostics do not borrow the tree.
pub fn analyze<'a, A: AstBorrow<'a>>(ast: A, options: AnalyzeOptions) -> SemanticReturn<'a, A> {
    // SAFETY: Analysis only reads handles obtained from this AST.
    let tree = unsafe { ast.as_ast_ctx().ast() };
    let (semantic, diagnostics) = analyze_impl(tree, options);
    SemanticReturn {
        semantic: SemanticAst { ast, semantic },
        diagnostics,
    }
}

// Keep AST traversal non-generic over the context's ownership or borrow form.
fn analyze_impl<'a>(tree: &Ast<'a>, options: AnalyzeOptions) -> (Semantic<'a>, Vec<Diagnostic>) {
    let root = tree.root_program();
    let mut builder = SemanticBuilder::new(tree, options.check_syntax);
    root.visit_with(&mut builder);
    builder.build()
}

/// Checks scope-dependent syntax errors without building a [`Semantic`] model.
///
/// Returns the same diagnostics as [`analyze`] with
/// [`AnalyzeOptions::check_syntax`] set to `checks`. Use [`SyntaxChecks::all`]
/// to include every check.
/// Reference resolution and the model's use indexes are not needed for these
/// checks and are skipped. Diagnostics do not borrow the tree.
pub fn check_syntax(ast: &AstCtx<'_>, checks: SyntaxChecks) -> Vec<Diagnostic> {
    if checks.is_empty() {
        return Vec::new();
    }
    // SAFETY: Syntax checking only reads handles obtained from this AST.
    let ast = unsafe { ast.ast() };
    let mut builder = SemanticBuilder::new(ast, checks);
    ast.root_program().visit_with(&mut builder);
    builder.into_diagnostics()
}
