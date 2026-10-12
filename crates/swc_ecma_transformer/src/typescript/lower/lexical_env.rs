use swc_next_ecma_ast::{
    Class, ClassElementData, FormalParameterItemData, FormalParameters, VisitMut, VisitMutWith,
};

/// Visit expressions evaluated in the enclosing lexical function environment.
/// Includes decorators, including parameter decorators, but excludes parameter
/// defaults, method bodies, field initializers and static blocks.
pub(super) fn visit_class_outer_expressions<'ast>(class: Class, visitor: &mut impl VisitMut<'ast>) {
    let decorators = class.decorators(visitor.ast());
    if let Some(decorators) = decorators.visit_mut_with(visitor) {
        class.set_decorators(visitor.ast(), decorators);
    }
    if let Some(super_class) = class.super_class(visitor.ast())
        && let Some(replacement) = super_class.visit_mut_with(visitor)
    {
        class.set_super_class(visitor.ast(), Some(replacement));
    }
    let elements = class.body(visitor.ast()).body(visitor.ast());
    for id in elements.iter() {
        let element = visitor.ast().get_node_in_sub_range(id);
        match visitor.ast().class_element_data(element) {
            ClassElementData::MethodDefinition(method) => {
                let decorators = method.decorators(visitor.ast());
                if let Some(decorators) = decorators.visit_mut_with(visitor) {
                    method.set_decorators(visitor.ast(), decorators);
                }
                let parameters = method.value(visitor.ast()).params(visitor.ast());
                visit_parameter_decorators(parameters, visitor);
                let key = method.key(visitor.ast());
                if let Some(key) = key.visit_mut_with(visitor) {
                    method.set_key(visitor.ast(), key);
                }
            }
            ClassElementData::PropertyDefinition(property) => {
                let decorators = property.decorators(visitor.ast());
                if let Some(decorators) = decorators.visit_mut_with(visitor) {
                    property.set_decorators(visitor.ast(), decorators);
                }
                let key = property.key(visitor.ast());
                if let Some(key) = key.visit_mut_with(visitor) {
                    property.set_key(visitor.ast(), key);
                }
            }
            _ => {}
        }
    }
}

fn visit_parameter_decorators<'ast>(
    parameters: FormalParameters,
    visitor: &mut impl VisitMut<'ast>,
) {
    let items = parameters.items(visitor.ast());
    for id in items.iter() {
        let item = visitor.ast().get_node_in_sub_range(id);
        let parameter = match visitor.ast().formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(parameter) => parameter,
            FormalParameterItemData::TsParameterProperty(property) => {
                let decorators = property.decorators(visitor.ast());
                if let Some(decorators) = decorators.visit_mut_with(visitor) {
                    property.set_decorators(visitor.ast(), decorators);
                }
                property.parameter(visitor.ast())
            }
        };
        let decorators = parameter.decorators(visitor.ast());
        if let Some(decorators) = decorators.visit_mut_with(visitor) {
            parameter.set_decorators(visitor.ast(), decorators);
        }
    }
    if let Some(rest) = parameters.rest(visitor.ast()) {
        let decorators = rest.decorators(visitor.ast());
        if let Some(decorators) = decorators.visit_mut_with(visitor) {
            rest.set_decorators(visitor.ast(), decorators);
        }
    }
}
