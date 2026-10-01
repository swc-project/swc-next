//! Namespace bodies, import pruning, and declaration scope markers.

use swc_next_ecma_ast::{
    Ast, Decl, DeclData, ImportDeclaration, ImportDeclarationSpecifier,
    ImportDeclarationSpecifierData, ImportOrExportKind, Span, Stmt, StmtData, SubRange,
    TsModuleBlock, TsModuleBody, TsModuleBodyData, TsModuleNameData,
};

use super::{DeclContext, Transformer};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ModuleScopeKind {
    Namespace,
    AmbientModule,
    Global,
}

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn transform_module_body(
        &mut self,
        body: Option<TsModuleBody>,
        ambient: bool,
        ast: &mut Ast<'ast>,
    ) {
        let Some(body) = body else {
            return;
        };
        match ast.ts_module_body_data(body) {
            TsModuleBodyData::TsModuleBlock(block) => {
                let (context, scope) = if ambient {
                    (DeclContext::AMBIENT_MODULE, ModuleScopeKind::AmbientModule)
                } else {
                    (DeclContext::NAMESPACE, ModuleScopeKind::Namespace)
                };
                self.transform_module_block(block, context, scope, ast);
            }
            TsModuleBodyData::TsModuleDeclaration(module) => {
                let context = if ambient {
                    DeclContext::AMBIENT_MODULE
                } else {
                    DeclContext::NAMESPACE
                };
                let declaration = Decl::TsModuleDeclaration(module);
                let _ = self.transform_decl(declaration, context, ast);
            }
        }
    }

    pub(super) fn transform_module_block(
        &mut self,
        block: TsModuleBlock,
        context: DeclContext,
        scope: ModuleScopeKind,
        ast: &mut Ast<'ast>,
    ) {
        let statements = self.transform_statements(block.body(ast), context, ast);
        let statements = self.normalize_module_statements(statements, scope, ast);
        let statements = ast.add_sub_range(statements);
        block.set_body(ast, statements);
    }

    fn normalize_module_statements(
        &self,
        mut statements: Vec<Stmt>,
        scope: ModuleScopeKind,
        ast: &mut Ast<'ast>,
    ) -> Vec<Stmt> {
        if scope == ModuleScopeKind::Global {
            return statements;
        }

        if statements.is_empty() && scope == ModuleScopeKind::Namespace {
            statements.push(self.empty_export_marker(ast));
            return statements;
        }

        let has_scope_marker = statements
            .iter()
            .any(|statement| match ast.stmt_data(*statement) {
                StmtData::ExportNamedDeclaration(export) => export.declaration(ast).is_none(),
                StmtData::ExportDefaultDeclaration(_)
                | StmtData::ExportAllDeclaration(_)
                | StmtData::TsExportAssignment(_) => true,
                _ => false,
            });
        if has_scope_marker {
            return statements;
        }

        let needs_scope_marker = scope == ModuleScopeKind::Namespace
            && statements
                .iter()
                .any(|statement| matches!(ast.stmt_data(*statement), StmtData::Declaration(_)))
            && statements.iter().any(|statement| {
                matches!(
                    ast.stmt_data(*statement),
                    StmtData::ExportNamedDeclaration(export) if export.declaration(ast).is_some()
                )
            });
        if needs_scope_marker {
            statements.push(self.empty_export_marker(ast));
            return statements;
        }

        statements
            .into_iter()
            .map(|statement| {
                let StmtData::ExportNamedDeclaration(export) = ast.stmt_data(statement) else {
                    return statement;
                };
                let Some(declaration) = export.declaration(ast) else {
                    return statement;
                };
                // Ambient declarations are implicitly exported, but import
                // aliases still need their explicit export modifier.
                if matches!(
                    ast.decl_data(declaration),
                    DeclData::TsImportEqualsDeclaration(_)
                ) {
                    return statement;
                }
                Stmt::Declaration(declaration)
            })
            .collect()
    }

    pub(super) fn transform_import(&self, import: ImportDeclaration, ast: &mut Ast<'ast>) -> bool {
        let specifiers = import.specifiers(ast);
        if specifiers.is_empty() {
            return true;
        }
        let mut remaining = specifiers.iter();
        let Some(first_unused) = remaining.position(|specifier| {
            !self.import_specifier_is_used(ast.get_node_in_sub_range(specifier), ast)
        }) else {
            return true;
        };
        let mut prefix = specifiers;
        let _ = prefix.split_off(first_unused);
        let mut retained = Vec::with_capacity(specifiers.len() - 1);
        retained.extend(
            prefix
                .iter()
                .map(|specifier| ast.get_node_in_sub_range(specifier)),
        );
        retained.extend(
            remaining
                .map(|specifier| ast.get_node_in_sub_range(specifier))
                .filter(|specifier| self.import_specifier_is_used(*specifier, ast)),
        );
        if retained.is_empty() {
            return false;
        }
        let specifiers = ast.add_sub_range(retained);
        import.set_specifiers(ast, specifiers);
        true
    }

    fn import_specifier_is_used(
        &self,
        specifier: ImportDeclarationSpecifier,
        ast: &Ast<'ast>,
    ) -> bool {
        let binding = match ast.import_declaration_specifier_data(specifier) {
            ImportDeclarationSpecifierData::ImportSpecifier(specifier) => specifier.local(ast),
            ImportDeclarationSpecifierData::ImportDefaultSpecifier(specifier) => {
                specifier.local(ast)
            }
            ImportDeclarationSpecifierData::ImportNamespaceSpecifier(specifier) => {
                specifier.local(ast)
            }
        };
        self.binding_is_used(binding)
    }

    pub(super) fn ensure_module_marker(
        &self,
        mut body: Vec<Stmt>,
        ast: &mut Ast<'ast>,
    ) -> Vec<Stmt> {
        let has_scope_marker = body
            .iter()
            .any(|statement| match ast.stmt_data(*statement) {
                StmtData::ExportNamedDeclaration(export) => {
                    export.declaration(ast).is_none() && export.source(ast).is_none()
                }
                StmtData::ExportDefaultDeclaration(_)
                | StmtData::ExportAllDeclaration(_)
                | StmtData::TsExportAssignment(_) => true,
                _ => false,
            });
        if has_scope_marker {
            return body;
        }
        let has_plain_declaration = body
            .iter()
            .any(|statement| match ast.stmt_data(*statement) {
                StmtData::Declaration(declaration) => match ast.decl_data(declaration) {
                    DeclData::TsGlobalDeclaration(_) => false,
                    DeclData::TsModuleDeclaration(module) => !matches!(
                        ast.ts_module_name_data(module.id(ast)),
                        TsModuleNameData::StringLiteral(_)
                    ),
                    _ => true,
                },
                _ => false,
            });
        if has_plain_declaration {
            body.push(self.empty_export_marker(ast));
            return body;
        }
        let has_module_indicator = body.iter().any(|statement| {
            matches!(
                ast.stmt_data(*statement),
                StmtData::ImportDeclaration(_)
                    | StmtData::ExportNamedDeclaration(_)
                    | StmtData::ExportDefaultDeclaration(_)
                    | StmtData::ExportAllDeclaration(_)
                    | StmtData::TsExportAssignment(_)
            )
        });
        if !has_module_indicator {
            body.push(self.empty_export_marker(ast));
        }
        body
    }

    fn empty_export_marker(&self, ast: &mut Ast<'ast>) -> Stmt {
        ast.stmt_export_named_declaration(
            Span::new(0, 0),
            None,
            SubRange::empty(),
            None,
            SubRange::empty(),
            ImportOrExportKind::Value,
        )
    }
}
