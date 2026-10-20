use super::{
    super::ReactRefreshOptions,
    build::*,
    digest::sha1_base64,
    hooks::{CustomHook, HookCollector, HookSignature},
};
use crate::uid::UidGenerator;
use swc_next_ecma_ast::{
    Argument, ArrowFunctionBody, ArrowFunctionExpression, AssignmentOperator, Ast, AstUtf8,
    DUMMY_SP, Decl, Expr, ExprData, Function, FunctionBody, FunctionType, GetSpan, IfStatement,
    NodeId, ObjectProperty, PropertyKind, Stmt, StmtData, SubRange, SwitchCase,
    VariableDeclaration, VariableKind, VisitMut, VisitMutWith, VisitWith,
};

use swc_next_ecma_semantic::Semantic;

struct Signature {
    binding: AstUtf8,
    key: String,
    force_reset: bool,
    custom_hooks: Vec<CustomHook>,
}

struct ExpressionSignature {
    expression: Expr,
    signature: Signature,
}

#[derive(Default)]
struct SignatureFrame {
    declarations: Vec<(AstUtf8, Expr)>,
    pending: Vec<Stmt>,
}

pub(super) struct SignatureTransformer<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    semantic: &'visit Semantic<'ast>,
    options: &'visit ReactRefreshOptions,
    uid: UidGenerator<'ast>,
    frame: SignatureFrame,
    in_statement_list: bool,
    statement_declaration: Option<VariableDeclaration>,
    direct_initializer: Option<(NodeId, AstUtf8)>,
    last_signature: Option<ExpressionSignature>,
    program_signature_declaration: Option<Stmt>,
}

impl<'visit, 'ast> SignatureTransformer<'visit, 'ast> {
    pub(super) fn new(
        ast: &'visit mut Ast<'ast>,
        semantic: &'visit Semantic<'ast>,
        options: &'visit ReactRefreshOptions,
        uid: UidGenerator<'ast>,
    ) -> Self {
        Self {
            ast,
            semantic,
            options,
            uid,
            frame: SignatureFrame::default(),
            in_statement_list: false,
            statement_declaration: None,
            direct_initializer: None,
            last_signature: None,
            program_signature_declaration: None,
        }
    }

    pub(super) fn program_signature_declaration(&self) -> Option<Stmt> {
        self.program_signature_declaration
    }

    fn collect_hooks(&self, body: ArrowFunctionBody, boundary: NodeId) -> Option<HookSignature> {
        // Read source spans and semantic references before child transforms replace them.
        let mut collector = HookCollector::new(self.ast, self.semantic, boundary);
        body.visit_with(&mut collector);
        collector.into_signature()
    }

    fn transform_function_body(&mut self, body: FunctionBody, hooks: HookSignature) -> Signature {
        let signature = self.create_signature(hooks);
        let call = call_expression(self.ast, signature.binding, []);
        let call = Stmt::ExpressionStatement(self.ast.expression_statement(DUMMY_SP, call));
        let existing = body.body(self.ast);
        let mut statements = Vec::with_capacity(existing.len() + 1);
        statements.push(call);
        statements.extend(existing.iter().map(|id| self.ast.get_node_in_sub_range(id)));
        let statements = self.ast.add_sub_range(statements);
        body.set_body(self.ast, statements);
        signature
    }

    fn transform_arrow(
        &mut self,
        arrow: ArrowFunctionExpression,
        hooks: HookSignature,
    ) -> Option<Signature> {
        if let Some(body) = arrow.body(self.ast).as_function_body(self.ast) {
            return Some(self.transform_function_body(body, hooks));
        }

        let expression = arrow.body(self.ast).as_expr(self.ast)?;
        let return_statement = self.ast.return_statement(DUMMY_SP, Some(expression));
        let signature = self.create_signature(hooks);
        let call = call_expression(self.ast, signature.binding, []);
        let call = Stmt::ExpressionStatement(self.ast.expression_statement(DUMMY_SP, call));
        let statements = self
            .ast
            .add_sub_range([call, Stmt::ReturnStatement(return_statement)]);
        let body = self
            .ast
            .function_body(DUMMY_SP, SubRange::empty(), statements);
        arrow.set_body(self.ast, ArrowFunctionBody::FunctionBody(body));
        arrow.set_expression(self.ast, false);
        Some(signature)
    }

