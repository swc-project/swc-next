use std::{hash::BuildHasherDefault, ops::RangeFull};

use indexmap::IndexMap;
use rustc_hash::FxHasher;
use swc_experimental_allocator::{Allocator, boxed::Box as AstBox, vec::Vec as AstVec};
use swc_experimental_ecma_ast::{
    AstBuilder, Class, ClassMember, Comments, Expr, GetSpan, IfStmt, SeqExpr, SimpleAssignTarget,
    Span, Stmt, UnaryExpr, UnaryOp, VisitMut, VisitMutWith,
};

pub fn remove_paren<'ast, N: VisitMutWith<'ast, ParenRemover<'ast>>>(
    root: &mut N,
    allocator: &'ast Allocator,
    comments: Option<&mut Comments>,
) {
    let mut visitor = ParenRemover {
        ast: AstBuilder { allocator },
        span_map: Default::default(),
    };
    root.visit_mut_with(&mut visitor);
    if let Some(c) = comments {
        for (to, from) in visitor.span_map.drain(RangeFull).rev() {
            c.move_leading(from.start, to.start);
            c.move_trailing(from.end, to.end);
        }
    }
}

pub struct ParenRemover<'a> {
    ast: AstBuilder<'a>,
    /// A hash map to preserve original span.
    ///
    /// Key is span of inner expression, and value is span of the paren
    /// expression.
    span_map: IndexMap<Span, Span, BuildHasherDefault<FxHasher>>,
}

impl<'a> ParenRemover<'a> {
    fn invalid_expr(&self) -> Expr<'a> {
        self.ast.expr_invalid()
    }

    fn unwrap_expr(&mut self, node: &mut Expr<'a>) {
        loop {
            match node {
                Expr::Seq(seq) if seq.exprs.len() == 1 => {
                    *node = std::mem::replace(&mut seq.exprs[0], self.invalid_expr());
                }
                Expr::Paren(expr) => {
                    let paren_span = expr.span();
                    let inner_expr = std::mem::replace(&mut expr.expr, self.invalid_expr());
                    let expr_span = inner_expr.span();
                    self.span_map.insert(expr_span, paren_span);
                    *node = inner_expr;
                }
                _ => return,
            }
        }
    }

    fn flatten_seq_expr(&mut self, node: &mut Expr<'a>) {
        let replacement = match node {
            Expr::Seq(seq) => {
                let span = seq.span;
                let exprs_len = seq.exprs.len();
                let flattened_len = seq.exprs.iter().map(seq_expr_len).sum();
                let exprs = std::mem::replace(&mut seq.exprs, AstVec::new_in(self.ast.allocator));

                let mut has_padding_value = false;
                let mut new_exprs = AstVec::with_capacity_in(flattened_len, self.ast.allocator);

                if flattened_len == exprs_len {
                    for (idx, expr) in exprs.into_iter().enumerate() {
                        if idx + 1 == exprs_len {
                            new_exprs.push(expr);
                        } else if let Some(expr) =
                            ignore_return_value(expr, &mut has_padding_value, self.ast.allocator)
                        {
                            new_exprs.push(expr);
                        }
                    }
                } else {
                    for (idx, expr) in exprs.into_iter().enumerate() {
                        let is_last = idx + 1 == exprs_len;
                        match expr {
                            Expr::Seq(seq) => {
                                let seq_exprs = AstBox::into_inner(seq).exprs;
                                if !is_last {
                                    new_exprs.extend(seq_exprs.into_iter().filter_map(|expr| {
                                        ignore_return_value(
                                            expr,
                                            &mut has_padding_value,
                                            self.ast.allocator,
                                        )
                                    }));
                                } else {
                                    let seq_exprs_len = seq_exprs.len();
                                    for (idx, expr) in seq_exprs.into_iter().enumerate() {
                                        if idx + 1 == seq_exprs_len {
                                            new_exprs.push(expr);
                                        } else if let Some(expr) = ignore_return_value(
                                            expr,
                                            &mut has_padding_value,
                                            self.ast.allocator,
                                        ) {
                                            new_exprs.push(expr);
                                        }
                                    }
                                }
                            }
                            expr => {
                                if is_last {
                                    new_exprs.push(expr);
                                } else if let Some(expr) = ignore_return_value(
                                    expr,
                                    &mut has_padding_value,
                                    self.ast.allocator,
                                ) {
                                    new_exprs.push(expr);
                                }
                            }
                        }
                    }
                }

                match new_exprs.len() {
                    0 => None,
                    1 => new_exprs.pop(),
                    _ => {
                        let exprs = ignore_padding_value(new_exprs, self.ast.allocator);
                        Some(Expr::Seq(self.ast.allocator.boxed(SeqExpr { span, exprs })))
                    }
                }
            }
            _ => None,
        };

        if let Some(replacement) = replacement {
            *node = replacement;
        }
    }
}

