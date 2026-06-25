#![allow(unused, clippy::useless_conversion, clippy::identity_op)]
use crate::*;
use swc_experimental_allocator::atom::{Atom, Wtf8Atom};
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
impl<'a> AstBuilder<'a> {
    #[inline]
    pub fn program_module(
        &self,
        span: Span,
        body: Vec<'a, ModuleItem<'a>>,
        shebang: Option<Atom<'a>>,
    ) -> Program<'a> {
        Program::Module(self.box_module(span, body, shebang))
    }
    #[inline]
    pub fn program_script(
        &self,
        span: Span,
        body: Vec<'a, Stmt<'a>>,
        shebang: Option<Atom<'a>>,
    ) -> Program<'a> {
        Program::Script(self.box_script(span, body, shebang))
    }
    #[inline]
    pub fn module(
        &self,
        span: Span,
        body: Vec<'a, ModuleItem<'a>>,
        shebang: Option<Atom<'a>>,
    ) -> Module<'a> {
        Module {
            span,
            body,
            shebang,
        }
    }
    #[inline]
    pub fn box_module(
        &self,
        span: Span,
        body: Vec<'a, ModuleItem<'a>>,
        shebang: Option<Atom<'a>>,
    ) -> Box<'a, Module<'a>> {
        self.allocator.boxed(self.module(span, body, shebang))
    }
    #[inline]
    pub fn script(
        &self,
        span: Span,
        body: Vec<'a, Stmt<'a>>,
        shebang: Option<Atom<'a>>,
    ) -> Script<'a> {
        Script {
            span,
            body,
            shebang,
        }
    }
    #[inline]
    pub fn box_script(
        &self,
        span: Span,
        body: Vec<'a, Stmt<'a>>,
        shebang: Option<Atom<'a>>,
    ) -> Box<'a, Script<'a>> {
        self.allocator.boxed(self.script(span, body, shebang))
    }
    #[inline]
    pub fn module_item_module_decl_import_decl(
        &self,
        span: Span,
        specifiers: Vec<'a, ImportSpecifier<'a>>,
        src: Box<'a, Str<'a>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
        phase: ImportPhase,
    ) -> ModuleItem<'a> {
        ModuleItem::ModuleDecl(self.allocator.boxed(ModuleDecl::Import(
            self.box_import_decl(span, specifiers, src, type_only, with, phase),
        )))
    }
    #[inline]
    pub fn module_item_module_decl_export_decl(
        &self,
        span: Span,
        decl: Decl<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::ModuleDecl(
            self.allocator
                .boxed(ModuleDecl::ExportDecl(self.box_export_decl(span, decl))),
        )
    }
    #[inline]
    pub fn module_item_module_decl_named_export(
        &self,
        span: Span,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        src: Option<Box<'a, Str<'a>>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
    ) -> ModuleItem<'a> {
        ModuleItem::ModuleDecl(self.allocator.boxed(ModuleDecl::ExportNamed(
            self.box_named_export(span, specifiers, src, type_only, with),
        )))
    }
    #[inline]
    pub fn module_item_module_decl_export_default_decl(
        &self,
        span: Span,
        decl: DefaultDecl<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::ModuleDecl(self.allocator.boxed(ModuleDecl::ExportDefaultDecl(
            self.box_export_default_decl(span, decl),
        )))
    }
    #[inline]
    pub fn module_item_module_decl_export_default_expr(
        &self,
        span: Span,
        expr: Expr<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::ModuleDecl(self.allocator.boxed(ModuleDecl::ExportDefaultExpr(
            self.box_export_default_expr(span, expr),
        )))
    }
    #[inline]
    pub fn module_item_module_decl_export_all(
        &self,
        span: Span,
        src: Box<'a, Str<'a>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
    ) -> ModuleItem<'a> {
        ModuleItem::ModuleDecl(self.allocator.boxed(ModuleDecl::ExportAll(
            self.box_export_all(span, src, type_only, with),
        )))
    }
    #[inline]
    pub fn module_item_stmt_block_stmt(
        &self,
        span: Span,
        stmts: Vec<'a, Stmt<'a>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Block(self.box_block_stmt(span, stmts))),
        )
    }
    #[inline]
    pub fn module_item_stmt_empty_stmt(&self, span: Span) -> ModuleItem<'a> {
        ModuleItem::Stmt(self.allocator.boxed(Stmt::Empty(self.box_empty_stmt(span))))
    }
    #[inline]
    pub fn module_item_stmt_debugger_stmt(&self, span: Span) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Debugger(self.box_debugger_stmt(span))),
        )
    }
    #[inline]
    pub fn module_item_stmt_with_stmt(
        &self,
        span: Span,
        obj: Expr<'a>,
        body: Stmt<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::With(self.box_with_stmt(span, obj, body))),
        )
    }
    #[inline]
    pub fn module_item_stmt_return_stmt(
        &self,
        span: Span,
        arg: Option<Expr<'a>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Return(self.box_return_stmt(span, arg))),
        )
    }
    #[inline]
    pub fn module_item_stmt_labeled_stmt(
        &self,
        span: Span,
        label: Box<'a, Ident<'a>>,
        body: Stmt<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Labeled(self.box_labeled_stmt(span, label, body))),
        )
    }
    #[inline]
    pub fn module_item_stmt_break_stmt(
        &self,
        span: Span,
        label: Option<Box<'a, Ident<'a>>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Break(self.box_break_stmt(span, label))),
        )
    }
    #[inline]
    pub fn module_item_stmt_continue_stmt(
        &self,
        span: Span,
        label: Option<Box<'a, Ident<'a>>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Continue(self.box_continue_stmt(span, label))),
        )
    }
    #[inline]
    pub fn module_item_stmt_if_stmt(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Stmt<'a>,
        alt: Option<Stmt<'a>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::If(self.box_if_stmt(span, test, cons, alt))),
        )
    }
    #[inline]
    pub fn module_item_stmt_switch_stmt(
        &self,
        span: Span,
        discriminant: Expr<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(self.allocator.boxed(Stmt::Switch(self.box_switch_stmt(
            span,
            discriminant,
            cases,
        ))))
    }
    #[inline]
    pub fn module_item_stmt_throw_stmt(&self, span: Span, arg: Expr<'a>) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Throw(self.box_throw_stmt(span, arg))),
        )
    }
    #[inline]
    pub fn module_item_stmt_try_stmt(
        &self,
        span: Span,
        block: Box<'a, BlockStmt<'a>>,
        handler: Option<Box<'a, CatchClause<'a>>>,
        finalizer: Option<Box<'a, BlockStmt<'a>>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(self.allocator.boxed(Stmt::Try(
            self.box_try_stmt(span, block, handler, finalizer),
        )))
    }
    #[inline]
    pub fn module_item_stmt_while_stmt(
        &self,
        span: Span,
        test: Expr<'a>,
        body: Stmt<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::While(self.box_while_stmt(span, test, body))),
        )
    }
    #[inline]
    pub fn module_item_stmt_do_while_stmt(
        &self,
        span: Span,
        test: Expr<'a>,
        body: Stmt<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::DoWhile(self.box_do_while_stmt(span, test, body))),
        )
    }
    #[inline]
    pub fn module_item_stmt_for_stmt(
        &self,
        span: Span,
        init: Option<VarDeclOrExpr<'a>>,
        test: Option<Expr<'a>>,
        update: Option<Expr<'a>>,
        body: Stmt<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::For(self.box_for_stmt(span, init, test, update, body))),
        )
    }
    #[inline]
    pub fn module_item_stmt_for_in_stmt(
        &self,
        span: Span,
        left: ForHead<'a>,
        right: Expr<'a>,
        body: Stmt<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::ForIn(self.box_for_in_stmt(span, left, right, body))),
        )
    }
    #[inline]
    pub fn module_item_stmt_for_of_stmt(
        &self,
        span: Span,
        is_await: bool,
        left: ForHead<'a>,
        right: Expr<'a>,
        body: Stmt<'a>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(self.allocator.boxed(Stmt::ForOf(
            self.box_for_of_stmt(span, is_await, left, right, body),
        )))
    }
    #[inline]
    pub fn module_item_stmt_decl_class_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        class: Box<'a, Class<'a>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Decl(self.allocator.boxed(Decl::Class(
                    self.box_class_decl(ident, declare, class),
                )))),
        )
    }
    #[inline]
    pub fn module_item_stmt_decl_fn_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        function: Box<'a, Function<'a>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator.boxed(Stmt::Decl(
                self.allocator
                    .boxed(Decl::Fn(self.box_fn_decl(ident, declare, function))),
            )),
        )
    }
    #[inline]
    pub fn module_item_stmt_decl_var_decl(
        &self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Decl(self.allocator.boxed(Decl::Var(
                    self.box_var_decl(span, kind, declare, decls),
                )))),
        )
    }
    #[inline]
    pub fn module_item_stmt_decl_using_decl(
        &self,
        span: Span,
        is_await: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Decl(self.allocator.boxed(Decl::Using(
                    self.box_using_decl(span, is_await, decls),
                )))),
        )
    }
    #[inline]
    pub fn module_item_stmt_expr_stmt(&self, span: Span, expr: Expr<'a>) -> ModuleItem<'a> {
        ModuleItem::Stmt(
            self.allocator
                .boxed(Stmt::Expr(self.box_expr_stmt(span, expr))),
        )
    }
    #[inline]
    pub fn module_decl_import_decl(
        &self,
        span: Span,
        specifiers: Vec<'a, ImportSpecifier<'a>>,
        src: Box<'a, Str<'a>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
        phase: ImportPhase,
    ) -> ModuleDecl<'a> {
        ModuleDecl::Import(self.box_import_decl(span, specifiers, src, type_only, with, phase))
    }
    #[inline]
    pub fn module_decl_export_decl(&self, span: Span, decl: Decl<'a>) -> ModuleDecl<'a> {
        ModuleDecl::ExportDecl(self.box_export_decl(span, decl))
    }
    #[inline]
    pub fn module_decl_named_export(
        &self,
        span: Span,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        src: Option<Box<'a, Str<'a>>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
    ) -> ModuleDecl<'a> {
        ModuleDecl::ExportNamed(self.box_named_export(span, specifiers, src, type_only, with))
    }
    #[inline]
    pub fn module_decl_export_default_decl(
        &self,
        span: Span,
        decl: DefaultDecl<'a>,
    ) -> ModuleDecl<'a> {
        ModuleDecl::ExportDefaultDecl(self.box_export_default_decl(span, decl))
    }
    #[inline]
    pub fn module_decl_export_default_expr(&self, span: Span, expr: Expr<'a>) -> ModuleDecl<'a> {
        ModuleDecl::ExportDefaultExpr(self.box_export_default_expr(span, expr))
    }
    #[inline]
    pub fn module_decl_export_all(
        &self,
        span: Span,
        src: Box<'a, Str<'a>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
    ) -> ModuleDecl<'a> {
        ModuleDecl::ExportAll(self.box_export_all(span, src, type_only, with))
    }
    #[inline]
    pub fn import_decl(
        &self,
        span: Span,
        specifiers: Vec<'a, ImportSpecifier<'a>>,
        src: Box<'a, Str<'a>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
        phase: ImportPhase,
    ) -> ImportDecl<'a> {
        ImportDecl {
            span,
            specifiers,
            src,
            type_only,
            with,
            phase,
        }
    }
    #[inline]
    pub fn box_import_decl(
        &self,
        span: Span,
        specifiers: Vec<'a, ImportSpecifier<'a>>,
        src: Box<'a, Str<'a>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
        phase: ImportPhase,
    ) -> Box<'a, ImportDecl<'a>> {
        self.allocator
            .boxed(self.import_decl(span, specifiers, src, type_only, with, phase))
    }
    #[inline]
    pub fn import_specifier_import_named_specifier(
        &self,
        span: Span,
        local: Box<'a, Ident<'a>>,
        imported: Option<ModuleExportName<'a>>,
        is_type_only: bool,
    ) -> ImportSpecifier<'a> {
        ImportSpecifier::Named(self.box_import_named_specifier(span, local, imported, is_type_only))
    }
    #[inline]
    pub fn import_specifier_import_default_specifier(
        &self,
        span: Span,
        local: Box<'a, Ident<'a>>,
    ) -> ImportSpecifier<'a> {
        ImportSpecifier::Default(self.box_import_default_specifier(span, local))
    }
    #[inline]
    pub fn import_specifier_import_star_as_specifier(
        &self,
        span: Span,
        local: Box<'a, Ident<'a>>,
    ) -> ImportSpecifier<'a> {
        ImportSpecifier::Namespace(self.box_import_star_as_specifier(span, local))
    }
    #[inline]
    pub fn import_named_specifier(
        &self,
        span: Span,
        local: Box<'a, Ident<'a>>,
        imported: Option<ModuleExportName<'a>>,
        is_type_only: bool,
    ) -> ImportNamedSpecifier<'a> {
        ImportNamedSpecifier {
            span,
            local,
            imported,
            is_type_only,
        }
    }
    #[inline]
    pub fn box_import_named_specifier(
        &self,
        span: Span,
        local: Box<'a, Ident<'a>>,
        imported: Option<ModuleExportName<'a>>,
        is_type_only: bool,
    ) -> Box<'a, ImportNamedSpecifier<'a>> {
        self.allocator
            .boxed(self.import_named_specifier(span, local, imported, is_type_only))
    }
    #[inline]
    pub fn import_default_specifier(
        &self,
        span: Span,
        local: Box<'a, Ident<'a>>,
    ) -> ImportDefaultSpecifier<'a> {
        ImportDefaultSpecifier { span, local }
    }
    #[inline]
    pub fn box_import_default_specifier(
        &self,
        span: Span,
        local: Box<'a, Ident<'a>>,
    ) -> Box<'a, ImportDefaultSpecifier<'a>> {
        self.allocator
            .boxed(self.import_default_specifier(span, local))
    }
    #[inline]
    pub fn import_star_as_specifier(
        &self,
        span: Span,
        local: Box<'a, Ident<'a>>,
    ) -> ImportStarAsSpecifier<'a> {
        ImportStarAsSpecifier { span, local }
    }
    #[inline]
    pub fn box_import_star_as_specifier(
        &self,
        span: Span,
        local: Box<'a, Ident<'a>>,
    ) -> Box<'a, ImportStarAsSpecifier<'a>> {
        self.allocator
            .boxed(self.import_star_as_specifier(span, local))
    }
    #[inline]
    pub fn export_decl(&self, span: Span, decl: Decl<'a>) -> ExportDecl<'a> {
        ExportDecl { span, decl }
    }
    #[inline]
    pub fn box_export_decl(&self, span: Span, decl: Decl<'a>) -> Box<'a, ExportDecl<'a>> {
        self.allocator.boxed(self.export_decl(span, decl))
    }
    #[inline]
    pub fn named_export(
        &self,
        span: Span,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        src: Option<Box<'a, Str<'a>>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
    ) -> NamedExport<'a> {
        NamedExport {
            span,
            specifiers,
            src,
            type_only,
            with,
        }
    }
    #[inline]
    pub fn box_named_export(
        &self,
        span: Span,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        src: Option<Box<'a, Str<'a>>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
    ) -> Box<'a, NamedExport<'a>> {
        self.allocator
            .boxed(self.named_export(span, specifiers, src, type_only, with))
    }
    #[inline]
    pub fn export_specifier_export_namespace_specifier(
        &self,
        span: Span,
        name: ModuleExportName<'a>,
    ) -> ExportSpecifier<'a> {
        ExportSpecifier::Namespace(self.box_export_namespace_specifier(span, name))
    }
    #[inline]
    pub fn export_specifier_export_default_specifier(
        &self,
        exported: Box<'a, Ident<'a>>,
    ) -> ExportSpecifier<'a> {
        ExportSpecifier::Default(self.box_export_default_specifier(exported))
    }
    #[inline]
    pub fn export_specifier_export_named_specifier(
        &self,
        span: Span,
        orig: ModuleExportName<'a>,
        exported: Option<ModuleExportName<'a>>,
        is_type_only: bool,
    ) -> ExportSpecifier<'a> {
        ExportSpecifier::Named(self.box_export_named_specifier(span, orig, exported, is_type_only))
    }
    #[inline]
    pub fn export_namespace_specifier(
        &self,
        span: Span,
        name: ModuleExportName<'a>,
    ) -> ExportNamespaceSpecifier<'a> {
        ExportNamespaceSpecifier { span, name }
    }
    #[inline]
    pub fn box_export_namespace_specifier(
        &self,
        span: Span,
        name: ModuleExportName<'a>,
    ) -> Box<'a, ExportNamespaceSpecifier<'a>> {
        self.allocator
            .boxed(self.export_namespace_specifier(span, name))
    }
    #[inline]
    pub fn module_export_name_ident(&self, span: Span, sym: Atom<'a>) -> ModuleExportName<'a> {
        ModuleExportName::Ident(self.box_ident(span, sym))
    }
    #[inline]
    pub fn module_export_name_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> ModuleExportName<'a> {
        ModuleExportName::Str(self.box_str(span, value, raw))
    }
    #[inline]
    pub fn export_default_specifier(
        &self,
        exported: Box<'a, Ident<'a>>,
    ) -> ExportDefaultSpecifier<'a> {
        ExportDefaultSpecifier { exported }
    }
    #[inline]
    pub fn box_export_default_specifier(
        &self,
        exported: Box<'a, Ident<'a>>,
    ) -> Box<'a, ExportDefaultSpecifier<'a>> {
        self.allocator
            .boxed(self.export_default_specifier(exported))
    }
    #[inline]
    pub fn export_named_specifier(
        &self,
        span: Span,
        orig: ModuleExportName<'a>,
        exported: Option<ModuleExportName<'a>>,
        is_type_only: bool,
    ) -> ExportNamedSpecifier<'a> {
        ExportNamedSpecifier {
            span,
            orig,
            exported,
            is_type_only,
        }
    }
    #[inline]
    pub fn box_export_named_specifier(
        &self,
        span: Span,
        orig: ModuleExportName<'a>,
        exported: Option<ModuleExportName<'a>>,
        is_type_only: bool,
    ) -> Box<'a, ExportNamedSpecifier<'a>> {
        self.allocator
            .boxed(self.export_named_specifier(span, orig, exported, is_type_only))
    }
    #[inline]
    pub fn export_default_decl(&self, span: Span, decl: DefaultDecl<'a>) -> ExportDefaultDecl<'a> {
        ExportDefaultDecl { span, decl }
    }
    #[inline]
    pub fn box_export_default_decl(
        &self,
        span: Span,
        decl: DefaultDecl<'a>,
    ) -> Box<'a, ExportDefaultDecl<'a>> {
        self.allocator.boxed(self.export_default_decl(span, decl))
    }
    #[inline]
    pub fn default_decl_class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> DefaultDecl<'a> {
        DefaultDecl::Class(self.box_class_expr(ident, class))
    }
    #[inline]
    pub fn default_decl_fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> DefaultDecl<'a> {
        DefaultDecl::Fn(self.box_fn_expr(ident, function))
    }
    #[inline]
    pub fn export_default_expr(&self, span: Span, expr: Expr<'a>) -> ExportDefaultExpr<'a> {
        ExportDefaultExpr { span, expr }
    }
    #[inline]
    pub fn box_export_default_expr(
        &self,
        span: Span,
        expr: Expr<'a>,
    ) -> Box<'a, ExportDefaultExpr<'a>> {
        self.allocator.boxed(self.export_default_expr(span, expr))
    }
    #[inline]
    pub fn export_all(
        &self,
        span: Span,
        src: Box<'a, Str<'a>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
    ) -> ExportAll<'a> {
        ExportAll {
            span,
            src,
            type_only,
            with,
        }
    }
    #[inline]
    pub fn box_export_all(
        &self,
        span: Span,
        src: Box<'a, Str<'a>>,
        type_only: bool,
        with: Option<Box<'a, ObjectLit<'a>>>,
    ) -> Box<'a, ExportAll<'a>> {
        self.allocator
            .boxed(self.export_all(span, src, type_only, with))
    }
    #[inline]
    pub fn block_stmt(&self, span: Span, stmts: Vec<'a, Stmt<'a>>) -> BlockStmt<'a> {
        BlockStmt {
            span,
            stmts,
            scope_id: Default::default(),
        }
    }
    #[inline]
    pub fn box_block_stmt(&self, span: Span, stmts: Vec<'a, Stmt<'a>>) -> Box<'a, BlockStmt<'a>> {
        self.allocator.boxed(self.block_stmt(span, stmts))
    }
    #[inline]
    pub fn stmt_block_stmt(&self, span: Span, stmts: Vec<'a, Stmt<'a>>) -> Stmt<'a> {
        Stmt::Block(self.box_block_stmt(span, stmts))
    }
    #[inline]
    pub fn stmt_empty_stmt(&self, span: Span) -> Stmt<'a> {
        Stmt::Empty(self.box_empty_stmt(span))
    }
    #[inline]
    pub fn stmt_debugger_stmt(&self, span: Span) -> Stmt<'a> {
        Stmt::Debugger(self.box_debugger_stmt(span))
    }
    #[inline]
    pub fn stmt_with_stmt(&self, span: Span, obj: Expr<'a>, body: Stmt<'a>) -> Stmt<'a> {
        Stmt::With(self.box_with_stmt(span, obj, body))
    }
    #[inline]
    pub fn stmt_return_stmt(&self, span: Span, arg: Option<Expr<'a>>) -> Stmt<'a> {
        Stmt::Return(self.box_return_stmt(span, arg))
    }
    #[inline]
    pub fn stmt_labeled_stmt(
        &self,
        span: Span,
        label: Box<'a, Ident<'a>>,
        body: Stmt<'a>,
    ) -> Stmt<'a> {
        Stmt::Labeled(self.box_labeled_stmt(span, label, body))
    }
    #[inline]
    pub fn stmt_break_stmt(&self, span: Span, label: Option<Box<'a, Ident<'a>>>) -> Stmt<'a> {
        Stmt::Break(self.box_break_stmt(span, label))
    }
    #[inline]
    pub fn stmt_continue_stmt(&self, span: Span, label: Option<Box<'a, Ident<'a>>>) -> Stmt<'a> {
        Stmt::Continue(self.box_continue_stmt(span, label))
    }
    #[inline]
    pub fn stmt_if_stmt(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Stmt<'a>,
        alt: Option<Stmt<'a>>,
    ) -> Stmt<'a> {
        Stmt::If(self.box_if_stmt(span, test, cons, alt))
    }
    #[inline]
    pub fn stmt_switch_stmt(
        &self,
        span: Span,
        discriminant: Expr<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Stmt<'a> {
        Stmt::Switch(self.box_switch_stmt(span, discriminant, cases))
    }
    #[inline]
    pub fn stmt_throw_stmt(&self, span: Span, arg: Expr<'a>) -> Stmt<'a> {
        Stmt::Throw(self.box_throw_stmt(span, arg))
    }
    #[inline]
    pub fn stmt_try_stmt(
        &self,
        span: Span,
        block: Box<'a, BlockStmt<'a>>,
        handler: Option<Box<'a, CatchClause<'a>>>,
        finalizer: Option<Box<'a, BlockStmt<'a>>>,
    ) -> Stmt<'a> {
        Stmt::Try(self.box_try_stmt(span, block, handler, finalizer))
    }
    #[inline]
    pub fn stmt_while_stmt(&self, span: Span, test: Expr<'a>, body: Stmt<'a>) -> Stmt<'a> {
        Stmt::While(self.box_while_stmt(span, test, body))
    }
    #[inline]
    pub fn stmt_do_while_stmt(&self, span: Span, test: Expr<'a>, body: Stmt<'a>) -> Stmt<'a> {
        Stmt::DoWhile(self.box_do_while_stmt(span, test, body))
    }
    #[inline]
    pub fn stmt_for_stmt(
        &self,
        span: Span,
        init: Option<VarDeclOrExpr<'a>>,
        test: Option<Expr<'a>>,
        update: Option<Expr<'a>>,
        body: Stmt<'a>,
    ) -> Stmt<'a> {
        Stmt::For(self.box_for_stmt(span, init, test, update, body))
    }
    #[inline]
    pub fn stmt_for_in_stmt(
        &self,
        span: Span,
        left: ForHead<'a>,
        right: Expr<'a>,
        body: Stmt<'a>,
    ) -> Stmt<'a> {
        Stmt::ForIn(self.box_for_in_stmt(span, left, right, body))
    }
    #[inline]
    pub fn stmt_for_of_stmt(
        &self,
        span: Span,
        is_await: bool,
        left: ForHead<'a>,
        right: Expr<'a>,
        body: Stmt<'a>,
    ) -> Stmt<'a> {
        Stmt::ForOf(self.box_for_of_stmt(span, is_await, left, right, body))
    }
    #[inline]
    pub fn stmt_decl_class_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        class: Box<'a, Class<'a>>,
    ) -> Stmt<'a> {
        Stmt::Decl(
            self.allocator
                .boxed(Decl::Class(self.box_class_decl(ident, declare, class))),
        )
    }
    #[inline]
    pub fn stmt_decl_fn_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        function: Box<'a, Function<'a>>,
    ) -> Stmt<'a> {
        Stmt::Decl(
            self.allocator
                .boxed(Decl::Fn(self.box_fn_decl(ident, declare, function))),
        )
    }
    #[inline]
    pub fn stmt_decl_var_decl(
        &self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> Stmt<'a> {
        Stmt::Decl(
            self.allocator
                .boxed(Decl::Var(self.box_var_decl(span, kind, declare, decls))),
        )
    }
    #[inline]
    pub fn stmt_decl_using_decl(
        &self,
        span: Span,
        is_await: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> Stmt<'a> {
        Stmt::Decl(
            self.allocator
                .boxed(Decl::Using(self.box_using_decl(span, is_await, decls))),
        )
    }
    #[inline]
    pub fn stmt_expr_stmt(&self, span: Span, expr: Expr<'a>) -> Stmt<'a> {
        Stmt::Expr(self.box_expr_stmt(span, expr))
    }
    #[inline]
    pub fn expr_stmt(&self, span: Span, expr: Expr<'a>) -> ExprStmt<'a> {
        ExprStmt { span, expr }
    }
    #[inline]
    pub fn box_expr_stmt(&self, span: Span, expr: Expr<'a>) -> Box<'a, ExprStmt<'a>> {
        self.allocator.boxed(self.expr_stmt(span, expr))
    }
    #[inline]
    pub fn empty_stmt(&self, span: Span) -> EmptyStmt {
        EmptyStmt { span }
    }
    #[inline]
    pub fn box_empty_stmt(&self, span: Span) -> Box<'a, EmptyStmt> {
        self.allocator.boxed(self.empty_stmt(span))
    }
    #[inline]
    pub fn debugger_stmt(&self, span: Span) -> DebuggerStmt {
        DebuggerStmt { span }
    }
    #[inline]
    pub fn box_debugger_stmt(&self, span: Span) -> Box<'a, DebuggerStmt> {
        self.allocator.boxed(self.debugger_stmt(span))
    }
    #[inline]
    pub fn with_stmt(&self, span: Span, obj: Expr<'a>, body: Stmt<'a>) -> WithStmt<'a> {
        WithStmt { span, obj, body }
    }
    #[inline]
    pub fn box_with_stmt(
        &self,
        span: Span,
        obj: Expr<'a>,
        body: Stmt<'a>,
    ) -> Box<'a, WithStmt<'a>> {
        self.allocator.boxed(self.with_stmt(span, obj, body))
    }
    #[inline]
    pub fn return_stmt(&self, span: Span, arg: Option<Expr<'a>>) -> ReturnStmt<'a> {
        ReturnStmt { span, arg }
    }
    #[inline]
    pub fn box_return_stmt(&self, span: Span, arg: Option<Expr<'a>>) -> Box<'a, ReturnStmt<'a>> {
        self.allocator.boxed(self.return_stmt(span, arg))
    }
    #[inline]
    pub fn labeled_stmt(
        &self,
        span: Span,
        label: Box<'a, Ident<'a>>,
        body: Stmt<'a>,
    ) -> LabeledStmt<'a> {
        LabeledStmt { span, label, body }
    }
    #[inline]
    pub fn box_labeled_stmt(
        &self,
        span: Span,
        label: Box<'a, Ident<'a>>,
        body: Stmt<'a>,
    ) -> Box<'a, LabeledStmt<'a>> {
        self.allocator.boxed(self.labeled_stmt(span, label, body))
    }
    #[inline]
    pub fn break_stmt(&self, span: Span, label: Option<Box<'a, Ident<'a>>>) -> BreakStmt<'a> {
        BreakStmt { span, label }
    }
    #[inline]
    pub fn box_break_stmt(
        &self,
        span: Span,
        label: Option<Box<'a, Ident<'a>>>,
    ) -> Box<'a, BreakStmt<'a>> {
        self.allocator.boxed(self.break_stmt(span, label))
    }
    #[inline]
    pub fn continue_stmt(&self, span: Span, label: Option<Box<'a, Ident<'a>>>) -> ContinueStmt<'a> {
        ContinueStmt { span, label }
    }
    #[inline]
    pub fn box_continue_stmt(
        &self,
        span: Span,
        label: Option<Box<'a, Ident<'a>>>,
    ) -> Box<'a, ContinueStmt<'a>> {
        self.allocator.boxed(self.continue_stmt(span, label))
    }
    #[inline]
    pub fn if_stmt(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Stmt<'a>,
        alt: Option<Stmt<'a>>,
    ) -> IfStmt<'a> {
        IfStmt {
            span,
            test,
            cons,
            alt,
        }
    }
    #[inline]
    pub fn box_if_stmt(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Stmt<'a>,
        alt: Option<Stmt<'a>>,
    ) -> Box<'a, IfStmt<'a>> {
        self.allocator.boxed(self.if_stmt(span, test, cons, alt))
    }
    #[inline]
    pub fn switch_stmt(
        &self,
        span: Span,
        discriminant: Expr<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> SwitchStmt<'a> {
        SwitchStmt {
            span,
            discriminant,
            cases,
        }
    }
    #[inline]
    pub fn box_switch_stmt(
        &self,
        span: Span,
        discriminant: Expr<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Box<'a, SwitchStmt<'a>> {
        self.allocator
            .boxed(self.switch_stmt(span, discriminant, cases))
    }
    #[inline]
    pub fn throw_stmt(&self, span: Span, arg: Expr<'a>) -> ThrowStmt<'a> {
        ThrowStmt { span, arg }
    }
    #[inline]
    pub fn box_throw_stmt(&self, span: Span, arg: Expr<'a>) -> Box<'a, ThrowStmt<'a>> {
        self.allocator.boxed(self.throw_stmt(span, arg))
    }
    #[inline]
    pub fn try_stmt(
        &self,
        span: Span,
        block: Box<'a, BlockStmt<'a>>,
        handler: Option<Box<'a, CatchClause<'a>>>,
        finalizer: Option<Box<'a, BlockStmt<'a>>>,
    ) -> TryStmt<'a> {
        TryStmt {
            span,
            block,
            handler,
            finalizer,
        }
    }
    #[inline]
    pub fn box_try_stmt(
        &self,
        span: Span,
        block: Box<'a, BlockStmt<'a>>,
        handler: Option<Box<'a, CatchClause<'a>>>,
        finalizer: Option<Box<'a, BlockStmt<'a>>>,
    ) -> Box<'a, TryStmt<'a>> {
        self.allocator
            .boxed(self.try_stmt(span, block, handler, finalizer))
    }
    #[inline]
    pub fn while_stmt(&self, span: Span, test: Expr<'a>, body: Stmt<'a>) -> WhileStmt<'a> {
        WhileStmt { span, test, body }
    }
    #[inline]
    pub fn box_while_stmt(
        &self,
        span: Span,
        test: Expr<'a>,
        body: Stmt<'a>,
    ) -> Box<'a, WhileStmt<'a>> {
        self.allocator.boxed(self.while_stmt(span, test, body))
    }
    #[inline]
    pub fn do_while_stmt(&self, span: Span, test: Expr<'a>, body: Stmt<'a>) -> DoWhileStmt<'a> {
        DoWhileStmt { span, test, body }
    }
    #[inline]
    pub fn box_do_while_stmt(
        &self,
        span: Span,
        test: Expr<'a>,
        body: Stmt<'a>,
    ) -> Box<'a, DoWhileStmt<'a>> {
        self.allocator.boxed(self.do_while_stmt(span, test, body))
    }
    #[inline]
    pub fn for_stmt(
        &self,
        span: Span,
        init: Option<VarDeclOrExpr<'a>>,
        test: Option<Expr<'a>>,
        update: Option<Expr<'a>>,
        body: Stmt<'a>,
    ) -> ForStmt<'a> {
        ForStmt {
            span,
            init,
            test,
            update,
            body,
        }
    }
    #[inline]
    pub fn box_for_stmt(
        &self,
        span: Span,
        init: Option<VarDeclOrExpr<'a>>,
        test: Option<Expr<'a>>,
        update: Option<Expr<'a>>,
        body: Stmt<'a>,
    ) -> Box<'a, ForStmt<'a>> {
        self.allocator
            .boxed(self.for_stmt(span, init, test, update, body))
    }
    #[inline]
    pub fn for_in_stmt(
        &self,
        span: Span,
        left: ForHead<'a>,
        right: Expr<'a>,
        body: Stmt<'a>,
    ) -> ForInStmt<'a> {
        ForInStmt {
            span,
            left,
            right,
            body,
        }
    }
    #[inline]
    pub fn box_for_in_stmt(
        &self,
        span: Span,
        left: ForHead<'a>,
        right: Expr<'a>,
        body: Stmt<'a>,
    ) -> Box<'a, ForInStmt<'a>> {
        self.allocator
            .boxed(self.for_in_stmt(span, left, right, body))
    }
    #[inline]
    pub fn for_of_stmt(
        &self,
        span: Span,
        is_await: bool,
        left: ForHead<'a>,
        right: Expr<'a>,
        body: Stmt<'a>,
    ) -> ForOfStmt<'a> {
        ForOfStmt {
            span,
            is_await,
            left,
            right,
            body,
        }
    }
    #[inline]
    pub fn box_for_of_stmt(
        &self,
        span: Span,
        is_await: bool,
        left: ForHead<'a>,
        right: Expr<'a>,
        body: Stmt<'a>,
    ) -> Box<'a, ForOfStmt<'a>> {
        self.allocator
            .boxed(self.for_of_stmt(span, is_await, left, right, body))
    }
    #[inline]
    pub fn switch_case(
        &self,
        span: Span,
        test: Option<Expr<'a>>,
        cons: Vec<'a, Stmt<'a>>,
    ) -> SwitchCase<'a> {
        SwitchCase { span, test, cons }
    }
    #[inline]
    pub fn box_switch_case(
        &self,
        span: Span,
        test: Option<Expr<'a>>,
        cons: Vec<'a, Stmt<'a>>,
    ) -> Box<'a, SwitchCase<'a>> {
        self.allocator.boxed(self.switch_case(span, test, cons))
    }
    #[inline]
    pub fn catch_clause(
        &self,
        span: Span,
        param: Option<Pat<'a>>,
        body: Box<'a, BlockStmt<'a>>,
    ) -> CatchClause<'a> {
        CatchClause { span, param, body }
    }
    #[inline]
    pub fn box_catch_clause(
        &self,
        span: Span,
        param: Option<Pat<'a>>,
        body: Box<'a, BlockStmt<'a>>,
    ) -> Box<'a, CatchClause<'a>> {
        self.allocator.boxed(self.catch_clause(span, param, body))
    }
    #[inline]
    pub fn for_head_var_decl(
        &self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> ForHead<'a> {
        ForHead::VarDecl(self.box_var_decl(span, kind, declare, decls))
    }
    #[inline]
    pub fn for_head_using_decl(
        &self,
        span: Span,
        is_await: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> ForHead<'a> {
        ForHead::UsingDecl(self.box_using_decl(span, is_await, decls))
    }
    #[inline]
    pub fn for_head_pat_binding_ident(&self, id: Box<'a, Ident<'a>>) -> ForHead<'a> {
        ForHead::Pat(self.allocator.boxed(Pat::Ident(self.box_binding_ident(id))))
    }
    #[inline]
    pub fn for_head_pat_array_pat(
        &self,
        span: Span,
        elems: Vec<'a, Option<Pat<'a>>>,
        optional: bool,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Array(self.box_array_pat(span, elems, optional))),
        )
    }
    #[inline]
    pub fn for_head_pat_rest_pat(&self, span: Span, dot3_token: Span, arg: Pat<'a>) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Rest(self.box_rest_pat(span, dot3_token, arg))),
        )
    }
    #[inline]
    pub fn for_head_pat_object_pat(
        &self,
        span: Span,
        props: Vec<'a, ObjectPatProp<'a>>,
        optional: bool,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Object(self.box_object_pat(span, props, optional))),
        )
    }
    #[inline]
    pub fn for_head_pat_assign_pat(
        &self,
        span: Span,
        left: Pat<'a>,
        right: Expr<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Assign(self.box_assign_pat(span, left, right))),
        )
    }
    #[inline]
    pub fn for_head_pat_invalid(&self) -> ForHead<'a> {
        ForHead::Pat(self.allocator.boxed(Pat::Invalid(self.box_invalid())))
    }
    #[inline]
    pub fn for_head_pat_expr_this_expr(&self, span: Span) -> ForHead<'a> {
        ForHead::Pat(self.allocator.boxed(Pat::Expr(
            self.allocator.boxed(Expr::This(self.box_this_expr(span))),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_array_lit(
        &self,
        span: Span,
        elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Array(self.box_array_lit(span, elems))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_object_lit(
        &self,
        span: Span,
        props: Vec<'a, PropOrSpread<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Object(self.box_object_lit(span, props))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Fn(self.box_fn_expr(ident, function))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_unary_expr(
        &self,
        span: Span,
        op: UnaryOp,
        arg: Expr<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Unary(self.box_unary_expr(span, op, arg))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_update_expr(
        &self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: Expr<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::Update(
                    self.box_update_expr(span, op, prefix, arg),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_bin_expr(
        &self,
        span: Span,
        op: BinaryOp,
        left: Expr<'a>,
        right: Expr<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Bin(self.box_bin_expr(span, op, left, right))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_assign_expr(
        &self,
        span: Span,
        op: AssignOp,
        left: AssignTarget<'a>,
        right: Expr<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::Assign(
                    self.box_assign_expr(span, op, left, right),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_member_expr(
        &self,
        span: Span,
        obj: Expr<'a>,
        prop: MemberProp<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Member(self.box_member_expr(span, obj, prop))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::SuperProp(
                    self.box_super_prop_expr(span, obj, prop),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_cond_expr(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Expr<'a>,
        alt: Expr<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Cond(self.box_cond_expr(span, test, cons, alt))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_call_expr(
        &self,
        span: Span,
        callee: Callee<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Call(self.box_call_expr(span, callee, args))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_new_expr(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Option<Vec<'a, ExprOrSpread<'a>>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::New(self.box_new_expr(span, callee, args))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_seq_expr(&self, span: Span, exprs: Vec<'a, Expr<'a>>) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Seq(self.box_seq_expr(span, exprs))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_ident(&self, span: Span, sym: Atom<'a>) -> ForHead<'a> {
        ForHead::Pat(self.allocator.boxed(Pat::Expr(
            self.allocator.boxed(Expr::Ident(self.box_ident(span, sym))),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_lit_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator.boxed(Expr::Lit(
                    self.allocator
                        .boxed(Lit::Str(self.box_str(span, value, raw))),
                )),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_lit_bool(&self, span: Span, value: bool) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::Lit(
                    self.allocator.boxed(Lit::Bool(self.box_bool(span, value))),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_lit_null(&self, span: Span) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::Lit(
                    self.allocator.boxed(Lit::Null(self.box_null(span))),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_lit_number(
        &self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator.boxed(Expr::Lit(
                    self.allocator
                        .boxed(Lit::Num(self.box_number(span, value, raw))),
                )),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_lit_big_int(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator.boxed(Expr::Lit(
                    self.allocator
                        .boxed(Lit::BigInt(self.box_big_int(span, value, raw))),
                )),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_lit_regex(
        &self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator.boxed(Expr::Lit(
                    self.allocator
                        .boxed(Lit::Regex(self.box_regex(span, exp, flags))),
                )),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_tpl(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
        quasis: Vec<'a, TplElement<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Tpl(self.box_tpl(span, exprs, quasis))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_tagged_tpl(
        &self,
        span: Span,
        tag: Expr<'a>,
        tpl: Box<'a, Tpl<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::TaggedTpl(self.box_tagged_tpl(span, tag, tpl))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_arrow_expr(
        &self,
        span: Span,
        params: Vec<'a, Pat<'a>>,
        body: BlockStmtOrExpr<'a>,
        is_async: bool,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::Arrow(
                    self.box_arrow_expr(span, params, body, is_async),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Class(self.box_class_expr(ident, class))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_yield_expr(
        &self,
        span: Span,
        arg: Option<Expr<'a>>,
        delegate: bool,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Yield(self.box_yield_expr(span, arg, delegate))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_meta_prop_expr(&self, span: Span, kind: MetaPropKind) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::MetaProp(self.box_meta_prop_expr(span, kind))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_await_expr(&self, span: Span, arg: Expr<'a>) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Await(self.box_await_expr(span, arg))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_paren_expr(&self, span: Span, expr: Expr<'a>) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::Paren(self.box_paren_expr(span, expr))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::JSXMember(
                    self.box_jsx_member_expr(span, obj, prop),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> ForHead<'a> {
        ForHead::Pat(self.allocator.boxed(Pat::Expr(self.allocator.boxed(
            Expr::JSXNamespacedName(self.box_jsx_namespaced_name(span, ns, name)),
        ))))
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_empty_expr(&self, span: Span) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::JSXEmpty(self.box_jsx_empty_expr(span))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::JSXElement(
                    self.box_jsx_element(span, opening, children, closing),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::JSXFragment(
                    self.box_jsx_fragment(span, opening, children, closing),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_private_name(&self, span: Span, name: Atom<'a>) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator.boxed(Pat::Expr(
                self.allocator
                    .boxed(Expr::PrivateName(self.box_private_name(span, name))),
            )),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> ForHead<'a> {
        ForHead::Pat(
            self.allocator
                .boxed(Pat::Expr(self.allocator.boxed(Expr::OptChain(
                    self.box_opt_chain_expr(span, optional, base),
                )))),
        )
    }
    #[inline]
    pub fn for_head_pat_expr_invalid(&self) -> ForHead<'a> {
        ForHead::Pat(self.allocator.boxed(Pat::Expr(
            self.allocator.boxed(Expr::Invalid(self.box_invalid())),
        )))
    }
    #[inline]
    pub fn var_decl_or_expr_var_decl(
        &self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::VarDecl(self.box_var_decl(span, kind, declare, decls))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_this_expr(&self, span: Span) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::This(self.box_this_expr(span))))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_array_lit(
        &self,
        span: Span,
        elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Array(self.box_array_lit(span, elems))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_object_lit(
        &self,
        span: Span,
        props: Vec<'a, PropOrSpread<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Object(self.box_object_lit(span, props))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Fn(self.box_fn_expr(ident, function))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_unary_expr(
        &self,
        span: Span,
        op: UnaryOp,
        arg: Expr<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Unary(self.box_unary_expr(span, op, arg))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_update_expr(
        &self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: Expr<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Update(self.box_update_expr(span, op, prefix, arg))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_bin_expr(
        &self,
        span: Span,
        op: BinaryOp,
        left: Expr<'a>,
        right: Expr<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Bin(self.box_bin_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_assign_expr(
        &self,
        span: Span,
        op: AssignOp,
        left: AssignTarget<'a>,
        right: Expr<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Assign(self.box_assign_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_member_expr(
        &self,
        span: Span,
        obj: Expr<'a>,
        prop: MemberProp<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Member(self.box_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::SuperProp(self.box_super_prop_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_cond_expr(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Expr<'a>,
        alt: Expr<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Cond(self.box_cond_expr(span, test, cons, alt))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_call_expr(
        &self,
        span: Span,
        callee: Callee<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Call(self.box_call_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_new_expr(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Option<Vec<'a, ExprOrSpread<'a>>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::New(self.box_new_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_seq_expr(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Seq(self.box_seq_expr(span, exprs))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_ident(&self, span: Span, sym: Atom<'a>) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::Ident(self.box_ident(span, sym))))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Str(self.box_str(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_bool(&self, span: Span, value: bool) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Bool(self.box_bool(span, value))),
        )))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_null(&self, span: Span) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Null(self.box_null(span))),
        )))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_number(
        &self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Num(self.box_number(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_big_int(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::BigInt(self.box_big_int(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_regex(
        &self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Regex(self.box_regex(span, exp, flags))),
            )),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_tpl(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
        quasis: Vec<'a, TplElement<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Tpl(self.box_tpl(span, exprs, quasis))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_tagged_tpl(
        &self,
        span: Span,
        tag: Expr<'a>,
        tpl: Box<'a, Tpl<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::TaggedTpl(self.box_tagged_tpl(span, tag, tpl))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_arrow_expr(
        &self,
        span: Span,
        params: Vec<'a, Pat<'a>>,
        body: BlockStmtOrExpr<'a>,
        is_async: bool,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::Arrow(
            self.box_arrow_expr(span, params, body, is_async),
        )))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Class(self.box_class_expr(ident, class))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_yield_expr(
        &self,
        span: Span,
        arg: Option<Expr<'a>>,
        delegate: bool,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Yield(self.box_yield_expr(span, arg, delegate))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_meta_prop_expr(
        &self,
        span: Span,
        kind: MetaPropKind,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::MetaProp(self.box_meta_prop_expr(span, kind))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_await_expr(&self, span: Span, arg: Expr<'a>) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Await(self.box_await_expr(span, arg))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_paren_expr(
        &self,
        span: Span,
        expr: Expr<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::Paren(self.box_paren_expr(span, expr))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::JSXMember(self.box_jsx_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::JSXNamespacedName(
            self.box_jsx_namespaced_name(span, ns, name),
        )))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_empty_expr(&self, span: Span) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::JSXEmpty(self.box_jsx_empty_expr(span))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::JSXElement(
            self.box_jsx_element(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::JSXFragment(
            self.box_jsx_fragment(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_private_name(
        &self,
        span: Span,
        name: Atom<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(
            self.allocator
                .boxed(Expr::PrivateName(self.box_private_name(span, name))),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_expr_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::OptChain(
            self.box_opt_chain_expr(span, optional, base),
        )))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_invalid(&self) -> VarDeclOrExpr<'a> {
        VarDeclOrExpr::Expr(self.allocator.boxed(Expr::Invalid(self.box_invalid())))
    }
    #[inline]
    pub fn decl_class_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        class: Box<'a, Class<'a>>,
    ) -> Decl<'a> {
        Decl::Class(self.box_class_decl(ident, declare, class))
    }
    #[inline]
    pub fn decl_fn_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        function: Box<'a, Function<'a>>,
    ) -> Decl<'a> {
        Decl::Fn(self.box_fn_decl(ident, declare, function))
    }
    #[inline]
    pub fn decl_var_decl(
        &self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> Decl<'a> {
        Decl::Var(self.box_var_decl(span, kind, declare, decls))
    }
    #[inline]
    pub fn decl_using_decl(
        &self,
        span: Span,
        is_await: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> Decl<'a> {
        Decl::Using(self.box_using_decl(span, is_await, decls))
    }
    #[inline]
    pub fn fn_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        function: Box<'a, Function<'a>>,
    ) -> FnDecl<'a> {
        FnDecl {
            ident,
            declare,
            function,
        }
    }
    #[inline]
    pub fn box_fn_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        function: Box<'a, Function<'a>>,
    ) -> Box<'a, FnDecl<'a>> {
        self.allocator.boxed(self.fn_decl(ident, declare, function))
    }
    #[inline]
    pub fn class_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        class: Box<'a, Class<'a>>,
    ) -> ClassDecl<'a> {
        ClassDecl {
            ident,
            declare,
            class,
        }
    }
    #[inline]
    pub fn box_class_decl(
        &self,
        ident: Box<'a, Ident<'a>>,
        declare: bool,
        class: Box<'a, Class<'a>>,
    ) -> Box<'a, ClassDecl<'a>> {
        self.allocator.boxed(self.class_decl(ident, declare, class))
    }
    #[inline]
    pub fn var_decl(
        &self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> VarDecl<'a> {
        VarDecl {
            span,
            kind,
            declare,
            decls,
        }
    }
    #[inline]
    pub fn box_var_decl(
        &self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> Box<'a, VarDecl<'a>> {
        self.allocator
            .boxed(self.var_decl(span, kind, declare, decls))
    }
    #[inline]
    pub fn var_declarator(
        &self,
        span: Span,
        name: Pat<'a>,
        init: Option<Expr<'a>>,
    ) -> VarDeclarator<'a> {
        VarDeclarator { span, name, init }
    }
    #[inline]
    pub fn box_var_declarator(
        &self,
        span: Span,
        name: Pat<'a>,
        init: Option<Expr<'a>>,
    ) -> Box<'a, VarDeclarator<'a>> {
        self.allocator.boxed(self.var_declarator(span, name, init))
    }
    #[inline]
    pub fn using_decl(
        &self,
        span: Span,
        is_await: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> UsingDecl<'a> {
        UsingDecl {
            span,
            is_await,
            decls,
        }
    }
    #[inline]
    pub fn box_using_decl(
        &self,
        span: Span,
        is_await: bool,
        decls: Vec<'a, VarDeclarator<'a>>,
    ) -> Box<'a, UsingDecl<'a>> {
        self.allocator.boxed(self.using_decl(span, is_await, decls))
    }
    #[inline]
    pub fn expr_this_expr(&self, span: Span) -> Expr<'a> {
        Expr::This(self.box_this_expr(span))
    }
    #[inline]
    pub fn expr_array_lit(
        &self,
        span: Span,
        elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) -> Expr<'a> {
        Expr::Array(self.box_array_lit(span, elems))
    }
    #[inline]
    pub fn expr_object_lit(&self, span: Span, props: Vec<'a, PropOrSpread<'a>>) -> Expr<'a> {
        Expr::Object(self.box_object_lit(span, props))
    }
    #[inline]
    pub fn expr_fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> Expr<'a> {
        Expr::Fn(self.box_fn_expr(ident, function))
    }
    #[inline]
    pub fn expr_unary_expr(&self, span: Span, op: UnaryOp, arg: Expr<'a>) -> Expr<'a> {
        Expr::Unary(self.box_unary_expr(span, op, arg))
    }
    #[inline]
    pub fn expr_update_expr(
        &self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: Expr<'a>,
    ) -> Expr<'a> {
        Expr::Update(self.box_update_expr(span, op, prefix, arg))
    }
    #[inline]
    pub fn expr_bin_expr(
        &self,
        span: Span,
        op: BinaryOp,
        left: Expr<'a>,
        right: Expr<'a>,
    ) -> Expr<'a> {
        Expr::Bin(self.box_bin_expr(span, op, left, right))
    }
    #[inline]
    pub fn expr_assign_expr(
        &self,
        span: Span,
        op: AssignOp,
        left: AssignTarget<'a>,
        right: Expr<'a>,
    ) -> Expr<'a> {
        Expr::Assign(self.box_assign_expr(span, op, left, right))
    }
    #[inline]
    pub fn expr_member_expr(&self, span: Span, obj: Expr<'a>, prop: MemberProp<'a>) -> Expr<'a> {
        Expr::Member(self.box_member_expr(span, obj, prop))
    }
    #[inline]
    pub fn expr_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> Expr<'a> {
        Expr::SuperProp(self.box_super_prop_expr(span, obj, prop))
    }
    #[inline]
    pub fn expr_cond_expr(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Expr<'a>,
        alt: Expr<'a>,
    ) -> Expr<'a> {
        Expr::Cond(self.box_cond_expr(span, test, cons, alt))
    }
    #[inline]
    pub fn expr_call_expr(
        &self,
        span: Span,
        callee: Callee<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> Expr<'a> {
        Expr::Call(self.box_call_expr(span, callee, args))
    }
    #[inline]
    pub fn expr_new_expr(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Option<Vec<'a, ExprOrSpread<'a>>>,
    ) -> Expr<'a> {
        Expr::New(self.box_new_expr(span, callee, args))
    }
    #[inline]
    pub fn expr_seq_expr(&self, span: Span, exprs: Vec<'a, Expr<'a>>) -> Expr<'a> {
        Expr::Seq(self.box_seq_expr(span, exprs))
    }
    #[inline]
    pub fn expr_ident(&self, span: Span, sym: Atom<'a>) -> Expr<'a> {
        Expr::Ident(self.box_ident(span, sym))
    }
    #[inline]
    pub fn expr_lit_str(&self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Expr<'a> {
        Expr::Lit(
            self.allocator
                .boxed(Lit::Str(self.box_str(span, value, raw))),
        )
    }
    #[inline]
    pub fn expr_lit_bool(&self, span: Span, value: bool) -> Expr<'a> {
        Expr::Lit(self.allocator.boxed(Lit::Bool(self.box_bool(span, value))))
    }
    #[inline]
    pub fn expr_lit_null(&self, span: Span) -> Expr<'a> {
        Expr::Lit(self.allocator.boxed(Lit::Null(self.box_null(span))))
    }
    #[inline]
    pub fn expr_lit_number(&self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Expr<'a> {
        Expr::Lit(
            self.allocator
                .boxed(Lit::Num(self.box_number(span, value, raw))),
        )
    }
    #[inline]
    pub fn expr_lit_big_int(&self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> Expr<'a> {
        Expr::Lit(
            self.allocator
                .boxed(Lit::BigInt(self.box_big_int(span, value, raw))),
        )
    }
    #[inline]
    pub fn expr_lit_regex(&self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Expr<'a> {
        Expr::Lit(
            self.allocator
                .boxed(Lit::Regex(self.box_regex(span, exp, flags))),
        )
    }
    #[inline]
    pub fn expr_tpl(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
        quasis: Vec<'a, TplElement<'a>>,
    ) -> Expr<'a> {
        Expr::Tpl(self.box_tpl(span, exprs, quasis))
    }
    #[inline]
    pub fn expr_tagged_tpl(&self, span: Span, tag: Expr<'a>, tpl: Box<'a, Tpl<'a>>) -> Expr<'a> {
        Expr::TaggedTpl(self.box_tagged_tpl(span, tag, tpl))
    }
    #[inline]
    pub fn expr_arrow_expr(
        &self,
        span: Span,
        params: Vec<'a, Pat<'a>>,
        body: BlockStmtOrExpr<'a>,
        is_async: bool,
    ) -> Expr<'a> {
        Expr::Arrow(self.box_arrow_expr(span, params, body, is_async))
    }
    #[inline]
    pub fn expr_class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> Expr<'a> {
        Expr::Class(self.box_class_expr(ident, class))
    }
    #[inline]
    pub fn expr_yield_expr(&self, span: Span, arg: Option<Expr<'a>>, delegate: bool) -> Expr<'a> {
        Expr::Yield(self.box_yield_expr(span, arg, delegate))
    }
    #[inline]
    pub fn expr_meta_prop_expr(&self, span: Span, kind: MetaPropKind) -> Expr<'a> {
        Expr::MetaProp(self.box_meta_prop_expr(span, kind))
    }
    #[inline]
    pub fn expr_await_expr(&self, span: Span, arg: Expr<'a>) -> Expr<'a> {
        Expr::Await(self.box_await_expr(span, arg))
    }
    #[inline]
    pub fn expr_paren_expr(&self, span: Span, expr: Expr<'a>) -> Expr<'a> {
        Expr::Paren(self.box_paren_expr(span, expr))
    }
    #[inline]
    pub fn expr_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> Expr<'a> {
        Expr::JSXMember(self.box_jsx_member_expr(span, obj, prop))
    }
    #[inline]
    pub fn expr_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> Expr<'a> {
        Expr::JSXNamespacedName(self.box_jsx_namespaced_name(span, ns, name))
    }
    #[inline]
    pub fn expr_jsx_empty_expr(&self, span: Span) -> Expr<'a> {
        Expr::JSXEmpty(self.box_jsx_empty_expr(span))
    }
    #[inline]
    pub fn expr_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> Expr<'a> {
        Expr::JSXElement(self.box_jsx_element(span, opening, children, closing))
    }
    #[inline]
    pub fn expr_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> Expr<'a> {
        Expr::JSXFragment(self.box_jsx_fragment(span, opening, children, closing))
    }
    #[inline]
    pub fn expr_private_name(&self, span: Span, name: Atom<'a>) -> Expr<'a> {
        Expr::PrivateName(self.box_private_name(span, name))
    }
    #[inline]
    pub fn expr_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> Expr<'a> {
        Expr::OptChain(self.box_opt_chain_expr(span, optional, base))
    }
    #[inline]
    pub fn expr_invalid(&self) -> Expr<'a> {
        Expr::Invalid(self.box_invalid())
    }
    #[inline]
    pub fn this_expr(&self, span: Span) -> ThisExpr {
        ThisExpr { span }
    }
    #[inline]
    pub fn box_this_expr(&self, span: Span) -> Box<'a, ThisExpr> {
        self.allocator.boxed(self.this_expr(span))
    }
    #[inline]
    pub fn array_lit(
        &self,
        span: Span,
        elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) -> ArrayLit<'a> {
        ArrayLit { span, elems }
    }
    #[inline]
    pub fn box_array_lit(
        &self,
        span: Span,
        elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) -> Box<'a, ArrayLit<'a>> {
        self.allocator.boxed(self.array_lit(span, elems))
    }
    #[inline]
    pub fn object_lit(&self, span: Span, props: Vec<'a, PropOrSpread<'a>>) -> ObjectLit<'a> {
        ObjectLit { span, props }
    }
    #[inline]
    pub fn box_object_lit(
        &self,
        span: Span,
        props: Vec<'a, PropOrSpread<'a>>,
    ) -> Box<'a, ObjectLit<'a>> {
        self.allocator.boxed(self.object_lit(span, props))
    }
    #[inline]
    pub fn prop_or_spread_spread_element(
        &self,
        dot3_token: Span,
        expr: Expr<'a>,
    ) -> PropOrSpread<'a> {
        PropOrSpread::Spread(self.box_spread_element(dot3_token, expr))
    }
    #[inline]
    pub fn prop_or_spread_prop_ident(&self, span: Span, sym: Atom<'a>) -> PropOrSpread<'a> {
        PropOrSpread::Prop(
            self.allocator
                .boxed(Prop::Shorthand(self.box_ident(span, sym))),
        )
    }
    #[inline]
    pub fn prop_or_spread_prop_key_value_prop(
        &self,
        key: PropName<'a>,
        value: Expr<'a>,
    ) -> PropOrSpread<'a> {
        PropOrSpread::Prop(
            self.allocator
                .boxed(Prop::KeyValue(self.box_key_value_prop(key, value))),
        )
    }
    #[inline]
    pub fn prop_or_spread_prop_assign_prop(
        &self,
        span: Span,
        key: Box<'a, Ident<'a>>,
        value: Expr<'a>,
    ) -> PropOrSpread<'a> {
        PropOrSpread::Prop(
            self.allocator
                .boxed(Prop::Assign(self.box_assign_prop(span, key, value))),
        )
    }
    #[inline]
    pub fn prop_or_spread_prop_getter_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> PropOrSpread<'a> {
        PropOrSpread::Prop(
            self.allocator
                .boxed(Prop::Getter(self.box_getter_prop(span, key, body))),
        )
    }
    #[inline]
    pub fn prop_or_spread_prop_setter_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        this_param: Option<Pat<'a>>,
        param: Pat<'a>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> PropOrSpread<'a> {
        PropOrSpread::Prop(self.allocator.boxed(Prop::Setter(
            self.box_setter_prop(span, key, this_param, param, body),
        )))
    }
    #[inline]
    pub fn prop_or_spread_prop_method_prop(
        &self,
        key: PropName<'a>,
        function: Box<'a, Function<'a>>,
    ) -> PropOrSpread<'a> {
        PropOrSpread::Prop(
            self.allocator
                .boxed(Prop::Method(self.box_method_prop(key, function))),
        )
    }
    #[inline]
    pub fn spread_element(&self, dot3_token: Span, expr: Expr<'a>) -> SpreadElement<'a> {
        SpreadElement { dot3_token, expr }
    }
    #[inline]
    pub fn box_spread_element(
        &self,
        dot3_token: Span,
        expr: Expr<'a>,
    ) -> Box<'a, SpreadElement<'a>> {
        self.allocator.boxed(self.spread_element(dot3_token, expr))
    }
    #[inline]
    pub fn unary_expr(&self, span: Span, op: UnaryOp, arg: Expr<'a>) -> UnaryExpr<'a> {
        UnaryExpr { span, op, arg }
    }
    #[inline]
    pub fn box_unary_expr(&self, span: Span, op: UnaryOp, arg: Expr<'a>) -> Box<'a, UnaryExpr<'a>> {
        self.allocator.boxed(self.unary_expr(span, op, arg))
    }
    #[inline]
    pub fn update_expr(
        &self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: Expr<'a>,
    ) -> UpdateExpr<'a> {
        UpdateExpr {
            span,
            op,
            prefix,
            arg,
        }
    }
    #[inline]
    pub fn box_update_expr(
        &self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: Expr<'a>,
    ) -> Box<'a, UpdateExpr<'a>> {
        self.allocator
            .boxed(self.update_expr(span, op, prefix, arg))
    }
    #[inline]
    pub fn bin_expr(
        &self,
        span: Span,
        op: BinaryOp,
        left: Expr<'a>,
        right: Expr<'a>,
    ) -> BinExpr<'a> {
        BinExpr {
            span,
            op,
            left,
            right,
        }
    }
    #[inline]
    pub fn box_bin_expr(
        &self,
        span: Span,
        op: BinaryOp,
        left: Expr<'a>,
        right: Expr<'a>,
    ) -> Box<'a, BinExpr<'a>> {
        self.allocator.boxed(self.bin_expr(span, op, left, right))
    }
    #[inline]
    pub fn fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> FnExpr<'a> {
        FnExpr { ident, function }
    }
    #[inline]
    pub fn box_fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> Box<'a, FnExpr<'a>> {
        self.allocator.boxed(self.fn_expr(ident, function))
    }
    #[inline]
    pub fn class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> ClassExpr<'a> {
        ClassExpr { ident, class }
    }
    #[inline]
    pub fn box_class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> Box<'a, ClassExpr<'a>> {
        self.allocator.boxed(self.class_expr(ident, class))
    }
    #[inline]
    pub fn assign_expr(
        &self,
        span: Span,
        op: AssignOp,
        left: AssignTarget<'a>,
        right: Expr<'a>,
    ) -> AssignExpr<'a> {
        AssignExpr {
            span,
            op,
            left,
            right,
        }
    }
    #[inline]
    pub fn box_assign_expr(
        &self,
        span: Span,
        op: AssignOp,
        left: AssignTarget<'a>,
        right: Expr<'a>,
    ) -> Box<'a, AssignExpr<'a>> {
        self.allocator
            .boxed(self.assign_expr(span, op, left, right))
    }
    #[inline]
    pub fn member_expr(&self, span: Span, obj: Expr<'a>, prop: MemberProp<'a>) -> MemberExpr<'a> {
        MemberExpr { span, obj, prop }
    }
    #[inline]
    pub fn box_member_expr(
        &self,
        span: Span,
        obj: Expr<'a>,
        prop: MemberProp<'a>,
    ) -> Box<'a, MemberExpr<'a>> {
        self.allocator.boxed(self.member_expr(span, obj, prop))
    }
    #[inline]
    pub fn member_prop_ident_name(&self, span: Span, sym: Atom<'a>) -> MemberProp<'a> {
        MemberProp::Ident(self.box_ident_name(span, sym))
    }
    #[inline]
    pub fn member_prop_private_name(&self, span: Span, name: Atom<'a>) -> MemberProp<'a> {
        MemberProp::PrivateName(self.box_private_name(span, name))
    }
    #[inline]
    pub fn member_prop_computed_prop_name(&self, span: Span, expr: Expr<'a>) -> MemberProp<'a> {
        MemberProp::Computed(self.box_computed_prop_name(span, expr))
    }
    #[inline]
    pub fn super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> SuperPropExpr<'a> {
        SuperPropExpr { span, obj, prop }
    }
    #[inline]
    pub fn box_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> Box<'a, SuperPropExpr<'a>> {
        self.allocator.boxed(self.super_prop_expr(span, obj, prop))
    }
    #[inline]
    pub fn super_prop_ident_name(&self, span: Span, sym: Atom<'a>) -> SuperProp<'a> {
        SuperProp::Ident(self.box_ident_name(span, sym))
    }
    #[inline]
    pub fn super_prop_computed_prop_name(&self, span: Span, expr: Expr<'a>) -> SuperProp<'a> {
        SuperProp::Computed(self.box_computed_prop_name(span, expr))
    }
    #[inline]
    pub fn cond_expr(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Expr<'a>,
        alt: Expr<'a>,
    ) -> CondExpr<'a> {
        CondExpr {
            span,
            test,
            cons,
            alt,
        }
    }
    #[inline]
    pub fn box_cond_expr(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Expr<'a>,
        alt: Expr<'a>,
    ) -> Box<'a, CondExpr<'a>> {
        self.allocator.boxed(self.cond_expr(span, test, cons, alt))
    }
    #[inline]
    pub fn call_expr(
        &self,
        span: Span,
        callee: Callee<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> CallExpr<'a> {
        CallExpr { span, callee, args }
    }
    #[inline]
    pub fn box_call_expr(
        &self,
        span: Span,
        callee: Callee<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> Box<'a, CallExpr<'a>> {
        self.allocator.boxed(self.call_expr(span, callee, args))
    }
    #[inline]
    pub fn new_expr(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Option<Vec<'a, ExprOrSpread<'a>>>,
    ) -> NewExpr<'a> {
        NewExpr { span, callee, args }
    }
    #[inline]
    pub fn box_new_expr(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Option<Vec<'a, ExprOrSpread<'a>>>,
    ) -> Box<'a, NewExpr<'a>> {
        self.allocator.boxed(self.new_expr(span, callee, args))
    }
    #[inline]
    pub fn seq_expr(&self, span: Span, exprs: Vec<'a, Expr<'a>>) -> SeqExpr<'a> {
        SeqExpr { span, exprs }
    }
    #[inline]
    pub fn box_seq_expr(&self, span: Span, exprs: Vec<'a, Expr<'a>>) -> Box<'a, SeqExpr<'a>> {
        self.allocator.boxed(self.seq_expr(span, exprs))
    }
    #[inline]
    pub fn arrow_expr(
        &self,
        span: Span,
        params: Vec<'a, Pat<'a>>,
        body: BlockStmtOrExpr<'a>,
        is_async: bool,
    ) -> ArrowExpr<'a> {
        ArrowExpr {
            span,
            params,
            body,
            is_async,
        }
    }
    #[inline]
    pub fn box_arrow_expr(
        &self,
        span: Span,
        params: Vec<'a, Pat<'a>>,
        body: BlockStmtOrExpr<'a>,
        is_async: bool,
    ) -> Box<'a, ArrowExpr<'a>> {
        self.allocator
            .boxed(self.arrow_expr(span, params, body, is_async))
    }
    #[inline]
    pub fn yield_expr(&self, span: Span, arg: Option<Expr<'a>>, delegate: bool) -> YieldExpr<'a> {
        YieldExpr {
            span,
            arg,
            delegate,
        }
    }
    #[inline]
    pub fn box_yield_expr(
        &self,
        span: Span,
        arg: Option<Expr<'a>>,
        delegate: bool,
    ) -> Box<'a, YieldExpr<'a>> {
        self.allocator.boxed(self.yield_expr(span, arg, delegate))
    }
    #[inline]
    pub fn meta_prop_expr(&self, span: Span, kind: MetaPropKind) -> MetaPropExpr {
        MetaPropExpr { span, kind }
    }
    #[inline]
    pub fn box_meta_prop_expr(&self, span: Span, kind: MetaPropKind) -> Box<'a, MetaPropExpr> {
        self.allocator.boxed(self.meta_prop_expr(span, kind))
    }
    #[inline]
    pub fn await_expr(&self, span: Span, arg: Expr<'a>) -> AwaitExpr<'a> {
        AwaitExpr { span, arg }
    }
    #[inline]
    pub fn box_await_expr(&self, span: Span, arg: Expr<'a>) -> Box<'a, AwaitExpr<'a>> {
        self.allocator.boxed(self.await_expr(span, arg))
    }
    #[inline]
    pub fn tpl(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
        quasis: Vec<'a, TplElement<'a>>,
    ) -> Tpl<'a> {
        Tpl {
            span,
            exprs,
            quasis,
        }
    }
    #[inline]
    pub fn box_tpl(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
        quasis: Vec<'a, TplElement<'a>>,
    ) -> Box<'a, Tpl<'a>> {
        self.allocator.boxed(self.tpl(span, exprs, quasis))
    }
    #[inline]
    pub fn tagged_tpl(&self, span: Span, tag: Expr<'a>, tpl: Box<'a, Tpl<'a>>) -> TaggedTpl<'a> {
        TaggedTpl { span, tag, tpl }
    }
    #[inline]
    pub fn box_tagged_tpl(
        &self,
        span: Span,
        tag: Expr<'a>,
        tpl: Box<'a, Tpl<'a>>,
    ) -> Box<'a, TaggedTpl<'a>> {
        self.allocator.boxed(self.tagged_tpl(span, tag, tpl))
    }
    #[inline]
    pub fn tpl_element(
        &self,
        span: Span,
        tail: bool,
        cooked: Option<Wtf8Atom<'a>>,
        raw: Atom<'a>,
    ) -> TplElement<'a> {
        TplElement {
            span,
            tail,
            cooked,
            raw,
        }
    }
    #[inline]
    pub fn box_tpl_element(
        &self,
        span: Span,
        tail: bool,
        cooked: Option<Wtf8Atom<'a>>,
        raw: Atom<'a>,
    ) -> Box<'a, TplElement<'a>> {
        self.allocator
            .boxed(self.tpl_element(span, tail, cooked, raw))
    }
    #[inline]
    pub fn paren_expr(&self, span: Span, expr: Expr<'a>) -> ParenExpr<'a> {
        ParenExpr { span, expr }
    }
    #[inline]
    pub fn box_paren_expr(&self, span: Span, expr: Expr<'a>) -> Box<'a, ParenExpr<'a>> {
        self.allocator.boxed(self.paren_expr(span, expr))
    }
    #[inline]
    pub fn callee_super(&self, span: Span) -> Callee<'a> {
        Callee::Super(self.box_super(span))
    }
    #[inline]
    pub fn callee_import(&self, span: Span, phase: ImportPhase) -> Callee<'a> {
        Callee::Import(self.box_import(span, phase))
    }
    #[inline]
    pub fn callee_expr_this_expr(&self, span: Span) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::This(self.box_this_expr(span))))
    }
    #[inline]
    pub fn callee_expr_array_lit(
        &self,
        span: Span,
        elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Array(self.box_array_lit(span, elems))),
        )
    }
    #[inline]
    pub fn callee_expr_object_lit(
        &self,
        span: Span,
        props: Vec<'a, PropOrSpread<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Object(self.box_object_lit(span, props))),
        )
    }
    #[inline]
    pub fn callee_expr_fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Fn(self.box_fn_expr(ident, function))),
        )
    }
    #[inline]
    pub fn callee_expr_unary_expr(&self, span: Span, op: UnaryOp, arg: Expr<'a>) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Unary(self.box_unary_expr(span, op, arg))),
        )
    }
    #[inline]
    pub fn callee_expr_update_expr(
        &self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: Expr<'a>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Update(self.box_update_expr(span, op, prefix, arg))),
        )
    }
    #[inline]
    pub fn callee_expr_bin_expr(
        &self,
        span: Span,
        op: BinaryOp,
        left: Expr<'a>,
        right: Expr<'a>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Bin(self.box_bin_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn callee_expr_assign_expr(
        &self,
        span: Span,
        op: AssignOp,
        left: AssignTarget<'a>,
        right: Expr<'a>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Assign(self.box_assign_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn callee_expr_member_expr(
        &self,
        span: Span,
        obj: Expr<'a>,
        prop: MemberProp<'a>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Member(self.box_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn callee_expr_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::SuperProp(self.box_super_prop_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn callee_expr_cond_expr(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Expr<'a>,
        alt: Expr<'a>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Cond(self.box_cond_expr(span, test, cons, alt))),
        )
    }
    #[inline]
    pub fn callee_expr_call_expr(
        &self,
        span: Span,
        callee: Callee<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Call(self.box_call_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn callee_expr_new_expr(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Option<Vec<'a, ExprOrSpread<'a>>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::New(self.box_new_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn callee_expr_seq_expr(&self, span: Span, exprs: Vec<'a, Expr<'a>>) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Seq(self.box_seq_expr(span, exprs))),
        )
    }
    #[inline]
    pub fn callee_expr_ident(&self, span: Span, sym: Atom<'a>) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::Ident(self.box_ident(span, sym))))
    }
    #[inline]
    pub fn callee_expr_lit_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Str(self.box_str(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn callee_expr_lit_bool(&self, span: Span, value: bool) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Bool(self.box_bool(span, value))),
        )))
    }
    #[inline]
    pub fn callee_expr_lit_null(&self, span: Span) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Null(self.box_null(span))),
        )))
    }
    #[inline]
    pub fn callee_expr_lit_number(
        &self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Num(self.box_number(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn callee_expr_lit_big_int(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::BigInt(self.box_big_int(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn callee_expr_lit_regex(&self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Callee<'a> {
        Callee::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Regex(self.box_regex(span, exp, flags))),
            )),
        )
    }
    #[inline]
    pub fn callee_expr_tpl(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
        quasis: Vec<'a, TplElement<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Tpl(self.box_tpl(span, exprs, quasis))),
        )
    }
    #[inline]
    pub fn callee_expr_tagged_tpl(
        &self,
        span: Span,
        tag: Expr<'a>,
        tpl: Box<'a, Tpl<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::TaggedTpl(self.box_tagged_tpl(span, tag, tpl))),
        )
    }
    #[inline]
    pub fn callee_expr_arrow_expr(
        &self,
        span: Span,
        params: Vec<'a, Pat<'a>>,
        body: BlockStmtOrExpr<'a>,
        is_async: bool,
    ) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::Arrow(
            self.box_arrow_expr(span, params, body, is_async),
        )))
    }
    #[inline]
    pub fn callee_expr_class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Class(self.box_class_expr(ident, class))),
        )
    }
    #[inline]
    pub fn callee_expr_yield_expr(
        &self,
        span: Span,
        arg: Option<Expr<'a>>,
        delegate: bool,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Yield(self.box_yield_expr(span, arg, delegate))),
        )
    }
    #[inline]
    pub fn callee_expr_meta_prop_expr(&self, span: Span, kind: MetaPropKind) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::MetaProp(self.box_meta_prop_expr(span, kind))),
        )
    }
    #[inline]
    pub fn callee_expr_await_expr(&self, span: Span, arg: Expr<'a>) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Await(self.box_await_expr(span, arg))),
        )
    }
    #[inline]
    pub fn callee_expr_paren_expr(&self, span: Span, expr: Expr<'a>) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::Paren(self.box_paren_expr(span, expr))),
        )
    }
    #[inline]
    pub fn callee_expr_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::JSXMember(self.box_jsx_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn callee_expr_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::JSXNamespacedName(
            self.box_jsx_namespaced_name(span, ns, name),
        )))
    }
    #[inline]
    pub fn callee_expr_jsx_empty_expr(&self, span: Span) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::JSXEmpty(self.box_jsx_empty_expr(span))),
        )
    }
    #[inline]
    pub fn callee_expr_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::JSXElement(
            self.box_jsx_element(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn callee_expr_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::JSXFragment(
            self.box_jsx_fragment(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn callee_expr_private_name(&self, span: Span, name: Atom<'a>) -> Callee<'a> {
        Callee::Expr(
            self.allocator
                .boxed(Expr::PrivateName(self.box_private_name(span, name))),
        )
    }
    #[inline]
    pub fn callee_expr_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::OptChain(
            self.box_opt_chain_expr(span, optional, base),
        )))
    }
    #[inline]
    pub fn callee_expr_invalid(&self) -> Callee<'a> {
        Callee::Expr(self.allocator.boxed(Expr::Invalid(self.box_invalid())))
    }
    #[inline]
    pub fn super_(&self, span: Span) -> Super {
        Super { span }
    }
    #[inline]
    pub fn box_super(&self, span: Span) -> Box<'a, Super> {
        self.allocator.boxed(self.super_(span))
    }
    #[inline]
    pub fn import(&self, span: Span, phase: ImportPhase) -> Import {
        Import { span, phase }
    }
    #[inline]
    pub fn box_import(&self, span: Span, phase: ImportPhase) -> Box<'a, Import> {
        self.allocator.boxed(self.import(span, phase))
    }
    #[inline]
    pub fn expr_or_spread(&self, spread: Option<Span>, expr: Expr<'a>) -> ExprOrSpread<'a> {
        ExprOrSpread { spread, expr }
    }
    #[inline]
    pub fn box_expr_or_spread(
        &self,
        spread: Option<Span>,
        expr: Expr<'a>,
    ) -> Box<'a, ExprOrSpread<'a>> {
        self.allocator.boxed(self.expr_or_spread(spread, expr))
    }
    #[inline]
    pub fn block_stmt_or_expr_block_stmt(
        &self,
        span: Span,
        stmts: Vec<'a, Stmt<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::BlockStmt(self.box_block_stmt(span, stmts))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_this_expr(&self, span: Span) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::This(self.box_this_expr(span))))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_array_lit(
        &self,
        span: Span,
        elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Array(self.box_array_lit(span, elems))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_object_lit(
        &self,
        span: Span,
        props: Vec<'a, PropOrSpread<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Object(self.box_object_lit(span, props))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Fn(self.box_fn_expr(ident, function))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_unary_expr(
        &self,
        span: Span,
        op: UnaryOp,
        arg: Expr<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Unary(self.box_unary_expr(span, op, arg))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_update_expr(
        &self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: Expr<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Update(self.box_update_expr(span, op, prefix, arg))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_bin_expr(
        &self,
        span: Span,
        op: BinaryOp,
        left: Expr<'a>,
        right: Expr<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Bin(self.box_bin_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_assign_expr(
        &self,
        span: Span,
        op: AssignOp,
        left: AssignTarget<'a>,
        right: Expr<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Assign(self.box_assign_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_member_expr(
        &self,
        span: Span,
        obj: Expr<'a>,
        prop: MemberProp<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Member(self.box_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::SuperProp(self.box_super_prop_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_cond_expr(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Expr<'a>,
        alt: Expr<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Cond(self.box_cond_expr(span, test, cons, alt))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_call_expr(
        &self,
        span: Span,
        callee: Callee<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Call(self.box_call_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_new_expr(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Option<Vec<'a, ExprOrSpread<'a>>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::New(self.box_new_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_seq_expr(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Seq(self.box_seq_expr(span, exprs))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_ident(&self, span: Span, sym: Atom<'a>) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::Ident(self.box_ident(span, sym))))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Str(self.box_str(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_bool(&self, span: Span, value: bool) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Bool(self.box_bool(span, value))),
        )))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_null(&self, span: Span) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Null(self.box_null(span))),
        )))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_number(
        &self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Num(self.box_number(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_big_int(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::BigInt(self.box_big_int(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_regex(
        &self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Regex(self.box_regex(span, exp, flags))),
            )),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_tpl(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
        quasis: Vec<'a, TplElement<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Tpl(self.box_tpl(span, exprs, quasis))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_tagged_tpl(
        &self,
        span: Span,
        tag: Expr<'a>,
        tpl: Box<'a, Tpl<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::TaggedTpl(self.box_tagged_tpl(span, tag, tpl))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_arrow_expr(
        &self,
        span: Span,
        params: Vec<'a, Pat<'a>>,
        body: BlockStmtOrExpr<'a>,
        is_async: bool,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::Arrow(
            self.box_arrow_expr(span, params, body, is_async),
        )))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Class(self.box_class_expr(ident, class))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_yield_expr(
        &self,
        span: Span,
        arg: Option<Expr<'a>>,
        delegate: bool,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Yield(self.box_yield_expr(span, arg, delegate))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_meta_prop_expr(
        &self,
        span: Span,
        kind: MetaPropKind,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::MetaProp(self.box_meta_prop_expr(span, kind))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_await_expr(
        &self,
        span: Span,
        arg: Expr<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Await(self.box_await_expr(span, arg))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_paren_expr(
        &self,
        span: Span,
        expr: Expr<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::Paren(self.box_paren_expr(span, expr))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::JSXMember(self.box_jsx_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::JSXNamespacedName(
            self.box_jsx_namespaced_name(span, ns, name),
        )))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_empty_expr(&self, span: Span) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::JSXEmpty(self.box_jsx_empty_expr(span))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::JSXElement(
            self.box_jsx_element(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::JSXFragment(
            self.box_jsx_fragment(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_private_name(
        &self,
        span: Span,
        name: Atom<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(
            self.allocator
                .boxed(Expr::PrivateName(self.box_private_name(span, name))),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::OptChain(
            self.box_opt_chain_expr(span, optional, base),
        )))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_invalid(&self) -> BlockStmtOrExpr<'a> {
        BlockStmtOrExpr::Expr(self.allocator.boxed(Expr::Invalid(self.box_invalid())))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_binding_ident(
        &self,
        id: Box<'a, Ident<'a>>,
    ) -> AssignTarget<'a> {
        AssignTarget::Simple(
            self.allocator
                .boxed(SimpleAssignTarget::Ident(self.box_binding_ident(id))),
        )
    }
    #[inline]
    pub fn assign_target_simple_assign_target_member_expr(
        &self,
        span: Span,
        obj: Expr<'a>,
        prop: MemberProp<'a>,
    ) -> AssignTarget<'a> {
        AssignTarget::Simple(self.allocator.boxed(SimpleAssignTarget::Member(
            self.box_member_expr(span, obj, prop),
        )))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> AssignTarget<'a> {
        AssignTarget::Simple(self.allocator.boxed(SimpleAssignTarget::SuperProp(
            self.box_super_prop_expr(span, obj, prop),
        )))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_paren_expr(
        &self,
        span: Span,
        expr: Expr<'a>,
    ) -> AssignTarget<'a> {
        AssignTarget::Simple(
            self.allocator
                .boxed(SimpleAssignTarget::Paren(self.box_paren_expr(span, expr))),
        )
    }
    #[inline]
    pub fn assign_target_simple_assign_target_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> AssignTarget<'a> {
        AssignTarget::Simple(self.allocator.boxed(SimpleAssignTarget::OptChain(
            self.box_opt_chain_expr(span, optional, base),
        )))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_invalid(&self) -> AssignTarget<'a> {
        AssignTarget::Simple(
            self.allocator
                .boxed(SimpleAssignTarget::Invalid(self.box_invalid())),
        )
    }
    #[inline]
    pub fn assign_target_assign_target_pat_array_pat(
        &self,
        span: Span,
        elems: Vec<'a, Option<Pat<'a>>>,
        optional: bool,
    ) -> AssignTarget<'a> {
        AssignTarget::Pat(self.allocator.boxed(AssignTargetPat::Array(
            self.box_array_pat(span, elems, optional),
        )))
    }
    #[inline]
    pub fn assign_target_assign_target_pat_object_pat(
        &self,
        span: Span,
        props: Vec<'a, ObjectPatProp<'a>>,
        optional: bool,
    ) -> AssignTarget<'a> {
        AssignTarget::Pat(self.allocator.boxed(AssignTargetPat::Object(
            self.box_object_pat(span, props, optional),
        )))
    }
    #[inline]
    pub fn assign_target_assign_target_pat_invalid(&self) -> AssignTarget<'a> {
        AssignTarget::Pat(
            self.allocator
                .boxed(AssignTargetPat::Invalid(self.box_invalid())),
        )
    }
    #[inline]
    pub fn assign_target_pat_array_pat(
        &self,
        span: Span,
        elems: Vec<'a, Option<Pat<'a>>>,
        optional: bool,
    ) -> AssignTargetPat<'a> {
        AssignTargetPat::Array(self.box_array_pat(span, elems, optional))
    }
    #[inline]
    pub fn assign_target_pat_object_pat(
        &self,
        span: Span,
        props: Vec<'a, ObjectPatProp<'a>>,
        optional: bool,
    ) -> AssignTargetPat<'a> {
        AssignTargetPat::Object(self.box_object_pat(span, props, optional))
    }
    #[inline]
    pub fn assign_target_pat_invalid(&self) -> AssignTargetPat<'a> {
        AssignTargetPat::Invalid(self.box_invalid())
    }
    #[inline]
    pub fn simple_assign_target_binding_ident(
        &self,
        id: Box<'a, Ident<'a>>,
    ) -> SimpleAssignTarget<'a> {
        SimpleAssignTarget::Ident(self.box_binding_ident(id))
    }
    #[inline]
    pub fn simple_assign_target_member_expr(
        &self,
        span: Span,
        obj: Expr<'a>,
        prop: MemberProp<'a>,
    ) -> SimpleAssignTarget<'a> {
        SimpleAssignTarget::Member(self.box_member_expr(span, obj, prop))
    }
    #[inline]
    pub fn simple_assign_target_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> SimpleAssignTarget<'a> {
        SimpleAssignTarget::SuperProp(self.box_super_prop_expr(span, obj, prop))
    }
    #[inline]
    pub fn simple_assign_target_paren_expr(
        &self,
        span: Span,
        expr: Expr<'a>,
    ) -> SimpleAssignTarget<'a> {
        SimpleAssignTarget::Paren(self.box_paren_expr(span, expr))
    }
    #[inline]
    pub fn simple_assign_target_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> SimpleAssignTarget<'a> {
        SimpleAssignTarget::OptChain(self.box_opt_chain_expr(span, optional, base))
    }
    #[inline]
    pub fn simple_assign_target_invalid(&self) -> SimpleAssignTarget<'a> {
        SimpleAssignTarget::Invalid(self.box_invalid())
    }
    #[inline]
    pub fn opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> OptChainExpr<'a> {
        OptChainExpr {
            span,
            optional,
            base,
        }
    }
    #[inline]
    pub fn box_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> Box<'a, OptChainExpr<'a>> {
        self.allocator
            .boxed(self.opt_chain_expr(span, optional, base))
    }
    #[inline]
    pub fn opt_chain_base_member_expr(
        &self,
        span: Span,
        obj: Expr<'a>,
        prop: MemberProp<'a>,
    ) -> OptChainBase<'a> {
        OptChainBase::Member(self.box_member_expr(span, obj, prop))
    }
    #[inline]
    pub fn opt_chain_base_opt_call(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> OptChainBase<'a> {
        OptChainBase::Call(self.box_opt_call(span, callee, args))
    }
    #[inline]
    pub fn opt_call(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> OptCall<'a> {
        OptCall { span, callee, args }
    }
    #[inline]
    pub fn box_opt_call(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> Box<'a, OptCall<'a>> {
        self.allocator.boxed(self.opt_call(span, callee, args))
    }
    #[inline]
    pub fn invalid(&self) -> Invalid {
        Invalid {}
    }
    #[inline]
    pub fn box_invalid(&self) -> Box<'a, Invalid> {
        self.allocator.boxed(self.invalid())
    }
    #[inline]
    pub fn function(
        &self,
        span: Span,
        params: Vec<'a, Param<'a>>,
        decorators: Vec<'a, Decorator<'a>>,
        body: Box<'a, BlockStmt<'a>>,
        is_generator: bool,
        is_async: bool,
    ) -> Function<'a> {
        Function {
            span,
            params,
            decorators,
            body,
            is_generator,
            is_async,
        }
    }
    #[inline]
    pub fn box_function(
        &self,
        span: Span,
        params: Vec<'a, Param<'a>>,
        decorators: Vec<'a, Decorator<'a>>,
        body: Box<'a, BlockStmt<'a>>,
        is_generator: bool,
        is_async: bool,
    ) -> Box<'a, Function<'a>> {
        self.allocator
            .boxed(self.function(span, params, decorators, body, is_generator, is_async))
    }
    #[inline]
    pub fn param(&self, span: Span, decorators: Vec<'a, Decorator<'a>>, pat: Pat<'a>) -> Param<'a> {
        Param {
            span,
            decorators,
            pat,
        }
    }
    #[inline]
    pub fn box_param(
        &self,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        pat: Pat<'a>,
    ) -> Box<'a, Param<'a>> {
        self.allocator.boxed(self.param(span, decorators, pat))
    }
    #[inline]
    pub fn param_or_ts_param_prop_param(
        &self,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        pat: Pat<'a>,
    ) -> ParamOrTsParamProp<'a> {
        ParamOrTsParamProp::Param(self.box_param(span, decorators, pat))
    }
    #[inline]
    pub fn class(
        &self,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        body: Vec<'a, ClassMember<'a>>,
        super_class: Option<Expr<'a>>,
        is_abstract: bool,
    ) -> Class<'a> {
        Class {
            span,
            decorators,
            body,
            super_class,
            is_abstract,
        }
    }
    #[inline]
    pub fn box_class(
        &self,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        body: Vec<'a, ClassMember<'a>>,
        super_class: Option<Expr<'a>>,
        is_abstract: bool,
    ) -> Box<'a, Class<'a>> {
        self.allocator
            .boxed(self.class(span, decorators, body, super_class, is_abstract))
    }
    #[inline]
    pub fn class_member_constructor(
        &self,
        span: Span,
        key: PropName<'a>,
        params: Vec<'a, ParamOrTsParamProp<'a>>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> ClassMember<'a> {
        ClassMember::Constructor(self.box_constructor(span, key, params, body))
    }
    #[inline]
    pub fn class_member_class_method(
        &self,
        span: Span,
        key: PropName<'a>,
        function: Box<'a, Function<'a>>,
        kind: MethodKind,
        is_static: bool,
    ) -> ClassMember<'a> {
        ClassMember::Method(self.box_class_method(span, key, function, kind, is_static))
    }
    #[inline]
    pub fn class_member_private_method(
        &self,
        span: Span,
        key: Box<'a, PrivateName<'a>>,
        function: Box<'a, Function<'a>>,
        kind: MethodKind,
        is_static: bool,
    ) -> ClassMember<'a> {
        ClassMember::PrivateMethod(self.box_private_method(span, key, function, kind, is_static))
    }
    #[inline]
    pub fn class_member_class_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        value: Option<Expr<'a>>,
        is_static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassMember<'a> {
        ClassMember::ClassProp(self.box_class_prop(span, key, value, is_static, decorators))
    }
    #[inline]
    pub fn class_member_private_prop(
        &self,
        span: Span,
        key: Box<'a, PrivateName<'a>>,
        value: Option<Expr<'a>>,
        is_static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassMember<'a> {
        ClassMember::PrivateProp(self.box_private_prop(span, key, value, is_static, decorators))
    }
    #[inline]
    pub fn class_member_empty_stmt(&self, span: Span) -> ClassMember<'a> {
        ClassMember::Empty(self.box_empty_stmt(span))
    }
    #[inline]
    pub fn class_member_static_block(
        &self,
        span: Span,
        body: Box<'a, BlockStmt<'a>>,
    ) -> ClassMember<'a> {
        ClassMember::StaticBlock(self.box_static_block(span, body))
    }
    #[inline]
    pub fn class_member_auto_accessor(
        &self,
        span: Span,
        key: Key<'a>,
        value: Option<Expr<'a>>,
        is_static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassMember<'a> {
        ClassMember::AutoAccessor(self.box_auto_accessor(span, key, value, is_static, decorators))
    }
    #[inline]
    pub fn class_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        value: Option<Expr<'a>>,
        is_static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassProp<'a> {
        ClassProp {
            span,
            key,
            value,
            is_static,
            decorators,
        }
    }
    #[inline]
    pub fn box_class_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        value: Option<Expr<'a>>,
        is_static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, ClassProp<'a>> {
        self.allocator
            .boxed(self.class_prop(span, key, value, is_static, decorators))
    }
    #[inline]
    pub fn private_prop(
        &self,
        span: Span,
        key: Box<'a, PrivateName<'a>>,
        value: Option<Expr<'a>>,
        is_static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> PrivateProp<'a> {
        PrivateProp {
            span,
            key,
            value,
            is_static,
            decorators,
        }
    }
    #[inline]
    pub fn box_private_prop(
        &self,
        span: Span,
        key: Box<'a, PrivateName<'a>>,
        value: Option<Expr<'a>>,
        is_static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, PrivateProp<'a>> {
        self.allocator
            .boxed(self.private_prop(span, key, value, is_static, decorators))
    }
    #[inline]
    pub fn class_method(
        &self,
        span: Span,
        key: PropName<'a>,
        function: Box<'a, Function<'a>>,
        kind: MethodKind,
        is_static: bool,
    ) -> ClassMethod<'a> {
        ClassMethod {
            span,
            key,
            function,
            kind,
            is_static,
        }
    }
    #[inline]
    pub fn box_class_method(
        &self,
        span: Span,
        key: PropName<'a>,
        function: Box<'a, Function<'a>>,
        kind: MethodKind,
        is_static: bool,
    ) -> Box<'a, ClassMethod<'a>> {
        self.allocator
            .boxed(self.class_method(span, key, function, kind, is_static))
    }
    #[inline]
    pub fn private_method(
        &self,
        span: Span,
        key: Box<'a, PrivateName<'a>>,
        function: Box<'a, Function<'a>>,
        kind: MethodKind,
        is_static: bool,
    ) -> PrivateMethod<'a> {
        PrivateMethod {
            span,
            key,
            function,
            kind,
            is_static,
        }
    }
    #[inline]
    pub fn box_private_method(
        &self,
        span: Span,
        key: Box<'a, PrivateName<'a>>,
        function: Box<'a, Function<'a>>,
        kind: MethodKind,
        is_static: bool,
    ) -> Box<'a, PrivateMethod<'a>> {
        self.allocator
            .boxed(self.private_method(span, key, function, kind, is_static))
    }
    #[inline]
    pub fn constructor(
        &self,
        span: Span,
        key: PropName<'a>,
        params: Vec<'a, ParamOrTsParamProp<'a>>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> Constructor<'a> {
        Constructor {
            span,
            key,
            params,
            body,
        }
    }
    #[inline]
    pub fn box_constructor(
        &self,
        span: Span,
        key: PropName<'a>,
        params: Vec<'a, ParamOrTsParamProp<'a>>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> Box<'a, Constructor<'a>> {
        self.allocator
            .boxed(self.constructor(span, key, params, body))
    }
    #[inline]
    pub fn decorator(&self, span: Span, expr: Expr<'a>) -> Decorator<'a> {
        Decorator { span, expr }
    }
    #[inline]
    pub fn box_decorator(&self, span: Span, expr: Expr<'a>) -> Box<'a, Decorator<'a>> {
        self.allocator.boxed(self.decorator(span, expr))
    }
    #[inline]
    pub fn static_block(&self, span: Span, body: Box<'a, BlockStmt<'a>>) -> StaticBlock<'a> {
        StaticBlock { span, body }
    }
    #[inline]
    pub fn box_static_block(
        &self,
        span: Span,
        body: Box<'a, BlockStmt<'a>>,
    ) -> Box<'a, StaticBlock<'a>> {
        self.allocator.boxed(self.static_block(span, body))
    }
    #[inline]
    pub fn key_private_name(&self, span: Span, name: Atom<'a>) -> Key<'a> {
        Key::Private(self.box_private_name(span, name))
    }
    #[inline]
    pub fn key_prop_name_ident_name(&self, span: Span, sym: Atom<'a>) -> Key<'a> {
        Key::Public(
            self.allocator
                .boxed(PropName::Ident(self.box_ident_name(span, sym))),
        )
    }
    #[inline]
    pub fn key_prop_name_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Key<'a> {
        Key::Public(
            self.allocator
                .boxed(PropName::Str(self.box_str(span, value, raw))),
        )
    }
    #[inline]
    pub fn key_prop_name_number(&self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Key<'a> {
        Key::Public(
            self.allocator
                .boxed(PropName::Num(self.box_number(span, value, raw))),
        )
    }
    #[inline]
    pub fn key_prop_name_computed_prop_name(&self, span: Span, expr: Expr<'a>) -> Key<'a> {
        Key::Public(
            self.allocator
                .boxed(PropName::Computed(self.box_computed_prop_name(span, expr))),
        )
    }
    #[inline]
    pub fn key_prop_name_big_int(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Key<'a> {
        Key::Public(
            self.allocator
                .boxed(PropName::BigInt(self.box_big_int(span, value, raw))),
        )
    }
    #[inline]
    pub fn auto_accessor(
        &self,
        span: Span,
        key: Key<'a>,
        value: Option<Expr<'a>>,
        is_static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> AutoAccessor<'a> {
        AutoAccessor {
            span,
            key,
            value,
            is_static,
            decorators,
        }
    }
    #[inline]
    pub fn box_auto_accessor(
        &self,
        span: Span,
        key: Key<'a>,
        value: Option<Expr<'a>>,
        is_static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, AutoAccessor<'a>> {
        self.allocator
            .boxed(self.auto_accessor(span, key, value, is_static, decorators))
    }
    #[inline]
    pub fn prop_ident(&self, span: Span, sym: Atom<'a>) -> Prop<'a> {
        Prop::Shorthand(self.box_ident(span, sym))
    }
    #[inline]
    pub fn prop_key_value_prop(&self, key: PropName<'a>, value: Expr<'a>) -> Prop<'a> {
        Prop::KeyValue(self.box_key_value_prop(key, value))
    }
    #[inline]
    pub fn prop_assign_prop(
        &self,
        span: Span,
        key: Box<'a, Ident<'a>>,
        value: Expr<'a>,
    ) -> Prop<'a> {
        Prop::Assign(self.box_assign_prop(span, key, value))
    }
    #[inline]
    pub fn prop_getter_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> Prop<'a> {
        Prop::Getter(self.box_getter_prop(span, key, body))
    }
    #[inline]
    pub fn prop_setter_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        this_param: Option<Pat<'a>>,
        param: Pat<'a>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> Prop<'a> {
        Prop::Setter(self.box_setter_prop(span, key, this_param, param, body))
    }
    #[inline]
    pub fn prop_method_prop(&self, key: PropName<'a>, function: Box<'a, Function<'a>>) -> Prop<'a> {
        Prop::Method(self.box_method_prop(key, function))
    }
    #[inline]
    pub fn key_value_prop(&self, key: PropName<'a>, value: Expr<'a>) -> KeyValueProp<'a> {
        KeyValueProp { key, value }
    }
    #[inline]
    pub fn box_key_value_prop(
        &self,
        key: PropName<'a>,
        value: Expr<'a>,
    ) -> Box<'a, KeyValueProp<'a>> {
        self.allocator.boxed(self.key_value_prop(key, value))
    }
    #[inline]
    pub fn assign_prop(
        &self,
        span: Span,
        key: Box<'a, Ident<'a>>,
        value: Expr<'a>,
    ) -> AssignProp<'a> {
        AssignProp { span, key, value }
    }
    #[inline]
    pub fn box_assign_prop(
        &self,
        span: Span,
        key: Box<'a, Ident<'a>>,
        value: Expr<'a>,
    ) -> Box<'a, AssignProp<'a>> {
        self.allocator.boxed(self.assign_prop(span, key, value))
    }
    #[inline]
    pub fn getter_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> GetterProp<'a> {
        GetterProp { span, key, body }
    }
    #[inline]
    pub fn box_getter_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> Box<'a, GetterProp<'a>> {
        self.allocator.boxed(self.getter_prop(span, key, body))
    }
    #[inline]
    pub fn setter_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        this_param: Option<Pat<'a>>,
        param: Pat<'a>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> SetterProp<'a> {
        SetterProp {
            span,
            key,
            this_param,
            param,
            body,
        }
    }
    #[inline]
    pub fn box_setter_prop(
        &self,
        span: Span,
        key: PropName<'a>,
        this_param: Option<Pat<'a>>,
        param: Pat<'a>,
        body: Option<Box<'a, BlockStmt<'a>>>,
    ) -> Box<'a, SetterProp<'a>> {
        self.allocator
            .boxed(self.setter_prop(span, key, this_param, param, body))
    }
    #[inline]
    pub fn method_prop(
        &self,
        key: PropName<'a>,
        function: Box<'a, Function<'a>>,
    ) -> MethodProp<'a> {
        MethodProp { key, function }
    }
    #[inline]
    pub fn box_method_prop(
        &self,
        key: PropName<'a>,
        function: Box<'a, Function<'a>>,
    ) -> Box<'a, MethodProp<'a>> {
        self.allocator.boxed(self.method_prop(key, function))
    }
    #[inline]
    pub fn prop_name_ident_name(&self, span: Span, sym: Atom<'a>) -> PropName<'a> {
        PropName::Ident(self.box_ident_name(span, sym))
    }
    #[inline]
    pub fn prop_name_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> PropName<'a> {
        PropName::Str(self.box_str(span, value, raw))
    }
    #[inline]
    pub fn prop_name_number(&self, span: Span, value: f64, raw: Option<Atom<'a>>) -> PropName<'a> {
        PropName::Num(self.box_number(span, value, raw))
    }
    #[inline]
    pub fn prop_name_computed_prop_name(&self, span: Span, expr: Expr<'a>) -> PropName<'a> {
        PropName::Computed(self.box_computed_prop_name(span, expr))
    }
    #[inline]
    pub fn prop_name_big_int(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> PropName<'a> {
        PropName::BigInt(self.box_big_int(span, value, raw))
    }
    #[inline]
    pub fn computed_prop_name(&self, span: Span, expr: Expr<'a>) -> ComputedPropName<'a> {
        ComputedPropName { span, expr }
    }
    #[inline]
    pub fn box_computed_prop_name(
        &self,
        span: Span,
        expr: Expr<'a>,
    ) -> Box<'a, ComputedPropName<'a>> {
        self.allocator.boxed(self.computed_prop_name(span, expr))
    }
    #[inline]
    pub fn pat_binding_ident(&self, id: Box<'a, Ident<'a>>) -> Pat<'a> {
        Pat::Ident(self.box_binding_ident(id))
    }
    #[inline]
    pub fn pat_array_pat(
        &self,
        span: Span,
        elems: Vec<'a, Option<Pat<'a>>>,
        optional: bool,
    ) -> Pat<'a> {
        Pat::Array(self.box_array_pat(span, elems, optional))
    }
    #[inline]
    pub fn pat_rest_pat(&self, span: Span, dot3_token: Span, arg: Pat<'a>) -> Pat<'a> {
        Pat::Rest(self.box_rest_pat(span, dot3_token, arg))
    }
    #[inline]
    pub fn pat_object_pat(
        &self,
        span: Span,
        props: Vec<'a, ObjectPatProp<'a>>,
        optional: bool,
    ) -> Pat<'a> {
        Pat::Object(self.box_object_pat(span, props, optional))
    }
    #[inline]
    pub fn pat_assign_pat(&self, span: Span, left: Pat<'a>, right: Expr<'a>) -> Pat<'a> {
        Pat::Assign(self.box_assign_pat(span, left, right))
    }
    #[inline]
    pub fn pat_invalid(&self) -> Pat<'a> {
        Pat::Invalid(self.box_invalid())
    }
    #[inline]
    pub fn pat_expr_this_expr(&self, span: Span) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::This(self.box_this_expr(span))))
    }
    #[inline]
    pub fn pat_expr_array_lit(
        &self,
        span: Span,
        elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Array(self.box_array_lit(span, elems))),
        )
    }
    #[inline]
    pub fn pat_expr_object_lit(&self, span: Span, props: Vec<'a, PropOrSpread<'a>>) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Object(self.box_object_lit(span, props))),
        )
    }
    #[inline]
    pub fn pat_expr_fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Fn(self.box_fn_expr(ident, function))),
        )
    }
    #[inline]
    pub fn pat_expr_unary_expr(&self, span: Span, op: UnaryOp, arg: Expr<'a>) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Unary(self.box_unary_expr(span, op, arg))),
        )
    }
    #[inline]
    pub fn pat_expr_update_expr(
        &self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: Expr<'a>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Update(self.box_update_expr(span, op, prefix, arg))),
        )
    }
    #[inline]
    pub fn pat_expr_bin_expr(
        &self,
        span: Span,
        op: BinaryOp,
        left: Expr<'a>,
        right: Expr<'a>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Bin(self.box_bin_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn pat_expr_assign_expr(
        &self,
        span: Span,
        op: AssignOp,
        left: AssignTarget<'a>,
        right: Expr<'a>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Assign(self.box_assign_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn pat_expr_member_expr(&self, span: Span, obj: Expr<'a>, prop: MemberProp<'a>) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Member(self.box_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn pat_expr_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::SuperProp(self.box_super_prop_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn pat_expr_cond_expr(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Expr<'a>,
        alt: Expr<'a>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Cond(self.box_cond_expr(span, test, cons, alt))),
        )
    }
    #[inline]
    pub fn pat_expr_call_expr(
        &self,
        span: Span,
        callee: Callee<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Call(self.box_call_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn pat_expr_new_expr(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Option<Vec<'a, ExprOrSpread<'a>>>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::New(self.box_new_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn pat_expr_seq_expr(&self, span: Span, exprs: Vec<'a, Expr<'a>>) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Seq(self.box_seq_expr(span, exprs))),
        )
    }
    #[inline]
    pub fn pat_expr_ident(&self, span: Span, sym: Atom<'a>) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::Ident(self.box_ident(span, sym))))
    }
    #[inline]
    pub fn pat_expr_lit_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Str(self.box_str(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn pat_expr_lit_bool(&self, span: Span, value: bool) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Bool(self.box_bool(span, value))),
        )))
    }
    #[inline]
    pub fn pat_expr_lit_null(&self, span: Span) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Null(self.box_null(span))),
        )))
    }
    #[inline]
    pub fn pat_expr_lit_number(&self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Pat<'a> {
        Pat::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Num(self.box_number(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn pat_expr_lit_big_int(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::BigInt(self.box_big_int(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn pat_expr_lit_regex(&self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Pat<'a> {
        Pat::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Regex(self.box_regex(span, exp, flags))),
            )),
        )
    }
    #[inline]
    pub fn pat_expr_tpl(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
        quasis: Vec<'a, TplElement<'a>>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Tpl(self.box_tpl(span, exprs, quasis))),
        )
    }
    #[inline]
    pub fn pat_expr_tagged_tpl(&self, span: Span, tag: Expr<'a>, tpl: Box<'a, Tpl<'a>>) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::TaggedTpl(self.box_tagged_tpl(span, tag, tpl))),
        )
    }
    #[inline]
    pub fn pat_expr_arrow_expr(
        &self,
        span: Span,
        params: Vec<'a, Pat<'a>>,
        body: BlockStmtOrExpr<'a>,
        is_async: bool,
    ) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::Arrow(
            self.box_arrow_expr(span, params, body, is_async),
        )))
    }
    #[inline]
    pub fn pat_expr_class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Class(self.box_class_expr(ident, class))),
        )
    }
    #[inline]
    pub fn pat_expr_yield_expr(
        &self,
        span: Span,
        arg: Option<Expr<'a>>,
        delegate: bool,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Yield(self.box_yield_expr(span, arg, delegate))),
        )
    }
    #[inline]
    pub fn pat_expr_meta_prop_expr(&self, span: Span, kind: MetaPropKind) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::MetaProp(self.box_meta_prop_expr(span, kind))),
        )
    }
    #[inline]
    pub fn pat_expr_await_expr(&self, span: Span, arg: Expr<'a>) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Await(self.box_await_expr(span, arg))),
        )
    }
    #[inline]
    pub fn pat_expr_paren_expr(&self, span: Span, expr: Expr<'a>) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::Paren(self.box_paren_expr(span, expr))),
        )
    }
    #[inline]
    pub fn pat_expr_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::JSXMember(self.box_jsx_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn pat_expr_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::JSXNamespacedName(
            self.box_jsx_namespaced_name(span, ns, name),
        )))
    }
    #[inline]
    pub fn pat_expr_jsx_empty_expr(&self, span: Span) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::JSXEmpty(self.box_jsx_empty_expr(span))),
        )
    }
    #[inline]
    pub fn pat_expr_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::JSXElement(
            self.box_jsx_element(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn pat_expr_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::JSXFragment(
            self.box_jsx_fragment(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn pat_expr_private_name(&self, span: Span, name: Atom<'a>) -> Pat<'a> {
        Pat::Expr(
            self.allocator
                .boxed(Expr::PrivateName(self.box_private_name(span, name))),
        )
    }
    #[inline]
    pub fn pat_expr_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::OptChain(
            self.box_opt_chain_expr(span, optional, base),
        )))
    }
    #[inline]
    pub fn pat_expr_invalid(&self) -> Pat<'a> {
        Pat::Expr(self.allocator.boxed(Expr::Invalid(self.box_invalid())))
    }
    #[inline]
    pub fn array_pat(
        &self,
        span: Span,
        elems: Vec<'a, Option<Pat<'a>>>,
        optional: bool,
    ) -> ArrayPat<'a> {
        ArrayPat {
            span,
            elems,
            optional,
        }
    }
    #[inline]
    pub fn box_array_pat(
        &self,
        span: Span,
        elems: Vec<'a, Option<Pat<'a>>>,
        optional: bool,
    ) -> Box<'a, ArrayPat<'a>> {
        self.allocator.boxed(self.array_pat(span, elems, optional))
    }
    #[inline]
    pub fn object_pat(
        &self,
        span: Span,
        props: Vec<'a, ObjectPatProp<'a>>,
        optional: bool,
    ) -> ObjectPat<'a> {
        ObjectPat {
            span,
            props,
            optional,
        }
    }
    #[inline]
    pub fn box_object_pat(
        &self,
        span: Span,
        props: Vec<'a, ObjectPatProp<'a>>,
        optional: bool,
    ) -> Box<'a, ObjectPat<'a>> {
        self.allocator.boxed(self.object_pat(span, props, optional))
    }
    #[inline]
    pub fn assign_pat(&self, span: Span, left: Pat<'a>, right: Expr<'a>) -> AssignPat<'a> {
        AssignPat { span, left, right }
    }
    #[inline]
    pub fn box_assign_pat(
        &self,
        span: Span,
        left: Pat<'a>,
        right: Expr<'a>,
    ) -> Box<'a, AssignPat<'a>> {
        self.allocator.boxed(self.assign_pat(span, left, right))
    }
    #[inline]
    pub fn rest_pat(&self, span: Span, dot3_token: Span, arg: Pat<'a>) -> RestPat<'a> {
        RestPat {
            span,
            dot3_token,
            arg,
        }
    }
    #[inline]
    pub fn box_rest_pat(&self, span: Span, dot3_token: Span, arg: Pat<'a>) -> Box<'a, RestPat<'a>> {
        self.allocator.boxed(self.rest_pat(span, dot3_token, arg))
    }
    #[inline]
    pub fn object_pat_prop_key_value_pat_prop(
        &self,
        key: PropName<'a>,
        value: Pat<'a>,
    ) -> ObjectPatProp<'a> {
        ObjectPatProp::KeyValue(self.box_key_value_pat_prop(key, value))
    }
    #[inline]
    pub fn object_pat_prop_assign_pat_prop(
        &self,
        span: Span,
        key: Box<'a, BindingIdent<'a>>,
        value: Option<Expr<'a>>,
    ) -> ObjectPatProp<'a> {
        ObjectPatProp::Assign(self.box_assign_pat_prop(span, key, value))
    }
    #[inline]
    pub fn object_pat_prop_rest_pat(
        &self,
        span: Span,
        dot3_token: Span,
        arg: Pat<'a>,
    ) -> ObjectPatProp<'a> {
        ObjectPatProp::Rest(self.box_rest_pat(span, dot3_token, arg))
    }
    #[inline]
    pub fn key_value_pat_prop(&self, key: PropName<'a>, value: Pat<'a>) -> KeyValuePatProp<'a> {
        KeyValuePatProp { key, value }
    }
    #[inline]
    pub fn box_key_value_pat_prop(
        &self,
        key: PropName<'a>,
        value: Pat<'a>,
    ) -> Box<'a, KeyValuePatProp<'a>> {
        self.allocator.boxed(self.key_value_pat_prop(key, value))
    }
    #[inline]
    pub fn assign_pat_prop(
        &self,
        span: Span,
        key: Box<'a, BindingIdent<'a>>,
        value: Option<Expr<'a>>,
    ) -> AssignPatProp<'a> {
        AssignPatProp { span, key, value }
    }
    #[inline]
    pub fn box_assign_pat_prop(
        &self,
        span: Span,
        key: Box<'a, BindingIdent<'a>>,
        value: Option<Expr<'a>>,
    ) -> Box<'a, AssignPatProp<'a>> {
        self.allocator.boxed(self.assign_pat_prop(span, key, value))
    }
    #[inline]
    pub fn ident(&self, span: Span, sym: Atom<'a>) -> Ident<'a> {
        Ident {
            span,
            sym,
            symbol_id: Default::default(),
        }
    }
    #[inline]
    pub fn box_ident(&self, span: Span, sym: Atom<'a>) -> Box<'a, Ident<'a>> {
        self.allocator.boxed(self.ident(span, sym))
    }
    #[inline]
    pub fn ident_name(&self, span: Span, sym: Atom<'a>) -> IdentName<'a> {
        IdentName { span, sym }
    }
    #[inline]
    pub fn box_ident_name(&self, span: Span, sym: Atom<'a>) -> Box<'a, IdentName<'a>> {
        self.allocator.boxed(self.ident_name(span, sym))
    }
    #[inline]
    pub fn private_name(&self, span: Span, name: Atom<'a>) -> PrivateName<'a> {
        PrivateName { span, name }
    }
    #[inline]
    pub fn box_private_name(&self, span: Span, name: Atom<'a>) -> Box<'a, PrivateName<'a>> {
        self.allocator.boxed(self.private_name(span, name))
    }
    #[inline]
    pub fn binding_ident(&self, id: Box<'a, Ident<'a>>) -> BindingIdent<'a> {
        BindingIdent { id }
    }
    #[inline]
    pub fn box_binding_ident(&self, id: Box<'a, Ident<'a>>) -> Box<'a, BindingIdent<'a>> {
        self.allocator.boxed(self.binding_ident(id))
    }
    #[inline]
    pub fn lit_str(&self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Lit<'a> {
        Lit::Str(self.box_str(span, value, raw))
    }
    #[inline]
    pub fn lit_bool(&self, span: Span, value: bool) -> Lit<'a> {
        Lit::Bool(self.box_bool(span, value))
    }
    #[inline]
    pub fn lit_null(&self, span: Span) -> Lit<'a> {
        Lit::Null(self.box_null(span))
    }
    #[inline]
    pub fn lit_number(&self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Lit<'a> {
        Lit::Num(self.box_number(span, value, raw))
    }
    #[inline]
    pub fn lit_big_int(&self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> Lit<'a> {
        Lit::BigInt(self.box_big_int(span, value, raw))
    }
    #[inline]
    pub fn lit_regex(&self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Lit<'a> {
        Lit::Regex(self.box_regex(span, exp, flags))
    }
    #[inline]
    pub fn str(&self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Str<'a> {
        Str { span, value, raw }
    }
    #[inline]
    pub fn box_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Box<'a, Str<'a>> {
        self.allocator.boxed(self.str(span, value, raw))
    }
    #[inline]
    pub fn bool(&self, span: Span, value: bool) -> Bool {
        Bool { span, value }
    }
    #[inline]
    pub fn box_bool(&self, span: Span, value: bool) -> Box<'a, Bool> {
        self.allocator.boxed(self.bool(span, value))
    }
    #[inline]
    pub fn null(&self, span: Span) -> Null {
        Null { span }
    }
    #[inline]
    pub fn box_null(&self, span: Span) -> Box<'a, Null> {
        self.allocator.boxed(self.null(span))
    }
    #[inline]
    pub fn number(&self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Number<'a> {
        Number { span, value, raw }
    }
    #[inline]
    pub fn box_number(&self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Box<'a, Number<'a>> {
        self.allocator.boxed(self.number(span, value, raw))
    }
    #[inline]
    pub fn big_int(&self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> BigInt<'a> {
        BigInt { span, value, raw }
    }
    #[inline]
    pub fn box_big_int(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Box<'a, BigInt<'a>> {
        self.allocator.boxed(self.big_int(span, value, raw))
    }
    #[inline]
    pub fn regex(&self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Regex<'a> {
        Regex { span, exp, flags }
    }
    #[inline]
    pub fn box_regex(&self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Box<'a, Regex<'a>> {
        self.allocator.boxed(self.regex(span, exp, flags))
    }
    #[inline]
    pub fn jsx_object_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> JSXObject<'a> {
        JSXObject::JSXMemberExpr(self.box_jsx_member_expr(span, obj, prop))
    }
    #[inline]
    pub fn jsx_object_ident(&self, span: Span, sym: Atom<'a>) -> JSXObject<'a> {
        JSXObject::Ident(self.box_ident(span, sym))
    }
    #[inline]
    pub fn jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> JSXMemberExpr<'a> {
        JSXMemberExpr { span, obj, prop }
    }
    #[inline]
    pub fn box_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> Box<'a, JSXMemberExpr<'a>> {
        self.allocator.boxed(self.jsx_member_expr(span, obj, prop))
    }
    #[inline]
    pub fn jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> JSXNamespacedName<'a> {
        JSXNamespacedName { span, ns, name }
    }
    #[inline]
    pub fn box_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> Box<'a, JSXNamespacedName<'a>> {
        self.allocator
            .boxed(self.jsx_namespaced_name(span, ns, name))
    }
    #[inline]
    pub fn jsx_empty_expr(&self, span: Span) -> JSXEmptyExpr {
        JSXEmptyExpr { span }
    }
    #[inline]
    pub fn box_jsx_empty_expr(&self, span: Span) -> Box<'a, JSXEmptyExpr> {
        self.allocator.boxed(self.jsx_empty_expr(span))
    }
    #[inline]
    pub fn jsx_expr_container(&self, span: Span, expr: JSXExpr<'a>) -> JSXExprContainer<'a> {
        JSXExprContainer { span, expr }
    }
    #[inline]
    pub fn box_jsx_expr_container(
        &self,
        span: Span,
        expr: JSXExpr<'a>,
    ) -> Box<'a, JSXExprContainer<'a>> {
        self.allocator.boxed(self.jsx_expr_container(span, expr))
    }
    #[inline]
    pub fn jsx_expr_jsx_empty_expr(&self, span: Span) -> JSXExpr<'a> {
        JSXExpr::JSXEmptyExpr(self.box_jsx_empty_expr(span))
    }
    #[inline]
    pub fn jsx_expr_expr_this_expr(&self, span: Span) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::This(self.box_this_expr(span))))
    }
    #[inline]
    pub fn jsx_expr_expr_array_lit(
        &self,
        span: Span,
        elems: Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Array(self.box_array_lit(span, elems))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_object_lit(
        &self,
        span: Span,
        props: Vec<'a, PropOrSpread<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Object(self.box_object_lit(span, props))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_fn_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        function: Box<'a, Function<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Fn(self.box_fn_expr(ident, function))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_unary_expr(&self, span: Span, op: UnaryOp, arg: Expr<'a>) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Unary(self.box_unary_expr(span, op, arg))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_update_expr(
        &self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: Expr<'a>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Update(self.box_update_expr(span, op, prefix, arg))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_bin_expr(
        &self,
        span: Span,
        op: BinaryOp,
        left: Expr<'a>,
        right: Expr<'a>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Bin(self.box_bin_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_assign_expr(
        &self,
        span: Span,
        op: AssignOp,
        left: AssignTarget<'a>,
        right: Expr<'a>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Assign(self.box_assign_expr(span, op, left, right))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_member_expr(
        &self,
        span: Span,
        obj: Expr<'a>,
        prop: MemberProp<'a>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Member(self.box_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_super_prop_expr(
        &self,
        span: Span,
        obj: Box<'a, Super>,
        prop: SuperProp<'a>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::SuperProp(self.box_super_prop_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_cond_expr(
        &self,
        span: Span,
        test: Expr<'a>,
        cons: Expr<'a>,
        alt: Expr<'a>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Cond(self.box_cond_expr(span, test, cons, alt))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_call_expr(
        &self,
        span: Span,
        callee: Callee<'a>,
        args: Vec<'a, ExprOrSpread<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Call(self.box_call_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_new_expr(
        &self,
        span: Span,
        callee: Expr<'a>,
        args: Option<Vec<'a, ExprOrSpread<'a>>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::New(self.box_new_expr(span, callee, args))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_seq_expr(&self, span: Span, exprs: Vec<'a, Expr<'a>>) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Seq(self.box_seq_expr(span, exprs))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_ident(&self, span: Span, sym: Atom<'a>) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::Ident(self.box_ident(span, sym))))
    }
    #[inline]
    pub fn jsx_expr_expr_lit_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Str(self.box_str(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_lit_bool(&self, span: Span, value: bool) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Bool(self.box_bool(span, value))),
        )))
    }
    #[inline]
    pub fn jsx_expr_expr_lit_null(&self, span: Span) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::Lit(
            self.allocator.boxed(Lit::Null(self.box_null(span))),
        )))
    }
    #[inline]
    pub fn jsx_expr_expr_lit_number(
        &self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Num(self.box_number(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_lit_big_int(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::BigInt(self.box_big_int(span, value, raw))),
            )),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_lit_regex(
        &self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator.boxed(Expr::Lit(
                self.allocator
                    .boxed(Lit::Regex(self.box_regex(span, exp, flags))),
            )),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_tpl(
        &self,
        span: Span,
        exprs: Vec<'a, Expr<'a>>,
        quasis: Vec<'a, TplElement<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Tpl(self.box_tpl(span, exprs, quasis))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_tagged_tpl(
        &self,
        span: Span,
        tag: Expr<'a>,
        tpl: Box<'a, Tpl<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::TaggedTpl(self.box_tagged_tpl(span, tag, tpl))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_arrow_expr(
        &self,
        span: Span,
        params: Vec<'a, Pat<'a>>,
        body: BlockStmtOrExpr<'a>,
        is_async: bool,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::Arrow(
            self.box_arrow_expr(span, params, body, is_async),
        )))
    }
    #[inline]
    pub fn jsx_expr_expr_class_expr(
        &self,
        ident: Option<Box<'a, Ident<'a>>>,
        class: Box<'a, Class<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Class(self.box_class_expr(ident, class))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_yield_expr(
        &self,
        span: Span,
        arg: Option<Expr<'a>>,
        delegate: bool,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Yield(self.box_yield_expr(span, arg, delegate))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_meta_prop_expr(&self, span: Span, kind: MetaPropKind) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::MetaProp(self.box_meta_prop_expr(span, kind))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_await_expr(&self, span: Span, arg: Expr<'a>) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Await(self.box_await_expr(span, arg))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_paren_expr(&self, span: Span, expr: Expr<'a>) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::Paren(self.box_paren_expr(span, expr))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::JSXMember(self.box_jsx_member_expr(span, obj, prop))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::JSXNamespacedName(
            self.box_jsx_namespaced_name(span, ns, name),
        )))
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_empty_expr(&self, span: Span) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::JSXEmpty(self.box_jsx_empty_expr(span))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::JSXElement(
            self.box_jsx_element(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::JSXFragment(
            self.box_jsx_fragment(span, opening, children, closing),
        )))
    }
    #[inline]
    pub fn jsx_expr_expr_private_name(&self, span: Span, name: Atom<'a>) -> JSXExpr<'a> {
        JSXExpr::Expr(
            self.allocator
                .boxed(Expr::PrivateName(self.box_private_name(span, name))),
        )
    }
    #[inline]
    pub fn jsx_expr_expr_opt_chain_expr(
        &self,
        span: Span,
        optional: bool,
        base: OptChainBase<'a>,
    ) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::OptChain(
            self.box_opt_chain_expr(span, optional, base),
        )))
    }
    #[inline]
    pub fn jsx_expr_expr_invalid(&self) -> JSXExpr<'a> {
        JSXExpr::Expr(self.allocator.boxed(Expr::Invalid(self.box_invalid())))
    }
    #[inline]
    pub fn jsx_spread_child(&self, span: Span, expr: Expr<'a>) -> JSXSpreadChild<'a> {
        JSXSpreadChild { span, expr }
    }
    #[inline]
    pub fn box_jsx_spread_child(&self, span: Span, expr: Expr<'a>) -> Box<'a, JSXSpreadChild<'a>> {
        self.allocator.boxed(self.jsx_spread_child(span, expr))
    }
    #[inline]
    pub fn jsx_element_name_ident(&self, span: Span, sym: Atom<'a>) -> JSXElementName<'a> {
        JSXElementName::Ident(self.box_ident(span, sym))
    }
    #[inline]
    pub fn jsx_element_name_jsx_member_expr(
        &self,
        span: Span,
        obj: JSXObject<'a>,
        prop: Box<'a, IdentName<'a>>,
    ) -> JSXElementName<'a> {
        JSXElementName::JSXMemberExpr(self.box_jsx_member_expr(span, obj, prop))
    }
    #[inline]
    pub fn jsx_element_name_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> JSXElementName<'a> {
        JSXElementName::JSXNamespacedName(self.box_jsx_namespaced_name(span, ns, name))
    }
    #[inline]
    pub fn jsx_opening_element(
        &self,
        span: Span,
        name: JSXElementName<'a>,
        attrs: Vec<'a, JSXAttrOrSpread<'a>>,
        self_closing: bool,
    ) -> JSXOpeningElement<'a> {
        JSXOpeningElement {
            span,
            name,
            attrs,
            self_closing,
        }
    }
    #[inline]
    pub fn box_jsx_opening_element(
        &self,
        span: Span,
        name: JSXElementName<'a>,
        attrs: Vec<'a, JSXAttrOrSpread<'a>>,
        self_closing: bool,
    ) -> Box<'a, JSXOpeningElement<'a>> {
        self.allocator
            .boxed(self.jsx_opening_element(span, name, attrs, self_closing))
    }
    #[inline]
    pub fn jsx_attr_or_spread_jsx_attr(
        &self,
        span: Span,
        name: JSXAttrName<'a>,
        value: Option<JSXAttrValue<'a>>,
    ) -> JSXAttrOrSpread<'a> {
        JSXAttrOrSpread::JSXAttr(self.box_jsx_attr(span, name, value))
    }
    #[inline]
    pub fn jsx_attr_or_spread_spread_element(
        &self,
        dot3_token: Span,
        expr: Expr<'a>,
    ) -> JSXAttrOrSpread<'a> {
        JSXAttrOrSpread::SpreadElement(self.box_spread_element(dot3_token, expr))
    }
    #[inline]
    pub fn jsx_closing_element(
        &self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> JSXClosingElement<'a> {
        JSXClosingElement { span, name }
    }
    #[inline]
    pub fn box_jsx_closing_element(
        &self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> Box<'a, JSXClosingElement<'a>> {
        self.allocator.boxed(self.jsx_closing_element(span, name))
    }
    #[inline]
    pub fn jsx_attr(
        &self,
        span: Span,
        name: JSXAttrName<'a>,
        value: Option<JSXAttrValue<'a>>,
    ) -> JSXAttr<'a> {
        JSXAttr { span, name, value }
    }
    #[inline]
    pub fn box_jsx_attr(
        &self,
        span: Span,
        name: JSXAttrName<'a>,
        value: Option<JSXAttrValue<'a>>,
    ) -> Box<'a, JSXAttr<'a>> {
        self.allocator.boxed(self.jsx_attr(span, name, value))
    }
    #[inline]
    pub fn jsx_attr_name_ident_name(&self, span: Span, sym: Atom<'a>) -> JSXAttrName<'a> {
        JSXAttrName::Ident(self.box_ident_name(span, sym))
    }
    #[inline]
    pub fn jsx_attr_name_jsx_namespaced_name(
        &self,
        span: Span,
        ns: Box<'a, IdentName<'a>>,
        name: Box<'a, IdentName<'a>>,
    ) -> JSXAttrName<'a> {
        JSXAttrName::JSXNamespacedName(self.box_jsx_namespaced_name(span, ns, name))
    }
    #[inline]
    pub fn jsx_attr_value_str(
        &self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> JSXAttrValue<'a> {
        JSXAttrValue::Str(self.box_str(span, value, raw))
    }
    #[inline]
    pub fn jsx_attr_value_jsx_expr_container(
        &self,
        span: Span,
        expr: JSXExpr<'a>,
    ) -> JSXAttrValue<'a> {
        JSXAttrValue::JSXExprContainer(self.box_jsx_expr_container(span, expr))
    }
    #[inline]
    pub fn jsx_attr_value_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> JSXAttrValue<'a> {
        JSXAttrValue::JSXElement(self.box_jsx_element(span, opening, children, closing))
    }
    #[inline]
    pub fn jsx_attr_value_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> JSXAttrValue<'a> {
        JSXAttrValue::JSXFragment(self.box_jsx_fragment(span, opening, children, closing))
    }
    #[inline]
    pub fn jsx_text(&self, span: Span, value: Atom<'a>, raw: Atom<'a>) -> JSXText<'a> {
        JSXText { span, value, raw }
    }
    #[inline]
    pub fn box_jsx_text(&self, span: Span, value: Atom<'a>, raw: Atom<'a>) -> Box<'a, JSXText<'a>> {
        self.allocator.boxed(self.jsx_text(span, value, raw))
    }
    #[inline]
    pub fn jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> JSXElement<'a> {
        JSXElement {
            span,
            opening,
            children,
            closing,
        }
    }
    #[inline]
    pub fn box_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> Box<'a, JSXElement<'a>> {
        self.allocator
            .boxed(self.jsx_element(span, opening, children, closing))
    }
    #[inline]
    pub fn jsx_element_child_jsx_text(
        &self,
        span: Span,
        value: Atom<'a>,
        raw: Atom<'a>,
    ) -> JSXElementChild<'a> {
        JSXElementChild::JSXText(self.box_jsx_text(span, value, raw))
    }
    #[inline]
    pub fn jsx_element_child_jsx_expr_container(
        &self,
        span: Span,
        expr: JSXExpr<'a>,
    ) -> JSXElementChild<'a> {
        JSXElementChild::JSXExprContainer(self.box_jsx_expr_container(span, expr))
    }
    #[inline]
    pub fn jsx_element_child_jsx_spread_child(
        &self,
        span: Span,
        expr: Expr<'a>,
    ) -> JSXElementChild<'a> {
        JSXElementChild::JSXSpreadChild(self.box_jsx_spread_child(span, expr))
    }
    #[inline]
    pub fn jsx_element_child_jsx_element(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningElement<'a>>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Option<Box<'a, JSXClosingElement<'a>>>,
    ) -> JSXElementChild<'a> {
        JSXElementChild::JSXElement(self.box_jsx_element(span, opening, children, closing))
    }
    #[inline]
    pub fn jsx_element_child_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> JSXElementChild<'a> {
        JSXElementChild::JSXFragment(self.box_jsx_fragment(span, opening, children, closing))
    }
    #[inline]
    pub fn jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> JSXFragment<'a> {
        JSXFragment {
            span,
            opening,
            children,
            closing,
        }
    }
    #[inline]
    pub fn box_jsx_fragment(
        &self,
        span: Span,
        opening: Box<'a, JSXOpeningFragment>,
        children: Vec<'a, JSXElementChild<'a>>,
        closing: Box<'a, JSXClosingFragment>,
    ) -> Box<'a, JSXFragment<'a>> {
        self.allocator
            .boxed(self.jsx_fragment(span, opening, children, closing))
    }
    #[inline]
    pub fn jsx_opening_fragment(&self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment { span }
    }
    #[inline]
    pub fn box_jsx_opening_fragment(&self, span: Span) -> Box<'a, JSXOpeningFragment> {
        self.allocator.boxed(self.jsx_opening_fragment(span))
    }
    #[inline]
    pub fn jsx_closing_fragment(&self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment { span }
    }
    #[inline]
    pub fn box_jsx_closing_fragment(&self, span: Span) -> Box<'a, JSXClosingFragment> {
        self.allocator.boxed(self.jsx_closing_fragment(span))
    }
}
