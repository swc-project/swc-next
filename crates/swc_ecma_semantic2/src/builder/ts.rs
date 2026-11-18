use swc_next_ecma_ast::{
    FormalParameterPatternData, Lang, PropertyKeyData, SourceType, TsParameterProperty,
};

use super::*;

impl SemanticBuilder<'_, '_> {
    pub(super) fn record_expando_assignment(&mut self, assignment: AssignmentExpression) {
        let tree = self.ast;
        let NodeKindData::MemberExpression(member) =
            tree.kind_data(assignment.left(tree).node_id())
        else {
            return;
        };
        let PropertyKeyData::IdentifierName(property) =
            tree.property_key_data(member.property(tree))
        else {
            return;
        };
        // The parser can strip parentheses from the assignment target.
        if member.computed(tree)
            || member.optional(tree)
            || tree.span(assignment.node_id()).start != tree.span(member.node_id()).start
        {
            return;
        }

        // tscgo looks in the nearest block and its containing declaration.
        // Object literals have no lexical scope here but still stop that lookup.
        let container = self
            .scope
            .ancestors(Some(self.current_scope))
            .find(|&scope| self.scope.get(scope).kind != ScopeKind::Block)
            .unwrap();
        let container_node = self.scope.get(container).node;
        if self
            .ancestors()
            .take_while(|&node| node != container_node)
            .any(|node| {
                matches!(
                    tree.node_kind(node),
                    NodeKind::ObjectExpression | NodeKind::JsxOpeningElement
                )
            })
        {
            return;
        }
        self.symbols
            .record_expando(member.object(tree), property, self.current_scope, container);
    }
}

fn ts_module_declaration_context_error(
    builder: &SemanticBuilder<'_, '_>,
    declaration: TsModuleDeclaration,
) -> Option<(NodeId, &'static str)> {
    let (parent, ancestors) = ts_module_element_context(builder)?;
    let parent_kind = builder.ast.node_kind(parent);
    let quoted = matches!(
        builder.ast.ts_module_name_data(declaration.id(builder.ast)),
        TsModuleNameData::StringLiteral(_)
    );
    if !is_ts_module_element_container(parent_kind) {
        let message = if quoted {
            "An ambient module declaration is only allowed at the top level in a file"
        } else {
            "A namespace declaration is only allowed at the top level of a namespace or module"
        };
        return Some((declaration.node_id(), message));
    }
    if quoted
        && parent_kind != NodeKind::Program
        && !is_ts_module_augmentation_external(builder.ast, parent, ancestors)
    {
        return Some((
            declaration.id(builder.ast).node_id(),
            "Ambient modules cannot be nested in other modules or namespaces",
        ));
    }
    None
}

fn ts_global_declaration_context_error(
    builder: &SemanticBuilder<'_, '_>,
    declaration: TsGlobalDeclaration,
) -> Option<(NodeId, &'static str)> {
    let (parent, ancestors) = ts_module_element_context(builder)?;
    let parent_kind = builder.ast.node_kind(parent);
    if !is_ts_module_element_container(parent_kind) {
        return Some((
            declaration.node_id(),
            "An ambient module declaration is only allowed at the top level in a file",
        ));
    }

    if is_ts_module_augmentation_external(builder.ast, parent, ancestors) {
        return None;
    }
    Some((
        declaration.id(builder.ast).node_id(),
        "Augmentations for the global scope can only be directly nested in external modules or ambient module declarations",
    ))
}

/// Recognizes external augmentation containers: an ESM/CommonJS program or the
/// direct body of a top-level ambient module in a Script.
/// `parent` is the effective parent after skipping an export declaration wrapper,
/// and `ancestors` starts immediately above it.
fn is_ts_module_augmentation_external(
    tree: &Ast<'_>,
    parent: NodeId,
    mut ancestors: impl Iterator<Item = NodeId>,
) -> bool {
    let external_module = matches!(
        tree.source_type(),
        SourceType::Module | SourceType::CommonJs
    );
    match tree.node_kind(parent) {
        NodeKind::Program => external_module,
        // In an external module, a quoted module declaration is an augmentation.
        NodeKind::TsModuleBlock if !external_module => {
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
        _ => false,
    }
}

pub(super) fn in_ts_syntactic_ambient_context(builder: &SemanticBuilder<'_, '_>) -> bool {
    if builder.ast.lang() == Lang::Dts {
        return true;
    }
    // Binding recovery also makes bare global bodies ambient. Only syntax can
    // exempt an inner augmentation from requiring its own declare modifier.
    builder
        .ancestors()
        .skip(1)
        .any(|node| match builder.ast.kind_data(node) {
            NodeKindData::VariableDeclaration(node) => node.declare(builder.ast),
            NodeKindData::Class(node) => node.declare(builder.ast),
            NodeKindData::TsFunction(node) => node.declare(builder.ast),
            NodeKindData::TsTypeAliasDeclaration(node) => node.declare(builder.ast),
            NodeKindData::TsInterfaceDeclaration(node) => node.declare(builder.ast),
            NodeKindData::TsEnumDeclaration(node) => node.declare(builder.ast),
            NodeKindData::TsModuleDeclaration(node) => node.declare(builder.ast),
            NodeKindData::TsGlobalDeclaration(node) => node.declare(builder.ast),
            _ => false,
        })
}

fn ts_module_element_context<'b>(
    builder: &'b SemanticBuilder<'_, '_>,
) -> Option<(NodeId, impl Iterator<Item = NodeId> + 'b)> {
    let mut ancestors = builder.ancestors().skip(1);
    let mut parent = ancestors.next()?;
    if builder.ast.node_kind(parent) == NodeKind::ExportNamedDeclaration {
        parent = ancestors.next()?;
    }
    Some((parent, ancestors))
}

