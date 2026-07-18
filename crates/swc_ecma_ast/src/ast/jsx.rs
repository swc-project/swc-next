use swc_experimental_ast_macros::ast;

use crate::ast::*;

#[ast(repr)]
pub enum JSXObject {
    JSXMemberExpr(JSXMemberExpr),
    Ident(Ident),
}

#[ast]
pub struct JSXMemberExpr {
    obj: JSXObject,
    prop: IdentName,
}

#[ast]
pub struct JSXNamespacedName {
    ns: IdentName,
    name: IdentName,
}

#[ast]
pub struct JSXEmptyExpr {}

#[ast]
pub struct JSXExprContainer {
    expr: JSXExpr,
}

#[ast(repr)]
pub enum JSXExpr {
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

    // JSXExpr
    JSXEmptyExpr(JSXEmptyExpr),
}

#[allow(non_snake_case)]
impl JSXExpr {
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
            Self::JSXEmptyExpr(_) => Err(self),
            _ => unsafe { Ok(std::mem::transmute(self)) },
        }
    }
}

#[ast]
pub struct JSXSpreadChild {
    expr: Expr,
}

#[ast(repr)]
pub enum JSXElementName {
    Ident(Ident),
    JSXMemberExpr(JSXMemberExpr),
    JSXNamespacedName(JSXNamespacedName),
}

#[ast]
pub struct JSXOpeningElement {
    name: JSXElementName,
    attrs: Vec<JSXAttrOrSpread>,
    self_closing: bool,
    // type_args: Option<Box<TsTypeParamInstantiation>>,
}

#[ast(repr)]
pub enum JSXAttrOrSpread {
    JSXAttr(JSXAttr),
    SpreadElement(SpreadElement),
}

#[ast]
pub struct JSXClosingElement {
    name: JSXElementName,
}

#[ast]
pub struct JSXAttr {
    name: JSXAttrName,
    value: Option<JSXAttrValue>,
}

#[ast(repr)]
pub enum JSXAttrName {
    Ident(IdentName),
    JSXNamespacedName(JSXNamespacedName),
}

#[ast(repr)]
pub enum JSXAttrValue {
    Str(Str),
    JSXExprContainer(JSXExprContainer),
    JSXElement(JSXElement),
    JSXFragment(JSXFragment),
}

#[ast]
pub struct JSXText {
    value: Atom,
    raw: Atom,
}

#[ast]
pub struct JSXElement {
    opening: JSXOpeningElement,
    children: Vec<JSXElementChild>,
    closing: Option<JSXClosingElement>,
}

#[ast(repr)]
pub enum JSXElementChild {
    JSXText(JSXText),
    JSXExprContainer(JSXExprContainer),
    JSXSpreadChild(JSXSpreadChild),
    JSXElement(JSXElement),
    JSXFragment(JSXFragment),
}

#[ast]
pub struct JSXFragment {
    opening: JSXOpeningFragment,
    children: Vec<JSXElementChild>,
    closing: JSXClosingFragment,
}

#[ast]
pub struct JSXOpeningFragment {}

#[ast]
pub struct JSXClosingFragment {}
