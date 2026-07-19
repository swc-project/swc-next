#![allow(unused)]
use crate::{Ast, GetSpan, NodeKind, SetSpan};
use crate::{ast::*, node_id::*};
impl Program {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Program {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Program {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Program {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Module {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Module {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Module {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Module {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Script {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Script {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Script {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Script {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ModuleItem {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ModuleItem {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ModuleItem {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ModuleItem {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ModuleDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ModuleDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ModuleDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ModuleDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ImportDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ImportSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportNamedSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ImportNamedSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportNamedSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportNamedSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportDefaultSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ImportDefaultSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportDefaultSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportDefaultSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportStarAsSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ImportStarAsSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportStarAsSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportStarAsSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExportDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl NamedExport {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for NamedExport {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for NamedExport {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for NamedExport {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExportSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportNamespaceSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExportNamespaceSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportNamespaceSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportNamespaceSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ModuleExportName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ModuleExportName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ModuleExportName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ModuleExportName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportDefaultSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExportDefaultSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportDefaultSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportDefaultSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportNamedSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExportNamedSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportNamedSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportNamedSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportDefaultDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExportDefaultDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportDefaultDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportDefaultDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl DefaultDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for DefaultDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for DefaultDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for DefaultDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportDefaultExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExportDefaultExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportDefaultExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportDefaultExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportAll {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExportAll {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportAll {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportAll {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BlockStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for BlockStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BlockStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BlockStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Stmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Stmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Stmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Stmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExprStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExprStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExprStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExprStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl EmptyStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for EmptyStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for EmptyStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for EmptyStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl DebuggerStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for DebuggerStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for DebuggerStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for DebuggerStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl WithStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for WithStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for WithStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for WithStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ReturnStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ReturnStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ReturnStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ReturnStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl LabeledStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for LabeledStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for LabeledStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for LabeledStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BreakStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for BreakStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BreakStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BreakStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ContinueStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ContinueStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ContinueStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ContinueStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl IfStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for IfStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for IfStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for IfStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SwitchStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for SwitchStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SwitchStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SwitchStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ThrowStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ThrowStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ThrowStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ThrowStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TryStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TryStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TryStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TryStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl WhileStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for WhileStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for WhileStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for WhileStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl DoWhileStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for DoWhileStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for DoWhileStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for DoWhileStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ForStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ForStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ForStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ForInStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ForInStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ForInStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForInStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ForOfStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ForOfStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ForOfStmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForOfStmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SwitchCase {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for SwitchCase {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SwitchCase {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SwitchCase {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl CatchClause {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for CatchClause {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for CatchClause {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for CatchClause {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ForHead {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ForHead {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ForHead {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForHead {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl VarDeclOrExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for VarDeclOrExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for VarDeclOrExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for VarDeclOrExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Decl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Decl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Decl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Decl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl FnDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for FnDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for FnDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FnDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ClassDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ClassDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ClassDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ClassDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl VarDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for VarDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for VarDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for VarDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl VarDeclarator {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for VarDeclarator {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for VarDeclarator {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for VarDeclarator {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl UsingDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for UsingDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for UsingDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for UsingDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Expr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Expr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Expr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Expr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ThisExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ThisExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ThisExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ThisExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ArrayLit {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ArrayLit {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ArrayLit {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrayLit {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ObjectLit {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ObjectLit {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ObjectLit {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectLit {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl PropOrSpread {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for PropOrSpread {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for PropOrSpread {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PropOrSpread {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SpreadElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for SpreadElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SpreadElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SpreadElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl UnaryExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for UnaryExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for UnaryExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for UnaryExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl UpdateExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for UpdateExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for UpdateExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for UpdateExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BinExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for BinExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BinExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BinExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl FnExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for FnExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for FnExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FnExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ClassExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ClassExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ClassExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ClassExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AssignExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for AssignExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AssignExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl MemberExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for MemberExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for MemberExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MemberExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl MemberProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for MemberProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for MemberProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MemberProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SuperPropExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for SuperPropExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SuperPropExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SuperPropExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SuperProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for SuperProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SuperProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SuperProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl CondExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for CondExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for CondExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for CondExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ImportExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl CallExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for CallExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for CallExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for CallExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl NewExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for NewExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for NewExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for NewExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SeqExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for SeqExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SeqExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SeqExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ArrowExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ArrowExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ArrowExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrowExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl YieldExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for YieldExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for YieldExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for YieldExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl MetaPropExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for MetaPropExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for MetaPropExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MetaPropExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AwaitExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for AwaitExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AwaitExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AwaitExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Tpl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Tpl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Tpl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Tpl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TaggedTpl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TaggedTpl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TaggedTpl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TaggedTpl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TplElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TplElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TplElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TplElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ParenExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ParenExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ParenExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ParenExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Callee {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Callee {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Callee {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Callee {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Super {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Super {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Super {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Super {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExprOrSpread {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExprOrSpread {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExprOrSpread {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExprOrSpread {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BlockStmtOrExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for BlockStmtOrExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BlockStmtOrExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BlockStmtOrExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AssignTarget {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for AssignTarget {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AssignTarget {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignTarget {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AssignTargetPat {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for AssignTargetPat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AssignTargetPat {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignTargetPat {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SimpleAssignTarget {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for SimpleAssignTarget {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SimpleAssignTarget {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SimpleAssignTarget {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl OptChainExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for OptChainExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for OptChainExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for OptChainExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl OptChainBase {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for OptChainBase {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for OptChainBase {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for OptChainBase {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl OptCall {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for OptCall {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for OptCall {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for OptCall {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Invalid {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Invalid {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Invalid {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Invalid {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Function {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Function {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Function {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Function {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Param {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Param {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Param {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Param {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ParamOrTsParamProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ParamOrTsParamProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ParamOrTsParamProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ParamOrTsParamProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Class {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Class {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Class {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Class {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ClassMember {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ClassMember {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ClassMember {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ClassMember {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ClassProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ClassProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ClassProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ClassProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl PrivateProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for PrivateProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for PrivateProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PrivateProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ClassMethod {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ClassMethod {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ClassMethod {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ClassMethod {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl PrivateMethod {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for PrivateMethod {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for PrivateMethod {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PrivateMethod {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Constructor {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Constructor {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Constructor {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Constructor {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Decorator {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Decorator {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Decorator {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Decorator {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl StaticBlock {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for StaticBlock {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for StaticBlock {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for StaticBlock {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Key {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Key {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Key {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Key {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AutoAccessor {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for AutoAccessor {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AutoAccessor {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AutoAccessor {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Prop {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Prop {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Prop {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Prop {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl KeyValueProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for KeyValueProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for KeyValueProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for KeyValueProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AssignProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for AssignProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AssignProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl GetterProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for GetterProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for GetterProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for GetterProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SetterProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for SetterProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SetterProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SetterProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl MethodProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for MethodProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for MethodProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MethodProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl PropName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for PropName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for PropName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PropName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ComputedPropName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ComputedPropName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ComputedPropName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ComputedPropName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Pat {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Pat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Pat {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Pat {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ArrayPat {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ArrayPat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ArrayPat {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrayPat {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ObjectPat {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ObjectPat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ObjectPat {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectPat {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AssignPat {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for AssignPat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AssignPat {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignPat {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl RestPat {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for RestPat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for RestPat {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for RestPat {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ObjectPatProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ObjectPatProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ObjectPatProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectPatProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl KeyValuePatProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for KeyValuePatProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for KeyValuePatProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for KeyValuePatProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AssignPatProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for AssignPatProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AssignPatProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignPatProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Ident {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Ident {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Ident {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Ident {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl IdentName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for IdentName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for IdentName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for IdentName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl PrivateName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for PrivateName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for PrivateName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PrivateName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BindingIdent {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for BindingIdent {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BindingIdent {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BindingIdent {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Lit {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Lit {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Lit {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Lit {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Str {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Str {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Str {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Str {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Bool {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Bool {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Bool {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Bool {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Null {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Null {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Null {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Null {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Number {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Number {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Number {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Number {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BigInt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for BigInt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BigInt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BigInt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Regex {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Regex {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Regex {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Regex {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXObject {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXObject {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXObject {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXObject {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXMemberExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXMemberExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXMemberExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXMemberExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXNamespacedName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXNamespacedName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXNamespacedName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXNamespacedName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXEmptyExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXEmptyExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXEmptyExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXEmptyExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXExprContainer {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXExprContainer {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXExprContainer {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXExprContainer {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXSpreadChild {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXSpreadChild {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXSpreadChild {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXSpreadChild {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXElementName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXElementName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXElementName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXElementName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXOpeningElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXOpeningElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXOpeningElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXOpeningElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXAttrOrSpread {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXAttrOrSpread {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXAttrOrSpread {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXAttrOrSpread {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXClosingElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXClosingElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXClosingElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXClosingElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXAttr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXAttr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXAttr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXAttr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXAttrName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXAttrName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXAttrName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXAttrName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXAttrValue {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXAttrValue {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXAttrValue {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXAttrValue {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXText {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXText {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXText {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXText {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXElementChild {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXElementChild {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXElementChild {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXElementChild {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXFragment {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXFragment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXFragment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXOpeningFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXOpeningFragment {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXOpeningFragment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXOpeningFragment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JSXClosingFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JSXClosingFragment {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JSXClosingFragment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JSXClosingFragment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl<'a> Ast<'a> {
    #[inline]
    pub fn program_data(&self, node: Program) -> ProgramData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::Module => ProgramData::Module(Module(node.0)),
            NodeKind::Script => ProgramData::Script(Script(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn module_item_data(&self, node: ModuleItem) -> ModuleItemData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ImportDecl => ModuleItemData::ModuleDecl(ModuleDecl(node.0)),
            NodeKind::ExportDecl => ModuleItemData::ModuleDecl(ModuleDecl(node.0)),
            NodeKind::NamedExport => ModuleItemData::ModuleDecl(ModuleDecl(node.0)),
            NodeKind::ExportDefaultDecl => ModuleItemData::ModuleDecl(ModuleDecl(node.0)),
            NodeKind::ExportDefaultExpr => ModuleItemData::ModuleDecl(ModuleDecl(node.0)),
            NodeKind::ExportAll => ModuleItemData::ModuleDecl(ModuleDecl(node.0)),
            NodeKind::BlockStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::EmptyStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::DebuggerStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::WithStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::ReturnStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::LabeledStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::BreakStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::ContinueStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::IfStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::SwitchStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::ThrowStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::TryStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::WhileStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::DoWhileStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::ForStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::ForInStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::ForOfStmt => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::ClassDecl => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::FnDecl => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::VarDecl => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::UsingDecl => ModuleItemData::Stmt(Stmt(node.0)),
            NodeKind::ExprStmt => ModuleItemData::Stmt(Stmt(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn module_decl_data(&self, node: ModuleDecl) -> ModuleDeclData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ImportDecl => ModuleDeclData::Import(ImportDecl(node.0)),
            NodeKind::ExportDecl => ModuleDeclData::ExportDecl(ExportDecl(node.0)),
            NodeKind::NamedExport => ModuleDeclData::ExportNamed(NamedExport(node.0)),
            NodeKind::ExportDefaultDecl => {
                ModuleDeclData::ExportDefaultDecl(ExportDefaultDecl(node.0))
            }
            NodeKind::ExportDefaultExpr => {
                ModuleDeclData::ExportDefaultExpr(ExportDefaultExpr(node.0))
            }
            NodeKind::ExportAll => ModuleDeclData::ExportAll(ExportAll(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn import_specifier_data(&self, node: ImportSpecifier) -> ImportSpecifierData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ImportNamedSpecifier => {
                ImportSpecifierData::Named(ImportNamedSpecifier(node.0))
            }
            NodeKind::ImportDefaultSpecifier => {
                ImportSpecifierData::Default(ImportDefaultSpecifier(node.0))
            }
            NodeKind::ImportStarAsSpecifier => {
                ImportSpecifierData::Namespace(ImportStarAsSpecifier(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn export_specifier_data(&self, node: ExportSpecifier) -> ExportSpecifierData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ExportNamespaceSpecifier => {
                ExportSpecifierData::Namespace(ExportNamespaceSpecifier(node.0))
            }
            NodeKind::ExportDefaultSpecifier => {
                ExportSpecifierData::Default(ExportDefaultSpecifier(node.0))
            }
            NodeKind::ExportNamedSpecifier => {
                ExportSpecifierData::Named(ExportNamedSpecifier(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn module_export_name_data(&self, node: ModuleExportName) -> ModuleExportNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::Ident => ModuleExportNameData::Ident(Ident(node.0)),
            NodeKind::Str => ModuleExportNameData::Str(Str(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn default_decl_data(&self, node: DefaultDecl) -> DefaultDeclData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ClassExpr => DefaultDeclData::Class(ClassExpr(node.0)),
            NodeKind::FnExpr => DefaultDeclData::Fn(FnExpr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn stmt_data(&self, node: Stmt) -> StmtData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::BlockStmt => StmtData::Block(BlockStmt(node.0)),
            NodeKind::EmptyStmt => StmtData::Empty(EmptyStmt(node.0)),
            NodeKind::DebuggerStmt => StmtData::Debugger(DebuggerStmt(node.0)),
            NodeKind::WithStmt => StmtData::With(WithStmt(node.0)),
            NodeKind::ReturnStmt => StmtData::Return(ReturnStmt(node.0)),
            NodeKind::LabeledStmt => StmtData::Labeled(LabeledStmt(node.0)),
            NodeKind::BreakStmt => StmtData::Break(BreakStmt(node.0)),
            NodeKind::ContinueStmt => StmtData::Continue(ContinueStmt(node.0)),
            NodeKind::IfStmt => StmtData::If(IfStmt(node.0)),
            NodeKind::SwitchStmt => StmtData::Switch(SwitchStmt(node.0)),
            NodeKind::ThrowStmt => StmtData::Throw(ThrowStmt(node.0)),
            NodeKind::TryStmt => StmtData::Try(TryStmt(node.0)),
            NodeKind::WhileStmt => StmtData::While(WhileStmt(node.0)),
            NodeKind::DoWhileStmt => StmtData::DoWhile(DoWhileStmt(node.0)),
            NodeKind::ForStmt => StmtData::For(ForStmt(node.0)),
            NodeKind::ForInStmt => StmtData::ForIn(ForInStmt(node.0)),
            NodeKind::ForOfStmt => StmtData::ForOf(ForOfStmt(node.0)),
            NodeKind::ClassDecl => StmtData::Decl(Decl(node.0)),
            NodeKind::FnDecl => StmtData::Decl(Decl(node.0)),
            NodeKind::VarDecl => StmtData::Decl(Decl(node.0)),
            NodeKind::UsingDecl => StmtData::Decl(Decl(node.0)),
            NodeKind::ExprStmt => StmtData::Expr(ExprStmt(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn for_head_data(&self, node: ForHead) -> ForHeadData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::VarDecl => ForHeadData::VarDecl(VarDecl(node.0)),
            NodeKind::UsingDecl => ForHeadData::UsingDecl(UsingDecl(node.0)),
            NodeKind::BindingIdent => ForHeadData::Pat(Pat(node.0)),
            NodeKind::ArrayPat => ForHeadData::Pat(Pat(node.0)),
            NodeKind::RestPat => ForHeadData::Pat(Pat(node.0)),
            NodeKind::ObjectPat => ForHeadData::Pat(Pat(node.0)),
            NodeKind::AssignPat => ForHeadData::Pat(Pat(node.0)),
            NodeKind::Invalid => ForHeadData::Pat(Pat(node.0)),
            NodeKind::ThisExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::ArrayLit => ForHeadData::Pat(Pat(node.0)),
            NodeKind::ObjectLit => ForHeadData::Pat(Pat(node.0)),
            NodeKind::FnExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::UnaryExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::UpdateExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::BinExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::AssignExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::MemberExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::SuperPropExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::CondExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::ImportExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::CallExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::NewExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::SeqExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::Ident => ForHeadData::Pat(Pat(node.0)),
            NodeKind::Str => ForHeadData::Pat(Pat(node.0)),
            NodeKind::Bool => ForHeadData::Pat(Pat(node.0)),
            NodeKind::Null => ForHeadData::Pat(Pat(node.0)),
            NodeKind::Number => ForHeadData::Pat(Pat(node.0)),
            NodeKind::BigInt => ForHeadData::Pat(Pat(node.0)),
            NodeKind::Regex => ForHeadData::Pat(Pat(node.0)),
            NodeKind::Tpl => ForHeadData::Pat(Pat(node.0)),
            NodeKind::TaggedTpl => ForHeadData::Pat(Pat(node.0)),
            NodeKind::ArrowExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::ClassExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::YieldExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::MetaPropExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::AwaitExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::ParenExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::JSXMemberExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::JSXNamespacedName => ForHeadData::Pat(Pat(node.0)),
            NodeKind::JSXEmptyExpr => ForHeadData::Pat(Pat(node.0)),
            NodeKind::JSXElement => ForHeadData::Pat(Pat(node.0)),
            NodeKind::JSXFragment => ForHeadData::Pat(Pat(node.0)),
            NodeKind::PrivateName => ForHeadData::Pat(Pat(node.0)),
            NodeKind::OptChainExpr => ForHeadData::Pat(Pat(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn var_decl_or_expr_data(&self, node: VarDeclOrExpr) -> VarDeclOrExprData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::VarDecl => VarDeclOrExprData::VarDecl(VarDecl(node.0)),
            NodeKind::ThisExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::ArrayLit => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::ObjectLit => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::FnExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::UnaryExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::UpdateExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::BinExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::AssignExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::MemberExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::SuperPropExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::CondExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::ImportExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::CallExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::NewExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::SeqExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::Ident => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::Str => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::Bool => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::Null => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::Number => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::BigInt => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::Regex => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::Tpl => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::TaggedTpl => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::ArrowExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::ClassExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::YieldExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::MetaPropExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::AwaitExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::ParenExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXMemberExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXNamespacedName => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXEmptyExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXElement => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXFragment => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::PrivateName => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::OptChainExpr => VarDeclOrExprData::Expr(Expr(node.0)),
            NodeKind::Invalid => VarDeclOrExprData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn decl_data(&self, node: Decl) -> DeclData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ClassDecl => DeclData::Class(ClassDecl(node.0)),
            NodeKind::FnDecl => DeclData::Fn(FnDecl(node.0)),
            NodeKind::VarDecl => DeclData::Var(VarDecl(node.0)),
            NodeKind::UsingDecl => DeclData::Using(UsingDecl(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn expr_data(&self, node: Expr) -> ExprData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ThisExpr => ExprData::This(ThisExpr(node.0)),
            NodeKind::ArrayLit => ExprData::Array(ArrayLit(node.0)),
            NodeKind::ObjectLit => ExprData::Object(ObjectLit(node.0)),
            NodeKind::FnExpr => ExprData::Fn(FnExpr(node.0)),
            NodeKind::UnaryExpr => ExprData::Unary(UnaryExpr(node.0)),
            NodeKind::UpdateExpr => ExprData::Update(UpdateExpr(node.0)),
            NodeKind::BinExpr => ExprData::Bin(BinExpr(node.0)),
            NodeKind::AssignExpr => ExprData::Assign(AssignExpr(node.0)),
            NodeKind::MemberExpr => ExprData::Member(MemberExpr(node.0)),
            NodeKind::SuperPropExpr => ExprData::SuperProp(SuperPropExpr(node.0)),
            NodeKind::CondExpr => ExprData::Cond(CondExpr(node.0)),
            NodeKind::ImportExpr => ExprData::Import(ImportExpr(node.0)),
            NodeKind::CallExpr => ExprData::Call(CallExpr(node.0)),
            NodeKind::NewExpr => ExprData::New(NewExpr(node.0)),
            NodeKind::SeqExpr => ExprData::Seq(SeqExpr(node.0)),
            NodeKind::Ident => ExprData::Ident(Ident(node.0)),
            NodeKind::Str => ExprData::Lit(Lit(node.0)),
            NodeKind::Bool => ExprData::Lit(Lit(node.0)),
            NodeKind::Null => ExprData::Lit(Lit(node.0)),
            NodeKind::Number => ExprData::Lit(Lit(node.0)),
            NodeKind::BigInt => ExprData::Lit(Lit(node.0)),
            NodeKind::Regex => ExprData::Lit(Lit(node.0)),
            NodeKind::Tpl => ExprData::Tpl(Tpl(node.0)),
            NodeKind::TaggedTpl => ExprData::TaggedTpl(TaggedTpl(node.0)),
            NodeKind::ArrowExpr => ExprData::Arrow(ArrowExpr(node.0)),
            NodeKind::ClassExpr => ExprData::Class(ClassExpr(node.0)),
            NodeKind::YieldExpr => ExprData::Yield(YieldExpr(node.0)),
            NodeKind::MetaPropExpr => ExprData::MetaProp(MetaPropExpr(node.0)),
            NodeKind::AwaitExpr => ExprData::Await(AwaitExpr(node.0)),
            NodeKind::ParenExpr => ExprData::Paren(ParenExpr(node.0)),
            NodeKind::JSXMemberExpr => ExprData::JSXMember(JSXMemberExpr(node.0)),
            NodeKind::JSXNamespacedName => ExprData::JSXNamespacedName(JSXNamespacedName(node.0)),
            NodeKind::JSXEmptyExpr => ExprData::JSXEmpty(JSXEmptyExpr(node.0)),
            NodeKind::JSXElement => ExprData::JSXElement(JSXElement(node.0)),
            NodeKind::JSXFragment => ExprData::JSXFragment(JSXFragment(node.0)),
            NodeKind::PrivateName => ExprData::PrivateName(PrivateName(node.0)),
            NodeKind::OptChainExpr => ExprData::OptChain(OptChainExpr(node.0)),
            NodeKind::Invalid => ExprData::Invalid(Invalid(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn prop_or_spread_data(&self, node: PropOrSpread) -> PropOrSpreadData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::SpreadElement => PropOrSpreadData::Spread(SpreadElement(node.0)),
            NodeKind::Ident => PropOrSpreadData::Prop(Prop(node.0)),
            NodeKind::KeyValueProp => PropOrSpreadData::Prop(Prop(node.0)),
            NodeKind::AssignProp => PropOrSpreadData::Prop(Prop(node.0)),
            NodeKind::GetterProp => PropOrSpreadData::Prop(Prop(node.0)),
            NodeKind::SetterProp => PropOrSpreadData::Prop(Prop(node.0)),
            NodeKind::MethodProp => PropOrSpreadData::Prop(Prop(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn member_prop_data(&self, node: MemberProp) -> MemberPropData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentName => MemberPropData::Ident(IdentName(node.0)),
            NodeKind::PrivateName => MemberPropData::PrivateName(PrivateName(node.0)),
            NodeKind::ComputedPropName => MemberPropData::Computed(ComputedPropName(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn super_prop_data(&self, node: SuperProp) -> SuperPropData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentName => SuperPropData::Ident(IdentName(node.0)),
            NodeKind::ComputedPropName => SuperPropData::Computed(ComputedPropName(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn callee_data(&self, node: Callee) -> CalleeData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::Super => CalleeData::Super(Super(node.0)),
            NodeKind::ThisExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::ArrayLit => CalleeData::Expr(Expr(node.0)),
            NodeKind::ObjectLit => CalleeData::Expr(Expr(node.0)),
            NodeKind::FnExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::UnaryExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::UpdateExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::BinExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::AssignExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::MemberExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::SuperPropExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::CondExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::ImportExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::CallExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::NewExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::SeqExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::Ident => CalleeData::Expr(Expr(node.0)),
            NodeKind::Str => CalleeData::Expr(Expr(node.0)),
            NodeKind::Bool => CalleeData::Expr(Expr(node.0)),
            NodeKind::Null => CalleeData::Expr(Expr(node.0)),
            NodeKind::Number => CalleeData::Expr(Expr(node.0)),
            NodeKind::BigInt => CalleeData::Expr(Expr(node.0)),
            NodeKind::Regex => CalleeData::Expr(Expr(node.0)),
            NodeKind::Tpl => CalleeData::Expr(Expr(node.0)),
            NodeKind::TaggedTpl => CalleeData::Expr(Expr(node.0)),
            NodeKind::ArrowExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::ClassExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::YieldExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::MetaPropExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::AwaitExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::ParenExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::JSXMemberExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::JSXNamespacedName => CalleeData::Expr(Expr(node.0)),
            NodeKind::JSXEmptyExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::JSXElement => CalleeData::Expr(Expr(node.0)),
            NodeKind::JSXFragment => CalleeData::Expr(Expr(node.0)),
            NodeKind::PrivateName => CalleeData::Expr(Expr(node.0)),
            NodeKind::OptChainExpr => CalleeData::Expr(Expr(node.0)),
            NodeKind::Invalid => CalleeData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn block_stmt_or_expr_data(&self, node: BlockStmtOrExpr) -> BlockStmtOrExprData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::BlockStmt => BlockStmtOrExprData::BlockStmt(BlockStmt(node.0)),
            NodeKind::ThisExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::ArrayLit => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::ObjectLit => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::FnExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::UnaryExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::UpdateExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::BinExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::AssignExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::MemberExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::SuperPropExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::CondExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::ImportExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::CallExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::NewExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::SeqExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::Ident => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::Str => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::Bool => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::Null => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::Number => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::BigInt => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::Regex => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::Tpl => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::TaggedTpl => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::ArrowExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::ClassExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::YieldExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::MetaPropExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::AwaitExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::ParenExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXMemberExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXNamespacedName => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXEmptyExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXElement => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::JSXFragment => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::PrivateName => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::OptChainExpr => BlockStmtOrExprData::Expr(Expr(node.0)),
            NodeKind::Invalid => BlockStmtOrExprData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn assign_target_data(&self, node: AssignTarget) -> AssignTargetData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::BindingIdent => AssignTargetData::Simple(SimpleAssignTarget(node.0)),
            NodeKind::MemberExpr => AssignTargetData::Simple(SimpleAssignTarget(node.0)),
            NodeKind::SuperPropExpr => AssignTargetData::Simple(SimpleAssignTarget(node.0)),
            NodeKind::ParenExpr => AssignTargetData::Simple(SimpleAssignTarget(node.0)),
            NodeKind::OptChainExpr => AssignTargetData::Simple(SimpleAssignTarget(node.0)),
            NodeKind::Invalid => AssignTargetData::Simple(SimpleAssignTarget(node.0)),
            NodeKind::ArrayPat => AssignTargetData::Pat(AssignTargetPat(node.0)),
            NodeKind::ObjectPat => AssignTargetData::Pat(AssignTargetPat(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn assign_target_pat_data(&self, node: AssignTargetPat) -> AssignTargetPatData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ArrayPat => AssignTargetPatData::Array(ArrayPat(node.0)),
            NodeKind::ObjectPat => AssignTargetPatData::Object(ObjectPat(node.0)),
            NodeKind::Invalid => AssignTargetPatData::Invalid(Invalid(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn simple_assign_target_data(&self, node: SimpleAssignTarget) -> SimpleAssignTargetData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::BindingIdent => SimpleAssignTargetData::Ident(BindingIdent(node.0)),
            NodeKind::MemberExpr => SimpleAssignTargetData::Member(MemberExpr(node.0)),
            NodeKind::SuperPropExpr => SimpleAssignTargetData::SuperProp(SuperPropExpr(node.0)),
            NodeKind::ParenExpr => SimpleAssignTargetData::Paren(ParenExpr(node.0)),
            NodeKind::OptChainExpr => SimpleAssignTargetData::OptChain(OptChainExpr(node.0)),
            NodeKind::Invalid => SimpleAssignTargetData::Invalid(Invalid(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn opt_chain_base_data(&self, node: OptChainBase) -> OptChainBaseData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::MemberExpr => OptChainBaseData::Member(MemberExpr(node.0)),
            NodeKind::OptCall => OptChainBaseData::Call(OptCall(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn param_or_ts_param_prop_data(&self, node: ParamOrTsParamProp) -> ParamOrTsParamPropData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::Param => ParamOrTsParamPropData::Param(Param(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn class_member_data(&self, node: ClassMember) -> ClassMemberData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::Constructor => ClassMemberData::Constructor(Constructor(node.0)),
            NodeKind::ClassMethod => ClassMemberData::Method(ClassMethod(node.0)),
            NodeKind::PrivateMethod => ClassMemberData::PrivateMethod(PrivateMethod(node.0)),
            NodeKind::ClassProp => ClassMemberData::ClassProp(ClassProp(node.0)),
            NodeKind::PrivateProp => ClassMemberData::PrivateProp(PrivateProp(node.0)),
            NodeKind::EmptyStmt => ClassMemberData::Empty(EmptyStmt(node.0)),
            NodeKind::StaticBlock => ClassMemberData::StaticBlock(StaticBlock(node.0)),
            NodeKind::AutoAccessor => ClassMemberData::AutoAccessor(AutoAccessor(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn key_data(&self, node: Key) -> KeyData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::PrivateName => KeyData::Private(PrivateName(node.0)),
            NodeKind::IdentName => KeyData::Public(PropName(node.0)),
            NodeKind::Str => KeyData::Public(PropName(node.0)),
            NodeKind::Number => KeyData::Public(PropName(node.0)),
            NodeKind::ComputedPropName => KeyData::Public(PropName(node.0)),
            NodeKind::BigInt => KeyData::Public(PropName(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn prop_data(&self, node: Prop) -> PropData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::Ident => PropData::Shorthand(Ident(node.0)),
            NodeKind::KeyValueProp => PropData::KeyValue(KeyValueProp(node.0)),
            NodeKind::AssignProp => PropData::Assign(AssignProp(node.0)),
            NodeKind::GetterProp => PropData::Getter(GetterProp(node.0)),
            NodeKind::SetterProp => PropData::Setter(SetterProp(node.0)),
            NodeKind::MethodProp => PropData::Method(MethodProp(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn prop_name_data(&self, node: PropName) -> PropNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentName => PropNameData::Ident(IdentName(node.0)),
            NodeKind::Str => PropNameData::Str(Str(node.0)),
            NodeKind::Number => PropNameData::Num(Number(node.0)),
            NodeKind::ComputedPropName => PropNameData::Computed(ComputedPropName(node.0)),
            NodeKind::BigInt => PropNameData::BigInt(BigInt(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn pat_data(&self, node: Pat) -> PatData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::BindingIdent => PatData::Ident(BindingIdent(node.0)),
            NodeKind::ArrayPat => PatData::Array(ArrayPat(node.0)),
            NodeKind::RestPat => PatData::Rest(RestPat(node.0)),
            NodeKind::ObjectPat => PatData::Object(ObjectPat(node.0)),
            NodeKind::AssignPat => PatData::Assign(AssignPat(node.0)),
            NodeKind::Invalid => PatData::Invalid(Invalid(node.0)),
            NodeKind::ThisExpr => PatData::Expr(Expr(node.0)),
            NodeKind::ArrayLit => PatData::Expr(Expr(node.0)),
            NodeKind::ObjectLit => PatData::Expr(Expr(node.0)),
            NodeKind::FnExpr => PatData::Expr(Expr(node.0)),
            NodeKind::UnaryExpr => PatData::Expr(Expr(node.0)),
            NodeKind::UpdateExpr => PatData::Expr(Expr(node.0)),
            NodeKind::BinExpr => PatData::Expr(Expr(node.0)),
            NodeKind::AssignExpr => PatData::Expr(Expr(node.0)),
            NodeKind::MemberExpr => PatData::Expr(Expr(node.0)),
            NodeKind::SuperPropExpr => PatData::Expr(Expr(node.0)),
            NodeKind::CondExpr => PatData::Expr(Expr(node.0)),
            NodeKind::ImportExpr => PatData::Expr(Expr(node.0)),
            NodeKind::CallExpr => PatData::Expr(Expr(node.0)),
            NodeKind::NewExpr => PatData::Expr(Expr(node.0)),
            NodeKind::SeqExpr => PatData::Expr(Expr(node.0)),
            NodeKind::Ident => PatData::Expr(Expr(node.0)),
            NodeKind::Str => PatData::Expr(Expr(node.0)),
            NodeKind::Bool => PatData::Expr(Expr(node.0)),
            NodeKind::Null => PatData::Expr(Expr(node.0)),
            NodeKind::Number => PatData::Expr(Expr(node.0)),
            NodeKind::BigInt => PatData::Expr(Expr(node.0)),
            NodeKind::Regex => PatData::Expr(Expr(node.0)),
            NodeKind::Tpl => PatData::Expr(Expr(node.0)),
            NodeKind::TaggedTpl => PatData::Expr(Expr(node.0)),
            NodeKind::ArrowExpr => PatData::Expr(Expr(node.0)),
            NodeKind::ClassExpr => PatData::Expr(Expr(node.0)),
            NodeKind::YieldExpr => PatData::Expr(Expr(node.0)),
            NodeKind::MetaPropExpr => PatData::Expr(Expr(node.0)),
            NodeKind::AwaitExpr => PatData::Expr(Expr(node.0)),
            NodeKind::ParenExpr => PatData::Expr(Expr(node.0)),
            NodeKind::JSXMemberExpr => PatData::Expr(Expr(node.0)),
            NodeKind::JSXNamespacedName => PatData::Expr(Expr(node.0)),
            NodeKind::JSXEmptyExpr => PatData::Expr(Expr(node.0)),
            NodeKind::JSXElement => PatData::Expr(Expr(node.0)),
            NodeKind::JSXFragment => PatData::Expr(Expr(node.0)),
            NodeKind::PrivateName => PatData::Expr(Expr(node.0)),
            NodeKind::OptChainExpr => PatData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn object_pat_prop_data(&self, node: ObjectPatProp) -> ObjectPatPropData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::KeyValuePatProp => ObjectPatPropData::KeyValue(KeyValuePatProp(node.0)),
            NodeKind::AssignPatProp => ObjectPatPropData::Assign(AssignPatProp(node.0)),
            NodeKind::RestPat => ObjectPatPropData::Rest(RestPat(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn lit_data(&self, node: Lit) -> LitData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::Str => LitData::Str(Str(node.0)),
            NodeKind::Bool => LitData::Bool(Bool(node.0)),
            NodeKind::Null => LitData::Null(Null(node.0)),
            NodeKind::Number => LitData::Num(Number(node.0)),
            NodeKind::BigInt => LitData::BigInt(BigInt(node.0)),
            NodeKind::Regex => LitData::Regex(Regex(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_object_data(&self, node: JSXObject) -> JSXObjectData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::JSXMemberExpr => JSXObjectData::JSXMemberExpr(JSXMemberExpr(node.0)),
            NodeKind::Ident => JSXObjectData::Ident(Ident(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_expr_data(&self, node: JSXExpr) -> JSXExprData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::JSXEmptyExpr => JSXExprData::JSXEmptyExpr(JSXEmptyExpr(node.0)),
            NodeKind::ThisExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::ArrayLit => JSXExprData::Expr(Expr(node.0)),
            NodeKind::ObjectLit => JSXExprData::Expr(Expr(node.0)),
            NodeKind::FnExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::UnaryExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::UpdateExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::BinExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::AssignExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::MemberExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::SuperPropExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::CondExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::ImportExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::CallExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::NewExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::SeqExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::Ident => JSXExprData::Expr(Expr(node.0)),
            NodeKind::Str => JSXExprData::Expr(Expr(node.0)),
            NodeKind::Bool => JSXExprData::Expr(Expr(node.0)),
            NodeKind::Null => JSXExprData::Expr(Expr(node.0)),
            NodeKind::Number => JSXExprData::Expr(Expr(node.0)),
            NodeKind::BigInt => JSXExprData::Expr(Expr(node.0)),
            NodeKind::Regex => JSXExprData::Expr(Expr(node.0)),
            NodeKind::Tpl => JSXExprData::Expr(Expr(node.0)),
            NodeKind::TaggedTpl => JSXExprData::Expr(Expr(node.0)),
            NodeKind::ArrowExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::ClassExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::YieldExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::MetaPropExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::AwaitExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::ParenExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::JSXMemberExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::JSXNamespacedName => JSXExprData::Expr(Expr(node.0)),
            NodeKind::JSXElement => JSXExprData::Expr(Expr(node.0)),
            NodeKind::JSXFragment => JSXExprData::Expr(Expr(node.0)),
            NodeKind::PrivateName => JSXExprData::Expr(Expr(node.0)),
            NodeKind::OptChainExpr => JSXExprData::Expr(Expr(node.0)),
            NodeKind::Invalid => JSXExprData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_element_name_data(&self, node: JSXElementName) -> JSXElementNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::Ident => JSXElementNameData::Ident(Ident(node.0)),
            NodeKind::JSXMemberExpr => JSXElementNameData::JSXMemberExpr(JSXMemberExpr(node.0)),
            NodeKind::JSXNamespacedName => {
                JSXElementNameData::JSXNamespacedName(JSXNamespacedName(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_attr_or_spread_data(&self, node: JSXAttrOrSpread) -> JSXAttrOrSpreadData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::JSXAttr => JSXAttrOrSpreadData::JSXAttr(JSXAttr(node.0)),
            NodeKind::SpreadElement => JSXAttrOrSpreadData::SpreadElement(SpreadElement(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_attr_name_data(&self, node: JSXAttrName) -> JSXAttrNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentName => JSXAttrNameData::Ident(IdentName(node.0)),
            NodeKind::JSXNamespacedName => {
                JSXAttrNameData::JSXNamespacedName(JSXNamespacedName(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_attr_value_data(&self, node: JSXAttrValue) -> JSXAttrValueData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::Str => JSXAttrValueData::Str(Str(node.0)),
            NodeKind::JSXExprContainer => {
                JSXAttrValueData::JSXExprContainer(JSXExprContainer(node.0))
            }
            NodeKind::JSXElement => JSXAttrValueData::JSXElement(JSXElement(node.0)),
            NodeKind::JSXFragment => JSXAttrValueData::JSXFragment(JSXFragment(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_element_child_data(&self, node: JSXElementChild) -> JSXElementChildData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::JSXText => JSXElementChildData::JSXText(JSXText(node.0)),
            NodeKind::JSXExprContainer => {
                JSXElementChildData::JSXExprContainer(JSXExprContainer(node.0))
            }
            NodeKind::JSXSpreadChild => JSXElementChildData::JSXSpreadChild(JSXSpreadChild(node.0)),
            NodeKind::JSXElement => JSXElementChildData::JSXElement(JSXElement(node.0)),
            NodeKind::JSXFragment => JSXElementChildData::JSXFragment(JSXFragment(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
