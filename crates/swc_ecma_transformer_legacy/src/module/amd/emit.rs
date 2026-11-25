use swc_next_ecma_ast::*;

use crate::common::build;

use super::super::{
    syntax_strip::SyntaxStrippedModule,
    util::{self, ModuleCtx},
};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/amd/emit.rs#L17>.
pub(super) fn emit(
    cx: &mut ModuleCtx<'_, '_>,
    module: SyntaxStrippedModule,
    module_id: Option<&str>,
    require: BindingIdentifier,
    module_param: Option<BindingIdentifier>,
) {
    let mut names = vec![util::string(cx.ast, DUMMY_SP, "require")];
    let mut params = vec![require];

    if let Some(exports) = module.exports {
        names.push(util::string(cx.ast, DUMMY_SP, "exports"));
        params.push(exports);
    }
    if let Some(module) = module_param {
        names.push(util::string(cx.ast, DUMMY_SP, "module"));
        params.push(module);
    }
    for dep in module.deps {
        names.push(Expr::StringLiteral(dep.source));
        params.push(dep.binding);
    }

    let mut args = Vec::with_capacity(3);
    if let Some(id) = module_id {
        args.push(util::string(cx.ast, DUMMY_SP, id));
    }
    args.push(util::array(cx.ast, names));
    let factory = util::function(cx.ast, None, params, module.directives, module.body, false);
    args.push(Expr::Function(factory));

    let define = cx.global("define");
    let expression = build::call(cx.ast, define, args, false);
    util::emit_wrapper(cx.ast, expression);
}