impl<'a> VisitMut<'a> for ParenRemover<'a> {
    fn visit_mut_class(&mut self, node: &mut Class<'a>) {
        node.visit_mut_children_with(self);
        node.body.retain(|m| !matches!(m, ClassMember::Empty(..)));
    }

    fn visit_mut_if_stmt(&mut self, node: &mut IfStmt<'a>) {
        node.visit_mut_children_with(self);
        if will_eat_else_token(&node.cons) {
            let span = node.cons.span();
            let cons = std::mem::replace(&mut node.cons, self.ast.stmt_empty_stmt(span));
            let mut stmts = AstVec::with_capacity_in(1, self.ast.allocator);
            stmts.push(cons);
            node.cons = self.ast.stmt_block_stmt(span, stmts);
        }
    }

    fn visit_mut_expr(&mut self, node: &mut Expr<'a>) {
        self.unwrap_expr(node);
        node.visit_mut_children_with(self);
        self.flatten_seq_expr(node);
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

fn seq_expr_len(expr: &Expr) -> usize {
    match expr {
        Expr::Paren(paren) => match &paren.expr {
            Expr::Seq(seq) => seq.exprs.len(),
            _ => 1,
        },
        Expr::Seq(seq) => seq.exprs.len(),
        _ => 1,
    }
}

fn ignore_return_value<'a>(
    expr: Expr<'a>,
    has_padding_value: &mut bool,
    allocator: &'a Allocator,
) -> Option<Expr<'a>> {
    match expr {
        Expr::Fn(..) | Expr::Arrow(..) | Expr::Lit(..) => {
            if *has_padding_value {
                None
            } else {
                *has_padding_value = true;
                Some(expr)
            }
        }
        Expr::Seq(seq) => {
            let SeqExpr { span, exprs } = AstBox::into_inner(seq);
            let len = exprs.len();
            let mut new_exprs = AstVec::with_capacity_in(len, allocator);

            for (idx, expr) in exprs.into_iter().enumerate() {
                if idx + 1 == len {
                    new_exprs.push(expr);
                } else if let Some(expr) = ignore_return_value(expr, has_padding_value, allocator) {
                    new_exprs.push(expr);
                }
            }

            match new_exprs.len() {
                0 | 1 => new_exprs.pop(),
                _ => Some(Expr::Seq(allocator.boxed(SeqExpr {
                    span,
                    exprs: new_exprs,
                }))),
            }
        }
        Expr::Unary(unary) => {
            let UnaryExpr { span, op, arg } = AstBox::into_inner(unary);
            if op == UnaryOp::Void {
                ignore_return_value(arg, has_padding_value, allocator)
            } else {
                Some(Expr::Unary(allocator.boxed(UnaryExpr { span, op, arg })))
            }
        }
        _ => Some(expr),
    }
}

fn ignore_padding_value<'a>(
    exprs: AstVec<'a, Expr<'a>>,
    allocator: &'a Allocator,
) -> AstVec<'a, Expr<'a>> {
    let len = exprs.len();
    if len <= 2 {
        return exprs;
    }

    let mut new_exprs = AstVec::with_capacity_in(len, allocator);
    for (idx, expr) in exprs.into_iter().enumerate() {
        match &expr {
            Expr::Fn(..) | Expr::Arrow(..) | Expr::Lit(..) if idx + 1 != len => {}
            _ => new_exprs.push(expr),
        }
    }
    new_exprs
}

fn will_eat_else_token(s: &Stmt) -> bool {
    match s {
        Stmt::If(s) => match &s.alt {
            Some(alt) => will_eat_else_token(alt),
            None => true,
        },
        Stmt::Block(..) => false,
        Stmt::Labeled(s) => will_eat_else_token(&s.body),
        Stmt::While(s) => will_eat_else_token(&s.body),
        Stmt::For(s) => will_eat_else_token(&s.body),
        Stmt::ForIn(s) => will_eat_else_token(&s.body),
        Stmt::ForOf(s) => will_eat_else_token(&s.body),
        _ => false,
    }
}
