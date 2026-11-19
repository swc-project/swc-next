//! AST queries used by the semantic builder and syntax checks.

use swc_next_ecma_ast::{
    Ast, BindingIdentifier, FormalParameterItemData, FormalParameters, NodeId, NodeKind,
    NodeKindData, SourceType, TsModuleNameData, TsParameterProperty,
};

/// <https://tc39.es/ecma262/#sec-static-semantics-containsexpression>
///
/// The first expression contained in the parameter list, a default
/// initializer or a computed property key. Plain destructuring without
/// either contains no expression.
pub(super) fn find_parameter_expression(
    tree: &Ast<'_>,
    params: FormalParameters,
) -> Option<NodeId> {
    for parameter in params.items(tree).iter() {
        let item = tree.get_node_in_sub_range(parameter);
        let parameter = match tree.formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(parameter) => parameter,
            FormalParameterItemData::TsParameterProperty(property) => property.parameter(tree),
        };
        if let Some(initializer) = parameter.initializer(tree) {
            return Some(initializer.node_id());
        }
        let pattern = parameter.pattern(tree).node_id();
        if let Some(found) = find_pattern_expression(tree, pattern) {
            return Some(found);
        }
    }
    let rest = params.rest(tree)?;
    find_pattern_expression(tree, rest.argument(tree).node_id())
}

