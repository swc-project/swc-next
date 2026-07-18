#![allow(unused)]
use crate::{Ast, GetSpan, SetSpan};
use crate::{ast::*, node_id::*};
impl Program {
    #[inline]
    pub fn node_id(self) -> NodeId {
        match self {
            Self::Module(it) => it.node_id(),
            Self::Script(it) => it.node_id(),
        }
    }
}
impl GetSpan for Program {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for Program {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl Module {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::ModuleDeclImport(it) => it.node_id(),
            Self::ModuleDeclExportDecl(it) => it.node_id(),
            Self::ModuleDeclExportNamed(it) => it.node_id(),
            Self::ModuleDeclExportDefaultDecl(it) => it.node_id(),
            Self::ModuleDeclExportDefaultExpr(it) => it.node_id(),
            Self::ModuleDeclExportAll(it) => it.node_id(),
            Self::StmtBlock(it) => it.node_id(),
            Self::StmtEmpty(it) => it.node_id(),
            Self::StmtDebugger(it) => it.node_id(),
            Self::StmtWith(it) => it.node_id(),
            Self::StmtReturn(it) => it.node_id(),
            Self::StmtLabeled(it) => it.node_id(),
            Self::StmtBreak(it) => it.node_id(),
            Self::StmtContinue(it) => it.node_id(),
            Self::StmtIf(it) => it.node_id(),
            Self::StmtSwitch(it) => it.node_id(),
            Self::StmtThrow(it) => it.node_id(),
            Self::StmtTry(it) => it.node_id(),
            Self::StmtWhile(it) => it.node_id(),
            Self::StmtDoWhile(it) => it.node_id(),
            Self::StmtFor(it) => it.node_id(),
            Self::StmtForIn(it) => it.node_id(),
            Self::StmtForOf(it) => it.node_id(),
            Self::StmtDeclClass(it) => it.node_id(),
            Self::StmtDeclFn(it) => it.node_id(),
            Self::StmtDeclVar(it) => it.node_id(),
            Self::StmtDeclUsing(it) => it.node_id(),
            Self::StmtExpr(it) => it.node_id(),
        }
    }
}
impl GetSpan for ModuleItem {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for ModuleItem {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ModuleDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        match self {
            Self::Import(it) => it.node_id(),
            Self::ExportDecl(it) => it.node_id(),
            Self::ExportNamed(it) => it.node_id(),
            Self::ExportDefaultDecl(it) => it.node_id(),
            Self::ExportDefaultExpr(it) => it.node_id(),
            Self::ExportAll(it) => it.node_id(),
        }
    }
}
impl GetSpan for ModuleDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for ModuleDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ImportDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Named(it) => it.node_id(),
            Self::Default(it) => it.node_id(),
            Self::Namespace(it) => it.node_id(),
        }
    }
}
impl GetSpan for ImportSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for ImportSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ImportNamedSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Namespace(it) => it.node_id(),
            Self::Default(it) => it.node_id(),
            Self::Named(it) => it.node_id(),
        }
    }
}
impl GetSpan for ExportSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for ExportSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ExportNamespaceSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
        }
    }
}
impl GetSpan for ModuleExportName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for ModuleExportName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ExportDefaultSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Class(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
        }
    }
}
impl GetSpan for DefaultDecl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for DefaultDecl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ExportDefaultExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Class(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
            Self::Var(it) => it.node_id(),
            Self::Using(it) => it.node_id(),
            Self::Block(it) => it.node_id(),
            Self::Empty(it) => it.node_id(),
            Self::Debugger(it) => it.node_id(),
            Self::With(it) => it.node_id(),
            Self::Return(it) => it.node_id(),
            Self::Labeled(it) => it.node_id(),
            Self::Break(it) => it.node_id(),
            Self::Continue(it) => it.node_id(),
            Self::If(it) => it.node_id(),
            Self::Switch(it) => it.node_id(),
            Self::Throw(it) => it.node_id(),
            Self::Try(it) => it.node_id(),
            Self::While(it) => it.node_id(),
            Self::DoWhile(it) => it.node_id(),
            Self::For(it) => it.node_id(),
            Self::ForIn(it) => it.node_id(),
            Self::ForOf(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
}
impl GetSpan for Stmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for Stmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ExprStmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::ExprThis(it) => it.node_id(),
            Self::ExprArray(it) => it.node_id(),
            Self::ExprObject(it) => it.node_id(),
            Self::ExprFn(it) => it.node_id(),
            Self::ExprUnary(it) => it.node_id(),
            Self::ExprUpdate(it) => it.node_id(),
            Self::ExprBin(it) => it.node_id(),
            Self::ExprAssign(it) => it.node_id(),
            Self::ExprMember(it) => it.node_id(),
            Self::ExprSuperProp(it) => it.node_id(),
            Self::ExprCond(it) => it.node_id(),
            Self::ExprImport(it) => it.node_id(),
            Self::ExprCall(it) => it.node_id(),
            Self::ExprNew(it) => it.node_id(),
            Self::ExprSeq(it) => it.node_id(),
            Self::ExprIdent(it) => it.node_id(),
            Self::ExprStr(it) => it.node_id(),
            Self::ExprBool(it) => it.node_id(),
            Self::ExprNull(it) => it.node_id(),
            Self::ExprNum(it) => it.node_id(),
            Self::ExprBigInt(it) => it.node_id(),
            Self::ExprRegex(it) => it.node_id(),
            Self::ExprTpl(it) => it.node_id(),
            Self::ExprTaggedTpl(it) => it.node_id(),
            Self::ExprArrow(it) => it.node_id(),
            Self::ExprClass(it) => it.node_id(),
            Self::ExprYield(it) => it.node_id(),
            Self::ExprMetaProp(it) => it.node_id(),
            Self::ExprAwait(it) => it.node_id(),
            Self::ExprParen(it) => it.node_id(),
            Self::ExprJSXMember(it) => it.node_id(),
            Self::ExprJSXNamespacedName(it) => it.node_id(),
            Self::ExprJSXEmpty(it) => it.node_id(),
            Self::ExprJSXElement(it) => it.node_id(),
            Self::ExprJSXFragment(it) => it.node_id(),
            Self::ExprPrivateName(it) => it.node_id(),
            Self::ExprOptChain(it) => it.node_id(),
            Self::ExprInvalid(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
            Self::Array(it) => it.node_id(),
            Self::Rest(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
            Self::VarDecl(it) => it.node_id(),
            Self::UsingDecl(it) => it.node_id(),
        }
    }
}
impl GetSpan for ForHead {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for ForHead {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl VarDeclOrExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        match self {
            Self::This(it) => it.node_id(),
            Self::Array(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
            Self::Unary(it) => it.node_id(),
            Self::Update(it) => it.node_id(),
            Self::Bin(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Member(it) => it.node_id(),
            Self::SuperProp(it) => it.node_id(),
            Self::Cond(it) => it.node_id(),
            Self::Import(it) => it.node_id(),
            Self::Call(it) => it.node_id(),
            Self::New(it) => it.node_id(),
            Self::Seq(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
            Self::Bool(it) => it.node_id(),
            Self::Null(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
            Self::Regex(it) => it.node_id(),
            Self::Tpl(it) => it.node_id(),
            Self::TaggedTpl(it) => it.node_id(),
            Self::Arrow(it) => it.node_id(),
            Self::Class(it) => it.node_id(),
            Self::Yield(it) => it.node_id(),
            Self::MetaProp(it) => it.node_id(),
            Self::Await(it) => it.node_id(),
            Self::Paren(it) => it.node_id(),
            Self::JSXMember(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
            Self::JSXEmpty(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::PrivateName(it) => it.node_id(),
            Self::OptChain(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
            Self::VarDecl(it) => it.node_id(),
        }
    }
}
impl GetSpan for VarDeclOrExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for VarDeclOrExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl Decl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        match self {
            Self::Class(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
            Self::Var(it) => it.node_id(),
            Self::Using(it) => it.node_id(),
        }
    }
}
impl GetSpan for Decl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for Decl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl FnDecl {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::This(it) => it.node_id(),
            Self::Array(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
            Self::Unary(it) => it.node_id(),
            Self::Update(it) => it.node_id(),
            Self::Bin(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Member(it) => it.node_id(),
            Self::SuperProp(it) => it.node_id(),
            Self::Cond(it) => it.node_id(),
            Self::Import(it) => it.node_id(),
            Self::Call(it) => it.node_id(),
            Self::New(it) => it.node_id(),
            Self::Seq(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
            Self::Bool(it) => it.node_id(),
            Self::Null(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
            Self::Regex(it) => it.node_id(),
            Self::Tpl(it) => it.node_id(),
            Self::TaggedTpl(it) => it.node_id(),
            Self::Arrow(it) => it.node_id(),
            Self::Class(it) => it.node_id(),
            Self::Yield(it) => it.node_id(),
            Self::MetaProp(it) => it.node_id(),
            Self::Await(it) => it.node_id(),
            Self::Paren(it) => it.node_id(),
            Self::JSXMember(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
            Self::JSXEmpty(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::PrivateName(it) => it.node_id(),
            Self::OptChain(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
        }
    }
}
impl GetSpan for Expr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for Expr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ThisExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Shorthand(it) => it.node_id(),
            Self::KeyValue(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Getter(it) => it.node_id(),
            Self::Setter(it) => it.node_id(),
            Self::Method(it) => it.node_id(),
            Self::Spread(it) => it.node_id(),
        }
    }
}
impl GetSpan for PropOrSpread {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for PropOrSpread {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl SpreadElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Ident(it) => it.node_id(),
            Self::PrivateName(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
        }
    }
}
impl GetSpan for MemberProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for MemberProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl SuperPropExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
        }
    }
}
impl GetSpan for SuperProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for SuperProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl CondExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::This(it) => it.node_id(),
            Self::Array(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
            Self::Unary(it) => it.node_id(),
            Self::Update(it) => it.node_id(),
            Self::Bin(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Member(it) => it.node_id(),
            Self::SuperProp(it) => it.node_id(),
            Self::Cond(it) => it.node_id(),
            Self::Import(it) => it.node_id(),
            Self::Call(it) => it.node_id(),
            Self::New(it) => it.node_id(),
            Self::Seq(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
            Self::Bool(it) => it.node_id(),
            Self::Null(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
            Self::Regex(it) => it.node_id(),
            Self::Tpl(it) => it.node_id(),
            Self::TaggedTpl(it) => it.node_id(),
            Self::Arrow(it) => it.node_id(),
            Self::Class(it) => it.node_id(),
            Self::Yield(it) => it.node_id(),
            Self::MetaProp(it) => it.node_id(),
            Self::Await(it) => it.node_id(),
            Self::Paren(it) => it.node_id(),
            Self::JSXMember(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
            Self::JSXEmpty(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::PrivateName(it) => it.node_id(),
            Self::OptChain(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
            Self::Super(it) => it.node_id(),
        }
    }
}
impl GetSpan for Callee {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for Callee {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl Super {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::This(it) => it.node_id(),
            Self::Array(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
            Self::Unary(it) => it.node_id(),
            Self::Update(it) => it.node_id(),
            Self::Bin(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Member(it) => it.node_id(),
            Self::SuperProp(it) => it.node_id(),
            Self::Cond(it) => it.node_id(),
            Self::Import(it) => it.node_id(),
            Self::Call(it) => it.node_id(),
            Self::New(it) => it.node_id(),
            Self::Seq(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
            Self::Bool(it) => it.node_id(),
            Self::Null(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
            Self::Regex(it) => it.node_id(),
            Self::Tpl(it) => it.node_id(),
            Self::TaggedTpl(it) => it.node_id(),
            Self::Arrow(it) => it.node_id(),
            Self::Class(it) => it.node_id(),
            Self::Yield(it) => it.node_id(),
            Self::MetaProp(it) => it.node_id(),
            Self::Await(it) => it.node_id(),
            Self::Paren(it) => it.node_id(),
            Self::JSXMember(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
            Self::JSXEmpty(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::PrivateName(it) => it.node_id(),
            Self::OptChain(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
            Self::BlockStmt(it) => it.node_id(),
        }
    }
}
impl GetSpan for BlockStmtOrExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for BlockStmtOrExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl AssignTarget {
    #[inline]
    pub fn node_id(self) -> NodeId {
        match self {
            Self::SimpleIdent(it) => it.node_id(),
            Self::SimpleMember(it) => it.node_id(),
            Self::SimpleSuperProp(it) => it.node_id(),
            Self::SimpleParen(it) => it.node_id(),
            Self::SimpleOptChain(it) => it.node_id(),
            Self::SimpleInvalid(it) => it.node_id(),
            Self::PatArray(it) => it.node_id(),
            Self::PatObject(it) => it.node_id(),
            Self::PatternInvalid(it) => it.node_id(),
        }
    }
}
impl GetSpan for AssignTarget {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for AssignTarget {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl AssignTargetPat {
    #[inline]
    pub fn node_id(self) -> NodeId {
        match self {
            Self::Array(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
        }
    }
}
impl GetSpan for AssignTargetPat {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for AssignTargetPat {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl SimpleAssignTarget {
    #[inline]
    pub fn node_id(self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Member(it) => it.node_id(),
            Self::SuperProp(it) => it.node_id(),
            Self::Paren(it) => it.node_id(),
            Self::OptChain(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
        }
    }
}
impl GetSpan for SimpleAssignTarget {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for SimpleAssignTarget {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl OptChainExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Member(it) => it.node_id(),
            Self::Call(it) => it.node_id(),
        }
    }
}
impl GetSpan for OptChainBase {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for OptChainBase {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl OptCall {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Param(it) => it.node_id(),
        }
    }
}
impl GetSpan for ParamOrTsParamProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for ParamOrTsParamProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl Class {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Constructor(it) => it.node_id(),
            Self::Method(it) => it.node_id(),
            Self::PrivateMethod(it) => it.node_id(),
            Self::ClassProp(it) => it.node_id(),
            Self::PrivateProp(it) => it.node_id(),
            Self::Empty(it) => it.node_id(),
            Self::StaticBlock(it) => it.node_id(),
            Self::AutoAccessor(it) => it.node_id(),
        }
    }
}
impl GetSpan for ClassMember {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for ClassMember {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ClassProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
            Self::Private(it) => it.node_id(),
        }
    }
}
impl GetSpan for Key {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for Key {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl AutoAccessor {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Shorthand(it) => it.node_id(),
            Self::KeyValue(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Getter(it) => it.node_id(),
            Self::Setter(it) => it.node_id(),
            Self::Method(it) => it.node_id(),
        }
    }
}
impl GetSpan for Prop {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for Prop {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl KeyValueProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
        }
    }
}
impl GetSpan for PropName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for PropName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ComputedPropName {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::ExprThis(it) => it.node_id(),
            Self::ExprArray(it) => it.node_id(),
            Self::ExprObject(it) => it.node_id(),
            Self::ExprFn(it) => it.node_id(),
            Self::ExprUnary(it) => it.node_id(),
            Self::ExprUpdate(it) => it.node_id(),
            Self::ExprBin(it) => it.node_id(),
            Self::ExprAssign(it) => it.node_id(),
            Self::ExprMember(it) => it.node_id(),
            Self::ExprSuperProp(it) => it.node_id(),
            Self::ExprCond(it) => it.node_id(),
            Self::ExprImport(it) => it.node_id(),
            Self::ExprCall(it) => it.node_id(),
            Self::ExprNew(it) => it.node_id(),
            Self::ExprSeq(it) => it.node_id(),
            Self::ExprIdent(it) => it.node_id(),
            Self::ExprStr(it) => it.node_id(),
            Self::ExprBool(it) => it.node_id(),
            Self::ExprNull(it) => it.node_id(),
            Self::ExprNum(it) => it.node_id(),
            Self::ExprBigInt(it) => it.node_id(),
            Self::ExprRegex(it) => it.node_id(),
            Self::ExprTpl(it) => it.node_id(),
            Self::ExprTaggedTpl(it) => it.node_id(),
            Self::ExprArrow(it) => it.node_id(),
            Self::ExprClass(it) => it.node_id(),
            Self::ExprYield(it) => it.node_id(),
            Self::ExprMetaProp(it) => it.node_id(),
            Self::ExprAwait(it) => it.node_id(),
            Self::ExprParen(it) => it.node_id(),
            Self::ExprJSXMember(it) => it.node_id(),
            Self::ExprJSXNamespacedName(it) => it.node_id(),
            Self::ExprJSXEmpty(it) => it.node_id(),
            Self::ExprJSXElement(it) => it.node_id(),
            Self::ExprJSXFragment(it) => it.node_id(),
            Self::ExprPrivateName(it) => it.node_id(),
            Self::ExprOptChain(it) => it.node_id(),
            Self::ExprInvalid(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
            Self::Array(it) => it.node_id(),
            Self::Rest(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
        }
    }
}
impl GetSpan for Pat {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for Pat {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl ArrayPat {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::KeyValue(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Rest(it) => it.node_id(),
        }
    }
}
impl GetSpan for ObjectPatProp {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for ObjectPatProp {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl KeyValuePatProp {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Str(it) => it.node_id(),
            Self::Bool(it) => it.node_id(),
            Self::Null(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
            Self::Regex(it) => it.node_id(),
        }
    }
}
impl GetSpan for Lit {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for Lit {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl Str {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::JSXMemberExpr(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
        }
    }
}
impl GetSpan for JSXObject {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for JSXObject {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl JSXMemberExpr {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::This(it) => it.node_id(),
            Self::Array(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
            Self::Unary(it) => it.node_id(),
            Self::Update(it) => it.node_id(),
            Self::Bin(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Member(it) => it.node_id(),
            Self::SuperProp(it) => it.node_id(),
            Self::Cond(it) => it.node_id(),
            Self::Import(it) => it.node_id(),
            Self::Call(it) => it.node_id(),
            Self::New(it) => it.node_id(),
            Self::Seq(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
            Self::Bool(it) => it.node_id(),
            Self::Null(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
            Self::Regex(it) => it.node_id(),
            Self::Tpl(it) => it.node_id(),
            Self::TaggedTpl(it) => it.node_id(),
            Self::Arrow(it) => it.node_id(),
            Self::Class(it) => it.node_id(),
            Self::Yield(it) => it.node_id(),
            Self::MetaProp(it) => it.node_id(),
            Self::Await(it) => it.node_id(),
            Self::Paren(it) => it.node_id(),
            Self::JSXMember(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
            Self::JSXEmpty(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::PrivateName(it) => it.node_id(),
            Self::OptChain(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
            Self::JSXEmptyExpr(it) => it.node_id(),
        }
    }
}
impl GetSpan for JSXExpr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for JSXExpr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl JSXSpreadChild {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Ident(it) => it.node_id(),
            Self::JSXMemberExpr(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
        }
    }
}
impl GetSpan for JSXElementName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for JSXElementName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl JSXOpeningElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::JSXAttr(it) => it.node_id(),
            Self::SpreadElement(it) => it.node_id(),
        }
    }
}
impl GetSpan for JSXAttrOrSpread {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for JSXAttrOrSpread {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl JSXClosingElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::Ident(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
        }
    }
}
impl GetSpan for JSXAttrName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for JSXAttrName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl JSXAttrValue {
    #[inline]
    pub fn node_id(self) -> NodeId {
        match self {
            Self::Str(it) => it.node_id(),
            Self::JSXExprContainer(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
        }
    }
}
impl GetSpan for JSXAttrValue {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for JSXAttrValue {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl JSXText {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
        match self {
            Self::JSXText(it) => it.node_id(),
            Self::JSXExprContainer(it) => it.node_id(),
            Self::JSXSpreadChild(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
        }
    }
}
impl GetSpan for JSXElementChild {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.node_id()) }
    }
}
impl SetSpan for JSXElementChild {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.node_id()) = span;
        }
    }
}
impl JSXFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
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
