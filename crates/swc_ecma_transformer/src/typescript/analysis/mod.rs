//! Collects module dependencies before erasing type-only imports and exports.

use super::TypeScriptOptions;
use modules::{ModuleAnalysis, ModuleCollector};
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
    Ast, NodeId, Program, SymbolId, TsImportEqualsDeclaration, Visit, VisitWith,
};
use swc_next_ecma_semantic::Semantic;

mod modules;

pub(super) struct TypeScriptAnalysis {
    modules: ModuleAnalysis,
}

impl TypeScriptAnalysis {
    pub(super) fn analyze(
        program: Program,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
        options: &TypeScriptOptions,
    ) -> Self {
        let mut collector = AnalysisCollector {
            ast,
            semantic,
            modules: ModuleCollector::new(options),
        };
        program.visit_with(&mut collector);
        let modules = ModuleAnalysis::finish(collector.modules, semantic, FxHashSet::default());
        Self { modules }
    }
    pub(super) fn import_equals_is_erased(&self, node: NodeId) -> bool {
        self.modules.import_equals_is_erased(node)
    }
    pub(super) fn has_runtime_usage(&self, symbol: SymbolId, semantic: &Semantic<'_>) -> bool {
        self.modules.has_runtime_usage(symbol, semantic)
    }
}

struct AnalysisCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    semantic: &'visit Semantic<'ast>,
    modules: ModuleCollector,
}
impl<'ast> Visit<'ast> for AnalysisCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }
    fn visit_ts_import_equals_declaration(&mut self, declaration: TsImportEqualsDeclaration) {
        self.modules
            .record_import_equals(declaration, self.ast, self.semantic);
        declaration.visit_children_with(self);
    }
}
