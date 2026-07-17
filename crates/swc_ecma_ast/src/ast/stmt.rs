use crate::{
    Ast, Lit,
    ast::{Decl, Expr, Pat, UsingDecl, VarDecl},
};
use swc_experimental_ast_macros::ast;

#[ast]
pub struct BlockStmt {
    stmts: Vec<Stmt>,
    scope_id: Cell<Option<ScopeId>>,
}

#[ast]
pub enum Stmt {
    Block(BlockStmt),
    Empty(EmptyStmt),
    Debugger(DebuggerStmt),
    With(WithStmt),
    Return(ReturnStmt),
    Labeled(LabeledStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    If(IfStmt),
    Switch(SwitchStmt),
    Throw(ThrowStmt),
    Try(TryStmt),
    While(WhileStmt),
    DoWhile(DoWhileStmt),
    For(ForStmt),
    ForIn(ForInStmt),
    ForOf(ForOfStmt),
    Decl(Decl),
    Expr(ExprStmt),
}

#[ast]
pub struct ExprStmt {
    expr: Expr,
}

#[ast]
pub struct EmptyStmt {}

#[ast]
pub struct DebuggerStmt {}

#[ast]
pub struct WithStmt {
    obj: Expr,
    body: Stmt,
}

#[ast]
pub struct ReturnStmt {
    arg: Option<Expr>,
}

#[ast]
pub struct LabeledStmt {
    label: Ident,
    body: Stmt,
}

#[ast]
pub struct BreakStmt {
    label: Option<Ident>,
}

#[ast]
pub struct ContinueStmt {
    label: Option<Ident>,
}

#[ast]
pub struct IfStmt {
    test: Expr,
    cons: Stmt,
    alt: Option<Stmt>,
}

#[ast]
pub struct SwitchStmt {
    discriminant: Expr,
    cases: Vec<SwitchCase>,
}

#[ast]
pub struct ThrowStmt {
    arg: Expr,
}

#[ast]
pub struct TryStmt {
    block: BlockStmt,
    handler: Option<CatchClause>,
    finalizer: Option<BlockStmt>,
}

#[ast]
pub struct WhileStmt {
    test: Expr,
    body: Stmt,
}

#[ast]
pub struct DoWhileStmt {
    test: Expr,
    body: Stmt,
}

#[ast]
pub struct ForStmt {
    init: Option<VarDeclOrExpr>,
    test: Option<Expr>,
    update: Option<Expr>,
    body: Stmt,
}

#[ast]
pub struct ForInStmt {
    left: ForHead,
    right: Expr,
    body: Stmt,
}

#[ast]
pub struct ForOfStmt {
    is_await: bool,
    left: ForHead,
    right: Expr,
    body: Stmt,
}

#[ast]
pub struct SwitchCase {
    test: Option<Expr>,
    cons: Vec<Stmt>,
}

#[ast]
pub struct CatchClause {
    param: Option<Pat>,
    body: BlockStmt,
}

#[ast]
pub enum ForHead {
    VarDecl(VarDecl),
    UsingDecl(UsingDecl),
    Pat(Pat),
}

#[ast]
pub enum VarDeclOrExpr {
    VarDecl(VarDecl),
    Expr(Expr),
}

impl Stmt {
    pub fn is_use_strict(&self, ast: &Ast<'_>) -> bool {
        match self {
            Stmt::Expr(expr) => match expr.expr(ast) {
                Expr::Lit(Lit::Str(s)) => {
                    let raw = s.raw(ast);
                    matches!(raw, Some(value) if value == "\"use strict\"" || value == "'use strict'")
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Returns true if the statement does not prevent the directives below
    /// `self` from being directives.
    pub fn can_precede_directive(&self, ast: &Ast<'_>) -> bool {
        match self {
            Stmt::Expr(expr) => matches!(expr.expr(ast), Expr::Lit(Lit::Str(_))),
            _ => false,
        }
    }
}
