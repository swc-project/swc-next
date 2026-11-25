//! External SWC runtime helper references and injection.

use std::collections::BTreeSet;

use swc_next_ecma_ast::*;

use crate::common::build;
use crate::common::resolver::context::Resolver;

#[derive(Default)]
pub(crate) struct Helpers {
    // SWC injects helpers in alphabetical order, regardless of first use.
    used: BTreeSet<&'static str>,
    scope: Option<ScopeId>,
}

impl Helpers {
    pub(crate) fn enable(&mut self, resolver: &mut Resolver, name: &'static str) -> ScopeId {
        self.used.insert(name);
        *self.scope.get_or_insert_with(|| resolver.fresh_scope())
    }

    pub(crate) fn scope(&self) -> Option<ScopeId> {
        self.scope
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/helpers/mod.rs#L15>.
    pub(crate) fn helper_expr(
        &mut self,
        ast: &mut Ast<'_>,
        resolver: &mut Resolver,
        name: &'static str,
    ) -> Expr {
        let scope = self.enable(resolver, name);
        let symbol = resolver.scoped_symbol(ast, scope, &format!("_{name}"));
        resolver.symbol_reference(ast, symbol)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/helpers/mod.rs#L219>.
    pub(crate) fn inject_helpers(&self, ast: &mut Ast<'_>, resolver: &mut Resolver) {
        let Some(scope) = self.scope else { return };
        let program = ast.root_program();
        let module = ast.is_module();
        let mut pass = InjectHelpers {
            ast,
            resolver,
            scope,
        };
        let statements = if module {
            pass.build_imports(&self.used)
        } else {
            pass.build_requires(&self.used)
        };
        let body = build::prepend_stmts(pass.ast, statements, program.body(pass.ast));
        program.set_body(pass.ast, body);
        if !module {
            program.visit_mut_with(&mut pass);
        }
    }
}

struct InjectHelpers<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    scope: ScopeId,
}

impl InjectHelpers<'_, '_> {
    fn build_imports(&mut self, used: &BTreeSet<&str>) -> Vec<Stmt> {
        used.iter()
            .map(|name| {
                let (binding, source) = self.binding_and_source(name);
                let name = self.ast.add_utf8("_");
                let imported =
                    ModuleExportName::IdentifierName(self.ast.identifier_name(DUMMY_SP, name));
                let specifier = self.ast.import_specifier(
                    DUMMY_SP,
                    imported,
                    binding,
                    ImportOrExportKind::Value,
                );
                let specifiers = self
                    .ast
                    .add_sub_range([ImportDeclarationSpecifier::ImportSpecifier(specifier)]);
                self.ast.stmt_import_declaration(
                    DUMMY_SP,
                    specifiers,
                    source,
                    SubRange::empty(),
                    None,
                    ImportOrExportKind::Value,
                )
            })
            .collect()
    }

    fn build_requires(&mut self, used: &BTreeSet<&str>) -> Vec<Stmt> {
        used.iter().map(|name| self.build_require(name)).collect()
    }

    fn build_require(&mut self, name: &str) -> Stmt {
        let (binding, source) = self.binding_and_source(name);
        let require = self.resolver.marked_reference(self.ast, "require", true);
        let init = build::call(self.ast, require, [Expr::StringLiteral(source)], false);
        Stmt::Declaration(build::variable(
            self.ast,
            VariableKind::Var,
            binding,
            Some(init),
            DUMMY_SP,
        ))
    }

    fn binding_and_source(&mut self, name: &str) -> (BindingIdentifier, StringLiteral) {
        let name = format!("_{name}");
        let symbol = self.resolver.scoped_symbol(self.ast, self.scope, &name);
        let binding = self.resolver.binding(self.ast, symbol);
        let path = self.ast.add_utf8(&format!("@swc/helpers/_/{name}"));
        let source = self
            .ast
            .string_literal(DUMMY_SP, path.into(), AstUtf8::EMPTY);
        (binding, source)
    }

    fn map_helper_ref_ident(&mut self, id: IdentifierReference) -> Option<Expr> {
        if self.resolver.scope_of(id.node_id()) == Some(self.scope) {
            let expression = Expr::IdentifierReference(id);
            let span = id.span(self.ast);
            let member = build::member(self.ast, expression, "_");
            member.set_span(self.ast, span);
            return Some(member);
        }
        None
    }
}

impl<'ast> VisitMut<'ast> for InjectHelpers<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        if let Some(id) = expression.as_identifier_reference(self.ast) {
            return self.map_helper_ref_ident(id);
        }
        expression.visit_mut_children_with(self);
        None
    }
}
