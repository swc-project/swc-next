//! SWC's enum constant evaluator before #12455.
use crate::common::resolver::context::{Id, Resolver};
use std::collections::{HashMap, HashSet};
use swc_next_allocator::wtf8::Wtf8Buf;
use swc_next_ecma_ast::*;

pub(super) type TsEnumRecord = HashMap<(Id, Wtf8Buf), TsEnumRecordValue>;

#[derive(Clone)]
pub(super) enum TsEnumRecordValue {
    String(Wtf8Buf),
    Number {
        value: f64,
        raw: Option<AstUtf8>,
        span: Span,
    },
    Opaque(Expr),
    Void,
}

impl TsEnumRecordValue {
    pub(super) fn number(value: f64) -> Self {
        Self::Number {
            value,
            raw: None,
            span: DUMMY_SP,
        }
    }
    pub(super) fn is_const(&self) -> bool {
        matches!(self, Self::String(_) | Self::Number { .. })
    }
    pub(super) fn inc(&self) -> Self {
        match self {
            Self::Number { value, .. } => Self::number(value + 1.0),
            _ => Self::Void,
        }
    }
    pub(super) fn expression(&self, ast: &mut Ast<'_>) -> Expr {
        match self {
            Self::String(value) => {
                let value = ast.add_wtf8(value);
                ast.expr_string_literal(DUMMY_SP, value, AstUtf8::EMPTY)
            }
            Self::Number { value, raw, span } => ast.expr_numeric_literal(*span, *value, *raw),
            Self::Opaque(expression) => *expression,
            Self::Void => {
                let zero = ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
                ast.expr_unary_expression(DUMMY_SP, zero, UnaryOperator::Void)
            }
        }
    }
    fn push_to_string(&self, output: &mut Wtf8Buf) -> bool {
        match self {
            Self::String(value) => output.push_wtf8(value),
            Self::Number { value, .. } => {
                output.push_str(&crate::common::build::number_string(*value))
            }
            _ => return false,
        }
        true
    }
}

#[derive(Clone, Copy)]
pub(super) struct EvalCtx {
    allow_const_var: bool,
    ts_is_opaque: bool,
}
impl EvalCtx {
    pub(super) const CONST_INIT: Self = Self {
        allow_const_var: true,
        ts_is_opaque: true,
    };
    pub(super) const MEMBER: Self = Self {
        allow_const_var: true,
        ts_is_opaque: false,
    };
    pub(super) const RECOMPUTE: Self = Self {
        allow_const_var: false,
        ts_is_opaque: false,
    };
}

pub(super) struct EnumValueComputer<'a> {
    pub(super) enum_id: &'a Id,
    pub(super) resolver: &'a Resolver,
    pub(super) record: &'a TsEnumRecord,
    pub(super) ambient: &'a TsEnumRecord,
    pub(super) const_vars: &'a HashMap<Id, TsEnumRecordValue>,
    pub(super) const_only: Option<&'a HashSet<Id>>,
    pub(super) ambient_const_only: Option<&'a HashSet<Id>>,
}

/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/ts_enum.rs#L134>.
pub(super) fn static_enum_member_name(ast: &Ast<'_>, member: MemberExpression) -> Option<Wtf8Buf> {
    let key = member.property(ast);
    if !member.computed(ast) {
        return key
            .as_identifier_name(ast)
            .map(|id| Wtf8Buf::from_str(ast.get_utf8(id.name(ast))));
    }
    match ast.expr_data(key.as_expr(ast)?) {
        ExprData::StringLiteral(string) => Some(ast.get_wtf8(string.value(ast)).to_owned()),
        ExprData::TemplateLiteral(template)
            if template.expressions(ast).is_empty() && template.quasis(ast).len() == 1 =>
        {
            let quasi = ast.get_node_in_sub_range(template.quasis(ast).iter().next().unwrap());
            (!quasi.is_cooked_undefined(ast)).then(|| ast.get_wtf8(quasi.cooked(ast)).to_owned())
        }
        _ => None,
    }
}

