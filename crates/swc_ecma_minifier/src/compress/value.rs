//! Compile-time values of constant expressions.

use swc_next_allocator::{
    CloneIn,
    wtf8::{Wtf8, Wtf8Buf},
};
use swc_next_ecma_ast::{Ast, BinaryOperator, Expr, ExprData, UnaryOperator};

/// A primitive value known without running code.
///
/// BigInt values are tracked by type only; operations that need their value
/// are not evaluated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Value<'a> {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(&'a Wtf8),
    BigInt,
}

impl<'a> Value<'a> {
    /// Returns the value of a constant expression.
    ///
    /// Constant expressions are literals, operators applied to literals
    /// (`!0`, `void 0`, `-1`, `typeof ""`), and binary operators applied to
    /// those (`1 / 0`, `-1 / 0`). Evaluating them has no side effects.
    ///
    /// Deeper operands are not evaluated, so each call takes constant time.
    /// The compressor folds children before their parents, so a foldable
    /// operand is already constant when its parent is inspected; recursive
    /// evaluation would instead make long operator chains quadratic.
    pub(super) fn of(ast: &Ast<'a>, expr: Expr) -> Option<Self> {
        let expr = skip_parentheses(ast, expr);
        match ast.expr_data(expr) {
            ExprData::BinaryExpression(binary) => Self::binary(
                ast,
                binary.operator(ast),
                Self::operand(ast, binary.left(ast))?,
                Self::operand(ast, binary.right(ast))?,
            ),
            _ => Self::operand(ast, expr),
        }
    }

    fn operand(ast: &Ast<'a>, expr: Expr) -> Option<Self> {
        let expr = skip_parentheses(ast, expr);
        match ast.expr_data(expr) {
            ExprData::UnaryExpression(unary) => Self::unary(
                unary.operator(ast),
                Self::literal(ast, unary.argument(ast))?,
            ),
            _ => Self::literal(ast, expr),
        }
    }

    fn literal(ast: &Ast<'a>, expr: Expr) -> Option<Self> {
        let expr = skip_parentheses(ast, expr);
        Some(match ast.expr_data(expr) {
            ExprData::StringLiteral(literal) => Self::String(ast.get_wtf8(literal.value(ast))),
            ExprData::NumericLiteral(literal) => Self::Number(literal.value(ast)),
            ExprData::BigIntLiteral(_) => Self::BigInt,
            ExprData::BooleanLiteral(literal) => Self::Boolean(literal.value(ast)),
            ExprData::NullLiteral(_) => Self::Null,
            ExprData::TemplateLiteral(template) if template.expressions(ast).is_empty() => {
                let quasi = ast.get_node_in_sub_range(template.quasis(ast).iter().next()?);
                if quasi.is_cooked_undefined(ast) {
                    return None;
                }
                Self::String(ast.get_wtf8(quasi.cooked(ast)))
            }
            _ => return None,
        })
    }

    /// Applies a unary operator. Returns `None` for `delete` and when the
    /// result is unknown or the operation throws.
    pub(super) fn unary(operator: UnaryOperator, value: Self) -> Option<Self> {
        Some(match operator {
            UnaryOperator::Void => Self::Undefined,
            UnaryOperator::LogicalNot => Self::Boolean(!value.to_boolean()?),
            UnaryOperator::Typeof => Self::String(Wtf8::from_str(value.type_of())),
            UnaryOperator::Negate | UnaryOperator::BitwiseNot if value == Self::BigInt => {
                Self::BigInt
            }
            UnaryOperator::Negate => Self::Number(-value.to_number()?),
            // `+` throws for BigInt, which has no number value here.
            UnaryOperator::Positive => Self::Number(value.to_number()?),
            UnaryOperator::BitwiseNot => Self::Number(f64::from(!to_int32(value.to_number()?))),
            UnaryOperator::Delete => return None,
        })
    }

