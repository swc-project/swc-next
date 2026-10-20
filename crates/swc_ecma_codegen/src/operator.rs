use swc_next_ecma_ast::{
    Ast, BinaryExpression, BinaryOperator, Expr, ExprData, LogicalExpression, LogicalOperator,
    Precedence, TsTupleElementData, TsType, TsTypeData, UnaryOperator, UpdateOperator,
};

#[derive(Clone, Copy)]
pub(crate) enum Binaryish {
    Binary(BinaryExpression),
    Logical(LogicalExpression),
}

impl Binaryish {
    pub(crate) fn left(self, ast: &Ast<'_>) -> Expr {
        match self {
            Self::Binary(e) => e.left(ast),
            Self::Logical(e) => e.left(ast),
        }
    }

    pub(crate) fn right(self, ast: &Ast<'_>) -> Expr {
        match self {
            Self::Binary(e) => e.right(ast),
            Self::Logical(e) => e.right(ast),
        }
    }

    pub(crate) fn operator(self, ast: &Ast<'_>) -> BinaryishOperator {
        match self {
            Self::Binary(e) => BinaryishOperator::Binary(e.operator(ast)),
            Self::Logical(e) => BinaryishOperator::Logical(e.operator(ast)),
        }
    }

    // Precedence levels describe the actual printed children, including syntax
    // restrictions beyond operator precedence. Both emission and TypeScript
    // fragment analysis must observe these same parentheses.
    pub(crate) fn left_precedence(
        self,
        ast: &Ast<'_>,
        operator: BinaryishOperator,
        minify: bool,
    ) -> Precedence {
        let precedence = operator.precedence();
        if operator.wraps_cast()
            && matches!(
                ast.expr_data(self.left(ast)),
                ExprData::TsAsExpression(_) | ExprData::TsSatisfiesExpression(_)
            )
        {
            return Precedence::Compare.next();
        }
        match operator {
            BinaryishOperator::Logical(LogicalOperator::NullishCoalescing) => {
                nullish_operand_precedence(ast, self.left(ast), precedence)
            }
            BinaryishOperator::Binary(BinaryOperator::Exponent) => {
                // A unary expression cannot directly precede `**`.
                if matches!(
                    ast.expr_data(self.left(ast)),
                    ExprData::UnaryExpression(_)
                        | ExprData::AwaitExpression(_)
                        | ExprData::TsTypeAssertion(_)
                ) {
                    Precedence::Call
                } else {
                    precedence.next()
                }
            }
            BinaryishOperator::Binary(BinaryOperator::LeftShift) => {
                left_shift_precedence(ast, self.left(ast), minify)
            }
            _ => precedence,
        }
    }

    // Omitting a cast's group before `<<` exposes comparison precedence at
    // the left edge. An enclosing operator must still contain that cast.
    pub(crate) fn printed_precedence(
        self,
        ast: &Ast<'_>,
        operator: BinaryishOperator,
        minify: bool,
    ) -> Precedence {
        if operator == BinaryishOperator::Binary(BinaryOperator::LeftShift) {
            operator
                .precedence()
                .min(self.left_precedence(ast, operator, minify))
        } else {
            operator.precedence()
        }
    }

    pub(crate) fn right_precedence(self, ast: &Ast<'_>, operator: BinaryishOperator) -> Precedence {
        let precedence = operator.precedence();
        match operator {
            BinaryishOperator::Binary(BinaryOperator::Exponent) => precedence,
            BinaryishOperator::Logical(LogicalOperator::NullishCoalescing) => {
                nullish_operand_precedence(ast, self.right(ast), precedence.next())
            }
            _ => precedence.next(),
        }
    }
}

// Base printer facts are immutable throughout planning. Carry them between
// continuations; grammar summaries still wait for finalized child groups.
#[derive(Clone, Copy)]
pub(crate) struct PreparedBinary {
    pub(crate) binary: Binaryish,
    pub(crate) operator: BinaryishOperator,
    pub(crate) left_level: Precedence,
    pub(crate) right_level: Precedence,
}