impl EnumValueComputer<'_> {
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/ts_enum.rs#L214>.
    pub(super) fn compute(
        &self,
        ast: &mut Ast<'_>,
        expression: Expr,
        context: EvalCtx,
    ) -> TsEnumRecordValue {
        let opaque = TsEnumRecordValue::Opaque(expression);
        match ast.expr_data(expression) {
            ExprData::StringLiteral(string) => {
                TsEnumRecordValue::String(ast.get_wtf8(string.value(ast)).to_owned())
            }
            ExprData::NumericLiteral(number) => TsEnumRecordValue::Number {
                value: number.value(ast),
                raw: number.raw(ast),
                span: number.span(ast),
            },
            ExprData::IdentifierReference(identifier) => {
                let id = self
                    .resolver
                    .id(identifier.node_id(), identifier.name(ast), ast);
                if id.scope == self.resolver.unresolved() {
                    let key = (self.enum_id.clone(), Wtf8Buf::from_str(&id.name));
                    if let Some(value) = self.record.get(&key).or_else(|| {
                        (context.allow_const_var && self.ambient_is_resolvable(&key.0))
                            .then(|| self.ambient.get(&key))
                            .flatten()
                    }) {
                        if value.is_const() {
                            value.clone()
                        } else {
                            // This is a reference to an opaque sibling member.
                            // TraverseCtx registration is done by the enum reference rewriter.
                            opaque
                        }
                    } else {
                        match id.name.as_str() {
                            "Infinity" => TsEnumRecordValue::Number {
                                value: f64::INFINITY,
                                raw: Some(identifier.name(ast)),
                                span: identifier.span(ast),
                            },
                            "NaN" => TsEnumRecordValue::Number {
                                value: f64::NAN,
                                raw: Some(identifier.name(ast)),
                                span: identifier.span(ast),
                            },
                            _ => opaque,
                        }
                    }
                } else if context.allow_const_var {
                    self.const_vars.get(&id).cloned().unwrap_or(opaque)
                } else {
                    opaque
                }
            }
            ExprData::ParenthesizedExpression(paren) => {
                self.compute(ast, paren.expression(ast), context)
            }
            ExprData::UnaryExpression(unary) => self.compute_unary(ast, unary, context),
            ExprData::BinaryExpression(binary) => self.compute_bin(ast, binary, context),
            ExprData::MemberExpression(member) => self.compute_member(ast, member, context),
            ExprData::ChainExpression(chain) => {
                if let Some(member) = chain.expression(ast).as_member_expression(ast) {
                    match self.compute_member(ast, member, context) {
                        TsEnumRecordValue::Opaque(_) => opaque,
                        value => value,
                    }
                } else {
                    opaque
                }
            }
            ExprData::TemplateLiteral(template) => self.compute_tpl(ast, template, context),
            ExprData::TsAsExpression(wrapper) => {
                self.wrapper(ast, wrapper.expression(ast), context)
            }
            ExprData::TsSatisfiesExpression(wrapper) => {
                self.wrapper(ast, wrapper.expression(ast), context)
            }
            ExprData::TsTypeAssertion(wrapper) => {
                self.wrapper(ast, wrapper.expression(ast), context)
            }
            ExprData::TsNonNullExpression(wrapper) => {
                self.wrapper(ast, wrapper.expression(ast), context)
            }
            ExprData::TsInstantiationExpression(wrapper) => {
                self.wrapper(ast, wrapper.expression(ast), context)
            }
            _ => opaque,
        }
    }

    fn wrapper(&self, ast: &mut Ast<'_>, inner: Expr, context: EvalCtx) -> TsEnumRecordValue {
        if context.ts_is_opaque {
            TsEnumRecordValue::Opaque(inner)
        } else {
            self.compute(
                ast,
                inner,
                EvalCtx {
                    allow_const_var: false,
                    ..context
                },
            )
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/ts_enum.rs#L309>.
    fn compute_unary(
        &self,
        ast: &mut Ast<'_>,
        unary: UnaryExpression,
        context: EvalCtx,
    ) -> TsEnumRecordValue {
        let operator = unary.operator(ast);
        if !matches!(
            operator,
            UnaryOperator::Positive | UnaryOperator::Negate | UnaryOperator::BitwiseNot
        ) {
            return TsEnumRecordValue::Opaque(Expr::UnaryExpression(unary));
        }
        let inner = self.compute(ast, unary.argument(ast), context);
        if let TsEnumRecordValue::Number { value, .. } = inner {
            TsEnumRecordValue::number(match operator {
                UnaryOperator::Positive => value,
                UnaryOperator::Negate => -value,
                UnaryOperator::BitwiseNot => f64::from(!to_int32(value)),
                _ => unreachable!(),
            })
        } else {
            let inner = inner.expression(ast);
            TsEnumRecordValue::Opaque(ast.expr_unary_expression(unary.span(ast), inner, operator))
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/ts_enum.rs#L335>.
    fn compute_bin(
        &self,
        ast: &mut Ast<'_>,
        binary: BinaryExpression,
        context: EvalCtx,
    ) -> TsEnumRecordValue {
        let operator = binary.operator(ast);
        if !matches!(
            operator,
            BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Modulo
                | BinaryOperator::Exponent
                | BinaryOperator::LeftShift
                | BinaryOperator::RightShift
                | BinaryOperator::UnsignedRightShift
                | BinaryOperator::BitwiseOr
                | BinaryOperator::BitwiseAnd
                | BinaryOperator::BitwiseXor
        ) {
            return TsEnumRecordValue::Opaque(Expr::BinaryExpression(binary));
        }
        let left = self.compute(ast, binary.left(ast), context);
        let right = self.compute(ast, binary.right(ast), context);
        if operator == BinaryOperator::Add
            && (matches!(left, TsEnumRecordValue::String(_))
                || matches!(right, TsEnumRecordValue::String(_)))
        {
            let mut value = Wtf8Buf::new();
            if left.push_to_string(&mut value) && right.push_to_string(&mut value) {
                return TsEnumRecordValue::String(value);
            }
        }
        if let (
            TsEnumRecordValue::Number { value: left, .. },
            TsEnumRecordValue::Number { value: right, .. },
        ) = (&left, &right)
        {
            return TsEnumRecordValue::number(match operator {
                BinaryOperator::Add => left + right,
                BinaryOperator::Subtract => left - right,
                BinaryOperator::Multiply => left * right,
                BinaryOperator::Divide => left / right,
                BinaryOperator::Modulo => left % right,
                BinaryOperator::Exponent => {
                    if right.is_nan() || left.abs() == 1.0 && right.is_infinite() {
                        f64::NAN
                    } else {
                        left.powf(*right)
                    }
                }
                BinaryOperator::LeftShift => {
                    f64::from(to_int32(*left).wrapping_shl(to_int32(*right) as u32 & 31))
                }
                BinaryOperator::RightShift => {
                    f64::from(to_int32(*left).wrapping_shr(to_int32(*right) as u32 & 31))
                }
                BinaryOperator::UnsignedRightShift => {
                    f64::from((to_int32(*left) as u32).wrapping_shr(to_int32(*right) as u32 & 31))
                }
                BinaryOperator::BitwiseOr => f64::from(to_int32(*left) | to_int32(*right)),
                BinaryOperator::BitwiseAnd => f64::from(to_int32(*left) & to_int32(*right)),
                BinaryOperator::BitwiseXor => f64::from(to_int32(*left) ^ to_int32(*right)),
                _ => unreachable!(),
            });
        }
        let left = if left.is_const() {
            left.expression(ast)
        } else {
            binary.left(ast)
        };
        let right = if right.is_const() {
            right.expression(ast)
        } else {
            binary.right(ast)
        };
        TsEnumRecordValue::Opaque(ast.expr_binary_expression(
            binary.span(ast),
            left,
            right,
            operator,
        ))
    }

    fn ambient_is_resolvable(&self, id: &Id) -> bool {
        self.ambient_const_only.is_none_or(|set| set.contains(id))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/ts_enum.rs#L409>.
    fn compute_member(
        &self,
        ast: &Ast<'_>,
        member: MemberExpression,
        context: EvalCtx,
    ) -> TsEnumRecordValue {
        let opaque = TsEnumRecordValue::Opaque(Expr::MemberExpression(member));
        let Some(name) = static_enum_member_name(ast, member) else {
            return opaque;
        };
        let Some(identifier) = member.object(ast).as_identifier_reference(ast) else {
            return opaque;
        };
        let id = self
            .resolver
            .id(identifier.node_id(), identifier.name(ast), ast);
        if self.const_only.is_some_and(|set| !set.contains(&id)) {
            return opaque;
        }
        let key = (id, name);
        self.record
            .get(&key)
            .or_else(|| {
                (context.allow_const_var && self.ambient_is_resolvable(&key.0))
                    .then(|| self.ambient.get(&key))
                    .flatten()
            })
            .filter(|value| !matches!(value, TsEnumRecordValue::Void))
            .cloned()
            .unwrap_or(opaque)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/ts_enum.rs#L451>.
    fn compute_tpl(
        &self,
        ast: &mut Ast<'_>,
        template: TemplateLiteral,
        context: EvalCtx,
    ) -> TsEnumRecordValue {
        let opaque = TsEnumRecordValue::Opaque(Expr::TemplateLiteral(template));
        let mut quasis = template.quasis(ast).iter();
        let Some(first) = quasis.next() else {
            return opaque;
        };
        let first = ast.get_node_in_sub_range(first);
        if first.is_cooked_undefined(ast) {
            return opaque;
        }
        let cooked = first.cooked(ast);
        let mut value = ast.get_wtf8(cooked).to_owned();
        for (quasi, expression) in quasis.zip(template.expressions(ast).iter()) {
            let expression = ast.get_node_in_sub_range(expression);
            if !self
                .compute(ast, expression, context)
                .push_to_string(&mut value)
            {
                return opaque;
            }
            let quasi = ast.get_node_in_sub_range(quasi);
            if quasi.is_cooked_undefined(ast) {
                return opaque;
            }
            let cooked = quasi.cooked(ast);
            value.push_wtf8(ast.get_wtf8(cooked));
        }
        TsEnumRecordValue::String(value)
    }
}

/// SWC `JsNumber`'s ECMAScript ToInt32 conversion.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/number.rs#L1>.
fn to_int32(value: f64) -> i32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    value.trunc().rem_euclid(4294967296.0) as u32 as i32
}
