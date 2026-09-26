//! Diagnostics for function properties without matching namespace declarations.

use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
    AssignmentTargetData, Ast, BindingPatternData, Decl, DeclData, Diagnostic,
    ExportDefaultDeclarationKindData, ExprData, GetSpan, SimpleAssignmentTargetData, Stmt,
    StmtData, TsModuleBodyData, TsModuleNameData,
};

use super::diagnostics::EXPANDO_FUNCTION_NOT_SUPPORTED;
use super::{DeclContext, Transformer};
use crate::SymbolUsage;

impl<'semantic> Transformer<'semantic> {
    pub(super) fn report_expando_assignments(
        &mut self,
        statements: &[Stmt],
        context: DeclContext,
        ast: &Ast<'_>,
    ) {
        let mut functions = FxHashSet::default();
        let mut mergeable_functions = FxHashSet::default();
        let mut exported_functions = FxHashSet::default();

        for statement in statements.iter().copied() {
            if let StmtData::ExportDefaultDeclaration(export) = ast.stmt_data(statement) {
                if let ExportDefaultDeclarationKindData::Function(function) =
                    ast.export_default_declaration_kind_data(export.declaration(ast))
                    && let Some(binding) = function.id(ast)
                {
                    let name = ast.get_utf8(binding.name(ast));
                    functions.insert(name);
                    mergeable_functions.insert(name);
                    exported_functions.insert(name);
                }
                continue;
            }
            let (declaration, exported) = match ast.stmt_data(statement) {
                StmtData::Declaration(declaration) => (Some(declaration), false),
                StmtData::ExportNamedDeclaration(export) => (export.declaration(ast), true),
                _ => (None, false),
            };
            let Some(declaration) = declaration else {
                continue;
            };
            match ast.decl_data(declaration) {
                DeclData::Function(function) => {
                    let Some(binding) = function.id(ast) else {
                        continue;
                    };
                    let name = ast.get_utf8(binding.name(ast));
                    let retained = exported
                        || context.retain_all
                        || self.binding_is_used_as(binding, SymbolUsage::VALUE);
                    if retained {
                        functions.insert(name);
                        mergeable_functions.insert(name);
                        if exported {
                            exported_functions.insert(name);
                        }
                    }
                }
                DeclData::VariableDeclaration(declaration) => {
                    for declarator in declaration.declarators(ast).iter() {
                        let declarator = ast.get_node_in_sub_range(declarator);
                        let BindingPatternData::BindingIdentifier(binding) =
                            ast.binding_pattern_data(declarator.id(ast))
                        else {
                            continue;
                        };
                        if declarator.type_annotation(ast).is_some()
                            || !declarator.init(ast).is_some_and(|initializer| {
                                let initializer = crate::surface::unparenthesized(initializer, ast);
                                matches!(
                                    ast.expr_data(initializer),
                                    ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
                                )
                            })
                        {
                            continue;
                        }
                        let retained = exported
                            || context.retain_all
                            || self.binding_is_used_as(binding, SymbolUsage::VALUE);
                        if retained {
                            let name = ast.get_utf8(binding.name(ast));
                            functions.insert(name);
                            if exported {
                                exported_functions.insert(name);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        for statement in statements.iter().copied() {
            let StmtData::ExpressionStatement(statement) = ast.stmt_data(statement) else {
                continue;
            };
            let expression = crate::surface::unparenthesized(statement.expression(ast), ast);
            let ExprData::AssignmentExpression(assignment) = ast.expr_data(expression) else {
                continue;
            };
            let AssignmentTargetData::SimpleAssignmentTarget(target) =
                ast.assignment_target_data(assignment.left(ast))
            else {
                continue;
            };
            let SimpleAssignmentTargetData::MemberExpression(member) =
                ast.simple_assignment_target_data(target)
            else {
                continue;
            };
            let object = crate::surface::unparenthesized(member.object(ast), ast);
            let ExprData::IdentifierReference(object) = ast.expr_data(object) else {
                continue;
            };
            let name = ast.get_utf8(object.name(ast));
            if !functions.contains(name) {
                continue;
            }
            let property = self.property_key_name(member.property(ast), ast);
            let declared = mergeable_functions.contains(name)
                && property.as_deref().is_some_and(|property| {
                    self.namespace_declares_exported_property(
                        statements,
                        name,
                        property,
                        exported_functions.contains(name),
                        ast,
                    )
                });
            if !declared {
                self.diagnostics.push(Diagnostic::error(
                    EXPANDO_FUNCTION_NOT_SUPPORTED,
                    member.span(ast),
                ));
            }
        }
    }

    fn namespace_declares_exported_property(
        &self,
        statements: &[Stmt],
        namespace_name: &str,
        property_name: &str,
        function_exported: bool,
        ast: &Ast<'_>,
    ) -> bool {
        for statement in statements.iter().copied() {
            let (declaration, namespace_exported) = match ast.stmt_data(statement) {
                StmtData::Declaration(declaration) => (Some(declaration), false),
                StmtData::ExportNamedDeclaration(export) => (export.declaration(ast), true),
                _ => (None, false),
            };
            let Some(declaration) = declaration else {
                continue;
            };
            let DeclData::TsModuleDeclaration(module) = ast.decl_data(declaration) else {
                continue;
            };
            let TsModuleNameData::BindingIdentifier(binding) =
                ast.ts_module_name_data(module.id(ast))
            else {
                continue;
            };
            if ast.get_utf8(binding.name(ast)) != namespace_name {
                continue;
            }
            if function_exported && !namespace_exported {
                continue;
            }
            let Some(body) = module.body(ast) else {
                continue;
            };
            let TsModuleBodyData::TsModuleBlock(block) = ast.ts_module_body_data(body) else {
                continue;
            };
            for statement in block.body(ast).iter() {
                let statement = ast.get_node_in_sub_range(statement);
                let declaration = match ast.stmt_data(statement) {
                    StmtData::ExportNamedDeclaration(export) => export.declaration(ast),
                    StmtData::Declaration(declaration) if module.declare(ast) => Some(declaration),
                    _ => None,
                };
                if declaration.is_some_and(|declaration| {
                    self.declaration_has_binding_name(declaration, property_name, ast)
                }) {
                    return true;
                }
            }
        }
        false
    }

    fn declaration_has_binding_name(&self, declaration: Decl, name: &str, ast: &Ast<'_>) -> bool {
        match ast.decl_data(declaration) {
            DeclData::VariableDeclaration(declaration) => declaration
                .declarators(ast)
                .iter()
                .map(|declarator| ast.get_node_in_sub_range(declarator))
                .any(|declarator| {
                    matches!(
                        ast.binding_pattern_data(declarator.id(ast)),
                        BindingPatternData::BindingIdentifier(binding)
                            if ast.get_utf8(binding.name(ast)) == name
                    )
                }),
            DeclData::Function(function) => function
                .id(ast)
                .is_some_and(|binding| ast.get_utf8(binding.name(ast)) == name),
            DeclData::Class(class) => class
                .id(ast)
                .is_some_and(|binding| ast.get_utf8(binding.name(ast)) == name),
            DeclData::TsFunction(function) => function
                .id(ast)
                .is_some_and(|binding| ast.get_utf8(binding.name(ast)) == name),
            DeclData::TsEnumDeclaration(declaration) => {
                ast.get_utf8(declaration.id(ast).name(ast)) == name
            }
            _ => false,
        }
    }
}
