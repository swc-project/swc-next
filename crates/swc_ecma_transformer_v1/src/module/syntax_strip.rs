use swc_next_ecma_ast::*;

use crate::common::build;

use super::{
    ModuleType,
    module_record::{
        ExportName, LocalExportEntries, LocalExportEntry, RequestedModules, SourceModule,
        SourceModuleItem, binding_name, default_export_name, external_ts_import_equals_source,
        reduce,
    },
    module_ref_rewriter::{ImportBinding, ImportMap},
    util::{self, ModuleCtx, define_es_module, emit_export_stmts},
};

pub(super) struct Dependency {
    pub binding: BindingIdentifier,
    pub source: StringLiteral,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/syntax_strip.rs#L11>.
pub(super) struct SyntaxStrippedModule {
    pub directives: SubRange<Directive>,
    pub body: Vec<Stmt>,
    pub import_map: ImportMap,
    pub deps: Vec<Dependency>,
    pub exports: Option<BindingIdentifier>,
    pub is_export_assign: bool,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/syntax_strip.rs#L21>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L69>.
pub(super) fn lower(cx: &mut ModuleCtx<'_, '_>, format: ModuleType) -> SyntaxStrippedModule {
    let SourceModule {
        requested_modules,
        mut local_export_entries,
        ts_export_assignment: export_assign,
        body,
        has_module_syntax,
    } = SourceModule::collect(cx.ast, cx.resolver);
    let directives = cx.ast.root_program().directives(cx.ast);
    let directives = util::use_strict(cx.ast, directives);
    let mut module = SyntaxStrippedModule {
        directives,
        body: Vec::with_capacity(body.len() + 4),
        deps: Vec::with_capacity(requested_modules.len()),
        import_map: ImportMap::default(),
        exports: None,
        is_export_assign: export_assign.is_some(),
    };
    let is_common_js = format == ModuleType::CommonJs;
    if has_module_syntax && !module.is_export_assign {
        let exports = exports(cx, &mut module, is_common_js);
        module.body.push(define_es_module(cx, exports));
    }

    let (mut bindings, imports) =
        handle_import_export(cx, &mut module, requested_modules, is_common_js);
    let mut statements = Vec::with_capacity(body.len());
    for item in body {
        statements.extend(lower_body_item(
            cx,
            &mut module,
            &mut local_export_entries,
            item,
            is_common_js,
        ));
    }

    bindings.extend(local_export_entries.into_values());
    if !bindings.is_empty() && !module.is_export_assign {
        util::sort_export_bindings(cx.ast, &mut bindings);
        let exports = exports(cx, &mut module, is_common_js);
        emit_export_stmts(cx, exports, bindings, &mut module.body);
    }

    module.body.extend(imports);
    module.body.extend(statements);
    if let Some(value) = export_assign {
        if is_common_js {
            let module_ref = cx.global("module");
            let target = build::member(cx.ast, module_ref, "exports");
            let value = build::assign(cx.ast, target, value);
            module
                .body
                .push(cx.ast.stmt_expression_statement(DUMMY_SP, value));
        } else {
            module
                .body
                .push(cx.ast.stmt_return_statement(DUMMY_SP, Some(value)));
        }
    }
    module
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L296>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L415>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd.rs#L161>.
fn handle_import_export(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SyntaxStrippedModule,
    requested_modules: RequestedModules,
    is_common_js: bool,
) -> (Vec<LocalExportEntry>, Vec<Stmt>) {
    let mut bindings = Vec::new();
    let mut imports = Vec::with_capacity(requested_modules.len());

    for (request, data) in requested_modules {
        let name = util::local_name_for_src(cx.ast.get_utf8(request.specifier));
        let binding = cx.private(&name);
        let namespace = cx.resolver.symbol_of(binding.node_id());
        let needs_binding = reduce(
            cx,
            data.entries,
            &mut module.import_map,
            &mut bindings,
            namespace,
        );

        let mut value = if is_common_js {
            cx.make_require_call(data.source)
        } else {
            module.deps.push(Dependency {
                binding,
                source: data.source,
            });
            cx.reference(binding)
        };
        if data.usage.has_star_export() {
            let helper = cx.helper("export_star");
            let exports = exports(cx, module, is_common_js);
            let target = cx.reference(exports);
            value = build::call(cx.ast, helper, [value, target], false);
        }
        let interop = data.usage.interop_require_helper();
        if let Some(helper) = interop {
            let helper = cx.helper(helper);
            value = build::call(cx.ast, helper, [value], true);
        }

        if is_common_js {
            if needs_binding {
                imports.push(util::binding_decl(
                    cx.ast,
                    cx.const_var_kind(),
                    binding,
                    Some(value),
                ));
            } else {
                imports.push(cx.ast.stmt_expression_statement(DUMMY_SP, value));
            }
        } else if interop.is_some() {
            let target = cx.reference(binding);
            let value = build::assign(cx.ast, target, value);
            imports.push(cx.ast.stmt_expression_statement(DUMMY_SP, value));
        } else if data.usage.has_star_export() {
            imports.push(cx.ast.stmt_expression_statement(DUMMY_SP, value));
        }
    }

    (bindings, imports)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L427>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L517>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd.rs#L263>.
fn lower_body_item(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SyntaxStrippedModule,
    local_export_entries: &mut LocalExportEntries,
    item: SourceModuleItem,
    is_common_js: bool,
) -> Option<Stmt> {
    match item {
        SourceModuleItem::Stmt(stmt) if !stmt.is_empty_statement(cx.ast) => Some(stmt),
        SourceModuleItem::Stmt(_) => None,
        SourceModuleItem::ExportDecl(decl) => Some(Stmt::Declaration(decl)),
        SourceModuleItem::ExportDefaultDecl(export) => {
            let (stmt, name, binding) = lower_export_default_decl(cx, export)?;
            local_export_entries.insert(
                name.name,
                LocalExportEntry {
                    exported: name,
                    local: cx.resolver.symbol_of(binding.node_id()),
                },
            );
            Some(stmt)
        }
        SourceModuleItem::ExportDefaultExpr { span, expr } => Some(lower_export_default_expr(
            cx,
            local_export_entries,
            span,
            expr,
        )),
        SourceModuleItem::TsImportEquals { import, exported } => lower_ts_import_equals(
            cx,
            module,
            local_export_entries,
            import,
            exported,
            is_common_js,
        ),
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L437>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L527>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd.rs#L273>.
fn lower_ts_import_equals(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SyntaxStrippedModule,
    local_export_entries: &mut LocalExportEntries,
    import: TsImportEqualsDeclaration,
    exported: bool,
    is_common_js: bool,
) -> Option<Stmt> {
    let source = external_ts_import_equals_source(cx.ast, import).unwrap();
    let source = util::source_literal(cx.ast, source);
    let local = import.id(cx.ast);
    let value = if is_common_js {
        cx.make_require_call(source)
    } else {
        let binding = if exported {
            cx.private(cx.ast.get_utf8(local.name(cx.ast)))
        } else {
            local
        };
        module.deps.push(Dependency { binding, source });
        if !exported {
            return None;
        }
        cx.reference(binding)
    };
    if exported {
        local_export_entries.remove(&binding_name(cx.ast, local).name);
        let exports = exports(cx, module, is_common_js);
        let namespace = cx.resolver.symbol_of(exports.node_id());
        let property = local.name(cx.ast);
        module.import_map.insert(
            cx.resolver.symbol_of(local.node_id()),
            ImportBinding {
                mod_ident: namespace,
                mod_prop: Some(property),
            },
        );
        let target = cx.reference(exports);
        let target = util::named_member(cx.ast, target, property, DUMMY_SP);
        let value = build::assign(cx.ast, target, value);
        Some(cx.ast.stmt_expression_statement(DUMMY_SP, value))
    } else {
        Some(util::binding_decl(
            cx.ast,
            cx.const_var_kind(),
            local,
            Some(value),
        ))
    }
}

fn exports(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SyntaxStrippedModule,
    is_common_js: bool,
) -> BindingIdentifier {
    *module.exports.get_or_insert_with(|| {
        if is_common_js {
            cx.global_binding("exports")
        } else {
            cx.private("exports")
        }
    })
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/syntax_strip.rs#L64>.
fn lower_export_default_decl(
    cx: &mut ModuleCtx<'_, '_>,
    export: ExportDefaultDeclaration,
) -> Option<(Stmt, ExportName, BindingIdentifier)> {
    let span = export.span(cx.ast);
    let name = default_export_name(cx.ast, span);
    let decl = export.declaration(cx.ast);
    match cx.ast.export_default_declaration_kind_data(decl) {
        ExportDefaultDeclarationKindData::Class(class) => {
            let binding = class.id(cx.ast).unwrap_or_else(|| cx.private("_default"));
            class.set_id(cx.ast, Some(binding));
            class.set_class_type(cx.ast, ClassType::ClassDeclaration);
            Some((Stmt::Declaration(Decl::Class(class)), name, binding))
        }
        ExportDefaultDeclarationKindData::Function(function) => {
            let binding = function
                .id(cx.ast)
                .unwrap_or_else(|| cx.private("_default"));
            function.set_id(cx.ast, Some(binding));
            function.set_function_type(cx.ast, FunctionType::FunctionDeclaration);
            Some((Stmt::Declaration(Decl::Function(function)), name, binding))
        }
        _ => None,
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/syntax_strip.rs#L101>.
fn lower_export_default_expr(
    cx: &mut ModuleCtx<'_, '_>,
    local_export_entries: &mut LocalExportEntries,
    span: Span,
    expr: Expr,
) -> Stmt {
    let binding = cx.private("_default");
    let name = default_export_name(cx.ast, span);
    local_export_entries.insert(
        name.name,
        LocalExportEntry {
            exported: name,
            local: cx.resolver.symbol_of(binding.node_id()),
        },
    );
    util::binding_decl(cx.ast, cx.const_var_kind(), binding, Some(expr))
}
