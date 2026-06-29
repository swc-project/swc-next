use crate::Span;
use swc_experimental_allocator::atom::Atom;
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ast_macros::ast;

use crate::{Expr, Ident, IdentName, SpreadElement, Str};

#[ast]
#[derive(Debug)]
pub enum JSXObject<'a> {
    JSXMemberExpr(Box<'a, JSXMemberExpr<'a>>),
    Ident(Box<'a, Ident<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct JSXMemberExpr<'a> {
    pub span: Span,
    pub obj: JSXObject<'a>,
    pub prop: Box<'a, IdentName<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct JSXNamespacedName<'a> {
    pub span: Span,
    pub ns: Box<'a, IdentName<'a>>,
    pub name: Box<'a, IdentName<'a>>,
}

#[ast]
#[derive(Debug, Clone)]
pub struct JSXEmptyExpr {
    pub span: Span,
}

#[ast]
#[derive(Debug)]
pub struct JSXExprContainer<'a> {
    pub span: Span,
    pub expr: JSXExpr<'a>,
}

#[ast]
#[derive(Debug)]
pub enum JSXExpr<'a> {
    JSXEmptyExpr(Box<'a, JSXEmptyExpr>),
    Expr(Box<'a, Expr<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct JSXSpreadChild<'a> {
    pub span: Span,
    pub expr: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub enum JSXElementName<'a> {
    Ident(Box<'a, Ident<'a>>),
    JSXMemberExpr(Box<'a, JSXMemberExpr<'a>>),
    JSXNamespacedName(Box<'a, JSXNamespacedName<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct JSXOpeningElement<'a> {
    pub span: Span,
    pub name: JSXElementName<'a>,
    pub attrs: Vec<'a, JSXAttrOrSpread<'a>>,
    pub self_closing: bool,
    // type_args: Option<Box<TsTypeParamInstantiation>>,
}

#[ast]
#[derive(Debug)]
pub enum JSXAttrOrSpread<'a> {
    JSXAttr(Box<'a, JSXAttr<'a>>),
    SpreadElement(Box<'a, SpreadElement<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct JSXClosingElement<'a> {
    pub span: Span,
    pub name: JSXElementName<'a>,
}

#[ast]
#[derive(Debug)]
pub struct JSXAttr<'a> {
    pub span: Span,
    pub name: JSXAttrName<'a>,
    pub value: Option<JSXAttrValue<'a>>,
}

#[ast]
#[derive(Debug)]
pub enum JSXAttrName<'a> {
    Ident(Box<'a, IdentName<'a>>),
    JSXNamespacedName(Box<'a, JSXNamespacedName<'a>>),
}

#[ast]
#[derive(Debug)]
pub enum JSXAttrValue<'a> {
    Str(Box<'a, Str<'a>>),
    JSXExprContainer(Box<'a, JSXExprContainer<'a>>),
    JSXElement(Box<'a, JSXElement<'a>>),
    JSXFragment(Box<'a, JSXFragment<'a>>),
}

#[ast]
#[derive(Debug, Clone)]
pub struct JSXText<'a> {
    pub span: Span,
    pub value: Atom<'a>,
    pub raw: Atom<'a>,
}

#[ast]
#[derive(Debug)]
pub struct JSXElement<'a> {
    pub span: Span,
    pub opening: Box<'a, JSXOpeningElement<'a>>,
    pub children: Vec<'a, JSXElementChild<'a>>,
    pub closing: Option<Box<'a, JSXClosingElement<'a>>>,
}

#[ast]
#[derive(Debug)]
pub enum JSXElementChild<'a> {
    JSXText(Box<'a, JSXText<'a>>),
    JSXExprContainer(Box<'a, JSXExprContainer<'a>>),
    JSXSpreadChild(Box<'a, JSXSpreadChild<'a>>),
    JSXElement(Box<'a, JSXElement<'a>>),
    JSXFragment(Box<'a, JSXFragment<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct JSXFragment<'a> {
    pub span: Span,
    pub opening: Box<'a, JSXOpeningFragment>,
    pub children: Vec<'a, JSXElementChild<'a>>,
    pub closing: Box<'a, JSXClosingFragment>,
}

#[ast]
#[derive(Debug, Clone)]
pub struct JSXOpeningFragment {
    pub span: Span,
}

#[ast]
#[derive(Debug, Clone)]
pub struct JSXClosingFragment {
    pub span: Span,
}