    /// Applies a binary operator. Returns `None` when the result is unknown
    /// or the operation throws. Concatenated strings are allocated in the AST
    /// arena.
    pub(super) fn binary(
        ast: &Ast<'a>,
        operator: BinaryOperator,
        left: Self,
        right: Self,
    ) -> Option<Self> {
        let number = |operation: fn(f64, f64) -> f64| {
            Some(Self::Number(operation(
                left.to_number()?,
                right.to_number()?,
            )))
        };
        let int32 = |operation: fn(i32, u32) -> f64| {
            Some(Self::Number(operation(
                to_int32(left.to_number()?),
                to_uint32(right.to_number()?),
            )))
        };
        Some(match operator {
            BinaryOperator::StrictEqual => Self::Boolean(left.strict_equals(right)?),
            BinaryOperator::StrictNotEqual => Self::Boolean(!left.strict_equals(right)?),
            BinaryOperator::Equal => Self::Boolean(left.loose_equals(right)?),
            BinaryOperator::NotEqual => Self::Boolean(!left.loose_equals(right)?),
            // An undefined comparison result (NaN) makes every relation false.
            BinaryOperator::LessThan => Self::Boolean(left.less_than(right)? == Some(true)),
            BinaryOperator::GreaterThan => Self::Boolean(right.less_than(left)? == Some(true)),
            BinaryOperator::LessThanOrEqual => Self::Boolean(right.less_than(left)? == Some(false)),
            BinaryOperator::GreaterThanOrEqual => {
                Self::Boolean(left.less_than(right)? == Some(false))
            }
            BinaryOperator::Add
                if matches!(left, Self::String(_)) || matches!(right, Self::String(_)) =>
            {
                let mut text = Wtf8Buf::new();
                left.push_string(&mut text)?;
                right.push_string(&mut text)?;
                let text: &Wtf8 = &text;
                Self::String(text.clone_in(ast.allocator()))
            }
            BinaryOperator::Add => return number(|left, right| left + right),
            BinaryOperator::Subtract => return number(|left, right| left - right),
            BinaryOperator::Multiply => return number(|left, right| left * right),
            BinaryOperator::Divide => return number(|left, right| left / right),
            // Rust's `%` truncates like ECMAScript's remainder.
            BinaryOperator::Modulo => return number(|left, right| left % right),
            BinaryOperator::Exponent => return number(exponentiate),
            BinaryOperator::BitwiseOr => {
                return int32(|left, right| f64::from(left | right as i32));
            }
            BinaryOperator::BitwiseXor => {
                return int32(|left, right| f64::from(left ^ right as i32));
            }
            BinaryOperator::BitwiseAnd => {
                return int32(|left, right| f64::from(left & right as i32));
            }
            // Shift counts are masked to five bits, as by `wrapping_shl`.
            BinaryOperator::LeftShift => {
                return int32(|left, right| f64::from(left.wrapping_shl(right)));
            }
            BinaryOperator::RightShift => {
                return int32(|left, right| f64::from(left.wrapping_shr(right)));
            }
            BinaryOperator::UnsignedRightShift => {
                return int32(|left, right| f64::from((left as u32).wrapping_shr(right)));
            }
            // Both throw for a primitive right operand.
            BinaryOperator::In | BinaryOperator::Instanceof => return None,
        })
    }

    /// ECMAScript `ToBoolean`.
    pub(super) fn to_boolean(self) -> Option<bool> {
        match self {
            Self::Undefined | Self::Null => Some(false),
            Self::Boolean(value) => Some(value),
            Self::Number(value) => Some(value != 0.0 && !value.is_nan()),
            Self::String(value) => Some(!value.is_empty()),
            Self::BigInt => None,
        }
    }

    /// ECMAScript `ToNumber`. Returns `None` for BigInt, which throws, and
    /// for numeric strings too large to convert exactly.
    pub(super) fn to_number(self) -> Option<f64> {
        match self {
            Self::Undefined => Some(f64::NAN),
            Self::Null => Some(0.0),
            Self::Boolean(value) => Some(f64::from(u8::from(value))),
            Self::Number(value) => Some(value),
            Self::String(value) => string_to_number(value),
            Self::BigInt => None,
        }
    }

