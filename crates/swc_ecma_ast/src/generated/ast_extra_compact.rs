#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, ExtraData, NodeKind};
use crate::{ast::*, node_id::*};
impl<'a> ExtraDataCompact<'a> for Program {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { program: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.program }
    }
}
impl<'a> ExtraDataCompact<'a> for Module {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Script {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ModuleItem {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ModuleDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { module_decl: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.module_decl }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            import_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.import_specifier }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportNamedSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportDefaultSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportStarAsSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for NamedExport {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            export_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.export_specifier }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportNamespaceSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ModuleExportName {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            module_export_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.module_export_name }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportNamedSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for DefaultDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { default_decl: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.default_decl }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExportAll {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for BlockStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Stmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { stmt: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for ExprStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for EmptyStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for DebuggerStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for WithStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ReturnStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for LabeledStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for BreakStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ContinueStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for IfStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for SwitchStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ThrowStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for TryStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for WhileStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for DoWhileStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ForStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ForInStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ForOfStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for SwitchCase {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for CatchClause {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ForHead {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { for_head: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.for_head }
    }
}
impl<'a> ExtraDataCompact<'a> for VarDeclOrExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            var_decl_or_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.var_decl_or_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for Decl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { decl: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.decl }
    }
}
impl<'a> ExtraDataCompact<'a> for FnDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ClassDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for VarDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for VarDeclarator {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for UsingDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Expr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { expr: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.expr }
    }
}
impl<'a> ExtraDataCompact<'a> for ThisExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ArrayLit {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectLit {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for PropOrSpread {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            prop_or_spread: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.prop_or_spread }
    }
}
impl<'a> ExtraDataCompact<'a> for SpreadElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for UnaryExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for UpdateExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for BinExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for FnExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ClassExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for MemberExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for MemberProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { member_prop: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.member_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for SuperPropExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for SuperProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { super_prop: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.super_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for CondExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ImportExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for CallExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for NewExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for SeqExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ArrowExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for YieldExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for MetaPropExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for AwaitExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Tpl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for TaggedTpl {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for TplElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ParenExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Callee {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { callee: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.callee }
    }
}
impl<'a> ExtraDataCompact<'a> for Super {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ExprOrSpread {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for BlockStmtOrExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            block_stmt_or_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.block_stmt_or_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignTarget {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignTargetPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            assign_target_pat: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.assign_target_pat }
    }
}
impl<'a> ExtraDataCompact<'a> for SimpleAssignTarget {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            simple_assign_target: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.simple_assign_target }
    }
}
impl<'a> ExtraDataCompact<'a> for OptChainExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for OptChainBase {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            opt_chain_base: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.opt_chain_base }
    }
}
impl<'a> ExtraDataCompact<'a> for OptCall {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Invalid {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Function {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Param {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ParamOrTsParamProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            param_or_ts_param_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.param_or_ts_param_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for Class {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ClassMember {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { class_member: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.class_member }
    }
}
impl<'a> ExtraDataCompact<'a> for ClassProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ClassMethod {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateMethod {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Constructor {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Decorator {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for StaticBlock {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Key {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { key: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.key }
    }
}
impl<'a> ExtraDataCompact<'a> for AutoAccessor {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Prop {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { prop: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.prop }
    }
}
impl<'a> ExtraDataCompact<'a> for KeyValueProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for GetterProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for SetterProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for MethodProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for PropName {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { prop_name: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.prop_name }
    }
}
impl<'a> ExtraDataCompact<'a> for ComputedPropName {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Pat {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { pat: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.pat }
    }
}
impl<'a> ExtraDataCompact<'a> for ArrayPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for RestPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectPatProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            object_pat_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.object_pat_prop }
    }
}
impl<'a> ExtraDataCompact<'a> for KeyValuePatProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for AssignPatProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Ident {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for IdentName {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateName {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for BindingIdent {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Lit {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { lit: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.lit }
    }
}
impl<'a> ExtraDataCompact<'a> for Str {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Bool {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Null {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Number {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for BigInt {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Regex {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXObject {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { jsx_object: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.jsx_object }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXMemberExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXNamespacedName {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXEmptyExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXExprContainer {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { jsx_expr: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.jsx_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXSpreadChild {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXElementName {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            jsx_element_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.jsx_element_name }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXOpeningElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXAttrOrSpread {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            jsx_attr_or_spread: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.jsx_attr_or_spread }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXClosingElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXAttr {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXAttrName {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            jsx_attr_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.jsx_attr_name }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXAttrValue {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            jsx_attr_value: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.jsx_attr_value }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXText {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXElementChild {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            jsx_element_child: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.jsx_element_child }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXFragment {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXOpeningFragment {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for JSXClosingFragment {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<ObjectLit> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| ObjectLit::from_node_id_unchecked(id, ast))
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Str> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| Str::from_node_id_unchecked(id, ast))
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Expr> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Ident> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| Ident::from_node_id_unchecked(id, ast))
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Stmt> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_stmt }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<CatchClause> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| CatchClause::from_node_id_unchecked(id, ast))
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<BlockStmt> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| BlockStmt::from_node_id_unchecked(id, ast))
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<VarDeclOrExpr> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_var_decl_or_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_var_decl_or_expr }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Pat> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData { optional_pat: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_pat }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<ExprOrSpread> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| ExprOrSpread::from_node_id_unchecked(id, ast))
        }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<JSXAttrValue> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_jsx_attr_value: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe { data.optional_jsx_attr_value }
    }
}
impl<'a> ExtraDataCompact<'a> for Option<JSXClosingElement> {
    #[inline]
    fn to_extra_data(self) -> ExtraData<'a> {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData<'a>, ast: &Ast<'a>) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| JSXClosingElement::from_node_id_unchecked(id, ast))
        }
    }
}
