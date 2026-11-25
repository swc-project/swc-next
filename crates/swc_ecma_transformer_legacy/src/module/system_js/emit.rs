use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::*;

use crate::common::build;

use super::super::{
    module_record::ExportName,
    util::{self, ModuleCtx},
};
use super::{
    ir::{DependencySlot, ExecuteStmt, ExportInit, SetterOp, SystemModule},
    lower::{export_call, export_names_call},
};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L30>.
pub(super) fn emit(
    cx: &mut ModuleCtx<'_, '_>,
    mut module: SystemModule,
    export: BindingIdentifier,
    context: BindingIdentifier,
    directives: SubRange<Directive>,
) -> Vec<Expr> {
    let sources = emit_dependency_array(cx.ast, &module.dependencies);
    let setters = emit_setters(cx, &module, export);
    let metadata = emit_metadata_array(cx, &module.dependencies);
    let export_inits = emit_export_inits(cx, &module, export);

    let mut body = Vec::new();
    emit_var_decl(cx, &mut body, std::mem::take(&mut module.wrapper_vars));
    emit_export_setters(cx, &mut body, &module, export);
    body.extend(
        module
            .wrapper_fns
            .iter()
            .map(|function| Stmt::Declaration(Decl::Function(*function))),
    );
    body.extend(export_inits);

    let execute = std::mem::take(&mut module.execute_stmts)
        .into_iter()
        .map(|stmt| emit_execute_stmt(cx, &module, stmt, export))
        .collect();
    let execute = util::function(
        cx.ast,
        None,
        [],
        SubRange::empty(),
        execute,
        module.async_execute,
    );
    let setters = util::property(cx.ast, "setters", setters, PropertyKind::Init);
    let execute = util::property(
        cx.ast,
        "execute",
        Expr::Function(execute),
        PropertyKind::Init,
    );
    let register_body = util::object(cx.ast, [setters, execute]);
    body.push(cx.ast.stmt_return_statement(DUMMY_SP, Some(register_body)));
    let register = util::function(cx.ast, None, [export, context], directives, body, false);

    let mut args = vec![sources, Expr::Function(register)];
    args.extend(metadata);
    args
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L112>.
fn emit_execute_stmt(
    cx: &mut ModuleCtx<'_, '_>,
    module: &SystemModule,
    stmt: ExecuteStmt,
    export: BindingIdentifier,
) -> Stmt {
    let expression = match stmt {
        ExecuteStmt::Source(stmt) => return stmt,
        ExecuteStmt::Assign { left, right } => cx.ast.expr_assignment_expression(
            right.span(cx.ast),
            left,
            right,
            AssignmentOperator::Assign,
        ),
        ExecuteStmt::ExportAssign { local, right } => {
            let symbol = cx.resolver.symbol_of(local.node_id());
            let target = cx.reference(local);
            let assign = build::assign(cx.ast, target, right);
            export_call(cx, export, &module.exports.local_exports[&symbol], assign)
        }
        ExecuteStmt::ExportValue {
            export: name,
            value,
        } => export_call(cx, export, &[name], value),
        ExecuteStmt::ExportBatch { value } => {
            let callee = cx.reference(export);
            build::call(cx.ast, callee, [value], false)
        }
    };
    cx.ast.stmt_expression_statement(DUMMY_SP, expression)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L149>.
fn emit_dependency_array(ast: &mut Ast<'_>, deps: &[DependencySlot]) -> Expr {
    util::array(ast, deps.iter().map(|dep| Expr::StringLiteral(dep.request)))
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L221>.
fn emit_setters(
    cx: &mut ModuleCtx<'_, '_>,
    module: &SystemModule,
    export_ident: BindingIdentifier,
) -> Expr {
    let setters = module
        .dependencies
        .iter()
        .map(|dep| emit_setter(cx, module, dep, export_ident))
        .collect::<Vec<_>>();
    util::array(cx.ast, setters)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L170>.
fn emit_metadata_array(cx: &mut ModuleCtx<'_, '_>, deps: &[DependencySlot]) -> Option<Expr> {
    if !deps.iter().any(|dep| !dep.attributes.is_empty()) {
        return None;
    }
    let elements = deps
        .iter()
        .map(|dep| {
            if dep.attributes.is_empty() {
                return util::object(cx.ast, []);
            }
            let attributes = dep
                .attributes
                .iter()
                .map(|attribute| {
                    let value = Expr::StringLiteral(cx.ast.string_literal(
                        DUMMY_SP,
                        attribute.value,
                        AstUtf8::EMPTY,
                    ));
                    let key = cx.ast.get_utf8(attribute.key);
                    object_lit_prop_name(cx.ast, key, DUMMY_SP, value)
                })
                .collect::<Vec<_>>();
            let object = util::object(cx.ast, attributes);
            let assert = util::property(cx.ast, "assert", object, PropertyKind::Init);
            util::object(cx.ast, [assert])
        })
        .collect::<Vec<_>>();
    Some(util::array(cx.ast, elements))
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L232>.
fn emit_setter(
    cx: &mut ModuleCtx<'_, '_>,
    module: &SystemModule,
    dep: &DependencySlot,
    export: BindingIdentifier,
) -> Expr {
    let mut stmts = Vec::new();
    let mut direct = Vec::new();
    let mut has_star = false;

    for entry in &dep.setter_ops {
        match *entry {
            SetterOp::Import { local, imported } => {
                let value = dependency_value(cx, dep.namespace, imported);
                let target = cx.reference(local);
                let assignment = build::assign(cx.ast, target, value);
                stmts.push(cx.ast.stmt_expression_statement(DUMMY_SP, assignment));
                let symbol = cx.resolver.symbol_of(local.node_id());
                if let Some(exports) = module.exports.local_exports.get(&symbol) {
                    let value = export_names_call(cx, export, exports, symbol);
                    stmts.push(cx.ast.stmt_expression_statement(DUMMY_SP, value));
                }
            }
            SetterOp::ReExport { imported, export } => direct.push((export, Some(imported))),
            SetterOp::ReExportNamespace { export } => direct.push((export, None)),
            SetterOp::StarExport => has_star = true,
        }
    }

    if has_star {
        let export_obj = cx.private("exportObj");
        let null = cx.ast.expr_null_literal(DUMMY_SP);
        let proto = util::property(cx.ast, "__proto__", null, PropertyKind::Init);
        let object = util::object(cx.ast, [proto]);
        stmts.push(util::binding_decl(
            cx.ast,
            VariableKind::Var,
            export_obj,
            Some(object),
        ));
        stmts.push(emit_star_loop(cx, export_obj, dep.namespace));
        for (exported, imported) in direct {
            let value = dependency_value(cx, dep.namespace, imported);
            let target = cx.reference(export_obj);
            let target = util::named_member(cx.ast, target, exported.name, exported.span);
            let value = build::assign(cx.ast, target, value);
            stmts.push(cx.ast.stmt_expression_statement(DUMMY_SP, value));
        }
        let callback = cx.reference(export);
        let object = cx.reference(export_obj);
        let value = build::call(cx.ast, callback, [object], false);
        stmts.push(cx.ast.stmt_expression_statement(DUMMY_SP, value));
    } else if !direct.is_empty() {
        let values = direct
            .into_iter()
            .map(|(name, imported)| (name, dependency_value(cx, dep.namespace, imported)))
            .collect();
        let value = export_values_call(cx, export, values);
        stmts.push(cx.ast.stmt_expression_statement(DUMMY_SP, value));
    }

    let params = (!dep.setter_ops.is_empty()).then_some(dep.namespace);
    Expr::Function(util::function(
        cx.ast,
        None,
        params,
        SubRange::empty(),
        stmts,
        false,
    ))
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/util.rs#L37>.
fn dependency_value(
    cx: &mut ModuleCtx<'_, '_>,
    namespace: BindingIdentifier,
    imported: Option<AstUtf8>,
) -> Expr {
    let namespace = cx.reference(namespace);
    if let Some(name) = imported {
        util::named_member(cx.ast, namespace, name, DUMMY_SP)
    } else {
        namespace
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L315>.
fn emit_star_loop(
    cx: &mut ModuleCtx<'_, '_>,
    export_obj: BindingIdentifier,
    namespace: BindingIdentifier,
) -> Stmt {
    let key = cx.private("key");
    let key_ref = cx.reference(key);
    let default = util::string(cx.ast, DUMMY_SP, "default");
    let default_test =
        cx.ast
            .expr_binary_expression(DUMMY_SP, key_ref, default, BinaryOperator::StrictNotEqual);
    let key_ref = cx.reference(key);
    let es_module = util::string(cx.ast, DUMMY_SP, "__esModule");
    let es_module_test =
        cx.ast
            .expr_binary_expression(DUMMY_SP, key_ref, es_module, BinaryOperator::StrictNotEqual);
    let test = cx.ast.expr_logical_expression(
        DUMMY_SP,
        default_test,
        es_module_test,
        LogicalOperator::And,
    );

    let object = cx.reference(export_obj);
    let key_ref = cx.reference(key);
    let target = build::computed(cx.ast, object, key_ref);
    let object = cx.reference(namespace);
    let key_ref = cx.reference(key);
    let value = build::computed(cx.ast, object, key_ref);
    let value = build::assign(cx.ast, target, value);
    let body = cx.ast.stmt_expression_statement(DUMMY_SP, value);
    let body = cx.ast.stmt_if_statement(DUMMY_SP, test, body, None);

    let var = build::variable(cx.ast, VariableKind::Var, key, None, DUMMY_SP);
    let left = ForStatementLeft::VariableDeclaration(var.as_variable_declaration(cx.ast).unwrap());
    let right = cx.reference(namespace);
    cx.ast.stmt_for_in_statement(DUMMY_SP, left, right, body)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L354>.
fn emit_var_decl(
    cx: &mut ModuleCtx<'_, '_>,
    stmts: &mut Vec<Stmt>,
    mut vars: Vec<BindingIdentifier>,
) {
    vars.sort_unstable_by_key(|binding| cx.ast.get_utf8(binding.name(cx.ast)));
    let mut seen = FxHashSet::default();
    let declarations: Vec<_> = vars
        .into_iter()
        .filter(|binding| seen.insert(cx.resolver.symbol_of(binding.node_id())))
        .map(|binding| {
            cx.ast.variable_declarator(
                binding.span(cx.ast),
                BindingPattern::BindingIdentifier(binding),
                None,
                None,
                false,
            )
        })
        .collect();
    if !declarations.is_empty() {
        stmts.push(build::declaration(cx.ast, VariableKind::Var, declarations));
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L380>.
fn emit_export_setters(
    cx: &mut ModuleCtx<'_, '_>,
    body: &mut Vec<Stmt>,
    module: &SystemModule,
    export: BindingIdentifier,
) {
    if !module.needs_export_setters {
        return;
    }

    let properties = module
        .exports
        .local_exports
        .iter()
        .map(|(&symbol, exports)| {
            let value = cx.private("_value");
            let target = cx.resolver.symbol_reference(cx.ast, symbol);
            let value_ref = cx.reference(value);
            let assignment = build::assign(cx.ast, target, value_ref);
            let assignment = cx.ast.stmt_expression_statement(DUMMY_SP, assignment);
            let update = export_names_call(cx, export, exports, symbol);
            let update = cx.ast.stmt_expression_statement(DUMMY_SP, update);
            let setter = util::function(
                cx.ast,
                None,
                [value],
                SubRange::empty(),
                vec![assignment, update],
                false,
            );
            let key = cx.ast.get_utf8(cx.resolver.symbol_name(symbol));
            util::property(cx.ast, key, Expr::Function(setter), PropertyKind::Set)
        })
        .collect::<Vec<_>>();
    let setters = util::object(cx.ast, properties);
    body.push(util::binding_decl(
        cx.ast,
        VariableKind::Var,
        module.export_setters,
        Some(setters),
    ));
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L431>.
fn emit_export_inits(
    cx: &mut ModuleCtx<'_, '_>,
    module: &SystemModule,
    export: BindingIdentifier,
) -> Vec<Stmt> {
    let mut initial_values = FxHashMap::default();
    for function in &module.wrapper_fns {
        let Some(binding) = function.id(cx.ast) else {
            continue;
        };
        let symbol = cx.resolver.symbol_of(binding.node_id());
        if let Some(exports) = module.exports.local_exports.get(&symbol) {
            initial_values.extend(
                exports
                    .iter()
                    .map(|name| (name.name, cx.reference(binding))),
            );
        }
    }
    for &ExportInit { export, value } in &module.export_inits {
        initial_values.insert(export.name, value);
    }

    let mut seen = FxHashSet::default();
    let values: Vec<_> = module
        .exports
        .announced
        .iter()
        .copied()
        .filter(|name| seen.insert(name.name))
        .map(|name| {
            let value = initial_values
                .remove(&name.name)
                .unwrap_or_else(|| build::undefined(cx.ast));
            (name, value)
        })
        .collect();
    if values.is_empty() {
        return Vec::new();
    }
    let expression = export_values_call(cx, export, values);
    vec![cx.ast.stmt_expression_statement(DUMMY_SP, expression)]
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/emit.rs#L481>.
pub(super) fn export_values_call(
    cx: &mut ModuleCtx<'_, '_>,
    export: BindingIdentifier,
    mut exports: Vec<(ExportName, Expr)>,
) -> Expr {
    let callback = cx.reference(export);
    if exports.len() == 1 {
        let (export, value) = exports.pop().unwrap();
        let name = util::string(cx.ast, export.span, cx.ast.get_utf8(export.name));
        return build::call(cx.ast, callback, [name, value], false);
    }

    let props = exports
        .into_iter()
        .map(|(export, value)| {
            let name = cx.ast.get_utf8(export.name);
            object_lit_prop_name(cx.ast, name, export.span, value)
        })
        .collect::<Vec<_>>();
    let object = util::object(cx.ast, props);
    build::call(cx.ast, callback, [object], false)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/util.rs#L49>.
fn object_lit_prop_name(
    ast: &mut Ast<'_>,
    name: &str,
    span: Span,
    value: Expr,
) -> ObjectPropertyKind {
    // A noncomputed __proto__ key would change the object prototype.
    let computed = name == "__proto__";
    let key = if computed {
        PropertyKey::Expr(util::string(ast, span, name))
    } else {
        util::prop_name(ast, name, span)
    };
    ast.object_property_kind_object_property(
        DUMMY_SP,
        key,
        value,
        PropertyKind::Init,
        false,
        false,
        computed,
    )
}
