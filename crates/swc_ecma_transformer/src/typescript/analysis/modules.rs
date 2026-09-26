use rustc_hash::{FxHashMap, FxHashSet};

use swc_next_ecma_ast::{Ast, ImportOrExportKind, NodeId, SymbolId, TsImportEqualsDeclaration};
use swc_next_ecma_semantic::{Semantic, SymbolFlags};

use super::super::TypeScriptOptions;

pub(super) struct ModuleCollector {
    initializer_references: FxHashSet<NodeId>,
    erased_aliases: FxHashSet<NodeId>,
    collect_runtime_usage: bool,
    alias_dependencies: FxHashMap<SymbolId, AliasDependency>,
}

struct AliasDependency {
    targets: Vec<SymbolId>,
}

impl ModuleCollector {
    pub(super) fn new(options: &TypeScriptOptions) -> Self {
        Self {
            initializer_references: FxHashSet::default(),
            erased_aliases: FxHashSet::default(),
            collect_runtime_usage: !options.verbatim_module_syntax,
            alias_dependencies: FxHashMap::default(),
        }
    }

    pub(super) fn record_import_equals(
        &mut self,
        declaration: TsImportEqualsDeclaration,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) {
        let alias = semantic.symbol_of(declaration.id(ast).node_id());
        let reference = semantic.import_equals_reference(ast, declaration);
        if self.collect_runtime_usage
            && let Some(reference) = reference
        {
            self.initializer_references
                .insert(semantic.reference(reference).node);
        }
        let explicit_type = declaration.import_kind(ast) == ImportOrExportKind::Type;
        let erased_target = self.collect_runtime_usage
            && reference.is_some_and(|id| !semantic.reference(id).flags.is_read());
        if explicit_type || erased_target {
            self.erased_aliases.insert(declaration.node_id());
            return;
        }
        if !self.collect_runtime_usage {
            return;
        }
        let (Some(alias), Some(reference)) = (alias, reference) else {
            return;
        };
        let mut targets: Vec<_> = semantic
            .reference_value_targets(ast, reference)
            .filter_map(|target| target.candidate())
            .collect();
        // The root preserves initialization dependencies; the terminal target
        // also keeps an otherwise erased namespace object available.
        if let Some(target) = semantic.ts_import_alias_target(alias)
            && !targets.contains(&target)
        {
            targets.push(target);
        }
        self.alias_dependencies
            .insert(alias, AliasDependency { targets });
    }
}

#[derive(Default)]
pub(super) struct ModuleAnalysis {
    ignored_references: FxHashSet<NodeId>,
    erased_aliases: FxHashSet<NodeId>,
    propagated_runtime_usage: FxHashSet<SymbolId>,
}

impl ModuleAnalysis {
    pub(super) fn finish(
        collector: ModuleCollector,
        semantic: &Semantic<'_>,
        mut ignored_references: FxHashSet<NodeId>,
    ) -> Self {
        ignored_references.extend(collector.initializer_references);
        let mut analysis = Self {
            erased_aliases: collector.erased_aliases,
            ignored_references,
            ..Self::default()
        };
        if !collector.collect_runtime_usage {
            return analysis;
        }
        let mut worklist = Vec::new();
        for &alias in collector.alias_dependencies.keys() {
            if semantic.symbol(alias).flags.contains(SymbolFlags::EXPORTED)
                || analysis.has_direct_runtime_usage(alias, semantic)
            {
                analysis.propagated_runtime_usage.insert(alias);
                worklist.push(alias);
            }
        }

        while let Some(alias) = worklist.pop() {
            let Some(dependency) = collector.alias_dependencies.get(&alias) else {
                continue;
            };
            for &target in &dependency.targets {
                if analysis.propagated_runtime_usage.insert(target) {
                    worklist.push(target);
                }
            }
        }
        analysis
    }

    pub(super) fn import_equals_is_erased(&self, node: NodeId) -> bool {
        self.erased_aliases.contains(&node)
    }

    pub(super) fn has_runtime_usage(&self, symbol: SymbolId, semantic: &Semantic<'_>) -> bool {
        self.propagated_runtime_usage.contains(&symbol)
            || self.has_direct_runtime_usage(symbol, semantic)
    }

    fn has_direct_runtime_usage(&self, symbol: SymbolId, semantic: &Semantic<'_>) -> bool {
        semantic.uses(symbol).iter().any(|&reference| {
            let reference = semantic.reference(reference);
            !self.ignored_references.contains(&reference.node)
                && (reference.flags.is_read() || reference.flags.is_write())
        })
    }
}
