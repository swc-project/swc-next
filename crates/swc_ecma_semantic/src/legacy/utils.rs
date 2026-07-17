use swc_experimental_allocator::atom::Atom;
use swc_experimental_ecma_ast::{Ast, Expr, Ident, PropName};
use swc_experimental_ecma_visit::{Visit, VisitWith};

pub struct DestructuringFinder<'found, 'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    found: &'found mut Vec<Atom<'ast>>,
}

pub fn find_pat_ids<
    'found,
    'visit,
    'ast,
    N: VisitWith<'ast, DestructuringFinder<'found, 'visit, 'ast>>,
>(
    ast: &'visit Ast<'ast>,
    node: N,
    found: &'found mut Vec<Atom<'ast>>,
) {
    let mut v = DestructuringFinder { ast, found };
    node.visit_with(&mut v);
}

impl<'found, 'visit, 'ast> Visit<'ast> for DestructuringFinder<'found, 'visit, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
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
