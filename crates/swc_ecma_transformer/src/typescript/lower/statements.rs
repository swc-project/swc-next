use super::TypeScriptLowerer;
use swc_next_ecma_ast::{
    DUMMY_SP, Decl, DeclData, ImportOrExportKind, Program, Stmt, StmtData, SubRange,
    TsModuleNameData, VisitMut, VisitMutWith,
};

pub(super) enum StatementExpansion {
    Remove,
    Single(Stmt),
    Pair(Stmt, Stmt),
}

impl StatementExpansion {
    pub(super) fn with_prefix(prefix: Option<Stmt>, statement: Stmt) -> Self {
        match prefix {
            Some(prefix) => Self::Pair(prefix, statement),
            None => Self::Single(statement),
        }
    }

    pub(super) fn extend_into(self, output: &mut Vec<Stmt>) {
        match self {
            Self::Remove => {}
            Self::Single(statement) => output.push(statement),
            Self::Pair(first, second) => output.extend([first, second]),
        }
    }
}

impl TypeScriptLowerer<'_, '_> {
    pub(super) fn has_module_indicator(&self, statements: SubRange<Stmt>) -> bool {
        statements.iter().any(|statement_id| {
            let statement = self.ast_ref().get_node_in_sub_range(statement_id);
            statement.is_module_indicator(self.ast_ref())
        })
    }

    pub(super) fn append_esm_marker(&mut self, program: Program) {
        let body = program.body(self.ast_ref());
        let mut statements = Vec::with_capacity(body.len() + 1);
        for statement_id in body.iter() {
            statements.push(self.ast_ref().get_node_in_sub_range(statement_id));
        }
        let marker = self.ast().stmt_export_named_declaration(
            DUMMY_SP,
            None,
            SubRange::empty(),
            None,
            SubRange::empty(),
            ImportOrExportKind::Value,
        );
        statements.push(marker);
        let body = self.ast().add_sub_range(statements);
        program.set_body(self.ast(), body);
    }

    pub(super) fn lower_statement_list(
        &mut self,
        statements: SubRange<Stmt>,
    ) -> Option<SubRange<Stmt>> {
        self.enter_binding_scope();
        if self.analysis.has_merged_runtime_bindings() {
            self.register_scope_runtime_bindings(statements);
        }
        let lowered = self.lower_statement_list_in_current_scope(statements);
        self.exit_binding_scope();
        lowered
    }

    pub(super) fn lower_statement_list_in_current_scope(
        &mut self,
        statements: SubRange<Stmt>,
    ) -> Option<SubRange<Stmt>> {
        debug_assert!(!self.runtime_binding_scopes.is_empty());

        let mut lowered: Option<Vec<Stmt>> = None;
        for (index, statement_id) in statements.iter().enumerate() {
            let statement = self.ast().get_node_in_sub_range(statement_id);
            if let Some(replacement) = self.lower_statement(statement) {
                let output = lowered.get_or_insert_with(|| {
                    let mut prefix = Vec::with_capacity(statements.len());
                    prefix.extend(
                        statements
                            .iter()
                            .take(index)
                            .map(|id| self.ast_ref().get_node_in_sub_range(id)),
                    );
                    prefix
                });
                replacement.extend_into(output);
            } else if let Some(output) = &mut lowered {
                output.push(statement);
            }
        }

        lowered.map(|statements| self.ast().add_sub_range(statements))
    }

    // None preserves the statement handle; children are still lowered in place.
    pub(super) fn lower_statement(&mut self, statement: Stmt) -> Option<StatementExpansion> {
        match self.ast_ref().stmt_data(statement) {
            StmtData::Declaration(declaration) => {
                if let Some(namespace) = declaration.as_ts_module_declaration(self.ast_ref()) {
                    return Some(self.lower_namespace(namespace, false));
                }
                if let Some(enum_declaration) = declaration.as_ts_enum_declaration(self.ast_ref()) {
                    return Some(self.lower_enum(enum_declaration, false));
                }
                if let Some(import_equals) =
                    declaration.as_ts_import_equals_declaration(self.ast_ref())
                {
                    let replacement = self
                        .lower_import_equals(import_equals, false)
                        .map_or(StatementExpansion::Remove, StatementExpansion::Single);
                    return Some(replacement);
                }
                if self.should_drop_declaration(declaration) {
                    return Some(StatementExpansion::Remove);
                }
            }
            StmtData::ImportDeclaration(import) if !self.lower_import_declaration(import) => {
                return Some(StatementExpansion::Remove);
            }
            StmtData::ExportNamedDeclaration(export) => {
                if let Some(namespace) = export
                    .declaration(self.ast_ref())
                    .and_then(|declaration| declaration.as_ts_module_declaration(self.ast_ref()))
                {
                    return Some(self.lower_namespace(namespace, true));
                }
                if let Some(enum_declaration) = export
                    .declaration(self.ast_ref())
                    .and_then(|declaration| declaration.as_ts_enum_declaration(self.ast_ref()))
                {
                    return Some(self.lower_enum(enum_declaration, true));
                }
                if let Some(import_equals) =
                    export.declaration(self.ast_ref()).and_then(|declaration| {
                        declaration.as_ts_import_equals_declaration(self.ast_ref())
                    })
                {
                    let replacement = self
                        .lower_import_equals(import_equals, true)
                        .map_or(StatementExpansion::Remove, StatementExpansion::Single);
                    return Some(replacement);
                }
                if export
                    .declaration(self.ast_ref())
                    .is_some_and(|declaration| self.should_drop_declaration(declaration))
                    || !self.lower_export_named_declaration(export)
                {
                    return Some(StatementExpansion::Remove);
                }
            }
            StmtData::ExportDefaultDeclaration(export) => {
                let declaration = export.declaration(self.ast_ref());
                if self.export_default_is_erased(export)
                    || declaration.is_ts_function(self.ast_ref())
                    || declaration.is_ts_interface_declaration(self.ast_ref())
                    || declaration
                        .as_class(self.ast_ref())
                        .is_some_and(|class| class.declare(self.ast_ref()))
                {
                    return Some(StatementExpansion::Remove);
                }
            }
            StmtData::ExportAllDeclaration(export) => {
                if export.export_kind(self.ast_ref()) == ImportOrExportKind::Type {
                    return Some(StatementExpansion::Remove);
                }
                export.set_export_kind(self.ast(), ImportOrExportKind::Value);
            }
            StmtData::TsNamespaceExportDeclaration(_) => return Some(StatementExpansion::Remove),
            _ => {}
        }

        statement.visit_mut_children_with(self);
        None
    }

    fn should_drop_declaration(&self, declaration: Decl) -> bool {
        let ast = self.ast_ref();
        match ast.decl_data(declaration) {
            DeclData::VariableDeclaration(declaration) => declaration.declare(ast),
            DeclData::Function(_) => false,
            DeclData::Class(class) => class.declare(ast),
            DeclData::TsFunction(_)
            | DeclData::TsTypeAliasDeclaration(_)
            | DeclData::TsInterfaceDeclaration(_)
            | DeclData::TsGlobalDeclaration(_) => true,
            DeclData::TsEnumDeclaration(declaration) => declaration.declare(ast),
            DeclData::TsModuleDeclaration(declaration) => {
                declaration.declare(ast)
                    || declaration.body(ast).is_none()
                    || matches!(
                        ast.ts_module_name_data(declaration.id(ast)),
                        TsModuleNameData::StringLiteral(_)
                    )
            }
            DeclData::TsImportEqualsDeclaration(declaration) => {
                declaration.import_kind(ast) == ImportOrExportKind::Type
            }
        }
    }
}
