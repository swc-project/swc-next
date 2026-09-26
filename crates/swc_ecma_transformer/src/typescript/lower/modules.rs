use swc_next_ecma_ast::{
    BindingIdentifier, BindingPattern, Decl, ExportNamedDeclaration, Expr, GetSpan,
    ImportDeclaration, ImportDeclarationSpecifierData, ImportOrExportKind, PropertyKey, Stmt,
    SubRange, TsImportEqualsDeclaration, TsModuleReferenceData, TsQualifiedName,
    TsQualifiedNameLeft, TsQualifiedNameLeftData, TsTypeName, TsTypeNameData, VariableKind,
    VisitMut, VisitMutWith,
};

use super::TypeScriptLowerer;

impl TypeScriptLowerer<'_, '_> {
    pub(super) fn lower_import_declaration(&mut self, declaration: ImportDeclaration) -> bool {
        if declaration.import_kind(self.ast_ref()) == ImportOrExportKind::Type {
            return false;
        }

        let specifiers = declaration.specifiers(self.ast_ref());
        let mut lowered: Option<Vec<_>> = None;
        for (index, specifier_id) in specifiers.iter().enumerate() {
            let specifier = self.ast_ref().get_node_in_sub_range(specifier_id);
            let data = self.ast_ref().import_declaration_specifier_data(specifier);
            let (local, explicit_type) = match data {
                ImportDeclarationSpecifierData::ImportSpecifier(specifier) => (
                    specifier.local(self.ast_ref()),
                    specifier.import_kind(self.ast_ref()) == ImportOrExportKind::Type,
                ),
                ImportDeclarationSpecifierData::ImportDefaultSpecifier(specifier) => {
                    (specifier.local(self.ast_ref()), false)
                }
                ImportDeclarationSpecifierData::ImportNamespaceSpecifier(specifier) => {
                    (specifier.local(self.ast_ref()), false)
                }
            };
            let keep = !explicit_type
                && (self.options.verbatim_module_syntax || self.binding_has_runtime_usage(local));
            if keep {
                if let ImportDeclarationSpecifierData::ImportSpecifier(specifier) = data {
                    specifier.set_import_kind(self.ast(), ImportOrExportKind::Value);
                }
                if let Some(output) = &mut lowered {
                    output.push(specifier);
                }
            } else {
                lowered.get_or_insert_with(|| {
                    let mut prefix = Vec::with_capacity(specifiers.len() - 1);
                    prefix.extend(
                        specifiers
                            .iter()
                            .take(index)
                            .map(|id| self.ast_ref().get_node_in_sub_range(id)),
                    );
                    prefix
                });
            }
        }

        if let Some(lowered) = lowered {
            if lowered.is_empty() && !self.options.verbatim_module_syntax {
                return false;
            }
            let specifiers = self.ast().add_sub_range(lowered);
            declaration.set_specifiers(self.ast(), specifiers);
        }
        declaration.set_import_kind(self.ast(), ImportOrExportKind::Value);
        true
    }

    pub(super) fn lower_export_named_declaration(
        &mut self,
        declaration: ExportNamedDeclaration,
    ) -> bool {
        if declaration.export_kind(self.ast_ref()) == ImportOrExportKind::Type {
            return false;
        }

        let specifiers = declaration.specifiers(self.ast_ref());
        let mut lowered: Option<Vec<_>> = None;
        for (index, specifier_id) in specifiers.iter().enumerate() {
            let specifier = self.ast_ref().get_node_in_sub_range(specifier_id);
            let explicit_type = specifier.export_kind(self.ast_ref()) == ImportOrExportKind::Type;
            let inferred_erasure = !self.options.verbatim_module_syntax
                && declaration.source(self.ast_ref()).is_none()
                && self.export_local_is_erased(specifier.local(self.ast_ref()));
            if !explicit_type && !inferred_erasure {
                specifier.set_export_kind(self.ast(), ImportOrExportKind::Value);
                if let Some(output) = &mut lowered {
                    output.push(specifier);
                }
            } else {
                lowered.get_or_insert_with(|| {
                    let mut prefix = Vec::with_capacity(specifiers.len() - 1);
                    prefix.extend(
                        specifiers
                            .iter()
                            .take(index)
                            .map(|id| self.ast_ref().get_node_in_sub_range(id)),
                    );
                    prefix
                });
            }
        }

        if let Some(lowered) = lowered {
            if lowered.is_empty() && !self.options.verbatim_module_syntax {
                return false;
            }
            let specifiers = self.ast().add_sub_range(lowered);
            declaration.set_specifiers(self.ast(), specifiers);
        }
        declaration.set_export_kind(self.ast(), ImportOrExportKind::Value);
        true
    }

    fn export_local_is_erased(&self, local: swc_next_ecma_ast::ModuleExportName) -> bool {
        let Some(identifier) = local.as_identifier_name(self.ast_ref()) else {
            return false;
        };
        self.export_reference_is_erased(identifier.node_id())
    }

    pub(super) fn export_default_is_erased(
        &self,
        declaration: swc_next_ecma_ast::ExportDefaultDeclaration,
    ) -> bool {
        if self.options.verbatim_module_syntax {
            return false;
        }
        let Some(mut expression) = declaration
            .declaration(self.ast_ref())
            .as_expr(self.ast_ref())
        else {
            return false;
        };
        while let swc_next_ecma_ast::ExprData::ParenthesizedExpression(parenthesized) =
            self.ast_ref().expr_data(expression)
        {
            expression = parenthesized.expression(self.ast_ref());
        }
        if expression.as_identifier_reference(self.ast_ref()).is_none() {
            return false;
        }
        self.export_reference_is_erased(expression.node_id())
    }

