use swc_experimental_allocator::atom::Atom;
use swc_experimental_ecma_ast::{Expr, Ident, PropName};
use swc_experimental_ecma_visit::{Visit, VisitWith};

pub struct DestructuringFinder<'found, 'ast> {
    found: &'found mut Vec<Atom<'ast>>,
}

pub fn find_pat_ids<'found, 'ast, N>(node: &N, found: &'found mut Vec<Atom<'ast>>)
where
    N: VisitWith<'ast, DestructuringFinder<'found, 'ast>>,
{
    let mut v = DestructuringFinder { found };
    node.visit_with(&mut v);
}

impl<'found, 'ast> Visit<'ast> for DestructuringFinder<'found, 'ast> {
    /// No-op (we don't care about expressions)
    fn visit_expr(&mut self, _: &Expr<'ast>) {}

    fn visit_ident(&mut self, i: &Ident<'ast>) {
        self.found.push(i.sym);
    }

    // fn visit_jsx_member_expr(&mut self, n: &JSXMemberExpr) {
    //     n.obj.visit_with(self);
    // }

    /// No-op (we don't care about expressions)
    fn visit_prop_name(&mut self, _: &PropName<'ast>) {}

    // fn visit_ts_type(&mut self, _: &TsType) {}
}
