use crate::common::{build as b, resolver::context::Resolver};
use swc_next_ecma_ast::*;

pub(super) struct ThisInStaticFolder<'a, 'ast> {
    pub ast: &'a mut Ast<'ast>,
    pub resolver: &'a mut Resolver,
    pub ident: BindingIdentifier,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/this_in_static.rs#L8>.
impl<'ast> VisitMut<'ast> for ThisInStaticFolder<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_function(&mut self, _: Function) -> Option<Function> {
        None
    }

    fn visit_mut_method_definition(&mut self, node: MethodDefinition) -> Option<MethodDefinition> {
        if node.kind(self.ast) != MethodDefinitionKind::Constructor {
            node.visit_mut_children_with(self);
        }
        None
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        node.visit_mut_children_with(self);
        node.is_this_expression(self.ast)
            .then(|| self.resolver.reference(self.ast, self.ident))
    }
}

pub(super) struct NewTargetInProp<'a, 'ast> {
    pub ast: &'a mut Ast<'ast>,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/this_in_static.rs#L27>.
impl<'ast> VisitMut<'ast> for NewTargetInProp<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_function(&mut self, _: Function) -> Option<Function> {
        None
    }

    fn visit_mut_method_definition(&mut self, node: MethodDefinition) -> Option<MethodDefinition> {
        if node.kind(self.ast) != MethodDefinitionKind::Constructor {
            node.visit_mut_children_with(self);
        }
        None
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        node.visit_mut_children_with(self);
        node.as_meta_property(self.ast)
            .filter(|meta| self.ast.get_utf8(meta.meta(self.ast).name(self.ast)) == "new")
            .map(|_| b::undefined(self.ast))
    }
}
