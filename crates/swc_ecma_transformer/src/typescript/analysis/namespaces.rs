use rustc_hash::{FxHashMap, FxHashSet};

use swc_next_ecma_ast::{
    Ast, BindingIdentifier, Decl, DeclData, ImportOrExportKind, NodeId, NodeKindData, Stmt,
    StmtData, SymbolId, TsModuleBody, TsModuleBodyData, TsModuleDeclaration, TsModuleName,
    TsModuleNameData,
};
use swc_next_ecma_semantic::{Semantic, SymbolFlags};

use super::super::TypeScriptOptions;
use super::{enums::EnumAnalysis, modules::ModuleAnalysis};

#[derive(Default)]
pub(super) struct NamespaceCollector {
    declarations: Vec<TsModuleDeclaration>,
    runtime_value_candidates: Vec<BindingIdentifier>,
}

impl NamespaceCollector {
    pub(super) fn record_statement(&mut self, statement: Stmt, ast: &Ast<'_>) {
        match ast.stmt_data(statement) {
            StmtData::Declaration(declaration) => {
                self.record_runtime_value_declaration(declaration, ast);
            }
            StmtData::ExportNamedDeclaration(export) => {
                if let Some(declaration) = export.declaration(ast) {
                    self.record_runtime_value_declaration(declaration, ast);
                }
            }
            _ => {}
        }
    }

    pub(super) fn record_declaration(&mut self, declaration: TsModuleDeclaration) {
        self.declarations.push(declaration);
    }

    fn record_runtime_value_declaration(&mut self, declaration: Decl, ast: &Ast<'_>) {
        let binding = match ast.decl_data(declaration) {
            DeclData::Function(function) => function.id(ast),
            DeclData::Class(class) if !class.declare(ast) => class.id(ast),
            _ => None,
        };
        if let Some(binding) = binding {
            self.runtime_value_candidates.push(binding);
        }
    }
}

#[derive(Default)]
pub(super) struct NamespaceAnalysis {
    instantiated_declarations: FxHashSet<NodeId>,
    has_merged_runtime_bindings: bool,
}

impl NamespaceAnalysis {
    pub(super) fn finish(
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
        options: &TypeScriptOptions,
        enums: &EnumAnalysis<'_>,
        modules: &ModuleAnalysis,
        collector: &NamespaceCollector,
    ) -> Self {
        if collector.declarations.is_empty() {
            return Self::default();
        }

        let has_merged_runtime_bindings =
            collector.runtime_value_candidates.iter().any(|binding| {
                let Some(symbol) = semantic.symbol_of(binding.node_id()) else {
                    return false;
                };
                let flags = semantic.symbol(symbol).flags;
                flags.intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS)
                    && flags.intersects(SymbolFlags::VALUE_MODULE | SymbolFlags::NAMESPACE_MODULE)
            });
        let mut analysis = Self {
            has_merged_runtime_bindings,
            ..Self::default()
        };

        let mut finalizer =
            NamespaceFinalizer::new(ast, semantic, options, enums, modules, &mut analysis);
        for &declaration in &collector.declarations {
            finalizer.namespace_instantiation(declaration);
        }
        analysis
    }

    pub(super) fn is_instantiated(&self, declaration: NodeId) -> bool {
        self.instantiated_declarations.contains(&declaration)
    }

    pub(super) fn has_merged_runtime_bindings(&self) -> bool {
        self.has_merged_runtime_bindings
    }
}

struct NamespaceFinalizer<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    semantic: &'visit Semantic<'ast>,
    options: &'visit TypeScriptOptions,
    enums: &'visit EnumAnalysis<'ast>,
    modules: &'visit ModuleAnalysis,
    analysis: &'visit mut NamespaceAnalysis,
    namespace_instantiations: FxHashMap<NodeId, NamespaceInstantiation>,
    runtime_namespace_usage: FxHashMap<SymbolId, bool>,
}

#[derive(Clone, Copy)]
struct NamespaceInstantiation {
    contributes_to_parent: bool,
    emits_runtime: bool,
}

impl<'visit, 'ast> NamespaceFinalizer<'visit, 'ast> {
    fn new(
        ast: &'visit Ast<'ast>,
        semantic: &'visit Semantic<'ast>,
        options: &'visit TypeScriptOptions,
        enums: &'visit EnumAnalysis<'ast>,
        modules: &'visit ModuleAnalysis,
        analysis: &'visit mut NamespaceAnalysis,
    ) -> Self {
        Self {
            ast,
            semantic,
            options,
            enums,
            modules,
            analysis,
            namespace_instantiations: FxHashMap::default(),
            runtime_namespace_usage: FxHashMap::default(),
        }
    }