impl PreparedBinary {
    pub(crate) fn new(
        ast: &Ast<'_>,
        binary: Binaryish,
        operator: BinaryishOperator,
        minify: bool,
    ) -> Self {
        Self {
            binary,
            operator,
            left_level: binary.left_precedence(ast, operator, minify),
            right_level: binary.right_precedence(ast, operator),
        }
    }

    pub(crate) fn expression(self) -> Expr {
        match self.binary {
            Binaryish::Binary(node) => Expr::BinaryExpression(node),
            Binaryish::Logical(node) => Expr::LogicalExpression(node),
        }
    }

    pub(crate) fn left(self, ast: &Ast<'_>) -> Expr {
        self.binary.left(ast)
    }

    pub(crate) fn right(self, ast: &Ast<'_>) -> Expr {
        self.binary.right(ast)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum BinaryishOperator {
    Binary(BinaryOperator),
    Logical(LogicalOperator),
}

impl BinaryishOperator {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Binary(op) => op.as_str(),
            Self::Logical(op) => op.as_str(),
        }
    }

    pub(crate) fn precedence(self) -> Precedence {
        match self {
            Self::Binary(op) => op.precedence(),
            Self::Logical(op) => op.precedence(),
        }
    }

    // `<`, `|`, and `&` can extend an `as` or `satisfies` type. Instantiation
    // expressions instead need grouping before `<`, `>`, `+`, and `-`.
    // TypeScript's scanner also starts `>=`, `>>`, and `>>>` with `>`.
    pub(crate) fn following_context(self) -> ExprContext {
        let forbids_instantiation = matches!(
            self,
            Self::Binary(
                BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::GreaterThanOrEqual
                    | BinaryOperator::RightShift
                    | BinaryOperator::UnsignedRightShift
                    | BinaryOperator::Add
                    | BinaryOperator::Subtract
            )
        );
        let mut context = ExprContext::empty();
        context.set(ExprContext::WRAP_TS_CAST, self.wraps_cast());
        context.set(ExprContext::NO_INSTANTIATION, forbids_instantiation);
        context
    }

    fn wraps_cast(self) -> bool {
        matches!(
            self,
            Self::Binary(
                BinaryOperator::LessThan | BinaryOperator::BitwiseOr | BinaryOperator::BitwiseAnd
            )
        )
    }

