use swc_next_ast_macros::ast;

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

#[ast(repr, nested)]
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

/// A JSX element, possibly self-closing.
///
/// See: [JSX Specification JSXElement](https://facebook.github.io/jsx/#prod-JSXElement)
///
/// ## Example
/// ```jsx
/// <Foo bar="baz">hello</Foo>
/// // ^^^^^^^^^^^^^^^ opening_element
/// //                 ^^^^^ children
/// //                      ^^^^^^ closing_element
/// ```
#[ast]
pub struct JsxElement {
    /// `jsx_opening_element`
    opening_element: JsxOpeningElement,
    /// `jsx_text`, `jsx_expression_container`, `jsx_spread_child`, `jsx_element`, or `jsx_fragment`
    children: Vec<JsxChild>,
    /// `jsx_closing_element`. `None` for self-closing tags like `<Foo />`.
    closing_element: Option<JsxClosingElement>,
}

/// The opening `<Foo ...>` of a JSX element.
///
/// ## Example
/// ```tsx
/// <Foo<T> bar={baz} />
/// //  ^^^ type_arguments
/// //      ^^^^^^^^^ attributes
/// ```
///
/// `name` is the tag (here `Foo`). `self_closing` is true for the
/// trailing `/>`.
#[ast]
pub struct JsxOpeningElement {
    /// `jsx_identifier`, `jsx_namespaced_name`, or `jsx_member_expression`
    name: JsxElementName,
    /// `ts_type_parameter_instantiation`. `None` when absent.
    type_arguments: Option<TsTypeParameterInstantiation>,
    /// `jsx_attribute` or `jsx_spread_attribute`.
    attributes: Vec<JsxAttributeItem>,
    self_closing: bool,
}

/// The closing `</Foo>` of a JSX element.
///
/// ## Example
/// ```jsx
/// </Foo>
/// // ^^^ name
/// ```
#[ast]
pub struct JsxClosingElement {
    /// `jsx_identifier`, `jsx_namespaced_name`, or `jsx_member_expression`
    name: JsxElementName,
}

/// A JSX fragment `<>...</>`.
///
/// See: [JSX Specification JSXFragment](https://facebook.github.io/jsx/#prod-JSXFragment)
///
/// ## Example
/// ```jsx
/// <>hello</>
/// //^^^^^ children
/// ```
///
/// `opening_fragment` is `<>`, `closing_fragment` is `</>`.
#[ast]
pub struct JsxFragment {
    /// `jsx_opening_fragment`
    opening_fragment: JsxOpeningFragment,
    /// `jsx_text`, `jsx_expression_container`, `jsx_spread_child`, `jsx_element`, or `jsx_fragment`
    children: Vec<JsxChild>,
    /// `jsx_closing_fragment`
    closing_fragment: JsxClosingFragment,
}

/// The opening `<>` of a JSX fragment.
#[ast]
pub struct JsxOpeningFragment {}

/// The closing `</>` of a JSX fragment.
#[ast]
pub struct JsxClosingFragment {}

/// An identifier used as a JSX tag name or attribute name.
///
/// ## Example
/// ```jsx
/// <Foo bar="baz" />
/// //   ^^^ JSXIdentifier (attribute name)
/// ```
///
/// The tag `Foo` is also a `jsx_identifier`.
#[ast]
pub struct JsxIdentifier {
    name: AstUtf8,
}

/// A JSX name of the form `namespace:name`.
///
/// ## Example
/// ```jsx
/// <svg:path />
/// // ^^^ namespace
/// //     ^^^^ name
/// ```
#[ast]
pub struct JsxNamespacedName {
    /// `jsx_identifier`
    namespace: JsxIdentifier,
    /// `jsx_identifier`
    name: JsxIdentifier,
}

/// A dotted JSX tag name.
///
/// ## Example
/// ```jsx
///   <Foo.Bar.Baz />
/// // ^^^^^^^ object
/// //         ^^^ property
/// ```
#[ast]
pub struct JsxMemberExpression {
    /// `jsx_identifier` or a nested `jsx_member_expression`.
    object: JsxMemberExpressionObject,
    /// `jsx_identifier`
    property: JsxIdentifier,
}

/// A single JSX attribute.
///
/// ## Example
/// ```jsx
/// <Foo bar="baz" disabled />
/// //   ^^^ name
/// //       ^^^^^ value
/// //             ^^^^^^^^ boolean attribute (value = None)
/// ```
#[ast]
pub struct JsxAttribute {
    /// `jsx_identifier` or `jsx_namespaced_name`.
    name: JsxAttributeName,
    /// `string_literal`, `jsx_expression_container`, `jsx_element`, or
    /// `jsx_fragment`. `None` for boolean-like attributes.
    value: Option<JsxAttributeValue>,
}

/// A spread attribute `{...props}`.
///
/// ## Example
/// ```jsx
/// <Foo {...props} />
/// //    ^^^^^^^^ argument
/// ```
#[ast]
pub struct JsxSpreadAttribute {
    /// any expression
    argument: Expr,
}

/// A `{expression}` container inside JSX.
///
/// ## Example
/// ```jsx
/// <Foo bar={baz}>{children}</Foo>
/// //       ^^^^^ attribute container
/// //             ^^^^^^^^^^ child container
/// ```
#[ast]
pub struct JsxExpressionContainer {
    /// any expression. `jsx_empty_expression` for `{}`.
    expression: JsxExpression,
}

/// The empty `{}` placeholder inside a JSX element.
#[ast]
pub struct JsxEmptyExpression {}

/// A span of raw text inside a JSX element or fragment.
///
/// ## Example
/// ```jsx
/// <Foo>hello world</Foo>
/// //   ^^^^^^^^^^^ value
/// ```
#[ast]
pub struct JsxText {
    value: AstUtf8,
}

/// A spread child `{...children}` inside a JSX element.
///
/// ## Example
/// ```jsx
/// <Foo>{...kids}</Foo>
/// //    ^^^^^^^ expression
/// ```
#[ast]
pub struct JsxSpreadChild {
    /// any expression
    expression: Expr,
}
