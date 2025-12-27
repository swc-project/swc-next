#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, ExtraData, NodeKind};
use crate::{ast::*, node_id::*};
impl ExtraDataCompact for Program {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { program: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.program }
    }
}
impl ExtraDataCompact for Module {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Script {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ModuleItem {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ModuleDecl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { module_decl: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.module_decl }
    }
}
impl ExtraDataCompact for ImportDecl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ImportSpecifier {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            import_specifier: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.import_specifier }
    }
}
impl ExtraDataCompact for ImportNamedSpecifier {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ImportDefaultSpecifier {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ImportStarAsSpecifier {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportDecl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for NamedExport {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportSpecifier {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            export_specifier: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.export_specifier }
    }
}
impl ExtraDataCompact for ExportNamespaceSpecifier {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ModuleExportName {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            module_export_name: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.module_export_name }
    }
}
impl ExtraDataCompact for ExportDefaultSpecifier {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportNamedSpecifier {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportDefaultDecl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for DefaultDecl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { default_decl: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.default_decl }
    }
}
impl ExtraDataCompact for ExportDefaultExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExportAll {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BlockStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Stmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { stmt: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.stmt }
    }
}
impl ExtraDataCompact for ExprStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for EmptyStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for DebuggerStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for WithStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ReturnStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for LabeledStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BreakStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ContinueStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for IfStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SwitchStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ThrowStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for TryStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for WhileStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for DoWhileStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ForStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ForInStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ForOfStmt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SwitchCase {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for CatchClause {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ForHead {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { for_head: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.for_head }
    }
}
impl ExtraDataCompact for VarDeclOrExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            var_decl_or_expr: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.var_decl_or_expr }
    }
}
impl ExtraDataCompact for Decl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { decl: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.decl }
    }
}
impl ExtraDataCompact for FnDecl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ClassDecl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for VarDecl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for VarDeclarator {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for UsingDecl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Expr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { expr: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.expr }
    }
}
impl ExtraDataCompact for ThisExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ArrayLit {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ObjectLit {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PropOrSpread {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            prop_or_spread: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.prop_or_spread }
    }
}
impl ExtraDataCompact for SpreadElement {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for UnaryExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for UpdateExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BinExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for FnExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ClassExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for MemberExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for MemberProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { member_prop: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.member_prop }
    }
}
impl ExtraDataCompact for SuperPropExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SuperProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { super_prop: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.super_prop }
    }
}
impl ExtraDataCompact for CondExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for CallExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for NewExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SeqExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ArrowExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for YieldExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for MetaPropExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AwaitExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Tpl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for TaggedTpl {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for TplElement {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ParenExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Callee {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { callee: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.callee }
    }
}
impl ExtraDataCompact for Super {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Import {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ExprOrSpread {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SpreadDot3Token {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BlockStmtOrExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            block_stmt_or_expr: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.block_stmt_or_expr }
    }
}
impl ExtraDataCompact for AssignTarget {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignTargetPat {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            assign_target_pat: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.assign_target_pat }
    }
}
impl ExtraDataCompact for SimpleAssignTarget {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            simple_assign_target: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.simple_assign_target }
    }
}
impl ExtraDataCompact for OptChainExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for OptChainBase {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            opt_chain_base: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.opt_chain_base }
    }
}
impl ExtraDataCompact for OptCall {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Invalid {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Function {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Param {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ParamOrTsParamProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            param_or_ts_param_prop: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.param_or_ts_param_prop }
    }
}
impl ExtraDataCompact for Class {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ClassMember {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { class_member: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.class_member }
    }
}
impl ExtraDataCompact for ClassProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PrivateProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ClassMethod {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PrivateMethod {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Constructor {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Decorator {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for StaticBlock {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Key {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { key: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.key }
    }
}
impl ExtraDataCompact for AutoAccessor {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Prop {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { prop: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.prop }
    }
}
impl ExtraDataCompact for KeyValueProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for GetterProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for SetterProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for MethodProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PropName {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { prop_name: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.prop_name }
    }
}
impl ExtraDataCompact for ComputedPropName {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Pat {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { pat: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.pat }
    }
}
impl ExtraDataCompact for ArrayPat {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ObjectPat {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignPat {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for RestPat {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for ObjectPatProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            object_pat_prop: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.object_pat_prop }
    }
}
impl ExtraDataCompact for KeyValuePatProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for AssignPatProp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Ident {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for IdentName {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for PrivateName {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BindingIdent {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Lit {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { lit: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.lit }
    }
}
impl ExtraDataCompact for Str {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Bool {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Null {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Number {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for BigInt {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Regex {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXObject {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { jsx_object: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_object }
    }
}
impl ExtraDataCompact for JSXMemberExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXNamespacedName {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXEmptyExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXExprContainer {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXExpr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { jsx_expr: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_expr }
    }
}
impl ExtraDataCompact for JSXSpreadChild {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXElementName {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_element_name: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_element_name }
    }
}
impl ExtraDataCompact for JSXOpeningElement {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXAttrOrSpread {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_attr_or_spread: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_attr_or_spread }
    }
}
impl ExtraDataCompact for JSXClosingElement {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXAttr {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXAttrName {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_attr_name: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_attr_name }
    }
}
impl ExtraDataCompact for JSXAttrValue {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_attr_value: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_attr_value }
    }
}
impl ExtraDataCompact for JSXText {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXElement {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXElementChild {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            jsx_element_child: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.jsx_element_child }
    }
}
impl ExtraDataCompact for JSXFragment {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXOpeningFragment {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for JSXClosingFragment {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            node: self.node_id(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { Self::from_node_id_unchecked(data.node, ast) }
    }
}
impl ExtraDataCompact for Option<ObjectLit> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| ObjectLit::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<ModuleExportName> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_module_export_name: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_module_export_name }
    }
}
impl ExtraDataCompact for Option<Str> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| Str::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<Expr> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_expr: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_expr }
    }
}
impl ExtraDataCompact for Option<Ident> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| Ident::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<Stmt> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_stmt: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_stmt }
    }
}
impl ExtraDataCompact for Option<CatchClause> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| CatchClause::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<BlockStmt> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| BlockStmt::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<VarDeclOrExpr> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_var_decl_or_expr: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_var_decl_or_expr }
    }
}
impl ExtraDataCompact for Option<Pat> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { optional_pat: self }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_pat }
    }
}
impl ExtraDataCompact for Option<ExprOrSpread> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| ExprOrSpread::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<SpreadDot3Token> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| SpreadDot3Token::from_node_id_unchecked(id, ast))
        }
    }
}
impl ExtraDataCompact for Option<JSXAttrValue> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_jsx_attr_value: self,
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe { data.optional_jsx_attr_value }
    }
}
impl ExtraDataCompact for Option<JSXClosingElement> {
    fn to_extra_data(self) -> ExtraData {
        ExtraData {
            optional_node: self.map(|n| n.node_id()).into(),
        }
    }
    unsafe fn from_extra_data(data: ExtraData, ast: &Ast) -> Self {
        unsafe {
            data.optional_node
                .to_option()
                .map(|id| JSXClosingElement::from_node_id_unchecked(id, ast))
        }
    }
}
