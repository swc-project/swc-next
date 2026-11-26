use rustc_hash::FxHashMap;
use swc_next_ecma_ast::*;

use crate::common::{build, resolver::context::Resolver};

use super::util::{self, ModuleCtx};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_ref_rewriter.rs#L11>.
pub(super) type ImportMap = FxHashMap<SymbolId, ImportBinding>;

#[derive(Clone, Copy)]
pub(super) struct ImportBinding {
    pub mod_ident: SymbolId,
    pub mod_prop: Option<AstUtf8>,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_ref_rewriter.rs#L88>.
pub(super) fn rewrite_import_bindings(
    stmts: &mut [Stmt],
    cx: &mut ModuleCtx<'_, '_>,
    import_map: &ImportMap,
) {
    let mut rewriter = ImportQuery {
        ast: cx.ast,
        resolver: cx.resolver,
        import_map,
        helper_scope: cx.helpers.scope(),
    };
    for stmt in stmts {
        if let Some(replacement) = stmt.visit_mut_with(&mut rewriter) {
            *stmt = replacement;
        }
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_ref_rewriter.rs#L13>.
struct ImportQuery<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    import_map: &'a ImportMap,
    helper_scope: Option<ScopeId>,
}

impl ImportQuery<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L2783>.
    fn rewrite_callee(&mut self, callee: Expr) -> Option<Expr> {
        let bare = build::unwrap(self.ast, callee);
        let binding = self.lookup(bare)?;
        let mut replacement = self.query_ref(bare, binding);
        if self.should_fix_this(bare, binding) {
            let zero = self.ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
            replacement = build::sequence(self.ast, vec![zero, replacement]);
        }
        Some(replacement)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_ref_rewriter.rs#L77>.
    fn should_fix_this(&self, expr: Expr, binding: ImportBinding) -> bool {
        binding.mod_prop.is_some() && self.resolver.scope_of(expr.node_id()) != self.helper_scope
    }

    fn lookup(&self, expr: Expr) -> Option<ImportBinding> {
        let id = expr.as_identifier_reference(self.ast)?;
        self.import_map
            .get(&self.resolver.symbol(id.node_id())?)
            .copied()
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/module_ref_rewriter.rs#L38>.
    fn query_ref(&mut self, expr: Expr, binding: ImportBinding) -> Expr {
        let span = expr.span(self.ast);
        let mod_ident = self.resolver.symbol_reference(self.ast, binding.mod_ident);
        if let Some(mod_prop) = binding.mod_prop {
            util::named_member(self.ast, mod_ident, mod_prop, span)
        } else {
            mod_ident.set_span(self.ast, span);
            mod_ident
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L2608>.
    fn visit_reference(&mut self, expr: Expr) {
        let expr = build::unwrap(self.ast, expr);
        if !expr.is_identifier_reference(self.ast) {
            expr.visit_mut_children_with(self);
        }
    }
}

impl<'ast> VisitMut<'ast> for ImportQuery<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L2650>.
    fn visit_mut_expr(&mut self, expr: Expr) -> Option<Expr> {
        if let Some(binding) = self.lookup(expr) {
            return Some(self.query_ref(expr, binding));
        }
        expr.visit_mut_children_with(self)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L2630>.
    fn visit_mut_object_property(&mut self, property: ObjectProperty) -> Option<ObjectProperty> {
        if property.shorthand(self.ast) && self.lookup(property.value(self.ast)).is_some() {
            property.set_shorthand(self.ast, false);
        }
        property.visit_mut_children_with(self)
    }

    fn visit_mut_call_expression(&mut self, call: CallExpression) -> Option<CallExpression> {
        let callee = call.callee(self.ast);
        if let Some(replacement) = self.rewrite_callee(callee) {
            call.set_callee(self.ast, replacement);
        }
        call.visit_mut_children_with(self)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L2801>.
    fn visit_mut_tagged_template_expression(
        &mut self,
        tag: TaggedTemplateExpression,
    ) -> Option<TaggedTemplateExpression> {
        if let Some(replacement) = self.rewrite_callee(tag.tag(self.ast)) {
            tag.set_tag(self.ast, replacement);
        }
        tag.visit_mut_children_with(self)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L2758>.
    fn visit_mut_unary_expression(&mut self, unary: UnaryExpression) -> Option<UnaryExpression> {
        if unary.operator(self.ast) == UnaryOperator::Delete {
            self.visit_reference(unary.argument(self.ast));
            return None;
        }
        unary.visit_mut_children_with(self)
    }
}
