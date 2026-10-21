use swc_next_ecma_ast::{
    Ast, BinaryExpression, BinaryOperator, Expr, LogicalExpression, LogicalOperator, Precedence,
    UnaryOperator, UpdateOperator,
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
        let continues_type = matches!(
            self,
            Self::Binary(
                BinaryOperator::LessThan | BinaryOperator::BitwiseOr | BinaryOperator::BitwiseAnd
            )
        );
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
        context.set(ExprContext::WRAP_TS_CAST, continues_type);
        context.set(ExprContext::NO_INSTANTIATION, forbids_instantiation);
        context
    }

    pub(crate) fn is_keyword(self) -> bool {
        matches!(
            self,
            Self::Binary(BinaryOperator::In | BinaryOperator::Instanceof)
        )
    }

    pub(crate) fn is_right_associative(self) -> bool {
        self == Self::Binary(BinaryOperator::Exponent)
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
