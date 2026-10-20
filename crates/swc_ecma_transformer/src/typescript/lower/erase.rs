use super::TypeScriptLowerer;
use swc_next_ecma_ast::{
    Ast, BindingPatternData, BindingProperty, ClassElement, Expr, ExprData, GetSpan,
    SimpleAssignmentTarget, SubRange, VisitMut, VisitMutWith,
};

impl TypeScriptLowerer<'_, '_> {
    pub(super) fn lower_typescript_expression(&mut self, expression: Expr) -> Option<Expr> {
        let inner = match self.ast_ref().expr_data(expression) {
            ExprData::MemberExpression(_) => {
                if let Some(value) = self.analysis.inline_const_enum_use(expression.node_id()) {
                    return Some(self.inline_enum_constant(expression, value));
                }
                None
            }
            ExprData::IdentifierReference(identifier) => {
                return self.namespace_identifier_reference(identifier);
            }
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

    pub(super) fn lower_class_elements(
        &mut self,
        elements: SubRange<ClassElement>,
    ) -> Option<SubRange<ClassElement>> {
        let mut lowered: Option<Vec<ClassElement>> = None;
        for (index, element_id) in elements.iter().enumerate() {
            let element = self.ast().get_node_in_sub_range(element_id);
            let drop = element.is_ts_method_definition(self.ast())
                || element.is_ts_index_signature(self.ast())
                || element
                    .as_method_definition(self.ast())
                    .is_some_and(|method| method.r#abstract(self.ast()))
                || element
                    .as_property_definition(self.ast())
                    .is_some_and(|property| {
                        property.declare(self.ast()) || property.r#abstract(self.ast())
                    });
            if drop {
                lowered.get_or_insert_with(|| {
                    let mut prefix = Vec::with_capacity(elements.len() - 1);
                    prefix.extend(
                        elements
                            .iter()
                            .take(index)
                            .map(|id| self.ast_ref().get_node_in_sub_range(id)),
                    );
                    prefix
                });
                continue;
            }
            element.visit_mut_children_with(self);
            if let Some(output) = &mut lowered {
                output.push(element);
            }
        }

        lowered.map(|elements| self.ast().add_sub_range(elements))
    }

    pub(super) fn unwrap_expression(
        &mut self,
        expression: Expr,
        removed_span: swc_next_ecma_ast::Span,
    ) -> Expr {
        let replacement = expression.visit_mut_with(self).unwrap_or(expression);
        let replacement_span = replacement.span(self.ast_ref());
        if !self.ast_ref().comments().is_empty() {
            self.comments.inherit(replacement_span, removed_span);
        }
        replacement
    }

    pub(super) fn expression_to_assignment_target(
        &self,
        expression: Expr,
    ) -> SimpleAssignmentTarget {
        match self.ast_ref().expr_data(expression) {
            ExprData::IdentifierReference(identifier) => {
                SimpleAssignmentTarget::IdentifierReference(identifier)
            }
            ExprData::MemberExpression(member) => SimpleAssignmentTarget::MemberExpression(member),
            _ => {
                unreachable!("a TypeScript assignment-target wrapper contains an assignment target")
            }
        }
    }
}

pub(super) fn update_binding_property_shorthand(ast: &mut Ast<'_>, property: BindingProperty) {
    if !property.shorthand(ast) {
        return;
    }
    let mut value = property.value(ast);
    if let BindingPatternData::AssignmentPattern(assignment) = ast.binding_pattern_data(value) {
        value = assignment.left(ast);
    }
    let is_identifier = match ast.binding_pattern_data(value) {
        BindingPatternData::BindingIdentifier(_) => true,
        BindingPatternData::SimpleAssignmentTarget(target) => target.is_identifier_reference(ast),
        _ => false,
    };
    if !is_identifier {
        property.set_shorthand(ast, false);
    }
}
