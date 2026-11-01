//! Side-effect analysis of expressions.

use swc_next_ecma_ast::{
    ArgumentData, Ast, BinaryOperator, Expr, ExprData, ObjectPropertyKindData, PropertyKeyData,
    UnaryOperator,
};

use super::value::Value;

/// Returns whether evaluating `expr` may be observable: it may throw or run
/// code that reads or writes state.
///
/// The analysis is syntactic and conservative. Identifier references and
/// `this` may throw (unresolved names and temporal dead zones), and
/// conversions of operands that are not constant may call user code, so
/// they count as side effects. Creating functions, objects, and arrays from
/// side-effect-free parts does not.
///
/// The cost is linear in the size of `expr`.
pub(super) fn may_have_side_effects(ast: &Ast<'_>, expr: Expr) -> bool {
    match ast.expr_data(expr) {
        ExprData::StringLiteral(_)
        | ExprData::NumericLiteral(_)
        | ExprData::BigIntLiteral(_)
        | ExprData::BooleanLiteral(_)
        | ExprData::NullLiteral(_)
        | ExprData::RegExpLiteral(_)
        | ExprData::Function(_)
        | ExprData::ArrowFunctionExpression(_) => false,
        ExprData::ParenthesizedExpression(parenthesized) => {
            may_have_side_effects(ast, parenthesized.expression(ast))
        }
        ExprData::SequenceExpression(sequence) => sequence
            .expressions(ast)
            .iter()
            .any(|id| may_have_side_effects(ast, ast.get_node_in_sub_range(id))),
        // `ToString` of a constant cannot call user code or throw.
        ExprData::TemplateLiteral(template) => template
            .expressions(ast)
            .iter()
            .any(|id| Value::of(ast, ast.get_node_in_sub_range(id)).is_none()),
        ExprData::UnaryExpression(unary) => match unary.operator(ast) {
            UnaryOperator::Void | UnaryOperator::LogicalNot | UnaryOperator::Typeof => {
                may_have_side_effects(ast, unary.argument(ast))
            }
            // Numeric conversion can call `valueOf` or throw.
            UnaryOperator::Negate | UnaryOperator::Positive | UnaryOperator::BitwiseNot => {
                Value::of(ast, expr).is_none()
            }
            UnaryOperator::Delete => true,
        },
        ExprData::BinaryExpression(binary) => match binary.operator(ast) {
            BinaryOperator::StrictEqual | BinaryOperator::StrictNotEqual => {
                may_have_side_effects(ast, binary.left(ast))
                    || may_have_side_effects(ast, binary.right(ast))
            }
            // Other operators convert their operands.
            _ => Value::of(ast, expr).is_none(),
        },
        ExprData::LogicalExpression(logical) => {
            may_have_side_effects(ast, logical.left(ast))
                || may_have_side_effects(ast, logical.right(ast))
        }
        ExprData::ConditionalExpression(conditional) => {
            may_have_side_effects(ast, conditional.test(ast))
                || may_have_side_effects(ast, conditional.consequent(ast))
                || may_have_side_effects(ast, conditional.alternate(ast))
        }
        ExprData::ArrayExpression(array) => array.elements(ast).iter().any(|id| {
            ast.get_node_in_sub_range(id)
                .is_some_and(|element| match ast.argument_data(element) {
                    ArgumentData::Expr(element) => may_have_side_effects(ast, element),
                    // Spread runs the iterator protocol.
                    ArgumentData::SpreadElement(_) => true,
                })
        }),
        ExprData::ObjectExpression(object) => object.properties(ast).iter().any(|id| {
            match ast.object_property_kind_data(ast.get_node_in_sub_range(id)) {
                ObjectPropertyKindData::ObjectProperty(property) => {
                    // A computed key that is not constant may call `toString`.
                    let key_has_effects = match ast.property_key_data(property.key(ast)) {
                        PropertyKeyData::IdentifierName(_) => false,
                        PropertyKeyData::Expr(key) => Value::of(ast, key).is_none(),
                    };
                    key_has_effects || may_have_side_effects(ast, property.value(ast))
                }
                // Spread runs getters.
                ObjectPropertyKindData::SpreadElement(_) => true,
            }
        }),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;

    use super::may_have_side_effects;
    use crate::compress::tests::{expression, parse};

    fn effects(source: &str) -> bool {
        let allocator = Allocator::new();
        let ast = parse(&allocator, source);
        may_have_side_effects(&ast, expression(&ast))
    }

    #[test]
    fn primitives() {
        assert!(!effects("(1, 'a', 1n, null, /a/)"));

        assert!(!effects("`a${1}${void 0}`"));

        assert!(effects("`${a}`"));

        assert!(!effects("void (!0, typeof 'a')"));

        assert!(!effects("-1 / 0"));

        assert!(effects("-a"));

        assert!(effects("1 + a"));

        assert!(effects("1 + 2 + 3"));

        assert!(!effects("1 === '' !== null"));

        assert!(effects("a === 1"));

        assert!(effects("delete 0"));

        assert!(!effects("0 ? 1 : 2 && null"));
    }

    #[test]
    fn references() {
        assert!(effects("x"));

        assert!(effects("this"));

        assert!(effects("f()"));

        assert!(effects("a.b"));
    }

    #[test]
    fn functions_and_classes() {
        assert!(!effects("(function () { run(); })"));

        assert!(!effects("() => run()"));

        assert!(effects("(class {})"));
    }

    #[test]
    fn literals() {
        assert!(!effects("[1, , []]"));

        assert!(effects("[...a]"));

        assert!(!effects(
            "({ a: 1, 'b': [], 1: 2, ['c']: 3, get d() {}, e() {} })"
        ));

        assert!(effects("({ [k]: 1 })"));

        assert!(effects("({ a })"));

        assert!(effects("({ ...a })"));
    }
}
