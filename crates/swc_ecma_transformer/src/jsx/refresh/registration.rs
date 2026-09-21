use super::{super::ReactRefreshOptions, build::*};
use crate::uid::UidGenerator;
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
    Argument, Ast, AstUtf8, CallExpression, DUMMY_SP, Decl, DeclData,
    ExportDefaultDeclarationKindData, Expr, ExprData, GetSpan, JsxElementName, JsxElementNameData,
    JsxMemberExpressionObjectData, JsxOpeningElement, NodeId, Program, Stmt, StmtData, SymbolId,
    TsModuleBodyData, TsModuleDeclaration, TsModuleNameData, TsQualifiedName,
    TsQualifiedNameLeftData, VariableDeclaration, VariableKind, Visit, VisitWith,
};

use swc_next_ecma_semantic::Semantic;

struct Registration {
    binding: AstUtf8,
    persistent_id: AstUtf8,
}

pub(super) fn transform<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: &ReactRefreshOptions,
    uid: &mut UidGenerator<'a>,
) {
    let used_in_jsx = UsedInJsx::collect(program, ast, semantic);
    let mut refresh = ReactRefresh {
        ast,
        options,
        uid,
        registrations: Vec::new(),
        used_in_jsx,
        semantic,
        persistent_prefix: None,
    };
    refresh.transform_program(program);
}

struct UsedInJsx<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    semantic: &'visit Semantic<'ast>,
    symbols: FxHashSet<SymbolId>,
}

impl<'visit, 'ast> UsedInJsx<'visit, 'ast> {
    fn collect(
        program: Program,
        ast: &'visit Ast<'ast>,
        semantic: &'visit Semantic<'ast>,
    ) -> FxHashSet<SymbolId> {
        let mut collector = Self {
            ast,
            semantic,
            symbols: FxHashSet::default(),
        };
        program.visit_with(&mut collector);
        collector.symbols
    }

    fn add_reference(&mut self, node: NodeId) {
        let symbol = self
            .semantic
            .reference_of(node)
            .and_then(|reference| self.semantic.reference(reference).symbol);
        if let Some(symbol) = symbol {
            self.symbols.insert(symbol);
        }
    }

    fn jsx_name_root(&self, name: JsxElementName) -> Option<NodeId> {
        match self.ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => Some(identifier.node_id()),
            JsxElementNameData::JsxMemberExpression(member) => {
                let mut object = member.object(self.ast);
                loop {
                    match self.ast.jsx_member_expression_object_data(object) {
                        JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                            return Some(identifier.node_id());
                        }
                        JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                            object = member.object(self.ast);
                        }
                    }
                }
            }
            JsxElementNameData::JsxNamespacedName(_) => None,
        }
    }
}

impl<'ast> Visit<'ast> for UsedInJsx<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_jsx_opening_element(&mut self, opening: JsxOpeningElement) {
        if let Some(root) = self.jsx_name_root(opening.name(self.ast)) {
            self.add_reference(root);
        }
        opening.visit_children_with(self);
    }

    fn visit_call_expression(&mut self, call: CallExpression) {
        let callee_name = match self
            .ast
            .expr_data(runtime_expression(call.callee(self.ast), self.ast))
        {
            ExprData::IdentifierReference(identifier) => {
                Some(self.ast.get_utf8(identifier.name(self.ast)))
            }
            ExprData::MemberExpression(member) => member
                .property(self.ast)
                .as_identifier_name(self.ast)
                .map(|identifier| self.ast.get_utf8(identifier.name(self.ast))),
            _ => None,
        };
        let jsx_like = callee_name
            .is_some_and(|name| matches!(name, "createElement" | "jsx" | "jsxs" | "jsxDEV"));
        if jsx_like
            && let Some(argument) = call.arguments(self.ast).iter().next()
            && let Some(expression) = self.ast.get_node_in_sub_range(argument).as_expr(self.ast)
            && let ExprData::IdentifierReference(identifier) =
                self.ast.expr_data(runtime_expression(expression, self.ast))
        {
            self.add_reference(identifier.node_id());
        }
        call.visit_children_with(self);
    }
}

struct ReactRefresh<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    options: &'visit ReactRefreshOptions,
    uid: &'visit mut UidGenerator<'ast>,
    registrations: Vec<Registration>,
    used_in_jsx: FxHashSet<SymbolId>,
    semantic: &'visit Semantic<'ast>,
    persistent_prefix: Option<AstUtf8>,
}

