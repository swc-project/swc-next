//! Shortening of property names.

use swc_next_ecma_ast::{
    Ast, ExprData, MemberExpression, ObjectProperty, PropertyKey, PropertyKeyData,
};

use super::value::skip_parentheses;

/// Returns the identifier name that `key` spells, when `key` is a string
/// literal.
///
/// Only ASCII names are converted. Reserved words are valid property names.
fn identifier_name(ast: &Ast<'_>, key: PropertyKey) -> Option<String> {
    let PropertyKeyData::Expr(expr) = ast.property_key_data(key) else {
        return None;
    };
    let ExprData::StringLiteral(literal) = ast.expr_data(expr) else {
        return None;
    };
    let name = ast.get_wtf8(literal.value(ast)).as_str()?;
    let mut bytes = name.bytes();
    let first = bytes.next()?;
    let valid = (first.is_ascii_alphabetic() || matches!(first, b'_' | b'$'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'));
    valid.then(|| name.to_owned())
}

/// Rewrites `a["b"]` to `a.b`.
pub(super) fn member(ast: &mut Ast<'_>, node: MemberExpression) {
    if !node.computed(ast) {
        return;
    }
    // `1.b` does not parse.
    if matches!(
        ast.expr_data(skip_parentheses(ast, node.object(ast))),
        ExprData::NumericLiteral(_)
    ) {
        return;
    }
    let key = node.property(ast);
    let Some(name) = identifier_name(ast, key) else {
        return;
    };
    let span = ast.span(key.node_id());
    let name = ast.add_utf8(&name);
    let property = ast.identifier_name(span, name);
    node.set_property(ast, PropertyKey::IdentifierName(property));
    node.set_computed(ast, false);
}

/// Rewrites the key of `{ "a": b }` to `a`.
pub(super) fn object_property(ast: &mut Ast<'_>, node: ObjectProperty) {
    if node.computed(ast) {
        return;
    }
    let key = node.key(ast);
    let Some(name) = identifier_name(ast, key) else {
        return;
    };
    let span = ast.span(key.node_id());
    let name = ast.add_utf8(&name);
    let key = ast.identifier_name(span, name);
    node.set_key(ast, PropertyKey::IdentifierName(key));
}