    fn create_signature(&mut self, hooks: HookSignature) -> Signature {
        let HookSignature {
            key,
            force_reset,
            custom_hooks,
        } = hooks;
        let binding = self.uid.fresh("_s", self.ast);
        let callee = qualified_name_expression(self.ast, &self.options.refresh_sig);
        let arguments = SubRange::empty();
        let init = self
            .ast
            .expr_call_expression(DUMMY_SP, callee, None, arguments, false, false);
        self.frame.declarations.push((binding, init));
        let key = if self.options.emit_full_signatures {
            key
        } else {
            sha1_base64(&key)
        };
        Signature {
            binding,
            key,
            force_reset,
            custom_hooks,
        }
    }

    fn signature_call(&mut self, signature: &Signature, component: Expr) -> Stmt {
        let call = self.signature_expression(signature, component);
        Stmt::ExpressionStatement(self.ast.expression_statement(DUMMY_SP, call))
    }

    fn signature_expression(&mut self, signature: &Signature, component: Expr) -> Expr {
        let mut arguments = Vec::with_capacity(4);
        arguments.push(Argument::Expr(component));
        let key = self.ast.add_utf8(&signature.key);
        let key = self
            .ast
            .expr_string_literal(DUMMY_SP, key.into(), AstUtf8::EMPTY);
        arguments.push(Argument::Expr(key));
        if signature.force_reset || !signature.custom_hooks.is_empty() {
            arguments.push(Argument::Expr(
                self.ast
                    .expr_boolean_literal(DUMMY_SP, signature.force_reset),
            ));
        }
        if !signature.custom_hooks.is_empty() {
            let elements = signature
                .custom_hooks
                .iter()
                .map(|hook| Some(Argument::Expr(hook.create_expression(self.ast))))
                .collect::<Vec<_>>();
            let elements = self.ast.add_sub_range(elements);
            let array = self.ast.expr_array_expression(DUMMY_SP, elements);
            let return_statement = self.ast.return_statement(DUMMY_SP, Some(array));
            let statements = self
                .ast
                .add_sub_range([Stmt::ReturnStatement(return_statement)]);
            let body = self
                .ast
                .function_body(DUMMY_SP, SubRange::empty(), statements);
            let params = self.ast.formal_parameters(
                DUMMY_SP,
                SubRange::empty(),
                None,
                swc_next_ecma_ast::FormalParameterKind::FormalParameters,
            );
            let callback = self.ast.expr_function(
                DUMMY_SP,
                FunctionType::FunctionExpression,
                None,
                None,
                None,
                params,
                None,
                body,
                false,
                false,
            );
            arguments.push(Argument::Expr(callback));
        }
        call_expression(self.ast, signature.binding, arguments)
    }

    fn queue_signature(&mut self, signature: &Signature, name: AstUtf8) {
        let component = self.ast.expr_identifier_reference(DUMMY_SP, name);
        let call = self.signature_call(signature, component);
        self.frame.pending.push(call);
    }

    fn direct_initializer(&self, expression: Expr) -> Option<AstUtf8> {
        let (initializer, name) = self.direct_initializer?;
        (runtime_expression(expression, self.ast).node_id() == initializer).then_some(name)
    }

    fn wrap_signature(&mut self, signature: Signature, expression: Expr) -> Expr {
        let expression = self.signature_expression(&signature, expression);
        self.last_signature = Some(ExpressionSignature {
            expression,
            signature,
        });
        expression
    }

    fn forward_signature(&mut self, expression: Expr) {
        let Some(mut signature) = self.last_signature.take() else {
            return;
        };
        let inner = match self.ast.expr_data(expression) {
            ExprData::AssignmentExpression(assignment)
                if assignment.operator(self.ast) == AssignmentOperator::Assign =>
            {
                assignment.right(self.ast)
            }
            ExprData::ParenthesizedExpression(wrapper) => wrapper.expression(self.ast),
            ExprData::TsAsExpression(wrapper) => wrapper.expression(self.ast),
            ExprData::TsSatisfiesExpression(wrapper) => wrapper.expression(self.ast),
            ExprData::TsNonNullExpression(wrapper) => wrapper.expression(self.ast),
            ExprData::TsTypeAssertion(wrapper) => wrapper.expression(self.ast),
            ExprData::TsInstantiationExpression(wrapper) => wrapper.expression(self.ast),
            _ => return,
        };
        if signature.expression == inner {
            signature.expression = expression;
            self.last_signature = Some(signature);
        }
    }

    fn transform_branch(&mut self, statement: Stmt) -> Stmt {
        // Initialize helpers in the enclosing list, but keep associations inside the branch.
        let outer_pending = std::mem::take(&mut self.frame.pending);
        let outer_declaration = self.statement_declaration.take();
        let statement = statement.visit_mut_with(self).unwrap_or(statement);
        let pending = std::mem::replace(&mut self.frame.pending, outer_pending);
        self.statement_declaration = outer_declaration;
        self.last_signature = None;
        if pending.is_empty() {
            return statement;
        }
        let mut body = Vec::with_capacity(pending.len() + 1);
        body.push(statement);
        body.extend(pending);
        let body = self.ast.add_sub_range(body);
        Stmt::BlockStatement(self.ast.block_statement(statement.span(self.ast), body))
    }

