//! Module import and export records.

use std::{borrow::Cow, string::String};

use rustc_hash::FxHashMap;
use swc_next_ecma_ast::*;

use crate::{
    scope::ScopeTree,
    symbol::{SymbolFlags, SymbolTable},
};

/// Shape of a module-record name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleRecordNameKind {
    /// A concrete imported, exported, or local name.
    Named,
    /// A namespace (`*`) name.
    Star,
    /// No name is present for this side of the record.
    None,
    /// A TypeScript `export =` record.
    Equals,
    /// A TypeScript `export as namespace` record.
    Global,
}

/// One dependency edge of the module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportRecord {
    /// Local binding, or `None` when the import binds nothing.
    pub symbol_id: Option<SymbolId>,
    /// Shape of the imported name.
    pub name_kind: ModuleRecordNameKind,
    /// Imported name; `"default"` for default imports.
    pub name: String,
    /// Decoded module specifier.
    pub specifier: String,
    /// Whether the import is value-only or type-only.
    pub kind: ImportOrExportKind,
    /// Import phase modifier, or `None` for regular imports.
    pub phase: Option<ImportPhase>,
    /// Smallest AST node that identifies this record.
    pub node: NodeId,
}

/// One exported name of the module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportRecord {
    /// Shape of the exported name.
    pub name_kind: ModuleRecordNameKind,
    /// Exported name.
    pub name: String,
    /// Backing local binding, or `None` when no local symbol backs the export.
    pub symbol_id: Option<SymbolId>,
    /// Shape of the name taken from a source module.
    pub from_kind: ModuleRecordNameKind,
    /// Name taken from the source module for a re-export.
    pub from_name: String,
    /// Decoded `from` specifier, empty for local exports.
    pub specifier: String,
    /// Whether the export is value-only or type-only.
    pub kind: ImportOrExportKind,
    /// Smallest AST node that identifies this record.
    pub node: NodeId,
}

/// Import and export records for one module.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModuleRecord {
    imports: Vec<ImportRecord>,
    exports: Vec<ExportRecord>,
}

impl ModuleRecord {
    /// Returns dependency edges in source order.
    pub fn imports(&self) -> &[ImportRecord] {
        &self.imports
    }

    /// Returns exported names in source order.
    pub fn exports(&self) -> &[ExportRecord] {
        &self.exports
    }
}

pub(crate) fn collect<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    scopes: &ScopeTree,
    symbols: &SymbolTable,
    diagnostics: &mut Vec<Diagnostic<'static>>,
) -> ModuleRecord {
    let mut collector = Collector {
        record: ModuleRecord::default(),
        top_scope: scopes.top_level_scope_id(),
        symbols,
        diagnostics,
        check_duplicate_exports: !ast.lang().is_typescript(),
        export_names: FxHashMap::default(),
        default_export_bindings: Vec::new(),
    };

    let _ = program.traverse_with(&mut collector, &mut TraverseCtx::new(ast, ()));
    collector.record
}

struct Collector<'table, 'diagnostics> {
    record: ModuleRecord,
    top_scope: ScopeId,
    symbols: &'table SymbolTable,
    diagnostics: &'diagnostics mut Vec<Diagnostic<'static>>,
    check_duplicate_exports: bool,
    export_names: FxHashMap<String, Span>,
    default_export_bindings: Vec<bool>,
}

