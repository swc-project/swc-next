use swc_next_ecma_ast::{Precedence, Span};

pub mod mask {
    pub const IS_NUMERIC_LITERAL: u32 = 1 << 13;
    pub const IS_BINARY_OP: u32 = 1 << 14;
    pub const IS_LOGICAL_OP: u32 = 1 << 15;
    pub const IS_UNARY_OP: u32 = 1 << 16;
    pub const IS_ASSIGNMENT_OP: u32 = 1 << 17;
    pub const IS_IDENTIFIER_LIKE: u32 = 1 << 18;
    pub const IS_UNCONDITIONALLY_RESERVED: u32 = 1 << 19;
    pub const IS_STRICT_MODE_RESERVED: u32 = 1 << 20;
    pub const IS_KEYWORD: u32 = 1 << 21;

    pub const PREC_SHIFT: u32 = 8;
    pub const PREC_OVERLAP: u32 = 0b11111;
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenTag {
    NumericLiteral = 1 | mask::IS_NUMERIC_LITERAL,
    HexLiteral = 2 | mask::IS_NUMERIC_LITERAL,
    OctalLiteral = 3 | mask::IS_NUMERIC_LITERAL,
    BinaryLiteral = 4 | mask::IS_NUMERIC_LITERAL,
    BigIntLiteral = 5 | mask::IS_NUMERIC_LITERAL,
    StringLiteral = 6,
    RegexLiteral = 7,
    NoSubstitutionTemplate = 8 | ((Precedence::Call as u32) << mask::PREC_SHIFT),
    TemplateHead = 9 | ((Precedence::Call as u32) << mask::PREC_SHIFT),
    TemplateMiddle = 10,
    TemplateTail = 11,

    True = 12 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    False = 13 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    NullLiteral =
        14 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,

    Plus = 15
        | ((Precedence::Add as u32) << mask::PREC_SHIFT)
        | mask::IS_BINARY_OP
        | mask::IS_UNARY_OP,
    Minus = 16
        | ((Precedence::Add as u32) << mask::PREC_SHIFT)
        | mask::IS_BINARY_OP
        | mask::IS_UNARY_OP,
    Star = 17 | ((Precedence::Multiply as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    Slash = 18 | ((Precedence::Multiply as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    Percent = 19 | ((Precedence::Multiply as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    Exponent = 20 | ((Precedence::Exponent as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,

    Assign = 21 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    PlusAssign =
        22 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    MinusAssign =
        23 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    StarAssign =
        24 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    SlashAssign =
        25 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    PercentAssign =
        26 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    ExponentAssign =
        27 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,

    Increment = 28 | ((Precedence::Postfix as u32) << mask::PREC_SHIFT),
    Decrement = 29 | ((Precedence::Postfix as u32) << mask::PREC_SHIFT),

    Equal = 30 | ((Precedence::Equality as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    NotEqual = 31 | ((Precedence::Equality as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    StrictEqual = 32 | ((Precedence::Equality as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    StrictNotEqual = 33 | ((Precedence::Equality as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    LessThan = 34 | ((Precedence::Compare as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    GreaterThan = 35 | ((Precedence::Compare as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    LessThanEqual = 36 | ((Precedence::Compare as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    GreaterThanEqual = 37 | ((Precedence::Compare as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,

    LogicalAnd = 38 | ((Precedence::LogicalAnd as u32) << mask::PREC_SHIFT) | mask::IS_LOGICAL_OP,
    LogicalOr = 39 | ((Precedence::LogicalOr as u32) << mask::PREC_SHIFT) | mask::IS_LOGICAL_OP,
    LogicalNot = 40 | ((Precedence::Prefix as u32) << mask::PREC_SHIFT) | mask::IS_UNARY_OP,

    BitwiseAnd = 41 | ((Precedence::BitwiseAnd as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    BitwiseOr = 42 | ((Precedence::BitwiseOr as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    BitwiseXor = 43 | ((Precedence::BitwiseXor as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    BitwiseNot = 44 | mask::IS_UNARY_OP,
    LeftShift = 45 | ((Precedence::Shift as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    RightShift = 46 | ((Precedence::Shift as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,
    UnsignedRightShift = 47 | ((Precedence::Shift as u32) << mask::PREC_SHIFT) | mask::IS_BINARY_OP,

    BitwiseAndAssign =
        48 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    BitwiseOrAssign =
        49 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    BitwiseXorAssign =
        50 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    LeftShiftAssign =
        51 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    RightShiftAssign =
        52 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    UnsignedRightShiftAssign =
        53 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,

    NullishCoalescing =
        54 | ((Precedence::Nullish as u32) << mask::PREC_SHIFT) | mask::IS_LOGICAL_OP,
    NullishAssign =
        55 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    LogicalAndAssign =
        56 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    LogicalOrAssign =
        57 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT) | mask::IS_ASSIGNMENT_OP,
    OptionalChaining = 58 | ((Precedence::Call as u32) << mask::PREC_SHIFT),

    LeftParen = 59 | ((Precedence::Call as u32) << mask::PREC_SHIFT),
    RightParen = 60,
    LeftBrace = 61,
    RightBrace = 62,
    LeftBracket = 63 | ((Precedence::Member as u32) << mask::PREC_SHIFT),
    RightBracket = 64,
    Semicolon = 65,
    Comma = 66 | ((Precedence::Comma as u32) << mask::PREC_SHIFT),
    Dot = 67 | ((Precedence::Member as u32) << mask::PREC_SHIFT),
    Spread = 68,
    Arrow = 69 | ((Precedence::Assignment as u32) << mask::PREC_SHIFT),
    Question = 70 | ((Precedence::Conditional as u32) << mask::PREC_SHIFT),
    Colon = 71,
    At = 72,

    If = 73 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Else = 74 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Switch = 75 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Case = 76 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Default = 77 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    For = 78 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    While = 79 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Do = 80 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Break = 81 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Continue = 82 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,

    Function = 83 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Return = 84 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Async = 85 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Await = 86 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Yield = 87 | mask::IS_KEYWORD | mask::IS_STRICT_MODE_RESERVED | mask::IS_IDENTIFIER_LIKE,

    Var = 88 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Let = 89 | mask::IS_KEYWORD | mask::IS_STRICT_MODE_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Const = 90 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Using = 91 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,

    Class = 92 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Extends = 93 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Super = 94 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Static = 95 | mask::IS_KEYWORD | mask::IS_STRICT_MODE_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Enum = 96 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Public = 97 | mask::IS_KEYWORD | mask::IS_STRICT_MODE_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Private = 98 | mask::IS_KEYWORD | mask::IS_STRICT_MODE_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Protected = 99 | mask::IS_KEYWORD | mask::IS_STRICT_MODE_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Interface = 100 | mask::IS_KEYWORD | mask::IS_STRICT_MODE_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Implements = 101 | mask::IS_KEYWORD | mask::IS_STRICT_MODE_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Package = 102 | mask::IS_KEYWORD | mask::IS_STRICT_MODE_RESERVED | mask::IS_IDENTIFIER_LIKE,

    Import = 103 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Export = 104 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    From = 105 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    As = 106
        | ((Precedence::Compare as u32) << mask::PREC_SHIFT)
        | mask::IS_KEYWORD
        | mask::IS_IDENTIFIER_LIKE,
    Namespace = 107 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Assert = 108 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Source = 109 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Defer = 110 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,

    Try = 111 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Catch = 112 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Finally = 113 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Throw = 114 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,

    New = 115 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    This = 116 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Typeof = 117
        | mask::IS_UNARY_OP
        | mask::IS_KEYWORD
        | mask::IS_UNCONDITIONALLY_RESERVED
        | mask::IS_IDENTIFIER_LIKE,
    Instanceof = 118
        | ((Precedence::Compare as u32) << mask::PREC_SHIFT)
        | mask::IS_BINARY_OP
        | mask::IS_KEYWORD
        | mask::IS_UNCONDITIONALLY_RESERVED
        | mask::IS_IDENTIFIER_LIKE,
    In = 119
        | ((Precedence::Compare as u32) << mask::PREC_SHIFT)
        | mask::IS_BINARY_OP
        | mask::IS_KEYWORD
        | mask::IS_UNCONDITIONALLY_RESERVED
        | mask::IS_IDENTIFIER_LIKE,
    Of = 120 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Delete = 121
        | mask::IS_UNARY_OP
        | mask::IS_KEYWORD
        | mask::IS_UNCONDITIONALLY_RESERVED
        | mask::IS_IDENTIFIER_LIKE,
    Void = 122
        | mask::IS_UNARY_OP
        | mask::IS_KEYWORD
        | mask::IS_UNCONDITIONALLY_RESERVED
        | mask::IS_IDENTIFIER_LIKE,
    With = 123 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,
    Debugger =
        124 | mask::IS_KEYWORD | mask::IS_UNCONDITIONALLY_RESERVED | mask::IS_IDENTIFIER_LIKE,

    Identifier = 125 | mask::IS_IDENTIFIER_LIKE,
    PrivateIdentifier = 126,

    Get = 127 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Set = 128 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Accessor = 129 | mask::IS_IDENTIFIER_LIKE,
    Constructor = 130 | mask::IS_IDENTIFIER_LIKE,

    Declare = 131 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Type = 132 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Abstract = 133 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Override = 134 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Readonly = 135 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Keyof = 136 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Unique = 137 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Infer = 138 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Out = 139 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Asserts = 140 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Satisfies = 141
        | ((Precedence::Compare as u32) << mask::PREC_SHIFT)
        | mask::IS_KEYWORD
        | mask::IS_IDENTIFIER_LIKE,
    Intrinsic = 142 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Is = 143 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Global = 144 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Module = 145 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Require = 146 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,

    Any = 147 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Bigint = 148 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Boolean = 149 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Never = 150 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Number = 151 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Object = 152 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    String = 153 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Symbol = 154 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Undefined = 155 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,
    Unknown = 156 | mask::IS_KEYWORD | mask::IS_IDENTIFIER_LIKE,

    JsxIdentifier = 157,
    JsxText = 158,
    Eof = 159,
}

impl TokenTag {
    pub fn raw(self) -> u32 {
        self as u32
    }

    pub fn precedence(self) -> Precedence {
        let precedence = ((self.raw() >> mask::PREC_SHIFT) & mask::PREC_OVERLAP) as u8;
        debug_assert!(precedence <= Precedence::Primary as u8);
        // SAFETY: `Precedence` is a contiguous `u8` enum starting at zero,
        // and token precedence is restricted to its range above.
        unsafe { std::mem::transmute(precedence) }
    }

    pub fn has_mask(self, mask: u32) -> bool {
        (self.raw() & mask) != 0
    }

    pub fn is_numeric_literal(self) -> bool {
        self.has_mask(mask::IS_NUMERIC_LITERAL)
    }

    pub fn is_binary_operator(self) -> bool {
        self.has_mask(mask::IS_BINARY_OP)
    }

    pub fn is_logical_operator(self) -> bool {
        self.has_mask(mask::IS_LOGICAL_OP)
    }

    pub fn is_unary_operator(self) -> bool {
        self.has_mask(mask::IS_UNARY_OP)
    }

    pub fn is_assignment_operator(self) -> bool {
        self.has_mask(mask::IS_ASSIGNMENT_OP)
    }

    pub fn is_identifier_like(self) -> bool {
        self.has_mask(mask::IS_IDENTIFIER_LIKE)
    }

    pub fn is_keyword(self) -> bool {
        self.has_mask(mask::IS_KEYWORD)
    }

    pub fn is_unconditionally_reserved(self) -> bool {
        self.has_mask(mask::IS_UNCONDITIONALLY_RESERVED)
    }

    pub fn is_strict_mode_reserved(self) -> bool {
        self.has_mask(mask::IS_STRICT_MODE_RESERVED)
    }

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    pub span: Span,
    pub tag: TokenTag,
    pub flags: u8,
}

impl Token {
    pub fn eof(pos: u32) -> Self {
        Self {
            span: Span {
                start: pos,
                end: pos,
            },
            tag: TokenTag::Eof,
            flags: 0,
        }
    }

    pub fn len(self) -> u32 {
        self.span.end - self.span.start
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn text(self, source: &[u8]) -> &[u8] {
        &source[self.span.start as usize..self.span.end as usize]
    }

    pub fn has(self, flag: TokenFlag) -> bool {
        (self.flags & flag_mask(flag)) != 0
    }

    pub fn has_line_terminator_before(self) -> bool {
        self.has(TokenFlag::LineTerminatorBefore)
    }

    pub fn is_escaped(self) -> bool {
        self.has(TokenFlag::Escaped)
    }

    pub fn has_invalid_escape(self) -> bool {
        self.has(TokenFlag::InvalidEscape)
    }

    pub fn has_lone_surrogates(self) -> bool {
        self.has(TokenFlag::LoneSurrogates)
    }
}
