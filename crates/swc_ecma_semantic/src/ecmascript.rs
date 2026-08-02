use swc_next_ecma_ast::{Ast, FormalParameters, NodeId, NodeKindData};

/// <https://tc39.es/ecma262/#sec-static-semantics-containsexpression>
///
/// The first expression contained in the parameter list, a default
/// initializer or a computed property key. Plain destructuring without
/// either contains no expression.
pub(crate) fn find_parameter_expression(
    tree: &Ast<'_>,
    params: FormalParameters,
) -> Option<NodeId> {
    for parameter in params.items(tree).iter() {
        let parameter = tree.get_node_in_sub_range(parameter).node_id();
        let pattern = match tree.kind_data(parameter) {
            NodeKindData::FormalParameter(parameter) => parameter.pattern(tree).node_id(),
            NodeKindData::TsParameterProperty(parameter) => parameter.parameter(tree).node_id(),
            _ => parameter,
        };
        if let Some(found) = find_pattern_expression(tree, Some(pattern)) {
            return Some(found);
        }
    }
    params
        .rest(tree)
        .and_then(|rest| find_pattern_expression(tree, Some(rest.node_id())))
}

/// <https://tc39.es/ecma262/#sec-static-semantics-containsexpression>
///
/// The first expression contained in one binding pattern, see
/// [`find_parameter_expression`]. Safe to call with `None`.
pub(crate) fn find_pattern_expression(tree: &Ast<'_>, pattern: Option<NodeId>) -> Option<NodeId> {
    let pattern = pattern?;
    match tree.kind_data(pattern) {
        NodeKindData::AssignmentPattern(_) => Some(pattern),
        NodeKindData::BindingRestElement(rest) => {
            find_pattern_expression(tree, Some(rest.argument(tree).node_id()))
        }
        NodeKindData::ArrayPattern(array) => {
            for element in array.elements(tree).iter() {
                let element = tree.get_node_in_sub_range(element);
                if let Some(found) = element
                    .and_then(|element| find_pattern_expression(tree, Some(element.node_id())))
                {
                    return Some(found);
                }
            }
            array
                .rest(tree)
                .and_then(|rest| find_pattern_expression(tree, Some(rest.node_id())))
        }
        NodeKindData::ObjectPattern(object) => {
            for property in object.properties(tree).iter() {
                let property = tree.get_node_in_sub_range(property);
                if property.computed(tree) {
                    return Some(property.key(tree).node_id());
                }
                if let Some(found) =
                    find_pattern_expression(tree, Some(property.value(tree).node_id()))
                {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

/// <https://tc39.es/ecma262/#sec-static-semantics-issimpleparameterlist>
pub(crate) fn find_non_simple_parameter(
    tree: &Ast<'_>,
    params: FormalParameters,
) -> Option<NodeId> {
    if let Some(rest) = params.rest(tree) {
        return Some(rest.node_id());
    }

    for parameter in params.items(tree).iter() {
        let parameter = tree.get_node_in_sub_range(parameter).node_id();
        let pattern = match tree.kind_data(parameter) {
            NodeKindData::FormalParameter(parameter) => parameter.pattern(tree).node_id(),
            NodeKindData::TsParameterProperty(parameter) => parameter.parameter(tree).node_id(),
            _ => return Some(parameter),
        };
        if !matches!(
            tree.kind_data(pattern),
            NodeKindData::BindingIdentifier(_) | NodeKindData::TsThisParameter(_)
        ) {
            return Some(pattern);
        }
    }

    None
}