    fn namespace_instantiation(
        &mut self,
        declaration: TsModuleDeclaration,
    ) -> NamespaceInstantiation {
        if let Some(&instantiation) = self.namespace_instantiations.get(&declaration.node_id()) {
            return instantiation;
        }

        let body_instantiated = declaration
            .body(self.ast)
            .is_none_or(|body| self.namespace_body_is_instantiated(body));
        let contributes_to_parent =
            body_instantiated || self.namespace_name_has_runtime_usage(declaration.id(self.ast));
        let emits_runtime = contributes_to_parent
            && !declaration.declare(self.ast)
            && !matches!(
                self.ast.ts_module_name_data(declaration.id(self.ast)),
                TsModuleNameData::StringLiteral(_)
            );
        let instantiation = NamespaceInstantiation {
            contributes_to_parent,
            emits_runtime,
        };
        self.namespace_instantiations
            .insert(declaration.node_id(), instantiation);
        if instantiation.emits_runtime {
            self.analysis
                .instantiated_declarations
                .insert(declaration.node_id());
        }
        instantiation
    }

    fn namespace_name_has_runtime_usage(&mut self, name: TsModuleName) -> bool {
        let mut node = name.node_id();
        loop {
            match self.ast.kind_data(node) {
                NodeKindData::TsQualifiedName(qualified) => {
                    if self.const_enum_namespace_is_used(qualified.right(self.ast).node_id()) {
                        return true;
                    }
                    node = qualified.left(self.ast).node_id();
                }
                _ => return self.const_enum_namespace_is_used(node),
            }
        }
    }

    fn const_enum_namespace_is_used(&mut self, node: NodeId) -> bool {
        let Some(symbol) = self.semantic.symbol_of(node) else {
            return false;
        };
        let flags = self.semantic.symbol(symbol).flags;
        if !flags.contains(SymbolFlags::CONST_ENUM_ONLY_MODULE) {
            return false;
        }
        if flags.contains(SymbolFlags::EXPORTED) {
            return true;
        }
        // Merged declarations share references. Scan them once, excluding uses
        // already replaced by enum inlining and dead import-equals aliases.
        *self
            .runtime_namespace_usage
            .entry(symbol)
            .or_insert_with(|| self.modules.has_runtime_usage(symbol, self.semantic))
    }

    fn namespace_body_is_instantiated(&mut self, body: TsModuleBody) -> bool {
        match self.ast.ts_module_body_data(body) {
            TsModuleBodyData::TsModuleBlock(block) => {
                !block.directives(self.ast).is_empty()
                    || block.body(self.ast).iter().any(|statement_id| {
                        let statement = self.ast.get_node_in_sub_range(statement_id);
                        self.statement_instantiates_namespace(statement)
                    })
            }
            TsModuleBodyData::TsModuleDeclaration(declaration) => {
                self.namespace_instantiation(declaration)
                    .contributes_to_parent
            }
        }
    }

    fn statement_instantiates_namespace(&mut self, statement: Stmt) -> bool {
        match self.ast.stmt_data(statement) {
            StmtData::Declaration(declaration) => {
                self.declaration_instantiates_namespace(declaration, false)
            }
            StmtData::ExportNamedDeclaration(export) => {
                export.declaration(self.ast).is_some_and(|declaration| {
                    self.declaration_instantiates_namespace(declaration, true)
                })
            }
            StmtData::ImportDeclaration(_)
            | StmtData::ExportDefaultDeclaration(_)
            | StmtData::ExportAllDeclaration(_)
            | StmtData::TsNamespaceExportDeclaration(_) => false,
            _ => true,
        }
    }

    fn declaration_instantiates_namespace(&mut self, declaration: Decl, exported: bool) -> bool {
        match self.ast.decl_data(declaration) {
            DeclData::VariableDeclaration(_) | DeclData::Function(_) | DeclData::Class(_) => true,
            DeclData::TsFunction(_) => true,
            DeclData::TsTypeAliasDeclaration(_)
            | DeclData::TsInterfaceDeclaration(_)
            | DeclData::TsGlobalDeclaration(_) => false,
            DeclData::TsEnumDeclaration(declaration) => {
                let ambient_const_enum = declaration.declare(self.ast)
                    && declaration.is_const(self.ast)
                    && !self.options.preserve_const_enums
                    && !self.options.verbatim_module_syntax;
                if ambient_const_enum {
                    return false;
                }
                let symbol = self
                    .semantic
                    .symbol_of(declaration.id(self.ast).node_id())
                    .expect("an enum declaration has a symbol");
                !self.enums.is_removable(symbol)
            }
            DeclData::TsModuleDeclaration(declaration) => {
                self.namespace_instantiation(declaration)
                    .contributes_to_parent
            }
            DeclData::TsImportEqualsDeclaration(declaration) => {
                if declaration.import_kind(self.ast) == ImportOrExportKind::Type {
                    return false;
                }
                let symbol = self.semantic.symbol_of(declaration.id(self.ast).node_id());
                exported
                    || self.options.verbatim_module_syntax
                    || symbol
                        .is_none_or(|symbol| self.modules.has_runtime_usage(symbol, self.semantic))
            }
        }
    }
}
