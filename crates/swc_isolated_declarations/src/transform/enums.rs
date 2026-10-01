//! Constant evaluation for isolated declaration enum members.

use std::borrow::Cow;

use rustc_hash::FxHashMap;
use swc_next_allocator::wtf8::Wtf8Buf;
use swc_next_ecma_ast::{
    Ast, AstWtf8, BinaryOperator, Diagnostic, Expr, ExprData, GetSpan, PropertyKey,
    PropertyKeyData, Span, SymbolId, TsEnumDeclaration, UnaryOperator,
};
use swc_next_ecma_semantic::{NameResolution, Semantic};
use zmij_ecma::Buffer;

use crate::surface::unparenthesized;

const CONST_ENUM_INITIALIZER_NOT_CONSTANT: &str =
    "const enum member initializers must be constant expressions.";

#[derive(Debug, Clone, Copy)]
enum ConstantValue {
    Number(f64),
    String(AstWtf8),
}

impl ConstantValue {
    fn to_expression(self, span: Span, ast: &mut Ast<'_>) -> Expr {
        match self {
            Self::Number(value) => ast.expr_numeric_literal(span, value, None),
            Self::String(value) => ast.expr_string_literal(span, value, ast.empty_utf8()),
        }
    }
}

enum EvaluatedValue {
    Number(f64),
    StoredString(AstWtf8),
    ComputedString(Wtf8Buf),
}

impl From<ConstantValue> for EvaluatedValue {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Number(value) => Self::Number(value),
            ConstantValue::String(value) => Self::StoredString(value),
        }
    }
}

impl EvaluatedValue {
    // Commit synthesized strings once; subsequent members reuse the AST pool handle.
    fn into_constant(self, ast: &mut Ast<'_>) -> ConstantValue {
        match self {
            Self::Number(value) => ConstantValue::Number(value),
            Self::StoredString(value) => ConstantValue::String(value),
            Self::ComputedString(value) => ConstantValue::String(ast.add_wtf8(&value)),
        }
    }

    fn append_to(self, output: &mut Wtf8Buf, ast: &Ast<'_>) {
        match self {
            Self::Number(value) if value.is_nan() => output.push_str("NaN"),
            Self::Number(value) if value == f64::INFINITY => output.push_str("Infinity"),
            Self::Number(value) if value == f64::NEG_INFINITY => output.push_str("-Infinity"),
            Self::Number(value) => {
                let mut buffer = Buffer::new();
                output.push_str(buffer.format(value));
            }
            Self::StoredString(value) => output.push_wtf8(ast.get_wtf8(value)),
            Self::ComputedString(value) => output.push_wtf8(&value),
        }
    }
}

pub(super) fn transform_enum(
    declaration: TsEnumDeclaration,
    ambient: bool,
    diagnostics: &mut Vec<Diagnostic<'static>>,
    semantic: &Semantic<'_>,
    ast: &mut Ast<'_>,
) {
    let members = declaration.body(ast).members(ast);
    let mut previous_number = Some(-1.0);
    let mut previous_members = FxHashMap::default();
    let is_const = declaration.is_const(ast);

    for member in members.iter() {
        let member = ast.get_node_in_sub_range(member);
        let value = match member.initializer(ast) {
            Some(initializer) => {
                let value = evaluate(initializer, semantic, &previous_members, ast);
                if is_const && value.is_none() {
                    diagnostics.push(Diagnostic::error(
                        CONST_ENUM_INITIALIZER_NOT_CONSTANT,
                        initializer.span(ast),
                    ));
                }
                value
            }
            None if ambient && !is_const => None,
            None => previous_number.map(|value| EvaluatedValue::Number(value + 1.0)),
        };
        let value = value.map(|value| value.into_constant(ast));

        previous_number = match value {
            Some(ConstantValue::Number(value)) => Some(value),
            _ => None,
        };
        let initializer = value.map(|value| value.to_expression(member.span(ast), ast));
        if let Some(value) = value
            && let Some(symbol) = semantic.symbol_of(member.node_id())
        {
            previous_members.insert(symbol, value);
        }
        member.set_initializer(ast, initializer);
    }
}

