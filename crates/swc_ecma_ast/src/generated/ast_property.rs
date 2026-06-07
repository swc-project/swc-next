#![allow(unused, clippy::useless_conversion, clippy::identity_op)]
use crate::span::{DUMMY_SP, GetSpan, SetSpan};
use crate::*;
use swc_experimental_allocator::atom::{Atom, Wtf8Atom};
use swc_experimental_allocator::boxed::Box;
impl<'a> Program<'a> {
    #[inline]
    pub const fn is_module(&self) -> bool {
        matches!(self, Self::Module { .. })
    }
    #[inline]
    pub const fn is_script(&self) -> bool {
        matches!(self, Self::Script { .. })
    }
    #[inline]
    pub fn as_module(&self) -> Option<&Module<'a>> {
        match self {
            Self::Module(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_script(&self) -> Option<&Script<'a>> {
        match self {
            Self::Script(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_module(&mut self) -> Option<&mut Module<'a>> {
        match self {
            Self::Module(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_script(&mut self) -> Option<&mut Script<'a>> {
        match self {
            Self::Script(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for Program<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Module(it) => it.span(),
            Self::Script(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for Program<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Module(it) => it.set_span(span),
            Self::Script(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for Module<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Module<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for Script<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Script<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> ModuleItem<'a> {
    #[inline]
    pub const fn is_module_decl(&self) -> bool {
        matches!(self, Self::ModuleDecl { .. })
    }
    #[inline]
    pub const fn is_stmt(&self) -> bool {
        matches!(self, Self::Stmt { .. })
    }
    #[inline]
    pub fn as_module_decl(&self) -> Option<&ModuleDecl<'a>> {
        match self {
            Self::ModuleDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt(&self) -> Option<&Stmt<'a>> {
        match self {
            Self::Stmt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_module_decl(&mut self) -> Option<&mut ModuleDecl<'a>> {
        match self {
            Self::ModuleDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_stmt(&mut self) -> Option<&mut Stmt<'a>> {
        match self {
            Self::Stmt(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for ModuleItem<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::ModuleDecl(it) => it.span(),
            Self::Stmt(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for ModuleItem<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::ModuleDecl(it) => it.set_span(span),
            Self::Stmt(it) => it.set_span(span),
        }
    }
}
impl<'a> ModuleDecl<'a> {
    #[inline]
    pub const fn is_import(&self) -> bool {
        matches!(self, Self::Import { .. })
    }
    #[inline]
    pub const fn is_export_decl(&self) -> bool {
        matches!(self, Self::ExportDecl { .. })
    }
    #[inline]
    pub const fn is_export_named(&self) -> bool {
        matches!(self, Self::ExportNamed { .. })
    }
    #[inline]
    pub const fn is_export_default_decl(&self) -> bool {
        matches!(self, Self::ExportDefaultDecl { .. })
    }
    #[inline]
    pub const fn is_export_default_expr(&self) -> bool {
        matches!(self, Self::ExportDefaultExpr { .. })
    }
    #[inline]
    pub const fn is_export_all(&self) -> bool {
        matches!(self, Self::ExportAll { .. })
    }
    #[inline]
    pub fn as_import(&self) -> Option<&ImportDecl<'a>> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_decl(&self) -> Option<&ExportDecl<'a>> {
        match self {
            Self::ExportDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_named(&self) -> Option<&NamedExport<'a>> {
        match self {
            Self::ExportNamed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_default_decl(&self) -> Option<&ExportDefaultDecl<'a>> {
        match self {
            Self::ExportDefaultDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_default_expr(&self) -> Option<&ExportDefaultExpr<'a>> {
        match self {
            Self::ExportDefaultExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_all(&self) -> Option<&ExportAll<'a>> {
        match self {
            Self::ExportAll(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_import(&mut self) -> Option<&mut ImportDecl<'a>> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_export_decl(&mut self) -> Option<&mut ExportDecl<'a>> {
        match self {
            Self::ExportDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_export_named(&mut self) -> Option<&mut NamedExport<'a>> {
        match self {
            Self::ExportNamed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_export_default_decl(&mut self) -> Option<&mut ExportDefaultDecl<'a>> {
        match self {
            Self::ExportDefaultDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_export_default_expr(&mut self) -> Option<&mut ExportDefaultExpr<'a>> {
        match self {
            Self::ExportDefaultExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_export_all(&mut self) -> Option<&mut ExportAll<'a>> {
        match self {
            Self::ExportAll(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for ModuleDecl<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Import(it) => it.span(),
            Self::ExportDecl(it) => it.span(),
            Self::ExportNamed(it) => it.span(),
            Self::ExportDefaultDecl(it) => it.span(),
            Self::ExportDefaultExpr(it) => it.span(),
            Self::ExportAll(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for ModuleDecl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Import(it) => it.set_span(span),
            Self::ExportDecl(it) => it.set_span(span),
            Self::ExportNamed(it) => it.set_span(span),
            Self::ExportDefaultDecl(it) => it.set_span(span),
            Self::ExportDefaultExpr(it) => it.set_span(span),
            Self::ExportAll(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for ImportDecl<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ImportDecl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> ImportSpecifier<'a> {
    #[inline]
    pub const fn is_named(&self) -> bool {
        matches!(self, Self::Named { .. })
    }
    #[inline]
    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Default { .. })
    }
    #[inline]
    pub const fn is_namespace(&self) -> bool {
        matches!(self, Self::Namespace { .. })
    }
    #[inline]
    pub fn as_named(&self) -> Option<&ImportNamedSpecifier<'a>> {
        match self {
            Self::Named(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_default(&self) -> Option<&ImportDefaultSpecifier<'a>> {
        match self {
            Self::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_namespace(&self) -> Option<&ImportStarAsSpecifier<'a>> {
        match self {
            Self::Namespace(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_named(&mut self) -> Option<&mut ImportNamedSpecifier<'a>> {
        match self {
            Self::Named(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_default(&mut self) -> Option<&mut ImportDefaultSpecifier<'a>> {
        match self {
            Self::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_namespace(&mut self) -> Option<&mut ImportStarAsSpecifier<'a>> {
        match self {
            Self::Namespace(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for ImportSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Named(it) => it.span(),
            Self::Default(it) => it.span(),
            Self::Namespace(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for ImportSpecifier<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Named(it) => it.set_span(span),
            Self::Default(it) => it.set_span(span),
            Self::Namespace(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for ImportNamedSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ImportNamedSpecifier<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ImportDefaultSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ImportDefaultSpecifier<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ImportStarAsSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ImportStarAsSpecifier<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ExportDecl<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ExportDecl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for NamedExport<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for NamedExport<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> ExportSpecifier<'a> {
    #[inline]
    pub const fn is_namespace(&self) -> bool {
        matches!(self, Self::Namespace { .. })
    }
    #[inline]
    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Default { .. })
    }
    #[inline]
    pub const fn is_named(&self) -> bool {
        matches!(self, Self::Named { .. })
    }
    #[inline]
    pub fn as_namespace(&self) -> Option<&ExportNamespaceSpecifier<'a>> {
        match self {
            Self::Namespace(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_default(&self) -> Option<&ExportDefaultSpecifier<'a>> {
        match self {
            Self::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_named(&self) -> Option<&ExportNamedSpecifier<'a>> {
        match self {
            Self::Named(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_namespace(&mut self) -> Option<&mut ExportNamespaceSpecifier<'a>> {
        match self {
            Self::Namespace(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_default(&mut self) -> Option<&mut ExportDefaultSpecifier<'a>> {
        match self {
            Self::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_named(&mut self) -> Option<&mut ExportNamedSpecifier<'a>> {
        match self {
            Self::Named(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for ExportSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Namespace(it) => it.span(),
            Self::Default(it) => it.span(),
            Self::Named(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for ExportSpecifier<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Namespace(it) => it.set_span(span),
            Self::Default(it) => it.set_span(span),
            Self::Named(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for ExportNamespaceSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ExportNamespaceSpecifier<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> ModuleExportName<'a> {
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub const fn is_str(&self) -> bool {
        matches!(self, Self::Str { .. })
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&Ident<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(&self) -> Option<&Str<'a>> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut Ident<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_str(&mut self) -> Option<&mut Str<'a>> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for ModuleExportName<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Ident(it) => it.span(),
            Self::Str(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for ModuleExportName<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Ident(it) => it.set_span(span),
            Self::Str(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for ExportDefaultSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.exported.span()
    }
}
impl<'a> SetSpan for ExportDefaultSpecifier<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.exported.set_span(span);
    }
}
impl<'a> GetSpan for ExportNamedSpecifier<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ExportNamedSpecifier<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ExportDefaultDecl<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ExportDefaultDecl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> DefaultDecl<'a> {
    #[inline]
    pub const fn is_class(&self) -> bool {
        matches!(self, Self::Class { .. })
    }
    #[inline]
    pub const fn is_fn(&self) -> bool {
        matches!(self, Self::Fn { .. })
    }
    #[inline]
    pub fn as_class(&self) -> Option<&ClassExpr<'a>> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(&self) -> Option<&FnExpr<'a>> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_class(&mut self) -> Option<&mut ClassExpr<'a>> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_fn(&mut self) -> Option<&mut FnExpr<'a>> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for DefaultDecl<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Class(it) => it.span(),
            Self::Fn(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for DefaultDecl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Class(it) => it.set_span(span),
            Self::Fn(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for ExportDefaultExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ExportDefaultExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ExportAll<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ExportAll<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for BlockStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for BlockStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> Stmt<'a> {
    #[inline]
    pub const fn is_block(&self) -> bool {
        matches!(self, Self::Block { .. })
    }
    #[inline]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty { .. })
    }
    #[inline]
    pub const fn is_debugger(&self) -> bool {
        matches!(self, Self::Debugger { .. })
    }
    #[inline]
    pub const fn is_with(&self) -> bool {
        matches!(self, Self::With { .. })
    }
    #[inline]
    pub const fn is_return(&self) -> bool {
        matches!(self, Self::Return { .. })
    }
    #[inline]
    pub const fn is_labeled(&self) -> bool {
        matches!(self, Self::Labeled { .. })
    }
    #[inline]
    pub const fn is_break(&self) -> bool {
        matches!(self, Self::Break { .. })
    }
    #[inline]
    pub const fn is_continue(&self) -> bool {
        matches!(self, Self::Continue { .. })
    }
    #[inline]
    pub const fn is_if(&self) -> bool {
        matches!(self, Self::If { .. })
    }
    #[inline]
    pub const fn is_switch(&self) -> bool {
        matches!(self, Self::Switch { .. })
    }
    #[inline]
    pub const fn is_throw(&self) -> bool {
        matches!(self, Self::Throw { .. })
    }
    #[inline]
    pub const fn is_try(&self) -> bool {
        matches!(self, Self::Try { .. })
    }
    #[inline]
    pub const fn is_while(&self) -> bool {
        matches!(self, Self::While { .. })
    }
    #[inline]
    pub const fn is_do_while(&self) -> bool {
        matches!(self, Self::DoWhile { .. })
    }
    #[inline]
    pub const fn is_for(&self) -> bool {
        matches!(self, Self::For { .. })
    }
    #[inline]
    pub const fn is_for_in(&self) -> bool {
        matches!(self, Self::ForIn { .. })
    }
    #[inline]
    pub const fn is_for_of(&self) -> bool {
        matches!(self, Self::ForOf { .. })
    }
    #[inline]
    pub const fn is_decl(&self) -> bool {
        matches!(self, Self::Decl { .. })
    }
    #[inline]
    pub const fn is_expr(&self) -> bool {
        matches!(self, Self::Expr { .. })
    }
    #[inline]
    pub fn as_block(&self) -> Option<&BlockStmt<'a>> {
        match self {
            Self::Block(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_empty(&self) -> Option<&EmptyStmt> {
        match self {
            Self::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_debugger(&self) -> Option<&DebuggerStmt> {
        match self {
            Self::Debugger(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_with(&self) -> Option<&WithStmt<'a>> {
        match self {
            Self::With(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_return(&self) -> Option<&ReturnStmt<'a>> {
        match self {
            Self::Return(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_labeled(&self) -> Option<&LabeledStmt<'a>> {
        match self {
            Self::Labeled(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_break(&self) -> Option<&BreakStmt<'a>> {
        match self {
            Self::Break(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_continue(&self) -> Option<&ContinueStmt<'a>> {
        match self {
            Self::Continue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_if(&self) -> Option<&IfStmt<'a>> {
        match self {
            Self::If(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_switch(&self) -> Option<&SwitchStmt<'a>> {
        match self {
            Self::Switch(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_throw(&self) -> Option<&ThrowStmt<'a>> {
        match self {
            Self::Throw(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_try(&self) -> Option<&TryStmt<'a>> {
        match self {
            Self::Try(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_while(&self) -> Option<&WhileStmt<'a>> {
        match self {
            Self::While(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_do_while(&self) -> Option<&DoWhileStmt<'a>> {
        match self {
            Self::DoWhile(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for(&self) -> Option<&ForStmt<'a>> {
        match self {
            Self::For(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_in(&self) -> Option<&ForInStmt<'a>> {
        match self {
            Self::ForIn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_of(&self) -> Option<&ForOfStmt<'a>> {
        match self {
            Self::ForOf(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_decl(&self) -> Option<&Decl<'a>> {
        match self {
            Self::Decl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&ExprStmt<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_block(&mut self) -> Option<&mut BlockStmt<'a>> {
        match self {
            Self::Block(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_empty(&mut self) -> Option<&mut EmptyStmt> {
        match self {
            Self::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_debugger(&mut self) -> Option<&mut DebuggerStmt> {
        match self {
            Self::Debugger(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_with(&mut self) -> Option<&mut WithStmt<'a>> {
        match self {
            Self::With(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_return(&mut self) -> Option<&mut ReturnStmt<'a>> {
        match self {
            Self::Return(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_labeled(&mut self) -> Option<&mut LabeledStmt<'a>> {
        match self {
            Self::Labeled(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_break(&mut self) -> Option<&mut BreakStmt<'a>> {
        match self {
            Self::Break(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_continue(&mut self) -> Option<&mut ContinueStmt<'a>> {
        match self {
            Self::Continue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_if(&mut self) -> Option<&mut IfStmt<'a>> {
        match self {
            Self::If(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_switch(&mut self) -> Option<&mut SwitchStmt<'a>> {
        match self {
            Self::Switch(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_throw(&mut self) -> Option<&mut ThrowStmt<'a>> {
        match self {
            Self::Throw(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_try(&mut self) -> Option<&mut TryStmt<'a>> {
        match self {
            Self::Try(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_while(&mut self) -> Option<&mut WhileStmt<'a>> {
        match self {
            Self::While(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_do_while(&mut self) -> Option<&mut DoWhileStmt<'a>> {
        match self {
            Self::DoWhile(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_for(&mut self) -> Option<&mut ForStmt<'a>> {
        match self {
            Self::For(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_for_in(&mut self) -> Option<&mut ForInStmt<'a>> {
        match self {
            Self::ForIn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_for_of(&mut self) -> Option<&mut ForOfStmt<'a>> {
        match self {
            Self::ForOf(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_decl(&mut self) -> Option<&mut Decl<'a>> {
        match self {
            Self::Decl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_expr(&mut self) -> Option<&mut ExprStmt<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for Stmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Block(it) => it.span(),
            Self::Empty(it) => it.span(),
            Self::Debugger(it) => it.span(),
            Self::With(it) => it.span(),
            Self::Return(it) => it.span(),
            Self::Labeled(it) => it.span(),
            Self::Break(it) => it.span(),
            Self::Continue(it) => it.span(),
            Self::If(it) => it.span(),
            Self::Switch(it) => it.span(),
            Self::Throw(it) => it.span(),
            Self::Try(it) => it.span(),
            Self::While(it) => it.span(),
            Self::DoWhile(it) => it.span(),
            Self::For(it) => it.span(),
            Self::ForIn(it) => it.span(),
            Self::ForOf(it) => it.span(),
            Self::Decl(it) => it.span(),
            Self::Expr(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for Stmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Block(it) => it.set_span(span),
            Self::Empty(it) => it.set_span(span),
            Self::Debugger(it) => it.set_span(span),
            Self::With(it) => it.set_span(span),
            Self::Return(it) => it.set_span(span),
            Self::Labeled(it) => it.set_span(span),
            Self::Break(it) => it.set_span(span),
            Self::Continue(it) => it.set_span(span),
            Self::If(it) => it.set_span(span),
            Self::Switch(it) => it.set_span(span),
            Self::Throw(it) => it.set_span(span),
            Self::Try(it) => it.set_span(span),
            Self::While(it) => it.set_span(span),
            Self::DoWhile(it) => it.set_span(span),
            Self::For(it) => it.set_span(span),
            Self::ForIn(it) => it.set_span(span),
            Self::ForOf(it) => it.set_span(span),
            Self::Decl(it) => it.set_span(span),
            Self::Expr(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for ExprStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ExprStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for EmptyStmt {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for EmptyStmt {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for DebuggerStmt {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for DebuggerStmt {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for WithStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for WithStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ReturnStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ReturnStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for LabeledStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for LabeledStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for BreakStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for BreakStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ContinueStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ContinueStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for IfStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for IfStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for SwitchStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for SwitchStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ThrowStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ThrowStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for TryStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for TryStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for WhileStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for WhileStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for DoWhileStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for DoWhileStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ForStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ForStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ForInStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ForInStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ForOfStmt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ForOfStmt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for SwitchCase<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for SwitchCase<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for CatchClause<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for CatchClause<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> ForHead<'a> {
    #[inline]
    pub const fn is_var_decl(&self) -> bool {
        matches!(self, Self::VarDecl { .. })
    }
    #[inline]
    pub const fn is_using_decl(&self) -> bool {
        matches!(self, Self::UsingDecl { .. })
    }
    #[inline]
    pub const fn is_pat(&self) -> bool {
        matches!(self, Self::Pat { .. })
    }
    #[inline]
    pub fn as_var_decl(&self) -> Option<&VarDecl<'a>> {
        match self {
            Self::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_using_decl(&self) -> Option<&UsingDecl<'a>> {
        match self {
            Self::UsingDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pat(&self) -> Option<&Pat<'a>> {
        match self {
            Self::Pat(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_var_decl(&mut self) -> Option<&mut VarDecl<'a>> {
        match self {
            Self::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_using_decl(&mut self) -> Option<&mut UsingDecl<'a>> {
        match self {
            Self::UsingDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_pat(&mut self) -> Option<&mut Pat<'a>> {
        match self {
            Self::Pat(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for ForHead<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::VarDecl(it) => it.span(),
            Self::UsingDecl(it) => it.span(),
            Self::Pat(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for ForHead<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::VarDecl(it) => it.set_span(span),
            Self::UsingDecl(it) => it.set_span(span),
            Self::Pat(it) => it.set_span(span),
        }
    }
}
impl<'a> VarDeclOrExpr<'a> {
    #[inline]
    pub const fn is_var_decl(&self) -> bool {
        matches!(self, Self::VarDecl { .. })
    }
    #[inline]
    pub const fn is_expr(&self) -> bool {
        matches!(self, Self::Expr { .. })
    }
    #[inline]
    pub fn as_var_decl(&self) -> Option<&VarDecl<'a>> {
        match self {
            Self::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_var_decl(&mut self) -> Option<&mut VarDecl<'a>> {
        match self {
            Self::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_expr(&mut self) -> Option<&mut Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for VarDeclOrExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::VarDecl(it) => it.span(),
            Self::Expr(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for VarDeclOrExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::VarDecl(it) => it.set_span(span),
            Self::Expr(it) => it.set_span(span),
        }
    }
}
impl<'a> Decl<'a> {
    #[inline]
    pub const fn is_class(&self) -> bool {
        matches!(self, Self::Class { .. })
    }
    #[inline]
    pub const fn is_fn(&self) -> bool {
        matches!(self, Self::Fn { .. })
    }
    #[inline]
    pub const fn is_var(&self) -> bool {
        matches!(self, Self::Var { .. })
    }
    #[inline]
    pub const fn is_using(&self) -> bool {
        matches!(self, Self::Using { .. })
    }
    #[inline]
    pub fn as_class(&self) -> Option<&ClassDecl<'a>> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(&self) -> Option<&FnDecl<'a>> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_var(&self) -> Option<&VarDecl<'a>> {
        match self {
            Self::Var(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_using(&self) -> Option<&UsingDecl<'a>> {
        match self {
            Self::Using(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_class(&mut self) -> Option<&mut ClassDecl<'a>> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_fn(&mut self) -> Option<&mut FnDecl<'a>> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_var(&mut self) -> Option<&mut VarDecl<'a>> {
        match self {
            Self::Var(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_using(&mut self) -> Option<&mut UsingDecl<'a>> {
        match self {
            Self::Using(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for Decl<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Class(it) => it.span(),
            Self::Fn(it) => it.span(),
            Self::Var(it) => it.span(),
            Self::Using(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for Decl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Class(it) => it.set_span(span),
            Self::Fn(it) => it.set_span(span),
            Self::Var(it) => it.set_span(span),
            Self::Using(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for FnDecl<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.function.span()
    }
}
impl<'a> SetSpan for FnDecl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.function.set_span(span);
    }
}
impl<'a> GetSpan for ClassDecl<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.class.span()
    }
}
impl<'a> SetSpan for ClassDecl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.class.set_span(span);
    }
}
impl<'a> GetSpan for VarDecl<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for VarDecl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for VarDeclarator<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for VarDeclarator<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for UsingDecl<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for UsingDecl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> Expr<'a> {
    #[inline]
    pub const fn is_this(&self) -> bool {
        matches!(self, Self::This { .. })
    }
    #[inline]
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array { .. })
    }
    #[inline]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object { .. })
    }
    #[inline]
    pub const fn is_fn(&self) -> bool {
        matches!(self, Self::Fn { .. })
    }
    #[inline]
    pub const fn is_unary(&self) -> bool {
        matches!(self, Self::Unary { .. })
    }
    #[inline]
    pub const fn is_update(&self) -> bool {
        matches!(self, Self::Update { .. })
    }
    #[inline]
    pub const fn is_bin(&self) -> bool {
        matches!(self, Self::Bin { .. })
    }
    #[inline]
    pub const fn is_assign(&self) -> bool {
        matches!(self, Self::Assign { .. })
    }
    #[inline]
    pub const fn is_member(&self) -> bool {
        matches!(self, Self::Member { .. })
    }
    #[inline]
    pub const fn is_super_prop(&self) -> bool {
        matches!(self, Self::SuperProp { .. })
    }
    #[inline]
    pub const fn is_cond(&self) -> bool {
        matches!(self, Self::Cond { .. })
    }
    #[inline]
    pub const fn is_call(&self) -> bool {
        matches!(self, Self::Call { .. })
    }
    #[inline]
    pub const fn is_new(&self) -> bool {
        matches!(self, Self::New { .. })
    }
    #[inline]
    pub const fn is_seq(&self) -> bool {
        matches!(self, Self::Seq { .. })
    }
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub const fn is_lit(&self) -> bool {
        matches!(self, Self::Lit { .. })
    }
    #[inline]
    pub const fn is_tpl(&self) -> bool {
        matches!(self, Self::Tpl { .. })
    }
    #[inline]
    pub const fn is_tagged_tpl(&self) -> bool {
        matches!(self, Self::TaggedTpl { .. })
    }
    #[inline]
    pub const fn is_arrow(&self) -> bool {
        matches!(self, Self::Arrow { .. })
    }
    #[inline]
    pub const fn is_class(&self) -> bool {
        matches!(self, Self::Class { .. })
    }
    #[inline]
    pub const fn is_yield(&self) -> bool {
        matches!(self, Self::Yield { .. })
    }
    #[inline]
    pub const fn is_meta_prop(&self) -> bool {
        matches!(self, Self::MetaProp { .. })
    }
    #[inline]
    pub const fn is_await(&self) -> bool {
        matches!(self, Self::Await { .. })
    }
    #[inline]
    pub const fn is_paren(&self) -> bool {
        matches!(self, Self::Paren { .. })
    }
    #[inline]
    pub const fn is_jsx_member(&self) -> bool {
        matches!(self, Self::JSXMember { .. })
    }
    #[inline]
    pub const fn is_jsx_namespaced_name(&self) -> bool {
        matches!(self, Self::JSXNamespacedName { .. })
    }
    #[inline]
    pub const fn is_jsx_empty(&self) -> bool {
        matches!(self, Self::JSXEmpty { .. })
    }
    #[inline]
    pub const fn is_jsx_element(&self) -> bool {
        matches!(self, Self::JSXElement { .. })
    }
    #[inline]
    pub const fn is_jsx_fragment(&self) -> bool {
        matches!(self, Self::JSXFragment { .. })
    }
    #[inline]
    pub const fn is_private_name(&self) -> bool {
        matches!(self, Self::PrivateName { .. })
    }
    #[inline]
    pub const fn is_opt_chain(&self) -> bool {
        matches!(self, Self::OptChain { .. })
    }
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid { .. })
    }
    #[inline]
    pub fn as_this(&self) -> Option<&ThisExpr> {
        match self {
            Self::This(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(&self) -> Option<&ArrayLit<'a>> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(&self) -> Option<&ObjectLit<'a>> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(&self) -> Option<&FnExpr<'a>> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary(&self) -> Option<&UnaryExpr<'a>> {
        match self {
            Self::Unary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_update(&self) -> Option<&UpdateExpr<'a>> {
        match self {
            Self::Update(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bin(&self) -> Option<&BinExpr<'a>> {
        match self {
            Self::Bin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(&self) -> Option<&AssignExpr<'a>> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(&self) -> Option<&MemberExpr<'a>> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(&self) -> Option<&SuperPropExpr<'a>> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_cond(&self) -> Option<&CondExpr<'a>> {
        match self {
            Self::Cond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(&self) -> Option<&CallExpr<'a>> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_new(&self) -> Option<&NewExpr<'a>> {
        match self {
            Self::New(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_seq(&self) -> Option<&SeqExpr<'a>> {
        match self {
            Self::Seq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&Ident<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_lit(&self) -> Option<&Lit<'a>> {
        match self {
            Self::Lit(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tpl(&self) -> Option<&Tpl<'a>> {
        match self {
            Self::Tpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tagged_tpl(&self) -> Option<&TaggedTpl<'a>> {
        match self {
            Self::TaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_arrow(&self) -> Option<&ArrowExpr<'a>> {
        match self {
            Self::Arrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(&self) -> Option<&ClassExpr<'a>> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_yield(&self) -> Option<&YieldExpr<'a>> {
        match self {
            Self::Yield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_meta_prop(&self) -> Option<&MetaPropExpr> {
        match self {
            Self::MetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_await(&self) -> Option<&AwaitExpr<'a>> {
        match self {
            Self::Await(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(&self) -> Option<&ParenExpr<'a>> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member(&self) -> Option<&JSXMemberExpr<'a>> {
        match self {
            Self::JSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(&self) -> Option<&JSXNamespacedName<'a>> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty(&self) -> Option<&JSXEmptyExpr> {
        match self {
            Self::JSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(&self) -> Option<&JSXElement<'a>> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(&self) -> Option<&JSXFragment<'a>> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(&self) -> Option<&PrivateName<'a>> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(&self) -> Option<&OptChainExpr<'a>> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(&self) -> Option<&Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_this(&mut self) -> Option<&mut ThisExpr> {
        match self {
            Self::This(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_array(&mut self) -> Option<&mut ArrayLit<'a>> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_object(&mut self) -> Option<&mut ObjectLit<'a>> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_fn(&mut self) -> Option<&mut FnExpr<'a>> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_unary(&mut self) -> Option<&mut UnaryExpr<'a>> {
        match self {
            Self::Unary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_update(&mut self) -> Option<&mut UpdateExpr<'a>> {
        match self {
            Self::Update(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_bin(&mut self) -> Option<&mut BinExpr<'a>> {
        match self {
            Self::Bin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_assign(&mut self) -> Option<&mut AssignExpr<'a>> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_member(&mut self) -> Option<&mut MemberExpr<'a>> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_super_prop(&mut self) -> Option<&mut SuperPropExpr<'a>> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_cond(&mut self) -> Option<&mut CondExpr<'a>> {
        match self {
            Self::Cond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_call(&mut self) -> Option<&mut CallExpr<'a>> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_new(&mut self) -> Option<&mut NewExpr<'a>> {
        match self {
            Self::New(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_seq(&mut self) -> Option<&mut SeqExpr<'a>> {
        match self {
            Self::Seq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut Ident<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_lit(&mut self) -> Option<&mut Lit<'a>> {
        match self {
            Self::Lit(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_tpl(&mut self) -> Option<&mut Tpl<'a>> {
        match self {
            Self::Tpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_tagged_tpl(&mut self) -> Option<&mut TaggedTpl<'a>> {
        match self {
            Self::TaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_arrow(&mut self) -> Option<&mut ArrowExpr<'a>> {
        match self {
            Self::Arrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_class(&mut self) -> Option<&mut ClassExpr<'a>> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_yield(&mut self) -> Option<&mut YieldExpr<'a>> {
        match self {
            Self::Yield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_meta_prop(&mut self) -> Option<&mut MetaPropExpr> {
        match self {
            Self::MetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_await(&mut self) -> Option<&mut AwaitExpr<'a>> {
        match self {
            Self::Await(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_paren(&mut self) -> Option<&mut ParenExpr<'a>> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_member(&mut self) -> Option<&mut JSXMemberExpr<'a>> {
        match self {
            Self::JSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_namespaced_name(&mut self) -> Option<&mut JSXNamespacedName<'a>> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_empty(&mut self) -> Option<&mut JSXEmptyExpr> {
        match self {
            Self::JSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_element(&mut self) -> Option<&mut JSXElement<'a>> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_fragment(&mut self) -> Option<&mut JSXFragment<'a>> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_private_name(&mut self) -> Option<&mut PrivateName<'a>> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_opt_chain(&mut self) -> Option<&mut OptChainExpr<'a>> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_invalid(&mut self) -> Option<&mut Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for Expr<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::This(it) => it.span(),
            Self::Array(it) => it.span(),
            Self::Object(it) => it.span(),
            Self::Fn(it) => it.span(),
            Self::Unary(it) => it.span(),
            Self::Update(it) => it.span(),
            Self::Bin(it) => it.span(),
            Self::Assign(it) => it.span(),
            Self::Member(it) => it.span(),
            Self::SuperProp(it) => it.span(),
            Self::Cond(it) => it.span(),
            Self::Call(it) => it.span(),
            Self::New(it) => it.span(),
            Self::Seq(it) => it.span(),
            Self::Ident(it) => it.span(),
            Self::Lit(it) => it.span(),
            Self::Tpl(it) => it.span(),
            Self::TaggedTpl(it) => it.span(),
            Self::Arrow(it) => it.span(),
            Self::Class(it) => it.span(),
            Self::Yield(it) => it.span(),
            Self::MetaProp(it) => it.span(),
            Self::Await(it) => it.span(),
            Self::Paren(it) => it.span(),
            Self::JSXMember(it) => it.span(),
            Self::JSXNamespacedName(it) => it.span(),
            Self::JSXEmpty(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::PrivateName(it) => it.span(),
            Self::OptChain(it) => it.span(),
            Self::Invalid(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for Expr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::This(it) => it.set_span(span),
            Self::Array(it) => it.set_span(span),
            Self::Object(it) => it.set_span(span),
            Self::Fn(it) => it.set_span(span),
            Self::Unary(it) => it.set_span(span),
            Self::Update(it) => it.set_span(span),
            Self::Bin(it) => it.set_span(span),
            Self::Assign(it) => it.set_span(span),
            Self::Member(it) => it.set_span(span),
            Self::SuperProp(it) => it.set_span(span),
            Self::Cond(it) => it.set_span(span),
            Self::Call(it) => it.set_span(span),
            Self::New(it) => it.set_span(span),
            Self::Seq(it) => it.set_span(span),
            Self::Ident(it) => it.set_span(span),
            Self::Lit(it) => it.set_span(span),
            Self::Tpl(it) => it.set_span(span),
            Self::TaggedTpl(it) => it.set_span(span),
            Self::Arrow(it) => it.set_span(span),
            Self::Class(it) => it.set_span(span),
            Self::Yield(it) => it.set_span(span),
            Self::MetaProp(it) => it.set_span(span),
            Self::Await(it) => it.set_span(span),
            Self::Paren(it) => it.set_span(span),
            Self::JSXMember(it) => it.set_span(span),
            Self::JSXNamespacedName(it) => it.set_span(span),
            Self::JSXEmpty(it) => it.set_span(span),
            Self::JSXElement(it) => it.set_span(span),
            Self::JSXFragment(it) => it.set_span(span),
            Self::PrivateName(it) => it.set_span(span),
            Self::OptChain(it) => it.set_span(span),
            Self::Invalid(it) => it.set_span(span),
        }
    }
}
impl GetSpan for ThisExpr {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for ThisExpr {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ArrayLit<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ArrayLit<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ObjectLit<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ObjectLit<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> PropOrSpread<'a> {
    #[inline]
    pub const fn is_spread(&self) -> bool {
        matches!(self, Self::Spread { .. })
    }
    #[inline]
    pub const fn is_prop(&self) -> bool {
        matches!(self, Self::Prop { .. })
    }
    #[inline]
    pub fn as_spread(&self) -> Option<&SpreadElement<'a>> {
        match self {
            Self::Spread(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_prop(&self) -> Option<&Prop<'a>> {
        match self {
            Self::Prop(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_spread(&mut self) -> Option<&mut SpreadElement<'a>> {
        match self {
            Self::Spread(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_prop(&mut self) -> Option<&mut Prop<'a>> {
        match self {
            Self::Prop(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for PropOrSpread<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Spread(it) => it.span(),
            Self::Prop(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for PropOrSpread<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Spread(it) => it.set_span(span),
            Self::Prop(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for SpreadElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.dot3_token.start, self.expr.span_hi())
    }
}
impl<'a> SetSpan for SpreadElement<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.dot3_token.start = span.start;
        let current = self.expr.span();
        self.expr.set_span(Span::new(current.start, span.end));
    }
}
impl<'a> GetSpan for UnaryExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for UnaryExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for UpdateExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for UpdateExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for BinExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for BinExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for FnExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.function.span()
    }
}
impl<'a> SetSpan for FnExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.function.set_span(span);
    }
}
impl<'a> GetSpan for ClassExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.class.span()
    }
}
impl<'a> SetSpan for ClassExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.class.set_span(span);
    }
}
impl<'a> GetSpan for AssignExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for AssignExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for MemberExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for MemberExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> MemberProp<'a> {
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub const fn is_private_name(&self) -> bool {
        matches!(self, Self::PrivateName { .. })
    }
    #[inline]
    pub const fn is_computed(&self) -> bool {
        matches!(self, Self::Computed { .. })
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&IdentName<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(&self) -> Option<&PrivateName<'a>> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(&self) -> Option<&ComputedPropName<'a>> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut IdentName<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_private_name(&mut self) -> Option<&mut PrivateName<'a>> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_computed(&mut self) -> Option<&mut ComputedPropName<'a>> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for MemberProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Ident(it) => it.span(),
            Self::PrivateName(it) => it.span(),
            Self::Computed(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for MemberProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Ident(it) => it.set_span(span),
            Self::PrivateName(it) => it.set_span(span),
            Self::Computed(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for SuperPropExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for SuperPropExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> SuperProp<'a> {
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub const fn is_computed(&self) -> bool {
        matches!(self, Self::Computed { .. })
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&IdentName<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(&self) -> Option<&ComputedPropName<'a>> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut IdentName<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_computed(&mut self) -> Option<&mut ComputedPropName<'a>> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for SuperProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Ident(it) => it.span(),
            Self::Computed(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for SuperProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Ident(it) => it.set_span(span),
            Self::Computed(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for CondExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for CondExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for CallExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for CallExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for NewExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for NewExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for SeqExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for SeqExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ArrowExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ArrowExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for YieldExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for YieldExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for MetaPropExpr {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for MetaPropExpr {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for AwaitExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for AwaitExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for Tpl<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Tpl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for TaggedTpl<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for TaggedTpl<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for TplElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for TplElement<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ParenExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ParenExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> Callee<'a> {
    #[inline]
    pub const fn is_super(&self) -> bool {
        matches!(self, Self::Super { .. })
    }
    #[inline]
    pub const fn is_import(&self) -> bool {
        matches!(self, Self::Import { .. })
    }
    #[inline]
    pub const fn is_expr(&self) -> bool {
        matches!(self, Self::Expr { .. })
    }
    #[inline]
    pub fn as_super(&self) -> Option<&Super> {
        match self {
            Self::Super(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import(&self) -> Option<&Import> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_super(&mut self) -> Option<&mut Super> {
        match self {
            Self::Super(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_import(&mut self) -> Option<&mut Import> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_expr(&mut self) -> Option<&mut Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for Callee<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Super(it) => it.span(),
            Self::Import(it) => it.span(),
            Self::Expr(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for Callee<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Super(it) => it.set_span(span),
            Self::Import(it) => it.set_span(span),
            Self::Expr(it) => it.set_span(span),
        }
    }
}
impl GetSpan for Super {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for Super {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for Import {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for Import {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> BlockStmtOrExpr<'a> {
    #[inline]
    pub const fn is_block_stmt(&self) -> bool {
        matches!(self, Self::BlockStmt { .. })
    }
    #[inline]
    pub const fn is_expr(&self) -> bool {
        matches!(self, Self::Expr { .. })
    }
    #[inline]
    pub fn as_block_stmt(&self) -> Option<&BlockStmt<'a>> {
        match self {
            Self::BlockStmt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_block_stmt(&mut self) -> Option<&mut BlockStmt<'a>> {
        match self {
            Self::BlockStmt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_expr(&mut self) -> Option<&mut Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for BlockStmtOrExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::BlockStmt(it) => it.span(),
            Self::Expr(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for BlockStmtOrExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::BlockStmt(it) => it.set_span(span),
            Self::Expr(it) => it.set_span(span),
        }
    }
}
impl<'a> AssignTarget<'a> {
    #[inline]
    pub const fn is_simple(&self) -> bool {
        matches!(self, Self::Simple { .. })
    }
    #[inline]
    pub const fn is_pat(&self) -> bool {
        matches!(self, Self::Pat { .. })
    }
    #[inline]
    pub fn as_simple(&self) -> Option<&SimpleAssignTarget<'a>> {
        match self {
            Self::Simple(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pat(&self) -> Option<&AssignTargetPat<'a>> {
        match self {
            Self::Pat(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_simple(&mut self) -> Option<&mut SimpleAssignTarget<'a>> {
        match self {
            Self::Simple(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_pat(&mut self) -> Option<&mut AssignTargetPat<'a>> {
        match self {
            Self::Pat(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for AssignTarget<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Simple(it) => it.span(),
            Self::Pat(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for AssignTarget<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Simple(it) => it.set_span(span),
            Self::Pat(it) => it.set_span(span),
        }
    }
}
impl<'a> AssignTargetPat<'a> {
    #[inline]
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array { .. })
    }
    #[inline]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object { .. })
    }
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid { .. })
    }
    #[inline]
    pub fn as_array(&self) -> Option<&ArrayPat<'a>> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(&self) -> Option<&ObjectPat<'a>> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(&self) -> Option<&Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_array(&mut self) -> Option<&mut ArrayPat<'a>> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_object(&mut self) -> Option<&mut ObjectPat<'a>> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_invalid(&mut self) -> Option<&mut Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for AssignTargetPat<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Array(it) => it.span(),
            Self::Object(it) => it.span(),
            Self::Invalid(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for AssignTargetPat<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Array(it) => it.set_span(span),
            Self::Object(it) => it.set_span(span),
            Self::Invalid(it) => it.set_span(span),
        }
    }
}
impl<'a> SimpleAssignTarget<'a> {
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub const fn is_member(&self) -> bool {
        matches!(self, Self::Member { .. })
    }
    #[inline]
    pub const fn is_super_prop(&self) -> bool {
        matches!(self, Self::SuperProp { .. })
    }
    #[inline]
    pub const fn is_paren(&self) -> bool {
        matches!(self, Self::Paren { .. })
    }
    #[inline]
    pub const fn is_opt_chain(&self) -> bool {
        matches!(self, Self::OptChain { .. })
    }
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid { .. })
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&BindingIdent<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(&self) -> Option<&MemberExpr<'a>> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(&self) -> Option<&SuperPropExpr<'a>> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(&self) -> Option<&ParenExpr<'a>> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(&self) -> Option<&OptChainExpr<'a>> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(&self) -> Option<&Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut BindingIdent<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_member(&mut self) -> Option<&mut MemberExpr<'a>> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_super_prop(&mut self) -> Option<&mut SuperPropExpr<'a>> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_paren(&mut self) -> Option<&mut ParenExpr<'a>> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_opt_chain(&mut self) -> Option<&mut OptChainExpr<'a>> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_invalid(&mut self) -> Option<&mut Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for SimpleAssignTarget<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Ident(it) => it.span(),
            Self::Member(it) => it.span(),
            Self::SuperProp(it) => it.span(),
            Self::Paren(it) => it.span(),
            Self::OptChain(it) => it.span(),
            Self::Invalid(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for SimpleAssignTarget<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Ident(it) => it.set_span(span),
            Self::Member(it) => it.set_span(span),
            Self::SuperProp(it) => it.set_span(span),
            Self::Paren(it) => it.set_span(span),
            Self::OptChain(it) => it.set_span(span),
            Self::Invalid(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for OptChainExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for OptChainExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> OptChainBase<'a> {
    #[inline]
    pub const fn is_member(&self) -> bool {
        matches!(self, Self::Member { .. })
    }
    #[inline]
    pub const fn is_call(&self) -> bool {
        matches!(self, Self::Call { .. })
    }
    #[inline]
    pub fn as_member(&self) -> Option<&MemberExpr<'a>> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(&self) -> Option<&OptCall<'a>> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_member(&mut self) -> Option<&mut MemberExpr<'a>> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_call(&mut self) -> Option<&mut OptCall<'a>> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for OptChainBase<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Member(it) => it.span(),
            Self::Call(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for OptChainBase<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Member(it) => it.set_span(span),
            Self::Call(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for OptCall<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for OptCall<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for Invalid {
    #[inline]
    fn span(&self) -> Span {
        DUMMY_SP
    }
}
impl SetSpan for Invalid {
    #[inline]
    fn set_span(&mut self, span: Span) {}
}
impl<'a> GetSpan for Function<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Function<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for Param<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Param<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> ParamOrTsParamProp<'a> {
    #[inline]
    pub const fn is_param(&self) -> bool {
        matches!(self, Self::Param { .. })
    }
    #[inline]
    pub fn as_param(&self) -> Option<&Param<'a>> {
        match self {
            Self::Param(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_param(&mut self) -> Option<&mut Param<'a>> {
        match self {
            Self::Param(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for ParamOrTsParamProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Param(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for ParamOrTsParamProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Param(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for Class<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Class<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> ClassMember<'a> {
    #[inline]
    pub const fn is_constructor(&self) -> bool {
        matches!(self, Self::Constructor { .. })
    }
    #[inline]
    pub const fn is_method(&self) -> bool {
        matches!(self, Self::Method { .. })
    }
    #[inline]
    pub const fn is_private_method(&self) -> bool {
        matches!(self, Self::PrivateMethod { .. })
    }
    #[inline]
    pub const fn is_class_prop(&self) -> bool {
        matches!(self, Self::ClassProp { .. })
    }
    #[inline]
    pub const fn is_private_prop(&self) -> bool {
        matches!(self, Self::PrivateProp { .. })
    }
    #[inline]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty { .. })
    }
    #[inline]
    pub const fn is_static_block(&self) -> bool {
        matches!(self, Self::StaticBlock { .. })
    }
    #[inline]
    pub const fn is_auto_accessor(&self) -> bool {
        matches!(self, Self::AutoAccessor { .. })
    }
    #[inline]
    pub fn as_constructor(&self) -> Option<&Constructor<'a>> {
        match self {
            Self::Constructor(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_method(&self) -> Option<&ClassMethod<'a>> {
        match self {
            Self::Method(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_method(&self) -> Option<&PrivateMethod<'a>> {
        match self {
            Self::PrivateMethod(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class_prop(&self) -> Option<&ClassProp<'a>> {
        match self {
            Self::ClassProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_prop(&self) -> Option<&PrivateProp<'a>> {
        match self {
            Self::PrivateProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_empty(&self) -> Option<&EmptyStmt> {
        match self {
            Self::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_static_block(&self) -> Option<&StaticBlock<'a>> {
        match self {
            Self::StaticBlock(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_auto_accessor(&self) -> Option<&AutoAccessor<'a>> {
        match self {
            Self::AutoAccessor(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_constructor(&mut self) -> Option<&mut Constructor<'a>> {
        match self {
            Self::Constructor(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_method(&mut self) -> Option<&mut ClassMethod<'a>> {
        match self {
            Self::Method(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_private_method(&mut self) -> Option<&mut PrivateMethod<'a>> {
        match self {
            Self::PrivateMethod(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_class_prop(&mut self) -> Option<&mut ClassProp<'a>> {
        match self {
            Self::ClassProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_private_prop(&mut self) -> Option<&mut PrivateProp<'a>> {
        match self {
            Self::PrivateProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_empty(&mut self) -> Option<&mut EmptyStmt> {
        match self {
            Self::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_static_block(&mut self) -> Option<&mut StaticBlock<'a>> {
        match self {
            Self::StaticBlock(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_auto_accessor(&mut self) -> Option<&mut AutoAccessor<'a>> {
        match self {
            Self::AutoAccessor(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for ClassMember<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Constructor(it) => it.span(),
            Self::Method(it) => it.span(),
            Self::PrivateMethod(it) => it.span(),
            Self::ClassProp(it) => it.span(),
            Self::PrivateProp(it) => it.span(),
            Self::Empty(it) => it.span(),
            Self::StaticBlock(it) => it.span(),
            Self::AutoAccessor(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for ClassMember<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Constructor(it) => it.set_span(span),
            Self::Method(it) => it.set_span(span),
            Self::PrivateMethod(it) => it.set_span(span),
            Self::ClassProp(it) => it.set_span(span),
            Self::PrivateProp(it) => it.set_span(span),
            Self::Empty(it) => it.set_span(span),
            Self::StaticBlock(it) => it.set_span(span),
            Self::AutoAccessor(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for ClassProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ClassProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for PrivateProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for PrivateProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ClassMethod<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ClassMethod<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for PrivateMethod<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for PrivateMethod<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for Constructor<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Constructor<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for Decorator<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Decorator<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for StaticBlock<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for StaticBlock<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> Key<'a> {
    #[inline]
    pub const fn is_private(&self) -> bool {
        matches!(self, Self::Private { .. })
    }
    #[inline]
    pub const fn is_public(&self) -> bool {
        matches!(self, Self::Public { .. })
    }
    #[inline]
    pub fn as_private(&self) -> Option<&PrivateName<'a>> {
        match self {
            Self::Private(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_public(&self) -> Option<&PropName<'a>> {
        match self {
            Self::Public(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_private(&mut self) -> Option<&mut PrivateName<'a>> {
        match self {
            Self::Private(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_public(&mut self) -> Option<&mut PropName<'a>> {
        match self {
            Self::Public(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for Key<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Private(it) => it.span(),
            Self::Public(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for Key<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Private(it) => it.set_span(span),
            Self::Public(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for AutoAccessor<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for AutoAccessor<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> Prop<'a> {
    #[inline]
    pub const fn is_shorthand(&self) -> bool {
        matches!(self, Self::Shorthand { .. })
    }
    #[inline]
    pub const fn is_key_value(&self) -> bool {
        matches!(self, Self::KeyValue { .. })
    }
    #[inline]
    pub const fn is_assign(&self) -> bool {
        matches!(self, Self::Assign { .. })
    }
    #[inline]
    pub const fn is_getter(&self) -> bool {
        matches!(self, Self::Getter { .. })
    }
    #[inline]
    pub const fn is_setter(&self) -> bool {
        matches!(self, Self::Setter { .. })
    }
    #[inline]
    pub const fn is_method(&self) -> bool {
        matches!(self, Self::Method { .. })
    }
    #[inline]
    pub fn as_shorthand(&self) -> Option<&Ident<'a>> {
        match self {
            Self::Shorthand(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_key_value(&self) -> Option<&KeyValueProp<'a>> {
        match self {
            Self::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(&self) -> Option<&AssignProp<'a>> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_getter(&self) -> Option<&GetterProp<'a>> {
        match self {
            Self::Getter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_setter(&self) -> Option<&SetterProp<'a>> {
        match self {
            Self::Setter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_method(&self) -> Option<&MethodProp<'a>> {
        match self {
            Self::Method(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_shorthand(&mut self) -> Option<&mut Ident<'a>> {
        match self {
            Self::Shorthand(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_key_value(&mut self) -> Option<&mut KeyValueProp<'a>> {
        match self {
            Self::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_assign(&mut self) -> Option<&mut AssignProp<'a>> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_getter(&mut self) -> Option<&mut GetterProp<'a>> {
        match self {
            Self::Getter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_setter(&mut self) -> Option<&mut SetterProp<'a>> {
        match self {
            Self::Setter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_method(&mut self) -> Option<&mut MethodProp<'a>> {
        match self {
            Self::Method(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for Prop<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Shorthand(it) => it.span(),
            Self::KeyValue(it) => it.span(),
            Self::Assign(it) => it.span(),
            Self::Getter(it) => it.span(),
            Self::Setter(it) => it.span(),
            Self::Method(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for Prop<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Shorthand(it) => it.set_span(span),
            Self::KeyValue(it) => it.set_span(span),
            Self::Assign(it) => it.set_span(span),
            Self::Getter(it) => it.set_span(span),
            Self::Setter(it) => it.set_span(span),
            Self::Method(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for KeyValueProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.key.span_lo(), self.value.span_hi())
    }
}
impl<'a> SetSpan for KeyValueProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        let current = self.key.span();
        self.key.set_span(Span::new(span.start, current.end));
        let current = self.value.span();
        self.value.set_span(Span::new(current.start, span.end));
    }
}
impl<'a> GetSpan for AssignProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for AssignProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for GetterProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for GetterProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for SetterProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for SetterProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for MethodProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.function.span()
    }
}
impl<'a> SetSpan for MethodProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.function.set_span(span);
    }
}
impl<'a> PropName<'a> {
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub const fn is_str(&self) -> bool {
        matches!(self, Self::Str { .. })
    }
    #[inline]
    pub const fn is_num(&self) -> bool {
        matches!(self, Self::Num { .. })
    }
    #[inline]
    pub const fn is_computed(&self) -> bool {
        matches!(self, Self::Computed { .. })
    }
    #[inline]
    pub const fn is_big_int(&self) -> bool {
        matches!(self, Self::BigInt { .. })
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&IdentName<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(&self) -> Option<&Str<'a>> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(&self) -> Option<&Number<'a>> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(&self) -> Option<&ComputedPropName<'a>> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(&self) -> Option<&BigInt<'a>> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut IdentName<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_str(&mut self) -> Option<&mut Str<'a>> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_num(&mut self) -> Option<&mut Number<'a>> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_computed(&mut self) -> Option<&mut ComputedPropName<'a>> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_big_int(&mut self) -> Option<&mut BigInt<'a>> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for PropName<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Ident(it) => it.span(),
            Self::Str(it) => it.span(),
            Self::Num(it) => it.span(),
            Self::Computed(it) => it.span(),
            Self::BigInt(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for PropName<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Ident(it) => it.set_span(span),
            Self::Str(it) => it.set_span(span),
            Self::Num(it) => it.set_span(span),
            Self::Computed(it) => it.set_span(span),
            Self::BigInt(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for ComputedPropName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ComputedPropName<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> Pat<'a> {
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array { .. })
    }
    #[inline]
    pub const fn is_rest(&self) -> bool {
        matches!(self, Self::Rest { .. })
    }
    #[inline]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object { .. })
    }
    #[inline]
    pub const fn is_assign(&self) -> bool {
        matches!(self, Self::Assign { .. })
    }
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid { .. })
    }
    #[inline]
    pub const fn is_expr(&self) -> bool {
        matches!(self, Self::Expr { .. })
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&BindingIdent<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(&self) -> Option<&ArrayPat<'a>> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_rest(&self) -> Option<&RestPat<'a>> {
        match self {
            Self::Rest(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(&self) -> Option<&ObjectPat<'a>> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(&self) -> Option<&AssignPat<'a>> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(&self) -> Option<&Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut BindingIdent<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_array(&mut self) -> Option<&mut ArrayPat<'a>> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_rest(&mut self) -> Option<&mut RestPat<'a>> {
        match self {
            Self::Rest(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_object(&mut self) -> Option<&mut ObjectPat<'a>> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_assign(&mut self) -> Option<&mut AssignPat<'a>> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_invalid(&mut self) -> Option<&mut Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_expr(&mut self) -> Option<&mut Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for Pat<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Ident(it) => it.span(),
            Self::Array(it) => it.span(),
            Self::Rest(it) => it.span(),
            Self::Object(it) => it.span(),
            Self::Assign(it) => it.span(),
            Self::Invalid(it) => it.span(),
            Self::Expr(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for Pat<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Ident(it) => it.set_span(span),
            Self::Array(it) => it.set_span(span),
            Self::Rest(it) => it.set_span(span),
            Self::Object(it) => it.set_span(span),
            Self::Assign(it) => it.set_span(span),
            Self::Invalid(it) => it.set_span(span),
            Self::Expr(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for ArrayPat<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ArrayPat<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for ObjectPat<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for ObjectPat<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for AssignPat<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for AssignPat<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for RestPat<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for RestPat<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> ObjectPatProp<'a> {
    #[inline]
    pub const fn is_key_value(&self) -> bool {
        matches!(self, Self::KeyValue { .. })
    }
    #[inline]
    pub const fn is_assign(&self) -> bool {
        matches!(self, Self::Assign { .. })
    }
    #[inline]
    pub const fn is_rest(&self) -> bool {
        matches!(self, Self::Rest { .. })
    }
    #[inline]
    pub fn as_key_value(&self) -> Option<&KeyValuePatProp<'a>> {
        match self {
            Self::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(&self) -> Option<&AssignPatProp<'a>> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_rest(&self) -> Option<&RestPat<'a>> {
        match self {
            Self::Rest(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_key_value(&mut self) -> Option<&mut KeyValuePatProp<'a>> {
        match self {
            Self::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_assign(&mut self) -> Option<&mut AssignPatProp<'a>> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_rest(&mut self) -> Option<&mut RestPat<'a>> {
        match self {
            Self::Rest(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for ObjectPatProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::KeyValue(it) => it.span(),
            Self::Assign(it) => it.span(),
            Self::Rest(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for ObjectPatProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::KeyValue(it) => it.set_span(span),
            Self::Assign(it) => it.set_span(span),
            Self::Rest(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for KeyValuePatProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.key.span_lo(), self.value.span_hi())
    }
}
impl<'a> SetSpan for KeyValuePatProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        let current = self.key.span();
        self.key.set_span(Span::new(span.start, current.end));
        let current = self.value.span();
        self.value.set_span(Span::new(current.start, span.end));
    }
}
impl<'a> GetSpan for AssignPatProp<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for AssignPatProp<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for Ident<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Ident<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for IdentName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for IdentName<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for PrivateName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for PrivateName<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> Lit<'a> {
    #[inline]
    pub const fn is_str(&self) -> bool {
        matches!(self, Self::Str { .. })
    }
    #[inline]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool { .. })
    }
    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null { .. })
    }
    #[inline]
    pub const fn is_num(&self) -> bool {
        matches!(self, Self::Num { .. })
    }
    #[inline]
    pub const fn is_big_int(&self) -> bool {
        matches!(self, Self::BigInt { .. })
    }
    #[inline]
    pub const fn is_regex(&self) -> bool {
        matches!(self, Self::Regex { .. })
    }
    #[inline]
    pub fn as_str(&self) -> Option<&Str<'a>> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(&self) -> Option<&Bool> {
        match self {
            Self::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null(&self) -> Option<&Null> {
        match self {
            Self::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(&self) -> Option<&Number<'a>> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(&self) -> Option<&BigInt<'a>> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_regex(&self) -> Option<&Regex<'a>> {
        match self {
            Self::Regex(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_str(&mut self) -> Option<&mut Str<'a>> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_bool(&mut self) -> Option<&mut Bool> {
        match self {
            Self::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_null(&mut self) -> Option<&mut Null> {
        match self {
            Self::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_num(&mut self) -> Option<&mut Number<'a>> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_big_int(&mut self) -> Option<&mut BigInt<'a>> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_regex(&mut self) -> Option<&mut Regex<'a>> {
        match self {
            Self::Regex(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for Lit<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Str(it) => it.span(),
            Self::Bool(it) => it.span(),
            Self::Null(it) => it.span(),
            Self::Num(it) => it.span(),
            Self::BigInt(it) => it.span(),
            Self::Regex(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for Lit<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Str(it) => it.set_span(span),
            Self::Bool(it) => it.set_span(span),
            Self::Null(it) => it.set_span(span),
            Self::Num(it) => it.set_span(span),
            Self::BigInt(it) => it.set_span(span),
            Self::Regex(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for Str<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Str<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for Bool {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for Bool {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for Null {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for Null {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for Number<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Number<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for BigInt<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for BigInt<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for Regex<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for Regex<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> JSXObject<'a> {
    #[inline]
    pub const fn is_jsx_member_expr(&self) -> bool {
        matches!(self, Self::JSXMemberExpr { .. })
    }
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub fn as_jsx_member_expr(&self) -> Option<&JSXMemberExpr<'a>> {
        match self {
            Self::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&Ident<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_member_expr(&mut self) -> Option<&mut JSXMemberExpr<'a>> {
        match self {
            Self::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut Ident<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for JSXObject<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::JSXMemberExpr(it) => it.span(),
            Self::Ident(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for JSXObject<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::JSXMemberExpr(it) => it.set_span(span),
            Self::Ident(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for JSXMemberExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXMemberExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for JSXNamespacedName<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXNamespacedName<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for JSXEmptyExpr {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for JSXEmptyExpr {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for JSXExprContainer<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXExprContainer<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> JSXExpr<'a> {
    #[inline]
    pub const fn is_jsx_empty_expr(&self) -> bool {
        matches!(self, Self::JSXEmptyExpr { .. })
    }
    #[inline]
    pub const fn is_expr(&self) -> bool {
        matches!(self, Self::Expr { .. })
    }
    #[inline]
    pub fn as_jsx_empty_expr(&self) -> Option<&JSXEmptyExpr> {
        match self {
            Self::JSXEmptyExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_empty_expr(&mut self) -> Option<&mut JSXEmptyExpr> {
        match self {
            Self::JSXEmptyExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_expr(&mut self) -> Option<&mut Expr<'a>> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for JSXExpr<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::JSXEmptyExpr(it) => it.span(),
            Self::Expr(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for JSXExpr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::JSXEmptyExpr(it) => it.set_span(span),
            Self::Expr(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for JSXSpreadChild<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXSpreadChild<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> JSXElementName<'a> {
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub const fn is_jsx_member_expr(&self) -> bool {
        matches!(self, Self::JSXMemberExpr { .. })
    }
    #[inline]
    pub const fn is_jsx_namespaced_name(&self) -> bool {
        matches!(self, Self::JSXNamespacedName { .. })
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&Ident<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member_expr(&self) -> Option<&JSXMemberExpr<'a>> {
        match self {
            Self::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(&self) -> Option<&JSXNamespacedName<'a>> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut Ident<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_member_expr(&mut self) -> Option<&mut JSXMemberExpr<'a>> {
        match self {
            Self::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_namespaced_name(&mut self) -> Option<&mut JSXNamespacedName<'a>> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for JSXElementName<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Ident(it) => it.span(),
            Self::JSXMemberExpr(it) => it.span(),
            Self::JSXNamespacedName(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for JSXElementName<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Ident(it) => it.set_span(span),
            Self::JSXMemberExpr(it) => it.set_span(span),
            Self::JSXNamespacedName(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for JSXOpeningElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXOpeningElement<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> JSXAttrOrSpread<'a> {
    #[inline]
    pub const fn is_jsx_attr(&self) -> bool {
        matches!(self, Self::JSXAttr { .. })
    }
    #[inline]
    pub const fn is_spread_element(&self) -> bool {
        matches!(self, Self::SpreadElement { .. })
    }
    #[inline]
    pub fn as_jsx_attr(&self) -> Option<&JSXAttr<'a>> {
        match self {
            Self::JSXAttr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_spread_element(&self) -> Option<&SpreadElement<'a>> {
        match self {
            Self::SpreadElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_attr(&mut self) -> Option<&mut JSXAttr<'a>> {
        match self {
            Self::JSXAttr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_spread_element(&mut self) -> Option<&mut SpreadElement<'a>> {
        match self {
            Self::SpreadElement(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for JSXAttrOrSpread<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::JSXAttr(it) => it.span(),
            Self::SpreadElement(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for JSXAttrOrSpread<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::JSXAttr(it) => it.set_span(span),
            Self::SpreadElement(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for JSXClosingElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXClosingElement<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for JSXAttr<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXAttr<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> JSXAttrName<'a> {
    #[inline]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident { .. })
    }
    #[inline]
    pub const fn is_jsx_namespaced_name(&self) -> bool {
        matches!(self, Self::JSXNamespacedName { .. })
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&IdentName<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(&self) -> Option<&JSXNamespacedName<'a>> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_ident(&mut self) -> Option<&mut IdentName<'a>> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_namespaced_name(&mut self) -> Option<&mut JSXNamespacedName<'a>> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for JSXAttrName<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Ident(it) => it.span(),
            Self::JSXNamespacedName(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for JSXAttrName<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Ident(it) => it.set_span(span),
            Self::JSXNamespacedName(it) => it.set_span(span),
        }
    }
}
impl<'a> JSXAttrValue<'a> {
    #[inline]
    pub const fn is_str(&self) -> bool {
        matches!(self, Self::Str { .. })
    }
    #[inline]
    pub const fn is_jsx_expr_container(&self) -> bool {
        matches!(self, Self::JSXExprContainer { .. })
    }
    #[inline]
    pub const fn is_jsx_element(&self) -> bool {
        matches!(self, Self::JSXElement { .. })
    }
    #[inline]
    pub const fn is_jsx_fragment(&self) -> bool {
        matches!(self, Self::JSXFragment { .. })
    }
    #[inline]
    pub fn as_str(&self) -> Option<&Str<'a>> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expr_container(&self) -> Option<&JSXExprContainer<'a>> {
        match self {
            Self::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(&self) -> Option<&JSXElement<'a>> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(&self) -> Option<&JSXFragment<'a>> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_str(&mut self) -> Option<&mut Str<'a>> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_expr_container(&mut self) -> Option<&mut JSXExprContainer<'a>> {
        match self {
            Self::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_element(&mut self) -> Option<&mut JSXElement<'a>> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_fragment(&mut self) -> Option<&mut JSXFragment<'a>> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for JSXAttrValue<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Str(it) => it.span(),
            Self::JSXExprContainer(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for JSXAttrValue<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::Str(it) => it.set_span(span),
            Self::JSXExprContainer(it) => it.set_span(span),
            Self::JSXElement(it) => it.set_span(span),
            Self::JSXFragment(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for JSXText<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXText<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> GetSpan for JSXElement<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXElement<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl<'a> JSXElementChild<'a> {
    #[inline]
    pub const fn is_jsx_text(&self) -> bool {
        matches!(self, Self::JSXText { .. })
    }
    #[inline]
    pub const fn is_jsx_expr_container(&self) -> bool {
        matches!(self, Self::JSXExprContainer { .. })
    }
    #[inline]
    pub const fn is_jsx_spread_child(&self) -> bool {
        matches!(self, Self::JSXSpreadChild { .. })
    }
    #[inline]
    pub const fn is_jsx_element(&self) -> bool {
        matches!(self, Self::JSXElement { .. })
    }
    #[inline]
    pub const fn is_jsx_fragment(&self) -> bool {
        matches!(self, Self::JSXFragment { .. })
    }
    #[inline]
    pub fn as_jsx_text(&self) -> Option<&JSXText<'a>> {
        match self {
            Self::JSXText(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expr_container(&self) -> Option<&JSXExprContainer<'a>> {
        match self {
            Self::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_spread_child(&self) -> Option<&JSXSpreadChild<'a>> {
        match self {
            Self::JSXSpreadChild(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(&self) -> Option<&JSXElement<'a>> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(&self) -> Option<&JSXFragment<'a>> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_text(&mut self) -> Option<&mut JSXText<'a>> {
        match self {
            Self::JSXText(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_expr_container(&mut self) -> Option<&mut JSXExprContainer<'a>> {
        match self {
            Self::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_spread_child(&mut self) -> Option<&mut JSXSpreadChild<'a>> {
        match self {
            Self::JSXSpreadChild(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_element(&mut self) -> Option<&mut JSXElement<'a>> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_mut_jsx_fragment(&mut self) -> Option<&mut JSXFragment<'a>> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
}
impl<'a> GetSpan for JSXElementChild<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::JSXText(it) => it.span(),
            Self::JSXExprContainer(it) => it.span(),
            Self::JSXSpreadChild(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
        }
    }
}
impl<'a> SetSpan for JSXElementChild<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        match self {
            Self::JSXText(it) => it.set_span(span),
            Self::JSXExprContainer(it) => it.set_span(span),
            Self::JSXSpreadChild(it) => it.set_span(span),
            Self::JSXElement(it) => it.set_span(span),
            Self::JSXFragment(it) => it.set_span(span),
        }
    }
}
impl<'a> GetSpan for JSXFragment<'a> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl<'a> SetSpan for JSXFragment<'a> {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for JSXOpeningFragment {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for JSXOpeningFragment {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
impl GetSpan for JSXClosingFragment {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
impl SetSpan for JSXClosingFragment {
    #[inline]
    fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
