use crate::Allocator;
use swc_experimental_allocator::atom::{Atom, Wtf8Atom};
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ast_macros::ast;

use crate::{
    Span,
    ast::*,
    span::{GetSpan, SetSpan},
};

#[ast]
#[derive(Debug)]
pub enum Expr<'a> {
    This(Box<'a, ThisExpr>),
    Array(Box<'a, ArrayLit<'a>>),
    Object(Box<'a, ObjectLit<'a>>),
    Fn(Box<'a, FnExpr<'a>>),
    Unary(Box<'a, UnaryExpr<'a>>),
    Update(Box<'a, UpdateExpr<'a>>),
    Bin(Box<'a, BinExpr<'a>>),
    Assign(Box<'a, AssignExpr<'a>>),
    Member(Box<'a, MemberExpr<'a>>),
    SuperProp(Box<'a, SuperPropExpr<'a>>),
    Cond(Box<'a, CondExpr<'a>>),
    Call(Box<'a, CallExpr<'a>>),
    New(Box<'a, NewExpr<'a>>),
    Seq(Box<'a, SeqExpr<'a>>),
    Ident(Box<'a, Ident<'a>>),
    Lit(Box<'a, Lit<'a>>),
    Tpl(Box<'a, Tpl<'a>>),
    TaggedTpl(Box<'a, TaggedTpl<'a>>),
    Arrow(Box<'a, ArrowExpr<'a>>),
    Class(Box<'a, ClassExpr<'a>>),
    Yield(Box<'a, YieldExpr<'a>>),
    MetaProp(Box<'a, MetaPropExpr>),
    Await(Box<'a, AwaitExpr<'a>>),
    Paren(Box<'a, ParenExpr<'a>>),
    JSXMember(Box<'a, JSXMemberExpr<'a>>),
    JSXNamespacedName(Box<'a, JSXNamespacedName<'a>>),
    JSXEmpty(Box<'a, JSXEmptyExpr>),
    JSXElement(Box<'a, JSXElement<'a>>),
    JSXFragment(Box<'a, JSXFragment<'a>>),
    // TsTypeAssertion(TsTypeAssertion),
    // TsConstAssertion(TsConstAssertion),
    // TsNonNull(TsNonNullExpr),
    // TsAs(TsAsExpr),
    // TsInstantiation(TsInstantiation),
    // TsSatisfies(TsSatisfiesExpr),
    PrivateName(Box<'a, PrivateName<'a>>),
    OptChain(Box<'a, OptChainExpr<'a>>),
    Invalid(Box<'a, Invalid>),
}

#[ast]
#[derive(Debug)]
pub struct ThisExpr {
    pub span: Span,
}

#[ast]
#[derive(Debug)]
pub struct ArrayLit<'a> {
    pub span: Span,
    pub elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
}

#[ast]
#[derive(Debug)]
pub struct ObjectLit<'a> {
    pub span: Span,
    pub props: Vec<'a, PropOrSpread<'a>>,
}

#[ast]
#[derive(Debug)]
pub enum PropOrSpread<'a> {
    Spread(Box<'a, SpreadElement<'a>>),
    Prop(Box<'a, Prop<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct SpreadElement<'a> {
    #[span(lo)]
    pub dot3_token: Span,
    #[span(hi)]
    pub expr: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct UnaryExpr<'a> {
    pub span: Span,
    pub op: UnaryOp,
    pub arg: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct UpdateExpr<'a> {
    pub span: Span,
    pub op: UpdateOp,
    pub prefix: bool,
    pub arg: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct BinExpr<'a> {
    pub span: Span,
    pub op: BinaryOp,
    pub left: Expr<'a>,
    pub right: Expr<'a>,
}
#[ast]
#[derive(Debug)]
pub struct FnExpr<'a> {
    pub ident: Option<Box<'a, Ident<'a>>>,
    #[span]
    pub function: Box<'a, Function<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct ClassExpr<'a> {
    pub ident: Option<Box<'a, Ident<'a>>>,
    #[span]
    pub class: Box<'a, Class<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct AssignExpr<'a> {
    pub span: Span,
    pub op: AssignOp,
    pub left: AssignTarget<'a>,
    pub right: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct MemberExpr<'a> {
    pub span: Span,
    pub obj: Expr<'a>,
    pub prop: MemberProp<'a>,
}

#[ast]
#[derive(Debug)]
pub enum MemberProp<'a> {
    Ident(Box<'a, IdentName<'a>>),
    PrivateName(Box<'a, PrivateName<'a>>),
    Computed(Box<'a, ComputedPropName<'a>>),
}

impl MemberProp<'_> {
    pub fn is_ident_with(&self, sym: &str) -> bool {
        matches!(self, MemberProp::Ident(i) if i.sym == sym)
    }
}

#[ast]
#[derive(Debug)]
pub struct SuperPropExpr<'a> {
    pub span: Span,
    pub obj: Box<'a, Super>,
    pub prop: SuperProp<'a>,
}

