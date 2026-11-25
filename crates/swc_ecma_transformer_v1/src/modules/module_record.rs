use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap};
use swc_next_ecma_ast::*;

use crate::common::{build, resolver::context::Resolver};

use super::{
    module_ref_rewriter::{ImportBinding, ImportMap},
    util::ModuleCtx,
};

/// SWC's source module record separates dependency requests from executable items.
/// Requests are ordered by first appearance and keyed by specifier and attributes.
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L121>.
#[derive(Default)]
pub(super) struct SourceModule {
    pub requested_modules: RequestedModules,
    pub local_export_entries: LocalExportEntries,
    pub ts_export_assignment: Option<Expr>,
    pub body: Vec<SourceModuleItem>,
    pub has_module_syntax: bool,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L22>.
pub(super) type RequestedModules = IndexMap<ModuleRequestRecord, RequestedModule, FxBuildHasher>;
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L26>.
pub(super) type LocalExportEntries = FxHashMap<AstUtf8, LocalExportEntry>;

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L29>.
#[derive(PartialEq, Eq, Hash)]
pub(super) struct ModuleRequestRecord {
    pub specifier: AstUtf8,
    pub attributes: Vec<ImportAttributeRecord>,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L47>.
#[derive(PartialEq, Eq, Hash)]
pub(super) struct ImportAttributeRecord {
    pub key: AstUtf8,
    pub value: AstWtf8,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L679>.
pub(super) struct RequestedModule {
    pub source: StringLiteral,
    pub entries: Vec<ModuleRecordEntry>,
    pub usage: ModuleRequestUsage,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L697>.
#[derive(Clone, Copy, Default)]
pub(super) struct ModuleRequestUsage {
    named: bool,
    default: bool,
    star_export: bool,
}

impl ModuleRequestUsage {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L718>.
    pub fn has_star_export(self) -> bool {
        self.star_export
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L706>.
    fn needs_interop(self) -> bool {
        self.default
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L714>.
    fn needs_namespace_object(self) -> bool {
        self.named && self.default
    }

    pub fn interop_require_helper(self) -> Option<&'static str> {
        if self.needs_interop() {
            Some(if self.needs_namespace_object() {
                "interop_require_wildcard"
            } else {
                "interop_require_default"
            })
        } else {
            None
        }
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L464>.
#[derive(Clone, Copy)]
pub(super) enum ModuleRecordEntry {
    ImportNamed {
        import_name: AstUtf8,
        local_name: BindingIdentifier,
    },
    ImportDefault {
        local_name: BindingIdentifier,
    },
    ImportNamespace {
        local_name: BindingIdentifier,
    },
    IndirectExportNamed {
        import_name: AstUtf8,
        export_name: ExportName,
    },
    IndirectExportDefault {
        export_name: ExportName,
    },
    IndirectExportNamespace {
        export_name: ExportName,
    },
    StarExport,
}

#[derive(Clone, Copy)]
pub(super) struct ExportName {
    pub name: AstUtf8,
    pub span: Span,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L958>.
#[derive(Clone, Copy)]
pub(super) struct LocalExportEntry {
    pub exported: ExportName,
    pub local: SymbolId,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L146>.
pub(super) enum SourceModuleItem {
    Stmt(Stmt),
    ExportDecl(Decl),
    ExportDefaultDecl(ExportDefaultDeclaration),
    ExportDefaultExpr {
        span: Span,
        expr: Expr,
    },
    TsImportEquals {
        import: TsImportEqualsDeclaration,
        exported: bool,
    },
}

impl SourceModule {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L138>.
    pub fn collect(ast: &mut Ast<'_>, resolver: &Resolver) -> Self {
        let mut module = Self::default();
        for slot in ast.root_program().body(ast).iter() {
            let stmt = ast.get_node_in_sub_range(slot);
            match ast.stmt_data(stmt) {
                StmtData::ImportDeclaration(import) => {
                    module.has_module_syntax = true;
                    module.collect_import_decl(ast, import);
                }
                StmtData::ExportNamedDeclaration(export) => {
                    let import = export
                        .declaration(ast)
                        .and_then(|decl| decl.as_ts_import_equals_declaration(ast));
                    module.has_module_syntax |= import.is_none_or(|import| {
                        external_ts_import_equals_source(ast, import).is_some()
                    });
                    module.collect_named_export(ast, resolver, export);
                }
                StmtData::ExportAllDeclaration(export) => {
                    module.has_module_syntax = true;
                    module.collect_export_all(ast, export);
                }
                StmtData::ExportDefaultDeclaration(export) => {
                    module.has_module_syntax = true;
                    module.collect_export_default_decl(ast, resolver, export);
                }
                StmtData::Declaration(decl) if decl.is_ts_import_equals_declaration(ast) => {
                    let import = decl.as_ts_import_equals_declaration(ast).unwrap();
                    module.collect_ts_import_equals(ast, resolver, import, false);
                }
                StmtData::TsExportAssignment(export) => {
                    module.has_module_syntax = true;
                    module
                        .ts_export_assignment
                        .get_or_insert(export.expression(ast));
                }
                StmtData::TsNamespaceExportDeclaration(_) => module.has_module_syntax = true,
                _ => module.body.push(SourceModuleItem::Stmt(stmt)),
            }
        }
        module
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L282>.
    fn collect_import_decl(&mut self, ast: &mut Ast<'_>, import: ImportDeclaration) {
        if import.import_kind(ast) == ImportOrExportKind::Type {
            return;
        }
        let source = import.source(ast);
        let attributes = import.attributes(ast);
        let specifiers = import.specifiers(ast);
        let module_request = self.module_request(ast, source, attributes);
        for slot in specifiers.iter() {
            let specifier = ast.get_node_in_sub_range(slot);
            let entry = match ast.import_declaration_specifier_data(specifier) {
                ImportDeclarationSpecifierData::ImportSpecifier(specifier) => {
                    if specifier.import_kind(ast) == ImportOrExportKind::Type {
                        continue;
                    }
                    let local_name = specifier.local(ast);
                    let import_name = export_name(ast, specifier.imported(ast)).name;
                    if ast.get_utf8(import_name) == "default" {
                        ModuleRecordEntry::ImportDefault { local_name }
                    } else {
                        ModuleRecordEntry::ImportNamed {
                            import_name,
                            local_name,
                        }
                    }
                }
                ImportDeclarationSpecifierData::ImportDefaultSpecifier(specifier) => {
                    ModuleRecordEntry::ImportDefault {
                        local_name: specifier.local(ast),
                    }
                }
                ImportDeclarationSpecifierData::ImportNamespaceSpecifier(specifier) => {
                    ModuleRecordEntry::ImportNamespace {
                        local_name: specifier.local(ast),
                    }
                }
            };
            module_request.insert(entry);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L420>.
    fn collect_export_all(&mut self, ast: &mut Ast<'_>, export: ExportAllDeclaration) {
        let exported = export.exported(ast).map(|name| export_name(ast, name));
        let source = export.source(ast);
        let attributes = export.attributes(ast);
        let module_request = self.module_request(ast, source, attributes);
        match exported {
            Some(exported) => module_request.insert(ModuleRecordEntry::IndirectExportNamespace {
                export_name: exported,
            }),
            None => module_request.insert(ModuleRecordEntry::StarExport),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L439>.
    fn collect_ts_import_equals(
        &mut self,
        ast: &mut Ast<'_>,
        resolver: &Resolver,
        import: TsImportEqualsDeclaration,
        exported: bool,
    ) {
        if external_ts_import_equals_source(ast, import).is_none() {
            return;
        }
        if exported {
            let id = import.id(ast);
            self.local_export_entry(binding_name(ast, id), resolver.symbol_of(id.node_id()));
        }
        self.body
            .push(SourceModuleItem::TsImportEquals { import, exported });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L396>.
    fn collect_export_default_decl(
        &mut self,
        ast: &mut Ast<'_>,
        resolver: &Resolver,
        export: ExportDefaultDeclaration,
    ) {
        let decl = ast.export_default_declaration_kind_data(export.declaration(ast));
        let binding = match decl {
            ExportDefaultDeclarationKindData::Class(class) => class.id(ast),
            ExportDefaultDeclarationKindData::Function(function) => function.id(ast),
            ExportDefaultDeclarationKindData::Expr(expr) => {
                self.body.push(SourceModuleItem::ExportDefaultExpr {
                    span: export.span(ast),
                    expr,
                });
                return;
            }
            _ => None,
        };
        if let Some(binding) = binding {
            let name = default_export_name(ast, export.span(ast));
            self.local_export_entry(name, resolver.symbol_of(binding.node_id()));
        }
        self.body.push(SourceModuleItem::ExportDefaultDecl(export));
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L328>.
    fn collect_named_export(
        &mut self,
        ast: &mut Ast<'_>,
        resolver: &Resolver,
        export: ExportNamedDeclaration,
    ) {
        if export.export_kind(ast) == ImportOrExportKind::Type {
            return;
        }
        if let Some(decl) = export.declaration(ast) {
            if let Some(import) = decl.as_ts_import_equals_declaration(ast) {
                self.collect_ts_import_equals(ast, resolver, import, true);
                return;
            }
            self.collect_export_decl(ast, resolver, decl);
            self.body.push(SourceModuleItem::ExportDecl(decl));
            return;
        }

        let specifiers = export.specifiers(ast);
        if let Some(source) = export.source(ast) {
            let attributes = export.attributes(ast);
            let module_request = self.module_request(ast, source, attributes);
            for slot in specifiers.iter() {
                let specifier = ast.get_node_in_sub_range(slot);
                if specifier.export_kind(ast) == ImportOrExportKind::Value {
                    let import_name = export_name(ast, specifier.local(ast)).name;
                    let export_name = export_name(ast, specifier.exported(ast));
                    let entry = if ast.get_utf8(import_name) == "default" {
                        ModuleRecordEntry::IndirectExportDefault { export_name }
                    } else {
                        ModuleRecordEntry::IndirectExportNamed {
                            import_name,
                            export_name,
                        }
                    };
                    module_request.insert(entry);
                }
            }
        } else {
            for slot in specifiers.iter() {
                let specifier = ast.get_node_in_sub_range(slot);
                if specifier.export_kind(ast) == ImportOrExportKind::Value {
                    let local = specifier.local(ast);
                    if let Some(local) = resolver.symbol(local.node_id()) {
                        self.local_export_entry(export_name(ast, specifier.exported(ast)), local);
                    }
                }
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L305>.
    fn collect_export_decl(&mut self, ast: &mut Ast<'_>, resolver: &Resolver, decl: Decl) {
        for binding in declaration_bindings(ast, decl) {
            self.local_export_entry(
                binding_name(ast, binding),
                resolver.symbol_of(binding.node_id()),
            );
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L356>.
    fn local_export_entry(&mut self, exported: ExportName, local: SymbolId) {
        self.local_export_entries
            .insert(exported.name, LocalExportEntry { exported, local });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L272>.
    fn module_request(
        &mut self,
        ast: &mut Ast<'_>,
        source: StringLiteral,
        attributes: SubRange<ImportAttribute>,
    ) -> &mut RequestedModule {
        let specifier = lossy_string(ast, source);
        let attributes = import_attributes(ast, attributes);
        let module_request = self
            .requested_modules
            .entry(ModuleRequestRecord {
                specifier,
                attributes,
            })
            .or_insert_with(|| {
                let source = ast.string_literal(source.span(ast), specifier.into(), AstUtf8::EMPTY);
                RequestedModule {
                    source,
                    entries: Vec::new(),
                    usage: ModuleRequestUsage::default(),
                }
            });
        if module_request.source.span(ast) == DUMMY_SP {
            module_request.source.set_span(ast, source.span(ast));
        }
        module_request
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L64>.
fn import_attributes(
    ast: &mut Ast<'_>,
    attributes: SubRange<ImportAttribute>,
) -> Vec<ImportAttributeRecord> {
    let mut attributes: Vec<_> = attributes
        .iter()
        .map(|slot| {
            let attribute = ast.get_node_in_sub_range(slot);
            let key = match ast.module_export_name_data(attribute.key(ast)) {
                ModuleExportNameData::IdentifierName(id) => ast.get_utf8(id.name(ast)),
                ModuleExportNameData::StringLiteral(string) => {
                    ast.get_wtf8(string.value(ast)).as_str()?
                }
            };
            let key = ast.add_utf8(key);
            let value = ast.get_wtf8(attribute.value(ast).value(ast));
            Some(ImportAttributeRecord {
                key,
                value: ast.add_wtf8(value),
            })
        })
        .collect::<Option<_>>()
        .unwrap_or_default();
    attributes.sort_unstable_by(|a, b| {
        ast.get_utf8(a.key)
            .encode_utf16()
            .cmp(ast.get_utf8(b.key).encode_utf16())
            .then_with(|| {
                ast.get_wtf8(a.value)
                    .as_bytes()
                    .cmp(ast.get_wtf8(b.value).as_bytes())
            })
    });
    attributes
}

impl RequestedModule {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L816>.
    fn insert(&mut self, entry: ModuleRecordEntry) {
        match entry {
            ModuleRecordEntry::ImportDefault { .. }
            | ModuleRecordEntry::IndirectExportDefault { .. } => self.usage.default = true,
            ModuleRecordEntry::ImportNamed { .. }
            | ModuleRecordEntry::IndirectExportNamed { .. } => self.usage.named = true,
            ModuleRecordEntry::ImportNamespace { .. }
            | ModuleRecordEntry::IndirectExportNamespace { .. } => {
                self.usage.named = true;
                self.usage.default = true;
            }
            ModuleRecordEntry::StarExport => {
                if self.usage.star_export {
                    return;
                }
                self.usage.star_export = true;
            }
        }
        self.entries.push(entry);
    }
}

pub(super) fn export_name(ast: &mut Ast<'_>, name: ModuleExportName) -> ExportName {
    let span = name.span(ast);
    let name = match ast.module_export_name_data(name) {
        ModuleExportNameData::IdentifierName(id) => ast.add_utf8(ast.get_utf8(id.name(ast))),
        ModuleExportNameData::StringLiteral(string) => lossy_string(ast, string),
    };
    ExportName { name, span }
}

pub(super) fn binding_name(ast: &mut Ast<'_>, id: BindingIdentifier) -> ExportName {
    ExportName {
        name: ast.add_utf8(ast.get_utf8(id.name(ast))),
        span: id.span(ast),
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L198>.
pub(super) fn default_export_name(ast: &mut Ast<'_>, span: Span) -> ExportName {
    ExportName {
        name: ast.add_utf8("default"),
        span,
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/wtf8.rs#L10>.
fn lossy_string(ast: &mut Ast<'_>, string: StringLiteral) -> AstUtf8 {
    let value = ast.get_wtf8(string.value(ast)).to_string_lossy();
    ast.add_utf8(&value)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L305>.
pub(super) fn declaration_bindings(ast: &Ast<'_>, decl: Decl) -> Vec<BindingIdentifier> {
    match ast.decl_data(decl) {
        DeclData::Class(class) => class.id(ast).into_iter().collect(),
        DeclData::Function(function) => function.id(ast).into_iter().collect(),
        DeclData::VariableDeclaration(var) => var
            .declarators(ast)
            .iter()
            .flat_map(|slot| build::find_pat_ids(ast, ast.get_node_in_sub_range(slot).id(ast)))
            .collect(),
        _ => Vec::new(),
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L154>.
pub(super) fn external_ts_import_equals_source(
    ast: &Ast<'_>,
    import: TsImportEqualsDeclaration,
) -> Option<StringLiteral> {
    if import.import_kind(ast) == ImportOrExportKind::Type {
        return None;
    }
    import
        .module_reference(ast)
        .as_ts_external_module_reference(ast)
        .map(|source| source.expression(ast))
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_record.rs#L845>.
pub(super) fn reduce(
    cx: &mut ModuleCtx<'_, '_>,
    module_entries: Vec<ModuleRecordEntry>,
    import_map: &mut ImportMap,
    export_bindings: &mut Vec<LocalExportEntry>,
    mod_ident: SymbolId,
) -> bool {
    let mut needs_binding = false;

    for entry in module_entries {
        match entry {
            ModuleRecordEntry::ImportNamed {
                import_name,
                local_name,
            } => {
                needs_binding = true;
                let local = cx.resolver.symbol_of(local_name.node_id());
                import_map.insert(
                    local,
                    ImportBinding {
                        mod_ident,
                        mod_prop: Some(import_name),
                    },
                );
            }
            ModuleRecordEntry::ImportDefault { local_name } => {
                needs_binding = true;
                let local = cx.resolver.symbol_of(local_name.node_id());
                let name = cx.ast.add_utf8("default");
                import_map.insert(
                    local,
                    ImportBinding {
                        mod_ident,
                        mod_prop: Some(name),
                    },
                );
            }
            ModuleRecordEntry::ImportNamespace { local_name } => {
                needs_binding = true;
                let local = cx.resolver.symbol_of(local_name.node_id());
                import_map.insert(
                    local,
                    ImportBinding {
                        mod_ident,
                        mod_prop: None,
                    },
                );
            }
            ModuleRecordEntry::IndirectExportNamed {
                import_name,
                export_name,
            } => {
                needs_binding = true;
                emit_indirect_export(
                    cx,
                    import_map,
                    export_bindings,
                    import_name,
                    export_name,
                    mod_ident,
                );
            }
            ModuleRecordEntry::IndirectExportDefault { export_name } => {
                needs_binding = true;
                let import_name = cx.ast.add_utf8("default");
                emit_indirect_export(
                    cx,
                    import_map,
                    export_bindings,
                    import_name,
                    export_name,
                    mod_ident,
                );
            }
            ModuleRecordEntry::IndirectExportNamespace { export_name } => {
                needs_binding = true;
                export_bindings.push(LocalExportEntry {
                    exported: export_name,
                    local: mod_ident,
                });
            }
            ModuleRecordEntry::StarExport => {}
        }
    }

    needs_binding
}

fn emit_indirect_export(
    cx: &mut ModuleCtx<'_, '_>,
    import_map: &mut ImportMap,
    export_bindings: &mut Vec<LocalExportEntry>,
    import_name: AstUtf8,
    export_name: ExportName,
    mod_ident: SymbolId,
) {
    let local = cx.private(cx.ast.get_utf8(import_name));
    let local = cx.resolver.symbol_of(local.node_id());
    import_map.insert(
        local,
        ImportBinding {
            mod_ident,
            mod_prop: Some(import_name),
        },
    );
    export_bindings.push(LocalExportEntry {
        exported: export_name,
        local,
    });
}