fn evaluate(
    expression: Expr,
    semantic: &Semantic<'_>,
    previous_members: &FxHashMap<SymbolId, ConstantValue>,
    ast: &Ast<'_>,
) -> Option<EvaluatedValue> {
    match ast.expr_data(unparenthesized(expression, ast)) {
        ExprData::StringLiteral(literal) => Some(EvaluatedValue::StoredString(literal.value(ast))),
        ExprData::NumericLiteral(literal) => Some(EvaluatedValue::Number(literal.value(ast))),
        ExprData::TemplateLiteral(template) => {
            let mut quasis = template.quasis(ast).iter();
            let first = ast.get_node_in_sub_range(quasis.next()?);
            if first.is_cooked_undefined(ast) {
                return None;
            }
            if template.expressions(ast).is_empty() {
                return Some(EvaluatedValue::StoredString(first.cooked(ast)));
            }
            let mut value = ast.get_wtf8(first.cooked(ast)).to_owned();

            for (expression, quasi) in template.expressions(ast).iter().zip(quasis) {
                evaluate(
                    ast.get_node_in_sub_range(expression),
                    semantic,
                    previous_members,
                    ast,
                )?
                .append_to(&mut value, ast);
                let quasi = ast.get_node_in_sub_range(quasi);
                if quasi.is_cooked_undefined(ast) {
                    return None;
                }
                value.push_wtf8(ast.get_wtf8(quasi.cooked(ast)));
            }

            Some(EvaluatedValue::ComputedString(value))
        }
        ExprData::UnaryExpression(unary) => {
            let value = evaluate(unary.argument(ast), semantic, previous_members, ast)?;
            evaluate_unary(unary.operator(ast), value)
        }
        ExprData::BinaryExpression(binary) => {
            let left = evaluate(binary.left(ast), semantic, previous_members, ast)?;
            let right = evaluate(binary.right(ast), semantic, previous_members, ast)?;
            evaluate_binary(binary.operator(ast), left, right, ast)
        }
        ExprData::IdentifierReference(identifier) => {
            if let Some(symbol) = resolve_symbol(expression, semantic, ast) {
                return previous_members.get(&symbol).copied().map(Into::into);
            }
            let reference = semantic.reference_of(identifier.node_id())?;
            if semantic.reference(reference).symbol.is_some() {
                return None;
            }
            match ast.get_utf8(identifier.name(ast)) {
                "Infinity" => Some(EvaluatedValue::Number(f64::INFINITY)),
                "NaN" => Some(EvaluatedValue::Number(f64::NAN)),
                _ => None,
            }
        }
        ExprData::MemberExpression(_) => {
            let symbol = resolve_symbol(expression, semantic, ast)?;
            previous_members.get(&symbol).copied().map(Into::into)
        }
        _ => None,
    }
}

fn evaluate_unary(operator: UnaryOperator, value: EvaluatedValue) -> Option<EvaluatedValue> {
    let value = match value {
        EvaluatedValue::Number(value) => value,
        EvaluatedValue::StoredString(_) | EvaluatedValue::ComputedString(_) => return None,
    };
    match operator {
        UnaryOperator::Negate => Some(EvaluatedValue::Number(-value)),
        UnaryOperator::Positive => Some(EvaluatedValue::Number(value)),
        UnaryOperator::BitwiseNot => Some(EvaluatedValue::Number(f64::from(!to_int32(value)))),
        UnaryOperator::LogicalNot
        | UnaryOperator::Typeof
        | UnaryOperator::Void
        | UnaryOperator::Delete => None,
    }
}