#[ast]
#[derive(Debug)]
pub enum SuperProp<'a> {
    Ident(Box<'a, IdentName<'a>>),
    Computed(Box<'a, ComputedPropName<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct CondExpr<'a> {
    pub span: Span,
    pub test: Expr<'a>,
    pub cons: Expr<'a>,
    pub alt: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct CallExpr<'a> {
    pub span: Span,
    pub callee: Callee<'a>,
    pub args: Vec<'a, ExprOrSpread<'a>>,
    // type_args: Option<Box<TsTypeParamInstantiation>>,
    // pub type_params: Option<TsTypeParamInstantiation>,
}

#[ast]
#[derive(Debug)]
pub struct NewExpr<'a> {
    pub span: Span,
    pub callee: Expr<'a>,
    pub args: Option<Vec<'a, ExprOrSpread<'a>>>,
    // type_args: Option<Box<TsTypeParamInstantiation>>,
    // pub type_params: Option<TsTypeParamInstantiation>,
}

#[ast]
#[derive(Debug)]
pub struct SeqExpr<'a> {
    pub span: Span,
    pub exprs: Vec<'a, Expr<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct ArrowExpr<'a> {
    pub span: Span,
    pub params: Vec<'a, Pat<'a>>,
    pub body: BlockStmtOrExpr<'a>,
    pub is_async: bool,
    // type_params: Option<Box<TsTypeParamDecl>>,
    // return_type: Option<Box<TsTypeAnn>>,
}

#[ast]
#[derive(Debug)]
pub struct YieldExpr<'a> {
    pub span: Span,
    pub arg: Option<Expr<'a>>,
    pub delegate: bool,
}

#[ast]
#[derive(Debug)]
pub struct MetaPropExpr {
    pub span: Span,
    pub kind: MetaPropKind,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaPropKind {
    /// `new.target`
    NewTarget,
    /// `import.meta`
    ImportMeta,
}

#[ast]
#[derive(Debug)]
pub struct AwaitExpr<'a> {
    pub span: Span,
    pub arg: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct Tpl<'a> {
    pub span: Span,
    pub exprs: Vec<'a, Expr<'a>>,
    pub quasis: Vec<'a, TplElement<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct TaggedTpl<'a> {
    pub span: Span,
    pub tag: Expr<'a>,
    // type_params: Option<TsTypeParamInstantiation>,
    pub tpl: Box<'a, Tpl<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct TplElement<'a> {
    pub span: Span,
    pub tail: bool,
    pub cooked: Option<Wtf8Atom<'a>>,
    pub raw: Atom<'a>,
}

#[ast]
#[derive(Debug)]
pub struct ParenExpr<'a> {
    pub span: Span,
    pub expr: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub enum Callee<'a> {
    Super(Box<'a, Super>),
    Import(Box<'a, Import>),
    Expr(Box<'a, Expr<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct Super {
    pub span: Span,
}

#[ast]
#[derive(Debug)]
pub struct Import {
    pub span: Span,
    pub phase: ImportPhase,
}

#[ast(skip_span)]
#[derive(Debug)]
pub struct ExprOrSpread<'a> {
    pub spread: Option<Span>,
    pub expr: Expr<'a>,
}

impl GetSpan for ExprOrSpread<'_> {
    #[inline]
    fn span(&self) -> Span {
        let expr_span = self.expr.span();
        match self.spread {
            Some(spread) => Span::new(spread.start, expr_span.end),
            None => expr_span,
        }
    }
}

impl SetSpan for ExprOrSpread<'_> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        if let Some(spread) = &mut self.spread {
            spread.start = span.start;
            let expr_span = self.expr.span();
            self.expr.set_span(Span::new(expr_span.start, span.end));
        } else {
            self.expr.set_span(span);
        }
    }
}

#[ast]
#[derive(Debug)]
pub enum BlockStmtOrExpr<'a> {
    BlockStmt(Box<'a, BlockStmt<'a>>),
    Expr(Box<'a, Expr<'a>>),
}

