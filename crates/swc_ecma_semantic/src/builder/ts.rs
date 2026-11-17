use swc_next_ecma_ast::{FormalParameterPatternData, Lang, SourceType, TsParameterProperty};

use super::*;
use crate::binder::{TsImportAliasIssue, TsNamespaceExportAliasIssue};

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

pub(super) fn check_ts_alias_issues(
    builder: &mut SemanticBuilder<'_, '_>,
    export_alias_issues: impl IntoIterator<Item = TsNamespaceExportAliasIssue>,
    import_alias_issues: impl IntoIterator<Item = TsImportAliasIssue>,
) {
    for issue in export_alias_issues {
        match issue {
            TsNamespaceExportAliasIssue::Missing { name, node } => {
                check_ts_namespace_export_alias_missing(builder, name, node);
            }
            TsNamespaceExportAliasIssue::NonLocal { name, node } => {
                check_ts_namespace_export_alias_non_local(builder, name, node);
            }
            TsNamespaceExportAliasIssue::Duplicate { node, previous } => {
                check_ts_namespace_export_alias_duplicate(builder, node, previous);
            }
            TsNamespaceExportAliasIssue::Conflict { node, previous } => {
                check_ts_namespace_export_alias_conflict(builder, node, previous);
            }
        }
    }
    for issue in import_alias_issues {
        match issue {
            TsImportAliasIssue::Conflict(issue) => {
                check_ts_import_alias_conflict(builder, issue);
            }
            TsImportAliasIssue::TypeOnlyExport { node, export } => {
                check_ts_import_alias_type_only_export(builder, node, export);
            }
        }
    }
}

fn check_ts_namespace_export_alias_missing(
    builder: &mut SemanticBuilder<'_, '_>,
    name: swc_next_ecma_ast::AstUtf8,
    node: NodeId,
) {
    if !builder.check_syntax.unresolved_exports() {
        return;
    }
    let name = builder.ast.get_utf8(name).as_bytes();
    builder.report_unresolved_export(builder.ast, name, node);
}

fn check_ts_namespace_export_alias_non_local(
    builder: &mut SemanticBuilder<'_, '_>,
    name: swc_next_ecma_ast::AstUtf8,
    node: NodeId,
) {
    if !builder.check_syntax.ts_non_local_exports() {
        return;
    }
    let name = builder.ast.get_utf8(name).as_bytes();
    builder.report_non_local_export(builder.ast, name, node);
}

fn check_ts_namespace_export_alias_duplicate(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    previous: NodeId,
) {
    if !builder.check_syntax.ts_duplicate_export_aliases() {
        return;
    }
    check_ts_namespace_export_alias_conflict_with_kind(builder, node, previous, true);
}

fn check_ts_namespace_export_alias_conflict(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    previous: NodeId,
) {
    if !builder.check_syntax.ts_export_alias_conflicts() {
        return;
    }
    check_ts_namespace_export_alias_conflict_with_kind(builder, node, previous, false);
}

fn check_ts_namespace_export_alias_conflict_with_kind(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    previous: NodeId,
    duplicate: bool,
) {
    let NodeKindData::ExportSpecifier(specifier) = builder.ast.kind_data(node) else {
        unreachable!("namespace export aliases are export specifiers")
    };
    let exported = specifier.exported(builder.ast).node_id();
    let name = module_export_name(builder.ast, exported);
    builder.report_ts_namespace_export_alias_conflict(builder.ast, name, node, previous, duplicate);
}

fn check_ts_import_alias_conflict(
    builder: &mut SemanticBuilder<'_, '_>,
    issue: crate::binder::TsImportAliasConflictIssue,
) {
    if !builder.check_syntax.ts_import_alias_conflicts() {
        return;
    }
    let name = builder.ast.get_wtf8(issue.name).as_bytes();
    builder.report_redeclaration(
        builder.ast,
        name,
        issue.node,
        issue.previous,
        issue.previous_flags,
    );
}

