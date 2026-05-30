use std::{hash::BuildHasherDefault, ops::RangeFull};

use indexmap::IndexMap;
use rustc_hash::FxHasher;
use swc_experimental_ecma_ast::{
    AstBuilder, Comments, Expr, GetSpan, SimpleAssignTarget, Span, VisitMut, VisitMutWith,
};

pub fn remove_paren<'ast, N>(
    mut root: N,
    ast: AstBuilder<'ast>,
    comments: Option<&mut Comments>,
) -> N
where
    N: VisitMutWith<'ast, ParenRemover<'ast>>,
{
    let mut visitor = ParenRemover {
        ast,
        span_map: Default::default(),
    };
    root.visit_mut_with(&mut visitor);
    if let Some(c) = comments {
        for (to, from) in visitor.span_map.drain(RangeFull).rev() {
            c.move_leading(from.start, to.start);
            c.move_trailing(from.end, to.end);
        }
    }
    root
}

pub struct ParenRemover<'a> {
    ast: AstBuilder<'a>,
    /// A hash map to preserve original span.
    ///
    /// Key is span of inner expression, and value is span of the paren
    /// expression.
    span_map: IndexMap<Span, Span, BuildHasherDefault<FxHasher>>,
}

impl<'a> VisitMut<'a> for ParenRemover<'a> {
    fn visit_mut_expr(&mut self, node: &mut Expr<'a>) {
        node.visit_mut_children_with(self);
        if let Expr::Paren(expr) = node {
            let paren_span = expr.span();
            let inner_expr = std::mem::replace(&mut expr.expr, self.ast.expr_invalid());
            let expr_span = inner_expr.span();
            self.span_map.insert(expr_span, paren_span);
            *node = inner_expr;
        }
    }

    fn visit_mut_simple_assign_target(&mut self, node: &mut SimpleAssignTarget<'a>) {
        node.visit_mut_children_with(self);
        if let SimpleAssignTarget::Paren(expr) = node {
            let paren_span = expr.span();
            let inner_expr = std::mem::replace(&mut expr.expr, self.ast.expr_invalid());
            let expr_span = inner_expr.span();
            let target = SimpleAssignTarget::try_from_expr(inner_expr, self.ast.allocator).unwrap();
            self.span_map.insert(expr_span, paren_span);
            *node = target;
        }
    }
}
