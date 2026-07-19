use swc_experimental_ast_macros::ast;

use super::js::*;

#[ast(repr)]
pub enum JsxElementName {
    JsxIdentifier(JsxIdentifier),
    JsxNamespacedName(JsxNamespacedName),
    JsxMemberExpression(JsxMemberExpression),
}

#[ast(repr)]
pub enum JsxMemberExpressionObject {
    JsxIdentifier(JsxIdentifier),
    JsxMemberExpression(JsxMemberExpression),
}

#[ast(repr)]
pub enum JsxAttributeName {
    JsxIdentifier(JsxIdentifier),
    JsxNamespacedName(JsxNamespacedName),
}

#[ast(repr)]
pub enum JsxAttributeItem {
    JsxAttribute(JsxAttribute),
    JsxSpreadAttribute(JsxSpreadAttribute),
}

#[ast(repr)]
pub enum JsxAttributeValue {
    StringLiteral(StringLiteral),
    JsxExpressionContainer(JsxExpressionContainer),
    JsxElement(JsxElement),
    JsxFragment(JsxFragment),
}

#[ast(repr)]
pub enum JsxExpression {
    Expr(Expr),
    JsxEmptyExpression(JsxEmptyExpression),
}

#[ast(repr)]
pub enum JsxChild {
    JsxText(JsxText),
    JsxExpressionContainer(JsxExpressionContainer),
    JsxSpreadChild(JsxSpreadChild),
    JsxElement(JsxElement),
    JsxFragment(JsxFragment),
}

#[ast]
pub struct JsxElement {
    opening_element: JsxOpeningElement,
    children: Vec<JsxChild>,
    closing_element: Option<JsxClosingElement>,
}

#[ast]
pub struct JsxOpeningElement {
    name: JsxElementName,
    attributes: Vec<JsxAttributeItem>,
    self_closing: bool,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[ast]
pub struct JsxClosingElement {
    name: JsxElementName,
}

#[ast]
pub struct JsxFragment {
    opening_fragment: JsxOpeningFragment,
    children: Vec<JsxChild>,
    closing_fragment: JsxClosingFragment,
}

#[ast]
pub struct JsxOpeningFragment {}

#[ast]
pub struct JsxClosingFragment {}

#[ast]
pub struct JsxIdentifier {
    name: Atom,
}

#[ast]
pub struct JsxNamespacedName {
    namespace: JsxIdentifier,
    name: JsxIdentifier,
}

#[ast]
pub struct JsxMemberExpression {
    object: JsxMemberExpressionObject,
    property: JsxIdentifier,
}

#[ast]
pub struct JsxAttribute {
    name: JsxAttributeName,
    value: Option<JsxAttributeValue>,
}

#[ast]
pub struct JsxSpreadAttribute {
    argument: Expr,
}

#[ast]
pub struct JsxExpressionContainer {
    expression: JsxExpression,
}

#[ast]
pub struct JsxEmptyExpression {}

#[ast]
pub struct JsxText {
    value: Wtf8Atom,
}

#[ast]
pub struct JsxSpreadChild {
    expression: Expr,
}
