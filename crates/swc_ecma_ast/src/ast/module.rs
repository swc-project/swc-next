use swc_experimental_ast_macros::ast;

use crate::ast::*;

#[ast(repr)]
pub enum Program {
    Module(Module),
    Script(Script),
}

#[ast]
pub struct Module {
    body: Vec<ModuleItem>,
    shebang: Option<Atom>,
}

#[ast]
pub struct Script {
    body: Vec<Stmt>,
    shebang: Option<Atom>,
}

#[ast(repr)]
pub enum ModuleItem {
    ModuleDeclImport(ImportDecl),
    ModuleDeclExportDecl(ExportDecl),
    ModuleDeclExportNamed(NamedExport),
    ModuleDeclExportDefaultDecl(ExportDefaultDecl),
    ModuleDeclExportDefaultExpr(ExportDefaultExpr),
    ModuleDeclExportAll(ExportAll),
    StmtBlock(BlockStmt),
    StmtEmpty(EmptyStmt),
    StmtDebugger(DebuggerStmt),
    StmtWith(WithStmt),
    StmtReturn(ReturnStmt),
    StmtLabeled(LabeledStmt),
    StmtBreak(BreakStmt),
    StmtContinue(ContinueStmt),
    StmtIf(IfStmt),
    StmtSwitch(SwitchStmt),
    StmtThrow(ThrowStmt),
    StmtTry(TryStmt),
    StmtWhile(WhileStmt),
    StmtDoWhile(DoWhileStmt),
    StmtFor(ForStmt),
    StmtForIn(ForInStmt),
    StmtForOf(ForOfStmt),
    StmtDeclClass(ClassDecl),
    StmtDeclFn(FnDecl),
    StmtDeclVar(VarDecl),
    StmtDeclUsing(UsingDecl),
    StmtExpr(ExprStmt),
}

#[allow(non_snake_case)]
impl ModuleItem {
    pub fn ModuleDecl(decl: ModuleDecl) -> Self {
        match decl {
            ModuleDecl::Import(node) => Self::ModuleDeclImport(node),
            ModuleDecl::ExportDecl(node) => Self::ModuleDeclExportDecl(node),
            ModuleDecl::ExportNamed(node) => Self::ModuleDeclExportNamed(node),
            ModuleDecl::ExportDefaultDecl(node) => Self::ModuleDeclExportDefaultDecl(node),
            ModuleDecl::ExportDefaultExpr(node) => Self::ModuleDeclExportDefaultExpr(node),
            ModuleDecl::ExportAll(node) => Self::ModuleDeclExportAll(node),
        }
    }

    pub fn Stmt(stmt: Stmt) -> Self {
        match stmt {
            Stmt::Block(node) => Self::StmtBlock(node),
            Stmt::Empty(node) => Self::StmtEmpty(node),
            Stmt::Debugger(node) => Self::StmtDebugger(node),
            Stmt::With(node) => Self::StmtWith(node),
            Stmt::Return(node) => Self::StmtReturn(node),
            Stmt::Labeled(node) => Self::StmtLabeled(node),
            Stmt::Break(node) => Self::StmtBreak(node),
            Stmt::Continue(node) => Self::StmtContinue(node),
            Stmt::If(node) => Self::StmtIf(node),
            Stmt::Switch(node) => Self::StmtSwitch(node),
            Stmt::Throw(node) => Self::StmtThrow(node),
            Stmt::Try(node) => Self::StmtTry(node),
            Stmt::While(node) => Self::StmtWhile(node),
            Stmt::DoWhile(node) => Self::StmtDoWhile(node),
            Stmt::For(node) => Self::StmtFor(node),
            Stmt::ForIn(node) => Self::StmtForIn(node),
            Stmt::ForOf(node) => Self::StmtForOf(node),
            Stmt::Class(node) => Self::StmtDeclClass(node),
            Stmt::Fn(node) => Self::StmtDeclFn(node),
            Stmt::Var(node) => Self::StmtDeclVar(node),
            Stmt::Using(node) => Self::StmtDeclUsing(node),
            Stmt::Expr(node) => Self::StmtExpr(node),
        }
    }

    pub fn into_module_decl(self) -> Option<ModuleDecl> {
        match self {
            Self::ModuleDeclImport(node) => Some(ModuleDecl::Import(node)),
            Self::ModuleDeclExportDecl(node) => Some(ModuleDecl::ExportDecl(node)),
            Self::ModuleDeclExportNamed(node) => Some(ModuleDecl::ExportNamed(node)),
            Self::ModuleDeclExportDefaultDecl(node) => Some(ModuleDecl::ExportDefaultDecl(node)),
            Self::ModuleDeclExportDefaultExpr(node) => Some(ModuleDecl::ExportDefaultExpr(node)),
            Self::ModuleDeclExportAll(node) => Some(ModuleDecl::ExportAll(node)),
            _ => None,
        }
    }

    pub fn into_stmt(self) -> Option<Stmt> {
        match self {
            Self::StmtBlock(node) => Some(Stmt::Block(node)),
            Self::StmtEmpty(node) => Some(Stmt::Empty(node)),
            Self::StmtDebugger(node) => Some(Stmt::Debugger(node)),
            Self::StmtWith(node) => Some(Stmt::With(node)),
            Self::StmtReturn(node) => Some(Stmt::Return(node)),
            Self::StmtLabeled(node) => Some(Stmt::Labeled(node)),
            Self::StmtBreak(node) => Some(Stmt::Break(node)),
            Self::StmtContinue(node) => Some(Stmt::Continue(node)),
            Self::StmtIf(node) => Some(Stmt::If(node)),
            Self::StmtSwitch(node) => Some(Stmt::Switch(node)),
            Self::StmtThrow(node) => Some(Stmt::Throw(node)),
            Self::StmtTry(node) => Some(Stmt::Try(node)),
            Self::StmtWhile(node) => Some(Stmt::While(node)),
            Self::StmtDoWhile(node) => Some(Stmt::DoWhile(node)),
            Self::StmtFor(node) => Some(Stmt::For(node)),
            Self::StmtForIn(node) => Some(Stmt::ForIn(node)),
            Self::StmtForOf(node) => Some(Stmt::ForOf(node)),
            Self::StmtDeclClass(node) => Some(Stmt::Class(node)),
            Self::StmtDeclFn(node) => Some(Stmt::Fn(node)),
            Self::StmtDeclVar(node) => Some(Stmt::Var(node)),
            Self::StmtDeclUsing(node) => Some(Stmt::Using(node)),
            Self::StmtExpr(node) => Some(Stmt::Expr(node)),
            _ => None,
        }
    }
}
