use std::{hash::BuildHasherDefault, ops::RangeFull};

use indexmap::IndexMap;
use rustc_hash::FxHasher;
use swc_experimental_ecma_ast::{
    Ast, Class, ClassMember, Comments, Expr, GetSpan, IfStmt, SimpleAssignTarget, Span, Stmt,
    UnaryOp, VisitMut, VisitMutWith,
};

pub fn remove_paren<'visit, 'ast: 'visit, N>(
    root: &mut N,
    ast: &'visit mut Ast<'ast>,
    comments: Option<&mut Comments<'ast>>,
) where
    N: Copy + VisitMutWith<'ast, ParenRemover<'visit, 'ast>>,
{
    let mut visitor = ParenRemover {
        ast,
        span_map: Default::default(),
    };
    *root = (*root).visit_mut_with(&mut visitor);
    if let Some(c) = comments {
        for (to, from) in visitor.span_map.drain(RangeFull).rev() {
            c.move_leading(from.start, to.start);
            c.move_trailing(from.end, to.end);
        }
    }
}

pub struct ParenRemover<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    /// A hash map to preserve original span.
    ///
    /// Key is span of inner expression, and value is span of the paren
    /// expression.
    span_map: IndexMap<Span, Span, BuildHasherDefault<FxHasher>>,
}

impl ParenRemover<'_, '_> {
    fn unwrap_expr(&mut self, mut node: Expr) -> Expr {
        loop {
            node = match node {
                Expr::Seq(seq) if seq.exprs(self.ast).len() == 1 => {
                    seq.exprs(self.ast).get_node(self.ast, 0).unwrap()
                }
                Expr::Paren(paren) => {
                    let paren_span = paren.span(self.ast);
                    let inner_expr = paren.expr(self.ast);
                    let expr_span = inner_expr.span(self.ast);
                    self.span_map.insert(expr_span, paren_span);
                    inner_expr
                }
                _ => return node,
            };
        }
    }

    fn flatten_seq_expr(&mut self, node: Expr) -> Expr {
        let Expr::Seq(seq) = node else {
            return node;
        };

        let span = seq.span(self.ast);
        let exprs = collect_range(seq.exprs(self.ast), self.ast);
        let exprs_len = exprs.len();
        let flattened_len = exprs.iter().map(|expr| seq_expr_len(*expr, self.ast)).sum();

        let mut has_padding_value = false;
        let mut new_exprs = Vec::with_capacity(flattened_len);

        if flattened_len == exprs_len {
            for (idx, expr) in exprs.into_iter().enumerate() {
                if idx + 1 == exprs_len {
                    new_exprs.push(expr);
                } else if let Some(expr) =
                    ignore_return_value(expr, &mut has_padding_value, self.ast)
                {
                    new_exprs.push(expr);
                }
            }
        } else {
            for (idx, expr) in exprs.into_iter().enumerate() {
                let is_last = idx + 1 == exprs_len;
                match expr {
                    Expr::Seq(seq) => {
                        let seq_exprs = collect_range(seq.exprs(self.ast), self.ast);
                        if !is_last {
                            for expr in seq_exprs {
                                if let Some(expr) =
                                    ignore_return_value(expr, &mut has_padding_value, self.ast)
                                {
                                    new_exprs.push(expr);
                                }
                            }
                        } else {
                            let seq_exprs_len = seq_exprs.len();
                            for (idx, expr) in seq_exprs.into_iter().enumerate() {
                                if idx + 1 == seq_exprs_len {
                                    new_exprs.push(expr);
                                } else if let Some(expr) =
                                    ignore_return_value(expr, &mut has_padding_value, self.ast)
                                {
                                    new_exprs.push(expr);
                                }
                            }
                        }
                    }
                    expr => {
                        if is_last {
                            new_exprs.push(expr);
                        } else if let Some(expr) =
                            ignore_return_value(expr, &mut has_padding_value, self.ast)
                        {
                            new_exprs.push(expr);
                        }
                    }
                }
            }
        }

        match new_exprs.len() {
            0 => node,
            1 => new_exprs.pop().unwrap(),
            _ => {
                let exprs = ignore_padding_value(new_exprs);
                let exprs = self.ast.add_typed_sub_range(exprs);
                Expr::Seq(self.ast.seq_expr(span, exprs))
            }
        }
    }
}