    fn export_reference_is_erased(&self, node: swc_next_ecma_ast::NodeId) -> bool {
        let Some(id) = self.semantic.reference_of(node) else {
            return false;
        };
        self.semantic.reference(id).symbol.is_some()
            && self
                .semantic
                .reference_value_targets(self.ast_ref(), id)
                .all(|target| {
                    target
                        .symbol()
                        .is_some_and(|symbol| self.analysis.value_is_erased(symbol, self.semantic))
                })
    }

    fn binding_has_runtime_usage(&self, binding: BindingIdentifier) -> bool {
        self.semantic
            .symbol_of(binding.node_id())
            .is_none_or(|symbol| self.analysis.has_runtime_usage(symbol, self.semantic))
    }

    pub(super) fn import_equals_is_erased(&self, declaration: TsImportEqualsDeclaration) -> bool {
        self.analysis.import_equals_is_erased(declaration.node_id())
    }

    pub(super) fn lower_import_equals(
        &mut self,
        declaration: TsImportEqualsDeclaration,
        exported: bool,
    ) -> Option<Stmt> {
        if self.import_equals_is_erased(declaration) {
            return None;
        }
        let binding = declaration.id(self.ast_ref());
        if !exported
            && !self.options.verbatim_module_syntax
            && !self.binding_has_runtime_usage(binding)
        {
            return None;
        }

        let module_reference = declaration.module_reference(self.ast_ref());
        let kind = match self.ast_ref().ts_module_reference_data(module_reference) {
            TsModuleReferenceData::TsTypeName(_) => VariableKind::Var,
            TsModuleReferenceData::TsExternalModuleReference(_) => VariableKind::Const,
        };
        let initializer = self.lower_import_equals_initializer(declaration);
        let initializer = initializer.visit_mut_with(self).unwrap_or(initializer);
        let span = declaration.span(self.ast_ref());
        let declarator = self.ast().variable_declarator(
            span,
            BindingPattern::BindingIdentifier(binding),
            None,
            Some(initializer),
            false,
        );
        let declarators = self.ast().add_sub_range([declarator]);
        let declaration = self
            .ast()
            .variable_declaration(span, kind, declarators, false);
        let declaration = Decl::VariableDeclaration(declaration);
        if exported {
            Some(self.ast().stmt_export_named_declaration(
                span,
                Some(declaration),
                SubRange::empty(),
                None,
                SubRange::empty(),
                ImportOrExportKind::Value,
            ))
        } else {
            Some(Stmt::Declaration(declaration))
        }
    }

    pub(super) fn lower_import_equals_initializer(
        &mut self,
        declaration: TsImportEqualsDeclaration,
    ) -> Expr {
        let module_reference = declaration.module_reference(self.ast_ref());
        match self.ast_ref().ts_module_reference_data(module_reference) {
            TsModuleReferenceData::TsTypeName(type_name) => self.lower_type_name(type_name),
            TsModuleReferenceData::TsExternalModuleReference(reference) => {
                let span = reference.span(self.ast_ref());
                let source = Expr::StringLiteral(reference.expression(self.ast_ref()));
                self.ast()
                    .expr_import_expression(span, source, None, None, true)
            }
        }
    }

    fn lower_type_name(&mut self, type_name: TsTypeName) -> Expr {
        match self.ast_ref().ts_type_name_data(type_name) {
            TsTypeNameData::IdentifierReference(identifier) => {
                Expr::IdentifierReference(identifier)
            }
            TsTypeNameData::TsQualifiedName(qualified) => self.lower_qualified_name(qualified),
        }
    }

    fn lower_qualified_name(&mut self, qualified: TsQualifiedName) -> Expr {
        let left = qualified.left(self.ast_ref());
        let right = qualified.right(self.ast_ref());
        let span = qualified.span(self.ast_ref());
        let object = self.lower_qualified_name_left(left);
        self.ast().expr_member_expression(
            span,
            object,
            PropertyKey::IdentifierName(right),
            false,
            false,
        )
    }

    fn lower_qualified_name_left(&mut self, left: TsQualifiedNameLeft) -> Expr {
        match self.ast_ref().ts_qualified_name_left_data(left) {
            TsQualifiedNameLeftData::IdentifierReference(identifier) => {
                Expr::IdentifierReference(identifier)
            }
            TsQualifiedNameLeftData::IdentifierName(identifier) => {
                let span = identifier.span(self.ast_ref());
                let name = identifier.name(self.ast_ref());
                Expr::IdentifierReference(self.ast().identifier_reference(span, name))
            }
            TsQualifiedNameLeftData::BindingIdentifier(identifier) => {
                let span = identifier.span(self.ast_ref());
                let name = identifier.name(self.ast_ref());
                Expr::IdentifierReference(self.ast().identifier_reference(span, name))
            }
            TsQualifiedNameLeftData::ThisExpression(this) => Expr::ThisExpression(this),
            TsQualifiedNameLeftData::TsQualifiedName(qualified) => {
                self.lower_qualified_name(qualified)
            }
        }
    }
}
