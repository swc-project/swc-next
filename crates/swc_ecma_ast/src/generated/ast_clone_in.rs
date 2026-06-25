#![allow(unused)]
use crate::ast::*;
use swc_experimental_allocator::{Allocator, CloneIn};
impl<'a, 'src> CloneIn<'a> for Program<'src> {
    type Cloned = Program<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Module(it) => Program::Module(it.clone_in(allocator)),
            Self::Script(it) => Program::Script(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Module<'src> {
    type Cloned = Module<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Module {
            span: self.span.clone_in(allocator),
            body: self.body.clone_in(allocator),
            shebang: self.shebang.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Script<'src> {
    type Cloned = Script<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Script {
            span: self.span.clone_in(allocator),
            body: self.body.clone_in(allocator),
            shebang: self.shebang.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ModuleItem<'src> {
    type Cloned = ModuleItem<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::ModuleDecl(it) => ModuleItem::ModuleDecl(it.clone_in(allocator)),
            Self::Stmt(it) => ModuleItem::Stmt(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ModuleDecl<'src> {
    type Cloned = ModuleDecl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Import(it) => ModuleDecl::Import(it.clone_in(allocator)),
            Self::ExportDecl(it) => ModuleDecl::ExportDecl(it.clone_in(allocator)),
            Self::ExportNamed(it) => ModuleDecl::ExportNamed(it.clone_in(allocator)),
            Self::ExportDefaultDecl(it) => ModuleDecl::ExportDefaultDecl(it.clone_in(allocator)),
            Self::ExportDefaultExpr(it) => ModuleDecl::ExportDefaultExpr(it.clone_in(allocator)),
            Self::ExportAll(it) => ModuleDecl::ExportAll(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ImportDecl<'src> {
    type Cloned = ImportDecl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ImportDecl {
            span: self.span.clone_in(allocator),
            specifiers: self.specifiers.clone_in(allocator),
            src: self.src.clone_in(allocator),
            type_only: self.type_only.clone_in(allocator),
            with: self.with.clone_in(allocator),
            phase: self.phase.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ImportSpecifier<'src> {
    type Cloned = ImportSpecifier<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Named(it) => ImportSpecifier::Named(it.clone_in(allocator)),
            Self::Default(it) => ImportSpecifier::Default(it.clone_in(allocator)),
            Self::Namespace(it) => ImportSpecifier::Namespace(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ImportNamedSpecifier<'src> {
    type Cloned = ImportNamedSpecifier<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ImportNamedSpecifier {
            span: self.span.clone_in(allocator),
            local: self.local.clone_in(allocator),
            imported: self.imported.clone_in(allocator),
            is_type_only: self.is_type_only.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ImportDefaultSpecifier<'src> {
    type Cloned = ImportDefaultSpecifier<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ImportDefaultSpecifier {
            span: self.span.clone_in(allocator),
            local: self.local.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ImportStarAsSpecifier<'src> {
    type Cloned = ImportStarAsSpecifier<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ImportStarAsSpecifier {
            span: self.span.clone_in(allocator),
            local: self.local.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExportDecl<'src> {
    type Cloned = ExportDecl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ExportDecl {
            span: self.span.clone_in(allocator),
            decl: self.decl.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for NamedExport<'src> {
    type Cloned = NamedExport<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        NamedExport {
            span: self.span.clone_in(allocator),
            specifiers: self.specifiers.clone_in(allocator),
            src: self.src.clone_in(allocator),
            type_only: self.type_only.clone_in(allocator),
            with: self.with.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExportSpecifier<'src> {
    type Cloned = ExportSpecifier<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Namespace(it) => ExportSpecifier::Namespace(it.clone_in(allocator)),
            Self::Default(it) => ExportSpecifier::Default(it.clone_in(allocator)),
            Self::Named(it) => ExportSpecifier::Named(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExportNamespaceSpecifier<'src> {
    type Cloned = ExportNamespaceSpecifier<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ExportNamespaceSpecifier {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ModuleExportName<'src> {
    type Cloned = ModuleExportName<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Ident(it) => ModuleExportName::Ident(it.clone_in(allocator)),
            Self::Str(it) => ModuleExportName::Str(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExportDefaultSpecifier<'src> {
    type Cloned = ExportDefaultSpecifier<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ExportDefaultSpecifier {
            exported: self.exported.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExportNamedSpecifier<'src> {
    type Cloned = ExportNamedSpecifier<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ExportNamedSpecifier {
            span: self.span.clone_in(allocator),
            orig: self.orig.clone_in(allocator),
            exported: self.exported.clone_in(allocator),
            is_type_only: self.is_type_only.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExportDefaultDecl<'src> {
    type Cloned = ExportDefaultDecl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ExportDefaultDecl {
            span: self.span.clone_in(allocator),
            decl: self.decl.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for DefaultDecl<'src> {
    type Cloned = DefaultDecl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Class(it) => DefaultDecl::Class(it.clone_in(allocator)),
            Self::Fn(it) => DefaultDecl::Fn(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExportDefaultExpr<'src> {
    type Cloned = ExportDefaultExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ExportDefaultExpr {
            span: self.span.clone_in(allocator),
            expr: self.expr.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExportAll<'src> {
    type Cloned = ExportAll<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ExportAll {
            span: self.span.clone_in(allocator),
            src: self.src.clone_in(allocator),
            type_only: self.type_only.clone_in(allocator),
            with: self.with.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for BlockStmt<'src> {
    type Cloned = BlockStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        BlockStmt {
            span: self.span.clone_in(allocator),
            stmts: self.stmts.clone_in(allocator),
            scope_id: self.scope_id.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Stmt<'src> {
    type Cloned = Stmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Block(it) => Stmt::Block(it.clone_in(allocator)),
            Self::Empty(it) => Stmt::Empty(it.clone_in(allocator)),
            Self::Debugger(it) => Stmt::Debugger(it.clone_in(allocator)),
            Self::With(it) => Stmt::With(it.clone_in(allocator)),
            Self::Return(it) => Stmt::Return(it.clone_in(allocator)),
            Self::Labeled(it) => Stmt::Labeled(it.clone_in(allocator)),
            Self::Break(it) => Stmt::Break(it.clone_in(allocator)),
            Self::Continue(it) => Stmt::Continue(it.clone_in(allocator)),
            Self::If(it) => Stmt::If(it.clone_in(allocator)),
            Self::Switch(it) => Stmt::Switch(it.clone_in(allocator)),
            Self::Throw(it) => Stmt::Throw(it.clone_in(allocator)),
            Self::Try(it) => Stmt::Try(it.clone_in(allocator)),
            Self::While(it) => Stmt::While(it.clone_in(allocator)),
            Self::DoWhile(it) => Stmt::DoWhile(it.clone_in(allocator)),
            Self::For(it) => Stmt::For(it.clone_in(allocator)),
            Self::ForIn(it) => Stmt::ForIn(it.clone_in(allocator)),
            Self::ForOf(it) => Stmt::ForOf(it.clone_in(allocator)),
            Self::Decl(it) => Stmt::Decl(it.clone_in(allocator)),
            Self::Expr(it) => Stmt::Expr(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExprStmt<'src> {
    type Cloned = ExprStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ExprStmt {
            span: self.span.clone_in(allocator),
            expr: self.expr.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for EmptyStmt {
    type Cloned = EmptyStmt;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        EmptyStmt {
            span: self.span.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for DebuggerStmt {
    type Cloned = DebuggerStmt;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        DebuggerStmt {
            span: self.span.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for WithStmt<'src> {
    type Cloned = WithStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        WithStmt {
            span: self.span.clone_in(allocator),
            obj: self.obj.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ReturnStmt<'src> {
    type Cloned = ReturnStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ReturnStmt {
            span: self.span.clone_in(allocator),
            arg: self.arg.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for LabeledStmt<'src> {
    type Cloned = LabeledStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        LabeledStmt {
            span: self.span.clone_in(allocator),
            label: self.label.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for BreakStmt<'src> {
    type Cloned = BreakStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        BreakStmt {
            span: self.span.clone_in(allocator),
            label: self.label.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ContinueStmt<'src> {
    type Cloned = ContinueStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ContinueStmt {
            span: self.span.clone_in(allocator),
            label: self.label.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for IfStmt<'src> {
    type Cloned = IfStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        IfStmt {
            span: self.span.clone_in(allocator),
            test: self.test.clone_in(allocator),
            cons: self.cons.clone_in(allocator),
            alt: self.alt.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for SwitchStmt<'src> {
    type Cloned = SwitchStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        SwitchStmt {
            span: self.span.clone_in(allocator),
            discriminant: self.discriminant.clone_in(allocator),
            cases: self.cases.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ThrowStmt<'src> {
    type Cloned = ThrowStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ThrowStmt {
            span: self.span.clone_in(allocator),
            arg: self.arg.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for TryStmt<'src> {
    type Cloned = TryStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        TryStmt {
            span: self.span.clone_in(allocator),
            block: self.block.clone_in(allocator),
            handler: self.handler.clone_in(allocator),
            finalizer: self.finalizer.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for WhileStmt<'src> {
    type Cloned = WhileStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        WhileStmt {
            span: self.span.clone_in(allocator),
            test: self.test.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for DoWhileStmt<'src> {
    type Cloned = DoWhileStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        DoWhileStmt {
            span: self.span.clone_in(allocator),
            test: self.test.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ForStmt<'src> {
    type Cloned = ForStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ForStmt {
            span: self.span.clone_in(allocator),
            init: self.init.clone_in(allocator),
            test: self.test.clone_in(allocator),
            update: self.update.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ForInStmt<'src> {
    type Cloned = ForInStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ForInStmt {
            span: self.span.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ForOfStmt<'src> {
    type Cloned = ForOfStmt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ForOfStmt {
            span: self.span.clone_in(allocator),
            is_await: self.is_await.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for SwitchCase<'src> {
    type Cloned = SwitchCase<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        SwitchCase {
            span: self.span.clone_in(allocator),
            test: self.test.clone_in(allocator),
            cons: self.cons.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for CatchClause<'src> {
    type Cloned = CatchClause<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        CatchClause {
            span: self.span.clone_in(allocator),
            param: self.param.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ForHead<'src> {
    type Cloned = ForHead<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::VarDecl(it) => ForHead::VarDecl(it.clone_in(allocator)),
            Self::UsingDecl(it) => ForHead::UsingDecl(it.clone_in(allocator)),
            Self::Pat(it) => ForHead::Pat(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for VarDeclOrExpr<'src> {
    type Cloned = VarDeclOrExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::VarDecl(it) => VarDeclOrExpr::VarDecl(it.clone_in(allocator)),
            Self::Expr(it) => VarDeclOrExpr::Expr(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Decl<'src> {
    type Cloned = Decl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Class(it) => Decl::Class(it.clone_in(allocator)),
            Self::Fn(it) => Decl::Fn(it.clone_in(allocator)),
            Self::Var(it) => Decl::Var(it.clone_in(allocator)),
            Self::Using(it) => Decl::Using(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for FnDecl<'src> {
    type Cloned = FnDecl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        FnDecl {
            ident: self.ident.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
            function: self.function.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ClassDecl<'src> {
    type Cloned = ClassDecl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ClassDecl {
            ident: self.ident.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
            class: self.class.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for VarDecl<'src> {
    type Cloned = VarDecl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        VarDecl {
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
            decls: self.decls.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for VarDeclarator<'src> {
    type Cloned = VarDeclarator<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        VarDeclarator {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            init: self.init.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for UsingDecl<'src> {
    type Cloned = UsingDecl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        UsingDecl {
            span: self.span.clone_in(allocator),
            is_await: self.is_await.clone_in(allocator),
            decls: self.decls.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Expr<'src> {
    type Cloned = Expr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::This(it) => Expr::This(it.clone_in(allocator)),
            Self::Array(it) => Expr::Array(it.clone_in(allocator)),
            Self::Object(it) => Expr::Object(it.clone_in(allocator)),
            Self::Fn(it) => Expr::Fn(it.clone_in(allocator)),
            Self::Unary(it) => Expr::Unary(it.clone_in(allocator)),
            Self::Update(it) => Expr::Update(it.clone_in(allocator)),
            Self::Bin(it) => Expr::Bin(it.clone_in(allocator)),
            Self::Assign(it) => Expr::Assign(it.clone_in(allocator)),
            Self::Member(it) => Expr::Member(it.clone_in(allocator)),
            Self::SuperProp(it) => Expr::SuperProp(it.clone_in(allocator)),
            Self::Cond(it) => Expr::Cond(it.clone_in(allocator)),
            Self::Call(it) => Expr::Call(it.clone_in(allocator)),
            Self::New(it) => Expr::New(it.clone_in(allocator)),
            Self::Seq(it) => Expr::Seq(it.clone_in(allocator)),
            Self::Ident(it) => Expr::Ident(it.clone_in(allocator)),
            Self::Lit(it) => Expr::Lit(it.clone_in(allocator)),
            Self::Tpl(it) => Expr::Tpl(it.clone_in(allocator)),
            Self::TaggedTpl(it) => Expr::TaggedTpl(it.clone_in(allocator)),
            Self::Arrow(it) => Expr::Arrow(it.clone_in(allocator)),
            Self::Class(it) => Expr::Class(it.clone_in(allocator)),
            Self::Yield(it) => Expr::Yield(it.clone_in(allocator)),
            Self::MetaProp(it) => Expr::MetaProp(it.clone_in(allocator)),
            Self::Await(it) => Expr::Await(it.clone_in(allocator)),
            Self::Paren(it) => Expr::Paren(it.clone_in(allocator)),
            Self::JSXMember(it) => Expr::JSXMember(it.clone_in(allocator)),
            Self::JSXNamespacedName(it) => Expr::JSXNamespacedName(it.clone_in(allocator)),
            Self::JSXEmpty(it) => Expr::JSXEmpty(it.clone_in(allocator)),
            Self::JSXElement(it) => Expr::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => Expr::JSXFragment(it.clone_in(allocator)),
            Self::PrivateName(it) => Expr::PrivateName(it.clone_in(allocator)),
            Self::OptChain(it) => Expr::OptChain(it.clone_in(allocator)),
            Self::Invalid(it) => Expr::Invalid(it.clone_in(allocator)),
        }
    }
}
impl<'a> CloneIn<'a> for ThisExpr {
    type Cloned = ThisExpr;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ThisExpr {
            span: self.span.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ArrayLit<'src> {
    type Cloned = ArrayLit<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ArrayLit {
            span: self.span.clone_in(allocator),
            elems: self.elems.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ObjectLit<'src> {
    type Cloned = ObjectLit<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ObjectLit {
            span: self.span.clone_in(allocator),
            props: self.props.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for PropOrSpread<'src> {
    type Cloned = PropOrSpread<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Spread(it) => PropOrSpread::Spread(it.clone_in(allocator)),
            Self::Prop(it) => PropOrSpread::Prop(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for SpreadElement<'src> {
    type Cloned = SpreadElement<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        SpreadElement {
            dot3_token: self.dot3_token.clone_in(allocator),
            expr: self.expr.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for UnaryExpr<'src> {
    type Cloned = UnaryExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        UnaryExpr {
            span: self.span.clone_in(allocator),
            op: self.op.clone_in(allocator),
            arg: self.arg.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for UpdateExpr<'src> {
    type Cloned = UpdateExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        UpdateExpr {
            span: self.span.clone_in(allocator),
            op: self.op.clone_in(allocator),
            prefix: self.prefix.clone_in(allocator),
            arg: self.arg.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for BinExpr<'src> {
    type Cloned = BinExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        BinExpr {
            span: self.span.clone_in(allocator),
            op: self.op.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for FnExpr<'src> {
    type Cloned = FnExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        FnExpr {
            ident: self.ident.clone_in(allocator),
            function: self.function.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ClassExpr<'src> {
    type Cloned = ClassExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ClassExpr {
            ident: self.ident.clone_in(allocator),
            class: self.class.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for AssignExpr<'src> {
    type Cloned = AssignExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        AssignExpr {
            span: self.span.clone_in(allocator),
            op: self.op.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for MemberExpr<'src> {
    type Cloned = MemberExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        MemberExpr {
            span: self.span.clone_in(allocator),
            obj: self.obj.clone_in(allocator),
            prop: self.prop.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for MemberProp<'src> {
    type Cloned = MemberProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Ident(it) => MemberProp::Ident(it.clone_in(allocator)),
            Self::PrivateName(it) => MemberProp::PrivateName(it.clone_in(allocator)),
            Self::Computed(it) => MemberProp::Computed(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for SuperPropExpr<'src> {
    type Cloned = SuperPropExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        SuperPropExpr {
            span: self.span.clone_in(allocator),
            obj: self.obj.clone_in(allocator),
            prop: self.prop.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for SuperProp<'src> {
    type Cloned = SuperProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Ident(it) => SuperProp::Ident(it.clone_in(allocator)),
            Self::Computed(it) => SuperProp::Computed(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for CondExpr<'src> {
    type Cloned = CondExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        CondExpr {
            span: self.span.clone_in(allocator),
            test: self.test.clone_in(allocator),
            cons: self.cons.clone_in(allocator),
            alt: self.alt.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for CallExpr<'src> {
    type Cloned = CallExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        CallExpr {
            span: self.span.clone_in(allocator),
            callee: self.callee.clone_in(allocator),
            args: self.args.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for NewExpr<'src> {
    type Cloned = NewExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        NewExpr {
            span: self.span.clone_in(allocator),
            callee: self.callee.clone_in(allocator),
            args: self.args.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for SeqExpr<'src> {
    type Cloned = SeqExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        SeqExpr {
            span: self.span.clone_in(allocator),
            exprs: self.exprs.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ArrowExpr<'src> {
    type Cloned = ArrowExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ArrowExpr {
            span: self.span.clone_in(allocator),
            params: self.params.clone_in(allocator),
            body: self.body.clone_in(allocator),
            is_async: self.is_async.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for YieldExpr<'src> {
    type Cloned = YieldExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        YieldExpr {
            span: self.span.clone_in(allocator),
            arg: self.arg.clone_in(allocator),
            delegate: self.delegate.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for MetaPropExpr {
    type Cloned = MetaPropExpr;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        MetaPropExpr {
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for AwaitExpr<'src> {
    type Cloned = AwaitExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        AwaitExpr {
            span: self.span.clone_in(allocator),
            arg: self.arg.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Tpl<'src> {
    type Cloned = Tpl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Tpl {
            span: self.span.clone_in(allocator),
            exprs: self.exprs.clone_in(allocator),
            quasis: self.quasis.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for TaggedTpl<'src> {
    type Cloned = TaggedTpl<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        TaggedTpl {
            span: self.span.clone_in(allocator),
            tag: self.tag.clone_in(allocator),
            tpl: self.tpl.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for TplElement<'src> {
    type Cloned = TplElement<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        TplElement {
            span: self.span.clone_in(allocator),
            tail: self.tail.clone_in(allocator),
            cooked: self.cooked.clone_in(allocator),
            raw: self.raw.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ParenExpr<'src> {
    type Cloned = ParenExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ParenExpr {
            span: self.span.clone_in(allocator),
            expr: self.expr.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Callee<'src> {
    type Cloned = Callee<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Super(it) => Callee::Super(it.clone_in(allocator)),
            Self::Import(it) => Callee::Import(it.clone_in(allocator)),
            Self::Expr(it) => Callee::Expr(it.clone_in(allocator)),
        }
    }
}
impl<'a> CloneIn<'a> for Super {
    type Cloned = Super;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Super {
            span: self.span.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for Import {
    type Cloned = Import;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Import {
            span: self.span.clone_in(allocator),
            phase: self.phase.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ExprOrSpread<'src> {
    type Cloned = ExprOrSpread<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ExprOrSpread {
            spread: self.spread.clone_in(allocator),
            expr: self.expr.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for BlockStmtOrExpr<'src> {
    type Cloned = BlockStmtOrExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::BlockStmt(it) => BlockStmtOrExpr::BlockStmt(it.clone_in(allocator)),
            Self::Expr(it) => BlockStmtOrExpr::Expr(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for AssignTarget<'src> {
    type Cloned = AssignTarget<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Simple(it) => AssignTarget::Simple(it.clone_in(allocator)),
            Self::Pat(it) => AssignTarget::Pat(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for AssignTargetPat<'src> {
    type Cloned = AssignTargetPat<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Array(it) => AssignTargetPat::Array(it.clone_in(allocator)),
            Self::Object(it) => AssignTargetPat::Object(it.clone_in(allocator)),
            Self::Invalid(it) => AssignTargetPat::Invalid(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for SimpleAssignTarget<'src> {
    type Cloned = SimpleAssignTarget<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Ident(it) => SimpleAssignTarget::Ident(it.clone_in(allocator)),
            Self::Member(it) => SimpleAssignTarget::Member(it.clone_in(allocator)),
            Self::SuperProp(it) => SimpleAssignTarget::SuperProp(it.clone_in(allocator)),
            Self::Paren(it) => SimpleAssignTarget::Paren(it.clone_in(allocator)),
            Self::OptChain(it) => SimpleAssignTarget::OptChain(it.clone_in(allocator)),
            Self::Invalid(it) => SimpleAssignTarget::Invalid(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for OptChainExpr<'src> {
    type Cloned = OptChainExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        OptChainExpr {
            span: self.span.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
            base: self.base.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for OptChainBase<'src> {
    type Cloned = OptChainBase<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Member(it) => OptChainBase::Member(it.clone_in(allocator)),
            Self::Call(it) => OptChainBase::Call(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for OptCall<'src> {
    type Cloned = OptCall<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        OptCall {
            span: self.span.clone_in(allocator),
            callee: self.callee.clone_in(allocator),
            args: self.args.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for Invalid {
    type Cloned = Invalid;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Invalid {}
    }
}
impl<'a, 'src> CloneIn<'a> for Function<'src> {
    type Cloned = Function<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Function {
            span: self.span.clone_in(allocator),
            params: self.params.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
            body: self.body.clone_in(allocator),
            is_generator: self.is_generator.clone_in(allocator),
            is_async: self.is_async.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Param<'src> {
    type Cloned = Param<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Param {
            span: self.span.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
            pat: self.pat.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ParamOrTsParamProp<'src> {
    type Cloned = ParamOrTsParamProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Param(it) => ParamOrTsParamProp::Param(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Class<'src> {
    type Cloned = Class<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Class {
            span: self.span.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
            body: self.body.clone_in(allocator),
            super_class: self.super_class.clone_in(allocator),
            is_abstract: self.is_abstract.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ClassMember<'src> {
    type Cloned = ClassMember<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Constructor(it) => ClassMember::Constructor(it.clone_in(allocator)),
            Self::Method(it) => ClassMember::Method(it.clone_in(allocator)),
            Self::PrivateMethod(it) => ClassMember::PrivateMethod(it.clone_in(allocator)),
            Self::ClassProp(it) => ClassMember::ClassProp(it.clone_in(allocator)),
            Self::PrivateProp(it) => ClassMember::PrivateProp(it.clone_in(allocator)),
            Self::Empty(it) => ClassMember::Empty(it.clone_in(allocator)),
            Self::StaticBlock(it) => ClassMember::StaticBlock(it.clone_in(allocator)),
            Self::AutoAccessor(it) => ClassMember::AutoAccessor(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ClassProp<'src> {
    type Cloned = ClassProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ClassProp {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
            is_static: self.is_static.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for PrivateProp<'src> {
    type Cloned = PrivateProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        PrivateProp {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
            is_static: self.is_static.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ClassMethod<'src> {
    type Cloned = ClassMethod<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ClassMethod {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            function: self.function.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            is_static: self.is_static.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for PrivateMethod<'src> {
    type Cloned = PrivateMethod<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        PrivateMethod {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            function: self.function.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            is_static: self.is_static.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Constructor<'src> {
    type Cloned = Constructor<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Constructor {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            params: self.params.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Decorator<'src> {
    type Cloned = Decorator<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Decorator {
            span: self.span.clone_in(allocator),
            expr: self.expr.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for StaticBlock<'src> {
    type Cloned = StaticBlock<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        StaticBlock {
            span: self.span.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Key<'src> {
    type Cloned = Key<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Private(it) => Key::Private(it.clone_in(allocator)),
            Self::Public(it) => Key::Public(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for AutoAccessor<'src> {
    type Cloned = AutoAccessor<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        AutoAccessor {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
            is_static: self.is_static.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Prop<'src> {
    type Cloned = Prop<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Shorthand(it) => Prop::Shorthand(it.clone_in(allocator)),
            Self::KeyValue(it) => Prop::KeyValue(it.clone_in(allocator)),
            Self::Assign(it) => Prop::Assign(it.clone_in(allocator)),
            Self::Getter(it) => Prop::Getter(it.clone_in(allocator)),
            Self::Setter(it) => Prop::Setter(it.clone_in(allocator)),
            Self::Method(it) => Prop::Method(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for KeyValueProp<'src> {
    type Cloned = KeyValueProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        KeyValueProp {
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for AssignProp<'src> {
    type Cloned = AssignProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        AssignProp {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for GetterProp<'src> {
    type Cloned = GetterProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        GetterProp {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for SetterProp<'src> {
    type Cloned = SetterProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        SetterProp {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            this_param: self.this_param.clone_in(allocator),
            param: self.param.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for MethodProp<'src> {
    type Cloned = MethodProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        MethodProp {
            key: self.key.clone_in(allocator),
            function: self.function.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for PropName<'src> {
    type Cloned = PropName<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Ident(it) => PropName::Ident(it.clone_in(allocator)),
            Self::Str(it) => PropName::Str(it.clone_in(allocator)),
            Self::Num(it) => PropName::Num(it.clone_in(allocator)),
            Self::Computed(it) => PropName::Computed(it.clone_in(allocator)),
            Self::BigInt(it) => PropName::BigInt(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ComputedPropName<'src> {
    type Cloned = ComputedPropName<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ComputedPropName {
            span: self.span.clone_in(allocator),
            expr: self.expr.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Pat<'src> {
    type Cloned = Pat<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Ident(it) => Pat::Ident(it.clone_in(allocator)),
            Self::Array(it) => Pat::Array(it.clone_in(allocator)),
            Self::Rest(it) => Pat::Rest(it.clone_in(allocator)),
            Self::Object(it) => Pat::Object(it.clone_in(allocator)),
            Self::Assign(it) => Pat::Assign(it.clone_in(allocator)),
            Self::Invalid(it) => Pat::Invalid(it.clone_in(allocator)),
            Self::Expr(it) => Pat::Expr(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ArrayPat<'src> {
    type Cloned = ArrayPat<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ArrayPat {
            span: self.span.clone_in(allocator),
            elems: self.elems.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ObjectPat<'src> {
    type Cloned = ObjectPat<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        ObjectPat {
            span: self.span.clone_in(allocator),
            props: self.props.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for AssignPat<'src> {
    type Cloned = AssignPat<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        AssignPat {
            span: self.span.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for RestPat<'src> {
    type Cloned = RestPat<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        RestPat {
            span: self.span.clone_in(allocator),
            dot3_token: self.dot3_token.clone_in(allocator),
            arg: self.arg.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for ObjectPatProp<'src> {
    type Cloned = ObjectPatProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::KeyValue(it) => ObjectPatProp::KeyValue(it.clone_in(allocator)),
            Self::Assign(it) => ObjectPatProp::Assign(it.clone_in(allocator)),
            Self::Rest(it) => ObjectPatProp::Rest(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for KeyValuePatProp<'src> {
    type Cloned = KeyValuePatProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        KeyValuePatProp {
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for AssignPatProp<'src> {
    type Cloned = AssignPatProp<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        AssignPatProp {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Ident<'src> {
    type Cloned = Ident<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Ident {
            span: self.span.clone_in(allocator),
            sym: self.sym.clone_in(allocator),
            symbol_id: self.symbol_id.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for IdentName<'src> {
    type Cloned = IdentName<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        IdentName {
            span: self.span.clone_in(allocator),
            sym: self.sym.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for PrivateName<'src> {
    type Cloned = PrivateName<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        PrivateName {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for BindingIdent<'src> {
    type Cloned = BindingIdent<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        BindingIdent {
            id: self.id.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Lit<'src> {
    type Cloned = Lit<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Str(it) => Lit::Str(it.clone_in(allocator)),
            Self::Bool(it) => Lit::Bool(it.clone_in(allocator)),
            Self::Null(it) => Lit::Null(it.clone_in(allocator)),
            Self::Num(it) => Lit::Num(it.clone_in(allocator)),
            Self::BigInt(it) => Lit::BigInt(it.clone_in(allocator)),
            Self::Regex(it) => Lit::Regex(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Str<'src> {
    type Cloned = Str<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Str {
            span: self.span.clone_in(allocator),
            value: self.value.clone_in(allocator),
            raw: self.raw.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for Bool {
    type Cloned = Bool;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Bool {
            span: self.span.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for Null {
    type Cloned = Null;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Null {
            span: self.span.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Number<'src> {
    type Cloned = Number<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Number {
            span: self.span.clone_in(allocator),
            value: self.value.clone_in(allocator),
            raw: self.raw.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for BigInt<'src> {
    type Cloned = BigInt<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        BigInt {
            span: self.span.clone_in(allocator),
            value: self.value.clone_in(allocator),
            raw: self.raw.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for Regex<'src> {
    type Cloned = Regex<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Regex {
            span: self.span.clone_in(allocator),
            exp: self.exp.clone_in(allocator),
            flags: self.flags.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXObject<'src> {
    type Cloned = JSXObject<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::JSXMemberExpr(it) => JSXObject::JSXMemberExpr(it.clone_in(allocator)),
            Self::Ident(it) => JSXObject::Ident(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXMemberExpr<'src> {
    type Cloned = JSXMemberExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXMemberExpr {
            span: self.span.clone_in(allocator),
            obj: self.obj.clone_in(allocator),
            prop: self.prop.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXNamespacedName<'src> {
    type Cloned = JSXNamespacedName<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXNamespacedName {
            span: self.span.clone_in(allocator),
            ns: self.ns.clone_in(allocator),
            name: self.name.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for JSXEmptyExpr {
    type Cloned = JSXEmptyExpr;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXEmptyExpr {
            span: self.span.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXExprContainer<'src> {
    type Cloned = JSXExprContainer<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXExprContainer {
            span: self.span.clone_in(allocator),
            expr: self.expr.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXExpr<'src> {
    type Cloned = JSXExpr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::JSXEmptyExpr(it) => JSXExpr::JSXEmptyExpr(it.clone_in(allocator)),
            Self::Expr(it) => JSXExpr::Expr(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXSpreadChild<'src> {
    type Cloned = JSXSpreadChild<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXSpreadChild {
            span: self.span.clone_in(allocator),
            expr: self.expr.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXElementName<'src> {
    type Cloned = JSXElementName<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Ident(it) => JSXElementName::Ident(it.clone_in(allocator)),
            Self::JSXMemberExpr(it) => JSXElementName::JSXMemberExpr(it.clone_in(allocator)),
            Self::JSXNamespacedName(it) => {
                JSXElementName::JSXNamespacedName(it.clone_in(allocator))
            }
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXOpeningElement<'src> {
    type Cloned = JSXOpeningElement<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXOpeningElement {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            attrs: self.attrs.clone_in(allocator),
            self_closing: self.self_closing.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXAttrOrSpread<'src> {
    type Cloned = JSXAttrOrSpread<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::JSXAttr(it) => JSXAttrOrSpread::JSXAttr(it.clone_in(allocator)),
            Self::SpreadElement(it) => JSXAttrOrSpread::SpreadElement(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXClosingElement<'src> {
    type Cloned = JSXClosingElement<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXClosingElement {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXAttr<'src> {
    type Cloned = JSXAttr<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXAttr {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXAttrName<'src> {
    type Cloned = JSXAttrName<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Ident(it) => JSXAttrName::Ident(it.clone_in(allocator)),
            Self::JSXNamespacedName(it) => JSXAttrName::JSXNamespacedName(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXAttrValue<'src> {
    type Cloned = JSXAttrValue<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::Str(it) => JSXAttrValue::Str(it.clone_in(allocator)),
            Self::JSXExprContainer(it) => JSXAttrValue::JSXExprContainer(it.clone_in(allocator)),
            Self::JSXElement(it) => JSXAttrValue::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => JSXAttrValue::JSXFragment(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXText<'src> {
    type Cloned = JSXText<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXText {
            span: self.span.clone_in(allocator),
            value: self.value.clone_in(allocator),
            raw: self.raw.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXElement<'src> {
    type Cloned = JSXElement<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXElement {
            span: self.span.clone_in(allocator),
            opening: self.opening.clone_in(allocator),
            children: self.children.clone_in(allocator),
            closing: self.closing.clone_in(allocator),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXElementChild<'src> {
    type Cloned = JSXElementChild<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        match self {
            Self::JSXText(it) => JSXElementChild::JSXText(it.clone_in(allocator)),
            Self::JSXExprContainer(it) => JSXElementChild::JSXExprContainer(it.clone_in(allocator)),
            Self::JSXSpreadChild(it) => JSXElementChild::JSXSpreadChild(it.clone_in(allocator)),
            Self::JSXElement(it) => JSXElementChild::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => JSXElementChild::JSXFragment(it.clone_in(allocator)),
        }
    }
}
impl<'a, 'src> CloneIn<'a> for JSXFragment<'src> {
    type Cloned = JSXFragment<'a>;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXFragment {
            span: self.span.clone_in(allocator),
            opening: self.opening.clone_in(allocator),
            children: self.children.clone_in(allocator),
            closing: self.closing.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for JSXOpeningFragment {
    type Cloned = JSXOpeningFragment;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXOpeningFragment {
            span: self.span.clone_in(allocator),
        }
    }
}
impl<'a> CloneIn<'a> for JSXClosingFragment {
    type Cloned = JSXClosingFragment;
    #[inline]
    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        JSXClosingFragment {
            span: self.span.clone_in(allocator),
        }
    }
}
