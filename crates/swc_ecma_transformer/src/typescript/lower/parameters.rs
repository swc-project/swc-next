use swc_next_ecma_ast::{
    Accessibility, ArrowFunctionBody, AssignmentOperator, AssignmentTarget, Ast, AstUtf8,
    BindingPattern, Class, DUMMY_SP, Expr, FormalParameterItem, FormalParameterKind, Function,
    FunctionBody, GetSpan, MethodDefinitionKind, PropertyKey, SimpleAssignmentTarget, Stmt,
    StmtData, SubRange, VariableKind, VisitMut, VisitMutWith,
};

use super::TypeScriptLowerer;

#[derive(Clone, Copy)]
struct ParameterProperty {
    name: AstUtf8,
    span: swc_next_ecma_ast::Span,
}

impl TypeScriptLowerer<'_, '_> {
    pub(super) fn lower_parameter_properties(&mut self, class: Class) {
        let body = class.body(self.ast_ref());
        let constructor = body.body(self.ast_ref()).iter().find_map(|element_id| {
            let element = self.ast_ref().get_node_in_sub_range(element_id);
            let method = element.as_method_definition(self.ast_ref())?;
            (method.kind(self.ast_ref()) == MethodDefinitionKind::Constructor).then_some(method)
        });
        let Some(constructor) = constructor else {
            return;
        };

        let function = constructor.value(self.ast_ref());
        let properties = self.convert_parameter_properties(function);
        if properties.is_empty() {
            return;
        }

        self.prepend_parameter_property_fields(body, &properties);
        let function_body = function.body(self.ast_ref());
        if class.super_class(self.ast_ref()).is_some() {
            SuperInjector::new(self, &properties).inject(function_body);
        } else {
            self.prepend_parameter_property_assignments(function_body, &properties);
        }
    }

    fn convert_parameter_properties(&mut self, function: Function) -> Vec<ParameterProperty> {
        let parameters = function.params(self.ast_ref());
        let items = parameters.items(self.ast_ref());
        let mut properties = Vec::new();

        for item_id in items.iter() {
            let item = self.ast_ref().get_node_in_sub_range(item_id);
            let Some(parameter_property) = item.as_ts_parameter_property(self.ast_ref()) else {
                continue;
            };

            let parameter = parameter_property.parameter(self.ast_ref());
            let decorators = parameter_property.decorators(self.ast_ref());
            if !decorators.is_empty() {
                let parameter_decorators = parameter.decorators(self.ast_ref());
                let decorators = if parameter_decorators.is_empty() {
                    decorators
                } else {
                    let decorators: Vec<_> = decorators
                        .iter()
                        .chain(parameter_decorators.iter())
                        .map(|id| self.ast_ref().get_node_in_sub_range(id))
                        .collect();
                    self.ast().add_sub_range(decorators)
                };
                parameter.set_decorators(self.ast(), decorators);
            }
            if let Some(binding) = parameter
                .pattern(self.ast_ref())
                .as_binding_identifier(self.ast_ref())
            {
                properties.push(ParameterProperty {
                    name: binding.name(self.ast_ref()),
                    span: binding.span(self.ast_ref()),
                });
            }

            items.replace_slot(
                self.ast(),
                item_id,
                FormalParameterItem::FormalParameter(parameter),
            );
        }

        properties
    }

    fn prepend_parameter_property_fields(
        &mut self,
        body: swc_next_ecma_ast::ClassBody,
        properties: &[ParameterProperty],
    ) {
        let elements = body.body(self.ast_ref());
        let mut lowered = Vec::with_capacity(properties.len() + elements.len());
        for property in properties {
            let field = parameter_property_field(self.ast(), *property);
            lowered.push(field);
        }
        for element_id in elements.iter() {
            lowered.push(self.ast_ref().get_node_in_sub_range(element_id));
        }
        let elements = self.ast().add_sub_range(lowered);
        body.set_body(self.ast(), elements);
    }

    fn prepend_parameter_property_assignments(
        &mut self,
        body: FunctionBody,
        properties: &[ParameterProperty],
    ) {
        let statements = body.body(self.ast_ref());
        let mut lowered = Vec::with_capacity(properties.len() + statements.len());
        for property in properties {
            let assignment = make_assignment(self.ast(), *property);
            lowered.push(self.ast().stmt_expression_statement(DUMMY_SP, assignment));
        }
        for statement_id in statements.iter() {
            lowered.push(self.ast_ref().get_node_in_sub_range(statement_id));
        }
        let statements = self.ast().add_sub_range(lowered);
        body.set_body(self.ast(), statements);
    }
}

/// Reserve the public instance field before other class fields initialize.
fn parameter_property_field(
    ast: &mut Ast<'_>,
    property: ParameterProperty,
) -> swc_next_ecma_ast::ClassElement {
    let key = ast.identifier_name(property.span, property.name);
    ast.class_element_property_definition(
        DUMMY_SP,
        SubRange::empty(),
        PropertyKey::IdentifierName(key),
        None,
        None,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        Accessibility::None,
    )
}

fn make_assignment(ast: &mut Ast<'_>, property: ParameterProperty) -> Expr {
    let object = ast.expr_this_expression(DUMMY_SP);
    let key = ast.identifier_name(property.span, property.name);
    let member = ast.member_expression(
        DUMMY_SP,
        object,
        PropertyKey::IdentifierName(key),
        false,
        false,
    );
    let left =
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::MemberExpression(member));
    let right = ast.expr_identifier_reference(property.span, property.name);
    ast.expr_assignment_expression(DUMMY_SP, left, right, AssignmentOperator::Assign)
}

