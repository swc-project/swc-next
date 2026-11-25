use std::path::Path;

use convert_case::{Case, Casing};
use swc_next_ecma_ast::*;

use crate::common::build;

use super::super::{
    syntax_strip::SyntaxStrippedModule,
    util::{self, ModuleCtx},
};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd/emit.rs#L17>.
pub(super) fn emit(cx: &mut ModuleCtx<'_, '_>, module: SyntaxStrippedModule, filename: &str) {
    let (adapter, params) = emit_adapter(cx, &module, filename);
    let factory = util::function(cx.ast, None, params, module.directives, module.body, false);
    let this = cx.ast.expr_this_expression(DUMMY_SP);
    let expression = build::call(cx.ast, adapter, [this, Expr::Function(factory)], false);
    util::emit_wrapper(cx.ast, expression);
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd/emit.rs#L83>.
fn emit_adapter(
    cx: &mut ModuleCtx<'_, '_>,
    module: &SyntaxStrippedModule,
    filename: &str,
) -> (Expr, Vec<BindingIdentifier>) {
    let global = cx.private("global");
    let factory = cx.private("factory");
    let global_export_name = cx.ast.add_utf8(&determine_export_name(filename));
    let mut cjs_args = Vec::new();
    let mut amd_deps = Vec::new();
    let mut browser_args = Vec::new();
    let mut params = Vec::new();

    if !module.is_export_assign
        && let Some(exports) = module.exports
    {
        // SWC emits the ambient exports in the adapter with an empty context.
        let name = cx.ast.add_utf8("exports");
        cjs_args.push(cx.ast.expr_identifier_reference(DUMMY_SP, name));
        amd_deps.push(util::string(cx.ast, DUMMY_SP, "exports"));
        let target = module_global_export_member(cx, global, global_export_name);
        let object = util::object(cx.ast, []);
        browser_args.push(build::assign(cx.ast, target, object));
        params.push(exports);
    }

    for dep in &module.deps {
        cjs_args.push(cx.make_require_call(dep.source));
        amd_deps.push(Expr::StringLiteral(util::source_literal(
            cx.ast, dep.source,
        )));
        let name = cx.ast.get_wtf8(dep.source.value(cx.ast)).to_string_lossy();
        let name = cx.ast.add_utf8(&global_name(&name));
        let global = cx.reference(global);
        browser_args.push(util::named_member(cx.ast, global, name, DUMMY_SP));
        params.push(dep.binding);
    }

    let module_ref = cx.global("module");
    let module_type = typeof_is(cx.ast, module_ref, "object", BinaryOperator::StrictEqual);
    let module_ref = cx.global("module");
    let exports = build::member(cx.ast, module_ref, "exports");
    let exports_type = typeof_is(cx.ast, exports, "object", BinaryOperator::StrictEqual);
    let cjs_test =
        cx.ast
            .expr_logical_expression(DUMMY_SP, module_type, exports_type, LogicalOperator::And);
    let factory_ref = cx.reference(factory);
    let mut cjs_body = build::call(cx.ast, factory_ref, cjs_args, false);
    if module.is_export_assign {
        let module_ref = cx.global("module");
        let exports = build::member(cx.ast, module_ref, "exports");
        cjs_body = build::assign(cx.ast, exports, cjs_body);
    }

    let define = cx.global("define");
    let define_type = typeof_is(cx.ast, define, "function", BinaryOperator::StrictEqual);
    let define = cx.global("define");
    let amd = build::member(cx.ast, define, "amd");
    let amd_test = cx
        .ast
        .expr_logical_expression(DUMMY_SP, define_type, amd, LogicalOperator::And);
    let define = cx.global("define");
    let deps = util::array(cx.ast, amd_deps);
    let factory_ref = cx.reference(factory);
    let amd_body = build::call(cx.ast, define, [deps, factory_ref], false);

    let global_this = cx.global("globalThis");
    let global_test = typeof_is(
        cx.ast,
        global_this,
        "undefined",
        BinaryOperator::StrictNotEqual,
    );
    let global_this = cx.global("globalThis");
    let global_ref = cx.reference(global);
    let self_ref = cx.global("self");
    let fallback =
        cx.ast
            .expr_logical_expression(DUMMY_SP, global_ref, self_ref, LogicalOperator::Or);
    let global_value =
        cx.ast
            .expr_conditional_expression(DUMMY_SP, global_test, global_this, fallback);
    let global_ref = cx.reference(global);
    let browser_test = build::assign(cx.ast, global_ref, global_value);
    let factory_ref = cx.reference(factory);
    let mut browser_body = build::call(cx.ast, factory_ref, browser_args, false);
    if module.is_export_assign {
        let target = module_global_export_member(cx, global, global_export_name);
        browser_body = build::assign(cx.ast, target, browser_body);
    }

    let browser_if = if_stmt(cx.ast, browser_test, browser_body, None);
    let amd_if = if_stmt(cx.ast, amd_test, amd_body, Some(browser_if));
    let cjs_if = if_stmt(cx.ast, cjs_test, cjs_body, Some(amd_if));
    let adapter = util::function(
        cx.ast,
        None,
        [global, factory],
        SubRange::empty(),
        vec![cjs_if],
        false,
    );
    (Expr::Function(adapter), params)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd/emit.rs#L239>.
fn if_stmt(ast: &mut Ast<'_>, test: Expr, body: Expr, alt: Option<Stmt>) -> Stmt {
    let body = ast.stmt_expression_statement(DUMMY_SP, body);
    ast.stmt_if_statement(DUMMY_SP, test, body, alt)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd/emit.rs#L90>.
fn typeof_is(ast: &mut Ast<'_>, value: Expr, type_name: &str, operator: BinaryOperator) -> Expr {
    let value = ast.expr_unary_expression(DUMMY_SP, value, UnaryOperator::Typeof);
    let type_name = util::string(ast, DUMMY_SP, type_name);
    ast.expr_binary_expression(DUMMY_SP, value, type_name, operator)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd/emit.rs#L250>.
fn module_global_export_member(
    cx: &mut ModuleCtx<'_, '_>,
    global: BindingIdentifier,
    global_export_name: AstUtf8,
) -> Expr {
    let global = cx.reference(global);
    util::named_member(cx.ast, global, global_export_name, DUMMY_SP)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd/config.rs#L73>.
fn determine_export_name(filename: &str) -> String {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(filename);
    global_name(stem)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd/config.rs#L65>.
fn global_name(source: &str) -> String {
    // Module names can contain separators such as dots that convert_case keeps.
    source
        .rsplit('/')
        .next()
        .unwrap()
        .replace(|c: char| !c.is_alphanumeric(), " ")
        .remove_empty()
        .to_case(Case::Camel)
}

#[cfg(test)]
mod tests {
    use super::global_name;

    #[test]
    fn global_names() {
        assert_eq!(global_name("./widget.js"), "widgetJs");

        assert_eq!(global_name("@scope/widget-tools"), "widgetTools");

        assert_eq!(
            global_name("@swc/helpers/_/_interop_require_default"),
            "interopRequireDefault"
        );
    }
}