impl Collector<'_, '_> {
    /// Records an exported name and reports a duplicate if already present.
    fn record_exported_name(&mut self, name: &str, span: Span) {
        // TypeScript declarations can merge across symbol spaces. Until this
        // pass resolves those spaces, rejecting repeated spellings can report
        // valid exports as duplicates.
        if !self.check_duplicate_exports {
            return;
        }
        if let Some(previous) = self.export_names.get(name).copied() {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: Cow::Owned(format!("Duplicate export '{}'", name)),
                span,
                help: None,
                labels: vec![Label {
                    span: previous,
                    message: Cow::Borrowed("previous export is here"),
                }],
            });
        } else {
            self.export_names.insert(name.into(), span);
        }
    }

    /// Resolves a local export against the top-level scope and reports an
    /// undefined export.
    fn check_unresolved_exports(&mut self, name: &str, span: Span) -> Option<SymbolId> {
        let symbol = self.symbols.find_in_scope(self.top_scope, name.as_bytes());
        if symbol.is_none() {
            self.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: Cow::Owned(format!("Export '{name}' is not defined")),
                span,
                help: None,
                labels: Vec::new(),
            });
        }
        symbol
    }

    #[allow(clippy::too_many_arguments)]
    fn enter_export_specifier(
        &mut self,
        node: ExportSpecifier,
        local_name: String,
        exported_name: String,
        source: Option<String>,
        kind: ImportOrExportKind,
        symbol_id: Option<SymbolId>,
        ast: &Ast<'_>,
    ) {
        self.record_exported_name(&exported_name, node.span(ast));
        self.record.exports.push(ExportRecord {
            name_kind: ModuleRecordNameKind::Named,
            name: exported_name,
            symbol_id,
            from_kind: if source.is_some() {
                ModuleRecordNameKind::Named
            } else {
                ModuleRecordNameKind::None
            },
            from_name: if source.is_some() {
                local_name
            } else {
                String::new()
            },
            specifier: source.unwrap_or_default(),
            kind,
            node: node.node_id(),
        });
    }
}

