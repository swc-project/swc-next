//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! This crate is a Rust port of the semantic-analysis design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

#![deny(clippy::all)]
#![deny(unused)]

mod analyzer;
mod module_record;
mod scope;
mod symbol;

use oxc_index::Idx;

pub use module_record::{ExportRecord, ImportRecord, ModuleRecord, ModuleRecordNameKind};
pub use scope::{Scope, ScopeFlags, ScopeKind, ScopeTree};
pub use swc_experimental_ecma_ast::{Diagnostic, Label, ScopeId, Severity, SymbolId};
pub use symbol::{
    Reference, ReferenceFlags, ReferenceId, ReferenceKind, Symbol, SymbolFlags, SymbolTable,
};

use swc_experimental_ecma_ast::{
    Ast, NodeId, NodeKind, Program, Traverse, TraverseAction, TraverseCtx, TraverseWith,
};

/// Complete semantic information for one parsed source file.
///
/// The model contains every scope, symbol, and reference, fully resolved and
/// cross-indexed. It is a snapshot of the AST at analysis time; run
/// [`analyze`] again after mutating bindings or scopes.
#[derive(Debug)]
pub struct SemanticModel {
    /// Every lexical scope, indexed by [`ScopeId`].
    pub scopes: ScopeTree,
    /// Every symbol and reference, indexed by their dense IDs.
    pub symbols: SymbolTable,
    /// Innermost lexical scope containing each AST node.
    pub node_scopes: Vec<ScopeId>,
    /// Symbol declared by or resolved at each node.
    pub node_symbols: Vec<Option<SymbolId>>,
    /// Reference recorded at each node.
    pub node_references: Vec<Option<ReferenceId>>,
    /// Pre-order traversal index for each node.
    pub node_order: Vec<u64>,
    module_record: ModuleRecord,
    diagnostics: Vec<Diagnostic<'static>>,
}

pub type Semantic = SemanticModel;

impl SemanticModel {
    /// Returns the complete scope tree.
    pub fn scope_tree(&self) -> &ScopeTree {
        &self.scopes
    }

