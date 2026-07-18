#![allow(unused)]
use crate::{Ast, NodeKind};
use crate::{ast::*, node_id::*};
use oxc_index::Idx;
impl NodeIdTrait for Program {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Module(it) => it.node_id(),
            Self::Script(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::Module => {
                Program::Module({ unsafe { Module::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Script => {
                Program::Script({ unsafe { Script::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for Module {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Script {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ModuleItem {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::ModuleDecl(it) => it.node_id(),
            Self::Stmt(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::BlockStmt => ModuleItem::Stmt(Stmt::Block({
                unsafe { BlockStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BreakStmt => ModuleItem::Stmt(Stmt::Break({
                unsafe { BreakStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ClassDecl => ModuleItem::Stmt(Stmt::Decl(Decl::Class({
                unsafe { ClassDecl::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ContinueStmt => ModuleItem::Stmt(Stmt::Continue({
                unsafe { ContinueStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::DebuggerStmt => ModuleItem::Stmt(Stmt::Debugger({
                unsafe { DebuggerStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::DoWhileStmt => ModuleItem::Stmt(Stmt::DoWhile({
                unsafe { DoWhileStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::EmptyStmt => ModuleItem::Stmt(Stmt::Empty({
                unsafe { EmptyStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ExportAll => ModuleItem::ModuleDecl(ModuleDecl::ExportAll({
                unsafe { ExportAll::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ExportDecl => ModuleItem::ModuleDecl(ModuleDecl::ExportDecl({
                unsafe { ExportDecl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ExportDefaultDecl => ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl({
                unsafe { ExportDefaultDecl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ExportDefaultExpr => ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr({
                unsafe { ExportDefaultExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ExprStmt => ModuleItem::Stmt(Stmt::Expr({
                unsafe { ExprStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::FnDecl => ModuleItem::Stmt(Stmt::Decl(Decl::Fn({
                unsafe { FnDecl::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ForInStmt => ModuleItem::Stmt(Stmt::ForIn({
                unsafe { ForInStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ForOfStmt => ModuleItem::Stmt(Stmt::ForOf({
                unsafe { ForOfStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ForStmt => ModuleItem::Stmt(Stmt::For({
                unsafe { ForStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::IfStmt => ModuleItem::Stmt(Stmt::If({
                unsafe { IfStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ImportDecl => ModuleItem::ModuleDecl(ModuleDecl::Import({
                unsafe { ImportDecl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::LabeledStmt => ModuleItem::Stmt(Stmt::Labeled({
                unsafe { LabeledStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::NamedExport => ModuleItem::ModuleDecl(ModuleDecl::ExportNamed({
                unsafe { NamedExport::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ReturnStmt => ModuleItem::Stmt(Stmt::Return({
                unsafe { ReturnStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::SwitchStmt => ModuleItem::Stmt(Stmt::Switch({
                unsafe { SwitchStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ThrowStmt => ModuleItem::Stmt(Stmt::Throw({
                unsafe { ThrowStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::TryStmt => ModuleItem::Stmt(Stmt::Try({
                unsafe { TryStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UsingDecl => ModuleItem::Stmt(Stmt::Decl(Decl::Using({
                unsafe { UsingDecl::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::VarDecl => ModuleItem::Stmt(Stmt::Decl(Decl::Var({
                unsafe { VarDecl::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::WhileStmt => ModuleItem::Stmt(Stmt::While({
                unsafe { WhileStmt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::WithStmt => ModuleItem::Stmt(Stmt::With({
                unsafe { WithStmt::from_node_id_unchecked(id, ast) }
            })),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ModuleDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Import(it) => it.node_id(),
            Self::ExportDecl(it) => it.node_id(),
            Self::ExportNamed(it) => it.node_id(),
            Self::ExportDefaultDecl(it) => it.node_id(),
            Self::ExportDefaultExpr(it) => it.node_id(),
            Self::ExportAll(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ExportAll => {
                ModuleDecl::ExportAll({ unsafe { ExportAll::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ExportDecl => {
                ModuleDecl::ExportDecl({ unsafe { ExportDecl::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ExportDefaultDecl => ModuleDecl::ExportDefaultDecl({
                unsafe { ExportDefaultDecl::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::ExportDefaultExpr => ModuleDecl::ExportDefaultExpr({
                unsafe { ExportDefaultExpr::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::ImportDecl => {
                ModuleDecl::Import({ unsafe { ImportDecl::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::NamedExport => {
                ModuleDecl::ExportNamed({ unsafe { NamedExport::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ImportDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ImportSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Named(it) => it.node_id(),
            Self::Default(it) => it.node_id(),
            Self::Namespace(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ImportDefaultSpecifier => ImportSpecifier::Default({
                unsafe { ImportDefaultSpecifier::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::ImportNamedSpecifier => ImportSpecifier::Named({
                unsafe { ImportNamedSpecifier::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::ImportStarAsSpecifier => ImportSpecifier::Namespace({
                unsafe { ImportStarAsSpecifier::from_node_id_unchecked(id, ast) }
            }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ImportNamedSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ImportDefaultSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ImportStarAsSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for NamedExport {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Namespace(it) => it.node_id(),
            Self::Default(it) => it.node_id(),
            Self::Named(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ExportDefaultSpecifier => ExportSpecifier::Default({
                unsafe { ExportDefaultSpecifier::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::ExportNamedSpecifier => ExportSpecifier::Named({
                unsafe { ExportNamedSpecifier::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::ExportNamespaceSpecifier => ExportSpecifier::Namespace({
                unsafe { ExportNamespaceSpecifier::from_node_id_unchecked(id, ast) }
            }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ExportNamespaceSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ModuleExportName {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::Ident => {
                ModuleExportName::Ident({ unsafe { Ident::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Str => {
                ModuleExportName::Str({ unsafe { Str::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ExportDefaultSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportNamedSpecifier {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportDefaultDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for DefaultDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Class(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ClassExpr => {
                DefaultDecl::Class({ unsafe { ClassExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::FnExpr => {
                DefaultDecl::Fn({ unsafe { FnExpr::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ExportDefaultExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportAll {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BlockStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Stmt {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
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
            Self::Decl(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::BlockStmt => {
                Stmt::Block({ unsafe { BlockStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::BreakStmt => {
                Stmt::Break({ unsafe { BreakStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ClassDecl => Stmt::Decl(Decl::Class({
                unsafe { ClassDecl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ContinueStmt => {
                Stmt::Continue({ unsafe { ContinueStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::DebuggerStmt => {
                Stmt::Debugger({ unsafe { DebuggerStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::DoWhileStmt => {
                Stmt::DoWhile({ unsafe { DoWhileStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::EmptyStmt => {
                Stmt::Empty({ unsafe { EmptyStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ExprStmt => {
                Stmt::Expr({ unsafe { ExprStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::FnDecl => Stmt::Decl(Decl::Fn({
                unsafe { FnDecl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ForInStmt => {
                Stmt::ForIn({ unsafe { ForInStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ForOfStmt => {
                Stmt::ForOf({ unsafe { ForOfStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ForStmt => Stmt::For({ unsafe { ForStmt::from_node_id_unchecked(id, ast) } }),
            NodeKind::IfStmt => Stmt::If({ unsafe { IfStmt::from_node_id_unchecked(id, ast) } }),
            NodeKind::LabeledStmt => {
                Stmt::Labeled({ unsafe { LabeledStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ReturnStmt => {
                Stmt::Return({ unsafe { ReturnStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::SwitchStmt => {
                Stmt::Switch({ unsafe { SwitchStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ThrowStmt => {
                Stmt::Throw({ unsafe { ThrowStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::TryStmt => Stmt::Try({ unsafe { TryStmt::from_node_id_unchecked(id, ast) } }),
            NodeKind::UsingDecl => Stmt::Decl(Decl::Using({
                unsafe { UsingDecl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::VarDecl => Stmt::Decl(Decl::Var({
                unsafe { VarDecl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::WhileStmt => {
                Stmt::While({ unsafe { WhileStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::WithStmt => {
                Stmt::With({ unsafe { WithStmt::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ExprStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for EmptyStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for DebuggerStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for WithStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ReturnStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for LabeledStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BreakStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ContinueStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for IfStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SwitchStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ThrowStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for TryStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for WhileStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for DoWhileStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ForStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ForInStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ForOfStmt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SwitchCase {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for CatchClause {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ForHead {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::VarDecl(it) => it.node_id(),
            Self::UsingDecl(it) => it.node_id(),
            Self::Pat(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ArrayLit => ForHead::Pat(Pat::Expr(Expr::Array({
                unsafe { ArrayLit::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ArrayPat => ForHead::Pat(Pat::Array({
                unsafe { ArrayPat::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ArrowExpr => ForHead::Pat(Pat::Expr(Expr::Arrow({
                unsafe { ArrowExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::AssignExpr => ForHead::Pat(Pat::Expr(Expr::Assign({
                unsafe { AssignExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::AssignPat => ForHead::Pat(Pat::Assign({
                unsafe { AssignPat::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AwaitExpr => ForHead::Pat(Pat::Expr(Expr::Await({
                unsafe { AwaitExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::BigInt => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::BigInt({
                unsafe { BigInt::from_node_id_unchecked(id, ast) }
            })))),
            NodeKind::BinExpr => ForHead::Pat(Pat::Expr(Expr::Bin({
                unsafe { BinExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::BindingIdent => ForHead::Pat(Pat::Ident({
                unsafe { BindingIdent::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Bool => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Bool({
                unsafe { Bool::from_node_id_unchecked(id, ast) }
            })))),
            NodeKind::CallExpr => ForHead::Pat(Pat::Expr(Expr::Call({
                unsafe { CallExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ClassExpr => ForHead::Pat(Pat::Expr(Expr::Class({
                unsafe { ClassExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::CondExpr => ForHead::Pat(Pat::Expr(Expr::Cond({
                unsafe { CondExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::FnExpr => ForHead::Pat(Pat::Expr(Expr::Fn({
                unsafe { FnExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Ident => ForHead::Pat(Pat::Expr(Expr::Ident({
                unsafe { Ident::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ImportExpr => ForHead::Pat(Pat::Expr(Expr::Import({
                unsafe { ImportExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Invalid => ForHead::Pat(Pat::Invalid({
                unsafe { Invalid::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXElement => ForHead::Pat(Pat::Expr(Expr::JSXElement({
                unsafe { JSXElement::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::JSXEmptyExpr => ForHead::Pat(Pat::Expr(Expr::JSXEmpty({
                unsafe { JSXEmptyExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::JSXFragment => ForHead::Pat(Pat::Expr(Expr::JSXFragment({
                unsafe { JSXFragment::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::JSXMemberExpr => ForHead::Pat(Pat::Expr(Expr::JSXMember({
                unsafe { JSXMemberExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::JSXNamespacedName => ForHead::Pat(Pat::Expr(Expr::JSXNamespacedName({
                unsafe { JSXNamespacedName::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::MemberExpr => ForHead::Pat(Pat::Expr(Expr::Member({
                unsafe { MemberExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::MetaPropExpr => ForHead::Pat(Pat::Expr(Expr::MetaProp({
                unsafe { MetaPropExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::NewExpr => ForHead::Pat(Pat::Expr(Expr::New({
                unsafe { NewExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Null => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Null({
                unsafe { Null::from_node_id_unchecked(id, ast) }
            })))),
            NodeKind::Number => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Num({
                unsafe { Number::from_node_id_unchecked(id, ast) }
            })))),
            NodeKind::ObjectLit => ForHead::Pat(Pat::Expr(Expr::Object({
                unsafe { ObjectLit::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ObjectPat => ForHead::Pat(Pat::Object({
                unsafe { ObjectPat::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::OptChainExpr => ForHead::Pat(Pat::Expr(Expr::OptChain({
                unsafe { OptChainExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ParenExpr => ForHead::Pat(Pat::Expr(Expr::Paren({
                unsafe { ParenExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::PrivateName => ForHead::Pat(Pat::Expr(Expr::PrivateName({
                unsafe { PrivateName::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Regex => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Regex({
                unsafe { Regex::from_node_id_unchecked(id, ast) }
            })))),
            NodeKind::RestPat => ForHead::Pat(Pat::Rest({
                unsafe { RestPat::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::SeqExpr => ForHead::Pat(Pat::Expr(Expr::Seq({
                unsafe { SeqExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Str => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Str({
                unsafe { Str::from_node_id_unchecked(id, ast) }
            })))),
            NodeKind::SuperPropExpr => ForHead::Pat(Pat::Expr(Expr::SuperProp({
                unsafe { SuperPropExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::TaggedTpl => ForHead::Pat(Pat::Expr(Expr::TaggedTpl({
                unsafe { TaggedTpl::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ThisExpr => ForHead::Pat(Pat::Expr(Expr::This({
                unsafe { ThisExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Tpl => ForHead::Pat(Pat::Expr(Expr::Tpl({
                unsafe { Tpl::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::UnaryExpr => ForHead::Pat(Pat::Expr(Expr::Unary({
                unsafe { UnaryExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::UpdateExpr => ForHead::Pat(Pat::Expr(Expr::Update({
                unsafe { UpdateExpr::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::UsingDecl => {
                ForHead::UsingDecl({ unsafe { UsingDecl::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::VarDecl => {
                ForHead::VarDecl({ unsafe { VarDecl::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::YieldExpr => ForHead::Pat(Pat::Expr(Expr::Yield({
                unsafe { YieldExpr::from_node_id_unchecked(id, ast) }
            }))),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for VarDeclOrExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::VarDecl(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ArrayLit => VarDeclOrExpr::Expr(Expr::Array({
                unsafe { ArrayLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ArrowExpr => VarDeclOrExpr::Expr(Expr::Arrow({
                unsafe { ArrowExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AssignExpr => VarDeclOrExpr::Expr(Expr::Assign({
                unsafe { AssignExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AwaitExpr => VarDeclOrExpr::Expr(Expr::Await({
                unsafe { AwaitExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BigInt => VarDeclOrExpr::Expr(Expr::Lit(Lit::BigInt({
                unsafe { BigInt::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::BinExpr => VarDeclOrExpr::Expr(Expr::Bin({
                unsafe { BinExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Bool => VarDeclOrExpr::Expr(Expr::Lit(Lit::Bool({
                unsafe { Bool::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::CallExpr => VarDeclOrExpr::Expr(Expr::Call({
                unsafe { CallExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ClassExpr => VarDeclOrExpr::Expr(Expr::Class({
                unsafe { ClassExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::CondExpr => VarDeclOrExpr::Expr(Expr::Cond({
                unsafe { CondExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::FnExpr => VarDeclOrExpr::Expr(Expr::Fn({
                unsafe { FnExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Ident => VarDeclOrExpr::Expr(Expr::Ident({
                unsafe { Ident::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ImportExpr => VarDeclOrExpr::Expr(Expr::Import({
                unsafe { ImportExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Invalid => VarDeclOrExpr::Expr(Expr::Invalid({
                unsafe { Invalid::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXElement => VarDeclOrExpr::Expr(Expr::JSXElement({
                unsafe { JSXElement::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXEmptyExpr => VarDeclOrExpr::Expr(Expr::JSXEmpty({
                unsafe { JSXEmptyExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXFragment => VarDeclOrExpr::Expr(Expr::JSXFragment({
                unsafe { JSXFragment::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXMemberExpr => VarDeclOrExpr::Expr(Expr::JSXMember({
                unsafe { JSXMemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXNamespacedName => VarDeclOrExpr::Expr(Expr::JSXNamespacedName({
                unsafe { JSXNamespacedName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MemberExpr => VarDeclOrExpr::Expr(Expr::Member({
                unsafe { MemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MetaPropExpr => VarDeclOrExpr::Expr(Expr::MetaProp({
                unsafe { MetaPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::NewExpr => VarDeclOrExpr::Expr(Expr::New({
                unsafe { NewExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Null => VarDeclOrExpr::Expr(Expr::Lit(Lit::Null({
                unsafe { Null::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Number => VarDeclOrExpr::Expr(Expr::Lit(Lit::Num({
                unsafe { Number::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ObjectLit => VarDeclOrExpr::Expr(Expr::Object({
                unsafe { ObjectLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::OptChainExpr => VarDeclOrExpr::Expr(Expr::OptChain({
                unsafe { OptChainExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ParenExpr => VarDeclOrExpr::Expr(Expr::Paren({
                unsafe { ParenExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::PrivateName => VarDeclOrExpr::Expr(Expr::PrivateName({
                unsafe { PrivateName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Regex => VarDeclOrExpr::Expr(Expr::Lit(Lit::Regex({
                unsafe { Regex::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::SeqExpr => VarDeclOrExpr::Expr(Expr::Seq({
                unsafe { SeqExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Str => VarDeclOrExpr::Expr(Expr::Lit(Lit::Str({
                unsafe { Str::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::SuperPropExpr => VarDeclOrExpr::Expr(Expr::SuperProp({
                unsafe { SuperPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::TaggedTpl => VarDeclOrExpr::Expr(Expr::TaggedTpl({
                unsafe { TaggedTpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ThisExpr => VarDeclOrExpr::Expr(Expr::This({
                unsafe { ThisExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Tpl => VarDeclOrExpr::Expr(Expr::Tpl({
                unsafe { Tpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UnaryExpr => VarDeclOrExpr::Expr(Expr::Unary({
                unsafe { UnaryExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UpdateExpr => VarDeclOrExpr::Expr(Expr::Update({
                unsafe { UpdateExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::VarDecl => {
                VarDeclOrExpr::VarDecl({ unsafe { VarDecl::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::YieldExpr => VarDeclOrExpr::Expr(Expr::Yield({
                unsafe { YieldExpr::from_node_id_unchecked(id, ast) }
            })),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for Decl {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Class(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
            Self::Var(it) => it.node_id(),
            Self::Using(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ClassDecl => {
                Decl::Class({ unsafe { ClassDecl::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::FnDecl => Decl::Fn({ unsafe { FnDecl::from_node_id_unchecked(id, ast) } }),
            NodeKind::UsingDecl => {
                Decl::Using({ unsafe { UsingDecl::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::VarDecl => Decl::Var({ unsafe { VarDecl::from_node_id_unchecked(id, ast) } }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for FnDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ClassDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for VarDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for VarDeclarator {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for UsingDecl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Expr {
    #[inline]
    fn node_id(self) -> NodeId {
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
            Self::Lit(it) => it.node_id(),
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
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ArrayLit => {
                Expr::Array({ unsafe { ArrayLit::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ArrowExpr => {
                Expr::Arrow({ unsafe { ArrowExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::AssignExpr => {
                Expr::Assign({ unsafe { AssignExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::AwaitExpr => {
                Expr::Await({ unsafe { AwaitExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::BigInt => Expr::Lit(Lit::BigInt({
                unsafe { BigInt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BinExpr => Expr::Bin({ unsafe { BinExpr::from_node_id_unchecked(id, ast) } }),
            NodeKind::Bool => Expr::Lit(Lit::Bool({
                unsafe { Bool::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::CallExpr => {
                Expr::Call({ unsafe { CallExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ClassExpr => {
                Expr::Class({ unsafe { ClassExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::CondExpr => {
                Expr::Cond({ unsafe { CondExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::FnExpr => Expr::Fn({ unsafe { FnExpr::from_node_id_unchecked(id, ast) } }),
            NodeKind::Ident => Expr::Ident({ unsafe { Ident::from_node_id_unchecked(id, ast) } }),
            NodeKind::ImportExpr => {
                Expr::Import({ unsafe { ImportExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Invalid => {
                Expr::Invalid({ unsafe { Invalid::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXElement => {
                Expr::JSXElement({ unsafe { JSXElement::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXEmptyExpr => {
                Expr::JSXEmpty({ unsafe { JSXEmptyExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXFragment => {
                Expr::JSXFragment({ unsafe { JSXFragment::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXMemberExpr => {
                Expr::JSXMember({ unsafe { JSXMemberExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXNamespacedName => Expr::JSXNamespacedName({
                unsafe { JSXNamespacedName::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::MemberExpr => {
                Expr::Member({ unsafe { MemberExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::MetaPropExpr => {
                Expr::MetaProp({ unsafe { MetaPropExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::NewExpr => Expr::New({ unsafe { NewExpr::from_node_id_unchecked(id, ast) } }),
            NodeKind::Null => Expr::Lit(Lit::Null({
                unsafe { Null::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Number => Expr::Lit(Lit::Num({
                unsafe { Number::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ObjectLit => {
                Expr::Object({ unsafe { ObjectLit::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::OptChainExpr => {
                Expr::OptChain({ unsafe { OptChainExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ParenExpr => {
                Expr::Paren({ unsafe { ParenExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::PrivateName => {
                Expr::PrivateName({ unsafe { PrivateName::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Regex => Expr::Lit(Lit::Regex({
                unsafe { Regex::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::SeqExpr => Expr::Seq({ unsafe { SeqExpr::from_node_id_unchecked(id, ast) } }),
            NodeKind::Str => Expr::Lit(Lit::Str({
                unsafe { Str::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::SuperPropExpr => {
                Expr::SuperProp({ unsafe { SuperPropExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::TaggedTpl => {
                Expr::TaggedTpl({ unsafe { TaggedTpl::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ThisExpr => {
                Expr::This({ unsafe { ThisExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Tpl => Expr::Tpl({ unsafe { Tpl::from_node_id_unchecked(id, ast) } }),
            NodeKind::UnaryExpr => {
                Expr::Unary({ unsafe { UnaryExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::UpdateExpr => {
                Expr::Update({ unsafe { UpdateExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::YieldExpr => {
                Expr::Yield({ unsafe { YieldExpr::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ThisExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ArrayLit {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ObjectLit {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PropOrSpread {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Spread(it) => it.node_id(),
            Self::Prop(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::AssignProp => PropOrSpread::Prop(Prop::Assign({
                unsafe { AssignProp::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::GetterProp => PropOrSpread::Prop(Prop::Getter({
                unsafe { GetterProp::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Ident => PropOrSpread::Prop(Prop::Shorthand({
                unsafe { Ident::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::KeyValueProp => PropOrSpread::Prop(Prop::KeyValue({
                unsafe { KeyValueProp::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MethodProp => PropOrSpread::Prop(Prop::Method({
                unsafe { MethodProp::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::SetterProp => PropOrSpread::Prop(Prop::Setter({
                unsafe { SetterProp::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::SpreadElement => {
                PropOrSpread::Spread({ unsafe { SpreadElement::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for SpreadElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for UnaryExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for UpdateExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BinExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for FnExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ClassExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AssignExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for MemberExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for MemberProp {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::PrivateName(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ComputedPropName => MemberProp::Computed({
                unsafe { ComputedPropName::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::IdentName => {
                MemberProp::Ident({ unsafe { IdentName::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::PrivateName => {
                MemberProp::PrivateName({ unsafe { PrivateName::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for SuperPropExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SuperProp {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ComputedPropName => SuperProp::Computed({
                unsafe { ComputedPropName::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::IdentName => {
                SuperProp::Ident({ unsafe { IdentName::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for CondExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ImportExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for CallExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for NewExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SeqExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ArrowExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for YieldExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for MetaPropExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AwaitExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Tpl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for TaggedTpl {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for TplElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ParenExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Callee {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Super(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ArrayLit => Callee::Expr(Expr::Array({
                unsafe { ArrayLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ArrowExpr => Callee::Expr(Expr::Arrow({
                unsafe { ArrowExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AssignExpr => Callee::Expr(Expr::Assign({
                unsafe { AssignExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AwaitExpr => Callee::Expr(Expr::Await({
                unsafe { AwaitExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BigInt => Callee::Expr(Expr::Lit(Lit::BigInt({
                unsafe { BigInt::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::BinExpr => Callee::Expr(Expr::Bin({
                unsafe { BinExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Bool => Callee::Expr(Expr::Lit(Lit::Bool({
                unsafe { Bool::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::CallExpr => Callee::Expr(Expr::Call({
                unsafe { CallExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ClassExpr => Callee::Expr(Expr::Class({
                unsafe { ClassExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::CondExpr => Callee::Expr(Expr::Cond({
                unsafe { CondExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::FnExpr => Callee::Expr(Expr::Fn({
                unsafe { FnExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Ident => Callee::Expr(Expr::Ident({
                unsafe { Ident::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ImportExpr => Callee::Expr(Expr::Import({
                unsafe { ImportExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Invalid => Callee::Expr(Expr::Invalid({
                unsafe { Invalid::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXElement => Callee::Expr(Expr::JSXElement({
                unsafe { JSXElement::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXEmptyExpr => Callee::Expr(Expr::JSXEmpty({
                unsafe { JSXEmptyExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXFragment => Callee::Expr(Expr::JSXFragment({
                unsafe { JSXFragment::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXMemberExpr => Callee::Expr(Expr::JSXMember({
                unsafe { JSXMemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXNamespacedName => Callee::Expr(Expr::JSXNamespacedName({
                unsafe { JSXNamespacedName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MemberExpr => Callee::Expr(Expr::Member({
                unsafe { MemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MetaPropExpr => Callee::Expr(Expr::MetaProp({
                unsafe { MetaPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::NewExpr => Callee::Expr(Expr::New({
                unsafe { NewExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Null => Callee::Expr(Expr::Lit(Lit::Null({
                unsafe { Null::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Number => Callee::Expr(Expr::Lit(Lit::Num({
                unsafe { Number::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ObjectLit => Callee::Expr(Expr::Object({
                unsafe { ObjectLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::OptChainExpr => Callee::Expr(Expr::OptChain({
                unsafe { OptChainExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ParenExpr => Callee::Expr(Expr::Paren({
                unsafe { ParenExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::PrivateName => Callee::Expr(Expr::PrivateName({
                unsafe { PrivateName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Regex => Callee::Expr(Expr::Lit(Lit::Regex({
                unsafe { Regex::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::SeqExpr => Callee::Expr(Expr::Seq({
                unsafe { SeqExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Str => Callee::Expr(Expr::Lit(Lit::Str({
                unsafe { Str::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Super => Callee::Super({ unsafe { Super::from_node_id_unchecked(id, ast) } }),
            NodeKind::SuperPropExpr => Callee::Expr(Expr::SuperProp({
                unsafe { SuperPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::TaggedTpl => Callee::Expr(Expr::TaggedTpl({
                unsafe { TaggedTpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ThisExpr => Callee::Expr(Expr::This({
                unsafe { ThisExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Tpl => Callee::Expr(Expr::Tpl({
                unsafe { Tpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UnaryExpr => Callee::Expr(Expr::Unary({
                unsafe { UnaryExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UpdateExpr => Callee::Expr(Expr::Update({
                unsafe { UpdateExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::YieldExpr => Callee::Expr(Expr::Yield({
                unsafe { YieldExpr::from_node_id_unchecked(id, ast) }
            })),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for Super {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExprOrSpread {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BlockStmtOrExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::BlockStmt(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ArrayLit => BlockStmtOrExpr::Expr(Expr::Array({
                unsafe { ArrayLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ArrowExpr => BlockStmtOrExpr::Expr(Expr::Arrow({
                unsafe { ArrowExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AssignExpr => BlockStmtOrExpr::Expr(Expr::Assign({
                unsafe { AssignExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AwaitExpr => BlockStmtOrExpr::Expr(Expr::Await({
                unsafe { AwaitExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BigInt => BlockStmtOrExpr::Expr(Expr::Lit(Lit::BigInt({
                unsafe { BigInt::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::BinExpr => BlockStmtOrExpr::Expr(Expr::Bin({
                unsafe { BinExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BlockStmt => BlockStmtOrExpr::BlockStmt({
                unsafe { BlockStmt::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::Bool => BlockStmtOrExpr::Expr(Expr::Lit(Lit::Bool({
                unsafe { Bool::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::CallExpr => BlockStmtOrExpr::Expr(Expr::Call({
                unsafe { CallExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ClassExpr => BlockStmtOrExpr::Expr(Expr::Class({
                unsafe { ClassExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::CondExpr => BlockStmtOrExpr::Expr(Expr::Cond({
                unsafe { CondExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::FnExpr => BlockStmtOrExpr::Expr(Expr::Fn({
                unsafe { FnExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Ident => BlockStmtOrExpr::Expr(Expr::Ident({
                unsafe { Ident::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ImportExpr => BlockStmtOrExpr::Expr(Expr::Import({
                unsafe { ImportExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Invalid => BlockStmtOrExpr::Expr(Expr::Invalid({
                unsafe { Invalid::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXElement => BlockStmtOrExpr::Expr(Expr::JSXElement({
                unsafe { JSXElement::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXEmptyExpr => BlockStmtOrExpr::Expr(Expr::JSXEmpty({
                unsafe { JSXEmptyExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXFragment => BlockStmtOrExpr::Expr(Expr::JSXFragment({
                unsafe { JSXFragment::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXMemberExpr => BlockStmtOrExpr::Expr(Expr::JSXMember({
                unsafe { JSXMemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXNamespacedName => BlockStmtOrExpr::Expr(Expr::JSXNamespacedName({
                unsafe { JSXNamespacedName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MemberExpr => BlockStmtOrExpr::Expr(Expr::Member({
                unsafe { MemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MetaPropExpr => BlockStmtOrExpr::Expr(Expr::MetaProp({
                unsafe { MetaPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::NewExpr => BlockStmtOrExpr::Expr(Expr::New({
                unsafe { NewExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Null => BlockStmtOrExpr::Expr(Expr::Lit(Lit::Null({
                unsafe { Null::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Number => BlockStmtOrExpr::Expr(Expr::Lit(Lit::Num({
                unsafe { Number::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ObjectLit => BlockStmtOrExpr::Expr(Expr::Object({
                unsafe { ObjectLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::OptChainExpr => BlockStmtOrExpr::Expr(Expr::OptChain({
                unsafe { OptChainExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ParenExpr => BlockStmtOrExpr::Expr(Expr::Paren({
                unsafe { ParenExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::PrivateName => BlockStmtOrExpr::Expr(Expr::PrivateName({
                unsafe { PrivateName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Regex => BlockStmtOrExpr::Expr(Expr::Lit(Lit::Regex({
                unsafe { Regex::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::SeqExpr => BlockStmtOrExpr::Expr(Expr::Seq({
                unsafe { SeqExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Str => BlockStmtOrExpr::Expr(Expr::Lit(Lit::Str({
                unsafe { Str::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::SuperPropExpr => BlockStmtOrExpr::Expr(Expr::SuperProp({
                unsafe { SuperPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::TaggedTpl => BlockStmtOrExpr::Expr(Expr::TaggedTpl({
                unsafe { TaggedTpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ThisExpr => BlockStmtOrExpr::Expr(Expr::This({
                unsafe { ThisExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Tpl => BlockStmtOrExpr::Expr(Expr::Tpl({
                unsafe { Tpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UnaryExpr => BlockStmtOrExpr::Expr(Expr::Unary({
                unsafe { UnaryExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UpdateExpr => BlockStmtOrExpr::Expr(Expr::Update({
                unsafe { UpdateExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::YieldExpr => BlockStmtOrExpr::Expr(Expr::Yield({
                unsafe { YieldExpr::from_node_id_unchecked(id, ast) }
            })),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for AssignTarget {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Simple(it) => it.node_id(),
            Self::Pat(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ArrayPat => AssignTarget::Pat(AssignTargetPat::Array({
                unsafe { ArrayPat::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BindingIdent => AssignTarget::Simple(SimpleAssignTarget::Ident({
                unsafe { BindingIdent::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Invalid => AssignTarget::Simple(SimpleAssignTarget::Invalid({
                unsafe { Invalid::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MemberExpr => AssignTarget::Simple(SimpleAssignTarget::Member({
                unsafe { MemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ObjectPat => AssignTarget::Pat(AssignTargetPat::Object({
                unsafe { ObjectPat::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::OptChainExpr => AssignTarget::Simple(SimpleAssignTarget::OptChain({
                unsafe { OptChainExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ParenExpr => AssignTarget::Simple(SimpleAssignTarget::Paren({
                unsafe { ParenExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::SuperPropExpr => AssignTarget::Simple(SimpleAssignTarget::SuperProp({
                unsafe { SuperPropExpr::from_node_id_unchecked(id, ast) }
            })),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for AssignTargetPat {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Array(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ArrayPat => {
                AssignTargetPat::Array({ unsafe { ArrayPat::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Invalid => {
                AssignTargetPat::Invalid({ unsafe { Invalid::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ObjectPat => {
                AssignTargetPat::Object({ unsafe { ObjectPat::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for SimpleAssignTarget {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Member(it) => it.node_id(),
            Self::SuperProp(it) => it.node_id(),
            Self::Paren(it) => it.node_id(),
            Self::OptChain(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::BindingIdent => SimpleAssignTarget::Ident({
                unsafe { BindingIdent::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::Invalid => {
                SimpleAssignTarget::Invalid({ unsafe { Invalid::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::MemberExpr => SimpleAssignTarget::Member({
                unsafe { MemberExpr::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::OptChainExpr => SimpleAssignTarget::OptChain({
                unsafe { OptChainExpr::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::ParenExpr => {
                SimpleAssignTarget::Paren({ unsafe { ParenExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::SuperPropExpr => SimpleAssignTarget::SuperProp({
                unsafe { SuperPropExpr::from_node_id_unchecked(id, ast) }
            }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for OptChainExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for OptChainBase {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Member(it) => it.node_id(),
            Self::Call(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::MemberExpr => {
                OptChainBase::Member({ unsafe { MemberExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::OptCall => {
                OptChainBase::Call({ unsafe { OptCall::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for OptCall {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Invalid {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Function {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Param {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ParamOrTsParamProp {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Param(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::Param => {
                ParamOrTsParamProp::Param({ unsafe { Param::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for Class {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ClassMember {
    #[inline]
    fn node_id(self) -> NodeId {
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
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::AutoAccessor => ClassMember::AutoAccessor({
                unsafe { AutoAccessor::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::ClassMethod => {
                ClassMember::Method({ unsafe { ClassMethod::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ClassProp => {
                ClassMember::ClassProp({ unsafe { ClassProp::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Constructor => ClassMember::Constructor({
                unsafe { Constructor::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::EmptyStmt => {
                ClassMember::Empty({ unsafe { EmptyStmt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::PrivateMethod => ClassMember::PrivateMethod({
                unsafe { PrivateMethod::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::PrivateProp => ClassMember::PrivateProp({
                unsafe { PrivateProp::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::StaticBlock => ClassMember::StaticBlock({
                unsafe { StaticBlock::from_node_id_unchecked(id, ast) }
            }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ClassProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PrivateProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ClassMethod {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PrivateMethod {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Constructor {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Decorator {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for StaticBlock {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Key {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Private(it) => it.node_id(),
            Self::Public(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::BigInt => Key::Public(PropName::BigInt({
                unsafe { BigInt::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ComputedPropName => Key::Public(PropName::Computed({
                unsafe { ComputedPropName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::IdentName => Key::Public(PropName::Ident({
                unsafe { IdentName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Number => Key::Public(PropName::Num({
                unsafe { Number::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::PrivateName => {
                Key::Private({ unsafe { PrivateName::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Str => Key::Public(PropName::Str({
                unsafe { Str::from_node_id_unchecked(id, ast) }
            })),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for AutoAccessor {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Prop {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Shorthand(it) => it.node_id(),
            Self::KeyValue(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Getter(it) => it.node_id(),
            Self::Setter(it) => it.node_id(),
            Self::Method(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::AssignProp => {
                Prop::Assign({ unsafe { AssignProp::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::GetterProp => {
                Prop::Getter({ unsafe { GetterProp::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Ident => {
                Prop::Shorthand({ unsafe { Ident::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::KeyValueProp => {
                Prop::KeyValue({ unsafe { KeyValueProp::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::MethodProp => {
                Prop::Method({ unsafe { MethodProp::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::SetterProp => {
                Prop::Setter({ unsafe { SetterProp::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for KeyValueProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AssignProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for GetterProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SetterProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for MethodProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PropName {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::BigInt => {
                PropName::BigInt({ unsafe { BigInt::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ComputedPropName => {
                PropName::Computed({ unsafe { ComputedPropName::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::IdentName => {
                PropName::Ident({ unsafe { IdentName::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Number => {
                PropName::Num({ unsafe { Number::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Str => PropName::Str({ unsafe { Str::from_node_id_unchecked(id, ast) } }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ComputedPropName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Pat {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Array(it) => it.node_id(),
            Self::Rest(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ArrayLit => Pat::Expr(Expr::Array({
                unsafe { ArrayLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ArrayPat => {
                Pat::Array({ unsafe { ArrayPat::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::ArrowExpr => Pat::Expr(Expr::Arrow({
                unsafe { ArrowExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AssignExpr => Pat::Expr(Expr::Assign({
                unsafe { AssignExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AssignPat => {
                Pat::Assign({ unsafe { AssignPat::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::AwaitExpr => Pat::Expr(Expr::Await({
                unsafe { AwaitExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BigInt => Pat::Expr(Expr::Lit(Lit::BigInt({
                unsafe { BigInt::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::BinExpr => Pat::Expr(Expr::Bin({
                unsafe { BinExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BindingIdent => {
                Pat::Ident({ unsafe { BindingIdent::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::Bool => Pat::Expr(Expr::Lit(Lit::Bool({
                unsafe { Bool::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::CallExpr => Pat::Expr(Expr::Call({
                unsafe { CallExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ClassExpr => Pat::Expr(Expr::Class({
                unsafe { ClassExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::CondExpr => Pat::Expr(Expr::Cond({
                unsafe { CondExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::FnExpr => Pat::Expr(Expr::Fn({
                unsafe { FnExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Ident => Pat::Expr(Expr::Ident({
                unsafe { Ident::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ImportExpr => Pat::Expr(Expr::Import({
                unsafe { ImportExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Invalid => {
                Pat::Invalid({ unsafe { Invalid::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXElement => Pat::Expr(Expr::JSXElement({
                unsafe { JSXElement::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXEmptyExpr => Pat::Expr(Expr::JSXEmpty({
                unsafe { JSXEmptyExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXFragment => Pat::Expr(Expr::JSXFragment({
                unsafe { JSXFragment::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXMemberExpr => Pat::Expr(Expr::JSXMember({
                unsafe { JSXMemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXNamespacedName => Pat::Expr(Expr::JSXNamespacedName({
                unsafe { JSXNamespacedName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MemberExpr => Pat::Expr(Expr::Member({
                unsafe { MemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MetaPropExpr => Pat::Expr(Expr::MetaProp({
                unsafe { MetaPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::NewExpr => Pat::Expr(Expr::New({
                unsafe { NewExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Null => Pat::Expr(Expr::Lit(Lit::Null({
                unsafe { Null::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Number => Pat::Expr(Expr::Lit(Lit::Num({
                unsafe { Number::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ObjectLit => Pat::Expr(Expr::Object({
                unsafe { ObjectLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ObjectPat => {
                Pat::Object({ unsafe { ObjectPat::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::OptChainExpr => Pat::Expr(Expr::OptChain({
                unsafe { OptChainExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ParenExpr => Pat::Expr(Expr::Paren({
                unsafe { ParenExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::PrivateName => Pat::Expr(Expr::PrivateName({
                unsafe { PrivateName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Regex => Pat::Expr(Expr::Lit(Lit::Regex({
                unsafe { Regex::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::RestPat => Pat::Rest({ unsafe { RestPat::from_node_id_unchecked(id, ast) } }),
            NodeKind::SeqExpr => Pat::Expr(Expr::Seq({
                unsafe { SeqExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Str => Pat::Expr(Expr::Lit(Lit::Str({
                unsafe { Str::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::SuperPropExpr => Pat::Expr(Expr::SuperProp({
                unsafe { SuperPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::TaggedTpl => Pat::Expr(Expr::TaggedTpl({
                unsafe { TaggedTpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ThisExpr => Pat::Expr(Expr::This({
                unsafe { ThisExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Tpl => Pat::Expr(Expr::Tpl({
                unsafe { Tpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UnaryExpr => Pat::Expr(Expr::Unary({
                unsafe { UnaryExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UpdateExpr => Pat::Expr(Expr::Update({
                unsafe { UpdateExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::YieldExpr => Pat::Expr(Expr::Yield({
                unsafe { YieldExpr::from_node_id_unchecked(id, ast) }
            })),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for ArrayPat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ObjectPat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AssignPat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for RestPat {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ObjectPatProp {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::KeyValue(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Rest(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::AssignPatProp => {
                ObjectPatProp::Assign({ unsafe { AssignPatProp::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::KeyValuePatProp => ObjectPatProp::KeyValue({
                unsafe { KeyValuePatProp::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::RestPat => {
                ObjectPatProp::Rest({ unsafe { RestPat::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for KeyValuePatProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AssignPatProp {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Ident {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for IdentName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PrivateName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BindingIdent {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Lit {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Str(it) => it.node_id(),
            Self::Bool(it) => it.node_id(),
            Self::Null(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
            Self::Regex(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::BigInt => Lit::BigInt({ unsafe { BigInt::from_node_id_unchecked(id, ast) } }),
            NodeKind::Bool => Lit::Bool({ unsafe { Bool::from_node_id_unchecked(id, ast) } }),
            NodeKind::Null => Lit::Null({ unsafe { Null::from_node_id_unchecked(id, ast) } }),
            NodeKind::Number => Lit::Num({ unsafe { Number::from_node_id_unchecked(id, ast) } }),
            NodeKind::Regex => Lit::Regex({ unsafe { Regex::from_node_id_unchecked(id, ast) } }),
            NodeKind::Str => Lit::Str({ unsafe { Str::from_node_id_unchecked(id, ast) } }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for Str {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Bool {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Null {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Number {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BigInt {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Regex {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXObject {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::JSXMemberExpr(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::Ident => {
                JSXObject::Ident({ unsafe { Ident::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXMemberExpr => JSXObject::JSXMemberExpr({
                unsafe { JSXMemberExpr::from_node_id_unchecked(id, ast) }
            }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for JSXMemberExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXNamespacedName {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXEmptyExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXExprContainer {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXExpr {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::JSXEmptyExpr(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::ArrayLit => JSXExpr::Expr(Expr::Array({
                unsafe { ArrayLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ArrowExpr => JSXExpr::Expr(Expr::Arrow({
                unsafe { ArrowExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AssignExpr => JSXExpr::Expr(Expr::Assign({
                unsafe { AssignExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::AwaitExpr => JSXExpr::Expr(Expr::Await({
                unsafe { AwaitExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::BigInt => JSXExpr::Expr(Expr::Lit(Lit::BigInt({
                unsafe { BigInt::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::BinExpr => JSXExpr::Expr(Expr::Bin({
                unsafe { BinExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Bool => JSXExpr::Expr(Expr::Lit(Lit::Bool({
                unsafe { Bool::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::CallExpr => JSXExpr::Expr(Expr::Call({
                unsafe { CallExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ClassExpr => JSXExpr::Expr(Expr::Class({
                unsafe { ClassExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::CondExpr => JSXExpr::Expr(Expr::Cond({
                unsafe { CondExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::FnExpr => JSXExpr::Expr(Expr::Fn({
                unsafe { FnExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Ident => JSXExpr::Expr(Expr::Ident({
                unsafe { Ident::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ImportExpr => JSXExpr::Expr(Expr::Import({
                unsafe { ImportExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Invalid => JSXExpr::Expr(Expr::Invalid({
                unsafe { Invalid::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXElement => JSXExpr::Expr(Expr::JSXElement({
                unsafe { JSXElement::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXEmptyExpr => {
                JSXExpr::JSXEmptyExpr({ unsafe { JSXEmptyExpr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXFragment => JSXExpr::Expr(Expr::JSXFragment({
                unsafe { JSXFragment::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXMemberExpr => JSXExpr::Expr(Expr::JSXMember({
                unsafe { JSXMemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::JSXNamespacedName => JSXExpr::Expr(Expr::JSXNamespacedName({
                unsafe { JSXNamespacedName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MemberExpr => JSXExpr::Expr(Expr::Member({
                unsafe { MemberExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::MetaPropExpr => JSXExpr::Expr(Expr::MetaProp({
                unsafe { MetaPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::NewExpr => JSXExpr::Expr(Expr::New({
                unsafe { NewExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Null => JSXExpr::Expr(Expr::Lit(Lit::Null({
                unsafe { Null::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::Number => JSXExpr::Expr(Expr::Lit(Lit::Num({
                unsafe { Number::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::ObjectLit => JSXExpr::Expr(Expr::Object({
                unsafe { ObjectLit::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::OptChainExpr => JSXExpr::Expr(Expr::OptChain({
                unsafe { OptChainExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ParenExpr => JSXExpr::Expr(Expr::Paren({
                unsafe { ParenExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::PrivateName => JSXExpr::Expr(Expr::PrivateName({
                unsafe { PrivateName::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Regex => JSXExpr::Expr(Expr::Lit(Lit::Regex({
                unsafe { Regex::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::SeqExpr => JSXExpr::Expr(Expr::Seq({
                unsafe { SeqExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Str => JSXExpr::Expr(Expr::Lit(Lit::Str({
                unsafe { Str::from_node_id_unchecked(id, ast) }
            }))),
            NodeKind::SuperPropExpr => JSXExpr::Expr(Expr::SuperProp({
                unsafe { SuperPropExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::TaggedTpl => JSXExpr::Expr(Expr::TaggedTpl({
                unsafe { TaggedTpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::ThisExpr => JSXExpr::Expr(Expr::This({
                unsafe { ThisExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::Tpl => JSXExpr::Expr(Expr::Tpl({
                unsafe { Tpl::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UnaryExpr => JSXExpr::Expr(Expr::Unary({
                unsafe { UnaryExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::UpdateExpr => JSXExpr::Expr(Expr::Update({
                unsafe { UpdateExpr::from_node_id_unchecked(id, ast) }
            })),
            NodeKind::YieldExpr => JSXExpr::Expr(Expr::Yield({
                unsafe { YieldExpr::from_node_id_unchecked(id, ast) }
            })),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for JSXSpreadChild {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXElementName {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::JSXMemberExpr(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::Ident => {
                JSXElementName::Ident({ unsafe { Ident::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXMemberExpr => JSXElementName::JSXMemberExpr({
                unsafe { JSXMemberExpr::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::JSXNamespacedName => JSXElementName::JSXNamespacedName({
                unsafe { JSXNamespacedName::from_node_id_unchecked(id, ast) }
            }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for JSXOpeningElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXAttrOrSpread {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::JSXAttr(it) => it.node_id(),
            Self::SpreadElement(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::JSXAttr => {
                JSXAttrOrSpread::JSXAttr({ unsafe { JSXAttr::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::SpreadElement => JSXAttrOrSpread::SpreadElement({
                unsafe { SpreadElement::from_node_id_unchecked(id, ast) }
            }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for JSXClosingElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXAttr {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXAttrName {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::IdentName => {
                JSXAttrName::Ident({ unsafe { IdentName::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXNamespacedName => JSXAttrName::JSXNamespacedName({
                unsafe { JSXNamespacedName::from_node_id_unchecked(id, ast) }
            }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for JSXAttrValue {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::Str(it) => it.node_id(),
            Self::JSXExprContainer(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::JSXElement => {
                JSXAttrValue::JSXElement({ unsafe { JSXElement::from_node_id_unchecked(id, ast) } })
            }
            NodeKind::JSXExprContainer => JSXAttrValue::JSXExprContainer({
                unsafe { JSXExprContainer::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::JSXFragment => JSXAttrValue::JSXFragment({
                unsafe { JSXFragment::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::Str => JSXAttrValue::Str({ unsafe { Str::from_node_id_unchecked(id, ast) } }),
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for JSXText {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXElement {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXElementChild {
    #[inline]
    fn node_id(self) -> NodeId {
        match self {
            Self::JSXText(it) => it.node_id(),
            Self::JSXExprContainer(it) => it.node_id(),
            Self::JSXSpreadChild(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self {
        let kind = unsafe { *ast.nodes.kind_unchecked(id) };
        match kind {
            NodeKind::JSXElement => JSXElementChild::JSXElement({
                unsafe { JSXElement::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::JSXExprContainer => JSXElementChild::JSXExprContainer({
                unsafe { JSXExprContainer::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::JSXFragment => JSXElementChild::JSXFragment({
                unsafe { JSXFragment::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::JSXSpreadChild => JSXElementChild::JSXSpreadChild({
                unsafe { JSXSpreadChild::from_node_id_unchecked(id, ast) }
            }),
            NodeKind::JSXText => {
                JSXElementChild::JSXText({ unsafe { JSXText::from_node_id_unchecked(id, ast) } })
            }
            _ => unreachable!(),
        }
    }
}
impl NodeIdTrait for JSXFragment {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXOpeningFragment {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXClosingFragment {
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast<'_>) -> Self {
        Self(node_id)
    }
}
