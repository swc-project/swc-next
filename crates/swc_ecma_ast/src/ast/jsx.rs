use swc_experimental_ast_macros::ast;

use crate::{Expr, Ident, IdentName, SpreadElement, Str};

#[ast]
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

#[ast]
pub enum JSXExpr {
    JSXEmptyExpr(JSXEmptyExpr),
    Expr(Expr),
}

#[ast]
pub struct JSXSpreadChild {
    expr: Expr,
}

#[ast]
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

#[ast]
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

#[ast]
pub enum JSXAttrName {
    Ident(IdentName),
    JSXNamespacedName(JSXNamespacedName),
}

#[ast]
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

#[ast]
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
