use swc_next_ecma_ast::*;

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1472>.
pub(super) fn string(ast: &mut Ast<'_>, span: Span, value: &str) -> Expr {
    let value = ast.add_utf8(value);
    ast.expr_string_literal(span, value.into(), AstUtf8::EMPTY)
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1506>.
pub(super) fn member(ast: &mut Ast<'_>, object: Expr, property: &str, span: Span) -> Expr {
    let name = ast.add_utf8(property);
    let property = ast.identifier_name(span, name);
    ast.expr_member_expression(
        span,
        object,
        PropertyKey::IdentifierName(property),
        false,
        false,
    )
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L499>.
pub(super) fn property(ast: &mut Ast<'_>, name: &str, value: Expr) -> ObjectPropertyKind {
    let name = ast.add_utf8(name);
    let key = PropertyKey::IdentifierName(ast.identifier_name(DUMMY_SP, name));
    key_value(ast, key, value)
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L840>.
pub(super) fn key_value(ast: &mut Ast<'_>, key: PropertyKey, value: Expr) -> ObjectPropertyKind {
    ObjectPropertyKind::ObjectProperty(ast.object_property(
        DUMMY_SP,
        key,
        value,
        PropertyKind::Init,
        false,
        false,
        false,
    ))
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L628>.
pub(super) fn object(ast: &mut Ast<'_>, properties: Vec<ObjectPropertyKind>) -> Expr {
    let properties = ast.add_sub_range(properties);
    ast.expr_object_expression(DUMMY_SP, properties)
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L657>.
pub(super) fn array(ast: &mut Ast<'_>, arguments: Vec<Argument>) -> Expr {
    let elements = arguments.into_iter().map(Some);
    let elements = ast.add_sub_range(elements);
    ast.expr_array_expression(DUMMY_SP, elements)
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L692>.
pub(super) fn call(ast: &mut Ast<'_>, span: Span, callee: Expr, arguments: Vec<Argument>) -> Expr {
    let arguments = ast.add_sub_range(arguments);
    ast.expr_call_expression(span, callee, None, arguments, false, true)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_ast/src/expr.rs#L179>.
pub(super) fn undefined(ast: &mut Ast<'_>) -> Expr {
    let zero = ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
    ast.expr_unary_expression(DUMMY_SP, zero, UnaryOperator::Void)
}
