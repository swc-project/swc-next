use std::iter;

use swc_next_ecma_ast::*;

use crate::common::build;

use super::{
    ModuleType, amd,
    module_ref_rewriter::rewrite_import_bindings,
    syntax_strip,
    util::{self, ModuleCtx},
};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L39>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L69>.
pub(super) fn common_js(cx: &mut ModuleCtx<'_, '_>) {
    let mut module = syntax_strip::lower(cx, ModuleType::CommonJs);
    rewrite_special_refs(cx, &mut module.body, None);
    rewrite_import_bindings(&mut module.body, cx, &module.import_map);

    let program = cx.ast.root_program();
    let body = cx.ast.add_sub_range(module.body);
    program.set_body(cx.ast, body);
    program.set_directives(cx.ast, module.directives);
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L159>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L261>.
pub(super) fn rewrite_special_refs(
    cx: &mut ModuleCtx<'_, '_>,
    body: &mut [Stmt],
    amd_require: Option<BindingIdentifier>,
) -> Option<BindingIdentifier> {
    let mut pass = SpecialRefRewriter {
        cx,
        amd_require,
        amd_module: None,
    };
    for stmt in body {
        if let Some(replacement) = stmt.visit_mut_with(&mut pass) {
            *stmt = replacement;
        }
    }
    pass.amd_module
}

struct SpecialRefRewriter<'a, 'b, 'ast> {
    cx: &'a mut ModuleCtx<'b, 'ast>,
    amd_require: Option<BindingIdentifier>,
    amd_module: Option<BindingIdentifier>,
}

impl<'ast> VisitMut<'ast> for SpecialRefRewriter<'_, '_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.cx.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L159>.
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd.rs#L261>.
    fn visit_mut_expr(&mut self, expr: Expr) -> Option<Expr> {
        match self.cx.ast.expr_data(expr) {
            ExprData::ImportExpression(import)
                if import.phase(self.cx.ast).is_none() && !import.sync(self.cx.ast) =>
            {
                import.visit_mut_children_with(self);
                let cx = &mut *self.cx;
                let mut source = import.source(cx.ast);
                let is_literal = source.is_string_literal(cx.ast);
                if let Some(literal) = source.as_string_literal(cx.ast) {
                    source = Expr::StringLiteral(util::source_literal(cx.ast, literal));
                }
                let span = import.span(cx.ast);
                if let Some(require) = self.amd_require {
                    let require = cx.reference(require);
                    Some(amd::amd_dynamic_import(cx, span, source, require))
                } else {
                    let args = iter::once(source).chain(import.options(cx.ast)).collect();
                    Some(cjs_dynamic_import(cx, span, args, is_literal))
                }
            }
            ExprData::MemberExpression(member) if util::meta_member(self.cx.ast, member) => {
                self.import_meta(member)
            }
            ExprData::ChainExpression(chain) => {
                let value = chain.expression(self.cx.ast);
                if let Some(member) = value.as_member_expression(self.cx.ast)
                    && util::meta_member(self.cx.ast, member)
                {
                    member.set_optional(self.cx.ast, false);
                    return Some(self.import_meta(member).unwrap_or(value));
                }
                expr.visit_mut_children_with(self)
            }
            _ => expr.visit_mut_children_with(self),
        }
    }
}

impl SpecialRefRewriter<'_, '_, '_> {
    fn import_meta(&mut self, member: MemberExpression) -> Option<Expr> {
        let name = util::member_name(self.cx.ast, member)?;
        let span = member.span(self.cx.ast);
        if let Some(require) = self.amd_require {
            let name = name.into_owned();
            return match name.as_str() {
                "resolve" | "dirname" => {
                    let object = self.cx.reference(require);
                    member.set_object(self.cx.ast, object);
                    let key = if member.computed(self.cx.ast) {
                        PropertyKey::Expr(util::string(self.cx.ast, DUMMY_SP, "toUrl"))
                    } else {
                        let key = self.cx.ast.add_utf8("toUrl");
                        self.cx.ast.property_key_identifier_name(DUMMY_SP, key)
                    };
                    member.set_property(self.cx.ast, key);
                    if name == "dirname" {
                        let directory = util::string(self.cx.ast, DUMMY_SP, ".");
                        let result = build::call(
                            self.cx.ast,
                            Expr::MemberExpression(member),
                            [directory],
                            false,
                        );
                        result.set_span(self.cx.ast, span);
                        Some(result)
                    } else {
                        None
                    }
                }
                "url" | "filename" | "main" => {
                    let module = *self
                        .amd_module
                        .get_or_insert_with(|| self.cx.private("module"));
                    Some(amd::import_meta(self.cx, span, &name, module))
                }
                _ => None,
            };
        }
        let cx = &mut *self.cx;
        let result = match name.as_ref() {
            "resolve" => {
                let require = cx.global("require");
                require.set_span(cx.ast, member.object(cx.ast).span(cx.ast));
                member.set_object(cx.ast, require);
                return None;
            }
            "url" => cjs_import_meta_url(cx),
            "filename" => cx.global("__filename"),
            "dirname" => cx.global("__dirname"),
            "main" => {
                let require = cx.global("require");
                let main = build::member(cx.ast, require, "main");
                let module = cx.global("module");
                cx.ast
                    .expr_binary_expression(span, main, module, BinaryOperator::Equal)
            }
            _ => return None,
        };
        result.set_span(cx.ast, span);
        Some(result)
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L590>.
fn cjs_dynamic_import(
    cx: &mut ModuleCtx<'_, '_>,
    span: Span,
    args: Vec<Expr>,
    is_literal: bool,
) -> Expr {
    let (resolve_args, params, require_args) = if is_literal {
        (Vec::new(), Vec::new(), args)
    } else {
        let param = cx.private("p");
        (args, vec![param], vec![cx.reference(param)])
    };

    let promise = cx.global("Promise");
    let resolve = build::member(cx.ast, promise, "resolve");
    let resolve = build::call(cx.ast, resolve, resolve_args, false);
    let then = build::member(cx.ast, resolve, "then");

    let require = cx.global("require");
    let require = build::call(cx.ast, require, require_args, false);
    let helper = cx.helper("interop_require_wildcard");
    let value = build::call(cx.ast, helper, [require], true);
    let callback = util::into_lazy_auto(cx, params, value);
    let result = build::call(cx.ast, then, [callback], false);
    result.set_span(cx.ast, span);
    result
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/common_js.rs#L633>.
fn cjs_import_meta_url(cx: &mut ModuleCtx<'_, '_>) -> Expr {
    let url = util::string(cx.ast, DUMMY_SP, "url");
    let require = cx.global("require");
    let require = build::call(cx.ast, require, [url], false);
    let path_to_file_url = build::member(cx.ast, require, "pathToFileURL");
    let filename = cx.global("__filename");
    let url = build::call(cx.ast, path_to_file_url, [filename], false);
    let to_string = build::member(cx.ast, url, "toString");
    build::call(cx.ast, to_string, [], false)
}