impl<'ast> ReactRefresh<'_, 'ast> {
    fn transform_program(&mut self, program: Program) {
        let existing = program.body(self.ast);
        let mut body = Vec::with_capacity(existing.len() * 2 + 2);
        for id in existing.iter() {
            let statement = self.ast.get_node_in_sub_range(id);
            let trailing = self.process_statement(statement);
            body.push(statement);
            if let Some(trailing) = trailing {
                body.push(trailing);
            }
        }
        self.append_registrations(&mut body);
        let body = self.ast.add_sub_range(body);
        program.set_body(self.ast, body);
    }

    fn process_statement(&mut self, statement: Stmt) -> Option<Stmt> {
        match self.ast.stmt_data(statement) {
            StmtData::Declaration(declaration) => self.process_declaration(declaration),
            StmtData::ExportNamedDeclaration(export) => export
                .declaration(self.ast)
                .and_then(|declaration| self.process_declaration(declaration)),
            StmtData::ExportDefaultDeclaration(export) => {
                match self
                    .ast
                    .export_default_declaration_kind_data(export.declaration(self.ast))
                {
                    ExportDefaultDeclarationKindData::Function(function) => {
                        let id = function.id(self.ast)?;
                        let name = self.ast.get_utf8(id.name(self.ast));
                        is_componentish_name(name).then(|| {
                            self.registration_assignment(id.name(self.ast), id.name(self.ast))
                        })
                    }
                    ExportDefaultDeclarationKindData::Expr(expression) => {
                        let ExprData::CallExpression(_) =
                            self.ast.expr_data(runtime_expression(expression, self.ast))
                        else {
                            return None;
                        };
                        let persistent_id = self.ast.add_utf8("%default%");
                        let (replacement, found) =
                            self.replace_inner_components(persistent_id, expression, false);
                        if found {
                            export.set_declaration(
                                self.ast,
                                swc_next_ecma_ast::ExportDefaultDeclarationKind::Expr(replacement),
                            );
                        }
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn process_declaration(&mut self, declaration: Decl) -> Option<Stmt> {
        match self.ast.decl_data(declaration) {
            DeclData::Function(function) => {
                let id = function.id(self.ast)?;
                let name = self.ast.get_utf8(id.name(self.ast));
                if !is_componentish_name(name) {
                    return None;
                }
                let persistent_id = self.component_id(id.name(self.ast));
                Some(self.registration_assignment(id.name(self.ast), persistent_id))
            }
            DeclData::VariableDeclaration(variable) => self.process_variable(variable),
            DeclData::TsModuleDeclaration(module) => {
                self.process_namespace(module, self.persistent_prefix.is_none());
                None
            }
            _ => None,
        }
    }

    fn process_namespace(&mut self, module: TsModuleDeclaration, enabled: bool) {
        if !enabled || module.declare(self.ast) {
            return;
        }
        let Some(name) = self.namespace_name(module) else {
            return;
        };
        let prefix = self.component_id(name);
        let Some(body) = module.body(self.ast) else {
            return;
        };
        match self.ast.ts_module_body_data(body) {
            TsModuleBodyData::TsModuleDeclaration(module) => {
                let previous = self.persistent_prefix.replace(prefix);
                self.process_namespace(module, true);
                self.persistent_prefix = previous;
            }
            TsModuleBodyData::TsModuleBlock(block) => {
                let existing = block.body(self.ast);
                let previous = self.persistent_prefix.replace(prefix);
                let mut transformed = Vec::with_capacity(existing.len() * 2);
                for id in existing.iter() {
                    let statement = self.ast.get_node_in_sub_range(id);
                    let trailing = match self.ast.stmt_data(statement) {
                        StmtData::Declaration(declaration) => {
                            if let DeclData::TsModuleDeclaration(module) =
                                self.ast.decl_data(declaration)
                            {
                                self.process_namespace(module, false);
                                None
                            } else {
                                self.process_declaration(declaration)
                            }
                        }
                        StmtData::ExportNamedDeclaration(export) => {
                            export.declaration(self.ast).and_then(|declaration| {
                                if let DeclData::TsModuleDeclaration(module) =
                                    self.ast.decl_data(declaration)
                                {
                                    self.process_namespace(module, true);
                                    None
                                } else {
                                    self.process_declaration(declaration)
                                }
                            })
                        }
                        _ => None,
                    };
                    transformed.push(statement);
                    if let Some(trailing) = trailing {
                        transformed.push(trailing);
                    }
                }
                self.persistent_prefix = previous;
                let transformed = self.ast.add_sub_range(transformed);
                block.set_body(self.ast, transformed);
            }
        }
    }

    fn namespace_name(&mut self, module: TsModuleDeclaration) -> Option<AstUtf8> {
        match self.ast.ts_module_name_data(module.id(self.ast)) {
            TsModuleNameData::BindingIdentifier(binding) => Some(binding.name(self.ast)),
            TsModuleNameData::TsQualifiedName(qualified) => {
                let name = self.qualified_namespace_name(qualified)?;
                Some(self.ast.add_utf8(&name))
            }
            TsModuleNameData::StringLiteral(_) => None,
        }
    }

    fn qualified_namespace_name(&self, qualified: TsQualifiedName) -> Option<String> {
        let mut name = match self
            .ast
            .ts_qualified_name_left_data(qualified.left(self.ast))
        {
            TsQualifiedNameLeftData::BindingIdentifier(binding) => {
                self.ast.get_utf8(binding.name(self.ast)).to_owned()
            }
            TsQualifiedNameLeftData::TsQualifiedName(qualified) => {
                self.qualified_namespace_name(qualified)?
            }
            TsQualifiedNameLeftData::IdentifierName(_)
            | TsQualifiedNameLeftData::IdentifierReference(_)
            | TsQualifiedNameLeftData::ThisExpression(_) => return None,
        };
        name.push('$');
        let right = qualified.right(self.ast);
        name.push_str(self.ast.get_utf8(right.name(self.ast)));
        Some(name)
    }

    fn component_id(&mut self, name: AstUtf8) -> AstUtf8 {
        let Some(prefix) = self.persistent_prefix else {
            return name;
        };
        let prefix = self.ast.get_utf8(prefix);
        let name = self.ast.get_utf8(name);
        let persistent_id = format!("{prefix}${name}");
        self.ast.add_utf8(&persistent_id)
    }

    fn process_variable(&mut self, declaration: VariableDeclaration) -> Option<Stmt> {
        if declaration.declarators(self.ast).len() != 1 {
            return None;
        }
        let id = declaration.declarators(self.ast).iter().next()?;
        let declarator = self.ast.get_node_in_sub_range(id);
        let binding = declarator.id(self.ast).as_binding_identifier(self.ast)?;
        let name = self.ast.get_utf8(binding.name(self.ast));
        if !is_componentish_name(name) {
            return None;
        }
        let init = declarator.init(self.ast)?;
        if is_arrow_returning_arrow(init, self.ast) {
            return None;
        }
        if is_require_like(init, self.ast) {
            return None;
        }
        let likely_component = matches!(
            self.ast.expr_data(runtime_expression(init, self.ast)),
            ExprData::Function(_)
                | ExprData::ArrowFunctionExpression(_)
                | ExprData::TaggedTemplateExpression(_)
                | ExprData::CallExpression(_)
        );
        let used_in_jsx = self
            .semantic
            .symbol_of(binding.node_id())
            .is_some_and(|symbol| self.used_in_jsx.contains(&symbol));
        let used_component_factory = used_in_jsx
            && matches!(
                self.ast.expr_data(runtime_expression(init, self.ast)),
                ExprData::TaggedTemplateExpression(_) | ExprData::CallExpression(_)
            );
        if !likely_component && !used_component_factory {
            return None;
        }
        let persistent_id = self.component_id(binding.name(self.ast));
        let (replacement, found) = self.replace_inner_components(persistent_id, init, true);
        if !found && !used_component_factory {
            return None;
        }
        if replacement != init {
            declarator.set_init(self.ast, Some(replacement));
        }
        Some(self.registration_assignment(binding.name(self.ast), persistent_id))
    }

    fn replace_inner_components(
        &mut self,
        inferred_name: AstUtf8,
        expression: Expr,
        is_variable_declarator: bool,
    ) -> (Expr, bool) {
        match self.ast.expr_data(runtime_expression(expression, self.ast)) {
            ExprData::IdentifierReference(identifier) => {
                let name = self.ast.get_utf8(identifier.name(self.ast));
                (expression, is_componentish_name(name))
            }
            ExprData::Function(_) | ExprData::ArrowFunctionExpression(_) => {
                if is_arrow_returning_arrow(expression, self.ast) {
                    return (expression, false);
                }
                if is_variable_declarator {
                    (expression, true)
                } else {
                    (self.wrap_registration(expression, inferred_name), true)
                }
            }
            ExprData::CallExpression(call) => {
                if !allowed_hoc_callee(call.callee(self.ast), self.ast) {
                    return (expression, false);
                }
                let Some(first_id) = call.arguments(self.ast).iter().next() else {
                    return (expression, false);
                };
                let first = self.ast.get_node_in_sub_range(first_id);
                let Some(first) = first.as_expr(self.ast) else {
                    return (expression, false);
                };
                let callee_span = call.callee(self.ast).span(self.ast);
                let callee = self
                    .ast
                    .get_utf8(self.ast.source_slice(callee_span.start, callee_span.end));
                let inferred_name_text = self.ast.get_utf8(inferred_name);
                let nested_name = format!("{inferred_name_text}${callee}");
                let nested_name = self.ast.add_utf8(&nested_name);
                let (first, found) = self.replace_inner_components(nested_name, first, false);
                if !found {
                    return (expression, false);
                }
                call.arguments(self.ast)
                    .replace_slot(self.ast, first_id, Argument::Expr(first));
                if is_variable_declarator {
                    (expression, true)
                } else {
                    (self.wrap_registration(expression, inferred_name), true)
                }
            }
            _ => (expression, false),
        }
    }

    fn wrap_registration(&mut self, expression: Expr, persistent_id: AstUtf8) -> Expr {
        let binding = self.create_registration(persistent_id);
        let left = self
            .ast
            .assignment_target_simple_assignment_target_identifier_reference(DUMMY_SP, binding);
        self.ast.expr_assignment_expression(
            DUMMY_SP,
            left,
            expression,
            swc_next_ecma_ast::AssignmentOperator::Assign,
        )
    }

    fn registration_assignment(&mut self, name: AstUtf8, persistent_id: AstUtf8) -> Stmt {
        let binding = self.create_registration(persistent_id);
        let left = self
            .ast
            .assignment_target_simple_assignment_target_identifier_reference(DUMMY_SP, binding);
        let right = self.ast.expr_identifier_reference(DUMMY_SP, name);
        let assignment = self.ast.expr_assignment_expression(
            DUMMY_SP,
            left,
            right,
            swc_next_ecma_ast::AssignmentOperator::Assign,
        );
        Stmt::ExpressionStatement(self.ast.expression_statement(DUMMY_SP, assignment))
    }

    fn create_registration(&mut self, persistent_id: AstUtf8) -> AstUtf8 {
        let binding = self.uid.fresh("_c", self.ast);
        self.registrations.push(Registration {
            binding,
            persistent_id,
        });
        binding
    }

    fn append_registrations(&mut self, body: &mut Vec<Stmt>) {
        if self.registrations.is_empty() {
            return;
        }
        let mut declarators = Vec::with_capacity(self.registrations.len());
        for registration in &self.registrations {
            let id = self
                .ast
                .binding_pattern_binding_identifier(DUMMY_SP, registration.binding);
            declarators.push(
                self.ast
                    .variable_declarator(DUMMY_SP, id, None, None, false),
            );
        }
        let declarators = self.ast.add_sub_range(declarators);
        let declaration =
            self.ast
                .variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false);
        body.push(Stmt::Declaration(Decl::VariableDeclaration(declaration)));

        for registration in &self.registrations {
            let callee = qualified_name_expression(self.ast, &self.options.refresh_reg);
            let binding = self
                .ast
                .expr_identifier_reference(DUMMY_SP, registration.binding);
            let persistent_id = registration.persistent_id;
            let persistent_id =
                self.ast
                    .expr_string_literal(DUMMY_SP, persistent_id.into(), AstUtf8::EMPTY);
            let arguments = self
                .ast
                .add_sub_range([Argument::Expr(binding), Argument::Expr(persistent_id)]);
            let call = self
                .ast
                .expr_call_expression(DUMMY_SP, callee, None, arguments, false, false);
            body.push(Stmt::ExpressionStatement(
                self.ast.expression_statement(DUMMY_SP, call),
            ));
        }
    }
}

fn allowed_hoc_callee(expression: Expr, ast: &Ast<'_>) -> bool {
    matches!(
        ast.expr_data(runtime_expression(expression, ast)),
        ExprData::IdentifierReference(_) | ExprData::MemberExpression(_)
    )
}

fn is_require_like(expression: Expr, ast: &Ast<'_>) -> bool {
    let ExprData::CallExpression(call) = ast.expr_data(runtime_expression(expression, ast)) else {
        return false;
    };
    match ast.expr_data(runtime_expression(call.callee(ast), ast)) {
        ExprData::IdentifierReference(identifier) => {
            ast.get_utf8(identifier.name(ast)).starts_with("require")
        }
        ExprData::ImportExpression(_) => true,
        _ => false,
    }
}

fn is_arrow_returning_arrow(expression: Expr, ast: &Ast<'_>) -> bool {
    let ExprData::ArrowFunctionExpression(arrow) =
        ast.expr_data(runtime_expression(expression, ast))
    else {
        return false;
    };
    arrow.body(ast).as_expr(ast).is_some_and(|body| {
        matches!(
            ast.expr_data(runtime_expression(body, ast)),
            ExprData::ArrowFunctionExpression(_)
        )
    })
}

fn is_componentish_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
}
