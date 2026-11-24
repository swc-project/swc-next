use crate::common::{TraverseCtx, build as b};
use swc_next_ecma_ast::*;

pub(super) struct ClassNameTdzFolder<'a, 'ast, 'r> {
    pub ast: &'a mut Ast<'ast>,
    pub ctx: &'a mut TraverseCtx<'r>,
    pub class_name: BindingIdentifier,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/class_name_tdz.rs#L15>.
impl<'ast> VisitMut<'ast> for ClassNameTdzFolder<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        if let Some(id) = node.as_identifier_reference(self.ast) {
            if self.ast.get_utf8(id.name(self.ast))
                == self.ast.get_utf8(self.class_name.name(self.ast))
                && self.ctx.resolver.node_scopes[id.node_id()]
                    == self.ctx.resolver.node_scopes[self.class_name.node_id()]
            {
                let helper = self.ctx.helper(self.ast, "class_name_tdz_error");
                let name = self.ast.get_utf8(id.name(self.ast)).to_owned();
                let name = b::string(self.ast, &name);
                let error = b::call(self.ast, helper, [name], false);
                return Some(b::sequence(self.ast, vec![error, node]));
            }
            return None;
        }
        if let Some(call) = node.as_call_expression(self.ast) {
            let static_helper = call
                .callee(self.ast)
                .as_identifier_reference(self.ast)
                .is_some_and(|id| {
                    matches!(
                        self.ast.get_utf8(id.name(self.ast)),
                        "_class_static_private_field_destructure"
                            | "_class_static_private_field_spec_set"
                            | "_class_static_private_field_update"
                            | "_class_static_private_method_get"
                            | "_class_static_private_field_spec_get"
                    )
                });
            if let Some(callee) = call.callee(self.ast).visit_mut_with(self) {
                call.set_callee(self.ast, callee);
            }
            let args = call.arguments(self.ast);
            for (index, slot) in args.iter().enumerate() {
                if static_helper && index == 1 {
                    continue;
                }
                let arg = self.ast.get_node_in_sub_range(slot);
                if let Some(arg) = arg.visit_mut_with(self) {
                    args.replace_slot(self.ast, slot, arg);
                }
            }
            return None;
        }
        node.visit_mut_children_with(self);
        None
    }
}
