use swc_experimental_ecma_ast::{Ast, Expr, Ident, PropName, Utf8Str};
use swc_experimental_ecma_visit::{Visit, VisitWith};

pub struct DestructuringFinder<'ast> {
    ast: &'ast Ast,
    found: Vec<Utf8Str<'ast>>,
}

pub fn find_pat_ids<'ast, N: VisitWith<DestructuringFinder<'ast>>>(
    ast: &'ast Ast,
    node: N,
) -> Vec<Utf8Str<'ast>> {
    let mut v = DestructuringFinder {
        ast,
        found: Vec::new(),
    };
    node.visit_with(&mut v);

    v.found
}

impl<'ast> Visit for DestructuringFinder<'ast> {
    fn ast(&self) -> &Ast {
        self.ast
    }

    /// No-op (we don't care about expressions)
    fn visit_expr(&mut self, _: Expr) {}

    fn visit_ident(&mut self, i: Ident) {
        self.found.push(self.ast.get_utf8(i.sym(self.ast)));
    }

    // fn visit_jsx_member_expr(&mut self, n: &JSXMemberExpr) {
    //     n.obj.visit_with(self);
    // }

    /// No-op (we don't care about expressions)
    fn visit_prop_name(&mut self, _: PropName) {}

    // fn visit_ts_type(&mut self, _: &TsType) {}
}
