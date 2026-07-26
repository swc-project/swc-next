use crate::{
    AssignmentOperator, BinaryOperator, LogicalOperator, TsTypeOperatorKind, UnaryOperator,
    UpdateOperator,
};

impl BinaryOperator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::StrictEqual => "===",
            Self::StrictNotEqual => "!==",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "%",
            Self::Exponent => "**",
            Self::BitwiseOr => "|",
            Self::BitwiseXor => "^",
            Self::BitwiseAnd => "&",
            Self::LeftShift => "<<",
            Self::RightShift => ">>",
            Self::UnsignedRightShift => ">>>",
            Self::In => "in",
            Self::Instanceof => "instanceof",
        }
    }

    pub const fn precedence(self) -> Precedence {
        match self {
            Self::Equal | Self::NotEqual | Self::StrictEqual | Self::StrictNotEqual => {
                Precedence::Equality
            }
            Self::LessThan
            | Self::LessThanOrEqual
            | Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::In
            | Self::Instanceof => Precedence::Compare,
            Self::Add | Self::Subtract => Precedence::Add,
            Self::Multiply | Self::Divide | Self::Modulo => Precedence::Multiply,
            Self::Exponent => Precedence::Exponent,
            Self::BitwiseOr => Precedence::BitwiseOr,
            Self::BitwiseXor => Precedence::BitwiseXor,
            Self::BitwiseAnd => Precedence::BitwiseAnd,
            Self::LeftShift | Self::RightShift | Self::UnsignedRightShift => Precedence::Shift,
        }
    }
}

impl LogicalOperator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::And => "&&",
            Self::Or => "||",
            Self::NullishCoalescing => "??",
        }
    }

    pub const fn precedence(self) -> Precedence {
        match self {
            Self::And => Precedence::LogicalAnd,
            Self::Or => Precedence::LogicalOr,
            Self::NullishCoalescing => Precedence::Nullish,
        }
    }
}

impl UnaryOperator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Negate => "-",
            Self::Positive => "+",
            Self::LogicalNot => "!",
            Self::BitwiseNot => "~",
            Self::Typeof => "typeof",
            Self::Void => "void",
            Self::Delete => "delete",
        }
    }
}

impl UpdateOperator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Increment => "++",
            Self::Decrement => "--",
        }
    }
}

impl AssignmentOperator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::AddAssign => "+=",
            Self::SubtractAssign => "-=",
            Self::MultiplyAssign => "*=",
            Self::DivideAssign => "/=",
            Self::ModuloAssign => "%=",
            Self::ExponentAssign => "**=",
            Self::LeftShiftAssign => "<<=",
            Self::RightShiftAssign => ">>=",
            Self::UnsignedRightShiftAssign => ">>>=",
            Self::BitwiseOrAssign => "|=",
            Self::BitwiseXorAssign => "^=",
            Self::BitwiseAndAssign => "&=",
            Self::LogicalOrAssign => "||=",
            Self::LogicalAndAssign => "&&=",
            Self::NullishAssign => "??=",
        }
    }
}

impl TsTypeOperatorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Keyof => "keyof",
            Self::Unique => "unique",
            Self::Readonly => "readonly",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Precedence {
    Lowest,
    Comma,
    Yield,
    Assignment,
    Conditional,
    Nullish,
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equality,
    Compare,
    Shift,
    Add,
    Multiply,
    Exponent,
    Prefix,
    Postfix,
    New,
    Call,
    Member,
    Primary,
}

impl Precedence {
    /// Returns the next tighter precedence level, saturating at [`Self::Primary`].
    pub const fn next(self) -> Self {
        match self {
            Self::Lowest => Self::Comma,
            Self::Comma => Self::Yield,
            Self::Yield => Self::Assignment,
            Self::Assignment => Self::Conditional,
            Self::Conditional => Self::Nullish,
            Self::Nullish => Self::LogicalOr,
            Self::LogicalOr => Self::LogicalAnd,
            Self::LogicalAnd => Self::BitwiseOr,
            Self::BitwiseOr => Self::BitwiseXor,
            Self::BitwiseXor => Self::BitwiseAnd,
            Self::BitwiseAnd => Self::Equality,
            Self::Equality => Self::Compare,
            Self::Compare => Self::Shift,
            Self::Shift => Self::Add,
            Self::Add => Self::Multiply,
            Self::Multiply => Self::Exponent,
            Self::Exponent => Self::Prefix,
            Self::Prefix => Self::Postfix,
            Self::Postfix => Self::New,
            Self::New => Self::Call,
            Self::Call => Self::Member,
            Self::Member => Self::Primary,
            Self::Primary => Self::Primary,
        }
    }
}