#[ast]
#[derive(Debug)]
pub enum AssignTarget<'a> {
    Simple(Box<'a, SimpleAssignTarget<'a>>),
    Pat(Box<'a, AssignTargetPat<'a>>),
}

#[ast]
#[derive(Debug)]
pub enum AssignTargetPat<'a> {
    Array(Box<'a, ArrayPat<'a>>),
    Object(Box<'a, ObjectPat<'a>>),
    Invalid(Box<'a, Invalid>),
}

#[ast]
#[derive(Debug)]
pub enum SimpleAssignTarget<'a> {
    Ident(Box<'a, BindingIdent<'a>>),
    Member(Box<'a, MemberExpr<'a>>),
    SuperProp(Box<'a, SuperPropExpr<'a>>),
    Paren(Box<'a, ParenExpr<'a>>),
    OptChain(Box<'a, OptChainExpr<'a>>),
    // TsAs(TsAsExpr),
    // TsSatisfies(TsSatisfiesExpr),
    // TsNonNull(TsNonNullExpr),
    // TsTypeAssertion(TsTypeAssertion),
    // TsInstantiation(TsInstantiation),
    Invalid(Box<'a, Invalid>),
}

#[ast]
#[derive(Debug)]
pub struct OptChainExpr<'a> {
    pub span: Span,
    pub optional: bool,
    pub base: OptChainBase<'a>,
}

#[ast]
#[derive(Debug)]
pub enum OptChainBase<'a> {
    Member(Box<'a, MemberExpr<'a>>),
    Call(Box<'a, OptCall<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct OptCall<'a> {
    pub span: Span,
    pub callee: Expr<'a>,
    pub args: Vec<'a, ExprOrSpread<'a>>,
    // type_args: Option<Box<TsTypeParamInstantiation>>,
    // pub type_params: Option<TsTypeParamInstantiation>,
}

#[ast]
#[derive(Debug)]
pub struct Invalid {}

impl<'a> Expr<'a> {
    pub fn is_ident_ref_to<S>(&self, ident: &S) -> bool
    where
        S: ?Sized + AsRef<str>,
    {
        match self {
            Expr::Ident(i) => i.sym == ident.as_ref(),
            _ => false,
        }
    }
}

impl<'a> AssignTarget<'a> {
    pub fn try_from_pat(p: Pat<'a>, allocator: &'a Allocator) -> Result<Self, Pat<'a>> {
        Ok(match p {
            Pat::Array(a) => AssignTarget::Pat(allocator.boxed(AssignTargetPat::Array(a))),
            Pat::Object(o) => AssignTarget::Pat(allocator.boxed(AssignTargetPat::Object(o))),
            Pat::Ident(i) => AssignTarget::Simple(allocator.boxed(SimpleAssignTarget::Ident(i))),
            Pat::Invalid(i) => {
                AssignTarget::Simple(allocator.boxed(SimpleAssignTarget::Invalid(i)))
            }
            Pat::Expr(e) => match Self::try_from_expr(Box::into_inner(e), allocator) {
                Ok(v) => v,
                Err(e) => return Err(Pat::Expr(allocator.boxed(e))),
            },
            _ => return Err(p),
        })
    }
    pub fn try_from_expr(e: Expr<'a>, allocator: &'a Allocator) -> Result<Self, Expr<'a>> {
        Ok(Self::Simple(
            allocator.boxed(SimpleAssignTarget::try_from_expr(e, allocator)?),
        ))
    }
}

impl<'a> SimpleAssignTarget<'a> {
    pub fn try_from_expr(e: Expr<'a>, allocator: &'a Allocator) -> Result<Self, Expr<'a>> {
        Ok(match e {
            Expr::Ident(i) => SimpleAssignTarget::Ident(allocator.boxed(BindingIdent { id: i })),
            Expr::Member(m) => SimpleAssignTarget::Member(m),
            Expr::SuperProp(s) => SimpleAssignTarget::SuperProp(s),
            Expr::OptChain(s) => SimpleAssignTarget::OptChain(s),
            Expr::Paren(s) => SimpleAssignTarget::Paren(s),
            // Expr::TsAs(a) => SimpleAssignTarget::TsAs(a),
            // Expr::TsSatisfies(s) => SimpleAssignTarget::TsSatisfies(s),
            // Expr::TsNonNull(n) => SimpleAssignTarget::TsNonNull(n),
            // Expr::TsTypeAssertion(a) => SimpleAssignTarget::TsTypeAssertion(a),
            // Expr::TsInstantiation(a) => SimpleAssignTarget::TsInstantiation(a),
            _ => return Err(e),
        })
    }
}