    /// Returns the complete symbol and reference table.
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbols
    }

    /// Returns the source file's import and export records.
    pub fn module_record(&self) -> &ModuleRecord {
        &self.module_record
    }

    /// Returns semantic early-error diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic<'static>] {
        &self.diagnostics
    }

    /// Returns the innermost lexical scope containing `node`.
    ///
    /// A scope-creating node maps to the scope it creates.
    pub fn node_scope(&self, node: NodeId) -> ScopeId {
        self.node_scopes[node.index()]
    }

    /// Returns the symbol declared by or resolved at `node`.
    ///
    /// Returns `None` when the node neither declares nor references a binding.
    pub fn node_symbol(&self, node: NodeId) -> Option<SymbolId> {
        self.node_symbols[node.index()]
    }

    /// Returns the reference recorded at `node`, if it is a reference site.
    pub fn node_reference(&self, node: NodeId) -> Option<ReferenceId> {
        self.node_references[node.index()]
    }

    pub fn node_order(&self, node: NodeId) -> u64 {
        self.node_order[node.index()]
    }

    /// Iterates every scope ID in creation order.
    pub fn scope_ids(&self) -> impl ExactSizeIterator<Item = ScopeId> + '_ {
        (0..self.scopes.len()).map(ScopeId::from_usize)
    }

    /// Iterates every symbol ID in declaration order.
    pub fn symbol_ids(&self) -> impl ExactSizeIterator<Item = SymbolId> + '_ {
        (0..self.symbols.symbol_len()).map(SymbolId::from_usize)
    }

    /// Iterates every reference ID in source order.
    pub fn reference_ids(&self) -> impl ExactSizeIterator<Item = ReferenceId> + '_ {
        (0..self.symbols.reference_len()).map(ReferenceId::from_usize)
    }

    /// Returns the scope with the given ID.
    pub fn scope(&self, id: ScopeId) -> &Scope {
        self.scopes.get(id)
    }

    /// Returns the symbol with the given ID.
    pub fn symbol(&self, id: SymbolId) -> &Symbol {
        self.symbols.symbol(id)
    }

    /// Returns the reference with the given ID.
    pub fn reference(&self, id: ReferenceId) -> &Reference {
        self.symbols.reference(id)
    }

    /// Returns every declaration node for `id`, in source order.
    ///
    /// Symbols formed by legal declaration merging have more than one entry.
    pub fn symbol_declarations(&self, id: SymbolId) -> &[NodeId] {
        &self.symbol(id).declarations
    }

    /// Returns every use of `id`, in source order.
    pub fn references_of(&self, id: SymbolId) -> &[ReferenceId] {
        &self.symbol(id).references
    }

    /// Resolves `name` from `scope` by walking outward through lexical scopes.
    pub fn resolve_name(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.symbols.resolve(&self.scopes, scope, name)
    }

    /// Returns whether renaming the symbol can be performed without dynamic
    /// name-resolution hazards.
    pub fn symbol_rename_safe(&self, id: SymbolId) -> bool {
        let symbol = self.symbol(id);
        !self.scope(symbol.scope).rename_unsafe
            && !symbol.flags.intersects(SymbolFlags::ANNEX_B_FUNCTION)
    }

    /// Returns whether `descendant` is nested inside `ancestor`.
    pub fn is_descendant_scope(&self, descendant: ScopeId, ancestor: ScopeId) -> bool {
        self.scopes
            .ancestors(descendant)
            .any(|scope| scope == ancestor)
    }

    /// Returns whether a symbol is referenced inside the given scope subtree.
    pub fn symbol_used_in_subtree(&self, symbol: SymbolId, scope: ScopeId) -> bool {
        self.references_of(symbol)
            .iter()
            .any(|reference| self.is_descendant_scope(self.reference(*reference).scope, scope))
    }

    /// Returns the source file's top-level scope.
    pub fn top_level_scope_id(&self) -> ScopeId {
        self.scopes.top_level_scope_id()
    }
}

/// Analyze a typed flat AST and write resolved symbol IDs back to identifier nodes.
pub fn analyze<'a>(program: Program, ast: &mut Ast<'a>) -> SemanticModel {
    let mut parts = analyzer::analyze_program(program, ast);
    let module_record = module_record::collect(
        program,
        ast,
        &parts.scopes,
        &parts.symbols,
        &mut parts.diagnostics,
    );
    let mut node_symbols = vec![None; ast.node_count()];
    let mut node_references = vec![None; ast.node_count()];
    for (symbol_id, symbol) in parts.symbols.symbols() {
        for declaration in &symbol.declarations {
            node_symbols[declaration.index()] = Some(symbol_id);
        }
    }
    for (reference_id, reference) in parts.symbols.references() {
        node_references[reference.node.index()] = Some(reference_id);
        if let Some(symbol_id) = reference.symbol_id {
            node_symbols[reference.node.index()] = Some(symbol_id);
        }
    }
    let node_order = build_traversal_order(program, ast);
    SemanticModel {
        scopes: parts.scopes,
        symbols: parts.symbols,
        node_scopes: parts.node_scopes,
        node_symbols,
        node_references,
        node_order,
        module_record,
        diagnostics: parts.diagnostics,
    }
}

fn build_traversal_order(program: Program, ast: &mut Ast<'_>) -> Vec<u64> {
    struct Order {
        node_order: Vec<u64>,
        next: u64,
    }

    impl<'a> Traverse<'a> for Order {
        fn enter_node(
            &mut self,
            node_id: NodeId,
            _kind: NodeKind,
            _ctx: &mut TraverseCtx<'_, 'a>,
        ) -> TraverseAction {
            self.node_order[node_id.index()] = self.next;
            self.next += 1;
            TraverseAction::Proceed
        }
    }

    let mut order = Order {
        node_order: vec![u64::MAX; ast.node_count()],
        next: 0,
    };
    let _ = program.traverse_with(&mut order, &mut TraverseCtx::new(ast, ()));
    order.node_order
}
