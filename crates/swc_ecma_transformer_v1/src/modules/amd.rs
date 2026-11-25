use std::sync::LazyLock;

use swc_next_ecma_ast::*;

use crate::common::build;

use super::{
    ModuleType, common_js,
    module_ref_rewriter::rewrite_import_bindings,
    syntax_strip,
    util::{self, ModuleCtx},
};

mod emit;

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L58>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L126>.
pub(super) fn amd(cx: &mut ModuleCtx<'_, '_>) {
    let module_id = get_amd_module_id_from_comments(cx.ast);
    // Source references to the ambient require use the AMD loader parameter.
    let require = cx.global_binding("require");
    let mut module = syntax_strip::lower(cx, ModuleType::Amd);
    let module_param = common_js::rewrite_special_refs(cx, &mut module.body, Some(require));
    rewrite_import_bindings(&mut module.body, cx, &module.import_map);

    emit::emit(cx, module, module_id, require, module_param);
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L587>.
fn get_amd_module_id_from_comments<'a>(ast: &Ast<'a>) -> Option<&'a str> {
    static AMD_MODULE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"(?i)^/\s*<amd-module.*?name\s*=\s*(?:(?:'([^']*)')|(?:"([^"]*)")).*?/>"#,
        )
        .unwrap()
    });
    let start = util::source_start(ast);
    ast.comments()
        .iter()
        .rev()
        .filter(|comment| comment.kind == CommentKind::Line && comment.span.end <= start)
        .find_map(|comment| {
            let text = ast.get_utf8(comment.value(ast.source().as_bytes()));
            let captures = AMD_MODULE.captures(text)?;
            captures
                .get(1)
                .or_else(|| captures.get(2))
                .map(|name| name.as_str())
        })
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L613>.
pub(super) fn amd_dynamic_import(
    cx: &mut ModuleCtx<'_, '_>,
    span: Span,
    source: Expr,
    require: Expr,
) -> Expr {
    let resolve = cx.private("resolve");
    let reject = cx.private("reject");
    let module = cx.private("m");

    let value = cx.reference(module);
    let helper = cx.helper("interop_require_wildcard");
    let value = build::call(cx.ast, helper, [value], true);
    let resolve_ref = cx.reference(resolve);
    let value = build::call(cx.ast, resolve_ref, [value], false);
    let callback = util::into_lazy_auto(cx, [module], value);

    let deps = util::array(cx.ast, [source]);
    let reject_ref = cx.reference(reject);
    let value = build::call(cx.ast, require, [deps, callback, reject_ref], false);
    let executor = util::into_lazy_auto(cx, [resolve, reject], value);
    let promise = cx.global("Promise");
    let args = cx.ast.add_sub_range([Argument::Expr(executor)]);
    cx.ast.expr_new_expression(span, promise, None, args)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L261>.
pub(super) fn import_meta(
    cx: &mut ModuleCtx<'_, '_>,
    span: Span,
    name: &str,
    module: BindingIdentifier,
) -> Expr {
    let module = cx.reference(module);
    let result = match name {
        "url" => amd_import_meta_url(cx, module),
        "filename" => amd_import_meta_filename(cx, module),
        "main" => {
            let id = build::member(cx.ast, module, "id");
            let main = util::string(cx.ast, DUMMY_SP, "main");
            cx.ast
                .expr_binary_expression(span, id, main, BinaryOperator::Equal)
        }
        _ => unreachable!(),
    };
    result.set_span(cx.ast, span);
    result
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L666>.
fn amd_import_meta_url(cx: &mut ModuleCtx<'_, '_>, module: Expr) -> Expr {
    let uri = build::member(cx.ast, module, "uri");
    let document = cx.global("document");
    let base = build::member(cx.ast, document, "baseURI");
    let url = cx.global("URL");
    let args = cx
        .ast
        .add_sub_range([Argument::Expr(uri), Argument::Expr(base)]);
    let url = cx.ast.expr_new_expression(DUMMY_SP, url, None, args);
    build::member(cx.ast, url, "href")
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L684>.
fn amd_import_meta_filename(cx: &mut ModuleCtx<'_, '_>, module: Expr) -> Expr {
    let uri = build::member(cx.ast, module, "uri");
    let split = build::member(cx.ast, uri, "split");
    let slash = util::string(cx.ast, DUMMY_SP, "/");
    let parts = build::call(cx.ast, split, [slash], false);
    let pop = build::member(cx.ast, parts, "pop");
    build::call(cx.ast, pop, [], false)
}
