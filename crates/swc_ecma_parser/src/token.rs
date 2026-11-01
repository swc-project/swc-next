use bitflags::bitflags;
use swc_next_ecma_ast::{Precedence, Span};

bitflags! {
    #[derive(Clone, Copy)]
    struct TokenFlags: u32 {
        const IS_NUMERIC_LITERAL = 1 << 13;
        const IS_BINARY_OP = 1 << 14;
        const IS_LOGICAL_OP = 1 << 15;
        const IS_UNARY_OP = 1 << 16;
        const IS_ASSIGNMENT_OP = 1 << 17;
        const IS_IDENTIFIER_LIKE = 1 << 18;
        const IS_UNCONDITIONALLY_RESERVED = 1 << 19;
        const IS_STRICT_MODE_RESERVED = 1 << 20;
        const IS_KEYWORD = 1 << 21;
        const CAN_HAVE_TRAILING_COMMENT = 1 << 22;

        const IS_BIN_OP = Self::IS_BINARY_OP.bits() | Self::IS_LOGICAL_OP.bits();
    }
}

macro_rules! flags {
    ($first:ident $(| $rest:ident)*) => {
        TokenFlags::$first.bits() $(| TokenFlags::$rest.bits())*
    };
}

const PREC_SHIFT: u32 = 8;
const PREC_OVERLAP: u32 = 0b11111;
const NUMERIC_LITERAL_FLAGS: TokenFlags =
    TokenFlags::IS_NUMERIC_LITERAL.union(TokenFlags::CAN_HAVE_TRAILING_COMMENT);

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenTag {
    NumericLiteral = 1 | NUMERIC_LITERAL_FLAGS.bits(),
    HexLiteral = 2 | NUMERIC_LITERAL_FLAGS.bits(),
    OctalLiteral = 3 | NUMERIC_LITERAL_FLAGS.bits(),
    BinaryLiteral = 4 | NUMERIC_LITERAL_FLAGS.bits(),
    BigIntLiteral = 5 | NUMERIC_LITERAL_FLAGS.bits(),
    StringLiteral = 6 | flags!(CAN_HAVE_TRAILING_COMMENT),
    RegexLiteral = 7 | flags!(CAN_HAVE_TRAILING_COMMENT),
    NoSubstitutionTemplate = 8 | ((Precedence::Call as u32) << PREC_SHIFT),
    TemplateHead =
        9 | ((Precedence::Call as u32) << PREC_SHIFT) | flags!(CAN_HAVE_TRAILING_COMMENT),
    TemplateMiddle = 10 | flags!(CAN_HAVE_TRAILING_COMMENT),
    TemplateTail = 11,

    True = 12 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    False = 13 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    NullLiteral = 14 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),

    Plus = 15 | ((Precedence::Add as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP | IS_UNARY_OP),
    Minus = 16 | ((Precedence::Add as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP | IS_UNARY_OP),
    Star = 17 | ((Precedence::Multiply as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    Slash = 18 | ((Precedence::Multiply as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    Percent = 19 | ((Precedence::Multiply as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    Exponent = 20 | ((Precedence::Exponent as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),

    Assign = 21 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    PlusAssign = 22 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    MinusAssign = 23 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    StarAssign = 24 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    SlashAssign = 25 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    PercentAssign = 26 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    ExponentAssign =
        27 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),

    Increment = 28 | ((Precedence::Postfix as u32) << PREC_SHIFT),
    Decrement = 29 | ((Precedence::Postfix as u32) << PREC_SHIFT),

    Equal = 30 | ((Precedence::Equality as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    NotEqual = 31 | ((Precedence::Equality as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    StrictEqual = 32 | ((Precedence::Equality as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    StrictNotEqual = 33 | ((Precedence::Equality as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    LessThan = 34 | ((Precedence::Compare as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    GreaterThan = 35 | ((Precedence::Compare as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    LessThanEqual = 36 | ((Precedence::Compare as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    GreaterThanEqual = 37 | ((Precedence::Compare as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),

    LogicalAnd = 38 | ((Precedence::LogicalAnd as u32) << PREC_SHIFT) | flags!(IS_LOGICAL_OP),
    LogicalOr = 39 | ((Precedence::LogicalOr as u32) << PREC_SHIFT) | flags!(IS_LOGICAL_OP),
    LogicalNot = 40 | ((Precedence::Prefix as u32) << PREC_SHIFT) | flags!(IS_UNARY_OP),

    BitwiseAnd = 41 | ((Precedence::BitwiseAnd as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    BitwiseOr = 42 | ((Precedence::BitwiseOr as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    BitwiseXor = 43 | ((Precedence::BitwiseXor as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    BitwiseNot = 44 | flags!(IS_UNARY_OP),
    LeftShift = 45 | ((Precedence::Shift as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    RightShift = 46 | ((Precedence::Shift as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),
    UnsignedRightShift = 47 | ((Precedence::Shift as u32) << PREC_SHIFT) | flags!(IS_BINARY_OP),

    BitwiseAndAssign =
        48 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    BitwiseOrAssign =
        49 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    BitwiseXorAssign =
        50 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    LeftShiftAssign =
        51 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    RightShiftAssign =
        52 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    UnsignedRightShiftAssign =
        53 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),

    // Use `||`'s precedence for left-associative recovery of mixed `??`/`||` chains.
    // The parser still diagnoses mixing `??` with `||` or `&&`.
    NullishCoalescing = 54 | ((Precedence::LogicalOr as u32) << PREC_SHIFT) | flags!(IS_LOGICAL_OP),
    NullishAssign = 55 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    LogicalAndAssign =
        56 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    LogicalOrAssign =
        57 | ((Precedence::Assignment as u32) << PREC_SHIFT) | flags!(IS_ASSIGNMENT_OP),
    OptionalChaining = 58 | ((Precedence::Call as u32) << PREC_SHIFT),

    LeftParen = 59 | ((Precedence::Call as u32) << PREC_SHIFT),
    RightParen = 60,
    LeftBrace = 61 | flags!(CAN_HAVE_TRAILING_COMMENT),
    RightBrace = 62 | flags!(CAN_HAVE_TRAILING_COMMENT),
    LeftBracket = 63 | ((Precedence::Member as u32) << PREC_SHIFT),
    RightBracket = 64,
    Semicolon = 65 | flags!(CAN_HAVE_TRAILING_COMMENT),
    Comma = 66 | ((Precedence::Comma as u32) << PREC_SHIFT),
    Dot = 67 | ((Precedence::Member as u32) << PREC_SHIFT),
    Spread = 68,
    Arrow = 69 | ((Precedence::Assignment as u32) << PREC_SHIFT),
    Question = 70 | ((Precedence::Conditional as u32) << PREC_SHIFT),
    Colon = 71,
    At = 72,

    If = 73 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Else = 74 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Switch = 75 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Case = 76 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Default = 77 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    For = 78 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    While = 79 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Do = 80 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Break = 81 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Continue = 82 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),

    Function = 83 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Return = 84 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Async = 85 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Await = 86 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Yield = 87 | flags!(IS_KEYWORD | IS_STRICT_MODE_RESERVED | IS_IDENTIFIER_LIKE),

    Var = 88 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Let = 89 | flags!(IS_KEYWORD | IS_STRICT_MODE_RESERVED | IS_IDENTIFIER_LIKE),
    Const = 90 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Using = 91 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),

    Class = 92 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Extends = 93 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Super = 94 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Static = 95 | flags!(IS_KEYWORD | IS_STRICT_MODE_RESERVED | IS_IDENTIFIER_LIKE),
    Enum = 96 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Public = 97 | flags!(IS_KEYWORD | IS_STRICT_MODE_RESERVED | IS_IDENTIFIER_LIKE),
    Private = 98 | flags!(IS_KEYWORD | IS_STRICT_MODE_RESERVED | IS_IDENTIFIER_LIKE),
    Protected = 99 | flags!(IS_KEYWORD | IS_STRICT_MODE_RESERVED | IS_IDENTIFIER_LIKE),
    Interface = 100 | flags!(IS_KEYWORD | IS_STRICT_MODE_RESERVED | IS_IDENTIFIER_LIKE),
    Implements = 101 | flags!(IS_KEYWORD | IS_STRICT_MODE_RESERVED | IS_IDENTIFIER_LIKE),
    Package = 102 | flags!(IS_KEYWORD | IS_STRICT_MODE_RESERVED | IS_IDENTIFIER_LIKE),

    Import = 103 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Export = 104 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    From = 105 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    As = 106
        | ((Precedence::Compare as u32) << PREC_SHIFT)
        | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Namespace = 107 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Assert = 108 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Source = 109 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Defer = 110 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),

    Try = 111 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Catch = 112 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Finally = 113 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Throw = 114 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),

    New = 115 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    This = 116 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Typeof = 117
        | flags!(IS_UNARY_OP | IS_KEYWORD)
        | flags!(IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Instanceof = 118
        | ((Precedence::Compare as u32) << PREC_SHIFT)
        | flags!(IS_BINARY_OP | IS_KEYWORD)
        | flags!(IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    In = 119
        | ((Precedence::Compare as u32) << PREC_SHIFT)
        | flags!(IS_BINARY_OP | IS_KEYWORD)
        | flags!(IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Of = 120 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Delete = 121
        | flags!(IS_UNARY_OP | IS_KEYWORD)
        | flags!(IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Void = 122
        | flags!(IS_UNARY_OP | IS_KEYWORD)
        | flags!(IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    With = 123 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),
    Debugger = 124 | flags!(IS_KEYWORD | IS_UNCONDITIONALLY_RESERVED | IS_IDENTIFIER_LIKE),

    Identifier = 125 | flags!(IS_IDENTIFIER_LIKE | CAN_HAVE_TRAILING_COMMENT),
    PrivateIdentifier = 126,

    Get = 127 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Set = 128 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Accessor = 129 | flags!(IS_IDENTIFIER_LIKE),
    Constructor = 130 | flags!(IS_IDENTIFIER_LIKE),

    Declare = 131 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Type = 132 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Abstract = 133 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Override = 134 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Readonly = 135 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Keyof = 136 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Unique = 137 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Infer = 138 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Out = 139 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Asserts = 140 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Satisfies = 141
        | ((Precedence::Compare as u32) << PREC_SHIFT)
        | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Intrinsic = 142 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Is = 143 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Global = 144 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Module = 145 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Require = 146 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),

    Any = 147 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Bigint = 148 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Boolean = 149 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Never = 150 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Number = 151 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Object = 152 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    String = 153 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Symbol = 154 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Undefined = 155 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),
    Unknown = 156 | flags!(IS_KEYWORD | IS_IDENTIFIER_LIKE),

    JsxIdentifier = 157 | flags!(CAN_HAVE_TRAILING_COMMENT),
    JsxText = 158 | flags!(CAN_HAVE_TRAILING_COMMENT),
    Eof = 159,
}

impl TokenTag {
    #[inline]
    pub fn raw(self) -> u32 {
        self as u32
    }

    #[inline]
    pub fn precedence(self) -> Precedence {
        let precedence = ((self.raw() >> PREC_SHIFT) & PREC_OVERLAP) as u8;
        debug_assert!(precedence <= Precedence::Primary as u8);
        // SAFETY: `Precedence` is a contiguous `u8` enum starting at zero,
        // and token precedence is restricted to its range above.
        unsafe { std::mem::transmute(precedence) }
    }

    #[inline]
    fn has_flag(self, flag: TokenFlags) -> bool {
        TokenFlags::from_bits_retain(self.raw()).contains(flag)
    }

    #[inline]
    pub fn is_numeric_literal(self) -> bool {
        self.has_flag(TokenFlags::IS_NUMERIC_LITERAL)
    }

    #[inline]
    pub fn is_binary_operator(self) -> bool {
        self.has_flag(TokenFlags::IS_BINARY_OP)
    }

    #[inline]
    pub fn can_have_trailing_line_comment(self) -> bool {
        !TokenFlags::from_bits_retain(self.raw()).intersects(TokenFlags::IS_BIN_OP)
    }

    #[inline]
    pub fn can_have_trailing_comment(self) -> bool {
        self.has_flag(TokenFlags::CAN_HAVE_TRAILING_COMMENT)
    }

    #[inline]
    pub fn is_logical_operator(self) -> bool {
        self.has_flag(TokenFlags::IS_LOGICAL_OP)
    }

    #[inline]
    pub fn is_unary_operator(self) -> bool {
        self.has_flag(TokenFlags::IS_UNARY_OP)
    }

    #[inline]
    pub fn is_assignment_operator(self) -> bool {
        self.has_flag(TokenFlags::IS_ASSIGNMENT_OP)
    }

    /// Lexical IdentifierName, including reserved words. This is not a
    /// binding/reference or expression/type-start grammar predicate.
    #[inline]
    pub fn is_identifier_like(self) -> bool {
        self.has_flag(TokenFlags::IS_IDENTIFIER_LIKE)
    }

    /// Context-free identifier candidate. Await, Yield and strict-mode words
    /// still need validation in their grammar role. Reserved words excluded
    /// here may nevertheless be consumed as names during parser recovery.
    #[inline]
    pub fn is_identifier_candidate(self) -> bool {
        self.is_identifier_like() && !self.is_unconditionally_reserved()
    }

    #[inline]
    pub fn is_keyword(self) -> bool {
        self.has_flag(TokenFlags::IS_KEYWORD)
    }

    #[inline]
    pub fn is_unconditionally_reserved(self) -> bool {
        self.has_flag(TokenFlags::IS_UNCONDITIONALLY_RESERVED)
    }

    #[inline]
    pub fn is_strict_mode_reserved(self) -> bool {
        self.has_flag(TokenFlags::IS_STRICT_MODE_RESERVED)
    }

    #[inline]
    pub fn is_reserved(self) -> bool {
        self.is_unconditionally_reserved() || self.is_strict_mode_reserved()
    }

    pub fn to_str(self) -> Option<&'static str> {
        Some(match self {
            Self::True => "true",
            Self::False => "false",
            Self::NullLiteral => "null",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Exponent => "**",
            Self::Assign => "=",
            Self::PlusAssign => "+=",
            Self::MinusAssign => "-=",
            Self::StarAssign => "*=",
            Self::SlashAssign => "/=",
            Self::PercentAssign => "%=",
            Self::ExponentAssign => "**=",
            Self::Increment => "++",
            Self::Decrement => "--",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::StrictEqual => "===",
            Self::StrictNotEqual => "!==",
            Self::LessThan => "<",
            Self::GreaterThan => ">",
            Self::LessThanEqual => "<=",
            Self::GreaterThanEqual => ">=",
            Self::LogicalAnd => "&&",
            Self::LogicalOr => "||",
            Self::LogicalNot => "!",
            Self::BitwiseAnd => "&",
            Self::BitwiseOr => "|",
            Self::BitwiseXor => "^",
            Self::BitwiseNot => "~",
            Self::LeftShift => "<<",
            Self::RightShift => ">>",
            Self::UnsignedRightShift => ">>>",
            Self::BitwiseAndAssign => "&=",
            Self::BitwiseOrAssign => "|=",
            Self::BitwiseXorAssign => "^=",
            Self::LeftShiftAssign => "<<=",
            Self::RightShiftAssign => ">>=",
            Self::UnsignedRightShiftAssign => ">>>=",
            Self::NullishCoalescing => "??",
            Self::NullishAssign => "??=",
            Self::LogicalAndAssign => "&&=",
            Self::LogicalOrAssign => "||=",
            Self::OptionalChaining => "?.",
            Self::LeftParen => "(",
            Self::RightParen => ")",
            Self::LeftBrace => "{",
            Self::RightBrace => "}",
            Self::LeftBracket => "[",
            Self::RightBracket => "]",
            Self::Semicolon => ";",
            Self::Comma => ",",
            Self::Dot => ".",
            Self::Spread => "...",
            Self::Arrow => "=>",
            Self::Question => "?",
            Self::Colon => ":",
            Self::At => "@",
            Self::If => "if",
            Self::Else => "else",
            Self::Switch => "switch",
            Self::Case => "case",
            Self::Default => "default",
            Self::For => "for",
            Self::While => "while",
            Self::Do => "do",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Function => "function",
            Self::Return => "return",
            Self::Async => "async",
            Self::Await => "await",
            Self::Yield => "yield",
            Self::Var => "var",
            Self::Let => "let",
            Self::Const => "const",
            Self::Using => "using",
            Self::Class => "class",
            Self::Extends => "extends",
            Self::Super => "super",
            Self::Static => "static",
            Self::Enum => "enum",
            Self::Public => "public",
            Self::Private => "private",
            Self::Protected => "protected",
            Self::Interface => "interface",
            Self::Implements => "implements",
            Self::Package => "package",
            Self::Import => "import",
            Self::Export => "export",
            Self::From => "from",
            Self::As => "as",
            Self::Namespace => "namespace",
            Self::Assert => "assert",
            Self::Source => "source",
            Self::Defer => "defer",
            Self::Try => "try",
            Self::Catch => "catch",
            Self::Finally => "finally",
            Self::Throw => "throw",
            Self::New => "new",
            Self::This => "this",
            Self::Typeof => "typeof",
            Self::Instanceof => "instanceof",
            Self::In => "in",
            Self::Of => "of",
            Self::Delete => "delete",
            Self::Void => "void",
            Self::With => "with",
            Self::Debugger => "debugger",
            Self::Declare => "declare",
            Self::Get => "get",
            Self::Set => "set",
            Self::Accessor => "accessor",
            Self::Constructor => "constructor",
            Self::Type => "type",
            Self::Abstract => "abstract",
            Self::Override => "override",
            Self::Readonly => "readonly",
            Self::Keyof => "keyof",
            Self::Unique => "unique",
            Self::Infer => "infer",
            Self::Out => "out",
            Self::Asserts => "asserts",
            Self::Satisfies => "satisfies",
            Self::Intrinsic => "intrinsic",
            Self::Is => "is",
            Self::Global => "global",
            Self::Module => "module",
            Self::Require => "require",
            Self::Any => "any",
            Self::Bigint => "bigint",
            Self::Boolean => "boolean",
            Self::Never => "never",
            Self::Number => "number",
            Self::Object => "object",
            Self::String => "string",
            Self::Symbol => "symbol",
            Self::Undefined => "undefined",
            Self::Unknown => "unknown",
            Self::Eof
            | Self::NumericLiteral
            | Self::HexLiteral
            | Self::OctalLiteral
            | Self::BinaryLiteral
            | Self::BigIntLiteral
            | Self::StringLiteral
            | Self::RegexLiteral
            | Self::NoSubstitutionTemplate
            | Self::TemplateHead
            | Self::TemplateMiddle
            | Self::TemplateTail
            | Self::Identifier
            | Self::PrivateIdentifier
            | Self::JsxIdentifier
            | Self::JsxText => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenFlag {
    LineTerminatorBefore,
    InvalidEscape,
    Escaped,
    LoneSurrogates,
}

pub fn flag_mask(flag: TokenFlag) -> u8 {
    1 << (flag as u8)
}

const START_SHIFT: u32 = 64;
const END_SHIFT: u32 = 96;
const FLAGS_SHIFT: u32 = 32;

// Bits 0..32: tag, 32..40: flags, 64..96: start, 96..128: end.
// The tag needs no shift, and the high word has the layout of a span.
// The remaining bits are always zero.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct Token(u128);

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("span", &self.span())
            .field("tag", &self.tag())
            .field("flags", &self.flags())
            .finish()
    }
}

impl Token {
    #[inline]
    pub const fn new(start: u32, end: u32, tag: TokenTag, flags: u8) -> Self {
        Self(
            ((start as u128) << START_SHIFT)
                | ((end as u128) << END_SHIFT)
                | (tag as u128)
                | ((flags as u128) << FLAGS_SHIFT),
        )
    }

    #[inline]
    pub const fn start(self) -> u32 {
        (self.0 >> START_SHIFT) as u32
    }

    #[inline]
    pub const fn end(self) -> u32 {
        (self.0 >> END_SHIFT) as u32
    }

    #[inline]
    pub const fn span(self) -> Span {
        Span {
            start: self.start(),
            end: self.end(),
        }
    }

    #[inline]
    pub const fn tag(self) -> TokenTag {
        // SAFETY: TokenTag has repr(u32), and only `new` writes these bits,
        // using the discriminant of a valid TokenTag.
        unsafe { std::mem::transmute::<u32, TokenTag>(self.0 as u32) }
    }

    #[inline]
    pub const fn flags(self) -> u8 {
        (self.0 >> FLAGS_SHIFT) as u8
    }

    #[inline]
    pub fn set_flags(&mut self, flags: u8) {
        self.0 = (self.0 & !(0xff_u128 << FLAGS_SHIFT)) | ((flags as u128) << FLAGS_SHIFT);
    }

    #[inline]
    pub fn eof(pos: u32) -> Self {
        Self::new(pos, pos, TokenTag::Eof, 0)
    }

    #[inline]
    pub fn text(self, source: &str) -> &str {
        &source[self.start() as usize..self.end() as usize]
    }

    #[inline]
    pub fn has(self, flag: TokenFlag) -> bool {
        (self.flags() & flag_mask(flag)) != 0
    }

    #[inline]
    pub fn has_line_terminator_before(self) -> bool {
        self.has(TokenFlag::LineTerminatorBefore)
    }

    /// Match a grammar keyword terminal, rather than an escaped identifier.
    #[inline]
    pub fn is_keyword(self, tag: TokenTag) -> bool {
        self.tag() == tag && !self.is_escaped()
    }

    #[inline]
    pub fn is_escaped(self) -> bool {
        self.has(TokenFlag::Escaped)
    }

    #[inline]
    pub fn has_invalid_escape(self) -> bool {
        self.has(TokenFlag::InvalidEscape)
    }

    #[inline]
    pub fn has_lone_surrogates(self) -> bool {
        self.has(TokenFlag::LoneSurrogates)
    }
}