    /// The result of the `typeof` operator.
    pub(super) fn type_of(self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Null => "object",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::BigInt => "bigint",
        }
    }

    /// Appends ECMAScript `ToString`. Returns `None` for BigInt.
    fn push_string(self, output: &mut Wtf8Buf) -> Option<()> {
        match self {
            Self::Undefined => output.push_str("undefined"),
            Self::Null => output.push_str("null"),
            Self::Boolean(value) => output.push_str(if value { "true" } else { "false" }),
            Self::Number(value) => push_number(output, value),
            Self::String(value) => output.push_wtf8(value),
            Self::BigInt => return None,
        }
        Some(())
    }

    /// `IsStrictlyEqual`.
    fn strict_equals(self, other: Self) -> Option<bool> {
        Some(match (self, other) {
            (Self::BigInt, Self::BigInt) => return None,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Undefined, Self::Undefined) | (Self::Null, Self::Null) => true,
            _ => false,
        })
    }

    /// `IsLooselyEqual` for primitive operands.
    fn loose_equals(self, other: Self) -> Option<bool> {
        match (self, other) {
            (Self::Undefined | Self::Null, Self::Undefined | Self::Null) => Some(true),
            (Self::Undefined | Self::Null, _) | (_, Self::Undefined | Self::Null) => Some(false),
            _ if std::mem::discriminant(&self) == std::mem::discriminant(&other) => {
                self.strict_equals(other)
            }
            (Self::BigInt, _) | (_, Self::BigInt) => None,
            // The remaining operands are booleans, numbers, and strings, which
            // are compared as numbers.
            _ => Some(self.to_number()? == other.to_number()?),
        }
    }

    /// `IsLessThan` for primitive operands. The inner `None` is the
    /// undefined result produced by NaN.
    fn less_than(self, other: Self) -> Option<Option<bool>> {
        if let (Self::String(left), Self::String(right)) = (self, other) {
            // Strings compare by UTF-16 code units, not by code points.
            return Some(Some(
                left.to_ill_formed_utf16().lt(right.to_ill_formed_utf16()),
            ));
        }
        let (left, right) = (self.to_number()?, other.to_number()?);
        Some((!left.is_nan() && !right.is_nan()).then_some(left < right))
    }
}

fn skip_parentheses(ast: &Ast<'_>, mut expr: Expr) -> Expr {
    while let ExprData::ParenthesizedExpression(parenthesized) = ast.expr_data(expr) {
        expr = parenthesized.expression(ast);
    }
    expr
}

/// ECMAScript `Number::toString` with radix 10.
fn push_number(output: &mut Wtf8Buf, value: f64) {
    if value.is_nan() {
        output.push_str("NaN");
    } else if value.is_infinite() {
        output.push_str(if value > 0.0 { "Infinity" } else { "-Infinity" });
    } else if value == 0.0 {
        // Negative zero is also printed as `0`.
        output.push_str("0");
    } else {
        output.push_str(zmij_ecma::Buffer::new().format_finite(value));
    }
}

/// ECMAScript `StringToNumber`.
fn string_to_number(value: &Wtf8) -> Option<f64> {
    // A lone surrogate is neither white space nor part of a numeral.
    let Some(text) = value.as_str() else {
        return Some(f64::NAN);
    };
    let text = text.trim_matches(is_white_space);
    if text.is_empty() {
        return Some(0.0);
    }
    let radix = match text.as_bytes() {
        [b'0', b'x' | b'X', ..] => 16,
        [b'0', b'o' | b'O', ..] => 8,
        [b'0', b'b' | b'B', ..] => 2,
        _ => return Some(decimal_to_number(text)),
    };
    let digits = &text[2..];
    if digits.is_empty() || !digits.chars().all(|digit| digit.is_digit(radix)) {
        return Some(f64::NAN);
    }
    // `as` rounds to the nearest double. Larger values are left unknown
    // rather than rounded twice.
    u128::from_str_radix(digits, radix)
        .ok()
        .map(|value| value as f64)
}

/// `StrDecimalLiteral` and its `Infinity` forms, or NaN for other text.
fn decimal_to_number(text: &str) -> f64 {
    let (negative, unsigned) = match text.as_bytes()[0] {
        b'-' => (true, &text[1..]),
        b'+' => (false, &text[1..]),
        _ => (false, text),
    };
    let magnitude = if unsigned == "Infinity" {
        f64::INFINITY
    } else if is_unsigned_decimal(unsigned) {
        // Validated text is a subset of what `f64::from_str` accepts, and it
        // rounds correctly.
        unsigned.parse().unwrap_or(f64::NAN)
    } else {
        return f64::NAN;
    };
    if negative { -magnitude } else { magnitude }
}

