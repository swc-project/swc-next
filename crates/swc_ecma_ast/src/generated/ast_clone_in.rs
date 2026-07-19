#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, GetSpan};
use crate::{ast::*, node_id::*};
impl<'a> CloneIn<'a> for Program {
    type Cloned = Program;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.program_data(*self);
        match data {
            ProgramData::Module(it) => Self::Module(it.clone_in(ast)),
            ProgramData::Script(it) => Self::Script(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for Module {
    type Cloned = Module;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let shebang = self.shebang(ast).clone_in(ast);
        ast.module(span, body, shebang)
    }
}
impl<'a> CloneIn<'a> for Script {
    type Cloned = Script;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let shebang = self.shebang(ast).clone_in(ast);
        ast.script(span, body, shebang)
    }
}
impl<'a> CloneIn<'a> for ModuleItem {
    type Cloned = ModuleItem;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.module_item_data(*self);
        match data {
            ModuleItemData::ModuleDecl(it) => Self::ModuleDecl(it.clone_in(ast)),
            ModuleItemData::Stmt(it) => Self::Stmt(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ModuleDecl {
    type Cloned = ModuleDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.module_decl_data(*self);
        match data {
            ModuleDeclData::Import(it) => Self::Import(it.clone_in(ast)),
            ModuleDeclData::ExportDecl(it) => Self::ExportDecl(it.clone_in(ast)),
            ModuleDeclData::ExportNamed(it) => Self::ExportNamed(it.clone_in(ast)),
            ModuleDeclData::ExportDefaultDecl(it) => Self::ExportDefaultDecl(it.clone_in(ast)),
            ModuleDeclData::ExportDefaultExpr(it) => Self::ExportDefaultExpr(it.clone_in(ast)),
            ModuleDeclData::ExportAll(it) => Self::ExportAll(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ImportDecl {
    type Cloned = ImportDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let specifiers = self.specifiers(ast).clone_in(ast);
        let src = self.src(ast).clone_in(ast);
        let type_only = self.type_only(ast).clone_in(ast);
        let with = self.with(ast).clone_in(ast);
        let phase = self.phase(ast).clone_in(ast);
        ast.import_decl(span, specifiers, src, type_only, with, phase)
    }
}
impl<'a> CloneIn<'a> for ImportSpecifier {
    type Cloned = ImportSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.import_specifier_data(*self);
        match data {
            ImportSpecifierData::Named(it) => Self::Named(it.clone_in(ast)),
            ImportSpecifierData::Default(it) => Self::Default(it.clone_in(ast)),
            ImportSpecifierData::Namespace(it) => Self::Namespace(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ImportNamedSpecifier {
    type Cloned = ImportNamedSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        let imported = self.imported(ast).clone_in(ast);
        let is_type_only = self.is_type_only(ast).clone_in(ast);
        ast.import_named_specifier(span, local, imported, is_type_only)
    }
}
impl<'a> CloneIn<'a> for ImportDefaultSpecifier {
    type Cloned = ImportDefaultSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        ast.import_default_specifier(span, local)
    }
}
impl<'a> CloneIn<'a> for ImportStarAsSpecifier {
    type Cloned = ImportStarAsSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        ast.import_star_as_specifier(span, local)
    }
}
impl<'a> CloneIn<'a> for ExportDecl {
    type Cloned = ExportDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decl = self.decl(ast).clone_in(ast);
        ast.export_decl(span, decl)
    }
}
impl<'a> CloneIn<'a> for NamedExport {
    type Cloned = NamedExport;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let specifiers = self.specifiers(ast).clone_in(ast);
        let src = self.src(ast).clone_in(ast);
        let type_only = self.type_only(ast).clone_in(ast);
        let with = self.with(ast).clone_in(ast);
        ast.named_export(span, specifiers, src, type_only, with)
    }
}
impl<'a> CloneIn<'a> for ExportSpecifier {
    type Cloned = ExportSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.export_specifier_data(*self);
        match data {
            ExportSpecifierData::Namespace(it) => Self::Namespace(it.clone_in(ast)),
            ExportSpecifierData::Default(it) => Self::Default(it.clone_in(ast)),
            ExportSpecifierData::Named(it) => Self::Named(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ExportNamespaceSpecifier {
    type Cloned = ExportNamespaceSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.export_namespace_specifier(span, name)
    }
}
impl<'a> CloneIn<'a> for ModuleExportName {
    type Cloned = ModuleExportName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.module_export_name_data(*self);
        match data {
            ModuleExportNameData::Ident(it) => Self::Ident(it.clone_in(ast)),
            ModuleExportNameData::Str(it) => Self::Str(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ExportDefaultSpecifier {
    type Cloned = ExportDefaultSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exported = self.exported(ast).clone_in(ast);
        ast.export_default_specifier(span, exported)
    }
}
impl<'a> CloneIn<'a> for ExportNamedSpecifier {
    type Cloned = ExportNamedSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let orig = self.orig(ast).clone_in(ast);
        let exported = self.exported(ast).clone_in(ast);
        let is_type_only = self.is_type_only(ast).clone_in(ast);
        ast.export_named_specifier(span, orig, exported, is_type_only)
    }
}
impl<'a> CloneIn<'a> for ExportDefaultDecl {
    type Cloned = ExportDefaultDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decl = self.decl(ast).clone_in(ast);
        ast.export_default_decl(span, decl)
    }
}
impl<'a> CloneIn<'a> for DefaultDecl {
    type Cloned = DefaultDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.default_decl_data(*self);
        match data {
            DefaultDeclData::Class(it) => Self::Class(it.clone_in(ast)),
            DefaultDeclData::Fn(it) => Self::Fn(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ExportDefaultExpr {
    type Cloned = ExportDefaultExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.export_default_expr(span, expr)
    }
}
impl<'a> CloneIn<'a> for ExportAll {
    type Cloned = ExportAll;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let src = self.src(ast).clone_in(ast);
        let type_only = self.type_only(ast).clone_in(ast);
        let with = self.with(ast).clone_in(ast);
        ast.export_all(span, src, type_only, with)
    }
}
impl<'a> CloneIn<'a> for BlockStmt {
    type Cloned = BlockStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let stmts = self.stmts(ast).clone_in(ast);
        ast.block_stmt(span, stmts)
    }
}
impl<'a> CloneIn<'a> for Stmt {
    type Cloned = Stmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.stmt_data(*self);
        match data {
            StmtData::Block(it) => Self::Block(it.clone_in(ast)),
            StmtData::Empty(it) => Self::Empty(it.clone_in(ast)),
            StmtData::Debugger(it) => Self::Debugger(it.clone_in(ast)),
            StmtData::With(it) => Self::With(it.clone_in(ast)),
            StmtData::Return(it) => Self::Return(it.clone_in(ast)),
            StmtData::Labeled(it) => Self::Labeled(it.clone_in(ast)),
            StmtData::Break(it) => Self::Break(it.clone_in(ast)),
            StmtData::Continue(it) => Self::Continue(it.clone_in(ast)),
            StmtData::If(it) => Self::If(it.clone_in(ast)),
            StmtData::Switch(it) => Self::Switch(it.clone_in(ast)),
            StmtData::Throw(it) => Self::Throw(it.clone_in(ast)),
            StmtData::Try(it) => Self::Try(it.clone_in(ast)),
            StmtData::While(it) => Self::While(it.clone_in(ast)),
            StmtData::DoWhile(it) => Self::DoWhile(it.clone_in(ast)),
            StmtData::For(it) => Self::For(it.clone_in(ast)),
            StmtData::ForIn(it) => Self::ForIn(it.clone_in(ast)),
            StmtData::ForOf(it) => Self::ForOf(it.clone_in(ast)),
            StmtData::Decl(it) => Self::Decl(it.clone_in(ast)),
            StmtData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ExprStmt {
    type Cloned = ExprStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.expr_stmt(span, expr)
    }
}
impl<'a> CloneIn<'a> for EmptyStmt {
    type Cloned = EmptyStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.empty_stmt(span)
    }
}
impl<'a> CloneIn<'a> for DebuggerStmt {
    type Cloned = DebuggerStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.debugger_stmt(span)
    }
}
impl<'a> CloneIn<'a> for WithStmt {
    type Cloned = WithStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let obj = self.obj(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.with_stmt(span, obj, body)
    }
}
impl<'a> CloneIn<'a> for ReturnStmt {
    type Cloned = ReturnStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.return_stmt(span, arg)
    }
}
impl<'a> CloneIn<'a> for LabeledStmt {
    type Cloned = LabeledStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.labeled_stmt(span, label, body)
    }
}
impl<'a> CloneIn<'a> for BreakStmt {
    type Cloned = BreakStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        ast.break_stmt(span, label)
    }
}
impl<'a> CloneIn<'a> for ContinueStmt {
    type Cloned = ContinueStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        ast.continue_stmt(span, label)
    }
}
impl<'a> CloneIn<'a> for IfStmt {
    type Cloned = IfStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let cons = self.cons(ast).clone_in(ast);
        let alt = self.alt(ast).clone_in(ast);
        ast.if_stmt(span, test, cons, alt)
    }
}
impl<'a> CloneIn<'a> for SwitchStmt {
    type Cloned = SwitchStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let discriminant = self.discriminant(ast).clone_in(ast);
        let cases = self.cases(ast).clone_in(ast);
        ast.switch_stmt(span, discriminant, cases)
    }
}
impl<'a> CloneIn<'a> for ThrowStmt {
    type Cloned = ThrowStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.throw_stmt(span, arg)
    }
}
impl<'a> CloneIn<'a> for TryStmt {
    type Cloned = TryStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let block = self.block(ast).clone_in(ast);
        let handler = self.handler(ast).clone_in(ast);
        let finalizer = self.finalizer(ast).clone_in(ast);
        ast.try_stmt(span, block, handler, finalizer)
    }
}
impl<'a> CloneIn<'a> for WhileStmt {
    type Cloned = WhileStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.while_stmt(span, test, body)
    }
}
impl<'a> CloneIn<'a> for DoWhileStmt {
    type Cloned = DoWhileStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.do_while_stmt(span, test, body)
    }
}
impl<'a> CloneIn<'a> for ForStmt {
    type Cloned = ForStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let init = self.init(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let update = self.update(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_stmt(span, init, test, update, body)
    }
}
impl<'a> CloneIn<'a> for ForInStmt {
    type Cloned = ForInStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_in_stmt(span, left, right, body)
    }
}
impl<'a> CloneIn<'a> for ForOfStmt {
    type Cloned = ForOfStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let is_await = self.is_await(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_of_stmt(span, is_await, left, right, body)
    }
}
impl<'a> CloneIn<'a> for SwitchCase {
    type Cloned = SwitchCase;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let cons = self.cons(ast).clone_in(ast);
        ast.switch_case(span, test, cons)
    }
}
impl<'a> CloneIn<'a> for CatchClause {
    type Cloned = CatchClause;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let param = self.param(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.catch_clause(span, param, body)
    }
}
impl<'a> CloneIn<'a> for ForHead {
    type Cloned = ForHead;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.for_head_data(*self);
        match data {
            ForHeadData::VarDecl(it) => Self::VarDecl(it.clone_in(ast)),
            ForHeadData::UsingDecl(it) => Self::UsingDecl(it.clone_in(ast)),
            ForHeadData::Pat(it) => Self::Pat(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for VarDeclOrExpr {
    type Cloned = VarDeclOrExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.var_decl_or_expr_data(*self);
        match data {
            VarDeclOrExprData::VarDecl(it) => Self::VarDecl(it.clone_in(ast)),
            VarDeclOrExprData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for Decl {
    type Cloned = Decl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.decl_data(*self);
        match data {
            DeclData::Class(it) => Self::Class(it.clone_in(ast)),
            DeclData::Fn(it) => Self::Fn(it.clone_in(ast)),
            DeclData::Var(it) => Self::Var(it.clone_in(ast)),
            DeclData::Using(it) => Self::Using(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for FnDecl {
    type Cloned = FnDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ident = self.ident(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        ast.fn_decl(span, ident, declare, function)
    }
}
impl<'a> CloneIn<'a> for ClassDecl {
    type Cloned = ClassDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ident = self.ident(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let class = self.class(ast).clone_in(ast);
        ast.class_decl(span, ident, declare, class)
    }
}
impl<'a> CloneIn<'a> for VarDecl {
    type Cloned = VarDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let decls = self.decls(ast).clone_in(ast);
        ast.var_decl(span, kind, declare, decls)
    }
}
impl<'a> CloneIn<'a> for VarDeclarator {
    type Cloned = VarDeclarator;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let init = self.init(ast).clone_in(ast);
        ast.var_declarator(span, name, init)
    }
}
impl<'a> CloneIn<'a> for UsingDecl {
    type Cloned = UsingDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let is_await = self.is_await(ast).clone_in(ast);
        let decls = self.decls(ast).clone_in(ast);
        ast.using_decl(span, is_await, decls)
    }
}
impl<'a> CloneIn<'a> for Expr {
    type Cloned = Expr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.expr_data(*self);
        match data {
            ExprData::This(it) => Self::This(it.clone_in(ast)),
            ExprData::Array(it) => Self::Array(it.clone_in(ast)),
            ExprData::Object(it) => Self::Object(it.clone_in(ast)),
            ExprData::Fn(it) => Self::Fn(it.clone_in(ast)),
            ExprData::Unary(it) => Self::Unary(it.clone_in(ast)),
            ExprData::Update(it) => Self::Update(it.clone_in(ast)),
            ExprData::Bin(it) => Self::Bin(it.clone_in(ast)),
            ExprData::Assign(it) => Self::Assign(it.clone_in(ast)),
            ExprData::Member(it) => Self::Member(it.clone_in(ast)),
            ExprData::SuperProp(it) => Self::SuperProp(it.clone_in(ast)),
            ExprData::Cond(it) => Self::Cond(it.clone_in(ast)),
            ExprData::Import(it) => Self::Import(it.clone_in(ast)),
            ExprData::Call(it) => Self::Call(it.clone_in(ast)),
            ExprData::New(it) => Self::New(it.clone_in(ast)),
            ExprData::Seq(it) => Self::Seq(it.clone_in(ast)),
            ExprData::Ident(it) => Self::Ident(it.clone_in(ast)),
            ExprData::Lit(it) => Self::Lit(it.clone_in(ast)),
            ExprData::Tpl(it) => Self::Tpl(it.clone_in(ast)),
            ExprData::TaggedTpl(it) => Self::TaggedTpl(it.clone_in(ast)),
            ExprData::Arrow(it) => Self::Arrow(it.clone_in(ast)),
            ExprData::Class(it) => Self::Class(it.clone_in(ast)),
            ExprData::Yield(it) => Self::Yield(it.clone_in(ast)),
            ExprData::MetaProp(it) => Self::MetaProp(it.clone_in(ast)),
            ExprData::Await(it) => Self::Await(it.clone_in(ast)),
            ExprData::Paren(it) => Self::Paren(it.clone_in(ast)),
            ExprData::JSXMember(it) => Self::JSXMember(it.clone_in(ast)),
            ExprData::JSXNamespacedName(it) => Self::JSXNamespacedName(it.clone_in(ast)),
            ExprData::JSXEmpty(it) => Self::JSXEmpty(it.clone_in(ast)),
            ExprData::JSXElement(it) => Self::JSXElement(it.clone_in(ast)),
            ExprData::JSXFragment(it) => Self::JSXFragment(it.clone_in(ast)),
            ExprData::PrivateName(it) => Self::PrivateName(it.clone_in(ast)),
            ExprData::OptChain(it) => Self::OptChain(it.clone_in(ast)),
            ExprData::Invalid(it) => Self::Invalid(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ThisExpr {
    type Cloned = ThisExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.this_expr(span)
    }
}
impl<'a> CloneIn<'a> for ArrayLit {
    type Cloned = ArrayLit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elems = self.elems(ast).clone_in(ast);
        ast.array_lit(span, elems)
    }
}
impl<'a> CloneIn<'a> for ObjectLit {
    type Cloned = ObjectLit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let props = self.props(ast).clone_in(ast);
        ast.object_lit(span, props)
    }
}
impl<'a> CloneIn<'a> for PropOrSpread {
    type Cloned = PropOrSpread;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.prop_or_spread_data(*self);
        match data {
            PropOrSpreadData::Spread(it) => Self::Spread(it.clone_in(ast)),
            PropOrSpreadData::Prop(it) => Self::Prop(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for SpreadElement {
    type Cloned = SpreadElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let dot_3_token = self.dot_3_token(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.spread_element(span, dot_3_token, expr)
    }
}
impl<'a> CloneIn<'a> for UnaryExpr {
    type Cloned = UnaryExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let op = self.op(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.unary_expr(span, op, arg)
    }
}
impl<'a> CloneIn<'a> for UpdateExpr {
    type Cloned = UpdateExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let op = self.op(ast).clone_in(ast);
        let prefix = self.prefix(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.update_expr(span, op, prefix, arg)
    }
}
impl<'a> CloneIn<'a> for BinExpr {
    type Cloned = BinExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let op = self.op(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.bin_expr(span, op, left, right)
    }
}
impl<'a> CloneIn<'a> for FnExpr {
    type Cloned = FnExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ident = self.ident(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        ast.fn_expr(span, ident, function)
    }
}
impl<'a> CloneIn<'a> for ClassExpr {
    type Cloned = ClassExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ident = self.ident(ast).clone_in(ast);
        let class = self.class(ast).clone_in(ast);
        ast.class_expr(span, ident, class)
    }
}
impl<'a> CloneIn<'a> for AssignExpr {
    type Cloned = AssignExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let op = self.op(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.assign_expr(span, op, left, right)
    }
}
impl<'a> CloneIn<'a> for MemberExpr {
    type Cloned = MemberExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let obj = self.obj(ast).clone_in(ast);
        let prop = self.prop(ast).clone_in(ast);
        ast.member_expr(span, obj, prop)
    }
}
impl<'a> CloneIn<'a> for MemberProp {
    type Cloned = MemberProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.member_prop_data(*self);
        match data {
            MemberPropData::Ident(it) => Self::Ident(it.clone_in(ast)),
            MemberPropData::PrivateName(it) => Self::PrivateName(it.clone_in(ast)),
            MemberPropData::Computed(it) => Self::Computed(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for SuperPropExpr {
    type Cloned = SuperPropExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let obj = self.obj(ast).clone_in(ast);
        let prop = self.prop(ast).clone_in(ast);
        ast.super_prop_expr(span, obj, prop)
    }
}
impl<'a> CloneIn<'a> for SuperProp {
    type Cloned = SuperProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.super_prop_data(*self);
        match data {
            SuperPropData::Ident(it) => Self::Ident(it.clone_in(ast)),
            SuperPropData::Computed(it) => Self::Computed(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for CondExpr {
    type Cloned = CondExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let cons = self.cons(ast).clone_in(ast);
        let alt = self.alt(ast).clone_in(ast);
        ast.cond_expr(span, test, cons, alt)
    }
}
impl<'a> CloneIn<'a> for ImportExpr {
    type Cloned = ImportExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let import_span = self.import_span(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let options = self.options(ast).clone_in(ast);
        let phase = self.phase(ast).clone_in(ast);
        ast.import_expr(span, import_span, source, options, phase)
    }
}
impl<'a> CloneIn<'a> for CallExpr {
    type Cloned = CallExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let args = self.args(ast).clone_in(ast);
        ast.call_expr(span, callee, args)
    }
}
impl<'a> CloneIn<'a> for NewExpr {
    type Cloned = NewExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let args = self.args(ast).clone_in(ast);
        ast.new_expr(span, callee, args)
    }
}
impl<'a> CloneIn<'a> for SeqExpr {
    type Cloned = SeqExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exprs = self.exprs(ast).clone_in(ast);
        ast.seq_expr(span, exprs)
    }
}
impl<'a> CloneIn<'a> for ArrowExpr {
    type Cloned = ArrowExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let is_async = self.is_async(ast).clone_in(ast);
        ast.arrow_expr(span, params, body, is_async)
    }
}
impl<'a> CloneIn<'a> for YieldExpr {
    type Cloned = YieldExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        let delegate = self.delegate(ast).clone_in(ast);
        ast.yield_expr(span, arg, delegate)
    }
}
impl<'a> CloneIn<'a> for MetaPropExpr {
    type Cloned = MetaPropExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        ast.meta_prop_expr(span, kind)
    }
}
impl<'a> CloneIn<'a> for AwaitExpr {
    type Cloned = AwaitExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.await_expr(span, arg)
    }
}
impl<'a> CloneIn<'a> for Tpl {
    type Cloned = Tpl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exprs = self.exprs(ast).clone_in(ast);
        let quasis = self.quasis(ast).clone_in(ast);
        ast.tpl(span, exprs, quasis)
    }
}
impl<'a> CloneIn<'a> for TaggedTpl {
    type Cloned = TaggedTpl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let tag = self.tag(ast).clone_in(ast);
        let tpl = self.tpl(ast).clone_in(ast);
        ast.tagged_tpl(span, tag, tpl)
    }
}
impl<'a> CloneIn<'a> for TplElement {
    type Cloned = TplElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let tail = self.tail(ast).clone_in(ast);
        let cooked = self.cooked(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.tpl_element(span, tail, cooked, raw)
    }
}
impl<'a> CloneIn<'a> for ParenExpr {
    type Cloned = ParenExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.paren_expr(span, expr)
    }
}
impl<'a> CloneIn<'a> for Callee {
    type Cloned = Callee;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.callee_data(*self);
        match data {
            CalleeData::Super(it) => Self::Super(it.clone_in(ast)),
            CalleeData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for Super {
    type Cloned = Super;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.super_(span)
    }
}
impl<'a> CloneIn<'a> for ExprOrSpread {
    type Cloned = ExprOrSpread;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let spread = self.spread(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.expr_or_spread(span, spread, expr)
    }
}
impl<'a> CloneIn<'a> for BlockStmtOrExpr {
    type Cloned = BlockStmtOrExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.block_stmt_or_expr_data(*self);
        match data {
            BlockStmtOrExprData::BlockStmt(it) => Self::BlockStmt(it.clone_in(ast)),
            BlockStmtOrExprData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for AssignTarget {
    type Cloned = AssignTarget;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.assign_target_data(*self);
        match data {
            AssignTargetData::Simple(it) => Self::Simple(it.clone_in(ast)),
            AssignTargetData::Pat(it) => Self::Pat(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for AssignTargetPat {
    type Cloned = AssignTargetPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.assign_target_pat_data(*self);
        match data {
            AssignTargetPatData::Array(it) => Self::Array(it.clone_in(ast)),
            AssignTargetPatData::Object(it) => Self::Object(it.clone_in(ast)),
            AssignTargetPatData::Invalid(it) => Self::Invalid(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for SimpleAssignTarget {
    type Cloned = SimpleAssignTarget;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.simple_assign_target_data(*self);
        match data {
            SimpleAssignTargetData::Ident(it) => Self::Ident(it.clone_in(ast)),
            SimpleAssignTargetData::Member(it) => Self::Member(it.clone_in(ast)),
            SimpleAssignTargetData::SuperProp(it) => Self::SuperProp(it.clone_in(ast)),
            SimpleAssignTargetData::Paren(it) => Self::Paren(it.clone_in(ast)),
            SimpleAssignTargetData::OptChain(it) => Self::OptChain(it.clone_in(ast)),
            SimpleAssignTargetData::Invalid(it) => Self::Invalid(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for OptChainExpr {
    type Cloned = OptChainExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let base = self.base(ast).clone_in(ast);
        ast.opt_chain_expr(span, optional, base)
    }
}
impl<'a> CloneIn<'a> for OptChainBase {
    type Cloned = OptChainBase;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.opt_chain_base_data(*self);
        match data {
            OptChainBaseData::Member(it) => Self::Member(it.clone_in(ast)),
            OptChainBaseData::Call(it) => Self::Call(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for OptCall {
    type Cloned = OptCall;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let args = self.args(ast).clone_in(ast);
        ast.opt_call(span, callee, args)
    }
}
impl<'a> CloneIn<'a> for Invalid {
    type Cloned = Invalid;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.invalid(span)
    }
}
impl<'a> CloneIn<'a> for Function {
    type Cloned = Function;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let is_generator = self.is_generator(ast).clone_in(ast);
        let is_async = self.is_async(ast).clone_in(ast);
        ast.function(span, params, decorators, body, is_generator, is_async)
    }
}
impl<'a> CloneIn<'a> for Param {
    type Cloned = Param;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let pat = self.pat(ast).clone_in(ast);
        ast.param(span, decorators, pat)
    }
}
impl<'a> CloneIn<'a> for ParamOrTsParamProp {
    type Cloned = ParamOrTsParamProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.param_or_ts_param_prop_data(*self);
        match data {
            ParamOrTsParamPropData::Param(it) => Self::Param(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for Class {
    type Cloned = Class;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let super_class = self.super_class(ast).clone_in(ast);
        let is_abstract = self.is_abstract(ast).clone_in(ast);
        ast.class(span, decorators, body, super_class, is_abstract)
    }
}
impl<'a> CloneIn<'a> for ClassMember {
    type Cloned = ClassMember;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.class_member_data(*self);
        match data {
            ClassMemberData::Constructor(it) => Self::Constructor(it.clone_in(ast)),
            ClassMemberData::Method(it) => Self::Method(it.clone_in(ast)),
            ClassMemberData::PrivateMethod(it) => Self::PrivateMethod(it.clone_in(ast)),
            ClassMemberData::ClassProp(it) => Self::ClassProp(it.clone_in(ast)),
            ClassMemberData::PrivateProp(it) => Self::PrivateProp(it.clone_in(ast)),
            ClassMemberData::Empty(it) => Self::Empty(it.clone_in(ast)),
            ClassMemberData::StaticBlock(it) => Self::StaticBlock(it.clone_in(ast)),
            ClassMemberData::AutoAccessor(it) => Self::AutoAccessor(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ClassProp {
    type Cloned = ClassProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        ast.class_prop(span, key, value, is_static, decorators)
    }
}
impl<'a> CloneIn<'a> for PrivateProp {
    type Cloned = PrivateProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        ast.private_prop(span, key, value, is_static, decorators)
    }
}
impl<'a> CloneIn<'a> for ClassMethod {
    type Cloned = ClassMethod;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        ast.class_method(span, key, function, kind, is_static)
    }
}
impl<'a> CloneIn<'a> for PrivateMethod {
    type Cloned = PrivateMethod;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        ast.private_method(span, key, function, kind, is_static)
    }
}
impl<'a> CloneIn<'a> for Constructor {
    type Cloned = Constructor;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.constructor(span, key, params, body)
    }
}
impl<'a> CloneIn<'a> for Decorator {
    type Cloned = Decorator;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.decorator(span, expr)
    }
}
impl<'a> CloneIn<'a> for StaticBlock {
    type Cloned = StaticBlock;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.static_block(span, body)
    }
}
impl<'a> CloneIn<'a> for Key {
    type Cloned = Key;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.key_data(*self);
        match data {
            KeyData::Private(it) => Self::Private(it.clone_in(ast)),
            KeyData::Public(it) => Self::Public(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for AutoAccessor {
    type Cloned = AutoAccessor;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        ast.auto_accessor(span, key, value, is_static, decorators)
    }
}
impl<'a> CloneIn<'a> for Prop {
    type Cloned = Prop;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.prop_data(*self);
        match data {
            PropData::Shorthand(it) => Self::Shorthand(it.clone_in(ast)),
            PropData::KeyValue(it) => Self::KeyValue(it.clone_in(ast)),
            PropData::Assign(it) => Self::Assign(it.clone_in(ast)),
            PropData::Getter(it) => Self::Getter(it.clone_in(ast)),
            PropData::Setter(it) => Self::Setter(it.clone_in(ast)),
            PropData::Method(it) => Self::Method(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for KeyValueProp {
    type Cloned = KeyValueProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.key_value_prop(span, key, value)
    }
}
impl<'a> CloneIn<'a> for AssignProp {
    type Cloned = AssignProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.assign_prop(span, key, value)
    }
}
impl<'a> CloneIn<'a> for GetterProp {
    type Cloned = GetterProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        ast.getter_prop(span, key, function)
    }
}
impl<'a> CloneIn<'a> for SetterProp {
    type Cloned = SetterProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        ast.setter_prop(span, key, function)
    }
}
impl<'a> CloneIn<'a> for MethodProp {
    type Cloned = MethodProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        ast.method_prop(span, key, function)
    }
}
impl<'a> CloneIn<'a> for PropName {
    type Cloned = PropName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.prop_name_data(*self);
        match data {
            PropNameData::Ident(it) => Self::Ident(it.clone_in(ast)),
            PropNameData::Str(it) => Self::Str(it.clone_in(ast)),
            PropNameData::Num(it) => Self::Num(it.clone_in(ast)),
            PropNameData::Computed(it) => Self::Computed(it.clone_in(ast)),
            PropNameData::BigInt(it) => Self::BigInt(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ComputedPropName {
    type Cloned = ComputedPropName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.computed_prop_name(span, expr)
    }
}
impl<'a> CloneIn<'a> for Pat {
    type Cloned = Pat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.pat_data(*self);
        match data {
            PatData::Ident(it) => Self::Ident(it.clone_in(ast)),
            PatData::Array(it) => Self::Array(it.clone_in(ast)),
            PatData::Rest(it) => Self::Rest(it.clone_in(ast)),
            PatData::Object(it) => Self::Object(it.clone_in(ast)),
            PatData::Assign(it) => Self::Assign(it.clone_in(ast)),
            PatData::Invalid(it) => Self::Invalid(it.clone_in(ast)),
            PatData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for ArrayPat {
    type Cloned = ArrayPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elems = self.elems(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.array_pat(span, elems, optional)
    }
}
impl<'a> CloneIn<'a> for ObjectPat {
    type Cloned = ObjectPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let props = self.props(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.object_pat(span, props, optional)
    }
}
impl<'a> CloneIn<'a> for AssignPat {
    type Cloned = AssignPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.assign_pat(span, left, right)
    }
}
impl<'a> CloneIn<'a> for RestPat {
    type Cloned = RestPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let dot_3_token = self.dot_3_token(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.rest_pat(span, dot_3_token, arg)
    }
}
impl<'a> CloneIn<'a> for ObjectPatProp {
    type Cloned = ObjectPatProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.object_pat_prop_data(*self);
        match data {
            ObjectPatPropData::KeyValue(it) => Self::KeyValue(it.clone_in(ast)),
            ObjectPatPropData::Assign(it) => Self::Assign(it.clone_in(ast)),
            ObjectPatPropData::Rest(it) => Self::Rest(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for KeyValuePatProp {
    type Cloned = KeyValuePatProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.key_value_pat_prop(span, key, value)
    }
}
impl<'a> CloneIn<'a> for AssignPatProp {
    type Cloned = AssignPatProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.assign_pat_prop(span, key, value)
    }
}
impl<'a> CloneIn<'a> for Ident {
    type Cloned = Ident;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let sym = self.sym(ast).clone_in(ast);
        ast.ident(span, sym)
    }
}
impl<'a> CloneIn<'a> for IdentName {
    type Cloned = IdentName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let sym = self.sym(ast).clone_in(ast);
        ast.ident_name(span, sym)
    }
}
impl<'a> CloneIn<'a> for PrivateName {
    type Cloned = PrivateName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.private_name(span, name)
    }
}
impl<'a> CloneIn<'a> for BindingIdent {
    type Cloned = BindingIdent;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        ast.binding_ident(span, id)
    }
}
impl<'a> CloneIn<'a> for Lit {
    type Cloned = Lit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.lit_data(*self);
        match data {
            LitData::Str(it) => Self::Str(it.clone_in(ast)),
            LitData::Bool(it) => Self::Bool(it.clone_in(ast)),
            LitData::Null(it) => Self::Null(it.clone_in(ast)),
            LitData::Num(it) => Self::Num(it.clone_in(ast)),
            LitData::BigInt(it) => Self::BigInt(it.clone_in(ast)),
            LitData::Regex(it) => Self::Regex(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for Str {
    type Cloned = Str;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.str(span, value, raw)
    }
}
impl<'a> CloneIn<'a> for Bool {
    type Cloned = Bool;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.bool(span, value)
    }
}
impl<'a> CloneIn<'a> for Null {
    type Cloned = Null;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.null(span)
    }
}
impl<'a> CloneIn<'a> for Number {
    type Cloned = Number;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.number(span, value, raw)
    }
}
impl<'a> CloneIn<'a> for BigInt {
    type Cloned = BigInt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.big_int(span, value, raw)
    }
}
impl<'a> CloneIn<'a> for Regex {
    type Cloned = Regex;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exp = self.exp(ast).clone_in(ast);
        let flags = self.flags(ast).clone_in(ast);
        ast.regex(span, exp, flags)
    }
}
impl<'a> CloneIn<'a> for JSXObject {
    type Cloned = JSXObject;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_object_data(*self);
        match data {
            JSXObjectData::JSXMemberExpr(it) => Self::JSXMemberExpr(it.clone_in(ast)),
            JSXObjectData::Ident(it) => Self::Ident(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for JSXMemberExpr {
    type Cloned = JSXMemberExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let obj = self.obj(ast).clone_in(ast);
        let prop = self.prop(ast).clone_in(ast);
        ast.jsx_member_expr(span, obj, prop)
    }
}
impl<'a> CloneIn<'a> for JSXNamespacedName {
    type Cloned = JSXNamespacedName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ns = self.ns(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_namespaced_name(span, ns, name)
    }
}
impl<'a> CloneIn<'a> for JSXEmptyExpr {
    type Cloned = JSXEmptyExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_empty_expr(span)
    }
}
impl<'a> CloneIn<'a> for JSXExprContainer {
    type Cloned = JSXExprContainer;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.jsx_expr_container(span, expr)
    }
}
impl<'a> CloneIn<'a> for JSXExpr {
    type Cloned = JSXExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_expr_data(*self);
        match data {
            JSXExprData::JSXEmptyExpr(it) => Self::JSXEmptyExpr(it.clone_in(ast)),
            JSXExprData::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for JSXSpreadChild {
    type Cloned = JSXSpreadChild;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.jsx_spread_child(span, expr)
    }
}
impl<'a> CloneIn<'a> for JSXElementName {
    type Cloned = JSXElementName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_element_name_data(*self);
        match data {
            JSXElementNameData::Ident(it) => Self::Ident(it.clone_in(ast)),
            JSXElementNameData::JSXMemberExpr(it) => Self::JSXMemberExpr(it.clone_in(ast)),
            JSXElementNameData::JSXNamespacedName(it) => Self::JSXNamespacedName(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for JSXOpeningElement {
    type Cloned = JSXOpeningElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let attrs = self.attrs(ast).clone_in(ast);
        let self_closing = self.self_closing(ast).clone_in(ast);
        ast.jsx_opening_element(span, name, attrs, self_closing)
    }
}
impl<'a> CloneIn<'a> for JSXAttrOrSpread {
    type Cloned = JSXAttrOrSpread;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_attr_or_spread_data(*self);
        match data {
            JSXAttrOrSpreadData::JSXAttr(it) => Self::JSXAttr(it.clone_in(ast)),
            JSXAttrOrSpreadData::SpreadElement(it) => Self::SpreadElement(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for JSXClosingElement {
    type Cloned = JSXClosingElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_closing_element(span, name)
    }
}
impl<'a> CloneIn<'a> for JSXAttr {
    type Cloned = JSXAttr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.jsx_attr(span, name, value)
    }
}
impl<'a> CloneIn<'a> for JSXAttrName {
    type Cloned = JSXAttrName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_attr_name_data(*self);
        match data {
            JSXAttrNameData::Ident(it) => Self::Ident(it.clone_in(ast)),
            JSXAttrNameData::JSXNamespacedName(it) => Self::JSXNamespacedName(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for JSXAttrValue {
    type Cloned = JSXAttrValue;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_attr_value_data(*self);
        match data {
            JSXAttrValueData::Str(it) => Self::Str(it.clone_in(ast)),
            JSXAttrValueData::JSXExprContainer(it) => Self::JSXExprContainer(it.clone_in(ast)),
            JSXAttrValueData::JSXElement(it) => Self::JSXElement(it.clone_in(ast)),
            JSXAttrValueData::JSXFragment(it) => Self::JSXFragment(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for JSXText {
    type Cloned = JSXText;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.jsx_text(span, value, raw)
    }
}
impl<'a> CloneIn<'a> for JSXElement {
    type Cloned = JSXElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let opening = self.opening(ast).clone_in(ast);
        let children = self.children(ast).clone_in(ast);
        let closing = self.closing(ast).clone_in(ast);
        ast.jsx_element(span, opening, children, closing)
    }
}
impl<'a> CloneIn<'a> for JSXElementChild {
    type Cloned = JSXElementChild;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let data = ast.jsx_element_child_data(*self);
        match data {
            JSXElementChildData::JSXText(it) => Self::JSXText(it.clone_in(ast)),
            JSXElementChildData::JSXExprContainer(it) => Self::JSXExprContainer(it.clone_in(ast)),
            JSXElementChildData::JSXSpreadChild(it) => Self::JSXSpreadChild(it.clone_in(ast)),
            JSXElementChildData::JSXElement(it) => Self::JSXElement(it.clone_in(ast)),
            JSXElementChildData::JSXFragment(it) => Self::JSXFragment(it.clone_in(ast)),
        }
    }
}
impl<'a> CloneIn<'a> for JSXFragment {
    type Cloned = JSXFragment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let opening = self.opening(ast).clone_in(ast);
        let children = self.children(ast).clone_in(ast);
        let closing = self.closing(ast).clone_in(ast);
        ast.jsx_fragment(span, opening, children, closing)
    }
}
impl<'a> CloneIn<'a> for JSXOpeningFragment {
    type Cloned = JSXOpeningFragment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_opening_fragment(span)
    }
}
impl<'a> CloneIn<'a> for JSXClosingFragment {
    type Cloned = JSXClosingFragment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_closing_fragment(span)
    }
}
