use swc_next_ecma_ast::{
    Argument, AssignmentOperator, AssignmentTarget, AstUtf8, DUMMY_SP, Directive, Expr,
    FormalParameterItem, FormalParameterKind, FormalParameterPattern, FunctionType,
    LogicalOperator, SimpleAssignmentTarget, Span, Stmt, SubRange, VisitMut,
};

use super::TypeScriptLowerer;

#[derive(Clone, Copy)]
pub(super) enum RuntimeTarget {
    Identifier(AstUtf8),
}

impl TypeScriptLowerer<'_, '_> {
    pub(super) fn object_iife_statement(
        &mut self,
        parameter: AstUtf8,
        directives: SubRange<Directive>,
        statements: SubRange<Stmt>,
        target: RuntimeTarget,
        span: Span,
    ) -> Stmt {
        let body = self.ast().function_body(DUMMY_SP, directives, statements);
        let parameter_binding = self.ast().binding_identifier(DUMMY_SP, parameter);
        let parameter = self.ast().formal_parameter(
            DUMMY_SP,
            SubRange::empty(),
            FormalParameterPattern::BindingIdentifier(parameter_binding),
            None,
            None,
            false,
        );
        let parameters = self
            .ast()
            .add_sub_range([FormalParameterItem::FormalParameter(parameter)]);
        let parameters = self.ast().formal_parameters(
            DUMMY_SP,
            parameters,
            None,
            FormalParameterKind::FormalParameters,
        );
        let function = self.ast().function(
            DUMMY_SP,
            FunctionType::FunctionExpression,
            None,
            None,
            None,
            parameters,
            None,
            body,
            false,
            false,
        );
        let argument = self.runtime_target_argument(target);
        let arguments = self.ast().add_sub_range([Argument::Expr(argument)]);
        let call =
            self.ast()
                .expr_call_expression(span, Expr::Function(function), None, arguments, false);
        self.ast().stmt_expression_statement(span, call)
    }

    fn runtime_target_argument(&mut self, target: RuntimeTarget) -> Expr {
        let RuntimeTarget::Identifier(name) = target;
        let existing = self.ast().expr_identifier_reference(DUMMY_SP, name);
        let object = self
            .ast()
            .expr_object_expression(DUMMY_SP, SubRange::empty());
        let identifier = self.ast().identifier_reference(DUMMY_SP, name);
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(identifier),
        );
        let assignment = self.ast().expr_assignment_expression(
            DUMMY_SP,
            target,
            object,
            AssignmentOperator::Assign,
        );
        self.ast()
            .expr_logical_expression(DUMMY_SP, existing, assignment, LogicalOperator::Or)
    }
}
