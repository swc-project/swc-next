use std::cmp::Ordering;

use swc_next_ecma_ast::*;

use crate::common::{build, resolver::context::Resolver};

use super::super::{
    module_record::{
        ExportName, LocalExportEntries, ModuleRecordEntry, RequestedModules, SourceModule,
        SourceModuleItem, default_export_name, external_ts_import_equals_source,
    },
    util::{self, ModuleCtx},
};
use super::{
    emit::export_values_call,
    ir::{DependencySlot, ExecuteStmt, ExportInit, ExportTable, SetterOp, SystemModule},
    pattern::replace_exported_pat,
};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L22>.
pub(super) fn lower(cx: &mut ModuleCtx<'_, '_>, source: SourceModule) -> SystemModule {
    let mut module = SystemModule::new(cx.private("_export_setters"));
    let mut exports = collect_local_exports(cx.ast, source.local_export_entries);
    exports
        .announced
        .extend(collect_default_value_exports(cx.ast, &source.body));
    exports
        .announced
        .sort_unstable_by_key(|export| cx.ast.get_utf8(export.name));
    module.exports = exports;

    lower_dependencies(cx, &mut module, source.requested_modules);
    for item in source.body {
        lower_source_item(cx, &mut module, item);
    }
    if let Some(value) = source.ts_export_assignment {
        module
            .execute_stmts
            .push(ExecuteStmt::ExportBatch { value });
    }
    module
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L78>.
fn collect_local_exports(ast: &Ast<'_>, local_export_entries: LocalExportEntries) -> ExportTable {
    let mut entries: Vec<_> = local_export_entries.into_values().collect();
    entries.sort_unstable_by_key(|export| ast.get_utf8(export.exported.name));
    let mut exports = ExportTable::default();
    for export in entries {
        exports.announced.push(export.exported);
        exports
            .local_exports
            .entry(export.local)
            .or_default()
            .push(export.exported);
    }
    exports
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L55>.
fn collect_default_value_exports(ast: &mut Ast<'_>, body: &[SourceModuleItem]) -> Vec<ExportName> {
    let mut exports = Vec::new();
    for item in body {
        let span = match *item {
            SourceModuleItem::ExportDefaultExpr { span, .. } => span,
            SourceModuleItem::ExportDefaultDecl(export) => {
                let anonymous =
                    match ast.export_default_declaration_kind_data(export.declaration(ast)) {
                        ExportDefaultDeclarationKindData::Class(class) => class.id(ast).is_none(),
                        ExportDefaultDeclarationKindData::Function(function) => {
                            function.id(ast).is_none()
                        }
                        _ => false,
                    };
                if !anonymous {
                    continue;
                }
                export.span(ast)
            }
            _ => continue,
        };
        exports.push(default_export_name(ast, span));
    }
    exports
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L92>.
fn lower_dependencies(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SystemModule,
    requested_modules: RequestedModules,
) {
    for (module_request, data) in requested_modules {
        let mut entries = data.entries;
        entries.sort_unstable_by(|a, b| compare_module_entries(cx.ast, *a, *b));
        let mut setter_ops = Vec::with_capacity(entries.len());
        for entry in entries {
            let op = match entry {
                ModuleRecordEntry::ImportDefault { local_name } => SetterOp::Import {
                    local: local_name,
                    imported: Some(cx.ast.add_utf8("default")),
                },
                ModuleRecordEntry::ImportNamespace { local_name } => SetterOp::Import {
                    local: local_name,
                    imported: None,
                },
                ModuleRecordEntry::ImportNamed {
                    import_name,
                    local_name,
                } => SetterOp::Import {
                    local: local_name,
                    imported: Some(import_name),
                },
                ModuleRecordEntry::IndirectExportDefault { export_name } => SetterOp::ReExport {
                    export: export_name,
                    imported: cx.ast.add_utf8("default"),
                },
                ModuleRecordEntry::IndirectExportNamed {
                    import_name,
                    export_name,
                } => SetterOp::ReExport {
                    export: export_name,
                    imported: import_name,
                },
                ModuleRecordEntry::IndirectExportNamespace { export_name } => {
                    SetterOp::ReExportNamespace {
                        export: export_name,
                    }
                }
                ModuleRecordEntry::StarExport => SetterOp::StarExport,
            };
            match op {
                SetterOp::Import { local, .. } => module.wrapper_vars.push(local),
                SetterOp::ReExport { export, .. } | SetterOp::ReExportNamespace { export } => {
                    module.exports.announced.push(export)
                }
                SetterOp::StarExport => {}
            }
            setter_ops.push(op);
        }
        let namespace = namespace_ident_for(cx, cx.ast.get_utf8(module_request.specifier));
        module.dependencies.push(DependencySlot {
            request: data.source,
            attributes: module_request.attributes,
            namespace,
            setter_ops,
        });
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L183>.
fn lower_source_item(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SystemModule,
    item: SourceModuleItem,
) {
    match item {
        SourceModuleItem::Stmt(stmt) => lower_stmt(cx, module, stmt),
        SourceModuleItem::ExportDecl(decl) => lower_stmt(cx, module, Stmt::Declaration(decl)),
        SourceModuleItem::ExportDefaultDecl(export) => lower_default_decl(cx, module, export),
        SourceModuleItem::ExportDefaultExpr { span, expr } => {
            let export = default_export_name(cx.ast, span);
            module.execute_stmts.push(ExecuteStmt::ExportValue {
                export,
                value: expr,
            });
        }
        SourceModuleItem::TsImportEquals { import, .. } => {
            lower_ts_import_equals(cx, module, import)
        }
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L202>.
fn lower_ts_import_equals(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SystemModule,
    import: TsImportEqualsDeclaration,
) {
    let local = import.id(cx.ast);
    let source = external_ts_import_equals_source(cx.ast, import).unwrap();
    let request = util::source_literal(cx.ast, source);
    let specifier = cx.ast.get_wtf8(request.value(cx.ast)).as_str().unwrap();
    let namespace = namespace_ident_for(cx, specifier);
    module.wrapper_vars.push(local);
    module.dependencies.push(DependencySlot {
        request,
        attributes: Vec::new(),
        namespace,
        setter_ops: vec![SetterOp::Import {
            local,
            imported: None,
        }],
    });
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L551>.
fn namespace_ident_for(cx: &mut ModuleCtx<'_, '_>, source: &str) -> BindingIdentifier {
    let name = util::local_name_for_src(source);
    cx.private(&format!("{name}_ns"))
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L560>.
fn compare_module_entries(ast: &Ast<'_>, a: ModuleRecordEntry, b: ModuleRecordEntry) -> Ordering {
    module_entry_rank(a)
        .cmp(&module_entry_rank(b))
        .then_with(|| module_entry_sort_name(ast, a).cmp(module_entry_sort_name(ast, b)))
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L566>.
fn module_entry_rank(entry: ModuleRecordEntry) -> u8 {
    match entry {
        ModuleRecordEntry::ImportDefault { .. } => 1,
        ModuleRecordEntry::ImportNamespace { .. } => 2,
        ModuleRecordEntry::ImportNamed { .. } => 3,
        ModuleRecordEntry::IndirectExportDefault { .. } => 4,
        ModuleRecordEntry::IndirectExportNamed { .. } => 5,
        ModuleRecordEntry::IndirectExportNamespace { .. } => 6,
        ModuleRecordEntry::StarExport => 7,
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L579>.
fn module_entry_sort_name<'a>(ast: &Ast<'a>, entry: ModuleRecordEntry) -> &'a str {
    match entry {
        ModuleRecordEntry::ImportDefault { local_name }
        | ModuleRecordEntry::ImportNamespace { local_name }
        | ModuleRecordEntry::ImportNamed { local_name, .. } => ast.get_utf8(local_name.name(ast)),
        ModuleRecordEntry::IndirectExportDefault { export_name }
        | ModuleRecordEntry::IndirectExportNamed { export_name, .. }
        | ModuleRecordEntry::IndirectExportNamespace { export_name } => {
            ast.get_utf8(export_name.name)
        }
        ModuleRecordEntry::StarExport => "",
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L225>.
fn lower_stmt(cx: &mut ModuleCtx<'_, '_>, module: &mut SystemModule, stmt: Stmt) {
    if let Some(decl) = stmt.as_declaration(cx.ast) {
        match cx.ast.decl_data(decl) {
            DeclData::Function(function) => {
                module.wrapper_fns.push(function);
                return;
            }
            DeclData::Class(class) => {
                if let Some(binding) = class.id(cx.ast) {
                    class.set_class_type(cx.ast, ClassType::ClassExpression);
                    module.wrapper_vars.push(binding);
                    let stmt = assign_or_export(cx, module, binding, Expr::Class(class));
                    module.execute_stmts.push(stmt);
                    return;
                }
            }
            DeclData::VariableDeclaration(var) => {
                lower_var_decl(cx, module, var);
                return;
            }
            _ => {}
        }
    }

    let mut visitor = VarScopedDecls {
        ast: cx.ast,
        resolver: cx.resolver,
        vars: &mut module.wrapper_vars,
    };
    let stmt = stmt.visit_mut_with(&mut visitor).unwrap_or(stmt);
    module.execute_stmts.push(ExecuteStmt::Source(stmt));
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L267>.
fn lower_default_decl(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SystemModule,
    export: ExportDefaultDeclaration,
) {
    let span = export.span(cx.ast);
    match cx
        .ast
        .export_default_declaration_kind_data(export.declaration(cx.ast))
    {
        ExportDefaultDeclarationKindData::Class(class) => {
            class.set_class_type(cx.ast, ClassType::ClassExpression);
            let value = Expr::Class(class);
            if let Some(binding) = class.id(cx.ast) {
                module.wrapper_vars.push(binding);
                let stmt = assign_or_export(cx, module, binding, value);
                module.execute_stmts.push(stmt);
            } else {
                let export = default_export_name(cx.ast, span);
                module
                    .execute_stmts
                    .push(ExecuteStmt::ExportValue { export, value });
            }
        }
        ExportDefaultDeclarationKindData::Function(function) => {
            if function.id(cx.ast).is_some() {
                function.set_function_type(cx.ast, FunctionType::FunctionDeclaration);
                module.wrapper_fns.push(function);
            } else {
                function.set_function_type(cx.ast, FunctionType::FunctionExpression);
                let export = default_export_name(cx.ast, span);
                module.export_inits.push(ExportInit {
                    export,
                    value: Expr::Function(function),
                });
            }
        }
        _ => {}
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L316>.
fn lower_var_decl(cx: &mut ModuleCtx<'_, '_>, module: &mut SystemModule, var: VariableDeclaration) {
    for slot in var.declarators(cx.ast).iter() {
        let declarator = cx.ast.get_node_in_sub_range(slot);
        let pattern = declarator.id(cx.ast);
        let init = declarator.init(cx.ast);
        if let Some(binding) = pattern.as_binding_identifier(cx.ast) {
            module.wrapper_vars.push(binding);
            if let Some(init) = init {
                let stmt = assign_or_export(cx, module, binding, init);
                module.execute_stmts.push(stmt);
            }
            continue;
        }

        module
            .wrapper_vars
            .extend(build::find_pat_ids(cx.ast, pattern));
        if let Some(right) = init {
            let target = build::assignment_target(cx.ast, cx.resolver, pattern);
            let left = replace_exported_pat(
                cx.ast,
                cx.resolver,
                &module.exports.local_exports,
                module.export_setters,
                target,
                &mut module.needs_export_setters,
            );
            module
                .execute_stmts
                .push(ExecuteStmt::Assign { left, right });
        }
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L364>.
fn assign_or_export(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SystemModule,
    local: BindingIdentifier,
    right: Expr,
) -> ExecuteStmt {
    let symbol = cx.resolver.symbol_of(local.node_id());
    if module.exports.local_exports.contains_key(&symbol) {
        ExecuteStmt::ExportAssign { local, right }
    } else {
        let local = cx.reference(local);
        let left = AssignmentTarget::SimpleAssignmentTarget(build::target(cx.ast, local));
        ExecuteStmt::Assign { left, right }
    }
}

// Functions are emitted in wrapper scope and need to close over vars declared
// inside execute statements. Lexical declarations in nested blocks stay there.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L372>.
struct VarScopedDecls<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    vars: &'a mut Vec<BindingIdentifier>,
}

impl VarScopedDecls<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L466>.
    fn var_decl_to_assignments(&mut self, var: VariableDeclaration) -> Option<Expr> {
        let mut expressions = Vec::new();
        for slot in var.declarators(self.ast).iter() {
            let decl = self.ast.get_node_in_sub_range(slot);
            let pattern = decl.id(self.ast);
            self.vars.extend(build::find_pat_ids(self.ast, pattern));
            if let Some(init) = decl.init(self.ast) {
                let target = build::assignment_target(self.ast, self.resolver, pattern);
                let span = decl.span(self.ast);
                expressions.push(self.ast.expr_assignment_expression(
                    span,
                    target,
                    init,
                    AssignmentOperator::Assign,
                ));
            }
        }
        (!expressions.is_empty()).then(|| build::sequence(self.ast, expressions))
    }
}

impl<'ast> VisitMut<'ast> for VarScopedDecls<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, _: Expr) -> Option<Expr> {
        None
    }

    fn visit_mut_decl(&mut self, _: Decl) -> Option<Decl> {
        None
    }

    fn visit_mut_stmt(&mut self, stmt: Stmt) -> Option<Stmt> {
        if let Some(decl) = stmt.as_declaration(self.ast)
            && let Some(var) = decl.as_variable_declaration(self.ast)
            && var.kind(self.ast) == VariableKind::Var
        {
            return Some(match self.var_decl_to_assignments(var) {
                Some(expr) => self.ast.stmt_expression_statement(DUMMY_SP, expr),
                None => self.ast.stmt_empty_statement(DUMMY_SP),
            });
        }
        stmt.visit_mut_children_with(self)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L427>.
    fn visit_mut_for_statement(&mut self, stmt: ForStatement) -> Option<ForStatement> {
        if let Some(var) = stmt
            .init(self.ast)
            .and_then(|init| init.as_variable_declaration(self.ast))
            && var.kind(self.ast) == VariableKind::Var
        {
            let replacement = self
                .var_decl_to_assignments(var)
                .map(ForStatementInit::Expr);
            stmt.set_init(self.ast, replacement);
        }
        stmt.visit_mut_children_with(self)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L438>.
    fn visit_mut_for_statement_left(&mut self, left: ForStatementLeft) -> Option<ForStatementLeft> {
        let var = left.as_variable_declaration(self.ast)?;
        if var.kind(self.ast) != VariableKind::Var {
            return None;
        }
        let slot = var.declarators(self.ast).iter().next()?;
        let pattern = self.ast.get_node_in_sub_range(slot).id(self.ast);
        self.vars.extend(build::find_pat_ids(self.ast, pattern));
        let target = build::assignment_target(self.ast, self.resolver, pattern);
        Some(ForStatementLeft::AssignmentTarget(target))
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L504>.
pub(super) fn export_call(
    cx: &mut ModuleCtx<'_, '_>,
    export: BindingIdentifier,
    names: &[ExportName],
    value: Expr,
) -> Expr {
    names.iter().fold(value, |value, name| {
        let name = util::string(cx.ast, name.span, cx.ast.get_utf8(name.name));
        let callback = cx.reference(export);
        build::call(cx.ast, callback, [name, value], false)
    })
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/lower.rs#L521>.
pub(super) fn export_names_call(
    cx: &mut ModuleCtx<'_, '_>,
    export: BindingIdentifier,
    names: &[ExportName],
    local: SymbolId,
) -> Expr {
    let values = names
        .iter()
        .map(|name| (*name, cx.resolver.symbol_reference(cx.ast, local)))
        .collect();
    export_values_call(cx, export, values)
}
