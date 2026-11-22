//! SWC's TypeScript import/export-assignment policy.
use super::{TsImportExportAssignConfig, transform::Transform, utils};
use swc_next_ecma_ast::*;

impl Transform<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1625>.
    pub(super) fn entity_expression(&mut self, reference: TsModuleReference) -> Expr {
        fn qualified(ast: &mut Ast<'_>, name: TsQualifiedName) -> Expr {
            let object = match ast.ts_qualified_name_left_data(name.left(ast)) {
                TsQualifiedNameLeftData::IdentifierReference(id) => Expr::IdentifierReference(id),
                TsQualifiedNameLeftData::TsQualifiedName(name) => qualified(ast, name),
                _ => unreachable!("import aliases use identifier entity names"),
            };
            let key = name.right(ast);
            ast.expr_member_expression(
                name.span(ast),
                object,
                PropertyKey::IdentifierName(key),
                false,
                false,
            )
        }
        let name = reference.as_ts_type_name(self.ast).unwrap();
        match self.ast.ts_type_name_data(name) {
            TsTypeNameData::IdentifierReference(id) => Expr::IdentifierReference(id),
            TsTypeNameData::TsQualifiedName(name) => qualified(self.ast, name),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1678>.
    pub(super) fn import_export_assignments(&mut self, body: &mut Vec<Stmt>) {
        let create_require = self.resolver.private_binding(self.ast, "_createRequire");
        let require = self.resolver.private_binding(self.ast, "__require");
        let mut inject = false;
        let mut assignment = None;
        for statement in body.iter_mut() {
            let (import, exported) = match self.ast.stmt_data(*statement) {
                StmtData::Declaration(declaration) => {
                    (declaration.as_ts_import_equals_declaration(self.ast), false)
                }
                StmtData::ExportNamedDeclaration(export) => (
                    export.declaration(self.ast).and_then(|declaration| {
                        declaration.as_ts_import_equals_declaration(self.ast)
                    }),
                    true,
                ),
                StmtData::TsExportAssignment(export) => {
                    assignment.get_or_insert(export);
                    *statement = self.empty();
                    continue;
                }
                _ => (None, false),
            };
            let Some(import) = import else { continue };
            if import.import_kind(self.ast) == ImportOrExportKind::Type {
                continue;
            }
            let reference = import.module_reference(self.ast);
            if reference.is_ts_type_name(self.ast) {
                let init = self.entity_expression(reference);
                let declaration = utils::variable(
                    self.ast,
                    VariableKind::Const,
                    import.id(self.ast),
                    Some(init),
                    if exported {
                        DUMMY_SP
                    } else {
                        import.span(self.ast)
                    },
                );
                *statement = if exported {
                    utils::export(self.ast, declaration)
                } else {
                    Stmt::Declaration(declaration)
                };
                continue;
            }
            let external = reference.as_ts_external_module_reference(self.ast).unwrap();
            let source = Expr::StringLiteral(external.expression(self.ast));
            match self.options.import_export_assign_config {
                TsImportExportAssignConfig::Classic => {
                    let callee = self.resolver.marked_reference(self.ast, "require", true);
                    let mut init = utils::call(self.ast, callee, [source], false);
                    if exported {
                        let exports = self.resolver.marked_reference(self.ast, "exports", true);
                        let left = utils::member(
                            self.ast,
                            exports,
                            &self.binding_id(import.id(self.ast)).name,
                        );
                        init = utils::assign(self.ast, left, init);
                    }
                    *statement = Stmt::Declaration(utils::variable(
                        self.ast,
                        VariableKind::Const,
                        import.id(self.ast),
                        Some(init),
                        import.span(self.ast),
                    ));
                }
                TsImportExportAssignConfig::Preserve => {}
                TsImportExportAssignConfig::NodeNext => {
                    inject = true;
                    let callee = self.resolver.reference(self.ast, require);
                    let init = utils::call(self.ast, callee, [source], false);
                    let declaration = utils::variable(
                        self.ast,
                        VariableKind::Const,
                        import.id(self.ast),
                        Some(init),
                        if exported {
                            DUMMY_SP
                        } else {
                            import.span(self.ast)
                        },
                    );
                    *statement = if exported {
                        utils::export(self.ast, declaration)
                    } else {
                        Stmt::Declaration(declaration)
                    };
                }
                TsImportExportAssignConfig::EsNext => self.diagnostics.push(Diagnostic::error(
                    "Import assignment cannot be used when targeting ECMAScript modules.",
                    import.span(self.ast),
                )),
            }
        }
        if inject {
            let name = self.ast.add_utf8("createRequire");
            let imported =
                ModuleExportName::IdentifierName(self.ast.identifier_name(DUMMY_SP, name));
            let specifier = self.ast.import_specifier(
                DUMMY_SP,
                imported,
                create_require,
                ImportOrExportKind::Value,
            );
            let specifiers = self
                .ast
                .add_sub_range([ImportDeclarationSpecifier::ImportSpecifier(specifier)]);
            let source = self.ast.add_utf8("module");
            let source = self
                .ast
                .string_literal(DUMMY_SP, source.into(), AstUtf8::EMPTY);
            let import = Stmt::ImportDeclaration(self.ast.import_declaration(
                DUMMY_SP,
                specifiers,
                source,
                SubRange::empty(),
                None,
                ImportOrExportKind::Value,
            ));
            let import_name = self.ast.add_utf8("import");
            let meta_name = self.ast.add_utf8("meta");
            let import_name = self.ast.identifier_name(DUMMY_SP, import_name);
            let meta_name = self.ast.identifier_name(DUMMY_SP, meta_name);
            let meta = self
                .ast
                .expr_meta_property(DUMMY_SP, import_name, meta_name);
            let url = utils::member(self.ast, meta, "url");
            let callee = self.resolver.reference(self.ast, create_require);
            let init = utils::call(self.ast, callee, [url], false);
            let declaration = Stmt::Declaration(utils::variable(
                self.ast,
                VariableKind::Const,
                require,
                Some(init),
                DUMMY_SP,
            ));
            body.splice(0..0, [import, declaration]);
        }
        if let Some(export) = assignment {
            match self.options.import_export_assign_config {
                TsImportExportAssignConfig::Classic => {
                    let module = self.resolver.marked_reference(self.ast, "module", true);
                    let left = utils::member(self.ast, module, "exports");
                    let right = export.expression(self.ast);
                    let expression = utils::assign(self.ast, left, right);
                    let statement = self
                        .ast
                        .stmt_expression_statement(export.span(self.ast), expression);
                    if let Some(last) = body
                        .last_mut()
                        .filter(|statement| statement.is_empty_statement(self.ast))
                    {
                        *last = statement;
                    } else {
                        body.push(statement);
                    }
                }
                TsImportExportAssignConfig::Preserve => body.push(Stmt::TsExportAssignment(export)),
                TsImportExportAssignConfig::NodeNext | TsImportExportAssignConfig::EsNext => {
                    self.diagnostics.push(Diagnostic::error(
                        "Export assignment cannot be used when targeting ECMAScript modules.",
                        export.span(self.ast),
                    ))
                }
            }
        }
    }
}
