use super::{StatementExpansion, TypeScriptLowerer, erase::update_binding_property_shorthand};
use swc_next_ecma_ast::{
    Accessibility, ArrowFunctionExpression, Ast, BindingProperty, CallExpression, CatchClause,
    Class, ClassElement, DUMMY_SP, Expr, ExprData, FormalParameter, FormalParameterRest, GetSpan,
    JsxOpeningElement, MethodDefinition, NewExpression, ObjectProperty, PropertyDefinition,
    SimpleAssignmentTarget, SimpleAssignmentTargetData, Stmt, SubRange, TaggedTemplateExpression,
    VariableDeclaration, VariableDeclarator, VisitMut, VisitMutWith,
};

impl<'ast> VisitMut<'ast> for TypeScriptLowerer<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_stmt(&mut self, statement: Stmt) -> Option<Stmt> {
        match self.lower_statement(statement)? {
            StatementExpansion::Remove => {
                Some(Stmt::EmptyStatement(self.ast().empty_statement(DUMMY_SP)))
            }
            StatementExpansion::Single(statement) => Some(statement),
        }
    }

    fn visit_mut_stmts(&mut self, statements: SubRange<Stmt>) -> Option<SubRange<Stmt>> {
        self.lower_statement_list(statements)
    }

    fn visit_mut_class_elements(
        &mut self,
        elements: SubRange<ClassElement>,
    ) -> Option<SubRange<ClassElement>> {
        self.lower_class_elements(elements)
    }

    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        let inner = match self.ast().expr_data(expression) {
            ExprData::TsAsExpression(wrapper) => Some(wrapper.expression(self.ast())),
            ExprData::TsSatisfiesExpression(wrapper) => Some(wrapper.expression(self.ast())),
            ExprData::TsTypeAssertion(wrapper) => Some(wrapper.expression(self.ast())),
            ExprData::TsNonNullExpression(wrapper) => Some(wrapper.expression(self.ast())),
            ExprData::TsInstantiationExpression(wrapper) => Some(wrapper.expression(self.ast())),
            _ => None,
        };
        if let Some(inner) = inner {
            let removed_span = expression.span(self.ast_ref());
            Some(self.unwrap_expression(inner, removed_span))
        } else {
            expression.visit_mut_children_with(self)
        }
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
        self.lower_parameter_properties(class);
        class.set_type_parameters(self.ast(), None);
        class.set_super_type_arguments(self.ast(), None);
        class.set_implements(self.ast(), SubRange::empty());
        class.set_abstract(self.ast(), false);
        class.set_declare(self.ast(), false);
        class.visit_mut_children_with(self);
        None
    }

    fn visit_mut_method_definition(
        &mut self,
        method: MethodDefinition,
    ) -> Option<MethodDefinition> {
        method.set_override(self.ast(), false);
        method.set_optional(self.ast(), false);
        method.set_abstract(self.ast(), false);
        method.set_accessibility(self.ast(), Accessibility::None);
        method.visit_mut_children_with(self)
    }

    fn visit_mut_property_definition(
        &mut self,
        property: PropertyDefinition,
    ) -> Option<PropertyDefinition> {
        property.set_type_annotation(self.ast(), None);
        property.set_declare(self.ast(), false);
        property.set_override(self.ast(), false);
        property.set_optional(self.ast(), false);
        property.set_definite(self.ast(), false);
        property.set_readonly(self.ast(), false);
        property.set_abstract(self.ast(), false);
        property.set_accessibility(self.ast(), Accessibility::None);
        property.visit_mut_children_with(self)
    }

    fn visit_mut_function(
        &mut self,
        function: swc_next_ecma_ast::Function,
    ) -> Option<swc_next_ecma_ast::Function> {
        function.set_type_parameters(self.ast(), None);
        function.set_this_param(self.ast(), None);
        function.set_return_type(self.ast(), None);
        function.visit_mut_children_with(self)
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
        element.visit_mut_children_with(self)
    }
}
