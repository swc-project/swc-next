use super::*;
use crate::binder::{TsImportAliasIssue, TsNamespaceExportAliasIssue};

pub(super) fn check_ts_alias_issues(
    checker: &mut Checker<'_, '_>,
    export_alias_issues: impl IntoIterator<Item = TsNamespaceExportAliasIssue>,
    import_alias_issues: impl IntoIterator<Item = TsImportAliasIssue>,
) {
    for issue in export_alias_issues {
        match issue {
            TsNamespaceExportAliasIssue::Missing { name, node } => {
                check_ts_namespace_export_alias_missing(checker, name, node);
            }
            TsNamespaceExportAliasIssue::NonLocal { name, node } => {
                check_ts_namespace_export_alias_non_local(checker, name, node);
            }
            TsNamespaceExportAliasIssue::Duplicate { node, previous } => {
                check_ts_namespace_export_alias_duplicate(checker, node, previous);
            }
            TsNamespaceExportAliasIssue::Conflict { node, previous } => {
                check_ts_namespace_export_alias_conflict(checker, node, previous);
            }
        }
    }
    for issue in import_alias_issues {
        match issue {
            TsImportAliasIssue::Conflict(issue) => {
                check_ts_import_alias_conflict(checker, issue);
            }
            TsImportAliasIssue::TypeOnlyExport { node, export } => {
                check_ts_import_alias_type_only_export(checker, node, export);
            }
        }
    }
}

fn check_ts_namespace_export_alias_missing(
    checker: &mut Checker<'_, '_>,
    name: swc_next_ecma_ast::AstUtf8,
    node: NodeId,
) {
    let checks = &mut checker.checks;
    let context = &checker.context;
    let name = context.ast.get_utf8(name).as_bytes();
    checks.report_unresolved_export(context.ast, name, node);
}

fn check_ts_namespace_export_alias_non_local(
    checker: &mut Checker<'_, '_>,
    name: swc_next_ecma_ast::AstUtf8,
    node: NodeId,
) {
    let checks = &mut checker.checks;
    let context = &checker.context;
    let name = context.ast.get_utf8(name).as_bytes();
    checks.report_non_local_export(context.ast, name, node);
}

fn check_ts_namespace_export_alias_duplicate(
    checker: &mut Checker<'_, '_>,
    node: NodeId,
    previous: NodeId,
) {
    check_ts_namespace_export_alias_conflict_with_kind(checker, node, previous, true);
}

fn check_ts_namespace_export_alias_conflict(
    checker: &mut Checker<'_, '_>,
    node: NodeId,
    previous: NodeId,
) {
    check_ts_namespace_export_alias_conflict_with_kind(checker, node, previous, false);
}

fn check_ts_namespace_export_alias_conflict_with_kind(
    checker: &mut Checker<'_, '_>,
    node: NodeId,
    previous: NodeId,
    duplicate: bool,
) {
    let checks = &mut checker.checks;
    let context = &checker.context;
    let NodeKindData::ExportSpecifier(specifier) = context.ast.kind_data(node) else {
        unreachable!("namespace export aliases are export specifiers")
    };
    let exported = specifier.exported(context.ast).node_id();
    let name = module_export_name(context.ast, exported);
    checks.report_ts_namespace_export_alias_conflict(context.ast, name, node, previous, duplicate);
}

fn check_ts_import_alias_conflict(
    checker: &mut Checker<'_, '_>,
    issue: crate::binder::TsImportAliasConflictIssue,
) {
    let checks = &mut checker.checks;
    let context = &checker.context;
    let name = context.ast.get_wtf8(issue.name).as_bytes();
    checks.report_redeclaration(
        context.ast,
        name,
        issue.node,
        issue.previous,
        issue.previous_flags,
    );
}

fn check_ts_import_alias_type_only_export(
    checker: &mut Checker<'_, '_>,
    node: NodeId,
    export: NodeId,
) {
    checker
        .checks
        .report_ts_import_alias_type_only_export(checker.context.ast, node, export);
}

impl<'traverse, 'a> CheckerState<'traverse, 'a> {
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
    checker: &mut Checker<'_, '_>,
    assignment: TsExportAssignment,
) {
    let expression = assignment.expression(checker.context.ast);
    check_ts_ambient_export_assignment_expression(checker, expression.node_id());
}

pub(super) fn check_ts_ambient_export_assignment_expression(
    checker: &mut Checker<'_, '_>,
    expression: NodeId,
) {
    let context = &checker.context;
    if !context.state.symbols.in_ambient_context()
        || is_ts_entity_name_expression(context.ast, expression)
        || !is_export_assignment_container(context)
    {
        return;
    }
    checker.checks.report(
        context.ast.span(expression),
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

fn is_export_assignment_container(context: &SemanticVisitCtx<'_, '_>) -> bool {
    let Some(parent) = context.parent() else {
        return false;
    };
    match context.ast.kind_data(parent) {
        NodeKindData::Program(_) => true,
        NodeKindData::TsModuleBlock(_) => {
            let Some(container) = context.ancestors().nth(2) else {
                return false;
            };
            match context.ast.kind_data(container) {
                NodeKindData::TsGlobalDeclaration(_) => true,
                NodeKindData::TsModuleDeclaration(declaration) => matches!(
                    context.ast.ts_module_name_data(declaration.id(context.ast)),
                    TsModuleNameData::StringLiteral(_)
                ),
                _ => false,
            }
        }
        _ => false,
    }
}
