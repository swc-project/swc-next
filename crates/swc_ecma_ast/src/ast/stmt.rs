use crate::{Ast, ast::*};
use swc_experimental_ast_macros::ast;

#[ast]
pub struct BlockStmt {
    stmts: Vec<Stmt>,
    scope_id: Cell<Option<ScopeId>>,
}

#[ast(repr)]
pub enum Stmt {
    // Decl
    Class(ClassDecl),
    Fn(FnDecl),
    Var(VarDecl),
    Using(UsingDecl),

    // Stmt
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
    Expr(ExprStmt),
}

#[allow(non_snake_case)]
impl Stmt {
    #[inline]
    pub fn Decl(decl: Decl) -> Self {
        const {
            assert!(std::mem::size_of::<Decl>() == std::mem::size_of::<Self>());
        }
        unsafe { std::mem::transmute(decl) }
    }
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

#[ast(repr)]
pub enum ForHead {
    // Pat::Expr
    ExprThis(ThisExpr),
    ExprArray(ArrayLit),
    ExprObject(ObjectLit),
    ExprFn(FnExpr),
    ExprUnary(UnaryExpr),
    ExprUpdate(UpdateExpr),
    ExprBin(BinExpr),
    ExprAssign(AssignExpr),
    ExprMember(MemberExpr),
    ExprSuperProp(SuperPropExpr),
    ExprCond(CondExpr),
    ExprImport(ImportExpr),
    ExprCall(CallExpr),
    ExprNew(NewExpr),
    ExprSeq(SeqExpr),
    ExprIdent(Ident),
    ExprStr(Str),
    ExprBool(Bool),
    ExprNull(Null),
    ExprNum(Number),
    ExprBigInt(BigInt),
    ExprRegex(Regex),
    ExprTpl(Tpl),
    ExprTaggedTpl(TaggedTpl),
    ExprArrow(ArrowExpr),
    ExprClass(ClassExpr),
    ExprYield(YieldExpr),
    ExprMetaProp(MetaPropExpr),
    ExprAwait(AwaitExpr),
    ExprParen(ParenExpr),
    ExprJSXMember(JSXMemberExpr),
    ExprJSXNamespacedName(JSXNamespacedName),
    ExprJSXEmpty(JSXEmptyExpr),
    ExprJSXElement(JSXElement),
    ExprJSXFragment(JSXFragment),
    ExprPrivateName(PrivateName),
    ExprOptChain(OptChainExpr),
    ExprInvalid(Invalid),

    // Pat
    Ident(BindingIdent),
    Array(ArrayPat),
    Rest(RestPat),
    Object(ObjectPat),
    Assign(AssignPat),
    Invalid(Invalid),

    // ForHead
    VarDecl(VarDecl),
    UsingDecl(UsingDecl),
}

#[allow(non_snake_case)]
impl ForHead {
    #[inline]
    pub fn Pat(pat: Pat) -> Self {
        const {
            assert!(std::mem::size_of::<Pat>() == std::mem::size_of::<Self>());
        }
        unsafe { std::mem::transmute(pat) }
    }

    #[inline]
    pub fn into_pat(self) -> Result<Pat, Self> {
        match self {
            Self::VarDecl(_) | Self::UsingDecl(_) => Err(self),
            _ => unsafe { Ok(std::mem::transmute(self)) },
        }
    }
}

#[ast(repr)]
pub enum VarDeclOrExpr {
    // Expr
    This(ThisExpr),
    Array(ArrayLit),
    Object(ObjectLit),
    Fn(FnExpr),
    Unary(UnaryExpr),
    Update(UpdateExpr),
    Bin(BinExpr),
    Assign(AssignExpr),
    Member(MemberExpr),
    SuperProp(SuperPropExpr),
    Cond(CondExpr),
    Import(ImportExpr),
    Call(CallExpr),
    New(NewExpr),
    Seq(SeqExpr),
    Ident(Ident),
    Str(Str),
    Bool(Bool),
    Null(Null),
    Num(Number),
    BigInt(BigInt),
    Regex(Regex),
    Tpl(Tpl),
    TaggedTpl(TaggedTpl),
    Arrow(ArrowExpr),
    Class(ClassExpr),
    Yield(YieldExpr),
    MetaProp(MetaPropExpr),
    Await(AwaitExpr),
    Paren(ParenExpr),
    JSXMember(JSXMemberExpr),
    JSXNamespacedName(JSXNamespacedName),
    JSXEmpty(JSXEmptyExpr),
    JSXElement(JSXElement),
    JSXFragment(JSXFragment),
    PrivateName(PrivateName),
    OptChain(OptChainExpr),
    Invalid(Invalid),

    // VarDeclOrExpr
    VarDecl(VarDecl),
}

#[allow(non_snake_case)]
impl VarDeclOrExpr {
    #[inline]
    pub fn Expr(expr: Expr) -> Self {
        const {
            assert!(std::mem::size_of::<Expr>() == std::mem::size_of::<Self>());
        }
        unsafe { std::mem::transmute(expr) }
    }

    #[inline]
    pub fn into_expr(self) -> Result<Expr, Self> {
        match self {
            Self::VarDecl(_) => Err(self),
            _ => unsafe { Ok(std::mem::transmute(self)) },
        }
    }
}

impl Stmt {
    pub fn is_use_strict(self, ast: &Ast<'_>) -> bool {
        match self {
            Stmt::Expr(expr) => match expr.expr(ast) {
                Expr::Str(s) => {
                    let raw = s.raw(ast);
                    matches!(raw, Some(value) if value.as_str() == "\"use strict\"" || value.as_str() == "'use strict'")
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Returns true if the statement does not prevent the directives below
    /// `self` from being directives.
    pub fn can_precede_directive(self, ast: &Ast<'_>) -> bool {
        match self {
            Stmt::Expr(expr) => matches!(expr.expr(ast), Expr::Str(_)),
            _ => false,
        }
    }
}
