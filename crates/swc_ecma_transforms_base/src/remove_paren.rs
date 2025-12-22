use std::{hash::BuildHasherDefault, ops::RangeFull};

use indexmap::IndexMap;
use rustc_hash::FxHasher;
use swc_core::common::comments::Comments;
use swc_experimental_ecma_ast::{
    Ast, Expr, NodeIdTrait, SimpleAssignTarget, Span, VisitMut, VisitMutWith,
};

pub fn remove_paren<'ast, N: VisitMutWith<ParenRemover<'ast>>>(
    root: N,
    ast: &'ast mut Ast,
    comments: Option<&dyn Comments>,
) {
    let mut visitor = ParenRemover {
        ast,
        span_map: Default::default(),
    };
    root.visit_mut_with(&mut visitor);

    if let Some(c) = comments {
        for (to, from) in visitor.span_map.drain(RangeFull).rev() {
            c.move_leading(from.lo, to.lo);
            c.move_trailing(from.hi, to.hi);
        }
    }
}

pub struct ParenRemover<'a> {
    ast: &'a mut Ast,
    /// A hash map to preserve original span.
    ///
    /// Key is span of inner expression, and value is span of the paren
    /// expression.
    span_map: IndexMap<Span, Span, BuildHasherDefault<FxHasher>>,
}

impl VisitMut for ParenRemover<'_> {
    fn ast(&mut self) -> &mut Ast {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Expr {
        let node = node.visit_mut_children_with(self);
        if let Expr::Paren(expr) = node {
            let paren_span = expr.span(self.ast);
            let inner_expr = expr.expr(self.ast);
            let expr_span = inner_expr.span(self.ast);
            self.span_map.insert(expr_span, paren_span);

            self.ast.free_node(node.node_id());
            return inner_expr;
        }
        node
    }

    fn visit_mut_simple_assign_target(&mut self, node: SimpleAssignTarget) -> SimpleAssignTarget {
        let node = node.visit_mut_children_with(self);
        if let SimpleAssignTarget::Paren(expr) = node {
            let paren_expr = expr.span(self.ast);
            let inner_expr = expr.expr(self.ast);
            let expr_span = inner_expr.span(self.ast);
            let target = SimpleAssignTarget::try_from_expr(self.ast, inner_expr).unwrap();
            self.span_map.insert(expr_span, paren_expr);

            self.ast.free_node(node.node_id());
            return target;
        }
        node
    }
}
