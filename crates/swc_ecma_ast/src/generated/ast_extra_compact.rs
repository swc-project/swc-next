#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, ExtraData};
use crate::{ast::*, node_id::*};
use oxc_index::Idx;
impl<'a> ExtraDataCompact<'a> for Program {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Module {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Script {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ModuleItem {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ModuleDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportNamedSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportDefaultSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportStarAsSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for NamedExport {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportNamespaceSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ModuleExportName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportNamedSpecifier {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for DefaultDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportDefaultExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExportAll {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BlockStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Stmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExprStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for EmptyStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for DebuggerStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for WithStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ReturnStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for LabeledStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BreakStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ContinueStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for IfStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SwitchStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ThrowStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TryStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for WhileStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for DoWhileStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ForStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ForInStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ForOfStmt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SwitchCase {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for CatchClause {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ForHead {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for VarDeclOrExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Decl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for FnDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ClassDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for VarDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for VarDeclarator {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for UsingDecl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Expr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ThisExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ArrayLit {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectLit {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for PropOrSpread {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SpreadElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for UnaryExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for UpdateExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BinExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for FnExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ClassExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AssignExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for MemberExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for MemberProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SuperPropExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SuperProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for CondExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ImportExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for CallExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for NewExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SeqExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ArrowExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for YieldExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for MetaPropExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AwaitExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Tpl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TaggedTpl {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for TplElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ParenExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Callee {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Super {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ExprOrSpread {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BlockStmtOrExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AssignTarget {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AssignTargetPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SimpleAssignTarget {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for OptChainExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for OptChainBase {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for OptCall {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Invalid {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Function {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Param {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ParamOrTsParamProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Class {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ClassMember {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ClassProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ClassMethod {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateMethod {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Constructor {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Decorator {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for StaticBlock {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Key {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AutoAccessor {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Prop {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for KeyValueProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AssignProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for GetterProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for SetterProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for MethodProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for PropName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ComputedPropName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Pat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ArrayPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AssignPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for RestPat {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for ObjectPatProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for KeyValuePatProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for AssignPatProp {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Ident {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for IdentName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for PrivateName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BindingIdent {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Lit {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Str {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Bool {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Null {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Number {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for BigInt {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Regex {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXObject {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXMemberExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXNamespacedName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXEmptyExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXExprContainer {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXExpr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXSpreadChild {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXElementName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXOpeningElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXAttrOrSpread {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXClosingElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXAttr {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXAttrName {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXAttrValue {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXText {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXElement {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXElementChild {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXFragment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXOpeningFragment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for JSXClosingFragment {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.node_id(),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        Self(unsafe { data.node_id })
    }
}
impl<'a> ExtraDataCompact<'a> for Option<ObjectLit> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| ObjectLit(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Str> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| Str(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Expr> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| Expr(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Ident> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| Ident(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Stmt> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| Stmt(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<CatchClause> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| CatchClause(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<BlockStmt> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| BlockStmt(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<VarDeclOrExpr> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| VarDeclOrExpr(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<Pat> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| Pat(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<ExprOrSpread> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| ExprOrSpread(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<JSXAttrValue> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| JSXAttrValue(node_id))
    }
}
impl<'a> ExtraDataCompact<'a> for Option<JSXClosingElement> {
    #[inline]
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            node_id: self.map_or(NodeId::from_raw_unchecked(u32::MAX), |node| node.node_id()),
        }
    }
    #[inline]
    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let node_id = unsafe { data.node_id };
        (node_id.index() != u32::MAX as usize).then(|| JSXClosingElement(node_id))
    }
}
