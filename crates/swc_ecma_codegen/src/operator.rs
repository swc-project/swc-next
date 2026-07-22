use swc_experimental_ecma_ast::{
    Ast, BinaryExpression, BinaryOperator, Expr, LogicalExpression, LogicalOperator,
};

use crate::util::Level;

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
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::Binary(op) => match op {
                BinaryOperator::Equal => "==",
                BinaryOperator::NotEqual => "!=",
                BinaryOperator::StrictEqual => "===",
                BinaryOperator::StrictNotEqual => "!==",
                BinaryOperator::LessThan => "<",
                BinaryOperator::LessThanOrEqual => "<=",
                BinaryOperator::GreaterThan => ">",
                BinaryOperator::GreaterThanOrEqual => ">=",
                BinaryOperator::Add => "+",
                BinaryOperator::Subtract => "-",
                BinaryOperator::Multiply => "*",
                BinaryOperator::Divide => "/",
                BinaryOperator::Modulo => "%",
                BinaryOperator::Exponent => "**",
                BinaryOperator::BitwiseOr => "|",
                BinaryOperator::BitwiseXor => "^",
                BinaryOperator::BitwiseAnd => "&",
                BinaryOperator::LeftShift => "<<",
                BinaryOperator::RightShift => ">>",
                BinaryOperator::UnsignedRightShift => ">>>",
                BinaryOperator::In => "in",
                BinaryOperator::Instanceof => "instanceof",
            },
            Self::Logical(LogicalOperator::And) => "&&",
            Self::Logical(LogicalOperator::Or) => "||",
            Self::Logical(LogicalOperator::NullishCoalescing) => "??",
        }
    }

    pub(crate) fn precedence(self) -> Level {
        match self {
            Self::Binary(op) => match op {
                BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::StrictEqual
                | BinaryOperator::StrictNotEqual => Level::Equality,
                BinaryOperator::LessThan
                | BinaryOperator::LessThanOrEqual
                | BinaryOperator::GreaterThan
                | BinaryOperator::GreaterThanOrEqual
                | BinaryOperator::In
                | BinaryOperator::Instanceof => Level::Compare,
                BinaryOperator::Add | BinaryOperator::Subtract => Level::Add,
                BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => {
                    Level::Multiply
                }
                BinaryOperator::Exponent => Level::Exponent,
                BinaryOperator::BitwiseOr => Level::BitwiseOr,
                BinaryOperator::BitwiseXor => Level::BitwiseXor,
                BinaryOperator::BitwiseAnd => Level::BitwiseAnd,
                BinaryOperator::LeftShift
                | BinaryOperator::RightShift
                | BinaryOperator::UnsignedRightShift => Level::Shift,
            },
            Self::Logical(LogicalOperator::And) => Level::LogicalAnd,
            Self::Logical(LogicalOperator::Or) => Level::LogicalOr,
            Self::Logical(LogicalOperator::NullishCoalescing) => Level::Nullish,
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

// Binary and logical expressions share this visitor because either kind can
// appear on the left spine of the other. Keeping the state on one reusable
// arena-backed stack avoids overflowing the native stack for deeply nested ASTs.
#[derive(Clone, Copy)]
pub(crate) struct BinaryExprVisitor {
    // Inputs
    pub(crate) e: Binaryish,
    pub(crate) level: Level,
    pub(crate) flags: bool,

    // Input for visiting the left child
    pub(crate) left_level: Level,
    pub(crate) left_flags: bool,

    // Local variables passed from `check_and_prepare` to
    // `visit_right_and_finish`
    pub(crate) operator: BinaryishOperator,
    pub(crate) wrap: bool,
    pub(crate) right_level: Level,
}