impl<'ast> VisitMut<'ast> for ParenRemover<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_class(&mut self, node: Class) -> Class {
        let node = node.visit_mut_children_with(self);
        let body = node.body(self.ast);
        let old_len = body.len();
        let members: Vec<_> = collect_range(body, self.ast)
            .into_iter()
            .filter(|member| !matches!(member, ClassMember::Empty(..)))
            .collect();
        if members.len() != old_len {
            let body = self.ast.add_typed_sub_range(members);
            node.set_body(self.ast, body);
        }
        node
    }

    fn visit_mut_if_stmt(&mut self, node: IfStmt) -> IfStmt {
        let node = node.visit_mut_children_with(self);
        let cons = node.cons(self.ast);
        if will_eat_else_token(cons, self.ast) {
            let span = cons.span(self.ast);
            let stmts = self.ast.add_typed_sub_range([cons]);
            let cons = self.ast.stmt_block_stmt(span, stmts);
            node.set_cons(self.ast, cons);
        }
        node
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Expr {
        let node = self.unwrap_expr(node);
        let node = node.visit_mut_children_with(self);
        self.flatten_seq_expr(node)
    }

    fn visit_mut_simple_assign_target(&mut self, node: SimpleAssignTarget) -> SimpleAssignTarget {
        let node = node.visit_mut_children_with(self);
        if let SimpleAssignTarget::Paren(paren) = node {
            let paren_span = paren.span(self.ast);
            let inner_expr = paren.expr(self.ast);
            let expr_span = inner_expr.span(self.ast);
            let target = SimpleAssignTarget::try_from_expr(self.ast, inner_expr).unwrap();
            self.span_map.insert(expr_span, paren_span);
            target
        } else {
            node
        }
    }
}

fn collect_range<'ast, T>(
    range: swc_experimental_ecma_ast::TypedSubRange<T>,
    ast: &Ast<'ast>,
) -> Vec<T>
where
    T: swc_experimental_ecma_ast::ExtraDataCompact<'ast>,
{
    range
        .iter()
        .map(|id| ast.get_node_in_sub_range(id))
        .collect()
}

fn seq_expr_len(expr: Expr, ast: &Ast<'_>) -> usize {
    match expr {
        Expr::Paren(paren) => match paren.expr(ast) {
            Expr::Seq(seq) => seq.exprs(ast).len(),
            _ => 1,
        },
        Expr::Seq(seq) => seq.exprs(ast).len(),
        _ => 1,
    }
}

fn ignore_return_value(
    expr: Expr,
    has_padding_value: &mut bool,
    ast: &mut Ast<'_>,
) -> Option<Expr> {
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
            let span = seq.span(ast);
            let exprs = collect_range(seq.exprs(ast), ast);
            let len = exprs.len();
            let mut new_exprs = Vec::with_capacity(len);

            for (idx, expr) in exprs.into_iter().enumerate() {
                if idx + 1 == len {
                    new_exprs.push(expr);
                } else if let Some(expr) = ignore_return_value(expr, has_padding_value, ast) {
                    new_exprs.push(expr);
                }
            }

            match new_exprs.len() {
                0 | 1 => new_exprs.pop(),
                _ => {
                    let exprs = ast.add_typed_sub_range(new_exprs);
                    Some(Expr::Seq(ast.seq_expr(span, exprs)))
                }
            }
        }
        Expr::Unary(unary) if unary.op(ast) == UnaryOp::Void => {
            ignore_return_value(unary.arg(ast), has_padding_value, ast)
        }
        _ => Some(expr),
    }
}

fn ignore_padding_value(exprs: Vec<Expr>) -> Vec<Expr> {
    let len = exprs.len();
    if len <= 2 {
        return exprs;
    }

    exprs
        .into_iter()
        .enumerate()
        .filter_map(|(idx, expr)| match expr {
            Expr::Fn(..) | Expr::Arrow(..) | Expr::Lit(..) if idx + 1 != len => None,
            _ => Some(expr),
        })
        .collect()
}

fn will_eat_else_token(stmt: Stmt, ast: &Ast<'_>) -> bool {
    match stmt {
        Stmt::If(if_stmt) => match if_stmt.alt(ast) {
            Some(alt) => will_eat_else_token(alt, ast),
            None => true,
        },
        Stmt::Block(..) => false,
        Stmt::Labeled(stmt) => will_eat_else_token(stmt.body(ast), ast),
        Stmt::While(stmt) => will_eat_else_token(stmt.body(ast), ast),
        Stmt::For(stmt) => will_eat_else_token(stmt.body(ast), ast),
        Stmt::ForIn(stmt) => will_eat_else_token(stmt.body(ast), ast),
        Stmt::ForOf(stmt) => will_eat_else_token(stmt.body(ast), ast),
        _ => false,
    }
}