    fn transform_statements(
        &mut self,
        statements: SubRange<Stmt>,
        initialize_signatures: bool,
    ) -> Option<SubRange<Stmt>> {
        let outer_frame = std::mem::take(&mut self.frame);
        let nested = std::mem::replace(&mut self.in_statement_list, true);
        let outer_declaration = self.statement_declaration.take();
        let mut transformed: Option<Vec<Stmt>> = None;
        for (index, id) in statements.iter().enumerate() {
            let original = self.ast.get_node_in_sub_range(id);
            self.statement_declaration = match self.ast.stmt_data(original) {
                StmtData::Declaration(declaration) => declaration.as_variable_declaration(self.ast),
                StmtData::ExportNamedDeclaration(export) => export
                    .declaration(self.ast)
                    .and_then(|declaration| declaration.as_variable_declaration(self.ast)),
                _ => None,
            };
            let statement = original.visit_mut_with(self).unwrap_or(original);
            if statement != original {
                statements.replace_slot(self.ast, id, statement);
            }
            if !self.frame.pending.is_empty() {
                let body = transformed.get_or_insert_with(|| {
                    let mut prefix = statements;
                    let _ = prefix.split_off(index);
                    let mut body = Vec::with_capacity(statements.len() + self.frame.pending.len());
                    body.extend(prefix.iter().map(|id| self.ast.get_node_in_sub_range(id)));
                    body
                });
                body.push(statement);
                body.append(&mut self.frame.pending);
            } else if let Some(body) = &mut transformed {
                body.push(statement);
            }
            self.last_signature = None;
        }
        let frame = std::mem::replace(&mut self.frame, outer_frame);
        self.in_statement_list = nested;
        self.statement_declaration = outer_declaration;
        let declarations = frame.declarations;
        if !initialize_signatures {
            // Case functions share the switch scope and can run from another case.
            // Initialize their signatures before entering the switch, while keeping
            // trailing association calls in the case where the functions are visible.
            self.frame.declarations.extend(declarations);
            return transformed.map(|body| self.ast.add_sub_range(body));
        }
        if !declarations.is_empty() {
            let mut declarators = Vec::with_capacity(declarations.len());
            for (binding, init) in declarations {
                let id = self
                    .ast
                    .binding_pattern_binding_identifier(DUMMY_SP, binding);
                declarators.push(self.ast.variable_declarator(
                    DUMMY_SP,
                    id,
                    None,
                    Some(init),
                    false,
                ));
            }
            let declarators = self.ast.add_sub_range(declarators);
            let declaration =
                self.ast
                    .variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false);
            let declaration = Stmt::Declaration(Decl::VariableDeclaration(declaration));
            let body = transformed.get_or_insert_with(|| {
                statements
                    .iter()
                    .map(|id| self.ast.get_node_in_sub_range(id))
                    .collect()
            });
            let insertion = if !nested {
                self.program_signature_declaration = Some(declaration);
                body.iter()
                    .take_while(|statement| statement.is_import_declaration(self.ast))
                    .count()
            } else {
                0
            };
            body.insert(insertion, declaration);
        }
        transformed.map(|body| self.ast.add_sub_range(body))
    }
}