fn check_ts_import_alias_type_only_export(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    export: NodeId,
) {
    if !builder.check_syntax.ts_type_only_import_aliases() {
        return;
    }
    builder.report_ts_import_alias_type_only_export(builder.ast, node, export);
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    fn report_ts_namespace_export_alias_conflict(
        &mut self,
        tree: &Ast<'a>,
        name: &[u8],
        node: NodeId,
        previous: NodeId,
        duplicate: bool,
    ) {
        let name = String::from_utf8_lossy(name);
        let message = if duplicate {
            self.fmt(format_args!("Export '{}' has already been declared", name))
        } else {
            self.fmt(format_args!(
                "Export declaration conflicts with exported declaration of '{}'",
                name
            ))
        };
        let current_span = tree.span(node);
        let previous_span = tree.span(previous);
        let labels = self.labels(&[
            self.label(previous_span, "the existing export is declared here"),
            self.label(current_span, "this export conflicts with it"),
        ]);
        self.report(
            current_span,
            message,
            ReportOptions {
                labels,
                ..ReportOptions::default()
            },
        );
    }

    fn report_ts_import_alias_type_only_export(
        &mut self,
        tree: &Ast<'a>,
        node: NodeId,
        export: NodeId,
    ) {
        let current_span = tree.span(node);
        let export_span = tree.span(export);
        let labels = self.labels(&[
            self.label(
                export_span,
                "the declaration was exported as type-only here",
            ),
            self.label(current_span, "this import alias references it here"),
        ]);
        self.report(
            current_span,
            "An import alias cannot reference a declaration that was exported using 'export type'",
            ReportOptions {
                labels,
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_ts_export_assignment(
    builder: &mut SemanticBuilder<'_, '_>,
    assignment: TsExportAssignment,
) {
    let expression = assignment.expression(builder.ast);
    check_ts_ambient_export_assignment_expression(builder, expression.node_id());
}

pub(super) fn check_ts_ambient_export_assignment_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: NodeId,
) {
    if !builder.check_syntax.ts_ambient_export_assignments() {
        return;
    }
    if !builder.symbols.in_ambient_context()
        || is_ts_entity_name_expression(builder.ast, expression)
        || !is_export_assignment_container(builder)
    {
        return;
    }
    builder.report(
        builder.ast.span(expression),
        "The expression of an export assignment must be an identifier or qualified name in an ambient context",
        ReportOptions::default(),
    );
}

fn is_ts_entity_name_expression(tree: &Ast<'_>, expression: NodeId) -> bool {
    let mut current = expression;
    loop {
        current = crate::ecmascript::unwrap_parentheses(tree, current);
        match tree.kind_data(current) {
            NodeKindData::IdentifierReference(_) => return true,
            NodeKindData::ChainExpression(chain) => {
                current = chain.expression(tree).node_id();
            }
            NodeKindData::MemberExpression(member) if !member.computed(tree) => {
                if !matches!(
                    tree.property_key_data(member.property(tree)),
                    PropertyKeyData::IdentifierName(_)
                ) {
                    return false;
                }
                current = member.object(tree).node_id();
            }
            _ => return false,
        }
    }
}

fn is_export_assignment_container(builder: &SemanticBuilder<'_, '_>) -> bool {
    let Some(parent) = builder.parent() else {
        return false;
    };
    match builder.ast.kind_data(parent) {
        NodeKindData::Program(_) => true,
        NodeKindData::TsModuleBlock(_) => {
            let Some(container) = builder.ancestors().nth(2) else {
                return false;
            };
            match builder.ast.kind_data(container) {
                NodeKindData::TsGlobalDeclaration(_) => true,
                NodeKindData::TsModuleDeclaration(declaration) => matches!(
                    builder.ast.ts_module_name_data(declaration.id(builder.ast)),
                    TsModuleNameData::StringLiteral(_)
                ),
                _ => false,
            }
        }
        _ => false,
    }
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

#[cold]
#[inline(never)]
pub(super) fn check_contextual_await(
    checker: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    message: &'static str,
) {
    if !checker.check_syntax.ts_contextual_await() {
        return;
    }
    let context = &*checker;
    if context.in_type_position()
        || context.symbols.in_ambient_context()
        || !context.ast.is_module()
        || !is_in_top_level_context(context, node)
    {
        return;
    }
    checker.report(context.ast.span(node), message, ReportOptions::default());
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

pub(super) fn check_parameter_property(
    checker: &mut SemanticBuilder<'_, '_>,
    property: TsParameterProperty,
) {
    if !checker.check_syntax.ts_parameter_properties() {
        return;
    }
    let Some(binding) = parameter_property_binding(checker.ast, property) else {
        checker.report(
            checker.ast.span(property.node_id()),
            "A parameter property must use an identifier binding",
            ReportOptions::default(),
        );
        return;
    };
    if checker.ast.get_utf8(binding.name(checker.ast)).as_bytes() == b"constructor" {
        checker.report(
            checker.ast.span(property.node_id()),
            "A parameter property cannot be named 'constructor'",
            ReportOptions::default(),
        );
    }
}
