use super::{StatementExpansion, TypeScriptLowerer, erase::update_binding_property_shorthand};
use crate::syntax::FollowingTransform;
use swc_next_ecma_ast::{
    Accessibility, ArrowFunctionExpression, Ast, BindingProperty, CallExpression, CatchClause,
    Class, ClassElement, DUMMY_SP, Expr, ExprData, FormalParameter, FormalParameterRest, GetSpan,
    JsxClosingElement, JsxOpeningElement, MethodDefinition, NewExpression, NodeKindData,
    ObjectProperty, PropertyDefinition, SimpleAssignmentTarget, SimpleAssignmentTargetData,
    StaticBlock, Stmt, SubRange, SwitchCase, SwitchStatement, TaggedTemplateExpression,
    VariableDeclaration, VariableDeclarator, VisitMut, VisitMutWith,
};

impl<'ast, F: FollowingTransform<'ast>> VisitMut<'ast> for TypeScriptLowerer<'_, 'ast, F> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_stmt(&mut self, statement: Stmt) -> Option<Stmt> {
        match self.lower_statement(statement)? {
            StatementExpansion::Remove => {
                Some(Stmt::EmptyStatement(self.ast().empty_statement(DUMMY_SP)))
            }
            StatementExpansion::Single(statement) => Some(statement),
            StatementExpansion::Pair(first, second) => {
                let body = self.ast().add_sub_range([first, second]);
                Some(Stmt::BlockStatement(
                    self.ast().block_statement(DUMMY_SP, body),
                ))
            }
        }
    }

    fn visit_mut_stmts(&mut self, statements: SubRange<Stmt>) -> Option<SubRange<Stmt>> {
        self.lower_statement_list(statements)
    }

    fn visit_mut_switch_statement(
        &mut self,
        statement: SwitchStatement,
    ) -> Option<SwitchStatement> {
        let discriminant = statement.discriminant(self.ast_ref());
        if let Some(discriminant) = discriminant.visit_mut_with(self) {
            statement.set_discriminant(self.ast(), discriminant);
        }

        self.enter_binding_scope();
        let cases = statement.cases(self.ast_ref());
        if self.analysis.has_merged_runtime_bindings() {
            for case_id in cases.iter() {
                let case = self.ast_ref().get_node_in_sub_range(case_id);
                self.register_scope_runtime_bindings(case.consequent(self.ast_ref()));
            }
        }
        if let Some(cases) = cases.visit_mut_with(self) {
            statement.set_cases(self.ast(), cases);
        }
        self.exit_binding_scope();
        None
    }

    fn visit_mut_switch_case(&mut self, case: SwitchCase) -> Option<SwitchCase> {
        let test = case.test(self.ast_ref());
        if let Some(test) = test.visit_mut_with(self) {
            case.set_test(self.ast(), test);
        }

        let consequent = case.consequent(self.ast_ref());
        if let Some(consequent) = self.lower_statement_list_in_current_scope(consequent) {
            case.set_consequent(self.ast(), consequent);
        }
        None
    }

    fn visit_mut_class_elements(
        &mut self,
        elements: SubRange<ClassElement>,
    ) -> Option<SubRange<ClassElement>> {
        self.lower_class_elements(elements)
    }

    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        self.following.enter_expression(expression);
        let replacement = self.lower_typescript_expression(expression);
        let current = replacement.unwrap_or(expression);
        let namespaces = &self.namespaces;
        let semantic = self.semantic;
        let following_replacement =
            self.following
                .exit_expression(current, self.ast, &|ast, name, symbol| {
                    namespaces.parameter_for_symbol(ast, semantic, name, symbol)
                });
        self.following.leave_node(expression.node_id());
        following_replacement.or(replacement)
    }

    fn visit_mut_object_property(&mut self, property: ObjectProperty) -> Option<ObjectProperty> {
        property.visit_mut_children_with(self);
        let shorthand_value_is_identifier = matches!(
            self.ast_ref().expr_data(property.value(self.ast_ref())),
            ExprData::IdentifierReference(_)
        );
        if property.shorthand(self.ast_ref()) && !shorthand_value_is_identifier {
            property.set_shorthand(self.ast(), false);
        }
        None
    }

    fn visit_mut_binding_property(&mut self, property: BindingProperty) -> Option<BindingProperty> {
        property.visit_mut_children_with(self);
        update_binding_property_shorthand(self.ast(), property);
        None
    }

    fn visit_mut_simple_assignment_target(
        &mut self,
        target: SimpleAssignmentTarget,
    ) -> Option<SimpleAssignmentTarget> {
        if let SimpleAssignmentTargetData::IdentifierReference(identifier) =
            self.ast().simple_assignment_target_data(target)
            && let Some(replacement) = self.namespace_identifier_target(identifier)
        {
            return Some(replacement);
        }
        let inner = match self.ast().simple_assignment_target_data(target) {
            SimpleAssignmentTargetData::TsAsExpression(wrapper) => {
                Some(wrapper.expression(self.ast()))
            }
            SimpleAssignmentTargetData::TsSatisfiesExpression(wrapper) => {
                Some(wrapper.expression(self.ast()))
            }
            SimpleAssignmentTargetData::TsTypeAssertion(wrapper) => {
                Some(wrapper.expression(self.ast()))
            }
            SimpleAssignmentTargetData::TsNonNullExpression(wrapper) => {
                Some(wrapper.expression(self.ast()))
            }
            _ => None,
        };
        if let Some(inner) = inner {
            let removed_span = target.span(self.ast_ref());
            let replacement = self.unwrap_expression(inner, removed_span);
            Some(self.expression_to_assignment_target(replacement))
        } else {
            target.visit_mut_children_with(self)
        }
    }

    fn visit_mut_formal_parameter(
        &mut self,
        parameter: FormalParameter,
    ) -> Option<FormalParameter> {
        parameter.set_type_annotation(self.ast(), None);
        parameter.set_optional(self.ast(), false);
        parameter.visit_mut_children_with(self)
    }

    fn visit_mut_formal_parameter_rest(
        &mut self,
        rest: FormalParameterRest,
    ) -> Option<FormalParameterRest> {
        rest.set_type_annotation(self.ast(), None);
        rest.visit_mut_children_with(self)
    }

    fn visit_mut_catch_clause(&mut self, clause: CatchClause) -> Option<CatchClause> {
        clause.set_type_annotation(self.ast(), None);
        clause.visit_mut_children_with(self)
    }

    fn visit_mut_variable_declaration(
        &mut self,
        declaration: VariableDeclaration,
    ) -> Option<VariableDeclaration> {
        declaration.set_declare(self.ast(), false);
        declaration.visit_mut_children_with(self)
    }

    fn visit_mut_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
    ) -> Option<VariableDeclarator> {
        declarator.set_definite(self.ast(), false);
        declarator.set_type_annotation(self.ast(), None);
        declarator.visit_mut_children_with(self)
    }

    fn visit_mut_class(&mut self, class: Class) -> Option<Class> {
        self.following
            .enter_node(NodeKindData::Class(class), self.ast);
        class.set_type_parameters(self.ast(), None);
        class.set_super_type_arguments(self.ast(), None);
        class.set_implements(self.ast(), SubRange::empty());
        class.set_abstract(self.ast(), false);
        class.set_declare(self.ast(), false);
        class.visit_mut_children_with(self);
        self.lower_parameter_properties(class);
        self.following.leave_node(class.node_id());
        None
    }

    fn visit_mut_method_definition(
        &mut self,
        method: MethodDefinition,
    ) -> Option<MethodDefinition> {
        self.following
            .enter_node(NodeKindData::MethodDefinition(method), self.ast);
        method.set_override(self.ast(), false);
        method.set_optional(self.ast(), false);
        method.set_abstract(self.ast(), false);
        method.set_accessibility(self.ast(), Accessibility::None);
        let replacement = method.visit_mut_children_with(self);
        self.following.leave_node(method.node_id());
        replacement
    }

    fn visit_mut_property_definition(
        &mut self,
        property: PropertyDefinition,
    ) -> Option<PropertyDefinition> {
        self.following
            .enter_node(NodeKindData::PropertyDefinition(property), self.ast);
        property.set_type_annotation(self.ast(), None);
        property.set_declare(self.ast(), false);
        property.set_override(self.ast(), false);
        property.set_optional(self.ast(), false);
        property.set_definite(self.ast(), false);
        property.set_readonly(self.ast(), false);
        property.set_abstract(self.ast(), false);
        property.set_accessibility(self.ast(), Accessibility::None);
        let replacement = property.visit_mut_children_with(self);
        self.following.leave_node(property.node_id());
        replacement
    }

    fn visit_mut_function(
        &mut self,
        function: swc_next_ecma_ast::Function,
    ) -> Option<swc_next_ecma_ast::Function> {
        self.following
            .enter_node(NodeKindData::Function(function), self.ast);
        function.set_type_parameters(self.ast(), None);
        function.set_this_param(self.ast(), None);
        function.set_return_type(self.ast(), None);
        let replacement = function.visit_mut_children_with(self);
        self.following.leave_node(function.node_id());
        replacement
    }

    fn visit_mut_static_block(&mut self, block: StaticBlock) -> Option<StaticBlock> {
        self.following
            .enter_node(NodeKindData::StaticBlock(block), self.ast);
        let replacement = block.visit_mut_children_with(self);
        self.following.leave_node(block.node_id());
        replacement
    }

    fn visit_mut_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
    ) -> Option<ArrowFunctionExpression> {
        arrow.set_type_parameters(self.ast(), None);
        arrow.set_return_type(self.ast(), None);
        arrow.visit_mut_children_with(self)
    }

    fn visit_mut_call_expression(&mut self, call: CallExpression) -> Option<CallExpression> {
        call.set_type_arguments(self.ast(), None);
        call.visit_mut_children_with(self)
    }

    fn visit_mut_new_expression(&mut self, expression: NewExpression) -> Option<NewExpression> {
        expression.set_type_arguments(self.ast(), None);
        expression.visit_mut_children_with(self)
    }

    fn visit_mut_tagged_template_expression(
        &mut self,
        template: TaggedTemplateExpression,
    ) -> Option<TaggedTemplateExpression> {
        template.set_type_arguments(self.ast(), None);
        template.visit_mut_children_with(self)
    }

    fn visit_mut_jsx_opening_element(
        &mut self,
        element: JsxOpeningElement,
    ) -> Option<JsxOpeningElement> {
        element.set_type_arguments(self.ast(), None);
        if let Some(name) = self.namespace_jsx_name(element.name(self.ast_ref())) {
            element.set_name(self.ast(), name);
        }
        element.visit_mut_children_with(self)
    }

    fn visit_mut_jsx_closing_element(
        &mut self,
        element: JsxClosingElement,
    ) -> Option<JsxClosingElement> {
        if let Some(name) = self.namespace_jsx_name(element.name(self.ast_ref())) {
            element.set_name(self.ast(), name);
        }
        element.visit_mut_children_with(self)
    }
}
