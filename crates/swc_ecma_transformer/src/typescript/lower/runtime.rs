use swc_next_ecma_ast::{
    Argument, AssignmentOperator, AssignmentTarget, AstUtf8, DUMMY_SP, Directive, Expr,
    FormalParameterItem, FormalParameterKind, FormalParameterPattern, FunctionType,
    LogicalOperator, PropertyKey, SimpleAssignmentTarget, Span, Stmt, SubRange, VisitMut,
};

use super::TypeScriptLowerer;

#[derive(Clone, Copy)]
pub(super) enum RuntimeTarget {
    Identifier(AstUtf8),
    IdentifierAndProperty {
        identifier: AstUtf8,
        object: AstUtf8,
        property: AstUtf8,
    },
}

/// A single location which supports both reads and assignments.
#[derive(Clone, Copy)]
enum RuntimeReference {
    Identifier(AstUtf8),
    Property { object: AstUtf8, property: AstUtf8 },
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
        let call = self.ast().expr_call_expression(
            span,
            Expr::Function(function),
            None,
            arguments,
            false,
            false,
        );
        self.ast().stmt_expression_statement(span, call)
    }

    fn runtime_target_argument(&mut self, target: RuntimeTarget) -> Expr {
        let (local, reference) = match target {
            RuntimeTarget::Identifier(name) => (None, RuntimeReference::Identifier(name)),
            RuntimeTarget::IdentifierAndProperty {
                identifier,
                object,
                property,
            } => (
                Some(identifier),
                RuntimeReference::Property { object, property },
            ),
        };
        let existing = self.runtime_target_read(reference);
        let assignment = self.runtime_target_initializer(reference);
        let value =
            self.ast()
                .expr_logical_expression(DUMMY_SP, existing, assignment, LogicalOperator::Or);
        let Some(local) = local else {
            return value;
        };
        let identifier = self.ast().identifier_reference(DUMMY_SP, local);
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(identifier),
        );
        self.ast()
            .expr_assignment_expression(DUMMY_SP, target, value, AssignmentOperator::Assign)
    }

    fn runtime_target_initializer(&mut self, target: RuntimeReference) -> Expr {
        let assignment_target = self.runtime_target_assignment(target);
        let object = self
            .ast()
            .expr_object_expression(DUMMY_SP, SubRange::empty());
        self.ast().expr_assignment_expression(
            DUMMY_SP,
            assignment_target,
            object,
            AssignmentOperator::Assign,
        )
    }

    fn runtime_target_read(&mut self, target: RuntimeReference) -> Expr {
        match target {
            RuntimeReference::Identifier(name) => {
                self.ast().expr_identifier_reference(DUMMY_SP, name)
            }
            RuntimeReference::Property { object, property } => {
                let member = self.runtime_target_member(object, property);
                Expr::MemberExpression(member)
            }
        }
    }

    fn runtime_target_assignment(&mut self, target: RuntimeReference) -> AssignmentTarget {
        let target = match target {
            RuntimeReference::Identifier(name) => SimpleAssignmentTarget::IdentifierReference(
                self.ast().identifier_reference(DUMMY_SP, name),
            ),
            RuntimeReference::Property { object, property } => {
                let member = self.runtime_target_member(object, property);
                SimpleAssignmentTarget::MemberExpression(member)
            }
        };
        AssignmentTarget::SimpleAssignmentTarget(target)
    }

    fn runtime_target_member(
        &mut self,
        object: AstUtf8,
        property: AstUtf8,
    ) -> swc_next_ecma_ast::MemberExpression {
        let object = self.ast().expr_identifier_reference(DUMMY_SP, object);
        let property = self.ast().identifier_name(DUMMY_SP, property);
        self.ast().member_expression(
            DUMMY_SP,
            object,
            PropertyKey::IdentifierName(property),
            false,
            false,
        )
    }
}
