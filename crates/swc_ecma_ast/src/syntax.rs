use crate::{
    AssignmentOperator, Ast, BinaryOperator, DeclData, LogicalOperator, Stmt, StmtData,
    TsTypeOperatorKind, UnaryOperator, UpdateOperator,
};
use unicode_id_start::{is_id_continue, is_id_start};

impl Stmt {
    /// Whether this statement marks its containing program as a ES module.
    pub fn is_module_indicator(self, ast: &Ast<'_>) -> bool {
        match ast.stmt_data(self) {
            StmtData::ImportDeclaration(_)
            | StmtData::ExportNamedDeclaration(_)
            | StmtData::ExportDefaultDeclaration(_)
            | StmtData::ExportAllDeclaration(_)
            | StmtData::TsExportAssignment(_) => true,
            StmtData::Declaration(declaration) => {
                let DeclData::TsImportEqualsDeclaration(declaration) = ast.decl_data(declaration)
                else {
                    return false;
                };
                declaration
                    .module_reference(ast)
                    .is_ts_external_module_reference(ast)
            }
            _ => false,
        }
    }
}

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

pub fn is_identifier(text: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(text) else {
        return false;
    };
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|code_point| matches!(code_point, '$' | '_') || is_id_start(code_point))
        && chars.all(|code_point| {
            matches!(code_point, '$' | '_' | '\u{200c}' | '\u{200d}') || is_id_continue(code_point)
        })
}

static ASCII_IDENT_CONTINUE: [bool; 256] = {
    let mut table = [false; 256];
    let mut byte = 0;
    while byte < 128 {
        let c = byte as u8;
        table[byte] = c.is_ascii_alphanumeric() || c == b'_' || c == b'$';
        byte += 1;
    }
    table
};

pub fn is_ident_start_ascii(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

#[inline]
pub fn is_ident_continue_ascii(byte: u8) -> bool {
    ASCII_IDENT_CONTINUE[byte as usize]
}

pub fn is_identifier_es5_and_esnext(text: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(text) else {
        return false;
    };
    let mut chars = text.chars();
    chars.next().is_some_and(is_identifier_start_es5_and_esnext)
        && chars.all(is_identifier_continue_es5_and_esnext)
}

pub fn is_identifier_start_es5_and_esnext(code_point: char) -> bool {
    matches!(code_point, '$' | '_')
        || (code_point <= '\u{ffff}'
            && !matches!(code_point, '\u{30fb}' | '\u{ff65}')
            && is_id_start(code_point))
}

pub fn is_identifier_continue_es5_and_esnext(code_point: char) -> bool {
    matches!(code_point, '$' | '_' | '\u{200c}' | '\u{200d}')
        || (code_point <= '\u{ffff}'
            && !matches!(code_point, '\u{30fb}' | '\u{ff65}')
            && is_id_continue(code_point))
}