fn is_ts_module_element_container(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Program | NodeKind::TsModuleBlock | NodeKind::TsModuleDeclaration
    )
}

/// Mirrors TypeScript's `isInTopLevelContext` test for contextual identifiers.
/// The parser owns `[Await]` grammar errors; this checker only adds the external
/// module rule, whose boundary depends on the surrounding runtime container.
fn is_in_top_level_context(context: &SemanticBuilder<'_, '_>, node: NodeId) -> bool {
    let tree = context.ast;
    for ancestor in context.ancestors().skip(1) {
        match tree.kind_data(ancestor) {
            NodeKindData::Program(_) => return true,
            NodeKindData::Function(function) => {
                let is_declaration_name = function.function_type(tree)
                    == FunctionType::FunctionDeclaration
                    && function.id(tree).is_some_and(|id| id.node_id() == node);
                if !is_declaration_name {
                    return function.r#async(tree);
                }
            }
            NodeKindData::TsFunction(function) => {
                let is_declaration_name = function.id(tree).is_some_and(|id| id.node_id() == node);
                if !is_declaration_name {
                    return function.r#async(tree);
                }
            }
            NodeKindData::ArrowFunctionExpression(_)
            | NodeKindData::StaticBlock(_)
            | NodeKindData::PropertyDefinition(_)
            | NodeKindData::MethodDefinition(_)
            | NodeKindData::TsMethodDefinition(_)
            | NodeKindData::TsEnumDeclaration(_)
            | NodeKindData::TsModuleDeclaration(_) => return false,
            _ => {}
        }
    }
    false
}

fn parameter_property_binding(
    tree: &Ast<'_>,
    property: TsParameterProperty,
) -> Option<swc_next_ecma_ast::BindingIdentifier> {
    let parameter = property.parameter(tree);
    let pattern = parameter.pattern(tree);
    let FormalParameterPatternData::BindingIdentifier(identifier) =
        tree.formal_parameter_pattern_data(pattern)
    else {
        return None;
    };
    Some(identifier)
}

pub(super) fn check_method_definition(
    builder: &mut SemanticBuilder<'_, '_>,
    method: MethodDefinition,
) {
    if !builder.check_syntax.ts_abstract_methods() {
        return;
    }
    if method.r#abstract(builder.ast) {
        builder.report(
            builder.ast.span(method.node_id()),
            "An abstract method cannot have an implementation",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_tuple_rest_type(builder: &mut SemanticBuilder<'_, '_>, rest: TsRestType) {
    if !builder.check_syntax.ts_tuple_rest() {
        return;
    }
    if let Some(member) = rest
        .type_annotation(builder.ast)
        .as_ts_named_tuple_member(builder.ast)
    {
        if member.optional(builder.ast) {
            builder.report(
                builder.ast.span(rest.node_id()),
                "A tuple member cannot be both optional and rest",
                ReportOptions::default(),
            );
        }
        return;
    }
    if let Some(parent) = builder.parent()
        && builder.ast.node_kind(parent) == NodeKind::TsNamedTupleMember
    {
        builder.report(
            builder.ast.span(rest.node_id()),
            "A labeled tuple element is declared as rest with '...' before the name, rather than before the type",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_tuple_optional_type(
    builder: &mut SemanticBuilder<'_, '_>,
    optional: TsOptionalType,
) {
    if !builder.check_syntax.ts_tuple_optional() {
        return;
    }
    if let Some(parent) = builder.parent()
        && builder.ast.node_kind(parent) == NodeKind::TsNamedTupleMember
    {
        builder.report(
            builder.ast.span(optional.node_id()),
            "A labeled tuple element is declared as optional with '?' after the name and before the colon, rather than after the type",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_ts_module_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: TsModuleDeclaration,
) {
    if !builder.check_syntax.ts_module_declarations() {
        return;
    }
    let Some((node, message)) = ts_module_declaration_context_error(builder, declaration) else {
        return;
    };
    builder.report(builder.ast.span(node), message, ReportOptions::default());
}

pub(super) fn check_ts_global_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: TsGlobalDeclaration,
) {
    if !builder.check_syntax.ts_global_declarations() {
        return;
    }
    if !declaration.declare(builder.ast) && !in_ts_syntactic_ambient_context(builder) {
        builder.report(
            builder.ast.span(declaration.id(builder.ast).node_id()),
            "Augmentations for the global scope should have 'declare' modifier unless they appear in already ambient context",
            ReportOptions::default(),
        );
    }
    let Some((node, message)) = ts_global_declaration_context_error(builder, declaration) else {
        return;
    };
    builder.report(builder.ast.span(node), message, ReportOptions::default());
}

#[cold]
pub(super) fn check_contextual_await(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    message: &'static str,
) {
    if !builder.check_syntax.ts_contextual_await() {
        return;
    }
    if builder.in_type_position()
        || builder.symbols.in_ambient_context()
        || !builder.ast.is_module()
        || !is_in_top_level_context(builder, node)
    {
        return;
    }
    builder.report(builder.ast.span(node), message, ReportOptions::default());
}

pub(super) fn check_parameter_property(
    builder: &mut SemanticBuilder<'_, '_>,
    property: TsParameterProperty,
) {
    if !builder.check_syntax.ts_parameter_properties() {
        return;
    }
    let Some(binding) = parameter_property_binding(builder.ast, property) else {
        builder.report(
            builder.ast.span(property.node_id()),
            "A parameter property must use an identifier binding",
            ReportOptions::default(),
        );
        return;
    };
    if builder.ast.get_utf8(binding.name(builder.ast)).as_bytes() == b"constructor" {
        builder.report(
            builder.ast.span(property.node_id()),
            "A parameter property cannot be named 'constructor'",
            ReportOptions::default(),
        );
    }
}