/// Keep parameter reads in the constructor's lexical environment. Direct
/// super statements can inline them; other sites call a lazily emitted arrow
/// defined in that environment, so nested bindings cannot capture the reads.
struct SuperInjector<'lower, 'visit, 'ast> {
    lowerer: &'lower mut TypeScriptLowerer<'visit, 'ast>,
    properties: &'lower [ParameterProperty],
    initializer: Option<AstUtf8>,
}

impl<'lower, 'visit, 'ast> SuperInjector<'lower, 'visit, 'ast> {
    fn new(
        lowerer: &'lower mut TypeScriptLowerer<'visit, 'ast>,
        properties: &'lower [ParameterProperty],
    ) -> Self {
        Self {
            lowerer,
            properties,
            initializer: None,
        }
    }

    fn inject(mut self, body: FunctionBody) {
        let statements = body.body(self.ast());
        for id in statements.iter() {
            let statement = self.ast().get_node_in_sub_range(id);
            if let StmtData::ExpressionStatement(statement) = self.ast().stmt_data(statement) {
                let expression = statement.expression(self.ast());
                if self.is_super_call(expression) {
                    // Arguments can themselves contain arrows with super calls.
                    expression.visit_mut_children_with(&mut self);
                    let mut expressions = Vec::with_capacity(self.properties.len() + 1);
                    expressions.push(expression);
                    for &property in self.properties {
                        expressions.push(make_assignment(self.lowerer.ast(), property));
                    }
                    // This top-level expression statement discards its value.
                    let expressions = self.ast().add_sub_range(expressions);
                    let expression = self.ast().expr_sequence_expression(DUMMY_SP, expressions);
                    statement.set_expression(self.ast(), expression);
                    continue;
                }
            }
            if let Some(replacement) = statement.visit_mut_with(&mut self) {
                statements.replace_slot(self.ast(), id, replacement);
            }
        }
        if let Some(name) = self.initializer {
            self.prepend_initializer(body, name);
        }
    }

    fn is_super_call(&mut self, expression: Expr) -> bool {
        expression
            .as_call_expression(self.ast())
            .is_some_and(|call| call.callee(self.ast()).is_super(self.ast()))
    }

    fn finish_super_sequence(&mut self, mut expressions: Vec<Expr>) -> Expr {
        // SuperCall binds and returns the same object, including a replacement
        // object returned by the base constructor.
        expressions.push(self.ast().expr_this_expression(DUMMY_SP));
        let expressions = self.ast().add_sub_range(expressions);
        self.ast().expr_sequence_expression(DUMMY_SP, expressions)
    }

    fn inject_after_super(&mut self, expression: Expr) -> Expr {
        let name = match self.initializer {
            Some(name) => name,
            None => {
                let name = self.lowerer.fresh_uid("_initProperties");
                self.initializer = Some(name);
                name
            }
        };
        let callee = self.ast().expr_identifier_reference(DUMMY_SP, name);
        let call = self.ast().expr_call_expression(
            DUMMY_SP,
            callee,
            None,
            SubRange::empty(),
            false,
            false,
        );
        self.finish_super_sequence(vec![expression, call])
    }

    fn prepend_initializer(&mut self, body: FunctionBody, name: AstUtf8) {
        let mut assignments = Vec::with_capacity(self.properties.len());
        for &property in self.properties {
            assignments.push(make_assignment(self.lowerer.ast(), property));
        }
        let assignments = self.ast().add_sub_range(assignments);
        let initializer_body = self.ast().expr_sequence_expression(DUMMY_SP, assignments);
        let parameters = self.ast().formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            None,
            FormalParameterKind::ArrowFormalParameters,
        );
        let initializer = self.ast().expr_arrow_function_expression(
            DUMMY_SP,
            true,
            false,
            None,
            parameters,
            None,
            ArrowFunctionBody::Expr(initializer_body),
        );
        let binding = self.ast().binding_identifier(DUMMY_SP, name);
        let declarator = self.ast().variable_declarator(
            DUMMY_SP,
            BindingPattern::BindingIdentifier(binding),
            None,
            Some(initializer),
            false,
        );
        let declarations = self.ast().add_sub_range([declarator]);
        let declaration =
            self.ast()
                .variable_declaration(DUMMY_SP, VariableKind::Const, declarations, false);
        let statements = body.body(self.ast());
        let mut lowered = Vec::with_capacity(statements.len() + 1);
        lowered.push(Stmt::Declaration(
            swc_next_ecma_ast::Decl::VariableDeclaration(declaration),
        ));
        lowered.extend(
            statements
                .iter()
                .map(|id| self.lowerer.ast_ref().get_node_in_sub_range(id)),
        );
        let statements = self.ast().add_sub_range(lowered);
        body.set_body(self.ast(), statements);
    }
}

impl<'ast> VisitMut<'ast> for SuperInjector<'_, '_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.lowerer.ast()
    }

    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        expression.visit_mut_children_with(self);
        self.is_super_call(expression)
            .then(|| self.inject_after_super(expression))
    }

    fn visit_mut_function(&mut self, _function: Function) -> Option<Function> {
        None
    }

    fn visit_mut_class(&mut self, class: Class) -> Option<Class> {
        super::lexical_env::visit_class_outer_expressions(class, self);
        None
    }
}
