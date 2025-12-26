use std::mem;

use swc_experimental_ast_macros::ast;

use crate::{
    Ast, ExtraData,
    ast::{ClassExpr, FnExpr, Ident, Str},
    node_id::ExtraDataCompact,
};

#[ast]
pub enum ModuleDecl {
    Import(ImportDecl),
    ExportDecl(ExportDecl),
    ExportNamed(NamedExport),
    ExportDefaultDecl(ExportDefaultDecl),
    ExportDefaultExpr(ExportDefaultExpr),
    ExportAll(ExportAll),
    // TsImportEquals(TsImportEqualsDecl),
    // TsExportAssignment(TsExportAssignment),
    // TsNamespaceExport(TsNamespaceExportDecl),
}

#[ast]
pub struct ImportDecl {
    specifiers: Vec<ImportSpecifier>,
    src: Str,
    type_only: bool,
    with: Option<ObjectLit>,
    phase: ImportPhase,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportPhase {
    #[default]
    Evaluation,
    Source,
    Defer,
}

impl ExtraDataCompact for ImportPhase {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[ast]
pub enum ImportSpecifier {
    Named(ImportNamedSpecifier),
    Default(ImportDefaultSpecifier),
    Namespace(ImportStarAsSpecifier),
}

#[ast]
pub struct ImportNamedSpecifier {
    local: Ident,
    imported: Option<ModuleExportName>,
    is_type_only: bool,
}

#[ast]
pub struct ImportDefaultSpecifier {
    local: Ident,
}

#[ast]
pub struct ImportStarAsSpecifier {
    local: Ident,
}

#[ast]
pub struct ExportDecl {
    decl: Decl,
}

#[ast]
pub struct NamedExport {
    specifiers: Vec<ExportSpecifier>,
    src: Option<Str>,
    type_only: bool,
    with: Option<ObjectLit>,
}

#[ast]
pub enum ExportSpecifier {
    Namespace(ExportNamespaceSpecifier),
    Default(ExportDefaultSpecifier),
    Named(ExportNamedSpecifier),
}

#[ast]
pub struct ExportNamespaceSpecifier {
    name: ModuleExportName,
}

#[ast]
pub enum ModuleExportName {
    Ident(Ident),
    Str(Str),
}

#[ast]
pub struct ExportDefaultSpecifier {
    exported: Ident,
}

#[ast]
pub struct ExportNamedSpecifier {
    orig: ModuleExportName,
    exported: Option<ModuleExportName>,
    is_type_only: bool,
}

#[ast]
pub struct ExportDefaultDecl {
    decl: DefaultDecl,
}

#[ast]
pub enum DefaultDecl {
    Class(ClassExpr),
    Fn(FnExpr),
    // TsInterfaceDecl(TsInterfaceDecl),
}

#[ast]
pub struct ExportDefaultExpr {
    expr: Expr,
}

#[ast]
pub struct ExportAll {
    src: Str,
    type_only: bool,
    with: Option<ObjectLit>,
}