/// `StrUnsignedDecimalLiteral` without `Infinity`. Rust also accepts `inf`,
/// `nan`, and signs, which ECMAScript does not accept here.
fn is_unsigned_decimal(text: &str) -> bool {
    let bytes = text.as_bytes();
    let digits = |start: usize| {
        bytes[start..]
            .iter()
            .position(|byte| !byte.is_ascii_digit())
            .map_or(bytes.len(), |length| start + length)
    };
    let integer_end = digits(0);
    let mut end = integer_end;
    let mut has_digits = integer_end != 0;
    if bytes.get(end) == Some(&b'.') {
        let fraction_end = digits(end + 1);
        has_digits |= fraction_end != end + 1;
        end = fraction_end;
    }
    if !has_digits {
        return false;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        end += 1;
        if matches!(bytes.get(end), Some(b'+' | b'-')) {
            end += 1;
        }
        let exponent_end = digits(end);
        if exponent_end == end {
            return false;
        }
        end = exponent_end;
    }
    end == bytes.len()
}

/// `StrWhiteSpaceChar`: `WhiteSpace` and `LineTerminator`.
fn is_white_space(character: char) -> bool {
    matches!(
        character,
        '\t' | '\n' | '\u{B}' | '\u{C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

fn to_uint32(value: f64) -> u32 {
    if !value.is_finite() {
        return 0;
    }
    value.trunc().rem_euclid(4_294_967_296.0) as u32
}

fn to_int32(value: f64) -> i32 {
    to_uint32(value) as i32
}

/// `Number::exponentiate`, which differs from `powf` for NaN exponents and
/// for a base of ±1 with an infinite exponent.
fn exponentiate(base: f64, exponent: f64) -> f64 {
    if exponent.is_nan() || (base.abs() == 1.0 && exponent.is_infinite()) {
        f64::NAN
    } else {
        base.powf(exponent)
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::{Allocator, wtf8::Wtf8Buf};

    use super::Value;
    use crate::compress::tests::{expression, parse};

    fn evaluate(source: &str) -> Option<String> {
        let allocator = Allocator::new();
        let ast = parse(&allocator, source);
        Value::of(&ast, expression(&ast)).map(describe)
    }

    /// Strings are quoted; other values use their `ToString` text.
    fn describe(value: Value<'_>) -> String {
        match value {
            Value::String(value) => format!("{:?}", value.to_string_lossy()),
            Value::Number(value) if value == 0.0 && value.is_sign_negative() => "-0".into(),
            Value::BigInt => "bigint".into(),
            value => {
                let mut text = Wtf8Buf::new();
                value.push_string(&mut text).unwrap();
                text.into_string_lossy()
            }
        }
    }

    #[test]
    fn literals() {
        assert_eq!(evaluate("('a')").as_deref(), Some("\"a\""));

        assert_eq!(evaluate("1.5").as_deref(), Some("1.5"));

        assert_eq!(evaluate("1n").as_deref(), Some("bigint"));

        assert_eq!(evaluate("((null))").as_deref(), Some("null"));

        assert_eq!(evaluate("`a`").as_deref(), Some("\"a\""));

        assert_eq!(evaluate("`${1}`"), None);

        assert_eq!(evaluate("x"), None);
    }

    #[test]
    fn unary_operators() {
        assert_eq!(evaluate("!0").as_deref(), Some("true"));

        assert_eq!(evaluate("void 'a'").as_deref(), Some("undefined"));

        assert_eq!(evaluate("-0").as_deref(), Some("-0"));

        assert_eq!(evaluate("~1").as_deref(), Some("-2"));

        assert_eq!(evaluate("typeof null").as_deref(), Some("\"object\""));

        assert_eq!(evaluate("-1n").as_deref(), Some("bigint"));

        assert_eq!(evaluate("+1n"), None);

        assert_eq!(evaluate("!1n"), None);

        assert_eq!(evaluate("delete 0"), None);

        assert_eq!(evaluate("!x"), None);
    }

    #[test]
    fn binary_operators() {
        assert_eq!(evaluate("-1 / 0").as_deref(), Some("-Infinity"));

        assert_eq!(evaluate("0 / 0").as_deref(), Some("NaN"));

        assert_eq!(evaluate("true + null").as_deref(), Some("1"));

        assert_eq!(evaluate("7 % -2").as_deref(), Some("1"));

        assert_eq!(evaluate("2 ** -1").as_deref(), Some("0.5"));

        assert_eq!(evaluate("1 ** void 0").as_deref(), Some("NaN"));

        assert_eq!(evaluate("-1 >>> 0").as_deref(), Some("4294967295"));

        assert_eq!(evaluate("1 << 33").as_deref(), Some("2"));

        assert_eq!(evaluate("5 & -2").as_deref(), Some("4"));

        assert_eq!(evaluate("1n + 1"), None);

        assert_eq!(evaluate("1 in ''"), None);

        assert_eq!(evaluate("1 + 2 + 3"), None);
    }

    #[test]
    fn string_concatenation() {
        assert_eq!(evaluate("'a' + void 0").as_deref(), Some("\"aundefined\""));

        assert_eq!(evaluate("'' + -0").as_deref(), Some("\"0\""));

        assert_eq!(evaluate("'' + 1e21").as_deref(), Some("\"1e+21\""));

        assert_eq!(evaluate("'' + 1e-7").as_deref(), Some("\"1e-7\""));

        assert_eq!(evaluate("'' + 0.000001").as_deref(), Some("\"0.000001\""));

        assert_eq!(evaluate("'' + 1 / 0").as_deref(), None);

        assert_eq!(evaluate("`${1 / 0}` + ''"), None);

        assert_eq!(evaluate("'a' + 1n"), None);
    }

    #[test]
    fn comparisons() {
        assert_eq!(evaluate("'1' == 1").as_deref(), Some("true"));

        assert_eq!(evaluate("'1' === 1").as_deref(), Some("false"));

        assert_eq!(evaluate("null == void 0").as_deref(), Some("true"));

        assert_eq!(evaluate("null == 0").as_deref(), Some("false"));

        assert_eq!(evaluate("true == '1'").as_deref(), Some("true"));

        assert_eq!(evaluate("1n == 1"), None);

        assert_eq!(evaluate("1n === 1").as_deref(), Some("false"));

        assert_eq!(evaluate("'b' < 'a'").as_deref(), Some("false"));

        assert_eq!(
            evaluate("'\\u{10000}' < '\\uFFFF'").as_deref(),
            Some("true")
        );

        assert_eq!(evaluate("1 <= void 0").as_deref(), Some("false"));

        assert_eq!(evaluate("1 >= void 0").as_deref(), Some("false"));

        assert_eq!(evaluate("null >= 0").as_deref(), Some("true"));
    }

    #[test]
    fn string_to_number() {
        assert_eq!(evaluate("+' \\n12\\t'").as_deref(), Some("12"));

        assert_eq!(evaluate("+''").as_deref(), Some("0"));

        assert_eq!(evaluate("+'\\uFEFF1'").as_deref(), Some("1"));

        assert_eq!(evaluate("+'\\u00851'").as_deref(), Some("NaN"));

        assert_eq!(evaluate("+'-1.5e3'").as_deref(), Some("-1500"));

        assert_eq!(evaluate("+'.5'").as_deref(), Some("0.5"));

        assert_eq!(evaluate("+'5.'").as_deref(), Some("5"));

        assert_eq!(evaluate("+'.'").as_deref(), Some("NaN"));

        assert_eq!(evaluate("+'1e'").as_deref(), Some("NaN"));

        assert_eq!(evaluate("+'-Infinity'").as_deref(), Some("-Infinity"));

        assert_eq!(evaluate("+'inf'").as_deref(), Some("NaN"));

        assert_eq!(evaluate("+'1_0'").as_deref(), Some("NaN"));

        assert_eq!(evaluate("+'0x1F'").as_deref(), Some("31"));

        assert_eq!(evaluate("+'0b102'").as_deref(), Some("NaN"));

        assert_eq!(evaluate("+'-0x1'").as_deref(), Some("NaN"));

        assert_eq!(evaluate("+'0x+1'").as_deref(), Some("NaN"));

        assert_eq!(evaluate("+'0x'").as_deref(), Some("NaN"));
    }
}
