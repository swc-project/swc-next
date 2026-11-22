use swc_next_ecma_ast::*;

use super::{Jsx, JsxRuntime, build::string};

impl Jsx<'_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L554>.
    pub(super) fn inject_runtime(&mut self, program: Program, ast: &mut Ast<'_>) {
        if self.runtime != JsxRuntime::Automatic {
            return;
        }
        // Each SWC injection prepends, so the runtime precedes createElement.
        if let Some(local) = self.import_create_element.take() {
            let source = self.import_source.clone();
            self.inject(program, vec![(local, "createElement")], &source, ast);
        }
        let mut imports = Vec::new();
        if let Some(local) = self.import_jsx.take() {
            imports.push((local, if self.development { "jsxDEV" } else { "jsx" }));
        }
        if !self.development
            && let Some(local) = self.import_jsxs.take()
        {
            imports.push((local, "jsxs"));
        }
        if let Some(local) = self.import_fragment.take() {
            imports.push((local, "Fragment"));
        }
        if !imports.is_empty() {
            let runtime = if self.development {
                "jsx-dev-runtime"
            } else {
                "jsx-runtime"
            };
            let source = format!("{}/{runtime}", self.import_source);
            self.inject(program, imports, &source, ast);
        }
    }

    /// SWC module injection: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1341>.
    /// SWC script injection: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1400>.
    /// SWC directive placement: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L346>.
    fn inject(
        &mut self,
        program: Program,
        imports: Vec<(BindingIdentifier, &str)>,
        source: &str,
        ast: &mut Ast<'_>,
    ) {
        let statement = if ast.is_module() {
            let specifiers = imports
                .into_iter()
                .map(|(local, imported)| {
                    let imported = ast.add_utf8(imported);
                    let imported =
                        ModuleExportName::IdentifierName(ast.identifier_name(DUMMY_SP, imported));
                    ImportDeclarationSpecifier::ImportSpecifier(ast.import_specifier(
                        DUMMY_SP,
                        imported,
                        local,
                        ImportOrExportKind::Value,
                    ))
                })
                .collect::<Vec<_>>();
            let specifiers = ast.add_sub_range(specifiers);
            let source = ast.add_utf8(source);
            let source = ast.string_literal(DUMMY_SP, source.into(), AstUtf8::EMPTY);
            ast.stmt_import_declaration(
                DUMMY_SP,
                specifiers,
                source,
                SubRange::empty(),
                None,
                ImportOrExportKind::Value,
            )
        } else {
            let properties = imports
                .into_iter()
                .map(|(local, imported)| {
                    let imported = ast.add_utf8(imported);
                    let key = PropertyKey::IdentifierName(ast.identifier_name(DUMMY_SP, imported));
                    ast.binding_property(
                        DUMMY_SP,
                        key,
                        BindingPattern::BindingIdentifier(local),
                        false,
                        false,
                    )
                })
                .collect::<Vec<_>>();
            let properties = ast.add_sub_range(properties);
            let pattern = ast.binding_pattern_object_pattern(DUMMY_SP, properties, None);
            let callee = self.resolver.marked_reference(ast, "require", true);
            let source = string(ast, DUMMY_SP, source);
            let arguments = ast.add_sub_range([Argument::Expr(source)]);
            let init = ast.expr_call_expression(DUMMY_SP, callee, None, arguments, false, false);
            let declarator = ast.variable_declarator(DUMMY_SP, pattern, None, Some(init), false);
            let declarations = ast.add_sub_range([declarator]);
            Stmt::Declaration(Decl::VariableDeclaration(ast.variable_declaration(
                DUMMY_SP,
                VariableKind::Const,
                declarations,
                false,
            )))
        };
        let mut body = vec![statement];
        body.extend(
            program
                .body(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id)),
        );
        let body = ast.add_sub_range(body);
        program.set_body(ast, body);
    }
}
