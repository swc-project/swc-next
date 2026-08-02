//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! This crate is a Rust port of the semantic-analysis design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

#![deny(clippy::all)]
#![deny(unused)]

use oxc_index::Idx;
use swc_next_allocator::{
    hash_map::{HashMap as ArenaHashMap, Values},
    vec::Vec as ArenaVec,
};
use swc_next_ecma_ast::{
    Ast, Diagnostic, NodeId, Program, ScopeId, SymbolId, TraverseCtx, TraverseWith,
};

use crate::{binder::Range, checker::Checker, context::SemanticContext, scope::ScopeTree};

mod binder;
mod checker;
mod context;
mod ecmascript;
pub mod scope;

pub use binder::{Reference, ReferenceFlags, ReferenceSpace, Symbol, SymbolFlags};
pub use swc_next_ecma_ast::ReferenceId;

type ScopeMap<'a> = ArenaHashMap<'a, &'a [u8], SymbolId>;

/// The complete semantic model of a tree, with every scope, symbol,
/// and reference fully resolved and cross-indexed. Backed by the
/// tree's arena and valid for the lifetime of the tree.
///
/// The query methods connect declaration nodes, reference nodes,
/// containing scopes, parents, uses, and name lookup without rebuilding
/// indexes outside the arena-backed model.
#[derive(Debug)]
pub struct Semantic<'a> {
    /// Every scope, indexed by [`ScopeId`].
    scopes: ScopeTree<'a>,
    /// Every symbol, in declaration order, indexed by [`SymbolId`].
    symbols: ArenaVec<'a, Symbol>,
    /// Every reference, in source order, indexed by [`ReferenceId`].
    references: ArenaVec<'a, Reference>,
    decl_nodes: ArenaVec<'a, NodeId>,
    use_ids: ArenaVec<'a, ReferenceId>,
    use_ranges: ArenaVec<'a, Range>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    hoisting_variables: ArenaVec<'a, ScopeMap<'a>>,
    node_scopes: ArenaVec<'a, ScopeId>,
    node_parents: ArenaVec<'a, Option<NodeId>>,
    node_symbols: ArenaVec<'a, Option<SymbolId>>,
    node_references: ArenaVec<'a, Option<ReferenceId>>,
}

impl<'a> Semantic<'a> {
    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        assert!(id.index() < self.symbols.len());
        self.symbols[id.index()]
    }

    /// The reference with the given id.
    pub fn reference(&self, id: ReferenceId) -> Reference {
        assert!(id.index() < self.references.len());
        self.references[id.index()]
    }

    /// The scope with the given id.
    pub fn scope(&self, id: ScopeId) -> scope::Scope {
        self.scopes.get(id)
    }

    /// The symbol declared at `node`, or the symbol the reference at
    /// `node` resolves to. `None` when the node neither declares nor
    /// references a binding.
    ///
    /// ## Example
    /// ```ts
    /// let a = 1; a;
    /// //  ^ the declared symbol
    /// //         ^ the same symbol, through the reference
    /// ```
    pub fn symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        assert!(node.index() < self.node_symbols.len());
        self.node_symbols[node.index()]
            .or_else(|| self.node_references[node.index()].and_then(|id| self.reference(id).symbol))
    }

    /// The reference recorded at `node`, or `None` when the node is
    /// not a reference site.
    pub fn reference_of(&self, node: NodeId) -> Option<ReferenceId> {
        assert!(node.index() < self.node_references.len());
        self.node_references[node.index()]
    }

    /// The innermost lexical scope containing `node`. A scope-creating
    /// node (function, block, class, ...) maps to the scope it creates;
    /// that scope's parent is the enclosing scope.
    pub fn scope_of(&self, node: NodeId) -> ScopeId {
        assert!(node.index() < self.node_scopes.len());
        self.node_scopes[node.index()]
    }

    /// The structural parent of `node`, or `None` at the root.
    pub fn parent_of(&self, node: NodeId) -> Option<NodeId> {
        assert!(node.index() < self.node_parents.len());
        self.node_parents[node.index()]
    }

    /// Walks from `node` up to the root, yielding `node` first.
    pub fn ancestors(&self, node: Option<NodeId>) -> SemanticAncestorIterator<'_> {
        assert!(node.is_none_or(|node| node.index() < self.node_parents.len()));
        SemanticAncestorIterator {
            node_parents: &self.node_parents,
            current: node,
        }
    }

    /// The `binding_identifier` node of every declaration of `id`, in
    /// source order. Usually a single element. Symbols whose declarations
    /// merge (`var` redeclarations, TS overloads, `class` + `interface`)
    /// have one entry per declaration. [`Semantic::parent_of`] reaches
    /// the enclosing declarator or declaration from there.
    ///
    /// ## Example
    /// ```ts
    /// var a = 1; var a = 2;
    /// //  ^ decls[0]
    /// //             ^ decls[1], same symbol
    /// ```
    ///
    /// A conflicting redeclaration is recorded here too, aliased onto
    /// the existing symbol so tooling on broken code still maps the node.
    pub fn decls(&self, id: SymbolId) -> &[NodeId] {
        let range = self.symbol(id).decls;
        let start = range.start as usize;
        let end = start + range.len as usize;
        assert!(end <= self.decl_nodes.len());
        &self.decl_nodes[start..end]
    }

    /// Every use site of `id`, in source order. Declaration sites are
    /// not uses; those are returned by [`Semantic::decls`].
    pub fn uses(&self, id: SymbolId) -> &[ReferenceId] {
        assert!(id.index() < self.use_ranges.len());
        let range = self.use_ranges[id.index()];
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
        assert!(scope_id.index() < self.scope_maps.len());
        self.scope_maps[scope_id.index()]
            .get(name)
            .copied()
            .or_else(|| self.hoisting_variables[scope_id.index()].get(name).copied())
    }

    /// Every symbol declared directly in `scope`. A hoisting `var`
    /// appears in its hoist target, not in blocks it passes through.
    pub fn bindings(&self, scope_id: ScopeId) -> BindingIterator<'_> {
        assert!(scope_id.index() < self.scope_maps.len());
        BindingIterator {
            inner: self.scope_maps[scope_id.index()].values(),
        }
    }

    /// The nearest binding of `name` visible in `space` from `scope`,
    /// walking the scope chain the way reference resolution does. A
    /// binding outside the space does not shadow. [`ReferenceSpace::Any`]
    /// matches by name alone.
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
        for ancestor in self.scopes.ancestors(Some(scope_id)) {
            let Some(&id) = self.scope_maps[ancestor.index()].get(name) else {
                continue;
            };
            if self.symbol(id).flags.visible_in(space) {
                return Some(id);
            }
        }
        None
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

