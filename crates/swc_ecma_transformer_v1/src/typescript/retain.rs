//! Pre-erasure concreteness and declaration retention.
use swc_next_ecma_ast::*;

/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/retain.rs#L28>.
pub(super) fn should_retain_decl(ast: &Ast<'_>, declaration: Decl) -> bool {
    match ast.decl_data(declaration) {
        DeclData::TsInterfaceDeclaration(_)
        | DeclData::TsTypeAliasDeclaration(_)
        | DeclData::TsFunction(_)
        | DeclData::TsGlobalDeclaration(_) => false,
        DeclData::Class(class) => !class.declare(ast),
        DeclData::VariableDeclaration(var) => !var.declare(ast),
        DeclData::TsEnumDeclaration(enumeration) => !enumeration.declare(ast),
        DeclData::TsImportEqualsDeclaration(import) => {
            import.import_kind(ast) != ImportOrExportKind::Type
        }
        DeclData::TsModuleDeclaration(module) => {
            !module.declare(ast) && namespace_body_is_concrete(ast, module)
        }
        _ => true,
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/retain.rs#L50>.
pub(super) fn namespace_body_is_concrete(ast: &Ast<'_>, module: TsModuleDeclaration) -> bool {
    match module.body(ast).map(|body| ast.ts_module_body_data(body)) {
        Some(TsModuleBodyData::TsModuleBlock(block)) => block
            .body(ast)
            .iter()
            .any(|slot| is_concrete(ast, ast.get_node_in_sub_range(slot))),
        Some(TsModuleBodyData::TsModuleDeclaration(module)) => {
            namespace_body_is_concrete(ast, module)
        }
        None => false,
    }
}

fn concrete_decl(ast: &Ast<'_>, declaration: Decl) -> bool {
    match ast.decl_data(declaration) {
        DeclData::TsInterfaceDeclaration(_)
        | DeclData::TsTypeAliasDeclaration(_)
        | DeclData::TsFunction(_) => false,
        DeclData::TsModuleDeclaration(module) => namespace_body_is_concrete(ast, module),
        DeclData::TsImportEqualsDeclaration(import) => {
            import.import_kind(ast) == ImportOrExportKind::Value
        }
        DeclData::TsGlobalDeclaration(global) => global
            .body(ast)
            .body(ast)
            .iter()
            .any(|slot| is_concrete(ast, ast.get_node_in_sub_range(slot))),
        _ => true,
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/retain.rs#L74>.
pub(super) fn is_concrete(ast: &Ast<'_>, statement: Stmt) -> bool {
    match ast.stmt_data(statement) {
        StmtData::Declaration(declaration) => concrete_decl(ast, declaration),
        StmtData::ExportNamedDeclaration(export) => {
            export.export_kind(ast) != ImportOrExportKind::Type
                && export
                    .declaration(ast)
                    .is_none_or(|decl| concrete_decl(ast, decl))
        }
        StmtData::ImportDeclaration(import) => import.import_kind(ast) != ImportOrExportKind::Type,
        StmtData::ExportAllDeclaration(export) => {
            export.export_kind(ast) != ImportOrExportKind::Type
        }
        StmtData::TsNamespaceExportDeclaration(_) | StmtData::EmptyStatement(_) => false,
        StmtData::ExportDefaultDeclaration(export) => !matches!(
            ast.export_default_declaration_kind_data(export.declaration(ast)),
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(_)
                | ExportDefaultDeclarationKindData::TsFunction(_)
        ),
        _ => true,
    }
}
