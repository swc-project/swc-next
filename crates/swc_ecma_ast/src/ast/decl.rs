use crate::Span;
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ast_macros::ast;

use crate::ast::{Class, Expr, Function, Ident, Pat};

#[ast]
#[derive(Debug)]
pub enum Decl<'a> {
    Class(Box<'a, ClassDecl<'a>>),
    Fn(Box<'a, FnDecl<'a>>),
    Var(Box<'a, VarDecl<'a>>),
    Using(Box<'a, UsingDecl<'a>>),
    // TsInterface(Box<TsInterfaceDecl>),
    // TsTypeAlias(Box<TsTypeAliasDecl>),
    // TsEnum(Box<TsEnumDecl>),
    // TsModule(Box<TsModuleDecl>),
}

#[ast]
#[derive(Debug)]
pub struct FnDecl<'a> {
    pub ident: Box<'a, Ident<'a>>,
    pub declare: bool,
    pub function: Box<'a, Function<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct ClassDecl<'a> {
    pub ident: Box<'a, Ident<'a>>,
    pub declare: bool,
    pub class: Box<'a, Class<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct VarDecl<'a> {
    pub span: Span,
    pub kind: VarDeclKind,
    pub declare: bool,
    pub decls: Vec<'a, VarDeclarator<'a>>,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum VarDeclKind {
    Var,
    Let,
    Const,
}

#[ast]
#[derive(Debug)]
pub struct VarDeclarator<'a> {
    pub span: Span,
    pub name: Pat<'a>,
    pub init: Option<Expr<'a>>,
    // pub definite: bool,
}

#[ast]
#[derive(Debug)]
pub struct UsingDecl<'a> {
    pub span: Span,
    pub is_await: bool,
    pub decls: Vec<'a, VarDeclarator<'a>>,
}
