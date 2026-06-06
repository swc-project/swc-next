use crate::Span;
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ast_macros::ast;

use crate::ast::{ClassExpr, Decl, Expr, FnExpr, Ident, ObjectLit, Str};

#[ast]
#[derive(Debug)]
pub enum ModuleDecl<'a> {
    Import(Box<'a, ImportDecl<'a>>),
    ExportDecl(Box<'a, ExportDecl<'a>>),
    ExportNamed(Box<'a, NamedExport<'a>>),
    ExportDefaultDecl(Box<'a, ExportDefaultDecl<'a>>),
    ExportDefaultExpr(Box<'a, ExportDefaultExpr<'a>>),
    ExportAll(Box<'a, ExportAll<'a>>),
    // TsImportEquals(TsImportEqualsDecl),
    // TsExportAssignment(TsExportAssignment),
    // TsNamespaceExport(TsNamespaceExportDecl),
}

#[ast]
#[derive(Debug)]
pub struct ImportDecl<'a> {
    pub span: Span,
    pub specifiers: Vec<'a, ImportSpecifier<'a>>,
    pub src: Box<'a, Str<'a>>,
    pub type_only: bool,
    pub with: Option<Box<'a, ObjectLit<'a>>>,
    pub phase: ImportPhase,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportPhase {
    #[default]
    Evaluation,
    Source,
    Defer,
}

#[ast]
#[derive(Debug)]
pub enum ImportSpecifier<'a> {
    Named(Box<'a, ImportNamedSpecifier<'a>>),
    Default(Box<'a, ImportDefaultSpecifier<'a>>),
    Namespace(Box<'a, ImportStarAsSpecifier<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct ImportNamedSpecifier<'a> {
    pub span: Span,
    pub local: Box<'a, Ident<'a>>,
    pub imported: Option<ModuleExportName<'a>>,
    pub is_type_only: bool,
}

#[ast]
#[derive(Debug)]
pub struct ImportDefaultSpecifier<'a> {
    pub span: Span,
    pub local: Box<'a, Ident<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct ImportStarAsSpecifier<'a> {
    pub span: Span,
    pub local: Box<'a, Ident<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct ExportDecl<'a> {
    pub span: Span,
    pub decl: Decl<'a>,
}

#[ast]
#[derive(Debug)]
pub struct NamedExport<'a> {
    pub span: Span,
    pub specifiers: Vec<'a, ExportSpecifier<'a>>,
    pub src: Option<Box<'a, Str<'a>>>,
    pub type_only: bool,
    pub with: Option<Box<'a, ObjectLit<'a>>>,
}

#[ast]
#[derive(Debug)]
pub enum ExportSpecifier<'a> {
    Namespace(Box<'a, ExportNamespaceSpecifier<'a>>),
    Default(Box<'a, ExportDefaultSpecifier<'a>>),
    Named(Box<'a, ExportNamedSpecifier<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct ExportNamespaceSpecifier<'a> {
    pub span: Span,
    pub name: ModuleExportName<'a>,
}

#[ast]
#[derive(Debug)]
pub enum ModuleExportName<'a> {
    Ident(Box<'a, Ident<'a>>),
    Str(Box<'a, Str<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct ExportDefaultSpecifier<'a> {
    #[span]
    pub exported: Box<'a, Ident<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct ExportNamedSpecifier<'a> {
    pub span: Span,
    pub orig: ModuleExportName<'a>,
    pub exported: Option<ModuleExportName<'a>>,
    pub is_type_only: bool,
}

#[ast]
#[derive(Debug)]
pub struct ExportDefaultDecl<'a> {
    pub span: Span,
    pub decl: DefaultDecl<'a>,
}

#[ast]
#[derive(Debug)]
pub enum DefaultDecl<'a> {
    Class(Box<'a, ClassExpr<'a>>),
    Fn(Box<'a, FnExpr<'a>>),
    // TsInterfaceDecl(TsInterfaceDecl),
}

#[ast]
#[derive(Debug)]
pub struct ExportDefaultExpr<'a> {
    pub span: Span,
    pub expr: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct ExportAll<'a> {
    pub span: Span,
    pub src: Box<'a, Str<'a>>,
    pub type_only: bool,
    pub with: Option<Box<'a, ObjectLit<'a>>>,
}
