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
    Unary(UnaryOperator),
    Update(UpdateOperator, bool),
}

// Binary and logical expressions share this visitor because either kind can
// appear on the left spine of the other. Keeping the state on one reusable
// arena-backed stack avoids overflowing the native stack for deeply nested ASTs.
#[derive(Clone, Copy)]
pub(crate) struct BinaryExprVisitor {
    // Inputs
    pub(crate) e: Binaryish,
    pub(crate) level: Precedence,
    pub(crate) flags: bool,

    // Input for visiting the left child
    pub(crate) left_level: Precedence,
    pub(crate) left_flags: bool,

    // Local variables passed from `check_and_prepare` to
    // `visit_right_and_finish`
    pub(crate) operator: BinaryishOperator,
    pub(crate) wrap: bool,
    pub(crate) right_level: Precedence,
}
