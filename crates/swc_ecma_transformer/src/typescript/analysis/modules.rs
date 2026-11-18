use rustc_hash::{FxHashMap, FxHashSet};

use swc_next_ecma_ast::{
    Ast, ImportOrExportKind, NodeId, NodeKindData, SymbolId, TsImportEqualsDeclaration,
    TsModuleReferenceData,
};
use swc_next_ecma_semantic2::{ScopeKind, Semantic, SymbolFlags};

use super::super::{
    TypeScriptOptions,
    bindings::{resolve_value_binding, value_binding},
};

pub(super) struct ModuleCollector {
    initializer_references: FxHashSet<NodeId>,
    erased_aliases: FxHashSet<NodeId>,
    collect_runtime_usage: bool,
    alias_dependencies: FxHashMap<SymbolId, AliasDependency>,
}

struct AliasDependency {
    declaration: NodeId,
    root: Option<SymbolId>,
    target: Option<SymbolId>,
    global: bool,
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

    pub(super) fn apply_enum_policy(&mut self, enums: &super::enums::EnumAnalysis<'_>) {
        self.alias_dependencies.retain(|_, dependency| {
            let erased = dependency
                .target
                .is_some_and(|target| enums.is_removable(target));
            if erased {
                self.erased_aliases.insert(dependency.declaration);
            }
            !erased
        });
    }

    pub(super) fn record_import_equals(
        &mut self,
        declaration: TsImportEqualsDeclaration,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) {
        let alias = semantic.symbol_of(declaration.id(ast).node_id());
        let reference = match ast.ts_module_reference_data(declaration.module_reference(ast)) {
            TsModuleReferenceData::TsTypeName(name) => {
                let mut root = name.node_id();
                while let NodeKindData::TsQualifiedName(qualified) = ast.kind_data(root) {
                    root = qualified.left(ast).node_id();
                }
                semantic.reference_of(root)
            }
            TsModuleReferenceData::TsExternalModuleReference(_) => None,
        };
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
        // A script's global aliases can be read by other scripts. The initializer
        // reference belongs to the declaration's lexical scope, even when its
        // target resolves to a symbol in another scope.
        let reference = semantic.reference(reference);
        let global = semantic.scope(reference.scope).kind == ScopeKind::Global;
        let root = reference
            .symbol
            .map(|symbol| value_binding(semantic, symbol));
        // The root preserves initialization dependencies; the terminal target
        // also keeps an otherwise erased namespace object available.
        let target = resolve_value_binding(semantic, alias);
        self.alias_dependencies.insert(
            alias,
            AliasDependency {
                declaration: declaration.node_id(),
                root,
                target,
                global,
            },
        );
    }
}

#[derive(Default)]
pub(super) struct ModuleAnalysis {
    ignored_references: FxHashSet<NodeId>,
    erased_aliases: FxHashSet<NodeId>,
    propagated_runtime_usage: FxHashSet<SymbolId>,
    export_locals: FxHashMap<SymbolId, Vec<SymbolId>>,
}

impl ModuleAnalysis {
    pub(super) fn with_implicit_usage(usage: FxHashSet<SymbolId>) -> Self {
        Self {
            propagated_runtime_usage: usage,
            ..Self::default()
        }
    }

    pub(super) fn finish(
        collector: ModuleCollector,
        semantic: &Semantic<'_>,
        mut ignored_references: FxHashSet<NodeId>,
        implicit_usage: FxHashSet<SymbolId>,
    ) -> Self {
        ignored_references.extend(collector.initializer_references);
        let mut analysis = Self {
            erased_aliases: collector.erased_aliases,
            ignored_references,
            ..Self::with_implicit_usage(implicit_usage)
        };
        if !collector.collect_runtime_usage {
            return analysis;
        }
        for entry in semantic.iter_symbols() {
            if entry.symbol.flags.contains(SymbolFlags::EXPORT_VALUE) {
                let export = value_binding(semantic, entry.id);
                analysis
                    .export_locals
                    .entry(export)
                    .or_default()
                    .push(entry.id);
            }
        }
        let mut worklist: Vec<_> = analysis.propagated_runtime_usage.iter().copied().collect();
        for (&alias, dependency) in &collector.alias_dependencies {
            if dependency.global
                || semantic.symbol(alias).flags.contains(SymbolFlags::EXPORTED)
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
            for target in [dependency.root, dependency.target].into_iter().flatten() {
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
        std::iter::once(symbol)
            .chain(
                self.export_locals
                    .get(&symbol)
                    .into_iter()
                    .flatten()
                    .copied(),
            )
            .flat_map(|symbol| semantic.uses(symbol))
            .any(|&reference| {
                let reference = semantic.reference(reference);
                !self.ignored_references.contains(&reference.node)
                    && (reference.flags.is_read() || reference.flags.is_write())
            })
    }
}
