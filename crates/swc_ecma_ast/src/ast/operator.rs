use std::mem;

use crate::{Ast, ExtraData, node_id::ExtraDataCompact};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub enum BinaryOp {
    /// `==`
    #[default]
    EqEq,
    /// `!=`
    NotEq,
    /// `===`
    EqEqEq,
    /// `!==`
    NotEqEq,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// `<<`
    LShift,
    /// `>>`
    RShift,
    /// `>>>`
    ZeroFillRShift,

    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,

    /// `|`
    BitOr,
    /// `^`
    BitXor,
    /// `&`
    BitAnd,

    /// `||`
    LogicalOr,

    /// `&&`
    LogicalAnd,

    /// `in`
    In,
    /// `instanceof`
    InstanceOf,

    /// `**`
    Exp,

    /// `??`
    NullishCoalescing,
}

impl ExtraDataCompact for BinaryOp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

impl BinaryOp {
    pub fn precedence(self) -> u8 {
        match self {
            BinaryOp::EqEq => 6,
            BinaryOp::NotEq => 6,
            BinaryOp::EqEqEq => 6,
            BinaryOp::NotEqEq => 6,
            BinaryOp::Lt => 7,
            BinaryOp::LtEq => 7,
            BinaryOp::Gt => 7,
            BinaryOp::GtEq => 7,
            BinaryOp::LShift => 8,
            BinaryOp::RShift => 8,
            BinaryOp::ZeroFillRShift => 8,

            BinaryOp::Add => 9,
            BinaryOp::Sub => 9,
            BinaryOp::Mul => 10,
            BinaryOp::Div => 10,
            BinaryOp::Mod => 10,

            BinaryOp::BitOr => 3,
            BinaryOp::BitXor => 4,

            BinaryOp::BitAnd => 5,

            BinaryOp::LogicalOr => 1,

            BinaryOp::LogicalAnd => 2,
            BinaryOp::In => 7,
            BinaryOp::InstanceOf => 7,

            BinaryOp::Exp => 11,

            BinaryOp::NullishCoalescing => 1,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub enum AssignOp {
    /// `=`
    #[default]
    Assign,
    /// `+=`
    AddAssign,
    /// `-=`
    SubAssign,
    /// `*=`
    MulAssign,
    /// `/=`
    DivAssign,
    /// `%=`
    ModAssign,
    /// `<<=`
    LShiftAssign,
    /// `>>=`
    RShiftAssign,
    /// `>>>=`
    ZeroFillRShiftAssign,
    /// `|=`
    BitOrAssign,
    /// `^=`
    BitXorAssign,
    /// `&=`
    BitAndAssign,

    /// `**=`
    ExpAssign,

    /// `&&=`
    AndAssign,

    /// `||=`
    OrAssign,

    /// `??=`
    NullishAssign,
}

impl ExtraDataCompact for AssignOp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub enum UpdateOp {
    /// `++`
    #[default]
    PlusPlus,
    /// `--`
    MinusMinus,
}

impl ExtraDataCompact for UpdateOp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub enum UnaryOp {
    /// `-`
    Minus,
    /// `+`
    Plus,
    /// `!`
    Bang,
    /// `~`
    Tilde,
    /// `typeof`
    TypeOf,
    /// `void`
    #[default]
    Void,
    /// `delete`
    Delete,
}

impl ExtraDataCompact for UnaryOp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}
