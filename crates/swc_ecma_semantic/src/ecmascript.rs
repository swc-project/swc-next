use swc_next_ecma_ast::{
    ArrayPattern, Ast, AstNode, BindingRestElement, FormalParameter, FormalParameters, NodeId,
    NodeKind, ObjectPattern, Span, TsParameterProperty,
};

#[derive(Clone, Copy)]
pub(crate) struct PropName<'a> {
    pub name: &'a [u8],
    pub span: Span,
    pub is_string_literal: bool,
}

impl PropName<'_> {
    pub fn equals(self, expected: &[u8]) -> bool {
        self.name == expected
    }
}

#[inline]
unsafe fn typed<N: AstNode>(node: NodeId) -> N {
    unsafe { N::from_node_id_unchecked(node) }
}

/// <https://tc39.es/ecma262/#sec-static-semantics-propname>
pub(crate) fn prop_name<'a>(tree: &'a Ast<'_>, key: NodeId) -> Option<PropName<'a>> {
    match tree.node_kind(key) {
        NodeKind::IdentifierName => {
            let id = unsafe { typed::<swc_next_ecma_ast::IdentifierName>(key) };
            Some(PropName {
                name: tree.string(id.name(tree)),
                span: tree.span(key),
                is_string_literal: false,
            })
        }
        NodeKind::StringLiteral => {
            let string = unsafe { typed::<swc_next_ecma_ast::StringLiteral>(key) };
            let name = tree.string(string.value(tree));
            (!name.is_empty()).then_some(PropName {
                name,
                span: tree.span(key),
                is_string_literal: true,
            })
        }
        _ => None,
    }
}

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
        let pattern = match tree.node_kind(parameter) {
            NodeKind::FormalParameter => unsafe { typed::<FormalParameter>(parameter) }
                .pattern(tree)
                .node_id(),
            NodeKind::TsParameterProperty => unsafe { typed::<TsParameterProperty>(parameter) }
                .parameter(tree)
                .node_id(),
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
    match tree.node_kind(pattern) {
        NodeKind::AssignmentPattern => Some(pattern),
        NodeKind::BindingRestElement => {
            let rest = unsafe { typed::<BindingRestElement>(pattern) };
            find_pattern_expression(tree, Some(rest.argument(tree).node_id()))
        }
        NodeKind::ArrayPattern => {
            let array = unsafe { typed::<ArrayPattern>(pattern) };
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
        NodeKind::ObjectPattern => {
            let object = unsafe { typed::<ObjectPattern>(pattern) };
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
        let pattern = match tree.node_kind(parameter) {
            NodeKind::FormalParameter => unsafe { typed::<FormalParameter>(parameter) }
                .pattern(tree)
                .node_id(),
            NodeKind::TsParameterProperty => unsafe { typed::<TsParameterProperty>(parameter) }
                .parameter(tree)
                .node_id(),
            _ => return Some(parameter),
        };
        if !matches!(
            tree.node_kind(pattern),
            NodeKind::BindingIdentifier | NodeKind::TsThisParameter
        ) {
            return Some(pattern);
        }
    }

    None
}