impl<'ast> Traverse<'ast> for Collector<'_, '_> {
    fn enter_binding_identifier(
        &mut self,
        node: BindingIdentifier,
        ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction {
        let mut ancestors = ctx.ancestors().skip(1);
        let is_default = loop {
            let Some(ancestor) = ancestors.next() else {
                return TraverseAction::Proceed;
            };
            match ctx.node_kind(ancestor) {
                NodeKind::FunctionBody | NodeKind::ClassBody | NodeKind::TsModuleBlock => {
                    return TraverseAction::Proceed;
                }
                NodeKind::ExportNamedDeclaration => break false,
                NodeKind::ExportDefaultDeclaration => break true,
                _ => {}
            }
        };
        if !ancestors
            .next()
            .is_some_and(|ancestor| ctx.node_kind(ancestor) == NodeKind::Program)
        {
            return TraverseAction::Proceed;
        }

        let ast = &*ctx.ast;
        let Some(symbol_id) = node.symbol_id(ast) else {
            return TraverseAction::Proceed;
        };
        let symbol = self.symbols.symbol(symbol_id);
        if !symbol.flags.intersects(SymbolFlags::EXPORTED) {
            return TraverseAction::Proceed;
        }
        let name: String = if is_default {
            if let Some(found) = self.default_export_bindings.last_mut() {
                *found = true;
            }
            "default".into()
        } else {
            String::from_utf8_lossy(ast.string(node.name(ast))).into_owned()
        };
        let kind = if symbol.flags.in_value_space() {
            ImportOrExportKind::Value
        } else {
            ImportOrExportKind::Type
        };
        let is_merged_ts_export = !is_default
            && ast.lang().is_typescript()
            && self.record.exports.iter().any(|record| {
                record.name == name
                    && record.symbol_id == Some(symbol_id)
                    && record.specifier.is_empty()
            });
        if !is_merged_ts_export {
            self.record_exported_name(&name, node.span(ast));
        }
        self.record.exports.push(ExportRecord {
            name_kind: ModuleRecordNameKind::Named,
            name,
            symbol_id: Some(symbol_id),
            from_kind: ModuleRecordNameKind::None,
            from_name: String::new(),
            specifier: String::new(),
            kind,
            node: node.node_id(),
        });
        TraverseAction::Proceed
    }

    fn enter_import_declaration(
        &mut self,
        node: ImportDeclaration,
        ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction {
        if !is_program_child(ctx) {
            return TraverseAction::Proceed;
        }
        let ast = &*ctx.ast;
        let specifier = string_literal(node.source(ast), ast);
        let specifiers = node.specifiers(ast);
        if specifiers.is_empty() {
            self.record.imports.push(ImportRecord {
                symbol_id: None,
                name_kind: ModuleRecordNameKind::None,
                name: String::new(),
                specifier,
                kind: node.import_kind(ast),
                phase: node.phase(ast),
                node: node.node_id(),
            });
            return TraverseAction::Proceed;
        }

        for specifier_id in specifiers.iter() {
            let specifier_node = ast.get_node_in_sub_range(specifier_id);
            let (local, name_kind, name, kind) =
                match ast.import_declaration_specifier_data(specifier_node) {
                    ImportDeclarationSpecifierData::ImportSpecifier(spec) => (
                        spec.local(ast),
                        ModuleRecordNameKind::Named,
                        module_export_name(spec.imported(ast), ast),
                        if node.import_kind(ast) == ImportOrExportKind::Type {
                            ImportOrExportKind::Type
                        } else {
                            spec.import_kind(ast)
                        },
                    ),
                    ImportDeclarationSpecifierData::ImportDefaultSpecifier(spec) => (
                        spec.local(ast),
                        ModuleRecordNameKind::Named,
                        "default".into(),
                        node.import_kind(ast),
                    ),
                    ImportDeclarationSpecifierData::ImportNamespaceSpecifier(spec) => (
                        spec.local(ast),
                        ModuleRecordNameKind::Star,
                        String::new(),
                        node.import_kind(ast),
                    ),
                };
            self.record.imports.push(ImportRecord {
                symbol_id: local.symbol_id(ast),
                name_kind,
                name,
                specifier: specifier.clone(),
                kind,
                phase: node.phase(ast),
                node: specifier_node.node_id(),
            });
        }
        TraverseAction::Proceed
    }

    fn enter_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
        ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction {
        if !is_program_child(ctx) {
            return TraverseAction::Proceed;
        }
        let ast = &*ctx.ast;
        if node.declaration(ast).is_some() {
            return TraverseAction::Proceed;
        }
        let source = node.source(ast).map(|source| string_literal(source, ast));
        for specifier_id in node.specifiers(ast).iter() {
            let specifier = ast.get_node_in_sub_range(specifier_id);
            let local_name = module_export_name(specifier.local(ast), ast);
            let exported_name = module_export_name(specifier.exported(ast), ast);
            let kind = if node.export_kind(ast) == ImportOrExportKind::Type {
                ImportOrExportKind::Type
            } else {
                specifier.export_kind(ast)
            };
            let symbol_id = source
                .is_none()
                .then(|| self.check_unresolved_exports(&local_name, specifier.span(ast)))
                .flatten();
            self.enter_export_specifier(
                specifier,
                local_name,
                exported_name,
                source.clone(),
                kind,
                symbol_id,
                ast,
            );
        }
        TraverseAction::Proceed
    }

    fn enter_export_default_declaration(
        &mut self,
        _node: ExportDefaultDeclaration,
        ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction {
        if !is_program_child(ctx) {
            return TraverseAction::Proceed;
        }
        self.default_export_bindings.push(false);
        TraverseAction::Proceed
    }

    fn exit_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
        ctx: &mut TraverseCtx<'_, 'ast>,
    ) {
        if !is_program_child(ctx) {
            return;
        }
        let has_binding = self
            .default_export_bindings
            .pop()
            .expect("top-level default export traversal is balanced");
        if has_binding {
            return;
        }
        self.record_exported_name("default", node.span(ctx.ast));
        self.record.exports.push(ExportRecord {
            name_kind: ModuleRecordNameKind::Named,
            name: "default".into(),
            symbol_id: None,
            from_kind: ModuleRecordNameKind::None,
            from_name: String::new(),
            specifier: String::new(),
            kind: ImportOrExportKind::Value,
            node: node.node_id(),
        });
    }

    fn enter_export_all_declaration(
        &mut self,
        node: ExportAllDeclaration,
        ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction {
        if !is_program_child(ctx) {
            return TraverseAction::Proceed;
        }
        let ast = &*ctx.ast;
        let exported = node.exported(ast);
        if let Some(name) = exported {
            self.record_exported_name(&module_export_name(name, ast), node.span(ast));
        }
        self.record.exports.push(ExportRecord {
            name_kind: exported.map_or(ModuleRecordNameKind::Star, |_| ModuleRecordNameKind::Named),
            name: exported
                .map(|name| module_export_name(name, ast))
                .unwrap_or_default(),
            symbol_id: None,
            from_kind: ModuleRecordNameKind::Star,
            from_name: String::new(),
            specifier: string_literal(node.source(ast), ast),
            kind: node.export_kind(ast),
            node: node.node_id(),
        });
        TraverseAction::Proceed
    }

    fn enter_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction {
        if !is_program_child(ctx) {
            return TraverseAction::Proceed;
        }
        let ast = &*ctx.ast;
        // Only the external-module-reference form is a dependency edge. A
        // qualified name aliases a namespace instead of another module.
        let TsModuleReferenceData::TsExternalModuleReference(reference) =
            ast.ts_module_reference_data(node.module_reference(ast))
        else {
            return TraverseAction::Proceed;
        };
        self.record.imports.push(ImportRecord {
            symbol_id: node.id(ast).symbol_id(ast),
            name_kind: ModuleRecordNameKind::Star,
            name: String::new(),
            specifier: string_literal(reference.expression(ast), ast),
            kind: node.import_kind(ast),
            phase: None,
            node: node.node_id(),
        });
        TraverseAction::Proceed
    }

    fn enter_ts_export_assignment(
        &mut self,
        node: TsExportAssignment,
        ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction {
        if !is_program_child(ctx) {
            return TraverseAction::Proceed;
        }
        self.record.exports.push(ExportRecord {
            name_kind: ModuleRecordNameKind::Equals,
            name: String::new(),
            symbol_id: None,
            from_kind: ModuleRecordNameKind::None,
            from_name: String::new(),
            specifier: String::new(),
            kind: ImportOrExportKind::Value,
            node: node.node_id(),
        });
        let _ = ctx;
        TraverseAction::Proceed
    }

    fn enter_ts_namespace_export_declaration(
        &mut self,
        node: TsNamespaceExportDeclaration,
        ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction {
        if !is_program_child(ctx) {
            return TraverseAction::Proceed;
        }
        let ast = &*ctx.ast;
        self.record.exports.push(ExportRecord {
            name_kind: ModuleRecordNameKind::Global,
            name: String::from_utf8_lossy(ast.string(node.id(ast).name(ast))).into_owned(),
            symbol_id: None,
            from_kind: ModuleRecordNameKind::None,
            from_name: String::new(),
            specifier: String::new(),
            kind: ImportOrExportKind::Value,
            node: node.node_id(),
        });
        TraverseAction::Proceed
    }
}

fn string_literal(node: StringLiteral, ast: &Ast<'_>) -> String {
    String::from_utf8_lossy(ast.string(node.value(ast))).into_owned()
}

fn module_export_name(node: ModuleExportName, ast: &Ast<'_>) -> String {
    match ast.module_export_name_data(node) {
        ModuleExportNameData::IdentifierName(identifier) => {
            String::from_utf8_lossy(ast.string(identifier.name(ast))).into_owned()
        }
        ModuleExportNameData::StringLiteral(string) => string_literal(string, ast),
    }
}

fn is_program_child(ctx: &TraverseCtx<'_, '_>) -> bool {
    ctx.parent()
        .is_some_and(|parent| ctx.node_kind(parent) == NodeKind::Program)
}
