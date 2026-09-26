//! Builds the emission plan before any AST mutation.
//!
//! Enum constants determine removable uses, module dependencies propagate the
//! remaining runtime reads before lowering enums and module bindings.

use swc_next_ecma_ast::{Ast, NodeId, Program, SymbolId, VisitWith};
use swc_next_ecma_semantic::{Semantic, SymbolFlags};

use self::{collector::AnalysisCollector, enums::EnumAnalysis, modules::ModuleAnalysis};
use super::TypeScriptOptions;
mod collector;
mod constants;
mod enums;
mod modules;

pub(super) use constants::{EnumMemberValue, TsEnumConstantValue};

#[derive(Default)]
pub(super) struct TypeScriptAnalysis<'ast> {
    enums: EnumAnalysis<'ast>,
    modules: ModuleAnalysis,
}

impl<'ast> TypeScriptAnalysis<'ast> {
    pub(super) fn analyze(
        program: Program,
        ast: &Ast<'ast>,
        semantic: &Semantic<'ast>,
        options: &TypeScriptOptions,
    ) -> Self {
        // Ordinary imports use semantic reference flags directly during lowering.
        // Only enums and import-equals need a precomputed emission plan.
        let source_module = semantic.symbol_of(program.node_id());
        let needs_plan = semantic.iter_symbols().any(|entry| {
            Some(entry.id) != source_module
                && entry.symbol.flags.intersects(
                    SymbolFlags::REGULAR_ENUM
                        | SymbolFlags::CONST_ENUM
                        | SymbolFlags::IMPORT_EQUALS,
                )
        });
        if !needs_plan {
            return Self::default();
        }

        Self::build_runtime_plan(program, ast, semantic, options)
    }

    // Keep collector storage out of the path that only erases types.
    #[inline(never)]
    fn build_runtime_plan(
        program: Program,
        ast: &Ast<'ast>,
        semantic: &Semantic<'ast>,
        options: &TypeScriptOptions,
    ) -> Self {
        let mut collector = AnalysisCollector::new(ast, semantic, options);
        program.visit_with(&mut collector);

        let (enums, inlined_references) = EnumAnalysis::finish(ast, semantic, &collector.enums);
        collector.modules.apply_enum_policy(semantic, &enums);
        let modules = ModuleAnalysis::finish(collector.modules, semantic, inlined_references);
        Self { enums, modules }
    }

    pub(super) fn value_is_erased(&self, symbol: SymbolId, semantic: &Semantic<'_>) -> bool {
        let target = semantic.ts_import_alias_target(symbol).unwrap_or(symbol);
        self.enums.is_removable(target)
    }

    pub(super) fn import_equals_is_erased(&self, node: NodeId) -> bool {
        self.modules.import_equals_is_erased(node)
    }

    pub(super) fn const_enum_is_removable(&self, symbol: SymbolId) -> bool {
        self.enums.is_removable(symbol)
    }

    pub(super) fn inline_const_enum_use(&self, node: NodeId) -> Option<TsEnumConstantValue<'ast>> {
        self.enums.inline_use(node)
    }

    pub(super) fn enum_member_value(&self, member: SymbolId) -> EnumMemberValue<'ast> {
        self.enums.member_value(member)
    }

    pub(super) fn has_runtime_usage(&self, symbol: SymbolId, semantic: &Semantic<'_>) -> bool {
        self.modules.has_runtime_usage(symbol, semantic)
    }
}
