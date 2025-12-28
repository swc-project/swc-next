#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, NodeKind};
use crate::{ast::*, node_id::*};
impl CloneIn for Program {
    type Cloned = Program;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Module(it) => Self::Module(it.clone_in(ast)),
            Self::Script(it) => Self::Script(it.clone_in(ast)),
        }
    }
}
impl CloneIn for Module {
    type Cloned = Module;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let shebang = self.shebang(ast).clone_in(ast);
        ast.module(span, body, shebang)
    }
}
impl CloneIn for Script {
    type Cloned = Script;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let shebang = self.shebang(ast).clone_in(ast);
        ast.script(span, body, shebang)
    }
}
impl CloneIn for ModuleItem {
    type Cloned = ModuleItem;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::ModuleDecl(it) => Self::ModuleDecl(it.clone_in(ast)),
            Self::Stmt(it) => Self::Stmt(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ModuleDecl {
    type Cloned = ModuleDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Import(it) => Self::Import(it.clone_in(ast)),
            Self::ExportDecl(it) => Self::ExportDecl(it.clone_in(ast)),
            Self::ExportNamed(it) => Self::ExportNamed(it.clone_in(ast)),
            Self::ExportDefaultDecl(it) => Self::ExportDefaultDecl(it.clone_in(ast)),
            Self::ExportDefaultExpr(it) => Self::ExportDefaultExpr(it.clone_in(ast)),
            Self::ExportAll(it) => Self::ExportAll(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ImportDecl {
    type Cloned = ImportDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let specifiers = self.specifiers(ast).clone_in(ast);
        let src = self.src(ast).clone_in(ast);
        let type_only = self.type_only(ast).clone_in(ast);
        let with = self.with(ast).clone_in(ast);
        let phase = self.phase(ast).clone_in(ast);
        ast.import_decl(span, specifiers, src, type_only, with, phase)
    }
}
impl CloneIn for ImportSpecifier {
    type Cloned = ImportSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Named(it) => Self::Named(it.clone_in(ast)),
            Self::Default(it) => Self::Default(it.clone_in(ast)),
            Self::Namespace(it) => Self::Namespace(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ImportNamedSpecifier {
    type Cloned = ImportNamedSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        let imported = self.imported(ast).clone_in(ast);
        let is_type_only = self.is_type_only(ast).clone_in(ast);
        ast.import_named_specifier(span, local, imported, is_type_only)
    }
}
impl CloneIn for ImportDefaultSpecifier {
    type Cloned = ImportDefaultSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        ast.import_default_specifier(span, local)
    }
}
impl CloneIn for ImportStarAsSpecifier {
    type Cloned = ImportStarAsSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        ast.import_star_as_specifier(span, local)
    }
}
impl CloneIn for ExportDecl {
    type Cloned = ExportDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decl = self.decl(ast).clone_in(ast);
        ast.export_decl(span, decl)
    }
}
impl CloneIn for NamedExport {
    type Cloned = NamedExport;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let specifiers = self.specifiers(ast).clone_in(ast);
        let src = self.src(ast).clone_in(ast);
        let type_only = self.type_only(ast).clone_in(ast);
        let with = self.with(ast).clone_in(ast);
        ast.named_export(span, specifiers, src, type_only, with)
    }
}
impl CloneIn for ExportSpecifier {
    type Cloned = ExportSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Namespace(it) => Self::Namespace(it.clone_in(ast)),
            Self::Default(it) => Self::Default(it.clone_in(ast)),
            Self::Named(it) => Self::Named(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ExportNamespaceSpecifier {
    type Cloned = ExportNamespaceSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.export_namespace_specifier(span, name)
    }
}
impl CloneIn for ModuleExportName {
    type Cloned = ModuleExportName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Str(it) => Self::Str(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ExportDefaultSpecifier {
    type Cloned = ExportDefaultSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exported = self.exported(ast).clone_in(ast);
        ast.export_default_specifier(span, exported)
    }
}
impl CloneIn for ExportNamedSpecifier {
    type Cloned = ExportNamedSpecifier;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let orig = self.orig(ast).clone_in(ast);
        let exported = self.exported(ast).clone_in(ast);
        let is_type_only = self.is_type_only(ast).clone_in(ast);
        ast.export_named_specifier(span, orig, exported, is_type_only)
    }
}
impl CloneIn for ExportDefaultDecl {
    type Cloned = ExportDefaultDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decl = self.decl(ast).clone_in(ast);
        ast.export_default_decl(span, decl)
    }
}
impl CloneIn for DefaultDecl {
    type Cloned = DefaultDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Class(it) => Self::Class(it.clone_in(ast)),
            Self::Fn(it) => Self::Fn(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ExportDefaultExpr {
    type Cloned = ExportDefaultExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.export_default_expr(span, expr)
    }
}
impl CloneIn for ExportAll {
    type Cloned = ExportAll;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let src = self.src(ast).clone_in(ast);
        let type_only = self.type_only(ast).clone_in(ast);
        let with = self.with(ast).clone_in(ast);
        ast.export_all(span, src, type_only, with)
    }
}
impl CloneIn for BlockStmt {
    type Cloned = BlockStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let stmts = self.stmts(ast).clone_in(ast);
        ast.block_stmt(span, stmts)
    }
}
impl CloneIn for Stmt {
    type Cloned = Stmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Block(it) => Self::Block(it.clone_in(ast)),
            Self::Empty(it) => Self::Empty(it.clone_in(ast)),
            Self::Debugger(it) => Self::Debugger(it.clone_in(ast)),
            Self::With(it) => Self::With(it.clone_in(ast)),
            Self::Return(it) => Self::Return(it.clone_in(ast)),
            Self::Labeled(it) => Self::Labeled(it.clone_in(ast)),
            Self::Break(it) => Self::Break(it.clone_in(ast)),
            Self::Continue(it) => Self::Continue(it.clone_in(ast)),
            Self::If(it) => Self::If(it.clone_in(ast)),
            Self::Switch(it) => Self::Switch(it.clone_in(ast)),
            Self::Throw(it) => Self::Throw(it.clone_in(ast)),
            Self::Try(it) => Self::Try(it.clone_in(ast)),
            Self::While(it) => Self::While(it.clone_in(ast)),
            Self::DoWhile(it) => Self::DoWhile(it.clone_in(ast)),
            Self::For(it) => Self::For(it.clone_in(ast)),
            Self::ForIn(it) => Self::ForIn(it.clone_in(ast)),
            Self::ForOf(it) => Self::ForOf(it.clone_in(ast)),
            Self::Decl(it) => Self::Decl(it.clone_in(ast)),
            Self::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ExprStmt {
    type Cloned = ExprStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.expr_stmt(span, expr)
    }
}
impl CloneIn for EmptyStmt {
    type Cloned = EmptyStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.empty_stmt(span)
    }
}
impl CloneIn for DebuggerStmt {
    type Cloned = DebuggerStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.debugger_stmt(span)
    }
}
impl CloneIn for WithStmt {
    type Cloned = WithStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let obj = self.obj(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.with_stmt(span, obj, body)
    }
}
impl CloneIn for ReturnStmt {
    type Cloned = ReturnStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.return_stmt(span, arg)
    }
}
impl CloneIn for LabeledStmt {
    type Cloned = LabeledStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.labeled_stmt(span, label, body)
    }
}
impl CloneIn for BreakStmt {
    type Cloned = BreakStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        ast.break_stmt(span, label)
    }
}
impl CloneIn for ContinueStmt {
    type Cloned = ContinueStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        ast.continue_stmt(span, label)
    }
}
impl CloneIn for IfStmt {
    type Cloned = IfStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let cons = self.cons(ast).clone_in(ast);
        let alt = self.alt(ast).clone_in(ast);
        ast.if_stmt(span, test, cons, alt)
    }
}
impl CloneIn for SwitchStmt {
    type Cloned = SwitchStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let discriminant = self.discriminant(ast).clone_in(ast);
        let cases = self.cases(ast).clone_in(ast);
        ast.switch_stmt(span, discriminant, cases)
    }
}
impl CloneIn for ThrowStmt {
    type Cloned = ThrowStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.throw_stmt(span, arg)
    }
}
impl CloneIn for TryStmt {
    type Cloned = TryStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let block = self.block(ast).clone_in(ast);
        let handler = self.handler(ast).clone_in(ast);
        let finalizer = self.finalizer(ast).clone_in(ast);
        ast.try_stmt(span, block, handler, finalizer)
    }
}
impl CloneIn for WhileStmt {
    type Cloned = WhileStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.while_stmt(span, test, body)
    }
}
impl CloneIn for DoWhileStmt {
    type Cloned = DoWhileStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.do_while_stmt(span, test, body)
    }
}
impl CloneIn for ForStmt {
    type Cloned = ForStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let init = self.init(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let update = self.update(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_stmt(span, init, test, update, body)
    }
}
impl CloneIn for ForInStmt {
    type Cloned = ForInStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_in_stmt(span, left, right, body)
    }
}
impl CloneIn for ForOfStmt {
    type Cloned = ForOfStmt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let is_await = self.is_await(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_of_stmt(span, is_await, left, right, body)
    }
}
impl CloneIn for SwitchCase {
    type Cloned = SwitchCase;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let cons = self.cons(ast).clone_in(ast);
        ast.switch_case(span, test, cons)
    }
}
impl CloneIn for CatchClause {
    type Cloned = CatchClause;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let param = self.param(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.catch_clause(span, param, body)
    }
}
impl CloneIn for ForHead {
    type Cloned = ForHead;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::VarDecl(it) => Self::VarDecl(it.clone_in(ast)),
            Self::UsingDecl(it) => Self::UsingDecl(it.clone_in(ast)),
            Self::Pat(it) => Self::Pat(it.clone_in(ast)),
        }
    }
}
impl CloneIn for VarDeclOrExpr {
    type Cloned = VarDeclOrExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::VarDecl(it) => Self::VarDecl(it.clone_in(ast)),
            Self::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl CloneIn for Decl {
    type Cloned = Decl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Class(it) => Self::Class(it.clone_in(ast)),
            Self::Fn(it) => Self::Fn(it.clone_in(ast)),
            Self::Var(it) => Self::Var(it.clone_in(ast)),
            Self::Using(it) => Self::Using(it.clone_in(ast)),
        }
    }
}
impl CloneIn for FnDecl {
    type Cloned = FnDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ident = self.ident(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        ast.fn_decl(span, ident, declare, function)
    }
}
impl CloneIn for ClassDecl {
    type Cloned = ClassDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ident = self.ident(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let class = self.class(ast).clone_in(ast);
        ast.class_decl(span, ident, declare, class)
    }
}
impl CloneIn for VarDecl {
    type Cloned = VarDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let decls = self.decls(ast).clone_in(ast);
        ast.var_decl(span, kind, declare, decls)
    }
}
impl CloneIn for VarDeclarator {
    type Cloned = VarDeclarator;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let init = self.init(ast).clone_in(ast);
        ast.var_declarator(span, name, init)
    }
}
impl CloneIn for UsingDecl {
    type Cloned = UsingDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let is_await = self.is_await(ast).clone_in(ast);
        let decls = self.decls(ast).clone_in(ast);
        ast.using_decl(span, is_await, decls)
    }
}
impl CloneIn for Expr {
    type Cloned = Expr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::This(it) => Self::This(it.clone_in(ast)),
            Self::Array(it) => Self::Array(it.clone_in(ast)),
            Self::Object(it) => Self::Object(it.clone_in(ast)),
            Self::Fn(it) => Self::Fn(it.clone_in(ast)),
            Self::Unary(it) => Self::Unary(it.clone_in(ast)),
            Self::Update(it) => Self::Update(it.clone_in(ast)),
            Self::Bin(it) => Self::Bin(it.clone_in(ast)),
            Self::Assign(it) => Self::Assign(it.clone_in(ast)),
            Self::Member(it) => Self::Member(it.clone_in(ast)),
            Self::SuperProp(it) => Self::SuperProp(it.clone_in(ast)),
            Self::Cond(it) => Self::Cond(it.clone_in(ast)),
            Self::Call(it) => Self::Call(it.clone_in(ast)),
            Self::New(it) => Self::New(it.clone_in(ast)),
            Self::Seq(it) => Self::Seq(it.clone_in(ast)),
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Lit(it) => Self::Lit(it.clone_in(ast)),
            Self::Tpl(it) => Self::Tpl(it.clone_in(ast)),
            Self::TaggedTpl(it) => Self::TaggedTpl(it.clone_in(ast)),
            Self::Arrow(it) => Self::Arrow(it.clone_in(ast)),
            Self::Class(it) => Self::Class(it.clone_in(ast)),
            Self::Yield(it) => Self::Yield(it.clone_in(ast)),
            Self::MetaProp(it) => Self::MetaProp(it.clone_in(ast)),
            Self::Await(it) => Self::Await(it.clone_in(ast)),
            Self::Paren(it) => Self::Paren(it.clone_in(ast)),
            Self::JSXMember(it) => Self::JSXMember(it.clone_in(ast)),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(it.clone_in(ast)),
            Self::JSXEmpty(it) => Self::JSXEmpty(it.clone_in(ast)),
            Self::JSXElement(it) => Self::JSXElement(it.clone_in(ast)),
            Self::JSXFragment(it) => Self::JSXFragment(it.clone_in(ast)),
            Self::PrivateName(it) => Self::PrivateName(it.clone_in(ast)),
            Self::OptChain(it) => Self::OptChain(it.clone_in(ast)),
            Self::Invalid(it) => Self::Invalid(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ThisExpr {
    type Cloned = ThisExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.this_expr(span)
    }
}
impl CloneIn for ArrayLit {
    type Cloned = ArrayLit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elems = self.elems(ast).clone_in(ast);
        ast.array_lit(span, elems)
    }
}
impl CloneIn for ObjectLit {
    type Cloned = ObjectLit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let props = self.props(ast).clone_in(ast);
        ast.object_lit(span, props)
    }
}
impl CloneIn for PropOrSpread {
    type Cloned = PropOrSpread;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::SpreadElement(it) => Self::SpreadElement(it.clone_in(ast)),
            Self::Prop(it) => Self::Prop(it.clone_in(ast)),
        }
    }
}
impl CloneIn for SpreadElement {
    type Cloned = SpreadElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let dot_3_token = self.dot_3_token(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.spread_element(span, dot_3_token, expr)
    }
}
impl CloneIn for UnaryExpr {
    type Cloned = UnaryExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let op = self.op(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.unary_expr(span, op, arg)
    }
}
impl CloneIn for UpdateExpr {
    type Cloned = UpdateExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let op = self.op(ast).clone_in(ast);
        let prefix = self.prefix(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.update_expr(span, op, prefix, arg)
    }
}
impl CloneIn for BinExpr {
    type Cloned = BinExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let op = self.op(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.bin_expr(span, op, left, right)
    }
}
impl CloneIn for FnExpr {
    type Cloned = FnExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ident = self.ident(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        ast.fn_expr(span, ident, function)
    }
}
impl CloneIn for ClassExpr {
    type Cloned = ClassExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ident = self.ident(ast).clone_in(ast);
        let class = self.class(ast).clone_in(ast);
        ast.class_expr(span, ident, class)
    }
}
impl CloneIn for AssignExpr {
    type Cloned = AssignExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let op = self.op(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.assign_expr(span, op, left, right)
    }
}
impl CloneIn for MemberExpr {
    type Cloned = MemberExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let obj = self.obj(ast).clone_in(ast);
        let prop = self.prop(ast).clone_in(ast);
        ast.member_expr(span, obj, prop)
    }
}
impl CloneIn for MemberProp {
    type Cloned = MemberProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::PrivateName(it) => Self::PrivateName(it.clone_in(ast)),
            Self::Computed(it) => Self::Computed(it.clone_in(ast)),
        }
    }
}
impl CloneIn for SuperPropExpr {
    type Cloned = SuperPropExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let obj = self.obj(ast).clone_in(ast);
        let prop = self.prop(ast).clone_in(ast);
        ast.super_prop_expr(span, obj, prop)
    }
}
impl CloneIn for SuperProp {
    type Cloned = SuperProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Computed(it) => Self::Computed(it.clone_in(ast)),
        }
    }
}
impl CloneIn for CondExpr {
    type Cloned = CondExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let cons = self.cons(ast).clone_in(ast);
        let alt = self.alt(ast).clone_in(ast);
        ast.cond_expr(span, test, cons, alt)
    }
}
impl CloneIn for CallExpr {
    type Cloned = CallExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let args = self.args(ast).clone_in(ast);
        ast.call_expr(span, callee, args)
    }
}
impl CloneIn for NewExpr {
    type Cloned = NewExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let args = self.args(ast).clone_in(ast);
        ast.new_expr(span, callee, args)
    }
}
impl CloneIn for SeqExpr {
    type Cloned = SeqExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exprs = self.exprs(ast).clone_in(ast);
        ast.seq_expr(span, exprs)
    }
}
impl CloneIn for ArrowExpr {
    type Cloned = ArrowExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let is_async = self.is_async(ast).clone_in(ast);
        let is_generator = self.is_generator(ast).clone_in(ast);
        ast.arrow_expr(span, params, body, is_async, is_generator)
    }
}
impl CloneIn for YieldExpr {
    type Cloned = YieldExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        let delegate = self.delegate(ast).clone_in(ast);
        ast.yield_expr(span, arg, delegate)
    }
}
impl CloneIn for MetaPropExpr {
    type Cloned = MetaPropExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        ast.meta_prop_expr(span, kind)
    }
}
impl CloneIn for AwaitExpr {
    type Cloned = AwaitExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.await_expr(span, arg)
    }
}
impl CloneIn for Tpl {
    type Cloned = Tpl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exprs = self.exprs(ast).clone_in(ast);
        let quasis = self.quasis(ast).clone_in(ast);
        ast.tpl(span, exprs, quasis)
    }
}
impl CloneIn for TaggedTpl {
    type Cloned = TaggedTpl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let tag = self.tag(ast).clone_in(ast);
        let tpl = self.tpl(ast).clone_in(ast);
        ast.tagged_tpl(span, tag, tpl)
    }
}
impl CloneIn for TplElement {
    type Cloned = TplElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let tail = self.tail(ast).clone_in(ast);
        let cooked = self.cooked(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.tpl_element(span, tail, cooked, raw)
    }
}
impl CloneIn for ParenExpr {
    type Cloned = ParenExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.paren_expr(span, expr)
    }
}
impl CloneIn for Callee {
    type Cloned = Callee;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Super(it) => Self::Super(it.clone_in(ast)),
            Self::Import(it) => Self::Import(it.clone_in(ast)),
            Self::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl CloneIn for Super {
    type Cloned = Super;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.super_(span)
    }
}
impl CloneIn for Import {
    type Cloned = Import;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let phase = self.phase(ast).clone_in(ast);
        ast.import(span, phase)
    }
}
impl CloneIn for ExprOrSpread {
    type Cloned = ExprOrSpread;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let spread = self.spread(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.expr_or_spread(span, spread, expr)
    }
}
impl CloneIn for SpreadDot3Token {
    type Cloned = SpreadDot3Token;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.spread_dot_3_token(span)
    }
}
impl CloneIn for BlockStmtOrExpr {
    type Cloned = BlockStmtOrExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::BlockStmt(it) => Self::BlockStmt(it.clone_in(ast)),
            Self::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl CloneIn for AssignTarget {
    type Cloned = AssignTarget;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Simple(it) => Self::Simple(it.clone_in(ast)),
            Self::Pat(it) => Self::Pat(it.clone_in(ast)),
        }
    }
}
impl CloneIn for AssignTargetPat {
    type Cloned = AssignTargetPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Array(it) => Self::Array(it.clone_in(ast)),
            Self::Object(it) => Self::Object(it.clone_in(ast)),
            Self::Invalid(it) => Self::Invalid(it.clone_in(ast)),
        }
    }
}
impl CloneIn for SimpleAssignTarget {
    type Cloned = SimpleAssignTarget;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Member(it) => Self::Member(it.clone_in(ast)),
            Self::SuperProp(it) => Self::SuperProp(it.clone_in(ast)),
            Self::Paren(it) => Self::Paren(it.clone_in(ast)),
            Self::OptChain(it) => Self::OptChain(it.clone_in(ast)),
            Self::Invalid(it) => Self::Invalid(it.clone_in(ast)),
        }
    }
}
impl CloneIn for OptChainExpr {
    type Cloned = OptChainExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let base = self.base(ast).clone_in(ast);
        ast.opt_chain_expr(span, optional, base)
    }
}
impl CloneIn for OptChainBase {
    type Cloned = OptChainBase;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Member(it) => Self::Member(it.clone_in(ast)),
            Self::Call(it) => Self::Call(it.clone_in(ast)),
        }
    }
}
impl CloneIn for OptCall {
    type Cloned = OptCall;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let args = self.args(ast).clone_in(ast);
        ast.opt_call(span, callee, args)
    }
}
impl CloneIn for Invalid {
    type Cloned = Invalid;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.invalid(span)
    }
}
impl CloneIn for Function {
    type Cloned = Function;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let is_generator = self.is_generator(ast).clone_in(ast);
        let is_async = self.is_async(ast).clone_in(ast);
        ast.function(span, params, decorators, body, is_generator, is_async)
    }
}
impl CloneIn for Param {
    type Cloned = Param;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let pat = self.pat(ast).clone_in(ast);
        ast.param(span, decorators, pat)
    }
}
impl CloneIn for ParamOrTsParamProp {
    type Cloned = ParamOrTsParamProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Param(it) => Self::Param(it.clone_in(ast)),
        }
    }
}
impl CloneIn for Class {
    type Cloned = Class;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let super_class = self.super_class(ast).clone_in(ast);
        let is_abstract = self.is_abstract(ast).clone_in(ast);
        ast.class(span, decorators, body, super_class, is_abstract)
    }
}
impl CloneIn for ClassMember {
    type Cloned = ClassMember;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Constructor(it) => Self::Constructor(it.clone_in(ast)),
            Self::Method(it) => Self::Method(it.clone_in(ast)),
            Self::PrivateMethod(it) => Self::PrivateMethod(it.clone_in(ast)),
            Self::ClassProp(it) => Self::ClassProp(it.clone_in(ast)),
            Self::PrivateProp(it) => Self::PrivateProp(it.clone_in(ast)),
            Self::Empty(it) => Self::Empty(it.clone_in(ast)),
            Self::StaticBlock(it) => Self::StaticBlock(it.clone_in(ast)),
            Self::AutoAccessor(it) => Self::AutoAccessor(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ClassProp {
    type Cloned = ClassProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        ast.class_prop(span, key, value, is_static, decorators)
    }
}
impl CloneIn for PrivateProp {
    type Cloned = PrivateProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        ast.private_prop(span, key, value, is_static, decorators)
    }
}
impl CloneIn for ClassMethod {
    type Cloned = ClassMethod;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        ast.class_method(span, key, function, kind, is_static)
    }
}
impl CloneIn for PrivateMethod {
    type Cloned = PrivateMethod;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        ast.private_method(span, key, function, kind, is_static)
    }
}
impl CloneIn for Constructor {
    type Cloned = Constructor;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.constructor(span, key, params, body)
    }
}
impl CloneIn for Decorator {
    type Cloned = Decorator;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.decorator(span, expr)
    }
}
impl CloneIn for StaticBlock {
    type Cloned = StaticBlock;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.static_block(span, body)
    }
}
impl CloneIn for Key {
    type Cloned = Key;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Private(it) => Self::Private(it.clone_in(ast)),
            Self::Public(it) => Self::Public(it.clone_in(ast)),
        }
    }
}
impl CloneIn for AutoAccessor {
    type Cloned = AutoAccessor;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        ast.auto_accessor(span, key, value, is_static, decorators)
    }
}
impl CloneIn for Prop {
    type Cloned = Prop;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Shorthand(it) => Self::Shorthand(it.clone_in(ast)),
            Self::KeyValue(it) => Self::KeyValue(it.clone_in(ast)),
            Self::Assign(it) => Self::Assign(it.clone_in(ast)),
            Self::Getter(it) => Self::Getter(it.clone_in(ast)),
            Self::Setter(it) => Self::Setter(it.clone_in(ast)),
            Self::Method(it) => Self::Method(it.clone_in(ast)),
        }
    }
}
impl CloneIn for KeyValueProp {
    type Cloned = KeyValueProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.key_value_prop(span, key, value)
    }
}
impl CloneIn for AssignProp {
    type Cloned = AssignProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.assign_prop(span, key, value)
    }
}
impl CloneIn for GetterProp {
    type Cloned = GetterProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.getter_prop(span, key, body)
    }
}
impl CloneIn for SetterProp {
    type Cloned = SetterProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let this_param = self.this_param(ast).clone_in(ast);
        let param = self.param(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.setter_prop(span, key, this_param, param, body)
    }
}
impl CloneIn for MethodProp {
    type Cloned = MethodProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let function = self.function(ast).clone_in(ast);
        ast.method_prop(span, key, function)
    }
}
impl CloneIn for PropName {
    type Cloned = PropName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Str(it) => Self::Str(it.clone_in(ast)),
            Self::Num(it) => Self::Num(it.clone_in(ast)),
            Self::Computed(it) => Self::Computed(it.clone_in(ast)),
            Self::BigInt(it) => Self::BigInt(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ComputedPropName {
    type Cloned = ComputedPropName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.computed_prop_name(span, expr)
    }
}
impl CloneIn for Pat {
    type Cloned = Pat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Array(it) => Self::Array(it.clone_in(ast)),
            Self::Rest(it) => Self::Rest(it.clone_in(ast)),
            Self::Object(it) => Self::Object(it.clone_in(ast)),
            Self::Assign(it) => Self::Assign(it.clone_in(ast)),
            Self::Invalid(it) => Self::Invalid(it.clone_in(ast)),
            Self::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl CloneIn for ArrayPat {
    type Cloned = ArrayPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elems = self.elems(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.array_pat(span, elems, optional)
    }
}
impl CloneIn for ObjectPat {
    type Cloned = ObjectPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let props = self.props(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.object_pat(span, props, optional)
    }
}
impl CloneIn for AssignPat {
    type Cloned = AssignPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.assign_pat(span, left, right)
    }
}
impl CloneIn for RestPat {
    type Cloned = RestPat;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let dot_3_token = self.dot_3_token(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        ast.rest_pat(span, dot_3_token, arg)
    }
}
impl CloneIn for ObjectPatProp {
    type Cloned = ObjectPatProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::KeyValue(it) => Self::KeyValue(it.clone_in(ast)),
            Self::Assign(it) => Self::Assign(it.clone_in(ast)),
            Self::Rest(it) => Self::Rest(it.clone_in(ast)),
        }
    }
}
impl CloneIn for KeyValuePatProp {
    type Cloned = KeyValuePatProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.key_value_pat_prop(span, key, value)
    }
}
impl CloneIn for AssignPatProp {
    type Cloned = AssignPatProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.assign_pat_prop(span, key, value)
    }
}
impl CloneIn for Ident {
    type Cloned = Ident;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let sym = self.sym(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.ident(span, sym, optional)
    }
}
impl CloneIn for IdentName {
    type Cloned = IdentName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let sym = self.sym(ast).clone_in(ast);
        ast.ident_name(span, sym)
    }
}
impl CloneIn for PrivateName {
    type Cloned = PrivateName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.private_name(span, name)
    }
}
impl CloneIn for BindingIdent {
    type Cloned = BindingIdent;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        ast.binding_ident(span, id)
    }
}
impl CloneIn for Lit {
    type Cloned = Lit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Str(it) => Self::Str(it.clone_in(ast)),
            Self::Bool(it) => Self::Bool(it.clone_in(ast)),
            Self::Null(it) => Self::Null(it.clone_in(ast)),
            Self::Num(it) => Self::Num(it.clone_in(ast)),
            Self::BigInt(it) => Self::BigInt(it.clone_in(ast)),
            Self::Regex(it) => Self::Regex(it.clone_in(ast)),
        }
    }
}
impl CloneIn for Str {
    type Cloned = Str;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.str(span, value, raw)
    }
}
impl CloneIn for Bool {
    type Cloned = Bool;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.bool(span, value)
    }
}
impl CloneIn for Null {
    type Cloned = Null;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.null(span)
    }
}
impl CloneIn for Number {
    type Cloned = Number;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.number(span, value, raw)
    }
}
impl CloneIn for BigInt {
    type Cloned = BigInt;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.big_int(span, value, raw)
    }
}
impl CloneIn for Regex {
    type Cloned = Regex;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exp = self.exp(ast).clone_in(ast);
        let flags = self.flags(ast).clone_in(ast);
        ast.regex(span, exp, flags)
    }
}
impl CloneIn for JSXObject {
    type Cloned = JSXObject;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::JSXMemberExpr(it) => Self::JSXMemberExpr(it.clone_in(ast)),
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
        }
    }
}
impl CloneIn for JSXMemberExpr {
    type Cloned = JSXMemberExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let obj = self.obj(ast).clone_in(ast);
        let prop = self.prop(ast).clone_in(ast);
        ast.jsx_member_expr(span, obj, prop)
    }
}
impl CloneIn for JSXNamespacedName {
    type Cloned = JSXNamespacedName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let ns = self.ns(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_namespaced_name(span, ns, name)
    }
}
impl CloneIn for JSXEmptyExpr {
    type Cloned = JSXEmptyExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_empty_expr(span)
    }
}
impl CloneIn for JSXExprContainer {
    type Cloned = JSXExprContainer;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.jsx_expr_container(span, expr)
    }
}
impl CloneIn for JSXExpr {
    type Cloned = JSXExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::JSXEmptyExpr(it) => Self::JSXEmptyExpr(it.clone_in(ast)),
            Self::Expr(it) => Self::Expr(it.clone_in(ast)),
        }
    }
}
impl CloneIn for JSXSpreadChild {
    type Cloned = JSXSpreadChild;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.jsx_spread_child(span, expr)
    }
}
impl CloneIn for JSXElementName {
    type Cloned = JSXElementName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::JSXMemberExpr(it) => Self::JSXMemberExpr(it.clone_in(ast)),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(it.clone_in(ast)),
        }
    }
}
impl CloneIn for JSXOpeningElement {
    type Cloned = JSXOpeningElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let attrs = self.attrs(ast).clone_in(ast);
        let self_closing = self.self_closing(ast).clone_in(ast);
        ast.jsx_opening_element(span, name, attrs, self_closing)
    }
}
impl CloneIn for JSXAttrOrSpread {
    type Cloned = JSXAttrOrSpread;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::JSXAttr(it) => Self::JSXAttr(it.clone_in(ast)),
            Self::SpreadElement(it) => Self::SpreadElement(it.clone_in(ast)),
        }
    }
}
impl CloneIn for JSXClosingElement {
    type Cloned = JSXClosingElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_closing_element(span, name)
    }
}
impl CloneIn for JSXAttr {
    type Cloned = JSXAttr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.jsx_attr(span, name, value)
    }
}
impl CloneIn for JSXAttrName {
    type Cloned = JSXAttrName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(it.clone_in(ast)),
        }
    }
}
impl CloneIn for JSXAttrValue {
    type Cloned = JSXAttrValue;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Str(it) => Self::Str(it.clone_in(ast)),
            Self::JSXExprContainer(it) => Self::JSXExprContainer(it.clone_in(ast)),
            Self::JSXElement(it) => Self::JSXElement(it.clone_in(ast)),
            Self::JSXFragment(it) => Self::JSXFragment(it.clone_in(ast)),
        }
    }
}
impl CloneIn for JSXText {
    type Cloned = JSXText;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.jsx_text(span, value, raw)
    }
}
impl CloneIn for JSXElement {
    type Cloned = JSXElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let opening = self.opening(ast).clone_in(ast);
        let children = self.children(ast).clone_in(ast);
        let closing = self.closing(ast).clone_in(ast);
        ast.jsx_element(span, opening, children, closing)
    }
}
impl CloneIn for JSXElementChild {
    type Cloned = JSXElementChild;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::JSXText(it) => Self::JSXText(it.clone_in(ast)),
            Self::JSXExprContainer(it) => Self::JSXExprContainer(it.clone_in(ast)),
            Self::JSXSpreadChild(it) => Self::JSXSpreadChild(it.clone_in(ast)),
            Self::JSXElement(it) => Self::JSXElement(it.clone_in(ast)),
            Self::JSXFragment(it) => Self::JSXFragment(it.clone_in(ast)),
        }
    }
}
impl CloneIn for JSXFragment {
    type Cloned = JSXFragment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let opening = self.opening(ast).clone_in(ast);
        let children = self.children(ast).clone_in(ast);
        let closing = self.closing(ast).clone_in(ast);
        ast.jsx_fragment(span, opening, children, closing)
    }
}
impl CloneIn for JSXOpeningFragment {
    type Cloned = JSXOpeningFragment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_opening_fragment(span)
    }
}
impl CloneIn for JSXClosingFragment {
    type Cloned = JSXClosingFragment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_closing_fragment(span)
    }
}
impl CloneIn for TsTypeAnn {
    type Cloned = TsTypeAnn;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_type_ann(span, type_ann)
    }
}
impl CloneIn for TsTypeParamDecl {
    type Cloned = TsTypeParamDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        ast.ts_type_param_decl(span, params)
    }
}
impl CloneIn for TsTypeParam {
    type Cloned = TsTypeParam;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let is_in = self.is_in(ast).clone_in(ast);
        let is_out = self.is_out(ast).clone_in(ast);
        let is_const = self.is_const(ast).clone_in(ast);
        let constraint = self.constraint(ast).clone_in(ast);
        let default = self.default(ast).clone_in(ast);
        ast.ts_type_param(span, name, is_in, is_out, is_const, constraint, default)
    }
}
impl CloneIn for TsTypeParamInstantiation {
    type Cloned = TsTypeParamInstantiation;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        ast.ts_type_param_instantiation(span, params)
    }
}
impl CloneIn for TsParamProp {
    type Cloned = TsParamProp;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let accessibility = self.accessibility(ast).clone_in(ast);
        let is_override = self.is_override(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let param = self.param(ast).clone_in(ast);
        ast.ts_param_prop(
            span,
            decorators,
            accessibility,
            is_override,
            readonly,
            param,
        )
    }
}
impl CloneIn for TsParamPropParam {
    type Cloned = TsParamPropParam;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Assign(it) => Self::Assign(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsQualifiedName {
    type Cloned = TsQualifiedName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.ts_qualified_name(span, left, right)
    }
}
impl CloneIn for TsEntityName {
    type Cloned = TsEntityName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::TsQualifiedName(it) => Self::TsQualifiedName(it.clone_in(ast)),
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsTypeElement {
    type Cloned = TsTypeElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::TsCallSignatureDecl(it) => Self::TsCallSignatureDecl(it.clone_in(ast)),
            Self::TsConstructSignatureDecl(it) => Self::TsConstructSignatureDecl(it.clone_in(ast)),
            Self::TsPropertySignature(it) => Self::TsPropertySignature(it.clone_in(ast)),
            Self::TsGetterSignature(it) => Self::TsGetterSignature(it.clone_in(ast)),
            Self::TsSetterSignature(it) => Self::TsSetterSignature(it.clone_in(ast)),
            Self::TsMethodSignature(it) => Self::TsMethodSignature(it.clone_in(ast)),
            Self::TsIndexSignature(it) => Self::TsIndexSignature(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsCallSignatureDecl {
    type Cloned = TsCallSignatureDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        let type_params = self.type_params(ast).clone_in(ast);
        ast.ts_call_signature_decl(span, params, type_ann, type_params)
    }
}
impl CloneIn for TsConstructSignatureDecl {
    type Cloned = TsConstructSignatureDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        let type_params = self.type_params(ast).clone_in(ast);
        ast.ts_construct_signature_decl(span, params, type_ann, type_params)
    }
}
impl CloneIn for TsPropertySignature {
    type Cloned = TsPropertySignature;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_property_signature(span, readonly, key, computed, optional, type_ann)
    }
}
impl CloneIn for TsGetterSignature {
    type Cloned = TsGetterSignature;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_getter_signature(span, key, computed, type_ann)
    }
}
impl CloneIn for TsSetterSignature {
    type Cloned = TsSetterSignature;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let param = self.param(ast).clone_in(ast);
        ast.ts_setter_signature(span, key, computed, param)
    }
}
impl CloneIn for TsMethodSignature {
    type Cloned = TsMethodSignature;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        let type_params = self.type_params(ast).clone_in(ast);
        ast.ts_method_signature(span, key, computed, optional, params, type_ann, type_params)
    }
}
impl CloneIn for TsIndexSignature {
    type Cloned = TsIndexSignature;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let is_static = self.is_static(ast).clone_in(ast);
        ast.ts_index_signature(span, params, type_ann, readonly, is_static)
    }
}
impl CloneIn for TsType {
    type Cloned = TsType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::TsKeywordType(it) => Self::TsKeywordType(it.clone_in(ast)),
            Self::TsThisType(it) => Self::TsThisType(it.clone_in(ast)),
            Self::TsFnOrConstructorType(it) => Self::TsFnOrConstructorType(it.clone_in(ast)),
            Self::TsTypeRef(it) => Self::TsTypeRef(it.clone_in(ast)),
            Self::TsTypeQuery(it) => Self::TsTypeQuery(it.clone_in(ast)),
            Self::TsTypeLit(it) => Self::TsTypeLit(it.clone_in(ast)),
            Self::TsArrayType(it) => Self::TsArrayType(it.clone_in(ast)),
            Self::TsTupleType(it) => Self::TsTupleType(it.clone_in(ast)),
            Self::TsOptionalType(it) => Self::TsOptionalType(it.clone_in(ast)),
            Self::TsRestType(it) => Self::TsRestType(it.clone_in(ast)),
            Self::TsUnionOrIntersectionType(it) => {
                Self::TsUnionOrIntersectionType(it.clone_in(ast))
            }
            Self::TsConditionalType(it) => Self::TsConditionalType(it.clone_in(ast)),
            Self::TsInferType(it) => Self::TsInferType(it.clone_in(ast)),
            Self::TsParenthesizedType(it) => Self::TsParenthesizedType(it.clone_in(ast)),
            Self::TsTypeOperator(it) => Self::TsTypeOperator(it.clone_in(ast)),
            Self::TsIndexedAccessType(it) => Self::TsIndexedAccessType(it.clone_in(ast)),
            Self::TsMappedType(it) => Self::TsMappedType(it.clone_in(ast)),
            Self::TsLitType(it) => Self::TsLitType(it.clone_in(ast)),
            Self::TsTypePredicate(it) => Self::TsTypePredicate(it.clone_in(ast)),
            Self::TsImportType(it) => Self::TsImportType(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsFnOrConstructorType {
    type Cloned = TsFnOrConstructorType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::TsFnType(it) => Self::TsFnType(it.clone_in(ast)),
            Self::TsConstructorType(it) => Self::TsConstructorType(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsKeywordType {
    type Cloned = TsKeywordType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        ast.ts_keyword_type(span, kind)
    }
}
impl CloneIn for TsThisType {
    type Cloned = TsThisType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_this_type(span)
    }
}
impl CloneIn for TsFnParam {
    type Cloned = TsFnParam;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Array(it) => Self::Array(it.clone_in(ast)),
            Self::Rest(it) => Self::Rest(it.clone_in(ast)),
            Self::Object(it) => Self::Object(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsFnType {
    type Cloned = TsFnType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let type_params = self.type_params(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_fn_type(span, params, type_params, type_ann)
    }
}
impl CloneIn for TsConstructorType {
    type Cloned = TsConstructorType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let type_params = self.type_params(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        let is_abstract = self.is_abstract(ast).clone_in(ast);
        ast.ts_constructor_type(span, params, type_params, type_ann, is_abstract)
    }
}
impl CloneIn for TsTypeRef {
    type Cloned = TsTypeRef;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_name = self.type_name(ast).clone_in(ast);
        let type_params = self.type_params(ast).clone_in(ast);
        ast.ts_type_ref(span, type_name, type_params)
    }
}
impl CloneIn for TsTypePredicate {
    type Cloned = TsTypePredicate;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let asserts = self.asserts(ast).clone_in(ast);
        let param_name = self.param_name(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_type_predicate(span, asserts, param_name, type_ann)
    }
}
impl CloneIn for TsThisTypeOrIdent {
    type Cloned = TsThisTypeOrIdent;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::TsThisType(it) => Self::TsThisType(it.clone_in(ast)),
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsTypeQuery {
    type Cloned = TsTypeQuery;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr_name = self.expr_name(ast).clone_in(ast);
        let type_args = self.type_args(ast).clone_in(ast);
        ast.ts_type_query(span, expr_name, type_args)
    }
}
impl CloneIn for TsTypeQueryExpr {
    type Cloned = TsTypeQueryExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::TsEntityName(it) => Self::TsEntityName(it.clone_in(ast)),
            Self::Import(it) => Self::Import(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsImportCallOptions {
    type Cloned = TsImportCallOptions;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let with = self.with(ast).clone_in(ast);
        ast.ts_import_call_options(span, with)
    }
}
impl CloneIn for TsImportType {
    type Cloned = TsImportType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let arg = self.arg(ast).clone_in(ast);
        let qualifier = self.qualifier(ast).clone_in(ast);
        let type_args = self.type_args(ast).clone_in(ast);
        let attributes = self.attributes(ast).clone_in(ast);
        ast.ts_import_type(span, arg, qualifier, type_args, attributes)
    }
}
impl CloneIn for TsTypeLit {
    type Cloned = TsTypeLit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let members = self.members(ast).clone_in(ast);
        ast.ts_type_lit(span, members)
    }
}
impl CloneIn for TsArrayType {
    type Cloned = TsArrayType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elem_type = self.elem_type(ast).clone_in(ast);
        ast.ts_array_type(span, elem_type)
    }
}
impl CloneIn for TsTupleType {
    type Cloned = TsTupleType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elem_types = self.elem_types(ast).clone_in(ast);
        ast.ts_tuple_type(span, elem_types)
    }
}
impl CloneIn for TsTupleElement {
    type Cloned = TsTupleElement;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        let ty = self.ty(ast).clone_in(ast);
        ast.ts_tuple_element(span, label, ty)
    }
}
impl CloneIn for TsOptionalType {
    type Cloned = TsOptionalType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_optional_type(span, type_ann)
    }
}
impl CloneIn for TsRestType {
    type Cloned = TsRestType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_rest_type(span, type_ann)
    }
}
impl CloneIn for TsUnionOrIntersectionType {
    type Cloned = TsUnionOrIntersectionType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::TsUnionType(it) => Self::TsUnionType(it.clone_in(ast)),
            Self::TsIntersectionType(it) => Self::TsIntersectionType(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsUnionType {
    type Cloned = TsUnionType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let types = self.types(ast).clone_in(ast);
        ast.ts_union_type(span, types)
    }
}
impl CloneIn for TsIntersectionType {
    type Cloned = TsIntersectionType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let types = self.types(ast).clone_in(ast);
        ast.ts_intersection_type(span, types)
    }
}
impl CloneIn for TsConditionalType {
    type Cloned = TsConditionalType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let check_type = self.check_type(ast).clone_in(ast);
        let extends_type = self.extends_type(ast).clone_in(ast);
        let true_type = self.true_type(ast).clone_in(ast);
        let false_type = self.false_type(ast).clone_in(ast);
        ast.ts_conditional_type(span, check_type, extends_type, true_type, false_type)
    }
}
impl CloneIn for TsInferType {
    type Cloned = TsInferType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_param = self.type_param(ast).clone_in(ast);
        ast.ts_infer_type(span, type_param)
    }
}
impl CloneIn for TsParenthesizedType {
    type Cloned = TsParenthesizedType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_parenthesized_type(span, type_ann)
    }
}
impl CloneIn for TsTypeOperator {
    type Cloned = TsTypeOperator;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let op = self.op(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_type_operator(span, op, type_ann)
    }
}
impl CloneIn for TsIndexedAccessType {
    type Cloned = TsIndexedAccessType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let obj_type = self.obj_type(ast).clone_in(ast);
        let index_type = self.index_type(ast).clone_in(ast);
        ast.ts_indexed_access_type(span, readonly, obj_type, index_type)
    }
}
impl CloneIn for TsMappedType {
    type Cloned = TsMappedType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let type_param = self.type_param(ast).clone_in(ast);
        let name_type = self.name_type(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_mapped_type(span, readonly, type_param, name_type, optional, type_ann)
    }
}
impl CloneIn for TsLitType {
    type Cloned = TsLitType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let lit = self.lit(ast).clone_in(ast);
        ast.ts_lit_type(span, lit)
    }
}
impl CloneIn for TsLit {
    type Cloned = TsLit;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Number(it) => Self::Number(it.clone_in(ast)),
            Self::Str(it) => Self::Str(it.clone_in(ast)),
            Self::Bool(it) => Self::Bool(it.clone_in(ast)),
            Self::BigInt(it) => Self::BigInt(it.clone_in(ast)),
            Self::Tpl(it) => Self::Tpl(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsTplLitType {
    type Cloned = TsTplLitType;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let types = self.types(ast).clone_in(ast);
        let quasis = self.quasis(ast).clone_in(ast);
        ast.ts_tpl_lit_type(span, types, quasis)
    }
}
impl CloneIn for TsInterfaceDecl {
    type Cloned = TsInterfaceDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let type_params = self.type_params(ast).clone_in(ast);
        let extends = self.extends(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.ts_interface_decl(span, id, declare, type_params, extends, body)
    }
}
impl CloneIn for TsInterfaceBody {
    type Cloned = TsInterfaceBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.ts_interface_body(span, body)
    }
}
impl CloneIn for TsExprWithTypeArgs {
    type Cloned = TsExprWithTypeArgs;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        let type_args = self.type_args(ast).clone_in(ast);
        ast.ts_expr_with_type_args(span, expr, type_args)
    }
}
impl CloneIn for TsTypeAliasDecl {
    type Cloned = TsTypeAliasDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_params = self.type_params(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_type_alias_decl(span, declare, id, type_params, type_ann)
    }
}
impl CloneIn for TsEnumDecl {
    type Cloned = TsEnumDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let is_const = self.is_const(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let members = self.members(ast).clone_in(ast);
        ast.ts_enum_decl(span, declare, is_const, id, members)
    }
}
impl CloneIn for TsEnumMember {
    type Cloned = TsEnumMember;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let init = self.init(ast).clone_in(ast);
        ast.ts_enum_member(span, id, init)
    }
}
impl CloneIn for TsEnumMemberId {
    type Cloned = TsEnumMemberId;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Str(it) => Self::Str(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsModuleDecl {
    type Cloned = TsModuleDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let global = self.global(ast).clone_in(ast);
        let namespace = self.namespace(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.ts_module_decl(span, declare, global, namespace, id, body)
    }
}
impl CloneIn for TsNamespaceBody {
    type Cloned = TsNamespaceBody;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::TsModuleBlock(it) => Self::TsModuleBlock(it.clone_in(ast)),
            Self::TsNamespaceDecl(it) => Self::TsNamespaceDecl(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsModuleBlock {
    type Cloned = TsModuleBlock;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.ts_module_block(span, body)
    }
}
impl CloneIn for TsNamespaceDecl {
    type Cloned = TsNamespaceDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let global = self.global(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.ts_namespace_decl(span, declare, global, id, body)
    }
}
impl CloneIn for TsModuleName {
    type Cloned = TsModuleName;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::Ident(it) => Self::Ident(it.clone_in(ast)),
            Self::Str(it) => Self::Str(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsImportEqualsDecl {
    type Cloned = TsImportEqualsDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let is_export = self.is_export(ast).clone_in(ast);
        let is_type_only = self.is_type_only(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let module_ref = self.module_ref(ast).clone_in(ast);
        ast.ts_import_equals_decl(span, is_export, is_type_only, id, module_ref)
    }
}
impl CloneIn for TsModuleRef {
    type Cloned = TsModuleRef;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        match self {
            Self::TsEntityName(it) => Self::TsEntityName(it.clone_in(ast)),
            Self::TsExternalModuleRef(it) => Self::TsExternalModuleRef(it.clone_in(ast)),
        }
    }
}
impl CloneIn for TsExternalModuleRef {
    type Cloned = TsExternalModuleRef;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.ts_external_module_ref(span, expr)
    }
}
impl CloneIn for TsExportAssignment {
    type Cloned = TsExportAssignment;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.ts_export_assignment(span, expr)
    }
}
impl CloneIn for TsNamespaceExportDecl {
    type Cloned = TsNamespaceExportDecl;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        ast.ts_namespace_export_decl(span, id)
    }
}
impl CloneIn for TsAsExpr {
    type Cloned = TsAsExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_as_expr(span, expr, type_ann)
    }
}
impl CloneIn for TsTypeAssertion {
    type Cloned = TsTypeAssertion;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_type_assertion(span, expr, type_ann)
    }
}
impl CloneIn for TsNonNullExpr {
    type Cloned = TsNonNullExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.ts_non_null_expr(span, expr)
    }
}
impl CloneIn for TsSatisfiesExpr {
    type Cloned = TsSatisfiesExpr;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        let type_ann = self.type_ann(ast).clone_in(ast);
        ast.ts_satisfies_expr(span, expr, type_ann)
    }
}
impl CloneIn for TsConstAssertion {
    type Cloned = TsConstAssertion;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        ast.ts_const_assertion(span, expr)
    }
}
impl CloneIn for TsInstantiation {
    type Cloned = TsInstantiation;
    #[inline]
    fn clone_in(&self, ast: &mut Ast) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr = self.expr(ast).clone_in(ast);
        let type_args = self.type_args(ast).clone_in(ast);
        ast.ts_instantiation(span, expr, type_args)
    }
}
