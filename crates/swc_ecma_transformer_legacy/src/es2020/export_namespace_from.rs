use crate::common::resolver::context::Resolver;
use swc_next_ecma_ast::*;

/// Next represents SWC's export-namespace specifier as `ExportAllDeclaration`.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2020/export_namespace_from.rs#L19>.
pub(crate) fn pass(ast: &mut Ast<'_>, resolver: &mut Resolver) {
    if !ast.is_module() {
        return;
    }
    let program = ast.root_program();
    ExportNamespaceFromPass { ast, resolver }.transform_export_namespace_from(program);
}

struct ExportNamespaceFromPass<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
}

impl ExportNamespaceFromPass<'_, '_> {
    fn transform_export_namespace_from(&mut self, program: Program) {
        let ast = &mut *self.ast;
        let resolver = &mut *self.resolver;
        let mut body = Vec::new();
        for slot in program.body(ast).iter() {
            let statement = ast.get_node_in_sub_range(slot);
            let Some(export) = statement.as_export_all_declaration(ast).filter(|export| {
                export.export_kind(ast) == ImportOrExportKind::Value
                    && export.exported(ast).is_some()
            }) else {
                body.push(statement);
                continue;
            };
            let name = export.exported(ast).unwrap();
            let text = match ast.module_export_name_data(name) {
                ModuleExportNameData::IdentifierName(id) => ast.get_utf8(id.name(ast)).to_owned(),
                ModuleExportNameData::StringLiteral(string) => ast
                    .get_wtf8(string.value(ast))
                    .to_string_lossy()
                    .into_owned(),
            };
            let binding = resolver.private_binding(ast, &format!("_{text}"));
            let specifier = ast.import_namespace_specifier(export.span(ast), binding);
            let specifiers =
                ast.add_sub_range([ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                    specifier,
                )]);
            body.push(Stmt::ImportDeclaration(ast.import_declaration(
                export.span(ast),
                specifiers,
                export.source(ast),
                export.attributes(ast),
                None,
                ImportOrExportKind::Value,
            )));
            let local = ast.identifier_name(export.span(ast), binding.name(ast));
            resolver.register(ast);
            resolver.node_scopes[local.node_id()] = resolver.node_scopes[binding.node_id()];
            let specifier = ast.export_specifier(
                export.span(ast),
                ModuleExportName::IdentifierName(local),
                name,
                ImportOrExportKind::Value,
            );
            let specifiers = ast.add_sub_range([specifier]);
            body.push(Stmt::ExportNamedDeclaration(ast.export_named_declaration(
                export.span(ast),
                None,
                specifiers,
                None,
                SubRange::empty(),
                ImportOrExportKind::Value,
            )));
        }
        let body = ast.add_sub_range(body);
        program.set_body(ast, body);
    }
}
