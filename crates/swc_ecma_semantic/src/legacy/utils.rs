use rustc_hash::FxHashSet;
use swc_experimental_ecma_ast::{Ast, Expr, Ident, PropName, Utf8Ref};
use swc_experimental_ecma_visit::{Visit, VisitWith};

pub struct DestructuringFinder<'ast> {
    ast: &'ast Ast,
    found: &'ast mut Vec<Utf8Ref>,
}

pub fn find_pat_ids<'ast, N: VisitWith<DestructuringFinder<'ast>>>(
    ast: &'ast Ast,
    node: N,
    found: &'ast mut Vec<Utf8Ref>,
) {
    let mut v = DestructuringFinder { ast, found };
    node.visit_with(&mut v);
}

impl<'ast> Visit for DestructuringFinder<'ast> {
    fn ast(&self) -> &Ast {
        self.ast
    }

    /// No-op (we don't care about expressions)
    fn visit_expr(&mut self, _: Expr) {}

    fn visit_ident(&mut self, i: Ident) {
        self.found.push(i.sym(self.ast));
    }

    // fn visit_jsx_member_expr(&mut self, n: &JSXMemberExpr) {
    //     n.obj.visit_with(self);
    // }

    /// No-op (we don't care about expressions)
    fn visit_prop_name(&mut self, _: PropName) {}

    // fn visit_ts_type(&mut self, _: &TsType) {}
}
