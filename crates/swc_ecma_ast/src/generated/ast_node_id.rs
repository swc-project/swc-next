#![allow(unused)]
use crate::{Ast, NodeKind};
use crate::{ast::*, node_id::*};
impl NodeIdTrait for Program {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Module(it) => it.node_id(),
            Self::Script(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::Module => Program::Module({ Module::from_node_id(id, ast) }),
            NodeKind::Script => Program::Script({ Script::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for Module {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Module);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Script {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Script);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ModuleItem {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::ModuleDecl(it) => it.node_id(),
            Self::Stmt(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::BlockStmt => {
                ModuleItem::Stmt(Stmt::Block({ BlockStmt::from_node_id(id, ast) }))
            }
            NodeKind::BreakStmt => {
                ModuleItem::Stmt(Stmt::Break({ BreakStmt::from_node_id(id, ast) }))
            }
            NodeKind::ClassDecl => ModuleItem::Stmt(Stmt::Decl(Decl::Class({
                ClassDecl::from_node_id(id, ast)
            }))),
            NodeKind::ContinueStmt => {
                ModuleItem::Stmt(Stmt::Continue({ ContinueStmt::from_node_id(id, ast) }))
            }
            NodeKind::DebuggerStmt => {
                ModuleItem::Stmt(Stmt::Debugger({ DebuggerStmt::from_node_id(id, ast) }))
            }
            NodeKind::DoWhileStmt => {
                ModuleItem::Stmt(Stmt::DoWhile({ DoWhileStmt::from_node_id(id, ast) }))
            }
            NodeKind::EmptyStmt => {
                ModuleItem::Stmt(Stmt::Empty({ EmptyStmt::from_node_id(id, ast) }))
            }
            NodeKind::ExportAll => {
                ModuleItem::ModuleDecl(ModuleDecl::ExportAll({ ExportAll::from_node_id(id, ast) }))
            }
            NodeKind::ExportDecl => ModuleItem::ModuleDecl(ModuleDecl::ExportDecl({
                ExportDecl::from_node_id(id, ast)
            })),
            NodeKind::ExportDefaultDecl => ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl({
                ExportDefaultDecl::from_node_id(id, ast)
            })),
            NodeKind::ExportDefaultExpr => ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr({
                ExportDefaultExpr::from_node_id(id, ast)
            })),
            NodeKind::ExprStmt => ModuleItem::Stmt(Stmt::Expr({ ExprStmt::from_node_id(id, ast) })),
            NodeKind::FnDecl => {
                ModuleItem::Stmt(Stmt::Decl(Decl::Fn({ FnDecl::from_node_id(id, ast) })))
            }
            NodeKind::ForInStmt => {
                ModuleItem::Stmt(Stmt::ForIn({ ForInStmt::from_node_id(id, ast) }))
            }
            NodeKind::ForOfStmt => {
                ModuleItem::Stmt(Stmt::ForOf({ ForOfStmt::from_node_id(id, ast) }))
            }
            NodeKind::ForStmt => ModuleItem::Stmt(Stmt::For({ ForStmt::from_node_id(id, ast) })),
            NodeKind::IfStmt => ModuleItem::Stmt(Stmt::If({ IfStmt::from_node_id(id, ast) })),
            NodeKind::ImportDecl => {
                ModuleItem::ModuleDecl(ModuleDecl::Import({ ImportDecl::from_node_id(id, ast) }))
            }
            NodeKind::LabeledStmt => {
                ModuleItem::Stmt(Stmt::Labeled({ LabeledStmt::from_node_id(id, ast) }))
            }
            NodeKind::NamedExport => ModuleItem::ModuleDecl(ModuleDecl::ExportNamed({
                NamedExport::from_node_id(id, ast)
            })),
            NodeKind::ReturnStmt => {
                ModuleItem::Stmt(Stmt::Return({ ReturnStmt::from_node_id(id, ast) }))
            }
            NodeKind::SwitchStmt => {
                ModuleItem::Stmt(Stmt::Switch({ SwitchStmt::from_node_id(id, ast) }))
            }
            NodeKind::ThrowStmt => {
                ModuleItem::Stmt(Stmt::Throw({ ThrowStmt::from_node_id(id, ast) }))
            }
            NodeKind::TryStmt => ModuleItem::Stmt(Stmt::Try({ TryStmt::from_node_id(id, ast) })),
            NodeKind::UsingDecl => ModuleItem::Stmt(Stmt::Decl(Decl::Using({
                UsingDecl::from_node_id(id, ast)
            }))),
            NodeKind::VarDecl => {
                ModuleItem::Stmt(Stmt::Decl(Decl::Var({ VarDecl::from_node_id(id, ast) })))
            }
            NodeKind::WhileStmt => {
                ModuleItem::Stmt(Stmt::While({ WhileStmt::from_node_id(id, ast) }))
            }
            NodeKind::WithStmt => ModuleItem::Stmt(Stmt::With({ WithStmt::from_node_id(id, ast) })),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ModuleDecl {
    #[inline]
    fn node_id(&self) -> NodeId {
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
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ExportAll => ModuleDecl::ExportAll({ ExportAll::from_node_id(id, ast) }),
            NodeKind::ExportDecl => ModuleDecl::ExportDecl({ ExportDecl::from_node_id(id, ast) }),
            NodeKind::ExportDefaultDecl => {
                ModuleDecl::ExportDefaultDecl({ ExportDefaultDecl::from_node_id(id, ast) })
            }
            NodeKind::ExportDefaultExpr => {
                ModuleDecl::ExportDefaultExpr({ ExportDefaultExpr::from_node_id(id, ast) })
            }
            NodeKind::ImportDecl => ModuleDecl::Import({ ImportDecl::from_node_id(id, ast) }),
            NodeKind::NamedExport => {
                ModuleDecl::ExportNamed({ NamedExport::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ImportDecl {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ImportDecl);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ImportSpecifier {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Named(it) => it.node_id(),
            Self::Default(it) => it.node_id(),
            Self::Namespace(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ImportDefaultSpecifier => {
                ImportSpecifier::Default({ ImportDefaultSpecifier::from_node_id(id, ast) })
            }
            NodeKind::ImportNamedSpecifier => {
                ImportSpecifier::Named({ ImportNamedSpecifier::from_node_id(id, ast) })
            }
            NodeKind::ImportStarAsSpecifier => {
                ImportSpecifier::Namespace({ ImportStarAsSpecifier::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ImportNamedSpecifier {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ImportNamedSpecifier);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ImportDefaultSpecifier {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ImportDefaultSpecifier);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ImportStarAsSpecifier {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ImportStarAsSpecifier);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportDecl {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ExportDecl);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for NamedExport {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::NamedExport);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportSpecifier {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Namespace(it) => it.node_id(),
            Self::Default(it) => it.node_id(),
            Self::Named(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ExportDefaultSpecifier => {
                ExportSpecifier::Default({ ExportDefaultSpecifier::from_node_id(id, ast) })
            }
            NodeKind::ExportNamedSpecifier => {
                ExportSpecifier::Named({ ExportNamedSpecifier::from_node_id(id, ast) })
            }
            NodeKind::ExportNamespaceSpecifier => {
                ExportSpecifier::Namespace({ ExportNamespaceSpecifier::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ExportNamespaceSpecifier {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ExportNamespaceSpecifier);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ModuleExportName {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::Ident => ModuleExportName::Ident({ Ident::from_node_id(id, ast) }),
            NodeKind::Str => ModuleExportName::Str({ Str::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ExportDefaultSpecifier {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ExportDefaultSpecifier);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportNamedSpecifier {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ExportNamedSpecifier);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportDefaultDecl {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ExportDefaultDecl);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for DefaultDecl {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Class(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ClassExpr => DefaultDecl::Class({ ClassExpr::from_node_id(id, ast) }),
            NodeKind::FnExpr => DefaultDecl::Fn({ FnExpr::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ExportDefaultExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ExportDefaultExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExportAll {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ExportAll);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BlockStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::BlockStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Stmt {
    #[inline]
    fn node_id(&self) -> NodeId {
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
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::BlockStmt => Stmt::Block({ BlockStmt::from_node_id(id, ast) }),
            NodeKind::BreakStmt => Stmt::Break({ BreakStmt::from_node_id(id, ast) }),
            NodeKind::ClassDecl => Stmt::Decl(Decl::Class({ ClassDecl::from_node_id(id, ast) })),
            NodeKind::ContinueStmt => Stmt::Continue({ ContinueStmt::from_node_id(id, ast) }),
            NodeKind::DebuggerStmt => Stmt::Debugger({ DebuggerStmt::from_node_id(id, ast) }),
            NodeKind::DoWhileStmt => Stmt::DoWhile({ DoWhileStmt::from_node_id(id, ast) }),
            NodeKind::EmptyStmt => Stmt::Empty({ EmptyStmt::from_node_id(id, ast) }),
            NodeKind::ExprStmt => Stmt::Expr({ ExprStmt::from_node_id(id, ast) }),
            NodeKind::FnDecl => Stmt::Decl(Decl::Fn({ FnDecl::from_node_id(id, ast) })),
            NodeKind::ForInStmt => Stmt::ForIn({ ForInStmt::from_node_id(id, ast) }),
            NodeKind::ForOfStmt => Stmt::ForOf({ ForOfStmt::from_node_id(id, ast) }),
            NodeKind::ForStmt => Stmt::For({ ForStmt::from_node_id(id, ast) }),
            NodeKind::IfStmt => Stmt::If({ IfStmt::from_node_id(id, ast) }),
            NodeKind::LabeledStmt => Stmt::Labeled({ LabeledStmt::from_node_id(id, ast) }),
            NodeKind::ReturnStmt => Stmt::Return({ ReturnStmt::from_node_id(id, ast) }),
            NodeKind::SwitchStmt => Stmt::Switch({ SwitchStmt::from_node_id(id, ast) }),
            NodeKind::ThrowStmt => Stmt::Throw({ ThrowStmt::from_node_id(id, ast) }),
            NodeKind::TryStmt => Stmt::Try({ TryStmt::from_node_id(id, ast) }),
            NodeKind::UsingDecl => Stmt::Decl(Decl::Using({ UsingDecl::from_node_id(id, ast) })),
            NodeKind::VarDecl => Stmt::Decl(Decl::Var({ VarDecl::from_node_id(id, ast) })),
            NodeKind::WhileStmt => Stmt::While({ WhileStmt::from_node_id(id, ast) }),
            NodeKind::WithStmt => Stmt::With({ WithStmt::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ExprStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ExprStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for EmptyStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::EmptyStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for DebuggerStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::DebuggerStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for WithStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::WithStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ReturnStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ReturnStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for LabeledStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::LabeledStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BreakStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::BreakStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ContinueStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ContinueStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for IfStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::IfStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SwitchStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::SwitchStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ThrowStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ThrowStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for TryStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::TryStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for WhileStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::WhileStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for DoWhileStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::DoWhileStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ForStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ForStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ForInStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ForInStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ForOfStmt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ForOfStmt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SwitchCase {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::SwitchCase);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for CatchClause {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::CatchClause);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ForHead {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::VarDecl(it) => it.node_id(),
            Self::UsingDecl(it) => it.node_id(),
            Self::Pat(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ArrayLit => {
                ForHead::Pat(Pat::Expr(Expr::Array({ ArrayLit::from_node_id(id, ast) })))
            }
            NodeKind::ArrayPat => ForHead::Pat(Pat::Array({ ArrayPat::from_node_id(id, ast) })),
            NodeKind::ArrowExpr => {
                ForHead::Pat(Pat::Expr(Expr::Arrow({ ArrowExpr::from_node_id(id, ast) })))
            }
            NodeKind::AssignExpr => ForHead::Pat(Pat::Expr(Expr::Assign({
                AssignExpr::from_node_id(id, ast)
            }))),
            NodeKind::AssignPat => ForHead::Pat(Pat::Assign({ AssignPat::from_node_id(id, ast) })),
            NodeKind::AwaitExpr => {
                ForHead::Pat(Pat::Expr(Expr::Await({ AwaitExpr::from_node_id(id, ast) })))
            }
            NodeKind::BigInt => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::BigInt({
                BigInt::from_node_id(id, ast)
            })))),
            NodeKind::BinExpr => {
                ForHead::Pat(Pat::Expr(Expr::Bin({ BinExpr::from_node_id(id, ast) })))
            }
            NodeKind::BindingIdent => {
                ForHead::Pat(Pat::Ident({ BindingIdent::from_node_id(id, ast) }))
            }
            NodeKind::Bool => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Bool({
                Bool::from_node_id(id, ast)
            })))),
            NodeKind::CallExpr => {
                ForHead::Pat(Pat::Expr(Expr::Call({ CallExpr::from_node_id(id, ast) })))
            }
            NodeKind::ClassExpr => {
                ForHead::Pat(Pat::Expr(Expr::Class({ ClassExpr::from_node_id(id, ast) })))
            }
            NodeKind::CondExpr => {
                ForHead::Pat(Pat::Expr(Expr::Cond({ CondExpr::from_node_id(id, ast) })))
            }
            NodeKind::FnExpr => {
                ForHead::Pat(Pat::Expr(Expr::Fn({ FnExpr::from_node_id(id, ast) })))
            }
            NodeKind::Ident => {
                ForHead::Pat(Pat::Expr(Expr::Ident({ Ident::from_node_id(id, ast) })))
            }
            NodeKind::Invalid => ForHead::Pat(Pat::Invalid({ Invalid::from_node_id(id, ast) })),
            NodeKind::JSXElement => ForHead::Pat(Pat::Expr(Expr::JSXElement({
                JSXElement::from_node_id(id, ast)
            }))),
            NodeKind::JSXEmptyExpr => ForHead::Pat(Pat::Expr(Expr::JSXEmpty({
                JSXEmptyExpr::from_node_id(id, ast)
            }))),
            NodeKind::JSXFragment => ForHead::Pat(Pat::Expr(Expr::JSXFragment({
                JSXFragment::from_node_id(id, ast)
            }))),
            NodeKind::JSXMemberExpr => ForHead::Pat(Pat::Expr(Expr::JSXMember({
                JSXMemberExpr::from_node_id(id, ast)
            }))),
            NodeKind::JSXNamespacedName => ForHead::Pat(Pat::Expr(Expr::JSXNamespacedName({
                JSXNamespacedName::from_node_id(id, ast)
            }))),
            NodeKind::MemberExpr => ForHead::Pat(Pat::Expr(Expr::Member({
                MemberExpr::from_node_id(id, ast)
            }))),
            NodeKind::MetaPropExpr => ForHead::Pat(Pat::Expr(Expr::MetaProp({
                MetaPropExpr::from_node_id(id, ast)
            }))),
            NodeKind::NewExpr => {
                ForHead::Pat(Pat::Expr(Expr::New({ NewExpr::from_node_id(id, ast) })))
            }
            NodeKind::Null => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Null({
                Null::from_node_id(id, ast)
            })))),
            NodeKind::Number => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Num({
                Number::from_node_id(id, ast)
            })))),
            NodeKind::ObjectLit => ForHead::Pat(Pat::Expr(Expr::Object({
                ObjectLit::from_node_id(id, ast)
            }))),
            NodeKind::ObjectPat => ForHead::Pat(Pat::Object({ ObjectPat::from_node_id(id, ast) })),
            NodeKind::OptChainExpr => ForHead::Pat(Pat::Expr(Expr::OptChain({
                OptChainExpr::from_node_id(id, ast)
            }))),
            NodeKind::ParenExpr => {
                ForHead::Pat(Pat::Expr(Expr::Paren({ ParenExpr::from_node_id(id, ast) })))
            }
            NodeKind::PrivateName => ForHead::Pat(Pat::Expr(Expr::PrivateName({
                PrivateName::from_node_id(id, ast)
            }))),
            NodeKind::Regex => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Regex({
                Regex::from_node_id(id, ast)
            })))),
            NodeKind::RestPat => ForHead::Pat(Pat::Rest({ RestPat::from_node_id(id, ast) })),
            NodeKind::SeqExpr => {
                ForHead::Pat(Pat::Expr(Expr::Seq({ SeqExpr::from_node_id(id, ast) })))
            }
            NodeKind::Str => ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Str({
                Str::from_node_id(id, ast)
            })))),
            NodeKind::SuperPropExpr => ForHead::Pat(Pat::Expr(Expr::SuperProp({
                SuperPropExpr::from_node_id(id, ast)
            }))),
            NodeKind::TaggedTpl => ForHead::Pat(Pat::Expr(Expr::TaggedTpl({
                TaggedTpl::from_node_id(id, ast)
            }))),
            NodeKind::ThisExpr => {
                ForHead::Pat(Pat::Expr(Expr::This({ ThisExpr::from_node_id(id, ast) })))
            }
            NodeKind::Tpl => ForHead::Pat(Pat::Expr(Expr::Tpl({ Tpl::from_node_id(id, ast) }))),
            NodeKind::UnaryExpr => {
                ForHead::Pat(Pat::Expr(Expr::Unary({ UnaryExpr::from_node_id(id, ast) })))
            }
            NodeKind::UpdateExpr => ForHead::Pat(Pat::Expr(Expr::Update({
                UpdateExpr::from_node_id(id, ast)
            }))),
            NodeKind::UsingDecl => ForHead::UsingDecl({ UsingDecl::from_node_id(id, ast) }),
            NodeKind::VarDecl => ForHead::VarDecl({ VarDecl::from_node_id(id, ast) }),
            NodeKind::YieldExpr => {
                ForHead::Pat(Pat::Expr(Expr::Yield({ YieldExpr::from_node_id(id, ast) })))
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for VarDeclOrExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::VarDecl(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ArrayLit => {
                VarDeclOrExpr::Expr(Expr::Array({ ArrayLit::from_node_id(id, ast) }))
            }
            NodeKind::ArrowExpr => {
                VarDeclOrExpr::Expr(Expr::Arrow({ ArrowExpr::from_node_id(id, ast) }))
            }
            NodeKind::AssignExpr => {
                VarDeclOrExpr::Expr(Expr::Assign({ AssignExpr::from_node_id(id, ast) }))
            }
            NodeKind::AwaitExpr => {
                VarDeclOrExpr::Expr(Expr::Await({ AwaitExpr::from_node_id(id, ast) }))
            }
            NodeKind::BigInt => {
                VarDeclOrExpr::Expr(Expr::Lit(Lit::BigInt({ BigInt::from_node_id(id, ast) })))
            }
            NodeKind::BinExpr => VarDeclOrExpr::Expr(Expr::Bin({ BinExpr::from_node_id(id, ast) })),
            NodeKind::Bool => {
                VarDeclOrExpr::Expr(Expr::Lit(Lit::Bool({ Bool::from_node_id(id, ast) })))
            }
            NodeKind::CallExpr => {
                VarDeclOrExpr::Expr(Expr::Call({ CallExpr::from_node_id(id, ast) }))
            }
            NodeKind::ClassExpr => {
                VarDeclOrExpr::Expr(Expr::Class({ ClassExpr::from_node_id(id, ast) }))
            }
            NodeKind::CondExpr => {
                VarDeclOrExpr::Expr(Expr::Cond({ CondExpr::from_node_id(id, ast) }))
            }
            NodeKind::FnExpr => VarDeclOrExpr::Expr(Expr::Fn({ FnExpr::from_node_id(id, ast) })),
            NodeKind::Ident => VarDeclOrExpr::Expr(Expr::Ident({ Ident::from_node_id(id, ast) })),
            NodeKind::Invalid => {
                VarDeclOrExpr::Expr(Expr::Invalid({ Invalid::from_node_id(id, ast) }))
            }
            NodeKind::JSXElement => {
                VarDeclOrExpr::Expr(Expr::JSXElement({ JSXElement::from_node_id(id, ast) }))
            }
            NodeKind::JSXEmptyExpr => {
                VarDeclOrExpr::Expr(Expr::JSXEmpty({ JSXEmptyExpr::from_node_id(id, ast) }))
            }
            NodeKind::JSXFragment => {
                VarDeclOrExpr::Expr(Expr::JSXFragment({ JSXFragment::from_node_id(id, ast) }))
            }
            NodeKind::JSXMemberExpr => {
                VarDeclOrExpr::Expr(Expr::JSXMember({ JSXMemberExpr::from_node_id(id, ast) }))
            }
            NodeKind::JSXNamespacedName => VarDeclOrExpr::Expr(Expr::JSXNamespacedName({
                JSXNamespacedName::from_node_id(id, ast)
            })),
            NodeKind::MemberExpr => {
                VarDeclOrExpr::Expr(Expr::Member({ MemberExpr::from_node_id(id, ast) }))
            }
            NodeKind::MetaPropExpr => {
                VarDeclOrExpr::Expr(Expr::MetaProp({ MetaPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::NewExpr => VarDeclOrExpr::Expr(Expr::New({ NewExpr::from_node_id(id, ast) })),
            NodeKind::Null => {
                VarDeclOrExpr::Expr(Expr::Lit(Lit::Null({ Null::from_node_id(id, ast) })))
            }
            NodeKind::Number => {
                VarDeclOrExpr::Expr(Expr::Lit(Lit::Num({ Number::from_node_id(id, ast) })))
            }
            NodeKind::ObjectLit => {
                VarDeclOrExpr::Expr(Expr::Object({ ObjectLit::from_node_id(id, ast) }))
            }
            NodeKind::OptChainExpr => {
                VarDeclOrExpr::Expr(Expr::OptChain({ OptChainExpr::from_node_id(id, ast) }))
            }
            NodeKind::ParenExpr => {
                VarDeclOrExpr::Expr(Expr::Paren({ ParenExpr::from_node_id(id, ast) }))
            }
            NodeKind::PrivateName => {
                VarDeclOrExpr::Expr(Expr::PrivateName({ PrivateName::from_node_id(id, ast) }))
            }
            NodeKind::Regex => {
                VarDeclOrExpr::Expr(Expr::Lit(Lit::Regex({ Regex::from_node_id(id, ast) })))
            }
            NodeKind::SeqExpr => VarDeclOrExpr::Expr(Expr::Seq({ SeqExpr::from_node_id(id, ast) })),
            NodeKind::Str => {
                VarDeclOrExpr::Expr(Expr::Lit(Lit::Str({ Str::from_node_id(id, ast) })))
            }
            NodeKind::SuperPropExpr => {
                VarDeclOrExpr::Expr(Expr::SuperProp({ SuperPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::TaggedTpl => {
                VarDeclOrExpr::Expr(Expr::TaggedTpl({ TaggedTpl::from_node_id(id, ast) }))
            }
            NodeKind::ThisExpr => {
                VarDeclOrExpr::Expr(Expr::This({ ThisExpr::from_node_id(id, ast) }))
            }
            NodeKind::Tpl => VarDeclOrExpr::Expr(Expr::Tpl({ Tpl::from_node_id(id, ast) })),
            NodeKind::UnaryExpr => {
                VarDeclOrExpr::Expr(Expr::Unary({ UnaryExpr::from_node_id(id, ast) }))
            }
            NodeKind::UpdateExpr => {
                VarDeclOrExpr::Expr(Expr::Update({ UpdateExpr::from_node_id(id, ast) }))
            }
            NodeKind::VarDecl => VarDeclOrExpr::VarDecl({ VarDecl::from_node_id(id, ast) }),
            NodeKind::YieldExpr => {
                VarDeclOrExpr::Expr(Expr::Yield({ YieldExpr::from_node_id(id, ast) }))
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for Decl {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Class(it) => it.node_id(),
            Self::Fn(it) => it.node_id(),
            Self::Var(it) => it.node_id(),
            Self::Using(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ClassDecl => Decl::Class({ ClassDecl::from_node_id(id, ast) }),
            NodeKind::FnDecl => Decl::Fn({ FnDecl::from_node_id(id, ast) }),
            NodeKind::UsingDecl => Decl::Using({ UsingDecl::from_node_id(id, ast) }),
            NodeKind::VarDecl => Decl::Var({ VarDecl::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for FnDecl {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::FnDecl);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ClassDecl {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ClassDecl);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for VarDecl {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::VarDecl);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for VarDeclarator {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::VarDeclarator);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for UsingDecl {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::UsingDecl);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Expr {
    #[inline]
    fn node_id(&self) -> NodeId {
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
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ArrayLit => Expr::Array({ ArrayLit::from_node_id(id, ast) }),
            NodeKind::ArrowExpr => Expr::Arrow({ ArrowExpr::from_node_id(id, ast) }),
            NodeKind::AssignExpr => Expr::Assign({ AssignExpr::from_node_id(id, ast) }),
            NodeKind::AwaitExpr => Expr::Await({ AwaitExpr::from_node_id(id, ast) }),
            NodeKind::BigInt => Expr::Lit(Lit::BigInt({ BigInt::from_node_id(id, ast) })),
            NodeKind::BinExpr => Expr::Bin({ BinExpr::from_node_id(id, ast) }),
            NodeKind::Bool => Expr::Lit(Lit::Bool({ Bool::from_node_id(id, ast) })),
            NodeKind::CallExpr => Expr::Call({ CallExpr::from_node_id(id, ast) }),
            NodeKind::ClassExpr => Expr::Class({ ClassExpr::from_node_id(id, ast) }),
            NodeKind::CondExpr => Expr::Cond({ CondExpr::from_node_id(id, ast) }),
            NodeKind::FnExpr => Expr::Fn({ FnExpr::from_node_id(id, ast) }),
            NodeKind::Ident => Expr::Ident({ Ident::from_node_id(id, ast) }),
            NodeKind::Invalid => Expr::Invalid({ Invalid::from_node_id(id, ast) }),
            NodeKind::JSXElement => Expr::JSXElement({ JSXElement::from_node_id(id, ast) }),
            NodeKind::JSXEmptyExpr => Expr::JSXEmpty({ JSXEmptyExpr::from_node_id(id, ast) }),
            NodeKind::JSXFragment => Expr::JSXFragment({ JSXFragment::from_node_id(id, ast) }),
            NodeKind::JSXMemberExpr => Expr::JSXMember({ JSXMemberExpr::from_node_id(id, ast) }),
            NodeKind::JSXNamespacedName => {
                Expr::JSXNamespacedName({ JSXNamespacedName::from_node_id(id, ast) })
            }
            NodeKind::MemberExpr => Expr::Member({ MemberExpr::from_node_id(id, ast) }),
            NodeKind::MetaPropExpr => Expr::MetaProp({ MetaPropExpr::from_node_id(id, ast) }),
            NodeKind::NewExpr => Expr::New({ NewExpr::from_node_id(id, ast) }),
            NodeKind::Null => Expr::Lit(Lit::Null({ Null::from_node_id(id, ast) })),
            NodeKind::Number => Expr::Lit(Lit::Num({ Number::from_node_id(id, ast) })),
            NodeKind::ObjectLit => Expr::Object({ ObjectLit::from_node_id(id, ast) }),
            NodeKind::OptChainExpr => Expr::OptChain({ OptChainExpr::from_node_id(id, ast) }),
            NodeKind::ParenExpr => Expr::Paren({ ParenExpr::from_node_id(id, ast) }),
            NodeKind::PrivateName => Expr::PrivateName({ PrivateName::from_node_id(id, ast) }),
            NodeKind::Regex => Expr::Lit(Lit::Regex({ Regex::from_node_id(id, ast) })),
            NodeKind::SeqExpr => Expr::Seq({ SeqExpr::from_node_id(id, ast) }),
            NodeKind::Str => Expr::Lit(Lit::Str({ Str::from_node_id(id, ast) })),
            NodeKind::SuperPropExpr => Expr::SuperProp({ SuperPropExpr::from_node_id(id, ast) }),
            NodeKind::TaggedTpl => Expr::TaggedTpl({ TaggedTpl::from_node_id(id, ast) }),
            NodeKind::ThisExpr => Expr::This({ ThisExpr::from_node_id(id, ast) }),
            NodeKind::Tpl => Expr::Tpl({ Tpl::from_node_id(id, ast) }),
            NodeKind::UnaryExpr => Expr::Unary({ UnaryExpr::from_node_id(id, ast) }),
            NodeKind::UpdateExpr => Expr::Update({ UpdateExpr::from_node_id(id, ast) }),
            NodeKind::YieldExpr => Expr::Yield({ YieldExpr::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ThisExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ThisExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ArrayLit {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ArrayLit);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ObjectLit {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ObjectLit);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PropOrSpread {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::SpreadElement(it) => it.node_id(),
            Self::Prop(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::AssignProp => {
                PropOrSpread::Prop(Prop::Assign({ AssignProp::from_node_id(id, ast) }))
            }
            NodeKind::GetterProp => {
                PropOrSpread::Prop(Prop::Getter({ GetterProp::from_node_id(id, ast) }))
            }
            NodeKind::Ident => {
                PropOrSpread::Prop(Prop::Shorthand({ Ident::from_node_id(id, ast) }))
            }
            NodeKind::KeyValueProp => {
                PropOrSpread::Prop(Prop::KeyValue({ KeyValueProp::from_node_id(id, ast) }))
            }
            NodeKind::MethodProp => {
                PropOrSpread::Prop(Prop::Method({ MethodProp::from_node_id(id, ast) }))
            }
            NodeKind::SetterProp => {
                PropOrSpread::Prop(Prop::Setter({ SetterProp::from_node_id(id, ast) }))
            }
            NodeKind::SpreadElement => {
                PropOrSpread::SpreadElement({ SpreadElement::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for SpreadElement {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::SpreadElement);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for UnaryExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::UnaryExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for UpdateExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::UpdateExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BinExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::BinExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for FnExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::FnExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ClassExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ClassExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AssignExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::AssignExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for MemberExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::MemberExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for MemberProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::PrivateName(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ComputedPropName => {
                MemberProp::Computed({ ComputedPropName::from_node_id(id, ast) })
            }
            NodeKind::IdentName => MemberProp::Ident({ IdentName::from_node_id(id, ast) }),
            NodeKind::PrivateName => {
                MemberProp::PrivateName({ PrivateName::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for SuperPropExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::SuperPropExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SuperProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ComputedPropName => {
                SuperProp::Computed({ ComputedPropName::from_node_id(id, ast) })
            }
            NodeKind::IdentName => SuperProp::Ident({ IdentName::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for CondExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::CondExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for CallExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::CallExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for NewExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::NewExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SeqExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::SeqExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ArrowExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ArrowExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for YieldExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::YieldExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for MetaPropExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::MetaPropExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AwaitExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::AwaitExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Tpl {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Tpl);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for TaggedTpl {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::TaggedTpl);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for TplElement {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::TplElement);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ParenExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ParenExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Callee {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Super(it) => it.node_id(),
            Self::Import(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ArrayLit => Callee::Expr(Expr::Array({ ArrayLit::from_node_id(id, ast) })),
            NodeKind::ArrowExpr => Callee::Expr(Expr::Arrow({ ArrowExpr::from_node_id(id, ast) })),
            NodeKind::AssignExpr => {
                Callee::Expr(Expr::Assign({ AssignExpr::from_node_id(id, ast) }))
            }
            NodeKind::AwaitExpr => Callee::Expr(Expr::Await({ AwaitExpr::from_node_id(id, ast) })),
            NodeKind::BigInt => {
                Callee::Expr(Expr::Lit(Lit::BigInt({ BigInt::from_node_id(id, ast) })))
            }
            NodeKind::BinExpr => Callee::Expr(Expr::Bin({ BinExpr::from_node_id(id, ast) })),
            NodeKind::Bool => Callee::Expr(Expr::Lit(Lit::Bool({ Bool::from_node_id(id, ast) }))),
            NodeKind::CallExpr => Callee::Expr(Expr::Call({ CallExpr::from_node_id(id, ast) })),
            NodeKind::ClassExpr => Callee::Expr(Expr::Class({ ClassExpr::from_node_id(id, ast) })),
            NodeKind::CondExpr => Callee::Expr(Expr::Cond({ CondExpr::from_node_id(id, ast) })),
            NodeKind::FnExpr => Callee::Expr(Expr::Fn({ FnExpr::from_node_id(id, ast) })),
            NodeKind::Ident => Callee::Expr(Expr::Ident({ Ident::from_node_id(id, ast) })),
            NodeKind::Import => Callee::Import({ Import::from_node_id(id, ast) }),
            NodeKind::Invalid => Callee::Expr(Expr::Invalid({ Invalid::from_node_id(id, ast) })),
            NodeKind::JSXElement => {
                Callee::Expr(Expr::JSXElement({ JSXElement::from_node_id(id, ast) }))
            }
            NodeKind::JSXEmptyExpr => {
                Callee::Expr(Expr::JSXEmpty({ JSXEmptyExpr::from_node_id(id, ast) }))
            }
            NodeKind::JSXFragment => {
                Callee::Expr(Expr::JSXFragment({ JSXFragment::from_node_id(id, ast) }))
            }
            NodeKind::JSXMemberExpr => {
                Callee::Expr(Expr::JSXMember({ JSXMemberExpr::from_node_id(id, ast) }))
            }
            NodeKind::JSXNamespacedName => Callee::Expr(Expr::JSXNamespacedName({
                JSXNamespacedName::from_node_id(id, ast)
            })),
            NodeKind::MemberExpr => {
                Callee::Expr(Expr::Member({ MemberExpr::from_node_id(id, ast) }))
            }
            NodeKind::MetaPropExpr => {
                Callee::Expr(Expr::MetaProp({ MetaPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::NewExpr => Callee::Expr(Expr::New({ NewExpr::from_node_id(id, ast) })),
            NodeKind::Null => Callee::Expr(Expr::Lit(Lit::Null({ Null::from_node_id(id, ast) }))),
            NodeKind::Number => {
                Callee::Expr(Expr::Lit(Lit::Num({ Number::from_node_id(id, ast) })))
            }
            NodeKind::ObjectLit => Callee::Expr(Expr::Object({ ObjectLit::from_node_id(id, ast) })),
            NodeKind::OptChainExpr => {
                Callee::Expr(Expr::OptChain({ OptChainExpr::from_node_id(id, ast) }))
            }
            NodeKind::ParenExpr => Callee::Expr(Expr::Paren({ ParenExpr::from_node_id(id, ast) })),
            NodeKind::PrivateName => {
                Callee::Expr(Expr::PrivateName({ PrivateName::from_node_id(id, ast) }))
            }
            NodeKind::Regex => {
                Callee::Expr(Expr::Lit(Lit::Regex({ Regex::from_node_id(id, ast) })))
            }
            NodeKind::SeqExpr => Callee::Expr(Expr::Seq({ SeqExpr::from_node_id(id, ast) })),
            NodeKind::Str => Callee::Expr(Expr::Lit(Lit::Str({ Str::from_node_id(id, ast) }))),
            NodeKind::Super => Callee::Super({ Super::from_node_id(id, ast) }),
            NodeKind::SuperPropExpr => {
                Callee::Expr(Expr::SuperProp({ SuperPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::TaggedTpl => {
                Callee::Expr(Expr::TaggedTpl({ TaggedTpl::from_node_id(id, ast) }))
            }
            NodeKind::ThisExpr => Callee::Expr(Expr::This({ ThisExpr::from_node_id(id, ast) })),
            NodeKind::Tpl => Callee::Expr(Expr::Tpl({ Tpl::from_node_id(id, ast) })),
            NodeKind::UnaryExpr => Callee::Expr(Expr::Unary({ UnaryExpr::from_node_id(id, ast) })),
            NodeKind::UpdateExpr => {
                Callee::Expr(Expr::Update({ UpdateExpr::from_node_id(id, ast) }))
            }
            NodeKind::YieldExpr => Callee::Expr(Expr::Yield({ YieldExpr::from_node_id(id, ast) })),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for Super {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Super);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Import {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Import);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ExprOrSpread {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ExprOrSpread);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SpreadDot3Token {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::SpreadDot3Token);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BlockStmtOrExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::BlockStmt(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ArrayLit => {
                BlockStmtOrExpr::Expr(Expr::Array({ ArrayLit::from_node_id(id, ast) }))
            }
            NodeKind::ArrowExpr => {
                BlockStmtOrExpr::Expr(Expr::Arrow({ ArrowExpr::from_node_id(id, ast) }))
            }
            NodeKind::AssignExpr => {
                BlockStmtOrExpr::Expr(Expr::Assign({ AssignExpr::from_node_id(id, ast) }))
            }
            NodeKind::AwaitExpr => {
                BlockStmtOrExpr::Expr(Expr::Await({ AwaitExpr::from_node_id(id, ast) }))
            }
            NodeKind::BigInt => {
                BlockStmtOrExpr::Expr(Expr::Lit(Lit::BigInt({ BigInt::from_node_id(id, ast) })))
            }
            NodeKind::BinExpr => {
                BlockStmtOrExpr::Expr(Expr::Bin({ BinExpr::from_node_id(id, ast) }))
            }
            NodeKind::BlockStmt => BlockStmtOrExpr::BlockStmt({ BlockStmt::from_node_id(id, ast) }),
            NodeKind::Bool => {
                BlockStmtOrExpr::Expr(Expr::Lit(Lit::Bool({ Bool::from_node_id(id, ast) })))
            }
            NodeKind::CallExpr => {
                BlockStmtOrExpr::Expr(Expr::Call({ CallExpr::from_node_id(id, ast) }))
            }
            NodeKind::ClassExpr => {
                BlockStmtOrExpr::Expr(Expr::Class({ ClassExpr::from_node_id(id, ast) }))
            }
            NodeKind::CondExpr => {
                BlockStmtOrExpr::Expr(Expr::Cond({ CondExpr::from_node_id(id, ast) }))
            }
            NodeKind::FnExpr => BlockStmtOrExpr::Expr(Expr::Fn({ FnExpr::from_node_id(id, ast) })),
            NodeKind::Ident => BlockStmtOrExpr::Expr(Expr::Ident({ Ident::from_node_id(id, ast) })),
            NodeKind::Invalid => {
                BlockStmtOrExpr::Expr(Expr::Invalid({ Invalid::from_node_id(id, ast) }))
            }
            NodeKind::JSXElement => {
                BlockStmtOrExpr::Expr(Expr::JSXElement({ JSXElement::from_node_id(id, ast) }))
            }
            NodeKind::JSXEmptyExpr => {
                BlockStmtOrExpr::Expr(Expr::JSXEmpty({ JSXEmptyExpr::from_node_id(id, ast) }))
            }
            NodeKind::JSXFragment => {
                BlockStmtOrExpr::Expr(Expr::JSXFragment({ JSXFragment::from_node_id(id, ast) }))
            }
            NodeKind::JSXMemberExpr => {
                BlockStmtOrExpr::Expr(Expr::JSXMember({ JSXMemberExpr::from_node_id(id, ast) }))
            }
            NodeKind::JSXNamespacedName => BlockStmtOrExpr::Expr(Expr::JSXNamespacedName({
                JSXNamespacedName::from_node_id(id, ast)
            })),
            NodeKind::MemberExpr => {
                BlockStmtOrExpr::Expr(Expr::Member({ MemberExpr::from_node_id(id, ast) }))
            }
            NodeKind::MetaPropExpr => {
                BlockStmtOrExpr::Expr(Expr::MetaProp({ MetaPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::NewExpr => {
                BlockStmtOrExpr::Expr(Expr::New({ NewExpr::from_node_id(id, ast) }))
            }
            NodeKind::Null => {
                BlockStmtOrExpr::Expr(Expr::Lit(Lit::Null({ Null::from_node_id(id, ast) })))
            }
            NodeKind::Number => {
                BlockStmtOrExpr::Expr(Expr::Lit(Lit::Num({ Number::from_node_id(id, ast) })))
            }
            NodeKind::ObjectLit => {
                BlockStmtOrExpr::Expr(Expr::Object({ ObjectLit::from_node_id(id, ast) }))
            }
            NodeKind::OptChainExpr => {
                BlockStmtOrExpr::Expr(Expr::OptChain({ OptChainExpr::from_node_id(id, ast) }))
            }
            NodeKind::ParenExpr => {
                BlockStmtOrExpr::Expr(Expr::Paren({ ParenExpr::from_node_id(id, ast) }))
            }
            NodeKind::PrivateName => {
                BlockStmtOrExpr::Expr(Expr::PrivateName({ PrivateName::from_node_id(id, ast) }))
            }
            NodeKind::Regex => {
                BlockStmtOrExpr::Expr(Expr::Lit(Lit::Regex({ Regex::from_node_id(id, ast) })))
            }
            NodeKind::SeqExpr => {
                BlockStmtOrExpr::Expr(Expr::Seq({ SeqExpr::from_node_id(id, ast) }))
            }
            NodeKind::Str => {
                BlockStmtOrExpr::Expr(Expr::Lit(Lit::Str({ Str::from_node_id(id, ast) })))
            }
            NodeKind::SuperPropExpr => {
                BlockStmtOrExpr::Expr(Expr::SuperProp({ SuperPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::TaggedTpl => {
                BlockStmtOrExpr::Expr(Expr::TaggedTpl({ TaggedTpl::from_node_id(id, ast) }))
            }
            NodeKind::ThisExpr => {
                BlockStmtOrExpr::Expr(Expr::This({ ThisExpr::from_node_id(id, ast) }))
            }
            NodeKind::Tpl => BlockStmtOrExpr::Expr(Expr::Tpl({ Tpl::from_node_id(id, ast) })),
            NodeKind::UnaryExpr => {
                BlockStmtOrExpr::Expr(Expr::Unary({ UnaryExpr::from_node_id(id, ast) }))
            }
            NodeKind::UpdateExpr => {
                BlockStmtOrExpr::Expr(Expr::Update({ UpdateExpr::from_node_id(id, ast) }))
            }
            NodeKind::YieldExpr => {
                BlockStmtOrExpr::Expr(Expr::Yield({ YieldExpr::from_node_id(id, ast) }))
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for AssignTarget {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Simple(it) => it.node_id(),
            Self::Pat(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ArrayPat => {
                AssignTarget::Pat(AssignTargetPat::Array({ ArrayPat::from_node_id(id, ast) }))
            }
            NodeKind::BindingIdent => AssignTarget::Simple(SimpleAssignTarget::Ident({
                BindingIdent::from_node_id(id, ast)
            })),
            NodeKind::Invalid => AssignTarget::Simple(SimpleAssignTarget::Invalid({
                Invalid::from_node_id(id, ast)
            })),
            NodeKind::MemberExpr => AssignTarget::Simple(SimpleAssignTarget::Member({
                MemberExpr::from_node_id(id, ast)
            })),
            NodeKind::ObjectPat => AssignTarget::Pat(AssignTargetPat::Object({
                ObjectPat::from_node_id(id, ast)
            })),
            NodeKind::OptChainExpr => AssignTarget::Simple(SimpleAssignTarget::OptChain({
                OptChainExpr::from_node_id(id, ast)
            })),
            NodeKind::ParenExpr => AssignTarget::Simple(SimpleAssignTarget::Paren({
                ParenExpr::from_node_id(id, ast)
            })),
            NodeKind::SuperPropExpr => AssignTarget::Simple(SimpleAssignTarget::SuperProp({
                SuperPropExpr::from_node_id(id, ast)
            })),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for AssignTargetPat {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Array(it) => it.node_id(),
            Self::Object(it) => it.node_id(),
            Self::Invalid(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ArrayPat => AssignTargetPat::Array({ ArrayPat::from_node_id(id, ast) }),
            NodeKind::Invalid => AssignTargetPat::Invalid({ Invalid::from_node_id(id, ast) }),
            NodeKind::ObjectPat => AssignTargetPat::Object({ ObjectPat::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for SimpleAssignTarget {
    #[inline]
    fn node_id(&self) -> NodeId {
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
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::BindingIdent => {
                SimpleAssignTarget::Ident({ BindingIdent::from_node_id(id, ast) })
            }
            NodeKind::Invalid => SimpleAssignTarget::Invalid({ Invalid::from_node_id(id, ast) }),
            NodeKind::MemberExpr => {
                SimpleAssignTarget::Member({ MemberExpr::from_node_id(id, ast) })
            }
            NodeKind::OptChainExpr => {
                SimpleAssignTarget::OptChain({ OptChainExpr::from_node_id(id, ast) })
            }
            NodeKind::ParenExpr => SimpleAssignTarget::Paren({ ParenExpr::from_node_id(id, ast) }),
            NodeKind::SuperPropExpr => {
                SimpleAssignTarget::SuperProp({ SuperPropExpr::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for OptChainExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::OptChainExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for OptChainBase {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Member(it) => it.node_id(),
            Self::Call(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::MemberExpr => OptChainBase::Member({ MemberExpr::from_node_id(id, ast) }),
            NodeKind::OptCall => OptChainBase::Call({ OptCall::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for OptCall {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::OptCall);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Invalid {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Invalid);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Function {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Function);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Param {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Param);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ParamOrTsParamProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Param(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::Param => ParamOrTsParamProp::Param({ Param::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for Class {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Class);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ClassMember {
    #[inline]
    fn node_id(&self) -> NodeId {
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
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::AutoAccessor => {
                ClassMember::AutoAccessor({ AutoAccessor::from_node_id(id, ast) })
            }
            NodeKind::ClassMethod => ClassMember::Method({ ClassMethod::from_node_id(id, ast) }),
            NodeKind::ClassProp => ClassMember::ClassProp({ ClassProp::from_node_id(id, ast) }),
            NodeKind::Constructor => {
                ClassMember::Constructor({ Constructor::from_node_id(id, ast) })
            }
            NodeKind::EmptyStmt => ClassMember::Empty({ EmptyStmt::from_node_id(id, ast) }),
            NodeKind::PrivateMethod => {
                ClassMember::PrivateMethod({ PrivateMethod::from_node_id(id, ast) })
            }
            NodeKind::PrivateProp => {
                ClassMember::PrivateProp({ PrivateProp::from_node_id(id, ast) })
            }
            NodeKind::StaticBlock => {
                ClassMember::StaticBlock({ StaticBlock::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ClassProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ClassProp);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PrivateProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::PrivateProp);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ClassMethod {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ClassMethod);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PrivateMethod {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::PrivateMethod);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Constructor {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Constructor);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Decorator {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Decorator);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for StaticBlock {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::StaticBlock);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Key {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Private(it) => it.node_id(),
            Self::Public(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::BigInt => Key::Public(PropName::BigInt({ BigInt::from_node_id(id, ast) })),
            NodeKind::ComputedPropName => Key::Public(PropName::Computed({
                ComputedPropName::from_node_id(id, ast)
            })),
            NodeKind::IdentName => {
                Key::Public(PropName::Ident({ IdentName::from_node_id(id, ast) }))
            }
            NodeKind::Number => Key::Public(PropName::Num({ Number::from_node_id(id, ast) })),
            NodeKind::PrivateName => Key::Private({ PrivateName::from_node_id(id, ast) }),
            NodeKind::Str => Key::Public(PropName::Str({ Str::from_node_id(id, ast) })),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for AutoAccessor {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::AutoAccessor);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Prop {
    #[inline]
    fn node_id(&self) -> NodeId {
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
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::AssignProp => Prop::Assign({ AssignProp::from_node_id(id, ast) }),
            NodeKind::GetterProp => Prop::Getter({ GetterProp::from_node_id(id, ast) }),
            NodeKind::Ident => Prop::Shorthand({ Ident::from_node_id(id, ast) }),
            NodeKind::KeyValueProp => Prop::KeyValue({ KeyValueProp::from_node_id(id, ast) }),
            NodeKind::MethodProp => Prop::Method({ MethodProp::from_node_id(id, ast) }),
            NodeKind::SetterProp => Prop::Setter({ SetterProp::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for KeyValueProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::KeyValueProp);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AssignProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::AssignProp);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for GetterProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::GetterProp);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for SetterProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::SetterProp);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for MethodProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::MethodProp);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PropName {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::Str(it) => it.node_id(),
            Self::Num(it) => it.node_id(),
            Self::Computed(it) => it.node_id(),
            Self::BigInt(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::BigInt => PropName::BigInt({ BigInt::from_node_id(id, ast) }),
            NodeKind::ComputedPropName => {
                PropName::Computed({ ComputedPropName::from_node_id(id, ast) })
            }
            NodeKind::IdentName => PropName::Ident({ IdentName::from_node_id(id, ast) }),
            NodeKind::Number => PropName::Num({ Number::from_node_id(id, ast) }),
            NodeKind::Str => PropName::Str({ Str::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ComputedPropName {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ComputedPropName);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Pat {
    #[inline]
    fn node_id(&self) -> NodeId {
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
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ArrayLit => Pat::Expr(Expr::Array({ ArrayLit::from_node_id(id, ast) })),
            NodeKind::ArrayPat => Pat::Array({ ArrayPat::from_node_id(id, ast) }),
            NodeKind::ArrowExpr => Pat::Expr(Expr::Arrow({ ArrowExpr::from_node_id(id, ast) })),
            NodeKind::AssignExpr => Pat::Expr(Expr::Assign({ AssignExpr::from_node_id(id, ast) })),
            NodeKind::AssignPat => Pat::Assign({ AssignPat::from_node_id(id, ast) }),
            NodeKind::AwaitExpr => Pat::Expr(Expr::Await({ AwaitExpr::from_node_id(id, ast) })),
            NodeKind::BigInt => {
                Pat::Expr(Expr::Lit(Lit::BigInt({ BigInt::from_node_id(id, ast) })))
            }
            NodeKind::BinExpr => Pat::Expr(Expr::Bin({ BinExpr::from_node_id(id, ast) })),
            NodeKind::BindingIdent => Pat::Ident({ BindingIdent::from_node_id(id, ast) }),
            NodeKind::Bool => Pat::Expr(Expr::Lit(Lit::Bool({ Bool::from_node_id(id, ast) }))),
            NodeKind::CallExpr => Pat::Expr(Expr::Call({ CallExpr::from_node_id(id, ast) })),
            NodeKind::ClassExpr => Pat::Expr(Expr::Class({ ClassExpr::from_node_id(id, ast) })),
            NodeKind::CondExpr => Pat::Expr(Expr::Cond({ CondExpr::from_node_id(id, ast) })),
            NodeKind::FnExpr => Pat::Expr(Expr::Fn({ FnExpr::from_node_id(id, ast) })),
            NodeKind::Ident => Pat::Expr(Expr::Ident({ Ident::from_node_id(id, ast) })),
            NodeKind::Invalid => Pat::Invalid({ Invalid::from_node_id(id, ast) }),
            NodeKind::JSXElement => {
                Pat::Expr(Expr::JSXElement({ JSXElement::from_node_id(id, ast) }))
            }
            NodeKind::JSXEmptyExpr => {
                Pat::Expr(Expr::JSXEmpty({ JSXEmptyExpr::from_node_id(id, ast) }))
            }
            NodeKind::JSXFragment => {
                Pat::Expr(Expr::JSXFragment({ JSXFragment::from_node_id(id, ast) }))
            }
            NodeKind::JSXMemberExpr => {
                Pat::Expr(Expr::JSXMember({ JSXMemberExpr::from_node_id(id, ast) }))
            }
            NodeKind::JSXNamespacedName => Pat::Expr(Expr::JSXNamespacedName({
                JSXNamespacedName::from_node_id(id, ast)
            })),
            NodeKind::MemberExpr => Pat::Expr(Expr::Member({ MemberExpr::from_node_id(id, ast) })),
            NodeKind::MetaPropExpr => {
                Pat::Expr(Expr::MetaProp({ MetaPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::NewExpr => Pat::Expr(Expr::New({ NewExpr::from_node_id(id, ast) })),
            NodeKind::Null => Pat::Expr(Expr::Lit(Lit::Null({ Null::from_node_id(id, ast) }))),
            NodeKind::Number => Pat::Expr(Expr::Lit(Lit::Num({ Number::from_node_id(id, ast) }))),
            NodeKind::ObjectLit => Pat::Expr(Expr::Object({ ObjectLit::from_node_id(id, ast) })),
            NodeKind::ObjectPat => Pat::Object({ ObjectPat::from_node_id(id, ast) }),
            NodeKind::OptChainExpr => {
                Pat::Expr(Expr::OptChain({ OptChainExpr::from_node_id(id, ast) }))
            }
            NodeKind::ParenExpr => Pat::Expr(Expr::Paren({ ParenExpr::from_node_id(id, ast) })),
            NodeKind::PrivateName => {
                Pat::Expr(Expr::PrivateName({ PrivateName::from_node_id(id, ast) }))
            }
            NodeKind::Regex => Pat::Expr(Expr::Lit(Lit::Regex({ Regex::from_node_id(id, ast) }))),
            NodeKind::RestPat => Pat::Rest({ RestPat::from_node_id(id, ast) }),
            NodeKind::SeqExpr => Pat::Expr(Expr::Seq({ SeqExpr::from_node_id(id, ast) })),
            NodeKind::Str => Pat::Expr(Expr::Lit(Lit::Str({ Str::from_node_id(id, ast) }))),
            NodeKind::SuperPropExpr => {
                Pat::Expr(Expr::SuperProp({ SuperPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::TaggedTpl => Pat::Expr(Expr::TaggedTpl({ TaggedTpl::from_node_id(id, ast) })),
            NodeKind::ThisExpr => Pat::Expr(Expr::This({ ThisExpr::from_node_id(id, ast) })),
            NodeKind::Tpl => Pat::Expr(Expr::Tpl({ Tpl::from_node_id(id, ast) })),
            NodeKind::UnaryExpr => Pat::Expr(Expr::Unary({ UnaryExpr::from_node_id(id, ast) })),
            NodeKind::UpdateExpr => Pat::Expr(Expr::Update({ UpdateExpr::from_node_id(id, ast) })),
            NodeKind::YieldExpr => Pat::Expr(Expr::Yield({ YieldExpr::from_node_id(id, ast) })),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for ArrayPat {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ArrayPat);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ObjectPat {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::ObjectPat);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AssignPat {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::AssignPat);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for RestPat {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::RestPat);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for ObjectPatProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::KeyValue(it) => it.node_id(),
            Self::Assign(it) => it.node_id(),
            Self::Rest(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::AssignPatProp => {
                ObjectPatProp::Assign({ AssignPatProp::from_node_id(id, ast) })
            }
            NodeKind::KeyValuePatProp => {
                ObjectPatProp::KeyValue({ KeyValuePatProp::from_node_id(id, ast) })
            }
            NodeKind::RestPat => ObjectPatProp::Rest({ RestPat::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for KeyValuePatProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::KeyValuePatProp);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for AssignPatProp {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::AssignPatProp);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Ident {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Ident);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for IdentName {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::IdentName);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for PrivateName {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::PrivateName);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BindingIdent {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::BindingIdent);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Lit {
    #[inline]
    fn node_id(&self) -> NodeId {
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
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::BigInt => Lit::BigInt({ BigInt::from_node_id(id, ast) }),
            NodeKind::Bool => Lit::Bool({ Bool::from_node_id(id, ast) }),
            NodeKind::Null => Lit::Null({ Null::from_node_id(id, ast) }),
            NodeKind::Number => Lit::Num({ Number::from_node_id(id, ast) }),
            NodeKind::Regex => Lit::Regex({ Regex::from_node_id(id, ast) }),
            NodeKind::Str => Lit::Str({ Str::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for Str {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Str);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Bool {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Bool);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Null {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Null);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Number {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Number);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for BigInt {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::BigInt);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for Regex {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::Regex);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXObject {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::JSXMemberExpr(it) => it.node_id(),
            Self::Ident(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::Ident => JSXObject::Ident({ Ident::from_node_id(id, ast) }),
            NodeKind::JSXMemberExpr => {
                JSXObject::JSXMemberExpr({ JSXMemberExpr::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for JSXMemberExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXMemberExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXNamespacedName {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXNamespacedName);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXEmptyExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXEmptyExpr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXExprContainer {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXExprContainer);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXExpr {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::JSXEmptyExpr(it) => it.node_id(),
            Self::Expr(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::ArrayLit => JSXExpr::Expr(Expr::Array({ ArrayLit::from_node_id(id, ast) })),
            NodeKind::ArrowExpr => JSXExpr::Expr(Expr::Arrow({ ArrowExpr::from_node_id(id, ast) })),
            NodeKind::AssignExpr => {
                JSXExpr::Expr(Expr::Assign({ AssignExpr::from_node_id(id, ast) }))
            }
            NodeKind::AwaitExpr => JSXExpr::Expr(Expr::Await({ AwaitExpr::from_node_id(id, ast) })),
            NodeKind::BigInt => {
                JSXExpr::Expr(Expr::Lit(Lit::BigInt({ BigInt::from_node_id(id, ast) })))
            }
            NodeKind::BinExpr => JSXExpr::Expr(Expr::Bin({ BinExpr::from_node_id(id, ast) })),
            NodeKind::Bool => JSXExpr::Expr(Expr::Lit(Lit::Bool({ Bool::from_node_id(id, ast) }))),
            NodeKind::CallExpr => JSXExpr::Expr(Expr::Call({ CallExpr::from_node_id(id, ast) })),
            NodeKind::ClassExpr => JSXExpr::Expr(Expr::Class({ ClassExpr::from_node_id(id, ast) })),
            NodeKind::CondExpr => JSXExpr::Expr(Expr::Cond({ CondExpr::from_node_id(id, ast) })),
            NodeKind::FnExpr => JSXExpr::Expr(Expr::Fn({ FnExpr::from_node_id(id, ast) })),
            NodeKind::Ident => JSXExpr::Expr(Expr::Ident({ Ident::from_node_id(id, ast) })),
            NodeKind::Invalid => JSXExpr::Expr(Expr::Invalid({ Invalid::from_node_id(id, ast) })),
            NodeKind::JSXElement => {
                JSXExpr::Expr(Expr::JSXElement({ JSXElement::from_node_id(id, ast) }))
            }
            NodeKind::JSXEmptyExpr => {
                JSXExpr::JSXEmptyExpr({ JSXEmptyExpr::from_node_id(id, ast) })
            }
            NodeKind::JSXFragment => {
                JSXExpr::Expr(Expr::JSXFragment({ JSXFragment::from_node_id(id, ast) }))
            }
            NodeKind::JSXMemberExpr => {
                JSXExpr::Expr(Expr::JSXMember({ JSXMemberExpr::from_node_id(id, ast) }))
            }
            NodeKind::JSXNamespacedName => JSXExpr::Expr(Expr::JSXNamespacedName({
                JSXNamespacedName::from_node_id(id, ast)
            })),
            NodeKind::MemberExpr => {
                JSXExpr::Expr(Expr::Member({ MemberExpr::from_node_id(id, ast) }))
            }
            NodeKind::MetaPropExpr => {
                JSXExpr::Expr(Expr::MetaProp({ MetaPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::NewExpr => JSXExpr::Expr(Expr::New({ NewExpr::from_node_id(id, ast) })),
            NodeKind::Null => JSXExpr::Expr(Expr::Lit(Lit::Null({ Null::from_node_id(id, ast) }))),
            NodeKind::Number => {
                JSXExpr::Expr(Expr::Lit(Lit::Num({ Number::from_node_id(id, ast) })))
            }
            NodeKind::ObjectLit => {
                JSXExpr::Expr(Expr::Object({ ObjectLit::from_node_id(id, ast) }))
            }
            NodeKind::OptChainExpr => {
                JSXExpr::Expr(Expr::OptChain({ OptChainExpr::from_node_id(id, ast) }))
            }
            NodeKind::ParenExpr => JSXExpr::Expr(Expr::Paren({ ParenExpr::from_node_id(id, ast) })),
            NodeKind::PrivateName => {
                JSXExpr::Expr(Expr::PrivateName({ PrivateName::from_node_id(id, ast) }))
            }
            NodeKind::Regex => {
                JSXExpr::Expr(Expr::Lit(Lit::Regex({ Regex::from_node_id(id, ast) })))
            }
            NodeKind::SeqExpr => JSXExpr::Expr(Expr::Seq({ SeqExpr::from_node_id(id, ast) })),
            NodeKind::Str => JSXExpr::Expr(Expr::Lit(Lit::Str({ Str::from_node_id(id, ast) }))),
            NodeKind::SuperPropExpr => {
                JSXExpr::Expr(Expr::SuperProp({ SuperPropExpr::from_node_id(id, ast) }))
            }
            NodeKind::TaggedTpl => {
                JSXExpr::Expr(Expr::TaggedTpl({ TaggedTpl::from_node_id(id, ast) }))
            }
            NodeKind::ThisExpr => JSXExpr::Expr(Expr::This({ ThisExpr::from_node_id(id, ast) })),
            NodeKind::Tpl => JSXExpr::Expr(Expr::Tpl({ Tpl::from_node_id(id, ast) })),
            NodeKind::UnaryExpr => JSXExpr::Expr(Expr::Unary({ UnaryExpr::from_node_id(id, ast) })),
            NodeKind::UpdateExpr => {
                JSXExpr::Expr(Expr::Update({ UpdateExpr::from_node_id(id, ast) }))
            }
            NodeKind::YieldExpr => JSXExpr::Expr(Expr::Yield({ YieldExpr::from_node_id(id, ast) })),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for JSXSpreadChild {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXSpreadChild);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXElementName {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::JSXMemberExpr(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::Ident => JSXElementName::Ident({ Ident::from_node_id(id, ast) }),
            NodeKind::JSXMemberExpr => {
                JSXElementName::JSXMemberExpr({ JSXMemberExpr::from_node_id(id, ast) })
            }
            NodeKind::JSXNamespacedName => {
                JSXElementName::JSXNamespacedName({ JSXNamespacedName::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for JSXOpeningElement {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXOpeningElement);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXAttrOrSpread {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::JSXAttr(it) => it.node_id(),
            Self::SpreadElement(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::JSXAttr => JSXAttrOrSpread::JSXAttr({ JSXAttr::from_node_id(id, ast) }),
            NodeKind::SpreadElement => {
                JSXAttrOrSpread::SpreadElement({ SpreadElement::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for JSXClosingElement {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXClosingElement);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXAttr {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXAttr);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXAttrName {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Ident(it) => it.node_id(),
            Self::JSXNamespacedName(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::IdentName => JSXAttrName::Ident({ IdentName::from_node_id(id, ast) }),
            NodeKind::JSXNamespacedName => {
                JSXAttrName::JSXNamespacedName({ JSXNamespacedName::from_node_id(id, ast) })
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for JSXAttrValue {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::Str(it) => it.node_id(),
            Self::JSXExprContainer(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::JSXElement => JSXAttrValue::JSXElement({ JSXElement::from_node_id(id, ast) }),
            NodeKind::JSXExprContainer => {
                JSXAttrValue::JSXExprContainer({ JSXExprContainer::from_node_id(id, ast) })
            }
            NodeKind::JSXFragment => {
                JSXAttrValue::JSXFragment({ JSXFragment::from_node_id(id, ast) })
            }
            NodeKind::Str => JSXAttrValue::Str({ Str::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for JSXText {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXText);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXElement {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXElement);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXElementChild {
    #[inline]
    fn node_id(&self) -> NodeId {
        match self {
            Self::JSXText(it) => it.node_id(),
            Self::JSXExprContainer(it) => it.node_id(),
            Self::JSXSpreadChild(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
        }
    }
    #[inline]
    fn from_node_id(id: NodeId, ast: &Ast) -> Self {
        match ast.nodes.kind(id) {
            NodeKind::JSXElement => {
                JSXElementChild::JSXElement({ JSXElement::from_node_id(id, ast) })
            }
            NodeKind::JSXExprContainer => {
                JSXElementChild::JSXExprContainer({ JSXExprContainer::from_node_id(id, ast) })
            }
            NodeKind::JSXFragment => {
                JSXElementChild::JSXFragment({ JSXFragment::from_node_id(id, ast) })
            }
            NodeKind::JSXSpreadChild => {
                JSXElementChild::JSXSpreadChild({ JSXSpreadChild::from_node_id(id, ast) })
            }
            NodeKind::JSXText => JSXElementChild::JSXText({ JSXText::from_node_id(id, ast) }),
            _ => unreachable!(),
        }
    }
    #[inline]
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
        Self::from_node_id(id, ast)
    }
}
impl NodeIdTrait for JSXFragment {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXFragment);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXOpeningFragment {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXOpeningFragment);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
impl NodeIdTrait for JSXClosingFragment {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.0
    }
    #[inline]
    fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
        assert!(*ast.nodes.kind(node_id) == NodeKind::JSXClosingFragment);
        Self(node_id)
    }
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
        Self(node_id)
    }
}