pub struct ScopeIterator<'a> {
    list: &'a [scope::Scope],
    index: u32,
}

impl Iterator for ScopeIterator<'_> {
    type Item = ScopeEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let scope = *self.list.get(index)?;
        self.index += 1;
        Some(ScopeEntry {
            id: ScopeId::from_usize(index),
            scope,
        })
    }
}

pub struct SemanticAncestorIterator<'a> {
    node_parents: &'a [Option<NodeId>],
    current: Option<NodeId>,
}

impl Iterator for SemanticAncestorIterator<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        assert!(node.index() < self.node_parents.len());
        self.current = self.node_parents[node.index()];
        Some(node)
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
    symbols: &'a [Symbol],
    index: u32,
}

impl Iterator for SymbolIterator<'_> {
    type Item = SymbolEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let symbol = *self.symbols.get(index)?;
        self.index += 1;
        Some(SymbolEntry {
            id: SymbolId::from_usize(index),
            symbol,
        })
    }
}

pub struct ReferenceIterator<'a> {
    references: &'a [Reference],
    index: u32,
}

impl Iterator for ReferenceIterator<'_> {
    type Item = ReferenceEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let reference = *self.references.get(index)?;
        self.index += 1;
        Some(ReferenceEntry {
            id: ReferenceId::from_usize(index),
            reference,
        })
    }
}

/// Semantic analysis output and its scope-dependent early errors.
pub struct SemanticReturn<'a> {
    pub semantic: Semantic<'a>,
    pub diagnostics: Vec<Diagnostic<'a>>,
}

/// Runs semantic analysis on a tree. Builds the complete `Semantic`
/// model and reports the scope-dependent early errors.
///
/// Diagnostics are returned separately from parser diagnostics. Their
/// messages use the tree's arena, so the result is valid as long as the
/// tree is alive.
pub fn analyze<'a>(root: Program, ast: &mut Ast<'a>) -> SemanticReturn<'a> {
    let mut visitor = Checker::new(ast.allocator());
    let state = SemanticContext::init(root, ast);
    let mut context = TraverseCtx::new(ast, state);
    let _ = root.traverse_with(&mut visitor, &mut context);
    let sem = context.state.finalize(context.ast);

    visitor.check_unresolved_exports(ast, &sem);
    SemanticReturn {
        semantic: sem,
        diagnostics: visitor.into_diagnostics(),
    }
}
