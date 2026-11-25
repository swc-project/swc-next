use swc_next_ecma_ast::*;

use crate::common::build;

use super::{
    module_record::SourceModule,
    util::{self, ModuleCtx},
};

mod emit;
mod ir;
mod lower;
mod pattern;
mod rewrite;

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js.rs#L19>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js.rs#L36>.
pub(super) fn system_js(cx: &mut ModuleCtx<'_, '_>) {
    let source = SourceModule::collect(cx.ast, cx.resolver);
    let program = cx.ast.root_program();
    let directives = program.directives(cx.ast);
    let directives = util::use_strict(cx.ast, directives);
    let export_ident = cx.private("_export");
    let context_ident = cx.private("_context");
    let mut module = lower::lower(cx, source);
    rewrite::rewrite_special_refs(cx, &mut module, context_ident);
    rewrite::rewrite_export_bindings(cx, &mut module, export_ident);
    module.async_execute = contains_top_level_await(cx.ast, &module.execute_stmts);
    let args = emit::emit(cx, module, export_ident, context_ident, directives);
    let system = cx.global("System");
    let register = build::member(cx.ast, system, "register");
    let call = build::call(cx.ast, register, args, false);
    util::emit_wrapper(cx.ast, call);
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L2437>.
fn contains_top_level_await(ast: &Ast<'_>, execute_stmts: &[ir::ExecuteStmt]) -> bool {
    let mut finder = TopLevelAwait { ast, found: false };
    execute_stmts.iter().any(|stmt| {
        stmt.visit_with(&mut finder);
        finder.found
    })
}

struct TopLevelAwait<'a, 'ast> {
    ast: &'a Ast<'ast>,
    found: bool,
}

impl<'ast> Visit<'ast> for TopLevelAwait<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_stmt(&mut self, stmt: Stmt) {
        if !self.found {
            stmt.visit_children_with(self);
        }
    }

    fn visit_function(&mut self, _: Function) {}
    fn visit_arrow_function_expression(&mut self, _: ArrowFunctionExpression) {}

    fn visit_class_element(&mut self, element: ClassElement) {
        match self.ast.class_element_data(element) {
            ClassElementData::MethodDefinition(method) if method.computed(self.ast) => {
                method.key(self.ast).visit_with(self)
            }
            ClassElementData::PropertyDefinition(field) if field.computed(self.ast) => {
                field.key(self.ast).visit_with(self)
            }
            _ => {}
        }
    }

    fn visit_for_of_statement(&mut self, stmt: ForOfStatement) {
        if stmt.r#await(self.ast) {
            self.found = true;
        } else {
            stmt.visit_children_with(self);
        }
    }

    fn visit_await_expression(&mut self, _: AwaitExpression) {
        self.found = true;
    }
}
