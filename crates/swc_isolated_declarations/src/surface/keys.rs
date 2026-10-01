//! Property identity and computed-key eligibility shared by analysis and emit.

use std::borrow::Cow;

use swc_next_ecma_ast::{
    Ast, Expr, ExprData, PropertyKey, PropertyKeyData, SymbolId, UnaryExpression, UnaryOperator,
};
use swc_next_ecma_semantic::Semantic;
use zmij_ecma::Buffer;

use super::expressions::unparenthesized;

/// Property identity is separate from its spelling for diagnostics. Static names
/// retain WTF-8 bytes; computed names retain their lexical binding identities.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PropertyIdentity<'ast> {
    Static(Cow<'ast, [u8]>),
    Binding(SymbolId),
    Unresolved(&'ast str),
    Member(Box<PropertyIdentity<'ast>>, Cow<'ast, [u8]>),
}

pub(crate) fn property_identity<'ast>(
    key: PropertyKey,
    ast: &Ast<'ast>,
    semantic: &Semantic<'_>,
) -> Option<PropertyIdentity<'ast>> {
    let expression = match ast.property_key_data(key) {
        PropertyKeyData::IdentifierName(identifier) => {
            return Some(PropertyIdentity::Static(Cow::Borrowed(
                ast.get_utf8(identifier.name(ast)).as_bytes(),
            )));
        }
        PropertyKeyData::Expr(expression) => unparenthesized(expression, ast),
    };
    match ast.expr_data(expression) {
        ExprData::IdentifierReference(identifier) => {
            let reference = semantic.reference_of(identifier.node_id())?;
            if let Some(symbol) = semantic.reference(reference).resolution().symbol() {
                Some(PropertyIdentity::Binding(symbol))
            } else if semantic.reference(reference).symbol.is_none() {
                Some(PropertyIdentity::Unresolved(
                    ast.get_utf8(identifier.name(ast)),
                ))
            } else {
                None
            }
        }
        ExprData::StringLiteral(literal) => Some(PropertyIdentity::Static(Cow::Borrowed(
            ast.get_wtf8(literal.value(ast)).as_bytes(),
        ))),
        ExprData::NumericLiteral(literal) => Some(numeric_property_identity(literal.value(ast))),
        ExprData::UnaryExpression(unary) => {
            signed_numeric_value(unary, ast).map(numeric_property_identity)
        }
        ExprData::TemplateLiteral(template) if template.expressions(ast).is_empty() => {
            let quasi = ast.get_node_in_sub_range(template.quasis(ast).first()?);
            if quasi.is_cooked_undefined(ast) {
                return None;
            }
            Some(PropertyIdentity::Static(Cow::Borrowed(
                ast.get_wtf8(quasi.cooked(ast)).as_bytes(),
            )))
        }
        ExprData::MemberExpression(member) if !member.optional(ast) => {
            let object = property_identity(PropertyKey::Expr(member.object(ast)), ast, semantic)?;
            let PropertyIdentity::Static(name) =
                property_identity(member.property(ast), ast, semantic)?
            else {
                return None;
            };
            Some(PropertyIdentity::Member(Box::new(object), name))
        }
        _ => None,
    }
}

pub(crate) fn signed_numeric_value(unary: UnaryExpression, ast: &Ast<'_>) -> Option<f64> {
    let ExprData::NumericLiteral(literal) =
        ast.expr_data(unparenthesized(unary.argument(ast), ast))
    else {
        return None;
    };
    match unary.operator(ast) {
        UnaryOperator::Negate => Some(-literal.value(ast)),
        UnaryOperator::Positive => Some(literal.value(ast)),
        _ => None,
    }
}

fn numeric_property_identity(value: f64) -> PropertyIdentity<'static> {
    let mut buffer = Buffer::new();
    PropertyIdentity::Static(Cow::Owned(buffer.format(value).as_bytes().to_vec()))
}

/// Object inference requires a literal name; classes can retain entity-name keys.
pub(crate) fn supported_object_key(key: PropertyKey, ast: &Ast<'_>) -> bool {
    let PropertyKeyData::Expr(expression) = ast.property_key_data(key) else {
        return true;
    };
    match ast.expr_data(unparenthesized(expression, ast)) {
        ExprData::StringLiteral(_) | ExprData::NumericLiteral(_) => true,
        ExprData::TemplateLiteral(template) => template.expressions(ast).is_empty(),
        ExprData::UnaryExpression(unary) => {
            matches!(
                unary.operator(ast),
                UnaryOperator::Negate | UnaryOperator::Positive
            ) && matches!(
                ast.expr_data(unparenthesized(unary.argument(ast), ast)),
                ExprData::NumericLiteral(_)
            )
        }
        _ => false,
    }
}

pub(crate) fn supported_computed_key(
    key: PropertyKey,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
) -> bool {
    match ast.property_key_data(key) {
        PropertyKeyData::IdentifierName(_) => true,
        PropertyKeyData::Expr(expression) => is_computed_name_expression(expression, ast, semantic),
    }
}
fn is_computed_name_expression(expression: Expr, ast: &Ast<'_>, semantic: &Semantic<'_>) -> bool {
    match ast.expr_data(unparenthesized(expression, ast)) {
        ExprData::IdentifierReference(_)
        | ExprData::StringLiteral(_)
        | ExprData::NumericLiteral(_) => true,
        ExprData::BigIntLiteral(_) => false,
        ExprData::TemplateLiteral(template) => template.expressions(ast).is_empty(),
        ExprData::UnaryExpression(unary) => {
            matches!(
                unary.operator(ast),
                UnaryOperator::Negate | UnaryOperator::Positive
            ) && matches!(
                ast.expr_data(unparenthesized(unary.argument(ast), ast)),
                ExprData::NumericLiteral(_)
            )
        }
        ExprData::MemberExpression(member) => {
            if member.computed(ast) || member.optional(ast) {
                return false;
            }
            if let Some(allowed) = well_known_symbol_object(member.object(ast), ast, semantic) {
                return allowed;
            }
            is_computed_name_expression(member.object(ast), ast, semantic)
        }
        ExprData::ParenthesizedExpression(_) => false,
        _ => false,
    }
}

fn well_known_symbol_object(object: Expr, ast: &Ast<'_>, semantic: &Semantic<'_>) -> Option<bool> {
    match ast.expr_data(unparenthesized(object, ast)) {
        ExprData::IdentifierReference(identifier)
            if ast.get_utf8(identifier.name(ast)) == "Symbol" =>
        {
            Some(
                semantic
                    .reference_of(identifier.node_id())
                    .is_some_and(|reference| semantic.reference(reference).symbol.is_none()),
            )
        }
        ExprData::MemberExpression(member)
            if !member.computed(ast)
                && !member.optional(ast)
                && matches!(property_identity(member.property(ast), ast, semantic), Some(PropertyIdentity::Static(name)) if name.as_ref() == b"Symbol") =>
        {
            let ExprData::IdentifierReference(identifier) =
                ast.expr_data(unparenthesized(member.object(ast), ast))
            else {
                return Some(false);
            };
            if ast.get_utf8(identifier.name(ast)) != "globalThis" {
                return Some(false);
            }
            Some(
                semantic
                    .reference_of(identifier.node_id())
                    .is_some_and(|reference| semantic.reference(reference).symbol.is_none()),
            )
        }
        _ => None,
    }
}