/// <https://tc39.es/ecma262/#sec-static-semantics-containsexpression>
///
/// The first expression contained in one binding pattern, see
/// [`find_parameter_expression`].
pub(super) fn find_pattern_expression(tree: &Ast<'_>, pattern: NodeId) -> Option<NodeId> {
    match tree.kind_data(pattern) {
        NodeKindData::AssignmentPattern(_) => Some(pattern),
        NodeKindData::BindingRestElement(rest) => {
            find_pattern_expression(tree, rest.argument(tree).node_id())
        }
        NodeKindData::ArrayPattern(array) => {
            for element in array.elements(tree).iter() {
                let Some(element) = tree.get_node_in_sub_range(element) else {
                    continue;
                };
                if let Some(found) = find_pattern_expression(tree, element.node_id()) {
                    return Some(found);
                }
            }
            let rest = array.rest(tree)?;
            find_pattern_expression(tree, rest.node_id())
        }
        NodeKindData::ObjectPattern(object) => {
            for property in object.properties(tree).iter() {
                let property = tree.get_node_in_sub_range(property);
                if property.computed(tree) {
                    return Some(property.key(tree).node_id());
                }
                let value = property.value(tree).node_id();
                if let Some(found) = find_pattern_expression(tree, value) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

/// <https://tc39.es/ecma262/#sec-static-semantics-issimpleparameterlist>
pub(super) fn find_non_simple_parameter(
    tree: &Ast<'_>,
    params: FormalParameters,
) -> Option<NodeId> {
    if let Some(rest) = params.rest(tree) {
        return Some(rest.node_id());
    }

    for parameter in params.items(tree).iter() {
        let item = tree.get_node_in_sub_range(parameter);
        let parameter = match tree.formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(parameter) => parameter,
            FormalParameterItemData::TsParameterProperty(property) => property.parameter(tree),
        };
        if parameter.initializer(tree).is_some() {
            return Some(parameter.node_id());
        }
        let pattern = parameter.pattern(tree).node_id();
        if !matches!(tree.node_kind(pattern), NodeKind::BindingIdentifier) {
            return Some(pattern);
        }
    }

    None
}

pub(super) fn is_eval_or_arguments(name: &[u8]) -> bool {
    matches!(name, b"eval" | b"arguments")
}

pub(super) fn is_eval_or_arguments_reference(tree: &Ast<'_>, node: NodeId) -> bool {
    let NodeKindData::IdentifierReference(identifier) = tree.kind_data(node) else {
        return false;
    };
    is_eval_or_arguments(tree.get_utf8(identifier.name(tree)).as_bytes())
}

pub(super) fn unwrap_parentheses(tree: &Ast<'_>, node: NodeId) -> NodeId {
    let mut current = node;
    let mut depth = 0u32;
    // The parser builds at most one parenthesis node per syntactic
    // level, so this chain is bounded by source nesting depth.
    loop {
        debug_assert!(depth < 1_000_000);
        let NodeKindData::ParenthesizedExpression(parenthesized) = tree.kind_data(current) else {
            return current;
        };
        current = parenthesized.expression(tree).node_id();
        depth += 1;
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum PrivateElementKind {
    Field,
    Method,
    Getter,
    Setter,
}

/// Returns the StringValue of a class element's PrivateIdentifier key, if any.
pub(super) fn private_element_name<'a>(tree: &'a Ast<'_>, node: NodeId) -> Option<&'a [u8]> {
    let key = match tree.kind_data(node) {
        NodeKindData::PropertyDefinition(property) => property.key(tree).node_id(),
        NodeKindData::MethodDefinition(method) => method.key(tree).node_id(),
        NodeKindData::TsMethodDefinition(method) => method.key(tree).node_id(),
        _ => return None,
    };
    let NodeKindData::PrivateIdentifier(identifier) = tree.kind_data(key) else {
        return None;
    };
    Some(tree.get_utf8(identifier.name(tree)).as_bytes())
}

pub(super) fn private_element_kind(tree: &Ast<'_>, node: NodeId) -> PrivateElementKind {
    let kind = match tree.kind_data(node) {
        NodeKindData::MethodDefinition(method) => Some(method.kind(tree)),
        NodeKindData::TsMethodDefinition(method) => Some(method.kind(tree)),
        _ => None,
    };
    match kind {
        Some(swc_next_ecma_ast::MethodDefinitionKind::Get) => PrivateElementKind::Getter,
        Some(swc_next_ecma_ast::MethodDefinitionKind::Set) => PrivateElementKind::Setter,
        Some(_) => PrivateElementKind::Method,
        None => PrivateElementKind::Field,
    }
}

pub(super) fn private_element_is_static(tree: &Ast<'_>, node: NodeId) -> bool {
    match tree.kind_data(node) {
        NodeKindData::PropertyDefinition(property) => property.r#static(tree),
        NodeKindData::MethodDefinition(method) => method.r#static(tree),
        NodeKindData::TsMethodDefinition(method) => method.r#static(tree),
        _ => false,
    }
}

pub(super) fn parameter_property_binding(
    tree: &Ast<'_>,
    property: TsParameterProperty,
) -> Option<BindingIdentifier> {
    let parameter = property.parameter(tree);
    let pattern = parameter.pattern(tree);
    pattern.as_binding_identifier(tree)
}

pub(super) fn is_function_boundary(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Function | NodeKind::ArrowFunctionExpression | NodeKind::StaticBlock
    )
}

pub(super) fn is_iteration(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::ForStatement
            | NodeKind::ForInStatement
            | NodeKind::ForOfStatement
            | NodeKind::WhileStatement
            | NodeKind::DoWhileStatement
    )
}

pub(super) fn import_attribute_name<'a>(tree: &'a Ast<'_>, key: NodeId) -> Option<&'a [u8]> {
    match tree.kind_data(key) {
        NodeKindData::IdentifierName(identifier) => {
            Some(tree.get_utf8(identifier.name(tree)).as_bytes())
        }
        NodeKindData::StringLiteral(string) => Some(tree.get_wtf8(string.value(tree)).as_bytes()),
        _ => None,
    }
}

/// Extracts the name from a ModuleExportName: an IdentifierName,
/// IdentifierReference, or StringLiteral.
pub(super) fn module_export_name<'a>(tree: &Ast<'a>, node: NodeId) -> &'a [u8] {
    match tree.kind_data(node) {
        NodeKindData::IdentifierName(identifier) => tree.get_utf8(identifier.name(tree)).as_bytes(),
        NodeKindData::IdentifierReference(identifier) => {
            tree.get_utf8(identifier.name(tree)).as_bytes()
        }
        NodeKindData::StringLiteral(literal) => tree.get_wtf8(literal.value(tree)).as_bytes(),
        _ => b"",
    }
}

/// Recognizes external augmentation containers: an ESM/CommonJS program or the
/// direct body of a top-level ambient module in a Script.
/// `parent` is the effective parent after skipping an export declaration wrapper,
/// and `ancestors` starts immediately above it.
pub(super) fn is_ts_module_augmentation_external(
    tree: &Ast<'_>,
    parent: NodeId,
    mut ancestors: impl Iterator<Item = NodeId>,
) -> bool {
    let external_module = matches!(
        tree.source_type(),
        SourceType::Module | SourceType::CommonJs
    );
    let parent_kind = tree.node_kind(parent);
    if parent_kind == NodeKind::Program {
        return external_module;
    }
    // In an external module, a quoted module declaration is an augmentation.
    if parent_kind != NodeKind::TsModuleBlock || external_module {
        return false;
    }
    let Some(owner) = ancestors.next() else {
        return false;
    };
    let ambient_module = match tree.kind_data(owner) {
        NodeKindData::TsModuleDeclaration(module) => matches!(
            tree.ts_module_name_data(module.id(tree)),
            TsModuleNameData::StringLiteral(_)
        ),
        NodeKindData::TsGlobalDeclaration(_) => true,
        _ => false,
    };
    if !ambient_module {
        return false;
    }
    let Some(program) = ancestors.next() else {
        return false;
    };
    tree.node_kind(program) == NodeKind::Program
}

pub(super) fn is_ts_module_element_container(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Program | NodeKind::TsModuleBlock | NodeKind::TsModuleDeclaration
    )
}
