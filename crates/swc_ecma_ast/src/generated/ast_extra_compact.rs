#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, ExtraData, NodeKind};
use crate::{ast::*, node_id::*};
impl ExtraDataCompact for Program {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { program: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.program }
    }
}
impl ExtraDataCompact for Module {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Script {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ModuleItem {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ModuleDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { module_decl: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.module_decl }
    }
}
impl ExtraDataCompact for ImportDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ImportSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            import_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.import_specifier }
    }
}
impl ExtraDataCompact for ImportNamedSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ImportDefaultSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ImportStarAsSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for NamedExport {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            export_specifier: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.export_specifier }
    }
}
impl ExtraDataCompact for ExportNamespaceSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ModuleExportName {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            module_export_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.module_export_name }
    }
}
impl ExtraDataCompact for ExportDefaultSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportNamedSpecifier {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportDefaultDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for DefaultDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { default_decl: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.default_decl }
    }
}
impl ExtraDataCompact for ExportDefaultExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportAll {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BlockStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Stmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { stmt: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.stmt }
    }
}
impl ExtraDataCompact for ExprStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for EmptyStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for DebuggerStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for WithStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ReturnStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for LabeledStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BreakStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ContinueStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for IfStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SwitchStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ThrowStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for TryStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for WhileStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for DoWhileStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ForStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ForInStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ForOfStmt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SwitchCase {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for CatchClause {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ForHead {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { for_head: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.for_head }
    }
}
impl ExtraDataCompact for VarDeclOrExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            var_decl_or_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.var_decl_or_expr }
    }
}
impl ExtraDataCompact for Decl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { decl: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.decl }
    }
}
impl ExtraDataCompact for FnDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ClassDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for VarDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for VarDeclarator {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for UsingDecl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Expr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { expr: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.expr }
    }
}
impl ExtraDataCompact for ThisExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ArrayLit {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ObjectLit {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PropOrSpread {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            prop_or_spread: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.prop_or_spread }
    }
}
impl ExtraDataCompact for SpreadElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for UnaryExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for UpdateExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BinExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for FnExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ClassExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for MemberExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for MemberProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { member_prop: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.member_prop }
    }
}
impl ExtraDataCompact for SuperPropExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SuperProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { super_prop: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.super_prop }
    }
}
impl ExtraDataCompact for CondExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for CallExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for NewExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SeqExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ArrowExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for YieldExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for MetaPropExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AwaitExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Tpl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for TaggedTpl {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for TplElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ParenExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Callee {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { callee: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.callee }
    }
}
impl ExtraDataCompact for Super {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Import {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExprOrSpread {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SpreadDot3Token {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BlockStmtOrExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            block_stmt_or_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.block_stmt_or_expr }
    }
}
impl ExtraDataCompact for AssignTarget {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignTargetPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            assign_target_pat: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.assign_target_pat }
    }
}
impl ExtraDataCompact for SimpleAssignTarget {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            simple_assign_target: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.simple_assign_target }
    }
}
impl ExtraDataCompact for OptChainExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for OptChainBase {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            opt_chain_base: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.opt_chain_base }
    }
}
impl ExtraDataCompact for OptCall {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Invalid {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Function {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Param {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ParamOrTsParamProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            param_or_ts_param_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.param_or_ts_param_prop }
    }
}
impl ExtraDataCompact for Class {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ClassMember {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { class_member: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.class_member }
    }
}
impl ExtraDataCompact for ClassProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PrivateProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ClassMethod {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PrivateMethod {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Constructor {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Decorator {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for StaticBlock {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Key {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { key: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.key }
    }
}
impl ExtraDataCompact for AutoAccessor {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Prop {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { prop: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.prop }
    }
}
impl ExtraDataCompact for KeyValueProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for GetterProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SetterProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for MethodProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PropName {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { prop_name: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.prop_name }
    }
}
impl ExtraDataCompact for ComputedPropName {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Pat {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { pat: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.pat }
    }
}
impl ExtraDataCompact for ArrayPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ObjectPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for RestPat {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ObjectPatProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            object_pat_prop: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.object_pat_prop }
    }
}
impl ExtraDataCompact for KeyValuePatProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignPatProp {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Ident {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for IdentName {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PrivateName {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BindingIdent {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Lit {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { lit: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.lit }
    }
}
impl ExtraDataCompact for Str {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Bool {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Null {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Number {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BigInt {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Regex {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXObject {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { jsx_object: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_object }
    }
}
impl ExtraDataCompact for JSXMemberExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXNamespacedName {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXEmptyExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXExprContainer {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXExpr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { jsx_expr: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_expr }
    }
}
impl ExtraDataCompact for JSXSpreadChild {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXElementName {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_element_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_element_name }
    }
}
impl ExtraDataCompact for JSXOpeningElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXAttrOrSpread {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_attr_or_spread: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_attr_or_spread }
    }
}
impl ExtraDataCompact for JSXClosingElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXAttr {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXAttrName {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_attr_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_attr_name }
    }
}
impl ExtraDataCompact for JSXAttrValue {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_attr_value: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_attr_value }
    }
}
impl ExtraDataCompact for JSXText {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXElement {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXElementChild {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_element_child: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_element_child }
    }
}
impl ExtraDataCompact for JSXFragment {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXOpeningFragment {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXClosingFragment {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Option<ObjectLit> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| ObjectLit::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<ModuleExportName> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_module_export_name: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_module_export_name }
    }
}
impl ExtraDataCompact for Option<Str> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| Str::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<Expr> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_expr }
    }
}
impl ExtraDataCompact for Option<Ident> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| Ident::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<Stmt> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_stmt: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_stmt }
    }
}
impl ExtraDataCompact for Option<CatchClause> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| CatchClause::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<BlockStmt> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| BlockStmt::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<VarDeclOrExpr> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_var_decl_or_expr: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_var_decl_or_expr }
    }
}
impl ExtraDataCompact for Option<Pat> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData { optional_pat: self }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_pat }
    }
}
impl ExtraDataCompact for Option<ExprOrSpread> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| ExprOrSpread::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<SpreadDot3Token> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| SpreadDot3Token::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<JSXAttrValue> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_jsx_attr_value: self,
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_jsx_attr_value }
    }
}
impl ExtraDataCompact for Option<JSXClosingElement> {
    #[inline]
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| JSXClosingElement::from_node_id_unchecked(id, ast))
        }
    }
}
