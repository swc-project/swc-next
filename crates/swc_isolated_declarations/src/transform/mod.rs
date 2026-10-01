//! Declaration transformation entry point and statement dispatch.

mod accessors;
mod classes;
mod declarations;
mod diagnostics;
mod enums;
mod expando;
mod inference;
mod modules;
mod objects;
mod properties;
mod signatures;
mod type_surface;

use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
    Ast, Diagnostic, ExportDefaultDeclarationKindData, GetSpan, NodeId, Stmt, StmtData, SubRange,
};
use swc_next_ecma_semantic::Semantic;

use crate::UsedSymbols;
use crate::surface::{DeclContext, DeclarationSurface, ExportExpression, export_expression};
use crate::type_usage::TypeUsageAnalyzer;

/// Transforms a semantically analyzed flat AST into isolated TypeScript declarations.
///
/// Returns diagnostics encountered during declaration generation.
///
/// The semantic model must have been built from `ast` before this function is called. The
/// transform consumes its symbol and reference indexes during declaration reachability analysis,
/// then mutates the AST in place. Re-run semantic analysis before performing later semantic work
/// on the transformed tree.
pub fn transform<'ast>(ast: &mut Ast<'ast>, semantic: &Semantic<'ast>) -> Vec<Diagnostic<'static>> {
    let (used, diagnostics, surface) = TypeUsageAnalyzer::analyze(ast, semantic);
    let mut transformer = Transformer {
        semantic,
        used,
        surface,
        diagnostics,
        next_default_id: 0,
        reserved_names: None,
        validated_type_methods: FxHashSet::default(),
    };
    transformer.transform_program(ast);
    transformer.diagnostics
}

struct Transformer<'semantic, 'ast> {
    semantic: &'semantic Semantic<'ast>,
    used: UsedSymbols,
    surface: DeclarationSurface,
    diagnostics: Vec<Diagnostic<'static>>,
    next_default_id: u32,
    reserved_names: Option<FxHashSet<&'ast [u8]>>,
    validated_type_methods: FxHashSet<NodeId>,
}

impl<'ast> Transformer<'_, 'ast> {
    fn transform_program(&mut self, ast: &mut Ast<'ast>) {
        let program = ast.root_program();
        let context = DeclContext::for_program(ast.source_type());
        let body = self.transform_statements(program.body(ast), context, ast);
        let body = if !DeclContext::for_program(ast.source_type()).retain_all {
            self.ensure_module_marker(body, ast)
        } else {
            body
        };
        let body = ast.add_sub_range(body);
        program.set_body(ast, body);
        // Run over emitted declarations so nested annotations and default interfaces share the
        // same retention, accessor, and computed-key rules as named declarations.
        self.transform_type_surfaces(program, ast);
    }

    fn transform_statements(
        &mut self,
        statements: SubRange<Stmt>,
        context: DeclContext,
        ast: &mut Ast<'ast>,
    ) -> Vec<Stmt> {
        self.report_expando_assignments(statements, context, ast);
        let mut transformed = Vec::with_capacity(statements.len());
        for statement in statements.iter() {
            let statement = ast.get_node_in_sub_range(statement);
            if self
                .surface
                .hidden_implementations
                .contains(&statement.node_id())
            {
                continue;
            }

            match ast.stmt_data(statement) {
                StmtData::Declaration(declaration) => {
                    if let Some(declaration) = self.transform_decl(declaration, context, ast) {
                        transformed.push(Stmt::Declaration(declaration));
                    }
                }
                StmtData::ImportDeclaration(declaration)
                    if self.transform_import(declaration, ast) =>
                {
                    transformed.push(statement);
                }
                StmtData::ImportDeclaration(_) => {}
                StmtData::ExportNamedDeclaration(export) => {
                    if let Some(declaration) = export.declaration(ast) {
                        let export_context = DeclContext {
                            retain_all: true,
                            ..context
                        };
                        let declaration = self.transform_decl(declaration, export_context, ast);
                        export.set_declaration(ast, declaration);
                    }
                    transformed.push(statement);
                }
                StmtData::ExportDefaultDeclaration(export) => {
                    let declaration = export.declaration(ast);
                    if let ExportDefaultDeclarationKindData::Expr(expression) =
                        ast.export_default_declaration_kind_data(declaration)
                        && !matches!(
                            export_expression(expression, ast),
                            ExportExpression::Reference(_)
                        )
                    {
                        let binding =
                            self.transform_export_expression_binding(expression, context, ast);
                        let reference = ast
                            .export_default_declaration_kind_expr_identifier_reference(
                                expression.span(ast),
                                binding.name,
                            );
                        export.set_declaration(ast, reference);
                        transformed.extend(binding.into_statements(statement));
                        continue;
                    }
                    let declaration = self.transform_default_declaration(declaration, context, ast);
                    export.set_declaration(ast, declaration);
                    transformed.push(statement);
                }
                StmtData::TsExportAssignment(export) => {
                    let expression = export.expression(ast);
                    if matches!(
                        export_expression(expression, ast),
                        ExportExpression::Reference(_)
                    ) {
                        transformed.push(statement);
                        continue;
                    }
                    let binding =
                        self.transform_export_expression_binding(expression, context, ast);
                    let reference =
                        ast.expr_identifier_reference(expression.span(ast), binding.name);
                    export.set_expression(ast, reference);
                    transformed.extend(binding.into_statements(statement));
                }
                StmtData::ExportAllDeclaration(_) => {
                    transformed.push(statement);
                }
                StmtData::TsNamespaceExportDeclaration(_) => {
                    transformed.push(statement);
                }
                _ => {}
            }
        }

        transformed
    }
}