impl<'ast> VisitMut<'ast> for SignatureTransformer<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_stmts(&mut self, statements: SubRange<Stmt>) -> Option<SubRange<Stmt>> {
        self.transform_statements(statements, true)
    }

    fn visit_mut_if_statement(&mut self, statement: IfStatement) -> Option<IfStatement> {
        if let Some(test) = statement.test(self.ast).visit_mut_with(self) {
            statement.set_test(self.ast, test);
        }
        let consequent = self.transform_branch(statement.consequent(self.ast));
        statement.set_consequent(self.ast, consequent);
        if let Some(alternate) = statement.alternate(self.ast) {
            let alternate = self.transform_branch(alternate);
            statement.set_alternate(self.ast, Some(alternate));
        }
        None
    }

    fn visit_mut_switch_case(&mut self, case: SwitchCase) -> Option<SwitchCase> {
        if let Some(test) = case.test(self.ast)
            && let Some(test) = test.visit_mut_with(self)
        {
            case.set_test(self.ast, Some(test));
        }
        if let Some(consequent) = self.transform_statements(case.consequent(self.ast), false) {
            case.set_consequent(self.ast, consequent);
        }
        None
    }

    fn visit_mut_variable_declaration(
        &mut self,
        declaration: VariableDeclaration,
    ) -> Option<VariableDeclaration> {
        // Only statement declarations can receive a trailing signature call.
        // Loop initializers must attach the signature to the expression itself.
        let direct = self.statement_declaration == Some(declaration);
        for id in declaration.declarators(self.ast).iter() {
            let declarator = self.ast.get_node_in_sub_range(id);
            let initializer = if direct {
                declarator
                    .id(self.ast)
                    .as_binding_identifier(self.ast)
                    .zip(declarator.init(self.ast))
                    .map(|(binding, expression)| {
                        (
                            runtime_expression(expression, self.ast).node_id(),
                            binding.name(self.ast),
                        )
                    })
            } else {
                None
            };
            let outer = std::mem::replace(&mut self.direct_initializer, initializer);
            declarator.visit_mut_children_with(self);
            self.direct_initializer = outer;
        }
        self.last_signature = None;
        None
    }

    fn visit_mut_function(&mut self, function: Function) -> Option<Function> {
        let hooks = if function.function_type(self.ast) == FunctionType::FunctionDeclaration {
            self.collect_hooks(
                ArrowFunctionBody::FunctionBody(function.body(self.ast)),
                function.node_id(),
            )
        } else {
            None
        };
        function.visit_mut_children_with(self);
        if let Some(hooks) = hooks {
            let signature = self.transform_function_body(function.body(self.ast), hooks);
            if let Some(id) = function.id(self.ast) {
                self.queue_signature(&signature, id.name(self.ast));
            }
        }
        None
    }

    fn visit_mut_object_property(&mut self, property: ObjectProperty) -> Option<ObjectProperty> {
        if !property.method(self.ast) && property.kind(self.ast) == PropertyKind::Init {
            property.visit_mut_children_with(self);
            return None;
        }
        if let Some(key) = property.key(self.ast).visit_mut_with(self) {
            property.set_key(self.ast, key);
        }
        // Visit the method function directly so signature calls cannot replace it.
        property.value(self.ast).visit_mut_children_with(self);
        self.last_signature = None;
        None
    }

    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        self.last_signature = None;
        match self.ast.expr_data(expression) {
            ExprData::Function(function)
                if function.function_type(self.ast) == FunctionType::FunctionExpression =>
            {
                let hooks = self.collect_hooks(
                    ArrowFunctionBody::FunctionBody(function.body(self.ast)),
                    function.node_id(),
                );
                expression.visit_mut_children_with(self);
                self.last_signature = None;
                let signature = self.transform_function_body(function.body(self.ast), hooks?);
                if let Some(name) = self.direct_initializer(expression) {
                    self.queue_signature(&signature, name);
                    None
                } else {
                    Some(self.wrap_signature(signature, expression))
                }
            }
            ExprData::ArrowFunctionExpression(arrow) => {
                let hooks = self.collect_hooks(arrow.body(self.ast), arrow.node_id());
                expression.visit_mut_children_with(self);
                self.last_signature = None;
                let signature = self.transform_arrow(arrow, hooks?)?;
                if let Some(name) = self.direct_initializer(expression) {
                    self.queue_signature(&signature, name);
                    None
                } else {
                    Some(self.wrap_signature(signature, expression))
                }
            }
            ExprData::CallExpression(call) => {
                let callee = call.callee(self.ast);
                if let Some(callee) = callee.visit_mut_with(self) {
                    call.set_callee(self.ast, callee);
                }
                self.last_signature = None;
                if let Some(type_arguments) = call.type_arguments(self.ast)
                    && let Some(type_arguments) = type_arguments.visit_mut_with(self)
                {
                    call.set_type_arguments(self.ast, Some(type_arguments));
                }
                let arguments = call.arguments(self.ast);
                let mut signature = None;
                for (index, id) in arguments.iter().enumerate() {
                    let argument = self.ast.get_node_in_sub_range(id);
                    let current = argument.visit_mut_with(self).unwrap_or(argument);
                    if current != argument {
                        arguments.replace_slot(self.ast, id, current);
                    }
                    if index == 0 {
                        signature = self.last_signature.take().filter(|signature| {
                            current.as_expr(self.ast) == Some(signature.expression)
                        });
                    } else {
                        self.last_signature = None;
                    }
                }
                let signature = signature?;
                Some(self.wrap_signature(signature.signature, expression))
            }
            _ => {
                expression.visit_mut_children_with(self);
                self.forward_signature(expression);
                None
            }
        }
    }
}
