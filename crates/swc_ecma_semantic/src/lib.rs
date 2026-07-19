//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.

#![deny(clippy::all)]
#![deny(unused)]

mod analyzer;
mod diagnostic;
mod module_record;
mod scope;
mod symbol;

pub use diagnostic::{Diagnostic, Label, Severity};
pub use module_record::{ExportRecord, ImportRecord, ModuleRecord, ModuleRecordNameKind};
pub use scope::{Scope, ScopeFlags, ScopeKind, ScopeTree};
pub use swc_experimental_ecma_ast::{ScopeId, SymbolId};
pub use symbol::{Reference, ReferenceId, ReferenceKind, Symbol, SymbolFlags, SymbolTable};

use swc_experimental_ecma_ast::{Ast, NodeId, Program};

/// Complete semantic information for one parsed source file.
#[derive(Debug)]
pub struct Semantic<'a> {
    scope_tree: ScopeTree,
    symbol_table: SymbolTable<'a>,
    node_scopes: Vec<ScopeId>,
    module_record: ModuleRecord,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Semantic<'a> {
    pub fn scope_tree(&self) -> &ScopeTree {
        &self.scope_tree
    }

    pub fn symbol_table(&self) -> &SymbolTable<'a> {
        &self.symbol_table
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

    pub fn top_level_scope_id(&self) -> ScopeId {
        self.scope_tree.top_level_scope_id()
    }
}

/// Analyze a typed flat AST and write resolved symbol IDs back to identifier nodes.
pub fn analyze<'a>(program: Program, ast: &mut Ast<'a>) -> Semantic<'a> {
    let mut parts = analyzer::analyze_program(program, ast);
    let module_record = module_record::collect(
        program,
        ast,
        &parts.scopes,
        &parts.symbols,
        &mut parts.diagnostics,
    );
    Semantic {
        scope_tree: parts.scopes,
        symbol_table: parts.symbols,
        node_scopes: parts.node_scopes,
        module_record,
        diagnostics: parts.diagnostics,
    }
}
