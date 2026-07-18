use swc_experimental_ast_macros::ast;

use crate::ast::*;

#[ast(repr)]
pub enum Pat {
    // Expr
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
}

#[allow(non_snake_case)]
impl Pat {
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
            Self::Ident(_)
            | Self::Array(_)
            | Self::Rest(_)
            | Self::Object(_)
            | Self::Assign(_)
            | Self::Invalid(_) => Err(self),
            _ => unsafe { Ok(std::mem::transmute(self)) },
        }
    }
}

#[ast]
pub struct ArrayPat {
    elems: Vec<Option<Pat>>,
    optional: bool,
    // type_ann: Option<TsTypeAnn>,
}

#[ast]
pub struct ObjectPat {
    pub props: Vec<ObjectPatProp>,
    pub optional: bool,
    // pub type_ann: Option<Box<TsTypeAnn>>,
}

#[ast]
pub struct AssignPat {
    pub left: Pat,
    pub right: Expr,
}

#[ast]
pub struct RestPat {
    dot3_token: Span,
    arg: Pat,
    // type_ann: Option<Box<TsTypeAnn>>,
}

#[ast(repr)]
pub enum ObjectPatProp {
    KeyValue(KeyValuePatProp),
    Assign(AssignPatProp),
    Rest(RestPat),
}

#[ast]
pub struct KeyValuePatProp {
    pub key: PropName,
    pub value: Pat,
}
#[ast]
pub struct AssignPatProp {
    pub key: BindingIdent,
    pub value: Option<Expr>,
}
