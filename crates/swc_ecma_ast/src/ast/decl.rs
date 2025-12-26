use std::mem;

use swc_experimental_ast_macros::ast;

use crate::{Ast, ExtraData, node_id::ExtraDataCompact};

#[ast]
pub enum Decl {
    Class(ClassDecl),
    Fn(FnDecl),
    Var(VarDecl),
    Using(UsingDecl),
    // TsInterface(Box<TsInterfaceDecl>),
    // TsTypeAlias(Box<TsTypeAliasDecl>),
    // TsEnum(Box<TsEnumDecl>),
    // TsModule(Box<TsModuleDecl>),
}

#[ast]
pub struct FnDecl {
    ident: Ident,
    declare: bool,
    function: Function,
}

#[ast]
pub struct ClassDecl {
    ident: Ident,
    declare: bool,
    class: Class,
}

#[ast]
pub struct VarDecl {
    kind: VarDeclKind,
    declare: bool,
    decls: Vec<VarDeclarator>,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum VarDeclKind {
    Var,
    Let,
    Const,
}

impl ExtraDataCompact for VarDeclKind {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[ast]
pub struct VarDeclarator {
    name: Pat,
    init: Option<Expr>,
    // pub definite: bool,
}

#[ast]
pub struct UsingDecl {
    is_await: bool,
    decls: Vec<VarDeclarator>,
}
