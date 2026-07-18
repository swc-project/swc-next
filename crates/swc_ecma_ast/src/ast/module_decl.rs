use std::mem;

use swc_experimental_ast_macros::ast;

use crate::{
    Ast, ExtraData, GetSpan,
    ast::{ClassExpr, FnExpr, Ident, Str},
    node_id::ExtraDataCompact,
};

#[ast(repr)]
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

impl crate::ExtraDataLayout for ImportPhase {
    const WORDS: usize = 1;
}

impl<'a> ExtraDataCompact<'a> for ImportPhase {
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[ast(repr)]
pub enum ImportSpecifier {
    Named(ImportNamedSpecifier),
    Default(ImportDefaultSpecifier),
    Namespace(ImportStarAsSpecifier),
}

#[ast]
pub struct ImportNamedSpecifier {
    local: Ident,
    imported: ModuleExportName,
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

#[ast(repr)]
pub enum ExportSpecifier {
    Namespace(ExportNamespaceSpecifier),
    Default(ExportDefaultSpecifier),
    Named(ExportNamedSpecifier),
}

#[ast]
pub struct ExportNamespaceSpecifier {
    name: ModuleExportName,
}

#[ast(repr)]
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
    exported: ModuleExportName,
    is_type_only: bool,
}

#[ast]
pub struct ExportDefaultDecl {
    decl: DefaultDecl,
}

#[ast(repr)]
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

impl ImportNamedSpecifier {
    #[inline]
    pub fn is_shorthand(self, ast: &Ast<'_>) -> bool {
        match self.imported(ast) {
            ModuleExportName::Ident(imported) => {
                let local = self.local(ast);
                imported.span(ast) == local.span(ast) && imported.sym(ast) == local.sym(ast)
            }
            ModuleExportName::Str(_) => false,
        }
    }
}

impl ExportNamedSpecifier {
    #[inline]
    pub fn is_shorthand(self, ast: &Ast<'_>) -> bool {
        match (self.orig(ast), self.exported(ast)) {
            (ModuleExportName::Ident(orig), ModuleExportName::Ident(exported)) => {
                orig.span(ast) == exported.span(ast) && orig.sym(ast) == exported.sym(ast)
            }
            (ModuleExportName::Str(orig), ModuleExportName::Str(exported)) => {
                orig.span(ast) == exported.span(ast) && orig.value(ast) == exported.value(ast)
            }
            _ => false,
        }
    }
}
