//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.

#![deny(clippy::all)]
#![deny(unused)]

mod analyzer;
mod diagnostic;
mod module_record;
mod scope;
mod symbol;

use oxc_index::Idx;

pub use diagnostic::{Diagnostic, Label, Severity};
pub use module_record::{ExportRecord, ImportRecord, ModuleRecord, ModuleRecordNameKind};
pub use scope::{Scope, ScopeFlags, ScopeKind, ScopeTree};
pub use swc_experimental_ecma_ast::{ScopeId, SymbolId};
pub use symbol::{
    Reference, ReferenceFlags, ReferenceId, ReferenceKind, Symbol, SymbolFlags, SymbolTable,
};

use swc_experimental_ecma_ast::{
    Ast, NodeId, NodeKind, Program, Traverse, TraverseAction, TraverseCtx, TraverseWith,
};

/// Complete semantic information for one parsed source file.
#[derive(Debug)]
pub struct SemanticModel {
    pub scopes: ScopeTree,
    pub symbols: SymbolTable,
    pub node_scopes: Vec<ScopeId>,
    pub node_symbols: Vec<Option<SymbolId>>,
    pub node_references: Vec<Option<ReferenceId>>,
    pub node_order: Vec<u64>,
    module_record: ModuleRecord,
    diagnostics: Vec<Diagnostic>,
}

pub type Semantic = SemanticModel;

impl SemanticModel {
    pub fn scope_tree(&self) -> &ScopeTree {
        &self.scopes
    }

    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbols
    }

    pub fn module_record(&self) -> &ModuleRecord {
        &self.module_record
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn node_scope(&self, node: NodeId) -> ScopeId {
        self.node_scopes[node.index()]
    }

    pub fn node_symbol(&self, node: NodeId) -> Option<SymbolId> {
        self.node_symbols[node.index()]
    }

    pub fn node_reference(&self, node: NodeId) -> Option<ReferenceId> {
        self.node_references[node.index()]
    }

    pub fn node_order(&self, node: NodeId) -> u64 {
        self.node_order[node.index()]
    }

    pub fn scope_ids(&self) -> impl ExactSizeIterator<Item = ScopeId> + '_ {
        (0..self.scopes.len()).map(ScopeId::from_usize)
    }

    pub fn symbol_ids(&self) -> impl ExactSizeIterator<Item = SymbolId> + '_ {
        (0..self.symbols.symbol_len()).map(SymbolId::from_usize)
    }

    pub fn reference_ids(&self) -> impl ExactSizeIterator<Item = ReferenceId> + '_ {
        (0..self.symbols.reference_len()).map(ReferenceId::from_usize)
    }

    pub fn scope(&self, id: ScopeId) -> &Scope {
        self.scopes.get(id)
    }

    pub fn symbol(&self, id: SymbolId) -> &Symbol {
        self.symbols.symbol(id)
    }

    pub fn reference(&self, id: ReferenceId) -> &Reference {
        self.symbols.reference(id)
    }

    pub fn symbol_declarations(&self, id: SymbolId) -> &[NodeId] {
        &self.symbol(id).declarations
    }

    pub fn references_of(&self, id: SymbolId) -> &[ReferenceId] {
        &self.symbol(id).references
    }

    pub fn resolve_name(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.symbols.resolve(&self.scopes, scope, name)
    }

    pub fn symbol_rename_safe(&self, id: SymbolId) -> bool {
        let symbol = self.symbol(id);
        !self.scope(symbol.scope).rename_unsafe
            && !symbol.flags.intersects(SymbolFlags::ANNEX_B_FUNCTION)
    }

    pub fn is_descendant_scope(&self, descendant: ScopeId, ancestor: ScopeId) -> bool {
        self.scopes
            .ancestors(descendant)
            .any(|scope| scope == ancestor)
    }

    pub fn symbol_used_in_subtree(&self, symbol: SymbolId, scope: ScopeId) -> bool {
        self.references_of(symbol)
            .iter()
            .any(|reference| self.is_descendant_scope(self.reference(*reference).scope, scope))
    }

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
    let _ = program.traverse_with(&mut order, &mut TraverseCtx::new(ast));
    order.node_order
}
