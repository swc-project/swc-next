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
