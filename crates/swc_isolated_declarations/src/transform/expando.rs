//! Diagnostics for function properties without matching namespace declarations.

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    AssignmentTargetData, Ast, BindingPatternData, Decl, DeclData, Diagnostic,
    ExportDefaultDeclarationKindData, ExprData, GetSpan, IdentifierReference, MemberExpression,
    SimpleAssignmentTargetData, Stmt, StmtData, SubRange, TsModuleBodyData, TsModuleNameData,
};

use super::diagnostics::EXPANDO_FUNCTION_NOT_SUPPORTED;
use super::{DeclContext, Transformer};
use crate::SymbolUsage;
use crate::surface::unparenthesized;

#[derive(Default)]
struct ExpandoFunction<'ast> {
    namespace_mergeable: bool,
    exported: bool,
    properties: FxHashSet<&'ast str>,
}

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn report_expando_assignments(
        &mut self,
        statements: SubRange<Stmt>,
        context: DeclContext,
        ast: &Ast<'ast>,
    ) {
        let has_assignment = statements
            .iter()
            .any(|statement| expando_member(ast.get_node_in_sub_range(statement), ast).is_some());
        if !has_assignment {
            return;
        }
        let mut functions = FxHashMap::<_, ExpandoFunction<'ast>>::default();

        for statement in statements.iter() {
            let statement = ast.get_node_in_sub_range(statement);
            if let StmtData::ExportDefaultDeclaration(export) = ast.stmt_data(statement) {
                if let ExportDefaultDeclarationKindData::Function(function) =
                    ast.export_default_declaration_kind_data(export.declaration(ast))
                    && let Some(binding) = function.id(ast)
                {
                    let name = ast.get_utf8(binding.name(ast));
                    let function = functions.entry(name).or_default();
                    function.namespace_mergeable = true;
                    function.exported = true;
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
                        let function = functions.entry(name).or_default();
                        function.namespace_mergeable = true;
                        function.exported |= exported;
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
                                let initializer = unparenthesized(initializer, ast);
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
                            let function = functions.entry(name).or_default();
                            function.exported |= exported;
                        }
                    }
                }
                _ => {}
            }
        }

        if functions.is_empty() {
            return;
        }
        let needs_properties = statements.iter().any(|statement| {
            let statement = ast.get_node_in_sub_range(statement);
            let Some((_, object)) = expando_member(statement, ast) else {
                return false;
            };
            functions
                .get(ast.get_utf8(object.name(ast)))
                .is_some_and(|function| function.namespace_mergeable)
        });
        if needs_properties {
            collect_namespace_properties(statements, &mut functions, ast);
        }

        for statement in statements.iter() {
            let statement = ast.get_node_in_sub_range(statement);
            let Some((member, object)) = expando_member(statement, ast) else {
                continue;
            };
            let name = ast.get_utf8(object.name(ast));
            let Some(function) = functions.get(name) else {
                continue;
            };
            let declared = function.namespace_mergeable
                && self
                    .property_key_name(member.property(ast), ast)
                    .is_some_and(|property| function.properties.contains(property.as_ref()));
            if !declared {
                self.diagnostics.push(Diagnostic::error(
                    EXPANDO_FUNCTION_NOT_SUPPORTED,
                    member.span(ast),
                ));
            }
        }
    }
}

fn expando_member(
    statement: Stmt,
    ast: &Ast<'_>,
) -> Option<(MemberExpression, IdentifierReference)> {
    let StmtData::ExpressionStatement(statement) = ast.stmt_data(statement) else {
        return None;
    };
    let expression = unparenthesized(statement.expression(ast), ast);
    let ExprData::AssignmentExpression(assignment) = ast.expr_data(expression) else {
        return None;
    };
    let AssignmentTargetData::SimpleAssignmentTarget(target) =
        ast.assignment_target_data(assignment.left(ast))
    else {
        return None;
    };
    let SimpleAssignmentTargetData::MemberExpression(member) =
        ast.simple_assignment_target_data(target)
    else {
        return None;
    };
    let object = unparenthesized(member.object(ast), ast);
    let ExprData::IdentifierReference(object) = ast.expr_data(object) else {
        return None;
    };
    Some((member, object))
}

fn collect_namespace_properties<'ast>(
    statements: SubRange<Stmt>,
    functions: &mut FxHashMap<&'ast str, ExpandoFunction<'ast>>,
    ast: &Ast<'ast>,
) {
    for statement in statements.iter() {
        let statement = ast.get_node_in_sub_range(statement);
        let (declaration, exported) = match ast.stmt_data(statement) {
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
        let TsModuleNameData::BindingIdentifier(binding) = ast.ts_module_name_data(module.id(ast))
        else {
            continue;
        };
        let name = ast.get_utf8(binding.name(ast));
        let Some(function) = functions.get_mut(name) else {
            continue;
        };
        if !function.namespace_mergeable || (function.exported && !exported) {
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
            if let Some(declaration) = declaration {
                collect_declaration_names(declaration, &mut function.properties, ast);
            }
        }
    }
}

fn collect_declaration_names<'ast>(
    declaration: Decl,
    names: &mut FxHashSet<&'ast str>,
    ast: &Ast<'ast>,
) {
    let binding = match ast.decl_data(declaration) {
        DeclData::VariableDeclaration(declaration) => {
            for declarator in declaration.declarators(ast).iter() {
                let declarator = ast.get_node_in_sub_range(declarator);
                if let BindingPatternData::BindingIdentifier(binding) =
                    ast.binding_pattern_data(declarator.id(ast))
                {
                    names.insert(ast.get_utf8(binding.name(ast)));
                }
            }
            return;
        }
        DeclData::Function(function) => function.id(ast),
        DeclData::Class(class) => class.id(ast),
        DeclData::TsFunction(function) => function.id(ast),
        DeclData::TsEnumDeclaration(declaration) => Some(declaration.id(ast)),
        _ => None,
    };
    if let Some(binding) = binding {
        names.insert(ast.get_utf8(binding.name(ast)));
    }
}