    pub(crate) fn is_keyword(self) -> bool {
        matches!(
            self,
            Self::Binary(BinaryOperator::In | BinaryOperator::Instanceof)
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperatorCode {
    Binary(BinaryOperator),
    Assignment,
    Unary(UnaryOperator),
    Update(UpdateOperator, bool),
    TsNonNull,
    TsTypeOpen,
    TsTypeClose,
}

// Binary and logical expressions share this visitor because either kind can
// appear on the left spine of the other. Keeping the state on one reusable
// heap-backed stack avoids overflowing the native stack for deeply nested ASTs.
#[derive(Clone, Copy)]
pub(crate) struct BinaryExprVisitor {
    // Inputs
    pub(crate) e: Binaryish,
    pub(crate) level: Precedence,
    pub(crate) flags: bool,
    // Carry grouping context through the rightmost ungrouped operand.
    pub(crate) context: ExprContext,

    // Input for visiting the left child
    pub(crate) left_level: Precedence,
    pub(crate) left_flags: bool,
    pub(crate) left_context: ExprContext,

    // Local variables passed from `prepare` to
    // `visit_right_and_finish`
    pub(crate) operator: BinaryishOperator,
    pub(crate) wrap: bool,
    pub(crate) right_level: Precedence,
    pub(crate) right_context: ExprContext,
}

bitflags::bitflags! {
    // Keep recursive printing context in one scalar instead of packing separate booleans.
    #[derive(Clone, Copy, Default)]
    pub(crate) struct ExprContext: u8 {
        const IN_NEW_CALLEE = 1 << 0;
        const WRAP_TS_CAST = 1 << 1;
        const NO_INSTANTIATION = 1 << 2;
    }
}

impl ExprContext {
    pub(crate) fn inside_group(self, grouped: bool) -> Self {
        if grouped { Self::empty() } else { self }
    }

    // Prefix expressions and transparent chains end in their operand, so the
    // following-token restriction on instantiations reaches that operand.
    pub(crate) fn instantiation_tail(self, grouped: bool) -> Self {
        self.inside_group(grouped) & Self::NO_INSTANTIATION
    }
}

// The printed type tail determines whether `<<` can follow a bare cast.
fn left_shift_precedence(ast: &Ast<'_>, left: Expr, minify: bool) -> Precedence {
    let (mut expression, type_annotation) = match ast.expr_data(left) {
        ExprData::TsAsExpression(assertion) => {
            (assertion.expression(ast), assertion.type_annotation(ast))
        }
        ExprData::TsSatisfiesExpression(assertion) => {
            (assertion.expression(ast), assertion.type_annotation(ast))
        }
        _ => return Precedence::Shift,
    };
    if !ts_type_can_precede_left_shift(ast, type_annotation, minify) {
        return Precedence::Shift;
    }
    // Keep grouping if the cast's operand would absorb the shift.
    loop {
        expression = match ast.expr_data(expression) {
            ExprData::TsAsExpression(assertion) => assertion.expression(ast),
            ExprData::TsSatisfiesExpression(assertion) => assertion.expression(ast),
            ExprData::BinaryExpression(binary)
                if binary.operator(ast).precedence() < Precedence::Shift =>
            {
                return Precedence::Shift;
            }
            ExprData::LogicalExpression(logical)
                if logical.operator(ast).precedence() < Precedence::Shift =>
            {
                return Precedence::Shift;
            }
            _ => break,
        };
    }
    Precedence::Compare
}

// `??` cannot directly contain `||` or `&&`.
fn nullish_operand_precedence(ast: &Ast<'_>, child: Expr, level: Precedence) -> Precedence {
    if matches!(
        ast.expr_data(child),
        ExprData::LogicalExpression(logical)
            if matches!(logical.operator(ast), LogicalOperator::And | LogicalOperator::Or)
    ) {
        Precedence::Prefix
    } else {
        level
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TsTypePrecedence {
    Lowest,
    Function,
    Conditional,
    Union,
    Intersection,
    TypeOperator,
    Postfix,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TsTypeContext {
    Default,
    // Conditional extends clauses and infer constraints accept function types
    // but not bare conditional types. Direct infer constraints consume `extends`.
    ExtendsClause,
    // Function returns allow conditional types again, so a constrained infer
    // needs grouping before the enclosing conditional's `?`.
    BeforeConditionalQuestion,
}

impl TsTypeContext {
    pub(crate) fn inside_group(self, grouped: bool) -> Self {
        if grouped { Self::Default } else { self }
    }

    pub(crate) fn function_return(self, grouped: bool) -> Self {
        match self.inside_group(grouped) {
            Self::Default => Self::Default,
            Self::ExtendsClause | Self::BeforeConditionalQuestion => {
                Self::BeforeConditionalQuestion
            }
        }
    }
}
fn ts_type_can_precede_left_shift(ast: &Ast<'_>, mut ty: TsType, minify: bool) -> bool {
    let mut level = TsTypePrecedence::Lowest;
    let mut context = TsTypeContext::Default;
    // Follow the printed right edge. Explicit or generated parentheses,
    // postfix operators, and other closing delimiters end the type instead.
    loop {
        ty = match ast.ts_type_data(ty) {
            TsTypeData::TsTypeQuery(query) => {
                // TypeScript requires a standalone `<` token to start these
                // type arguments, leaving `<<` for expression parsing.
                // https://github.com/microsoft/TypeScript/blob/15dee00a1c6549a4953ab17ccac4c56d252ba4cc/tsc/internal/parser/parser.go#L3067-L3072
                // `typeof import(...)` takes the separate import-type path.
                // https://github.com/microsoft/TypeScript/blob/15dee00a1c6549a4953ab17ccac4c56d252ba4cc/tsc/internal/parser/parser.go#L2846-L2850
                return !query.expr_name(ast).is_ts_import_type(ast);
            }
            TsTypeData::TsUnionType(t) if level <= TsTypePrecedence::Union => {
                level = if minify {
                    TsTypePrecedence::Union
                } else {
                    TsTypePrecedence::Intersection
                };
                context = TsTypeContext::Default;
                let Some(last) = t.types(ast).last() else {
                    return false;
                };
                ast.get_node_in_sub_range(last)
            }
            TsTypeData::TsIntersectionType(t) if level <= TsTypePrecedence::Intersection => {
                level = if minify {
                    TsTypePrecedence::Intersection
                } else {
                    TsTypePrecedence::TypeOperator
                };
                context = TsTypeContext::Default;
                let Some(last) = t.types(ast).last() else {
                    return false;
                };
                ast.get_node_in_sub_range(last)
            }
            TsTypeData::TsConditionalType(t)
                if level <= TsTypePrecedence::Conditional
                    && context != TsTypeContext::ExtendsClause =>
            {
                level = TsTypePrecedence::Lowest;
                t.false_type(ast)
            }
            TsTypeData::TsTypeOperator(t) if level <= TsTypePrecedence::TypeOperator => {
                level = TsTypePrecedence::TypeOperator;
                t.type_annotation(ast)
            }
            TsTypeData::TsFunctionType(t) if level <= TsTypePrecedence::Function => {
                level = TsTypePrecedence::Function;
                context = context.function_return(false);
                t.return_type(ast).type_annotation(ast)
            }
            TsTypeData::TsConstructorType(t) if level <= TsTypePrecedence::Function => {
                level = TsTypePrecedence::Function;
                context = context.function_return(false);
                t.return_type(ast).type_annotation(ast)
            }
            TsTypeData::TsInferType(t) => {
                let Some(constraint) = t.type_parameter(ast).constraint(ast) else {
                    return false;
                };
                let precedence = if context == TsTypeContext::ExtendsClause {
                    TsTypePrecedence::TypeOperator
                } else {
                    TsTypePrecedence::Conditional
                };
                if level > precedence || context == TsTypeContext::BeforeConditionalQuestion {
                    return false;
                }
                level = TsTypePrecedence::Lowest;
                context = TsTypeContext::ExtendsClause;
                constraint
            }
            TsTypeData::TsTypePredicate(t) => {
                let Some(annotation) = t.type_annotation(ast) else {
                    return false;
                };
                level = TsTypePrecedence::Lowest;
                annotation.type_annotation(ast)
            }
            TsTypeData::TsRestType(t) => {
                level = TsTypePrecedence::Lowest;
                context = TsTypeContext::Default;
                match ast.ts_tuple_element_data(t.type_annotation(ast)) {
                    TsTupleElementData::TsType(t) => t,
                    TsTupleElementData::TsNamedTupleMember(t) => t.element_type(ast),
                }
            }
            TsTypeData::TsJsDocNullableType(t) if !t.postfix(ast) => {
                level = TsTypePrecedence::TypeOperator;
                context = TsTypeContext::Default;
                t.type_annotation(ast)
            }
            TsTypeData::TsJsDocNonNullableType(t) if !t.postfix(ast) => {
                level = TsTypePrecedence::TypeOperator;
                context = TsTypeContext::Default;
                t.type_annotation(ast)
            }
            _ => return false,
        };
    }
}
