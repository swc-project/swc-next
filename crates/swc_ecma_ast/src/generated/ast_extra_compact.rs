#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, ExtraData};
use crate::{ast::*, node_id::*};
use oxc_index::Idx;
const _: () = assert!(std::mem::size_of::<Program>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for Program {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_program: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_program) }
    }
}
impl<'a> ExtraDataCompact<'a> for Module {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_module: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_module }
    }
}
impl<'a> ExtraDataCompact<'a> for Script {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_script: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_script }
    }
}
const _: () = assert!(std::mem::size_of::<ModuleItem>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for ModuleItem {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_module_item: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_module_item) }
    }
}
const _: () = assert!(std::mem::size_of::<ModuleDecl>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for ModuleDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_module_decl: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_module_decl) }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_import_decl: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_import_decl }
    }
}
const _: () = assert!(std::mem::size_of::<ImportSpecifier>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for ImportSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_import_specifier: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_import_specifier) }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportNamedSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_import_named_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_import_named_specifier }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportDefaultSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_import_default_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_import_default_specifier }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportStarAsSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_import_star_as_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_import_star_as_specifier }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_export_decl: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_export_decl }
    }
}
impl<'a> ExtraDataCompact<'a> for NamedExport {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_named_export: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_named_export }
    }
}
const _: () = assert!(std::mem::size_of::<ExportSpecifier>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for ExportSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_export_specifier: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_export_specifier) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportNamespaceSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_export_namespace_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_export_namespace_specifier }
    }
}
const _: () = assert!(std::mem::size_of::<ModuleExportName>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for ModuleExportName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_module_export_name: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_module_export_name) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_export_default_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_export_default_specifier }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportNamedSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_export_named_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_export_named_specifier }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_export_default_decl: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_export_default_decl }
    }
}
const _: () = assert!(std::mem::size_of::<DefaultDecl>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for DefaultDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_default_decl: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_default_decl) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_export_default_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_export_default_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportAll {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_export_all: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_export_all }
    }
}
impl<'a> ExtraDataCompact<'a> for BlockStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_block_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_block_stmt }
    }
}
const _: () = assert!(std::mem::size_of::<Stmt>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for Stmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_stmt: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_stmt) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExprStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_expr_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_expr_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for EmptyStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_empty_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_empty_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for DebuggerStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_debugger_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_debugger_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for WithStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_with_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_with_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for ReturnStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_return_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_return_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for LabeledStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_labeled_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_labeled_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for BreakStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_break_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_break_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for ContinueStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_continue_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_continue_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for IfStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_if_stmt: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_if_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for SwitchStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_switch_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_switch_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for ThrowStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_throw_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_throw_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for TryStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_try_stmt: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_try_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for WhileStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_while_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_while_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for DoWhileStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_do_while_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_do_while_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for ForStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_for_stmt: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_for_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for ForInStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_for_in_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_for_in_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for ForOfStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_for_of_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_for_of_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for SwitchCase {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_switch_case: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_switch_case }
    }
}
impl<'a> ExtraDataCompact<'a> for CatchClause {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_catch_clause: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_catch_clause }
    }
}
const _: () = assert!(std::mem::size_of::<ForHead>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for ForHead {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_for_head: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_for_head) }
    }
}
const _: () = assert!(std::mem::size_of::<VarDeclOrExpr>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for VarDeclOrExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_var_decl_or_expr: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_var_decl_or_expr) }
    }
}
const _: () = assert!(std::mem::size_of::<Decl>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for Decl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_decl: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_decl) }
    }
}
impl<'a> ExtraDataCompact<'a> for FnDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_fn_decl: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_fn_decl }
    }
}
impl<'a> ExtraDataCompact<'a> for ClassDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_class_decl: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_class_decl }
    }
}
impl<'a> ExtraDataCompact<'a> for VarDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_var_decl: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_var_decl }
    }
}
impl<'a> ExtraDataCompact<'a> for VarDeclarator {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_var_declarator: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_var_declarator }
    }
}
impl<'a> ExtraDataCompact<'a> for UsingDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_using_decl: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_using_decl }
    }
}
const _: () = assert!(std::mem::size_of::<Expr>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for Expr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_expr: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_expr) }
    }
}
impl<'a> ExtraDataCompact<'a> for ThisExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_this_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_this_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for ArrayLit {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_array_lit: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_array_lit }
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectLit {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_object_lit: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_object_lit }
    }
}
const _: () = assert!(std::mem::size_of::<PropOrSpread>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for PropOrSpread {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_prop_or_spread: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_prop_or_spread) }
    }
}
impl<'a> ExtraDataCompact<'a> for SpreadElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_spread_element: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_spread_element }
    }
}
impl<'a> ExtraDataCompact<'a> for UnaryExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_unary_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_unary_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for UpdateExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_update_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_update_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for BinExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_bin_expr: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_bin_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for FnExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_fn_expr: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_fn_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for ClassExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_class_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_class_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_assign_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_assign_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for MemberExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_member_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_member_expr }
    }
}
const _: () = assert!(std::mem::size_of::<MemberProp>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for MemberProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_member_prop: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_member_prop) }
    }
}
impl<'a> ExtraDataCompact<'a> for SuperPropExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_super_prop_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_super_prop_expr }
    }
}
const _: () = assert!(std::mem::size_of::<SuperProp>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for SuperProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_super_prop: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_super_prop) }
    }
}
impl<'a> ExtraDataCompact<'a> for CondExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_cond_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_cond_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_import_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_import_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for CallExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_call_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_call_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for NewExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_new_expr: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_new_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for SeqExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_seq_expr: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_seq_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for ArrowExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_arrow_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_arrow_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for YieldExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_yield_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_yield_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for MetaPropExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_meta_prop_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_meta_prop_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for AwaitExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_await_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_await_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for Tpl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_tpl: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_tpl }
    }
}
impl<'a> ExtraDataCompact<'a> for TaggedTpl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_tagged_tpl: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_tagged_tpl }
    }
}
impl<'a> ExtraDataCompact<'a> for TplElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_tpl_element: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_tpl_element }
    }
}
impl<'a> ExtraDataCompact<'a> for ParenExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_paren_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_paren_expr }
    }
}
const _: () = assert!(std::mem::size_of::<Callee>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for Callee {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_callee: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_callee) }
    }
}
impl<'a> ExtraDataCompact<'a> for Super {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_super: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_super }
    }
}
impl<'a> ExtraDataCompact<'a> for ExprOrSpread {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_expr_or_spread: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_expr_or_spread }
    }
}
const _: () = assert!(std::mem::size_of::<BlockStmtOrExpr>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for BlockStmtOrExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_block_stmt_or_expr: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_block_stmt_or_expr) }
    }
}
const _: () = assert!(std::mem::size_of::<AssignTarget>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for AssignTarget {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_assign_target: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_assign_target) }
    }
}
const _: () = assert!(std::mem::size_of::<AssignTargetPat>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for AssignTargetPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_assign_target_pat: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_assign_target_pat) }
    }
}
const _: () = assert!(std::mem::size_of::<SimpleAssignTarget>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for SimpleAssignTarget {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_simple_assign_target: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_simple_assign_target) }
    }
}
impl<'a> ExtraDataCompact<'a> for OptChainExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_opt_chain_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_opt_chain_expr }
    }
}
const _: () = assert!(std::mem::size_of::<OptChainBase>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for OptChainBase {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_opt_chain_base: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_opt_chain_base) }
    }
}
impl<'a> ExtraDataCompact<'a> for OptCall {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_opt_call: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_opt_call }
    }
}
impl<'a> ExtraDataCompact<'a> for Invalid {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_invalid: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_invalid }
    }
}
impl<'a> ExtraDataCompact<'a> for Function {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_function: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_function }
    }
}
impl<'a> ExtraDataCompact<'a> for Param {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_param: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_param }
    }
}
const _: () = assert!(std::mem::size_of::<ParamOrTsParamProp>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for ParamOrTsParamProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_param_or_ts_param_prop: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_param_or_ts_param_prop) }
    }
}
impl<'a> ExtraDataCompact<'a> for Class {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_class: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_class }
    }
}
const _: () = assert!(std::mem::size_of::<ClassMember>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for ClassMember {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_class_member: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_class_member) }
    }
}
impl<'a> ExtraDataCompact<'a> for ClassProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_class_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_class_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_private_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_private_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for ClassMethod {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_class_method: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_class_method }
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateMethod {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_private_method: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_private_method }
    }
}
impl<'a> ExtraDataCompact<'a> for Constructor {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_constructor: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_constructor }
    }
}
impl<'a> ExtraDataCompact<'a> for Decorator {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_decorator: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_decorator }
    }
}
impl<'a> ExtraDataCompact<'a> for StaticBlock {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_static_block: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_static_block }
    }
}
const _: () = assert!(std::mem::size_of::<Key>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for Key {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_key: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_key) }
    }
}
impl<'a> ExtraDataCompact<'a> for AutoAccessor {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_auto_accessor: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_auto_accessor }
    }
}
const _: () = assert!(std::mem::size_of::<Prop>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for Prop {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_prop: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_prop) }
    }
}
impl<'a> ExtraDataCompact<'a> for KeyValueProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_key_value_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_key_value_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_assign_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_assign_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for GetterProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_getter_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_getter_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for SetterProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_setter_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_setter_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for MethodProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_method_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_method_prop }
    }
}
const _: () = assert!(std::mem::size_of::<PropName>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for PropName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_prop_name: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_prop_name) }
    }
}
impl<'a> ExtraDataCompact<'a> for ComputedPropName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_computed_prop_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_computed_prop_name }
    }
}
const _: () = assert!(std::mem::size_of::<Pat>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for Pat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_pat: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_pat) }
    }
}
impl<'a> ExtraDataCompact<'a> for ArrayPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_array_pat: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_array_pat }
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_object_pat: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_object_pat }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_assign_pat: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_assign_pat }
    }
}
impl<'a> ExtraDataCompact<'a> for RestPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_rest_pat: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_rest_pat }
    }
}
const _: () = assert!(std::mem::size_of::<ObjectPatProp>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for ObjectPatProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_object_pat_prop: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_object_pat_prop) }
    }
}
impl<'a> ExtraDataCompact<'a> for KeyValuePatProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_key_value_pat_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_key_value_pat_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignPatProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_assign_pat_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_assign_pat_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for Ident {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_ident: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_ident }
    }
}
impl<'a> ExtraDataCompact<'a> for IdentName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_ident_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_ident_name }
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_private_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_private_name }
    }
}
impl<'a> ExtraDataCompact<'a> for BindingIdent {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_binding_ident: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_binding_ident }
    }
}
const _: () = assert!(std::mem::size_of::<Lit>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for Lit {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_lit: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_lit) }
    }
}
impl<'a> ExtraDataCompact<'a> for Str {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_str: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_str }
    }
}
impl<'a> ExtraDataCompact<'a> for Bool {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_bool: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_bool }
    }
}
impl<'a> ExtraDataCompact<'a> for Null {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_null: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_null }
    }
}
impl<'a> ExtraDataCompact<'a> for Number {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_number: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_number }
    }
}
impl<'a> ExtraDataCompact<'a> for BigInt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_big_int: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_big_int }
    }
}
impl<'a> ExtraDataCompact<'a> for Regex {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_regex: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_regex }
    }
}
const _: () = assert!(std::mem::size_of::<JSXObject>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for JSXObject {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_object: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_jsx_object) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXMemberExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_member_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_member_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXNamespacedName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_namespaced_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_namespaced_name }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXEmptyExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_empty_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_empty_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXExprContainer {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_expr_container: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_expr_container }
    }
}
const _: () = assert!(std::mem::size_of::<JSXExpr>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for JSXExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_expr: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_jsx_expr) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXSpreadChild {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_spread_child: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_spread_child }
    }
}
const _: () = assert!(std::mem::size_of::<JSXElementName>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for JSXElementName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_element_name: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_jsx_element_name) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXOpeningElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_opening_element: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_opening_element }
    }
}
const _: () = assert!(std::mem::size_of::<JSXAttrOrSpread>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for JSXAttrOrSpread {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_attr_or_spread: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_jsx_attr_or_spread) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXClosingElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_closing_element: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_closing_element }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXAttr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_jsx_attr: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_attr }
    }
}
const _: () = assert!(std::mem::size_of::<JSXAttrName>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for JSXAttrName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_attr_name: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_jsx_attr_name) }
    }
}
const _: () = assert!(std::mem::size_of::<JSXAttrValue>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for JSXAttrValue {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_attr_value: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_jsx_attr_value) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXText {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData { ast_jsx_text: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_text }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_element: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_element }
    }
}
const _: () = assert!(std::mem::size_of::<JSXElementChild>() == std::mem::size_of::<u64>());
impl<'a> ExtraDataCompact<'a> for JSXElementChild {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_element_child: unsafe { std::mem::transmute::<Self, u64>(self) },
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { std::mem::transmute::<u64, Self>(data.ast_jsx_element_child) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXFragment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_fragment: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_fragment }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXOpeningFragment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_opening_fragment: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_opening_fragment }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXClosingFragment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            ast_jsx_closing_fragment: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        unsafe { data.ast_jsx_closing_fragment }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<ObjectLit> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_object_lit: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_ast_object_lit }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Str> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_str: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_ast_str }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Expr> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_expr: self
                .map_or(u64::MAX, |node| unsafe { node.to_extra_data(ast).other }),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        {
            let raw = unsafe { data.optional_ast_expr };
            (raw != u64::MAX).then(|| unsafe {
                ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast)
            })
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Ident> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_ident: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_ast_ident }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Stmt> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_stmt: self
                .map_or(u64::MAX, |node| unsafe { node.to_extra_data(ast).other }),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        {
            let raw = unsafe { data.optional_ast_stmt };
            (raw != u64::MAX).then(|| unsafe {
                ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast)
            })
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<CatchClause> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_catch_clause: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_ast_catch_clause }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<BlockStmt> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_block_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_ast_block_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<VarDeclOrExpr> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_var_decl_or_expr: self
                .map_or(u64::MAX, |node| unsafe { node.to_extra_data(ast).other }),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        {
            let raw = unsafe { data.optional_ast_var_decl_or_expr };
            (raw != u64::MAX).then(|| unsafe {
                ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast)
            })
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Pat> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_pat: self
                .map_or(u64::MAX, |node| unsafe { node.to_extra_data(ast).other }),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        {
            let raw = unsafe { data.optional_ast_pat };
            (raw != u64::MAX).then(|| unsafe {
                ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast)
            })
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<ExprOrSpread> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_expr_or_spread: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_ast_expr_or_spread }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<JSXAttrValue> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_jsx_attr_value: self
                .map_or(u64::MAX, |node| unsafe { node.to_extra_data(ast).other }),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        {
            let raw = unsafe { data.optional_ast_jsx_attr_value };
            (raw != u64::MAX).then(|| unsafe {
                ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast)
            })
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<JSXClosingElement> {
    #[inline]
    fn to_extra_data(self, ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            optional_ast_jsx_closing_element: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_ast_jsx_closing_element }
    }
}
