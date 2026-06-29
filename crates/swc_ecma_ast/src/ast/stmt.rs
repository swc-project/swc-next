use std::cell::Cell;

use crate::{Span, semantic::ScopeId};
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ast_macros::ast;

use crate::ast::{Decl, Expr, Ident, Lit, Pat, UsingDecl, VarDecl};

#[ast]
#[derive(Debug)]
pub struct BlockStmt<'a> {
    pub span: Span,
    pub stmts: Vec<'a, Stmt<'a>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug)]
pub enum Stmt<'a> {
    Block(Box<'a, BlockStmt<'a>>),
    Empty(Box<'a, EmptyStmt>),
    Debugger(Box<'a, DebuggerStmt>),
    With(Box<'a, WithStmt<'a>>),
    Return(Box<'a, ReturnStmt<'a>>),
    Labeled(Box<'a, LabeledStmt<'a>>),
    Break(Box<'a, BreakStmt<'a>>),
    Continue(Box<'a, ContinueStmt<'a>>),
    If(Box<'a, IfStmt<'a>>),
    Switch(Box<'a, SwitchStmt<'a>>),
    Throw(Box<'a, ThrowStmt<'a>>),
    Try(Box<'a, TryStmt<'a>>),
    While(Box<'a, WhileStmt<'a>>),
    DoWhile(Box<'a, DoWhileStmt<'a>>),
    For(Box<'a, ForStmt<'a>>),
    ForIn(Box<'a, ForInStmt<'a>>),
    ForOf(Box<'a, ForOfStmt<'a>>),
    Decl(Box<'a, Decl<'a>>),
    Expr(Box<'a, ExprStmt<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct ExprStmt<'a> {
    pub span: Span,
    pub expr: Expr<'a>,
}

#[ast]
#[derive(Debug, Clone)]
pub struct EmptyStmt {
    pub span: Span,
}

#[ast]
#[derive(Debug, Clone)]
pub struct DebuggerStmt {
    pub span: Span,
}

#[ast]
#[derive(Debug)]
pub struct WithStmt<'a> {
    pub span: Span,
    pub obj: Expr<'a>,
    pub body: Stmt<'a>,
}

#[ast]
#[derive(Debug)]
pub struct ReturnStmt<'a> {
    pub span: Span,
    pub arg: Option<Expr<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct LabeledStmt<'a> {
    pub span: Span,
    pub label: Box<'a, Ident<'a>>,
    pub body: Stmt<'a>,
}

#[ast]
#[derive(Debug)]
pub struct BreakStmt<'a> {
    pub span: Span,
    pub label: Option<Box<'a, Ident<'a>>>,
}

#[ast]
#[derive(Debug)]
pub struct ContinueStmt<'a> {
    pub span: Span,
    pub label: Option<Box<'a, Ident<'a>>>,
}

#[ast]
#[derive(Debug)]
pub struct IfStmt<'a> {
    pub span: Span,
    pub test: Expr<'a>,
    pub cons: Stmt<'a>,
    pub alt: Option<Stmt<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct SwitchStmt<'a> {
    pub span: Span,
    pub discriminant: Expr<'a>,
    pub cases: Vec<'a, SwitchCase<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct ThrowStmt<'a> {
    pub span: Span,
    pub arg: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct TryStmt<'a> {
    pub span: Span,
    pub block: Box<'a, BlockStmt<'a>>,
    pub handler: Option<Box<'a, CatchClause<'a>>>,
    pub finalizer: Option<Box<'a, BlockStmt<'a>>>,
}

#[ast]
#[derive(Debug)]
pub struct WhileStmt<'a> {
    pub span: Span,
    pub test: Expr<'a>,
    pub body: Stmt<'a>,
}

#[ast]
#[derive(Debug)]
pub struct DoWhileStmt<'a> {
    pub span: Span,
    pub test: Expr<'a>,
    pub body: Stmt<'a>,
}

#[ast]
#[derive(Debug)]
pub struct ForStmt<'a> {
    pub span: Span,
    pub init: Option<VarDeclOrExpr<'a>>,
    pub test: Option<Expr<'a>>,
    pub update: Option<Expr<'a>>,
    pub body: Stmt<'a>,
}

#[ast]
#[derive(Debug)]
pub struct ForInStmt<'a> {
    pub span: Span,
    pub left: ForHead<'a>,
    pub right: Expr<'a>,
    pub body: Stmt<'a>,
}

#[ast]
#[derive(Debug)]
pub struct ForOfStmt<'a> {
    pub span: Span,
    pub is_await: bool,
    pub left: ForHead<'a>,
    pub right: Expr<'a>,
    pub body: Stmt<'a>,
}

#[ast]
#[derive(Debug)]
pub struct SwitchCase<'a> {
    pub span: Span,
    pub test: Option<Expr<'a>>,
    pub cons: Vec<'a, Stmt<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct CatchClause<'a> {
    pub span: Span,
    pub param: Option<Pat<'a>>,
    pub body: Box<'a, BlockStmt<'a>>,
}

#[ast]
#[derive(Debug)]
pub enum ForHead<'a> {
    VarDecl(Box<'a, VarDecl<'a>>),
    UsingDecl(Box<'a, UsingDecl<'a>>),
    Pat(Box<'a, Pat<'a>>),
}

#[ast]
#[derive(Debug)]
pub enum VarDeclOrExpr<'a> {
    VarDecl(Box<'a, VarDecl<'a>>),
    Expr(Box<'a, Expr<'a>>),
}

impl Stmt<'_> {
    pub fn is_use_strict(&self) -> bool {
        match self {
            Stmt::Expr(expr) => match &expr.expr {
                Expr::Lit(lit) => match &**lit {
                    Lit::Str(s) => {
                        matches!(
                            s.raw.as_ref().map(|raw| raw.as_str()),
                            Some("\"use strict\"" | "'use strict'")
                        )
                    }
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        }
    }

    /// Returns true if the statement does not prevent the directives below
    /// `self` from being directives.
    pub fn can_precede_directive(&self) -> bool {
        matches!(self, Stmt::Expr(expr) if matches!(&expr.expr, Expr::Lit(lit) if matches!(&**lit, Lit::Str(_))))
    }
}
