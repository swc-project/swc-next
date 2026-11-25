use swc_next_ecma_ast::*;

use crate::common::build;

use super::super::util::ModuleCtx;
use super::{
    ir::{LocalExports, SystemModule},
    lower::export_names_call,
    pattern::replace_exported_pat,
};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L15>.
pub(super) fn rewrite_special_refs(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SystemModule,
    context_ident: BindingIdentifier,
) {
    let mut visitor = SpecialRefRewriter { cx, context_ident };
    for function in &mut module.wrapper_fns {
        function.visit_mut_with(&mut visitor);
    }
    for init in &mut module.export_inits {
        if let Some(replacement) = init.value.visit_mut_with(&mut visitor) {
            init.value = replacement;
        }
    }
    for stmt in &mut module.execute_stmts {
        stmt.visit_mut_with(&mut visitor);
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L56>.
struct SpecialRefRewriter<'a, 'b, 'ast> {
    cx: &'a mut ModuleCtx<'b, 'ast>,
    context_ident: BindingIdentifier,
}

impl SpecialRefRewriter<'_, '_, '_> {
    fn context_member(&mut self, key: &str) -> Expr {
        let context_ident = self.cx.reference(self.context_ident);
        build::member(self.cx.ast, context_ident, key)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L128>.
    fn module_name_expr(&mut self, span: Span) -> Expr {
        let test = self.context_member("meta");
        let meta = self.context_member("meta");
        let url = build::member(self.cx.ast, meta, "url");
        let id = self.context_member("id");
        self.cx.ast.expr_conditional_expression(span, test, url, id)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L124>.
    fn is_unresolved_module_name(&self, expr: Expr) -> bool {
        expr.as_identifier_reference(self.cx.ast).is_some_and(|id| {
            self.cx.ast.get_utf8(id.name(self.cx.ast)) == "__moduleName"
                && self.cx.resolver.is_unresolved(id.node_id())
        })
    }
}

impl<'ast> VisitMut<'ast> for SpecialRefRewriter<'_, '_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.cx.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L67>.
    fn visit_mut_expr(&mut self, expr: Expr) -> Option<Expr> {
        let span = expr.span(self.cx.ast);
        match self.cx.ast.expr_data(expr) {
            ExprData::IdentifierReference(_) if self.is_unresolved_module_name(expr) => {
                Some(self.module_name_expr(span))
            }
            ExprData::MetaProperty(meta)
                if self
                    .cx
                    .ast
                    .get_utf8(meta.meta(self.cx.ast).name(self.cx.ast))
                    == "import" =>
            {
                Some(self.context_member("meta"))
            }
            ExprData::ImportExpression(import)
                if import.phase(self.cx.ast).is_none() && !import.sync(self.cx.ast) =>
            {
                import.visit_mut_children_with(self);
                let source = import.source(self.cx.ast);
                let args: Vec<_> = std::iter::once(source)
                    .chain(import.options(self.cx.ast))
                    .collect();
                let callee = self.context_member("import");
                let call = build::call(self.cx.ast, callee, args, false);
                call.set_span(self.cx.ast, span);
                Some(call)
            }
            _ => expr.visit_mut_children_with(self),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L107>.
    fn visit_mut_object_property(&mut self, property: ObjectProperty) -> Option<ObjectProperty> {
        if property.shorthand(self.cx.ast)
            && self.is_unresolved_module_name(property.value(self.cx.ast))
        {
            property.set_shorthand(self.cx.ast, false);
        }
        property.visit_mut_children_with(self)
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L41>.
pub(super) fn rewrite_export_bindings(
    cx: &mut ModuleCtx<'_, '_>,
    module: &mut SystemModule,
    export_ident: BindingIdentifier,
) {
    let mut visitor = ExportBindingRewriter {
        cx,
        exports: &module.exports.local_exports,
        export_ident,
        export_setters: module.export_setters,
        needs_export_setters: &mut module.needs_export_setters,
        old_temp: None,
    };
    for function in &mut module.wrapper_fns {
        function.visit_mut_with(&mut visitor);
    }
    for init in &mut module.export_inits {
        if let Some(replacement) = init.value.visit_mut_with(&mut visitor) {
            init.value = replacement;
        }
    }
    for stmt in &mut module.execute_stmts {
        stmt.visit_mut_with(&mut visitor);
    }
    module.wrapper_vars.extend(visitor.old_temp);
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L141>.
struct ExportBindingRewriter<'a, 'b, 'ast> {
    cx: &'a mut ModuleCtx<'b, 'ast>,
    exports: &'a LocalExports,
    export_ident: BindingIdentifier,
    export_setters: BindingIdentifier,
    needs_export_setters: &'a mut bool,
    old_temp: Option<BindingIdentifier>,
}

impl ExportBindingRewriter<'_, '_, '_> {
    fn exported_symbol(&self, id: IdentifierReference) -> Option<SymbolId> {
        self.cx
            .resolver
            .symbol(id.node_id())
            .filter(|symbol| self.exports.contains_key(symbol))
    }

    fn replace_pattern(&mut self, target: AssignmentTarget) -> AssignmentTarget {
        replace_exported_pat(
            self.cx.ast,
            self.cx.resolver,
            self.exports,
            self.export_setters,
            target,
            self.needs_export_setters,
        )
    }

    fn assignment(&mut self, expr: Expr, assign: AssignmentExpression) -> Option<Expr> {
        let left = assign.left(self.cx.ast);
        if let Some(id) = left
            .as_simple_assignment_target(self.cx.ast)
            .and_then(|target| target.as_identifier_reference(self.cx.ast))
        {
            assign.visit_mut_children_with(self);
            let symbol = self.exported_symbol(id)?;
            let value = self.cx.resolver.symbol_reference(self.cx.ast, symbol);
            let export =
                export_names_call(self.cx, self.export_ident, &self.exports[&symbol], symbol);
            let result = build::sequence(self.cx.ast, vec![expr, export, value]);
            result.set_span(self.cx.ast, expr.span(self.cx.ast));
            return Some(result);
        }
        if !left.is_simple_assignment_target(self.cx.ast) {
            let replacement = self.replace_pattern(left);
            assign.set_left(self.cx.ast, replacement);
        }
        assign.visit_mut_children_with(self);
        None
    }

    fn update(&mut self, expr: Expr, update: UpdateExpression) -> Option<Expr> {
        let target = update.argument(self.cx.ast);
        let Some(id) = target.as_identifier_reference(self.cx.ast) else {
            update.visit_mut_children_with(self);
            return None;
        };
        let symbol = self.exported_symbol(id)?;
        let export = export_names_call(self.cx, self.export_ident, &self.exports[&symbol], symbol);
        let (update, value) = if update.prefix(self.cx.ast) {
            (expr, self.cx.resolver.symbol_reference(self.cx.ast, symbol))
        } else {
            let old_temp = *self.old_temp.get_or_insert_with(|| self.cx.private("_old"));
            let target = self.cx.reference(old_temp);
            let update = build::assign(self.cx.ast, target, expr);
            (update, self.cx.reference(old_temp))
        };
        let result = build::sequence(self.cx.ast, vec![update, export, value]);
        result.set_span(self.cx.ast, expr.span(self.cx.ast));
        Some(result)
    }
}

impl<'ast> VisitMut<'ast> for ExportBindingRewriter<'_, '_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.cx.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L182>.
    fn visit_mut_expr(&mut self, expr: Expr) -> Option<Expr> {
        match self.cx.ast.expr_data(expr) {
            ExprData::AssignmentExpression(assign) => self.assignment(expr, assign),
            ExprData::UpdateExpression(update) => self.update(expr, update),
            _ => expr.visit_mut_children_with(self),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/system_js/rewrite.rs#L284>.
    fn visit_mut_for_statement_left(&mut self, left: ForStatementLeft) -> Option<ForStatementLeft> {
        if let Some(target) = left.as_assignment_target(self.cx.ast) {
            let target = self.replace_pattern(target);
            let target = target.visit_mut_with(self).unwrap_or(target);
            return Some(ForStatementLeft::AssignmentTarget(target));
        }
        left.visit_mut_children_with(self)
    }
}