fn evaluate_binary(
    operator: BinaryOperator,
    left: EvaluatedValue,
    right: EvaluatedValue,
    ast: &Ast<'_>,
) -> Option<EvaluatedValue> {
    if operator == BinaryOperator::Add
        && (!matches!(left, EvaluatedValue::Number(_))
            || !matches!(right, EvaluatedValue::Number(_)))
    {
        let mut value = Wtf8Buf::new();
        left.append_to(&mut value, ast);
        right.append_to(&mut value, ast);
        return Some(EvaluatedValue::ComputedString(value));
    }

    let EvaluatedValue::Number(left) = left else {
        return None;
    };
    let EvaluatedValue::Number(right) = right else {
        return None;
    };
    let value = match operator {
        BinaryOperator::Add => left + right,
        BinaryOperator::Subtract => left - right,
        BinaryOperator::Multiply => left * right,
        BinaryOperator::Divide => left / right,
        BinaryOperator::Modulo => left % right,
        BinaryOperator::Exponent => left.powf(right),
        BinaryOperator::BitwiseOr => f64::from(to_int32(left) | to_int32(right)),
        BinaryOperator::BitwiseXor => f64::from(to_int32(left) ^ to_int32(right)),
        BinaryOperator::BitwiseAnd => f64::from(to_int32(left) & to_int32(right)),
        BinaryOperator::LeftShift => f64::from(to_int32(left).wrapping_shl(to_uint32(right) & 31)),
        BinaryOperator::RightShift => f64::from(to_int32(left) >> (to_uint32(right) & 31)),
        BinaryOperator::UnsignedRightShift => f64::from(to_uint32(left) >> (to_uint32(right) & 31)),
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::StrictEqual
        | BinaryOperator::StrictNotEqual
        | BinaryOperator::LessThan
        | BinaryOperator::LessThanOrEqual
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterThanOrEqual
        | BinaryOperator::In
        | BinaryOperator::Instanceof => return None,
    };
    Some(EvaluatedValue::Number(value))
}

fn to_uint32(value: f64) -> u32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    value.trunc().rem_euclid(4_294_967_296.0) as u32
}

fn to_int32(value: f64) -> i32 {
    to_uint32(value) as i32
}

fn resolve_symbol(expression: Expr, semantic: &Semantic<'_>, ast: &Ast<'_>) -> Option<SymbolId> {
    let expression = unparenthesized(expression, ast);
    match ast.expr_data(expression) {
        ExprData::IdentifierReference(identifier) => {
            let reference = semantic.reference_of(identifier.node_id())?;
            unique_symbol(semantic.reference_value_targets(ast, reference))
        }
        ExprData::MemberExpression(member) if !member.optional(ast) => {
            let owner = resolve_symbol(member.object(ast), semantic, ast)?;
            let owner = semantic.ts_import_alias_target(owner).unwrap_or(owner);
            let name = property_name(member.property(ast), ast)?;
            unique_symbol(semantic.ts_namespace_value_targets(owner, name.as_ref()))
        }
        _ => None,
    }
}

fn unique_symbol(mut targets: impl Iterator<Item = NameResolution>) -> Option<SymbolId> {
    let symbol = targets.next()?.symbol()?;
    targets
        .all(|target| target.symbol() == Some(symbol))
        .then_some(symbol)
}

fn property_name<'ast>(key: PropertyKey, ast: &Ast<'ast>) -> Option<Cow<'ast, [u8]>> {
    match ast.property_key_data(key) {
        PropertyKeyData::IdentifierName(identifier) => {
            Some(Cow::Borrowed(ast.get_utf8(identifier.name(ast)).as_bytes()))
        }
        PropertyKeyData::Expr(expression) => {
            match ast.expr_data(unparenthesized(expression, ast)) {
                ExprData::StringLiteral(literal) => {
                    Some(Cow::Borrowed(ast.get_wtf8(literal.value(ast)).as_bytes()))
                }
                ExprData::NumericLiteral(literal) => {
                    let mut buffer = Buffer::new();
                    Some(Cow::Owned(
                        buffer.format(literal.value(ast)).as_bytes().to_vec(),
                    ))
                }
                ExprData::TemplateLiteral(template) if template.expressions(ast).is_empty() => {
                    let quasi = ast.get_node_in_sub_range(template.quasis(ast).first()?);
                    (!quasi.is_cooked_undefined(ast))
                        .then(|| Cow::Borrowed(ast.get_wtf8(quasi.cooked(ast)).as_bytes()))
                }
                _ => None,
            }
        }
    }
}
