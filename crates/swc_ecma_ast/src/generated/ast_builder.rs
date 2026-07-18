#![allow(unused, clippy::useless_conversion, clippy::identity_op)]
use crate::*;
impl<'a> Ast<'a> {
    #[inline]
    pub fn program_module(
        &mut self,
        span: Span,
        body: TypedSubRange<ModuleItem>,
        shebang: Option<Atom<'a>>,
    ) -> Program {
        Program::Module(self.module(span, body, shebang).into())
    }
    #[inline]
    pub fn program_script(
        &mut self,
        span: Span,
        body: TypedSubRange<Stmt>,
        shebang: Option<Atom<'a>>,
    ) -> Program {
        Program::Script(self.script(span, body, shebang).into())
    }
    #[inline]
    pub fn module(
        &mut self,
        span: Span,
        body: TypedSubRange<ModuleItem>,
        shebang: Option<Atom<'a>>,
    ) -> Module {
        let _f0 = self.add_extra(shebang.to_extra_data(self));
        Module(self.add_node(
            span,
            NodeKind::Module,
            NodePayload::partial(0u64 | unsafe { body.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn script(
        &mut self,
        span: Span,
        body: TypedSubRange<Stmt>,
        shebang: Option<Atom<'a>>,
    ) -> Script {
        let _f0 = self.add_extra(shebang.to_extra_data(self));
        Script(self.add_node(
            span,
            NodeKind::Script,
            NodePayload::partial(0u64 | unsafe { body.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn module_item_module_decl_import_decl(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ImportSpecifier>,
        src: Str,
        type_only: bool,
        with: Option<ObjectLit>,
        phase: ImportPhase,
    ) -> ModuleItem {
        ModuleItem::ModuleDecl(ModuleDecl::Import(
            self.import_decl(span, specifiers, src, type_only, with, phase)
                .into(),
        ))
    }
    #[inline]
    pub fn module_item_module_decl_export_decl(&mut self, span: Span, decl: Decl) -> ModuleItem {
        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(self.export_decl(span, decl).into()))
    }
    #[inline]
    pub fn module_item_module_decl_named_export(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ExportSpecifier>,
        src: Option<Str>,
        type_only: bool,
        with: Option<ObjectLit>,
    ) -> ModuleItem {
        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
            self.named_export(span, specifiers, src, type_only, with)
                .into(),
        ))
    }
    #[inline]
    pub fn module_item_module_decl_export_default_decl(
        &mut self,
        span: Span,
        decl: DefaultDecl,
    ) -> ModuleItem {
        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(
            self.export_default_decl(span, decl).into(),
        ))
    }
    #[inline]
    pub fn module_item_module_decl_export_default_expr(
        &mut self,
        span: Span,
        expr: Expr,
    ) -> ModuleItem {
        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
            self.export_default_expr(span, expr).into(),
        ))
    }
    #[inline]
    pub fn module_item_module_decl_export_all(
        &mut self,
        span: Span,
        src: Str,
        type_only: bool,
        with: Option<ObjectLit>,
    ) -> ModuleItem {
        ModuleItem::ModuleDecl(ModuleDecl::ExportAll(
            self.export_all(span, src, type_only, with).into(),
        ))
    }
    #[inline]
    pub fn module_item_stmt_block_stmt(
        &mut self,
        span: Span,
        stmts: TypedSubRange<Stmt>,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Block(self.block_stmt(span, stmts).into()))
    }
    #[inline]
    pub fn module_item_stmt_empty_stmt(&mut self, span: Span) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Empty(self.empty_stmt(span).into()))
    }
    #[inline]
    pub fn module_item_stmt_debugger_stmt(&mut self, span: Span) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Debugger(self.debugger_stmt(span).into()))
    }
    #[inline]
    pub fn module_item_stmt_with_stmt(&mut self, span: Span, obj: Expr, body: Stmt) -> ModuleItem {
        ModuleItem::Stmt(Stmt::With(self.with_stmt(span, obj, body).into()))
    }
    #[inline]
    pub fn module_item_stmt_return_stmt(&mut self, span: Span, arg: Option<Expr>) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Return(self.return_stmt(span, arg).into()))
    }
    #[inline]
    pub fn module_item_stmt_labeled_stmt(
        &mut self,
        span: Span,
        label: Ident,
        body: Stmt,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Labeled(self.labeled_stmt(span, label, body).into()))
    }
    #[inline]
    pub fn module_item_stmt_break_stmt(&mut self, span: Span, label: Option<Ident>) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Break(self.break_stmt(span, label).into()))
    }
    #[inline]
    pub fn module_item_stmt_continue_stmt(
        &mut self,
        span: Span,
        label: Option<Ident>,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Continue(self.continue_stmt(span, label).into()))
    }
    #[inline]
    pub fn module_item_stmt_if_stmt(
        &mut self,
        span: Span,
        test: Expr,
        cons: Stmt,
        alt: Option<Stmt>,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::If(self.if_stmt(span, test, cons, alt).into()))
    }
    #[inline]
    pub fn module_item_stmt_switch_stmt(
        &mut self,
        span: Span,
        discriminant: Expr,
        cases: TypedSubRange<SwitchCase>,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Switch(
            self.switch_stmt(span, discriminant, cases).into(),
        ))
    }
    #[inline]
    pub fn module_item_stmt_throw_stmt(&mut self, span: Span, arg: Expr) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Throw(self.throw_stmt(span, arg).into()))
    }
    #[inline]
    pub fn module_item_stmt_try_stmt(
        &mut self,
        span: Span,
        block: BlockStmt,
        handler: Option<CatchClause>,
        finalizer: Option<BlockStmt>,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Try(
            self.try_stmt(span, block, handler, finalizer).into(),
        ))
    }
    #[inline]
    pub fn module_item_stmt_while_stmt(
        &mut self,
        span: Span,
        test: Expr,
        body: Stmt,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::While(self.while_stmt(span, test, body).into()))
    }
    #[inline]
    pub fn module_item_stmt_do_while_stmt(
        &mut self,
        span: Span,
        test: Expr,
        body: Stmt,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::DoWhile(self.do_while_stmt(span, test, body).into()))
    }
    #[inline]
    pub fn module_item_stmt_for_stmt(
        &mut self,
        span: Span,
        init: Option<VarDeclOrExpr>,
        test: Option<Expr>,
        update: Option<Expr>,
        body: Stmt,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::For(
            self.for_stmt(span, init, test, update, body).into(),
        ))
    }
    #[inline]
    pub fn module_item_stmt_for_in_stmt(
        &mut self,
        span: Span,
        left: ForHead,
        right: Expr,
        body: Stmt,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::ForIn(
            self.for_in_stmt(span, left, right, body).into(),
        ))
    }
    #[inline]
    pub fn module_item_stmt_for_of_stmt(
        &mut self,
        span: Span,
        is_await: bool,
        left: ForHead,
        right: Expr,
        body: Stmt,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::ForOf(
            self.for_of_stmt(span, is_await, left, right, body).into(),
        ))
    }
    #[inline]
    pub fn module_item_stmt_decl_class_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        class: Class,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Decl(Decl::Class(
            self.class_decl(span, ident, declare, class).into(),
        )))
    }
    #[inline]
    pub fn module_item_stmt_decl_fn_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        function: Function,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Decl(Decl::Fn(
            self.fn_decl(span, ident, declare, function).into(),
        )))
    }
    #[inline]
    pub fn module_item_stmt_decl_var_decl(
        &mut self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Decl(Decl::Var(
            self.var_decl(span, kind, declare, decls).into(),
        )))
    }
    #[inline]
    pub fn module_item_stmt_decl_using_decl(
        &mut self,
        span: Span,
        is_await: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Decl(Decl::Using(
            self.using_decl(span, is_await, decls).into(),
        )))
    }
    #[inline]
    pub fn module_item_stmt_expr_stmt(&mut self, span: Span, expr: Expr) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Expr(self.expr_stmt(span, expr).into()))
    }
    #[inline]
    pub fn module_decl_import_decl(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ImportSpecifier>,
        src: Str,
        type_only: bool,
        with: Option<ObjectLit>,
        phase: ImportPhase,
    ) -> ModuleDecl {
        ModuleDecl::Import(
            self.import_decl(span, specifiers, src, type_only, with, phase)
                .into(),
        )
    }
    #[inline]
    pub fn module_decl_export_decl(&mut self, span: Span, decl: Decl) -> ModuleDecl {
        ModuleDecl::ExportDecl(self.export_decl(span, decl).into())
    }
    #[inline]
    pub fn module_decl_named_export(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ExportSpecifier>,
        src: Option<Str>,
        type_only: bool,
        with: Option<ObjectLit>,
    ) -> ModuleDecl {
        ModuleDecl::ExportNamed(
            self.named_export(span, specifiers, src, type_only, with)
                .into(),
        )
    }
    #[inline]
    pub fn module_decl_export_default_decl(&mut self, span: Span, decl: DefaultDecl) -> ModuleDecl {
        ModuleDecl::ExportDefaultDecl(self.export_default_decl(span, decl).into())
    }
    #[inline]
    pub fn module_decl_export_default_expr(&mut self, span: Span, expr: Expr) -> ModuleDecl {
        ModuleDecl::ExportDefaultExpr(self.export_default_expr(span, expr).into())
    }
    #[inline]
    pub fn module_decl_export_all(
        &mut self,
        span: Span,
        src: Str,
        type_only: bool,
        with: Option<ObjectLit>,
    ) -> ModuleDecl {
        ModuleDecl::ExportAll(self.export_all(span, src, type_only, with).into())
    }
    #[inline]
    pub fn import_decl(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ImportSpecifier>,
        src: Str,
        type_only: bool,
        with: Option<ObjectLit>,
        phase: ImportPhase,
    ) -> ImportDecl {
        let _f0 = self.add_extra(src.to_extra_data(self));
        let _f1 = self.add_extra(type_only.to_extra_data(self));
        let _f2 = self.add_extra(with.to_extra_data(self));
        let _f3 = self.add_extra(phase.to_extra_data(self));
        ImportDecl(self.add_node(
            span,
            NodeKind::ImportDecl,
            NodePayload::partial(0u64 | unsafe { specifiers.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn import_specifier_import_named_specifier(
        &mut self,
        span: Span,
        local: Ident,
        imported: ModuleExportName,
        is_type_only: bool,
    ) -> ImportSpecifier {
        ImportSpecifier::Named(
            self.import_named_specifier(span, local, imported, is_type_only)
                .into(),
        )
    }
    #[inline]
    pub fn import_specifier_import_default_specifier(
        &mut self,
        span: Span,
        local: Ident,
    ) -> ImportSpecifier {
        ImportSpecifier::Default(self.import_default_specifier(span, local).into())
    }
    #[inline]
    pub fn import_specifier_import_star_as_specifier(
        &mut self,
        span: Span,
        local: Ident,
    ) -> ImportSpecifier {
        ImportSpecifier::Namespace(self.import_star_as_specifier(span, local).into())
    }
    #[inline]
    pub fn import_named_specifier(
        &mut self,
        span: Span,
        local: Ident,
        imported: ModuleExportName,
        is_type_only: bool,
    ) -> ImportNamedSpecifier {
        ImportNamedSpecifier(self.add_node(
            span,
            NodeKind::ImportNamedSpecifier,
            NodePayload::inline(
                0u64 | (local.node_id().index() as u32 as u64)
                    | ((imported.node_id().index() as u32 as u64) << 32usize),
                0u32 | is_type_only as u32,
            ),
        ))
    }
    #[inline]
    pub fn import_default_specifier(&mut self, span: Span, local: Ident) -> ImportDefaultSpecifier {
        ImportDefaultSpecifier(self.add_node(
            span,
            NodeKind::ImportDefaultSpecifier,
            NodePayload::inline(0u64 | (local.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn import_star_as_specifier(&mut self, span: Span, local: Ident) -> ImportStarAsSpecifier {
        ImportStarAsSpecifier(self.add_node(
            span,
            NodeKind::ImportStarAsSpecifier,
            NodePayload::inline(0u64 | (local.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn export_decl(&mut self, span: Span, decl: Decl) -> ExportDecl {
        ExportDecl(self.add_node(
            span,
            NodeKind::ExportDecl,
            NodePayload::inline(0u64 | (decl.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn named_export(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ExportSpecifier>,
        src: Option<Str>,
        type_only: bool,
        with: Option<ObjectLit>,
    ) -> NamedExport {
        let _f0 = self.add_extra(src.to_extra_data(self));
        let _f1 = self.add_extra(type_only.to_extra_data(self));
        let _f2 = self.add_extra(with.to_extra_data(self));
        NamedExport(self.add_node(
            span,
            NodeKind::NamedExport,
            NodePayload::partial(0u64 | unsafe { specifiers.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn export_specifier_export_namespace_specifier(
        &mut self,
        span: Span,
        name: ModuleExportName,
    ) -> ExportSpecifier {
        ExportSpecifier::Namespace(self.export_namespace_specifier(span, name).into())
    }
    #[inline]
    pub fn export_specifier_export_default_specifier(
        &mut self,
        span: Span,
        exported: Ident,
    ) -> ExportSpecifier {
        ExportSpecifier::Default(self.export_default_specifier(span, exported).into())
    }
    #[inline]
    pub fn export_specifier_export_named_specifier(
        &mut self,
        span: Span,
        orig: ModuleExportName,
        exported: ModuleExportName,
        is_type_only: bool,
    ) -> ExportSpecifier {
        ExportSpecifier::Named(
            self.export_named_specifier(span, orig, exported, is_type_only)
                .into(),
        )
    }
    #[inline]
    pub fn export_namespace_specifier(
        &mut self,
        span: Span,
        name: ModuleExportName,
    ) -> ExportNamespaceSpecifier {
        ExportNamespaceSpecifier(self.add_node(
            span,
            NodeKind::ExportNamespaceSpecifier,
            NodePayload::inline(0u64 | (name.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn module_export_name_ident(&mut self, span: Span, sym: Atom<'a>) -> ModuleExportName {
        ModuleExportName::Ident(self.ident(span, sym).into())
    }
    #[inline]
    pub fn module_export_name_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> ModuleExportName {
        ModuleExportName::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn export_default_specifier(
        &mut self,
        span: Span,
        exported: Ident,
    ) -> ExportDefaultSpecifier {
        ExportDefaultSpecifier(self.add_node(
            span,
            NodeKind::ExportDefaultSpecifier,
            NodePayload::inline(0u64 | (exported.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn export_named_specifier(
        &mut self,
        span: Span,
        orig: ModuleExportName,
        exported: ModuleExportName,
        is_type_only: bool,
    ) -> ExportNamedSpecifier {
        ExportNamedSpecifier(self.add_node(
            span,
            NodeKind::ExportNamedSpecifier,
            NodePayload::inline(
                0u64 | (orig.node_id().index() as u32 as u64)
                    | ((exported.node_id().index() as u32 as u64) << 32usize),
                0u32 | is_type_only as u32,
            ),
        ))
    }
    #[inline]
    pub fn export_default_decl(&mut self, span: Span, decl: DefaultDecl) -> ExportDefaultDecl {
        ExportDefaultDecl(self.add_node(
            span,
            NodeKind::ExportDefaultDecl,
            NodePayload::inline(0u64 | (decl.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn default_decl_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> DefaultDecl {
        DefaultDecl::Class(self.class_expr(span, ident, class).into())
    }
    #[inline]
    pub fn default_decl_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> DefaultDecl {
        DefaultDecl::Fn(self.fn_expr(span, ident, function).into())
    }
    #[inline]
    pub fn export_default_expr(&mut self, span: Span, expr: Expr) -> ExportDefaultExpr {
        ExportDefaultExpr(self.add_node(
            span,
            NodeKind::ExportDefaultExpr,
            NodePayload::inline(0u64 | (expr.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn export_all(
        &mut self,
        span: Span,
        src: Str,
        type_only: bool,
        with: Option<ObjectLit>,
    ) -> ExportAll {
        ExportAll(self.add_node(
            span,
            NodeKind::ExportAll,
            NodePayload::inline(
                0u64 | (src.node_id().index() as u32 as u64)
                    | ((crate::OptionalNodeId::from(with.map(|n| n.node_id())).into_raw() as u64)
                        << 32usize),
                0u32 | type_only as u32,
            ),
        ))
    }
    #[inline]
    pub fn block_stmt(&mut self, span: Span, stmts: TypedSubRange<Stmt>) -> BlockStmt {
        let _inline_default_1 = <Option<ScopeId>>::default();
        BlockStmt(self.add_node(
            span,
            NodeKind::BlockStmt,
            NodePayload::inline(
                0u64 | unsafe { stmts.to_extra_data(self).other },
                0u32 | _inline_default_1.map_or(0, |id| id.index() as u32 + 1),
            ),
        ))
    }
    #[inline]
    pub fn stmt_block_stmt(&mut self, span: Span, stmts: TypedSubRange<Stmt>) -> Stmt {
        Stmt::Block(self.block_stmt(span, stmts).into())
    }
    #[inline]
    pub fn stmt_empty_stmt(&mut self, span: Span) -> Stmt {
        Stmt::Empty(self.empty_stmt(span).into())
    }
    #[inline]
    pub fn stmt_debugger_stmt(&mut self, span: Span) -> Stmt {
        Stmt::Debugger(self.debugger_stmt(span).into())
    }
    #[inline]
    pub fn stmt_with_stmt(&mut self, span: Span, obj: Expr, body: Stmt) -> Stmt {
        Stmt::With(self.with_stmt(span, obj, body).into())
    }
    #[inline]
    pub fn stmt_return_stmt(&mut self, span: Span, arg: Option<Expr>) -> Stmt {
        Stmt::Return(self.return_stmt(span, arg).into())
    }
    #[inline]
    pub fn stmt_labeled_stmt(&mut self, span: Span, label: Ident, body: Stmt) -> Stmt {
        Stmt::Labeled(self.labeled_stmt(span, label, body).into())
    }
    #[inline]
    pub fn stmt_break_stmt(&mut self, span: Span, label: Option<Ident>) -> Stmt {
        Stmt::Break(self.break_stmt(span, label).into())
    }
    #[inline]
    pub fn stmt_continue_stmt(&mut self, span: Span, label: Option<Ident>) -> Stmt {
        Stmt::Continue(self.continue_stmt(span, label).into())
    }
    #[inline]
    pub fn stmt_if_stmt(&mut self, span: Span, test: Expr, cons: Stmt, alt: Option<Stmt>) -> Stmt {
        Stmt::If(self.if_stmt(span, test, cons, alt).into())
    }
    #[inline]
    pub fn stmt_switch_stmt(
        &mut self,
        span: Span,
        discriminant: Expr,
        cases: TypedSubRange<SwitchCase>,
    ) -> Stmt {
        Stmt::Switch(self.switch_stmt(span, discriminant, cases).into())
    }
    #[inline]
    pub fn stmt_throw_stmt(&mut self, span: Span, arg: Expr) -> Stmt {
        Stmt::Throw(self.throw_stmt(span, arg).into())
    }
    #[inline]
    pub fn stmt_try_stmt(
        &mut self,
        span: Span,
        block: BlockStmt,
        handler: Option<CatchClause>,
        finalizer: Option<BlockStmt>,
    ) -> Stmt {
        Stmt::Try(self.try_stmt(span, block, handler, finalizer).into())
    }
    #[inline]
    pub fn stmt_while_stmt(&mut self, span: Span, test: Expr, body: Stmt) -> Stmt {
        Stmt::While(self.while_stmt(span, test, body).into())
    }
    #[inline]
    pub fn stmt_do_while_stmt(&mut self, span: Span, test: Expr, body: Stmt) -> Stmt {
        Stmt::DoWhile(self.do_while_stmt(span, test, body).into())
    }
    #[inline]
    pub fn stmt_for_stmt(
        &mut self,
        span: Span,
        init: Option<VarDeclOrExpr>,
        test: Option<Expr>,
        update: Option<Expr>,
        body: Stmt,
    ) -> Stmt {
        Stmt::For(self.for_stmt(span, init, test, update, body).into())
    }
    #[inline]
    pub fn stmt_for_in_stmt(&mut self, span: Span, left: ForHead, right: Expr, body: Stmt) -> Stmt {
        Stmt::ForIn(self.for_in_stmt(span, left, right, body).into())
    }
    #[inline]
    pub fn stmt_for_of_stmt(
        &mut self,
        span: Span,
        is_await: bool,
        left: ForHead,
        right: Expr,
        body: Stmt,
    ) -> Stmt {
        Stmt::ForOf(self.for_of_stmt(span, is_await, left, right, body).into())
    }
    #[inline]
    pub fn stmt_decl_class_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        class: Class,
    ) -> Stmt {
        Stmt::Decl(Decl::Class(
            self.class_decl(span, ident, declare, class).into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_fn_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        function: Function,
    ) -> Stmt {
        Stmt::Decl(Decl::Fn(
            self.fn_decl(span, ident, declare, function).into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_var_decl(
        &mut self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> Stmt {
        Stmt::Decl(Decl::Var(self.var_decl(span, kind, declare, decls).into()))
    }
    #[inline]
    pub fn stmt_decl_using_decl(
        &mut self,
        span: Span,
        is_await: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> Stmt {
        Stmt::Decl(Decl::Using(self.using_decl(span, is_await, decls).into()))
    }
    #[inline]
    pub fn stmt_expr_stmt(&mut self, span: Span, expr: Expr) -> Stmt {
        Stmt::Expr(self.expr_stmt(span, expr).into())
    }
    #[inline]
    pub fn expr_stmt(&mut self, span: Span, expr: Expr) -> ExprStmt {
        ExprStmt(self.add_node(
            span,
            NodeKind::ExprStmt,
            NodePayload::inline(0u64 | (expr.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn empty_stmt(&mut self, span: Span) -> EmptyStmt {
        EmptyStmt(self.add_node(span, NodeKind::EmptyStmt, NodePayload::inline(0, 0)))
    }
    #[inline]
    pub fn debugger_stmt(&mut self, span: Span) -> DebuggerStmt {
        DebuggerStmt(self.add_node(span, NodeKind::DebuggerStmt, NodePayload::inline(0, 0)))
    }
    #[inline]
    pub fn with_stmt(&mut self, span: Span, obj: Expr, body: Stmt) -> WithStmt {
        WithStmt(self.add_node(
            span,
            NodeKind::WithStmt,
            NodePayload::inline(
                0u64 | (obj.node_id().index() as u32 as u64)
                    | ((body.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn return_stmt(&mut self, span: Span, arg: Option<Expr>) -> ReturnStmt {
        ReturnStmt(self.add_node(
            span,
            NodeKind::ReturnStmt,
            NodePayload::inline(
                0u64 | (crate::OptionalNodeId::from(arg.map(|n| n.node_id())).into_raw() as u64),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn labeled_stmt(&mut self, span: Span, label: Ident, body: Stmt) -> LabeledStmt {
        LabeledStmt(self.add_node(
            span,
            NodeKind::LabeledStmt,
            NodePayload::inline(
                0u64 | (label.node_id().index() as u32 as u64)
                    | ((body.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn break_stmt(&mut self, span: Span, label: Option<Ident>) -> BreakStmt {
        BreakStmt(self.add_node(
            span,
            NodeKind::BreakStmt,
            NodePayload::inline(
                0u64 | (crate::OptionalNodeId::from(label.map(|n| n.node_id())).into_raw() as u64),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn continue_stmt(&mut self, span: Span, label: Option<Ident>) -> ContinueStmt {
        ContinueStmt(self.add_node(
            span,
            NodeKind::ContinueStmt,
            NodePayload::inline(
                0u64 | (crate::OptionalNodeId::from(label.map(|n| n.node_id())).into_raw() as u64),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn if_stmt(&mut self, span: Span, test: Expr, cons: Stmt, alt: Option<Stmt>) -> IfStmt {
        IfStmt(self.add_node(
            span,
            NodeKind::IfStmt,
            NodePayload::inline(
                0u64 | (test.node_id().index() as u32 as u64)
                    | ((cons.node_id().index() as u32 as u64) << 32usize),
                0u32 | crate::OptionalNodeId::from(alt.map(|n| n.node_id())).into_raw(),
            ),
        ))
    }
    #[inline]
    pub fn switch_stmt(
        &mut self,
        span: Span,
        discriminant: Expr,
        cases: TypedSubRange<SwitchCase>,
    ) -> SwitchStmt {
        SwitchStmt(self.add_node(
            span,
            NodeKind::SwitchStmt,
            NodePayload::inline(
                0u64 | unsafe { cases.to_extra_data(self).other },
                0u32 | discriminant.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn throw_stmt(&mut self, span: Span, arg: Expr) -> ThrowStmt {
        ThrowStmt(self.add_node(
            span,
            NodeKind::ThrowStmt,
            NodePayload::inline(0u64 | (arg.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn try_stmt(
        &mut self,
        span: Span,
        block: BlockStmt,
        handler: Option<CatchClause>,
        finalizer: Option<BlockStmt>,
    ) -> TryStmt {
        TryStmt(self.add_node(
            span,
            NodeKind::TryStmt,
            NodePayload::inline(
                0u64 | (block.node_id().index() as u32 as u64)
                    | ((crate::OptionalNodeId::from(handler.map(|n| n.node_id())).into_raw()
                        as u64)
                        << 32usize),
                0u32 | crate::OptionalNodeId::from(finalizer.map(|n| n.node_id())).into_raw(),
            ),
        ))
    }
    #[inline]
    pub fn while_stmt(&mut self, span: Span, test: Expr, body: Stmt) -> WhileStmt {
        WhileStmt(self.add_node(
            span,
            NodeKind::WhileStmt,
            NodePayload::inline(
                0u64 | (test.node_id().index() as u32 as u64)
                    | ((body.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn do_while_stmt(&mut self, span: Span, test: Expr, body: Stmt) -> DoWhileStmt {
        DoWhileStmt(self.add_node(
            span,
            NodeKind::DoWhileStmt,
            NodePayload::inline(
                0u64 | (test.node_id().index() as u32 as u64)
                    | ((body.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn for_stmt(
        &mut self,
        span: Span,
        init: Option<VarDeclOrExpr>,
        test: Option<Expr>,
        update: Option<Expr>,
        body: Stmt,
    ) -> ForStmt {
        let _f0 = self.add_extra(update.to_extra_data(self));
        let _f1 = self.add_extra(body.to_extra_data(self));
        ForStmt(self.add_node(
            span,
            NodeKind::ForStmt,
            NodePayload::partial(
                0u64 | (crate::OptionalNodeId::from(init.map(|n| n.node_id())).into_raw() as u64)
                    | ((crate::OptionalNodeId::from(test.map(|n| n.node_id())).into_raw() as u64)
                        << 32usize),
                _f0,
            ),
        ))
    }
    #[inline]
    pub fn for_in_stmt(&mut self, span: Span, left: ForHead, right: Expr, body: Stmt) -> ForInStmt {
        ForInStmt(self.add_node(
            span,
            NodeKind::ForInStmt,
            NodePayload::inline(
                0u64 | (left.node_id().index() as u32 as u64)
                    | ((right.node_id().index() as u32 as u64) << 32usize),
                0u32 | body.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn for_of_stmt(
        &mut self,
        span: Span,
        is_await: bool,
        left: ForHead,
        right: Expr,
        body: Stmt,
    ) -> ForOfStmt {
        let _f0 = self.add_extra(is_await.to_extra_data(self));
        let _f1 = self.add_extra(body.to_extra_data(self));
        ForOfStmt(self.add_node(
            span,
            NodeKind::ForOfStmt,
            NodePayload::partial(
                0u64 | (left.node_id().index() as u32 as u64)
                    | ((right.node_id().index() as u32 as u64) << 32usize),
                _f0,
            ),
        ))
    }
    #[inline]
    pub fn switch_case(
        &mut self,
        span: Span,
        test: Option<Expr>,
        cons: TypedSubRange<Stmt>,
    ) -> SwitchCase {
        SwitchCase(self.add_node(
            span,
            NodeKind::SwitchCase,
            NodePayload::inline(
                0u64 | unsafe { cons.to_extra_data(self).other },
                0u32 | crate::OptionalNodeId::from(test.map(|n| n.node_id())).into_raw(),
            ),
        ))
    }
    #[inline]
    pub fn catch_clause(&mut self, span: Span, param: Option<Pat>, body: BlockStmt) -> CatchClause {
        CatchClause(self.add_node(
            span,
            NodeKind::CatchClause,
            NodePayload::inline(
                0u64 | (crate::OptionalNodeId::from(param.map(|n| n.node_id())).into_raw() as u64)
                    | ((body.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn for_head_var_decl(
        &mut self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> ForHead {
        ForHead::VarDecl(self.var_decl(span, kind, declare, decls).into())
    }
    #[inline]
    pub fn for_head_using_decl(
        &mut self,
        span: Span,
        is_await: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> ForHead {
        ForHead::UsingDecl(self.using_decl(span, is_await, decls).into())
    }
    #[inline]
    pub fn for_head_pat_binding_ident(&mut self, span: Span, id: Ident) -> ForHead {
        ForHead::Pat(Pat::Ident(self.binding_ident(span, id).into()))
    }
    #[inline]
    pub fn for_head_pat_array_pat(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<Pat>>,
        optional: bool,
    ) -> ForHead {
        ForHead::Pat(Pat::Array(self.array_pat(span, elems, optional).into()))
    }
    #[inline]
    pub fn for_head_pat_rest_pat(&mut self, span: Span, dot3_token: Span, arg: Pat) -> ForHead {
        ForHead::Pat(Pat::Rest(self.rest_pat(span, dot3_token, arg).into()))
    }
    #[inline]
    pub fn for_head_pat_object_pat(
        &mut self,
        span: Span,
        props: TypedSubRange<ObjectPatProp>,
        optional: bool,
    ) -> ForHead {
        ForHead::Pat(Pat::Object(self.object_pat(span, props, optional).into()))
    }
    #[inline]
    pub fn for_head_pat_assign_pat(&mut self, span: Span, left: Pat, right: Expr) -> ForHead {
        ForHead::Pat(Pat::Assign(self.assign_pat(span, left, right).into()))
    }
    #[inline]
    pub fn for_head_pat_invalid(&mut self, span: Span) -> ForHead {
        ForHead::Pat(Pat::Invalid(self.invalid(span).into()))
    }
    #[inline]
    pub fn for_head_pat_expr_this_expr(&mut self, span: Span) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::This(self.this_expr(span).into())))
    }
    #[inline]
    pub fn for_head_pat_expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Array(self.array_lit(span, elems).into())))
    }
    #[inline]
    pub fn for_head_pat_expr_object_lit(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Object(self.object_lit(span, props).into())))
    }
    #[inline]
    pub fn for_head_pat_expr_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Fn(
            self.fn_expr(span, ident, function).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Unary(
            self.unary_expr(span, op, arg).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Update(
            self.update_expr(span, op, prefix, arg).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_bin_expr(
        &mut self,
        span: Span,
        op: BinaryOp,
        left: Expr,
        right: Expr,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Bin(
            self.bin_expr(span, op, left, right).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Assign(
            self.assign_expr(span, op, left, right).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Member(
            self.member_expr(span, obj, prop).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::SuperProp(
            self.super_prop_expr(span, obj, prop).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_cond_expr(
        &mut self,
        span: Span,
        test: Expr,
        cons: Expr,
        alt: Expr,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Cond(
            self.cond_expr(span, test, cons, alt).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Call(
            self.call_expr(span, callee, args).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::New(
            self.new_expr(span, callee, args).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_seq_expr(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Seq(self.seq_expr(span, exprs).into())))
    }
    #[inline]
    pub fn for_head_pat_expr_ident(&mut self, span: Span, sym: Atom<'a>) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Ident(self.ident(span, sym).into())))
    }
    #[inline]
    pub fn for_head_pat_expr_lit_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Str(
            self.str(span, value, raw).into(),
        ))))
    }
    #[inline]
    pub fn for_head_pat_expr_lit_bool(&mut self, span: Span, value: bool) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Bool(
            self.bool(span, value).into(),
        ))))
    }
    #[inline]
    pub fn for_head_pat_expr_lit_null(&mut self, span: Span) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Null(self.null(span).into()))))
    }
    #[inline]
    pub fn for_head_pat_expr_lit_number(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Num(
            self.number(span, value, raw).into(),
        ))))
    }
    #[inline]
    pub fn for_head_pat_expr_lit_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Lit(Lit::BigInt(
            self.big_int(span, value, raw).into(),
        ))))
    }
    #[inline]
    pub fn for_head_pat_expr_lit_regex(
        &mut self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Lit(Lit::Regex(
            self.regex(span, exp, flags).into(),
        ))))
    }
    #[inline]
    pub fn for_head_pat_expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Tpl(self.tpl(span, exprs, quasis).into())))
    }
    #[inline]
    pub fn for_head_pat_expr_tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::TaggedTpl(
            self.tagged_tpl(span, tag, tpl).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Arrow(
            self.arrow_expr(span, params, body, is_async).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Class(
            self.class_expr(span, ident, class).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_yield_expr(
        &mut self,
        span: Span,
        arg: Option<Expr>,
        delegate: bool,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Yield(
            self.yield_expr(span, arg, delegate).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::MetaProp(
            self.meta_prop_expr(span, kind).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_await_expr(&mut self, span: Span, arg: Expr) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Await(self.await_expr(span, arg).into())))
    }
    #[inline]
    pub fn for_head_pat_expr_paren_expr(&mut self, span: Span, expr: Expr) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Paren(self.paren_expr(span, expr).into())))
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::JSXMember(
            self.jsx_member_expr(span, obj, prop).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::JSXNamespacedName(
            self.jsx_namespaced_name(span, ns, name).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_empty_expr(&mut self, span: Span) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::JSXEmpty(self.jsx_empty_expr(span).into())))
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::JSXElement(
            self.jsx_element(span, opening, children, closing).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::JSXFragment(
            self.jsx_fragment(span, opening, children, closing).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_private_name(&mut self, span: Span, name: Atom<'a>) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::PrivateName(
            self.private_name(span, name).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::OptChain(
            self.opt_chain_expr(span, optional, base).into(),
        )))
    }
    #[inline]
    pub fn for_head_pat_expr_invalid(&mut self, span: Span) -> ForHead {
        ForHead::Pat(Pat::Expr(Expr::Invalid(self.invalid(span).into())))
    }
    #[inline]
    pub fn var_decl_or_expr_var_decl(
        &mut self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::VarDecl(self.var_decl(span, kind, declare, decls).into())
    }
    #[inline]
    pub fn var_decl_or_expr_expr_this_expr(&mut self, span: Span) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::This(self.this_expr(span).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Array(self.array_lit(span, elems).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_object_lit(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Object(self.object_lit(span, props).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Fn(self.fn_expr(span, ident, function).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_unary_expr(
        &mut self,
        span: Span,
        op: UnaryOp,
        arg: Expr,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Unary(self.unary_expr(span, op, arg).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Update(self.update_expr(span, op, prefix, arg).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_bin_expr(
        &mut self,
        span: Span,
        op: BinaryOp,
        left: Expr,
        right: Expr,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Bin(self.bin_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Assign(self.assign_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Member(self.member_expr(span, obj, prop).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::SuperProp(
            self.super_prop_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_cond_expr(
        &mut self,
        span: Span,
        test: Expr,
        cons: Expr,
        alt: Expr,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Cond(self.cond_expr(span, test, cons, alt).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        ))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Call(self.call_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::New(self.new_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_seq_expr(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Seq(self.seq_expr(span, exprs).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_ident(&mut self, span: Span, sym: Atom<'a>) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Ident(self.ident(span, sym).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Lit(Lit::Str(self.str(span, value, raw).into())))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_bool(&mut self, span: Span, value: bool) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Lit(Lit::Bool(self.bool(span, value).into())))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_null(&mut self, span: Span) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Lit(Lit::Null(self.null(span).into())))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_number(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Lit(Lit::Num(self.number(span, value, raw).into())))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Lit(Lit::BigInt(
            self.big_int(span, value, raw).into(),
        )))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_lit_regex(
        &mut self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Lit(Lit::Regex(self.regex(span, exp, flags).into())))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Tpl(self.tpl(span, exprs, quasis).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_tagged_tpl(
        &mut self,
        span: Span,
        tag: Expr,
        tpl: Tpl,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::TaggedTpl(self.tagged_tpl(span, tag, tpl).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Arrow(
            self.arrow_expr(span, params, body, is_async).into(),
        ))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Class(self.class_expr(span, ident, class).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_yield_expr(
        &mut self,
        span: Span,
        arg: Option<Expr>,
        delegate: bool,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Yield(self.yield_expr(span, arg, delegate).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_meta_prop_expr(
        &mut self,
        span: Span,
        kind: MetaPropKind,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::MetaProp(self.meta_prop_expr(span, kind).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_await_expr(&mut self, span: Span, arg: Expr) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Await(self.await_expr(span, arg).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_paren_expr(&mut self, span: Span, expr: Expr) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Paren(self.paren_expr(span, expr).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::JSXMember(
            self.jsx_member_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::JSXNamespacedName(
            self.jsx_namespaced_name(span, ns, name).into(),
        ))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_empty_expr(&mut self, span: Span) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::JSXEmpty(self.jsx_empty_expr(span).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::JSXElement(
            self.jsx_element(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::JSXFragment(
            self.jsx_fragment(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_private_name(
        &mut self,
        span: Span,
        name: Atom<'a>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::PrivateName(self.private_name(span, name).into()))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::OptChain(
            self.opt_chain_expr(span, optional, base).into(),
        ))
    }
    #[inline]
    pub fn var_decl_or_expr_expr_invalid(&mut self, span: Span) -> VarDeclOrExpr {
        VarDeclOrExpr::Expr(Expr::Invalid(self.invalid(span).into()))
    }
    #[inline]
    pub fn decl_class_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        class: Class,
    ) -> Decl {
        Decl::Class(self.class_decl(span, ident, declare, class).into())
    }
    #[inline]
    pub fn decl_fn_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        function: Function,
    ) -> Decl {
        Decl::Fn(self.fn_decl(span, ident, declare, function).into())
    }
    #[inline]
    pub fn decl_var_decl(
        &mut self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> Decl {
        Decl::Var(self.var_decl(span, kind, declare, decls).into())
    }
    #[inline]
    pub fn decl_using_decl(
        &mut self,
        span: Span,
        is_await: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> Decl {
        Decl::Using(self.using_decl(span, is_await, decls).into())
    }
    #[inline]
    pub fn fn_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        function: Function,
    ) -> FnDecl {
        FnDecl(self.add_node(
            span,
            NodeKind::FnDecl,
            NodePayload::inline(
                0u64 | (ident.node_id().index() as u32 as u64)
                    | ((function.node_id().index() as u32 as u64) << 32usize),
                0u32 | declare as u32,
            ),
        ))
    }
    #[inline]
    pub fn class_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        class: Class,
    ) -> ClassDecl {
        ClassDecl(self.add_node(
            span,
            NodeKind::ClassDecl,
            NodePayload::inline(
                0u64 | (ident.node_id().index() as u32 as u64)
                    | ((class.node_id().index() as u32 as u64) << 32usize),
                0u32 | declare as u32,
            ),
        ))
    }
    #[inline]
    pub fn var_decl(
        &mut self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> VarDecl {
        VarDecl(self.add_node(
            span,
            NodeKind::VarDecl,
            NodePayload::inline(
                0u64 | unsafe { decls.to_extra_data(self).other },
                0u32 | kind as u32 | ((declare as u32) << 8usize),
            ),
        ))
    }
    #[inline]
    pub fn var_declarator(&mut self, span: Span, name: Pat, init: Option<Expr>) -> VarDeclarator {
        VarDeclarator(self.add_node(
            span,
            NodeKind::VarDeclarator,
            NodePayload::inline(
                0u64 | (name.node_id().index() as u32 as u64)
                    | ((crate::OptionalNodeId::from(init.map(|n| n.node_id())).into_raw() as u64)
                        << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn using_decl(
        &mut self,
        span: Span,
        is_await: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> UsingDecl {
        UsingDecl(self.add_node(
            span,
            NodeKind::UsingDecl,
            NodePayload::inline(
                0u64 | unsafe { decls.to_extra_data(self).other },
                0u32 | is_await as u32,
            ),
        ))
    }
    #[inline]
    pub fn expr_this_expr(&mut self, span: Span) -> Expr {
        Expr::This(self.this_expr(span).into())
    }
    #[inline]
    pub fn expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> Expr {
        Expr::Array(self.array_lit(span, elems).into())
    }
    #[inline]
    pub fn expr_object_lit(&mut self, span: Span, props: TypedSubRange<PropOrSpread>) -> Expr {
        Expr::Object(self.object_lit(span, props).into())
    }
    #[inline]
    pub fn expr_fn_expr(&mut self, span: Span, ident: Option<Ident>, function: Function) -> Expr {
        Expr::Fn(self.fn_expr(span, ident, function).into())
    }
    #[inline]
    pub fn expr_unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> Expr {
        Expr::Unary(self.unary_expr(span, op, arg).into())
    }
    #[inline]
    pub fn expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> Expr {
        Expr::Update(self.update_expr(span, op, prefix, arg).into())
    }
    #[inline]
    pub fn expr_bin_expr(&mut self, span: Span, op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::Bin(self.bin_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> Expr {
        Expr::Assign(self.assign_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn expr_member_expr(&mut self, span: Span, obj: Expr, prop: MemberProp) -> Expr {
        Expr::Member(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn expr_super_prop_expr(&mut self, span: Span, obj: Super, prop: SuperProp) -> Expr {
        Expr::SuperProp(self.super_prop_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn expr_cond_expr(&mut self, span: Span, test: Expr, cons: Expr, alt: Expr) -> Expr {
        Expr::Cond(self.cond_expr(span, test, cons, alt).into())
    }
    #[inline]
    pub fn expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> Expr {
        Expr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        )
    }
    #[inline]
    pub fn expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Expr {
        Expr::Call(self.call_expr(span, callee, args).into())
    }
    #[inline]
    pub fn expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Expr {
        Expr::New(self.new_expr(span, callee, args).into())
    }
    #[inline]
    pub fn expr_seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> Expr {
        Expr::Seq(self.seq_expr(span, exprs).into())
    }
    #[inline]
    pub fn expr_ident(&mut self, span: Span, sym: Atom<'a>) -> Expr {
        Expr::Ident(self.ident(span, sym).into())
    }
    #[inline]
    pub fn expr_lit_str(&mut self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Expr {
        Expr::Lit(Lit::Str(self.str(span, value, raw).into()))
    }
    #[inline]
    pub fn expr_lit_bool(&mut self, span: Span, value: bool) -> Expr {
        Expr::Lit(Lit::Bool(self.bool(span, value).into()))
    }
    #[inline]
    pub fn expr_lit_null(&mut self, span: Span) -> Expr {
        Expr::Lit(Lit::Null(self.null(span).into()))
    }
    #[inline]
    pub fn expr_lit_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Expr {
        Expr::Lit(Lit::Num(self.number(span, value, raw).into()))
    }
    #[inline]
    pub fn expr_lit_big_int(&mut self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> Expr {
        Expr::Lit(Lit::BigInt(self.big_int(span, value, raw).into()))
    }
    #[inline]
    pub fn expr_lit_regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Expr {
        Expr::Lit(Lit::Regex(self.regex(span, exp, flags).into()))
    }
    #[inline]
    pub fn expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> Expr {
        Expr::Tpl(self.tpl(span, exprs, quasis).into())
    }
    #[inline]
    pub fn expr_tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> Expr {
        Expr::TaggedTpl(self.tagged_tpl(span, tag, tpl).into())
    }
    #[inline]
    pub fn expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> Expr {
        Expr::Arrow(self.arrow_expr(span, params, body, is_async).into())
    }
    #[inline]
    pub fn expr_class_expr(&mut self, span: Span, ident: Option<Ident>, class: Class) -> Expr {
        Expr::Class(self.class_expr(span, ident, class).into())
    }
    #[inline]
    pub fn expr_yield_expr(&mut self, span: Span, arg: Option<Expr>, delegate: bool) -> Expr {
        Expr::Yield(self.yield_expr(span, arg, delegate).into())
    }
    #[inline]
    pub fn expr_meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> Expr {
        Expr::MetaProp(self.meta_prop_expr(span, kind).into())
    }
    #[inline]
    pub fn expr_await_expr(&mut self, span: Span, arg: Expr) -> Expr {
        Expr::Await(self.await_expr(span, arg).into())
    }
    #[inline]
    pub fn expr_paren_expr(&mut self, span: Span, expr: Expr) -> Expr {
        Expr::Paren(self.paren_expr(span, expr).into())
    }
    #[inline]
    pub fn expr_jsx_member_expr(&mut self, span: Span, obj: JSXObject, prop: IdentName) -> Expr {
        Expr::JSXMember(self.jsx_member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn expr_jsx_namespaced_name(&mut self, span: Span, ns: IdentName, name: IdentName) -> Expr {
        Expr::JSXNamespacedName(self.jsx_namespaced_name(span, ns, name).into())
    }
    #[inline]
    pub fn expr_jsx_empty_expr(&mut self, span: Span) -> Expr {
        Expr::JSXEmpty(self.jsx_empty_expr(span).into())
    }
    #[inline]
    pub fn expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> Expr {
        Expr::JSXElement(self.jsx_element(span, opening, children, closing).into())
    }
    #[inline]
    pub fn expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> Expr {
        Expr::JSXFragment(self.jsx_fragment(span, opening, children, closing).into())
    }
    #[inline]
    pub fn expr_private_name(&mut self, span: Span, name: Atom<'a>) -> Expr {
        Expr::PrivateName(self.private_name(span, name).into())
    }
    #[inline]
    pub fn expr_opt_chain_expr(&mut self, span: Span, optional: bool, base: OptChainBase) -> Expr {
        Expr::OptChain(self.opt_chain_expr(span, optional, base).into())
    }
    #[inline]
    pub fn expr_invalid(&mut self, span: Span) -> Expr {
        Expr::Invalid(self.invalid(span).into())
    }
    #[inline]
    pub fn this_expr(&mut self, span: Span) -> ThisExpr {
        ThisExpr(self.add_node(span, NodeKind::ThisExpr, NodePayload::inline(0, 0)))
    }
    #[inline]
    pub fn array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> ArrayLit {
        ArrayLit(self.add_node(
            span,
            NodeKind::ArrayLit,
            NodePayload::inline(0u64 | unsafe { elems.to_extra_data(self).other }, 0u32),
        ))
    }
    #[inline]
    pub fn object_lit(&mut self, span: Span, props: TypedSubRange<PropOrSpread>) -> ObjectLit {
        ObjectLit(self.add_node(
            span,
            NodeKind::ObjectLit,
            NodePayload::inline(0u64 | unsafe { props.to_extra_data(self).other }, 0u32),
        ))
    }
    #[inline]
    pub fn prop_or_spread_spread_element(
        &mut self,
        span: Span,
        dot3_token: Span,
        expr: Expr,
    ) -> PropOrSpread {
        PropOrSpread::Spread(self.spread_element(span, dot3_token, expr).into())
    }
    #[inline]
    pub fn prop_or_spread_prop_ident(&mut self, span: Span, sym: Atom<'a>) -> PropOrSpread {
        PropOrSpread::Prop(Prop::Shorthand(self.ident(span, sym).into()))
    }
    #[inline]
    pub fn prop_or_spread_prop_key_value_prop(
        &mut self,
        span: Span,
        key: PropName,
        value: Expr,
    ) -> PropOrSpread {
        PropOrSpread::Prop(Prop::KeyValue(self.key_value_prop(span, key, value).into()))
    }
    #[inline]
    pub fn prop_or_spread_prop_assign_prop(
        &mut self,
        span: Span,
        key: Ident,
        value: Expr,
    ) -> PropOrSpread {
        PropOrSpread::Prop(Prop::Assign(self.assign_prop(span, key, value).into()))
    }
    #[inline]
    pub fn prop_or_spread_prop_getter_prop(
        &mut self,
        span: Span,
        key: PropName,
        function: Function,
    ) -> PropOrSpread {
        PropOrSpread::Prop(Prop::Getter(self.getter_prop(span, key, function).into()))
    }
    #[inline]
    pub fn prop_or_spread_prop_setter_prop(
        &mut self,
        span: Span,
        key: PropName,
        function: Function,
    ) -> PropOrSpread {
        PropOrSpread::Prop(Prop::Setter(self.setter_prop(span, key, function).into()))
    }
    #[inline]
    pub fn prop_or_spread_prop_method_prop(
        &mut self,
        span: Span,
        key: PropName,
        function: Function,
    ) -> PropOrSpread {
        PropOrSpread::Prop(Prop::Method(self.method_prop(span, key, function).into()))
    }
    #[inline]
    pub fn spread_element(&mut self, span: Span, dot3_token: Span, expr: Expr) -> SpreadElement {
        SpreadElement(self.add_node(
            span,
            NodeKind::SpreadElement,
            NodePayload::inline(
                0u64 | unsafe { dot3_token.to_extra_data(self).other },
                0u32 | expr.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> UnaryExpr {
        UnaryExpr(self.add_node(
            span,
            NodeKind::UnaryExpr,
            NodePayload::inline(
                0u64 | ((op as u32 as u64) << 32usize) | (arg.node_id().index() as u32 as u64),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> UpdateExpr {
        UpdateExpr(self.add_node(
            span,
            NodeKind::UpdateExpr,
            NodePayload::inline(
                0u64 | ((op as u32 as u64) << 32usize)
                    | ((prefix as u32 as u64) << 40usize)
                    | (arg.node_id().index() as u32 as u64),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn bin_expr(&mut self, span: Span, op: BinaryOp, left: Expr, right: Expr) -> BinExpr {
        BinExpr(self.add_node(
            span,
            NodeKind::BinExpr,
            NodePayload::inline(
                0u64 | (left.node_id().index() as u32 as u64)
                    | ((right.node_id().index() as u32 as u64) << 32usize),
                0u32 | op as u32,
            ),
        ))
    }
    #[inline]
    pub fn fn_expr(&mut self, span: Span, ident: Option<Ident>, function: Function) -> FnExpr {
        FnExpr(self.add_node(
            span,
            NodeKind::FnExpr,
            NodePayload::inline(
                0u64 | (crate::OptionalNodeId::from(ident.map(|n| n.node_id())).into_raw() as u64)
                    | ((function.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn class_expr(&mut self, span: Span, ident: Option<Ident>, class: Class) -> ClassExpr {
        ClassExpr(self.add_node(
            span,
            NodeKind::ClassExpr,
            NodePayload::inline(
                0u64 | (crate::OptionalNodeId::from(ident.map(|n| n.node_id())).into_raw() as u64)
                    | ((class.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> AssignExpr {
        AssignExpr(self.add_node(
            span,
            NodeKind::AssignExpr,
            NodePayload::inline(
                0u64 | (left.node_id().index() as u32 as u64)
                    | ((right.node_id().index() as u32 as u64) << 32usize),
                0u32 | op as u32,
            ),
        ))
    }
    #[inline]
    pub fn member_expr(&mut self, span: Span, obj: Expr, prop: MemberProp) -> MemberExpr {
        MemberExpr(self.add_node(
            span,
            NodeKind::MemberExpr,
            NodePayload::inline(
                0u64 | (obj.node_id().index() as u32 as u64)
                    | ((prop.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn member_prop_ident_name(&mut self, span: Span, sym: Atom<'a>) -> MemberProp {
        MemberProp::Ident(self.ident_name(span, sym).into())
    }
    #[inline]
    pub fn member_prop_private_name(&mut self, span: Span, name: Atom<'a>) -> MemberProp {
        MemberProp::PrivateName(self.private_name(span, name).into())
    }
    #[inline]
    pub fn member_prop_computed_prop_name(&mut self, span: Span, expr: Expr) -> MemberProp {
        MemberProp::Computed(self.computed_prop_name(span, expr).into())
    }
    #[inline]
    pub fn super_prop_expr(&mut self, span: Span, obj: Super, prop: SuperProp) -> SuperPropExpr {
        SuperPropExpr(self.add_node(
            span,
            NodeKind::SuperPropExpr,
            NodePayload::inline(
                0u64 | (obj.node_id().index() as u32 as u64)
                    | ((prop.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn super_prop_ident_name(&mut self, span: Span, sym: Atom<'a>) -> SuperProp {
        SuperProp::Ident(self.ident_name(span, sym).into())
    }
    #[inline]
    pub fn super_prop_computed_prop_name(&mut self, span: Span, expr: Expr) -> SuperProp {
        SuperProp::Computed(self.computed_prop_name(span, expr).into())
    }
    #[inline]
    pub fn cond_expr(&mut self, span: Span, test: Expr, cons: Expr, alt: Expr) -> CondExpr {
        CondExpr(self.add_node(
            span,
            NodeKind::CondExpr,
            NodePayload::inline(
                0u64 | (test.node_id().index() as u32 as u64)
                    | ((cons.node_id().index() as u32 as u64) << 32usize),
                0u32 | alt.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> ImportExpr {
        let _f0 = self.add_extra(source.to_extra_data(self));
        let _f1 = self.add_extra(options.to_extra_data(self));
        let _f2 = self.add_extra(phase.to_extra_data(self));
        ImportExpr(self.add_node(
            span,
            NodeKind::ImportExpr,
            NodePayload::partial(0u64 | unsafe { import_span.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> CallExpr {
        CallExpr(self.add_node(
            span,
            NodeKind::CallExpr,
            NodePayload::inline(
                0u64 | unsafe { args.to_extra_data(self).other },
                0u32 | callee.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> NewExpr {
        NewExpr(self.add_node(
            span,
            NodeKind::NewExpr,
            NodePayload::inline(
                0u64 | unsafe { args.to_extra_data(self).other },
                0u32 | callee.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> SeqExpr {
        SeqExpr(self.add_node(
            span,
            NodeKind::SeqExpr,
            NodePayload::inline(0u64 | unsafe { exprs.to_extra_data(self).other }, 0u32),
        ))
    }
    #[inline]
    pub fn arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> ArrowExpr {
        let _f0 = self.add_extra(body.to_extra_data(self));
        let _f1 = self.add_extra(is_async.to_extra_data(self));
        ArrowExpr(self.add_node(
            span,
            NodeKind::ArrowExpr,
            NodePayload::partial(0u64 | unsafe { params.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn yield_expr(&mut self, span: Span, arg: Option<Expr>, delegate: bool) -> YieldExpr {
        YieldExpr(self.add_node(
            span,
            NodeKind::YieldExpr,
            NodePayload::inline(
                0u64 | (crate::OptionalNodeId::from(arg.map(|n| n.node_id())).into_raw() as u64)
                    | ((delegate as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> MetaPropExpr {
        MetaPropExpr(self.add_node(
            span,
            NodeKind::MetaPropExpr,
            NodePayload::inline(0u64 | (kind as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn await_expr(&mut self, span: Span, arg: Expr) -> AwaitExpr {
        AwaitExpr(self.add_node(
            span,
            NodeKind::AwaitExpr,
            NodePayload::inline(0u64 | (arg.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> Tpl {
        let _f0 = self.add_extra(quasis.to_extra_data(self));
        Tpl(self.add_node(
            span,
            NodeKind::Tpl,
            NodePayload::partial(0u64 | unsafe { exprs.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> TaggedTpl {
        TaggedTpl(self.add_node(
            span,
            NodeKind::TaggedTpl,
            NodePayload::inline(
                0u64 | (tag.node_id().index() as u32 as u64)
                    | ((tpl.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn tpl_element(
        &mut self,
        span: Span,
        tail: bool,
        cooked: Option<Wtf8Atom<'a>>,
        raw: Atom<'a>,
    ) -> TplElement {
        let _f0 = self.add_extra(tail.to_extra_data(self));
        let _f1 = self.add_extra(raw.to_extra_data(self));
        TplElement(self.add_node(
            span,
            NodeKind::TplElement,
            NodePayload::partial(0u64 | unsafe { cooked.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn paren_expr(&mut self, span: Span, expr: Expr) -> ParenExpr {
        ParenExpr(self.add_node(
            span,
            NodeKind::ParenExpr,
            NodePayload::inline(0u64 | (expr.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn callee_super(&mut self, span: Span) -> Callee {
        Callee::Super(self.super_(span).into())
    }
    #[inline]
    pub fn callee_expr_this_expr(&mut self, span: Span) -> Callee {
        Callee::Expr(Expr::This(self.this_expr(span).into()))
    }
    #[inline]
    pub fn callee_expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> Callee {
        Callee::Expr(Expr::Array(self.array_lit(span, elems).into()))
    }
    #[inline]
    pub fn callee_expr_object_lit(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
    ) -> Callee {
        Callee::Expr(Expr::Object(self.object_lit(span, props).into()))
    }
    #[inline]
    pub fn callee_expr_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> Callee {
        Callee::Expr(Expr::Fn(self.fn_expr(span, ident, function).into()))
    }
    #[inline]
    pub fn callee_expr_unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> Callee {
        Callee::Expr(Expr::Unary(self.unary_expr(span, op, arg).into()))
    }
    #[inline]
    pub fn callee_expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> Callee {
        Callee::Expr(Expr::Update(self.update_expr(span, op, prefix, arg).into()))
    }
    #[inline]
    pub fn callee_expr_bin_expr(
        &mut self,
        span: Span,
        op: BinaryOp,
        left: Expr,
        right: Expr,
    ) -> Callee {
        Callee::Expr(Expr::Bin(self.bin_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn callee_expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> Callee {
        Callee::Expr(Expr::Assign(self.assign_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn callee_expr_member_expr(&mut self, span: Span, obj: Expr, prop: MemberProp) -> Callee {
        Callee::Expr(Expr::Member(self.member_expr(span, obj, prop).into()))
    }
    #[inline]
    pub fn callee_expr_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> Callee {
        Callee::Expr(Expr::SuperProp(
            self.super_prop_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn callee_expr_cond_expr(
        &mut self,
        span: Span,
        test: Expr,
        cons: Expr,
        alt: Expr,
    ) -> Callee {
        Callee::Expr(Expr::Cond(self.cond_expr(span, test, cons, alt).into()))
    }
    #[inline]
    pub fn callee_expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> Callee {
        Callee::Expr(Expr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        ))
    }
    #[inline]
    pub fn callee_expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Callee {
        Callee::Expr(Expr::Call(self.call_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn callee_expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Callee {
        Callee::Expr(Expr::New(self.new_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn callee_expr_seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> Callee {
        Callee::Expr(Expr::Seq(self.seq_expr(span, exprs).into()))
    }
    #[inline]
    pub fn callee_expr_ident(&mut self, span: Span, sym: Atom<'a>) -> Callee {
        Callee::Expr(Expr::Ident(self.ident(span, sym).into()))
    }
    #[inline]
    pub fn callee_expr_lit_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Callee {
        Callee::Expr(Expr::Lit(Lit::Str(self.str(span, value, raw).into())))
    }
    #[inline]
    pub fn callee_expr_lit_bool(&mut self, span: Span, value: bool) -> Callee {
        Callee::Expr(Expr::Lit(Lit::Bool(self.bool(span, value).into())))
    }
    #[inline]
    pub fn callee_expr_lit_null(&mut self, span: Span) -> Callee {
        Callee::Expr(Expr::Lit(Lit::Null(self.null(span).into())))
    }
    #[inline]
    pub fn callee_expr_lit_number(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> Callee {
        Callee::Expr(Expr::Lit(Lit::Num(self.number(span, value, raw).into())))
    }
    #[inline]
    pub fn callee_expr_lit_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Callee {
        Callee::Expr(Expr::Lit(Lit::BigInt(
            self.big_int(span, value, raw).into(),
        )))
    }
    #[inline]
    pub fn callee_expr_lit_regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Callee {
        Callee::Expr(Expr::Lit(Lit::Regex(self.regex(span, exp, flags).into())))
    }
    #[inline]
    pub fn callee_expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> Callee {
        Callee::Expr(Expr::Tpl(self.tpl(span, exprs, quasis).into()))
    }
    #[inline]
    pub fn callee_expr_tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> Callee {
        Callee::Expr(Expr::TaggedTpl(self.tagged_tpl(span, tag, tpl).into()))
    }
    #[inline]
    pub fn callee_expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> Callee {
        Callee::Expr(Expr::Arrow(
            self.arrow_expr(span, params, body, is_async).into(),
        ))
    }
    #[inline]
    pub fn callee_expr_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> Callee {
        Callee::Expr(Expr::Class(self.class_expr(span, ident, class).into()))
    }
    #[inline]
    pub fn callee_expr_yield_expr(
        &mut self,
        span: Span,
        arg: Option<Expr>,
        delegate: bool,
    ) -> Callee {
        Callee::Expr(Expr::Yield(self.yield_expr(span, arg, delegate).into()))
    }
    #[inline]
    pub fn callee_expr_meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> Callee {
        Callee::Expr(Expr::MetaProp(self.meta_prop_expr(span, kind).into()))
    }
    #[inline]
    pub fn callee_expr_await_expr(&mut self, span: Span, arg: Expr) -> Callee {
        Callee::Expr(Expr::Await(self.await_expr(span, arg).into()))
    }
    #[inline]
    pub fn callee_expr_paren_expr(&mut self, span: Span, expr: Expr) -> Callee {
        Callee::Expr(Expr::Paren(self.paren_expr(span, expr).into()))
    }
    #[inline]
    pub fn callee_expr_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> Callee {
        Callee::Expr(Expr::JSXMember(
            self.jsx_member_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn callee_expr_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> Callee {
        Callee::Expr(Expr::JSXNamespacedName(
            self.jsx_namespaced_name(span, ns, name).into(),
        ))
    }
    #[inline]
    pub fn callee_expr_jsx_empty_expr(&mut self, span: Span) -> Callee {
        Callee::Expr(Expr::JSXEmpty(self.jsx_empty_expr(span).into()))
    }
    #[inline]
    pub fn callee_expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> Callee {
        Callee::Expr(Expr::JSXElement(
            self.jsx_element(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn callee_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> Callee {
        Callee::Expr(Expr::JSXFragment(
            self.jsx_fragment(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn callee_expr_private_name(&mut self, span: Span, name: Atom<'a>) -> Callee {
        Callee::Expr(Expr::PrivateName(self.private_name(span, name).into()))
    }
    #[inline]
    pub fn callee_expr_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> Callee {
        Callee::Expr(Expr::OptChain(
            self.opt_chain_expr(span, optional, base).into(),
        ))
    }
    #[inline]
    pub fn callee_expr_invalid(&mut self, span: Span) -> Callee {
        Callee::Expr(Expr::Invalid(self.invalid(span).into()))
    }
    #[inline]
    pub fn super_(&mut self, span: Span) -> Super {
        Super(self.add_node(span, NodeKind::Super, NodePayload::inline(0, 0)))
    }
    #[inline]
    pub fn expr_or_spread(&mut self, span: Span, spread: Option<Span>, expr: Expr) -> ExprOrSpread {
        ExprOrSpread(self.add_node(
            span,
            NodeKind::ExprOrSpread,
            NodePayload::inline(
                0u64 | unsafe { spread.to_extra_data(self).other },
                0u32 | expr.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_block_stmt(
        &mut self,
        span: Span,
        stmts: TypedSubRange<Stmt>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::BlockStmt(self.block_stmt(span, stmts).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_this_expr(&mut self, span: Span) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::This(self.this_expr(span).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Array(self.array_lit(span, elems).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_object_lit(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Object(self.object_lit(span, props).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Fn(self.fn_expr(span, ident, function).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_unary_expr(
        &mut self,
        span: Span,
        op: UnaryOp,
        arg: Expr,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Unary(self.unary_expr(span, op, arg).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Update(self.update_expr(span, op, prefix, arg).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_bin_expr(
        &mut self,
        span: Span,
        op: BinaryOp,
        left: Expr,
        right: Expr,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Bin(self.bin_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Assign(self.assign_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Member(self.member_expr(span, obj, prop).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::SuperProp(
            self.super_prop_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_cond_expr(
        &mut self,
        span: Span,
        test: Expr,
        cons: Expr,
        alt: Expr,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Cond(self.cond_expr(span, test, cons, alt).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Call(self.call_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::New(self.new_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_seq_expr(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Seq(self.seq_expr(span, exprs).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_ident(&mut self, span: Span, sym: Atom<'a>) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Ident(self.ident(span, sym).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Lit(Lit::Str(self.str(span, value, raw).into())))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_bool(&mut self, span: Span, value: bool) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Lit(Lit::Bool(self.bool(span, value).into())))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_null(&mut self, span: Span) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Lit(Lit::Null(self.null(span).into())))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_number(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Lit(Lit::Num(self.number(span, value, raw).into())))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Lit(Lit::BigInt(
            self.big_int(span, value, raw).into(),
        )))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_lit_regex(
        &mut self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Lit(Lit::Regex(self.regex(span, exp, flags).into())))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Tpl(self.tpl(span, exprs, quasis).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_tagged_tpl(
        &mut self,
        span: Span,
        tag: Expr,
        tpl: Tpl,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::TaggedTpl(self.tagged_tpl(span, tag, tpl).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Arrow(
            self.arrow_expr(span, params, body, is_async).into(),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Class(self.class_expr(span, ident, class).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_yield_expr(
        &mut self,
        span: Span,
        arg: Option<Expr>,
        delegate: bool,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Yield(self.yield_expr(span, arg, delegate).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_meta_prop_expr(
        &mut self,
        span: Span,
        kind: MetaPropKind,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::MetaProp(self.meta_prop_expr(span, kind).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_await_expr(&mut self, span: Span, arg: Expr) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Await(self.await_expr(span, arg).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_paren_expr(
        &mut self,
        span: Span,
        expr: Expr,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Paren(self.paren_expr(span, expr).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::JSXMember(
            self.jsx_member_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::JSXNamespacedName(
            self.jsx_namespaced_name(span, ns, name).into(),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_empty_expr(&mut self, span: Span) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::JSXEmpty(self.jsx_empty_expr(span).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::JSXElement(
            self.jsx_element(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::JSXFragment(
            self.jsx_fragment(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_private_name(
        &mut self,
        span: Span,
        name: Atom<'a>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::PrivateName(self.private_name(span, name).into()))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::OptChain(
            self.opt_chain_expr(span, optional, base).into(),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_expr_invalid(&mut self, span: Span) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Expr(Expr::Invalid(self.invalid(span).into()))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_binding_ident(
        &mut self,
        span: Span,
        id: Ident,
    ) -> AssignTarget {
        AssignTarget::Simple(SimpleAssignTarget::Ident(
            self.binding_ident(span, id).into(),
        ))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> AssignTarget {
        AssignTarget::Simple(SimpleAssignTarget::Member(
            self.member_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> AssignTarget {
        AssignTarget::Simple(SimpleAssignTarget::SuperProp(
            self.super_prop_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_paren_expr(
        &mut self,
        span: Span,
        expr: Expr,
    ) -> AssignTarget {
        AssignTarget::Simple(SimpleAssignTarget::Paren(
            self.paren_expr(span, expr).into(),
        ))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> AssignTarget {
        AssignTarget::Simple(SimpleAssignTarget::OptChain(
            self.opt_chain_expr(span, optional, base).into(),
        ))
    }
    #[inline]
    pub fn assign_target_simple_assign_target_invalid(&mut self, span: Span) -> AssignTarget {
        AssignTarget::Simple(SimpleAssignTarget::Invalid(self.invalid(span).into()))
    }
    #[inline]
    pub fn assign_target_assign_target_pat_array_pat(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<Pat>>,
        optional: bool,
    ) -> AssignTarget {
        AssignTarget::Pat(AssignTargetPat::Array(
            self.array_pat(span, elems, optional).into(),
        ))
    }
    #[inline]
    pub fn assign_target_assign_target_pat_object_pat(
        &mut self,
        span: Span,
        props: TypedSubRange<ObjectPatProp>,
        optional: bool,
    ) -> AssignTarget {
        AssignTarget::Pat(AssignTargetPat::Object(
            self.object_pat(span, props, optional).into(),
        ))
    }
    #[inline]
    pub fn assign_target_assign_target_pat_invalid(&mut self, span: Span) -> AssignTarget {
        AssignTarget::Pat(AssignTargetPat::Invalid(self.invalid(span).into()))
    }
    #[inline]
    pub fn assign_target_pat_array_pat(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<Pat>>,
        optional: bool,
    ) -> AssignTargetPat {
        AssignTargetPat::Array(self.array_pat(span, elems, optional).into())
    }
    #[inline]
    pub fn assign_target_pat_object_pat(
        &mut self,
        span: Span,
        props: TypedSubRange<ObjectPatProp>,
        optional: bool,
    ) -> AssignTargetPat {
        AssignTargetPat::Object(self.object_pat(span, props, optional).into())
    }
    #[inline]
    pub fn assign_target_pat_invalid(&mut self, span: Span) -> AssignTargetPat {
        AssignTargetPat::Invalid(self.invalid(span).into())
    }
    #[inline]
    pub fn simple_assign_target_binding_ident(
        &mut self,
        span: Span,
        id: Ident,
    ) -> SimpleAssignTarget {
        SimpleAssignTarget::Ident(self.binding_ident(span, id).into())
    }
    #[inline]
    pub fn simple_assign_target_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> SimpleAssignTarget {
        SimpleAssignTarget::Member(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn simple_assign_target_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> SimpleAssignTarget {
        SimpleAssignTarget::SuperProp(self.super_prop_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn simple_assign_target_paren_expr(
        &mut self,
        span: Span,
        expr: Expr,
    ) -> SimpleAssignTarget {
        SimpleAssignTarget::Paren(self.paren_expr(span, expr).into())
    }
    #[inline]
    pub fn simple_assign_target_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> SimpleAssignTarget {
        SimpleAssignTarget::OptChain(self.opt_chain_expr(span, optional, base).into())
    }
    #[inline]
    pub fn simple_assign_target_invalid(&mut self, span: Span) -> SimpleAssignTarget {
        SimpleAssignTarget::Invalid(self.invalid(span).into())
    }
    #[inline]
    pub fn opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> OptChainExpr {
        OptChainExpr(self.add_node(
            span,
            NodeKind::OptChainExpr,
            NodePayload::inline(
                0u64 | ((optional as u32 as u64) << 32usize)
                    | (base.node_id().index() as u32 as u64),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn opt_chain_base_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> OptChainBase {
        OptChainBase::Member(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn opt_chain_base_opt_call(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> OptChainBase {
        OptChainBase::Call(self.opt_call(span, callee, args).into())
    }
    #[inline]
    pub fn opt_call(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> OptCall {
        OptCall(self.add_node(
            span,
            NodeKind::OptCall,
            NodePayload::inline(
                0u64 | unsafe { args.to_extra_data(self).other },
                0u32 | callee.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn invalid(&mut self, span: Span) -> Invalid {
        Invalid(self.add_node(span, NodeKind::Invalid, NodePayload::inline(0, 0)))
    }
    #[inline]
    pub fn function(
        &mut self,
        span: Span,
        params: TypedSubRange<Param>,
        decorators: TypedSubRange<Decorator>,
        body: BlockStmt,
        is_generator: bool,
        is_async: bool,
    ) -> Function {
        let _f0 = self.add_extra(decorators.to_extra_data(self));
        let _f1 = self.add_extra(body.to_extra_data(self));
        let _f2 = self.add_extra(is_generator.to_extra_data(self));
        let _f3 = self.add_extra(is_async.to_extra_data(self));
        Function(self.add_node(
            span,
            NodeKind::Function,
            NodePayload::partial(0u64 | unsafe { params.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn param(&mut self, span: Span, decorators: TypedSubRange<Decorator>, pat: Pat) -> Param {
        Param(self.add_node(
            span,
            NodeKind::Param,
            NodePayload::inline(
                0u64 | unsafe { decorators.to_extra_data(self).other },
                0u32 | pat.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn param_or_ts_param_prop_param(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        pat: Pat,
    ) -> ParamOrTsParamProp {
        ParamOrTsParamProp::Param(self.param(span, decorators, pat).into())
    }
    #[inline]
    pub fn class(
        &mut self,
        span: Span,
        decorators: TypedSubRange<Decorator>,
        body: TypedSubRange<ClassMember>,
        super_class: Option<Expr>,
        is_abstract: bool,
    ) -> Class {
        let _f0 = self.add_extra(body.to_extra_data(self));
        let _f1 = self.add_extra(super_class.to_extra_data(self));
        let _f2 = self.add_extra(is_abstract.to_extra_data(self));
        Class(self.add_node(
            span,
            NodeKind::Class,
            NodePayload::partial(0u64 | unsafe { decorators.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn class_member_constructor(
        &mut self,
        span: Span,
        key: PropName,
        params: TypedSubRange<ParamOrTsParamProp>,
        body: Option<BlockStmt>,
    ) -> ClassMember {
        ClassMember::Constructor(self.constructor(span, key, params, body).into())
    }
    #[inline]
    pub fn class_member_class_method(
        &mut self,
        span: Span,
        key: PropName,
        function: Function,
        kind: MethodKind,
        is_static: bool,
    ) -> ClassMember {
        ClassMember::Method(
            self.class_method(span, key, function, kind, is_static)
                .into(),
        )
    }
    #[inline]
    pub fn class_member_private_method(
        &mut self,
        span: Span,
        key: PrivateName,
        function: Function,
        kind: MethodKind,
        is_static: bool,
    ) -> ClassMember {
        ClassMember::PrivateMethod(
            self.private_method(span, key, function, kind, is_static)
                .into(),
        )
    }
    #[inline]
    pub fn class_member_class_prop(
        &mut self,
        span: Span,
        key: PropName,
        value: Option<Expr>,
        is_static: bool,
        decorators: TypedSubRange<Decorator>,
    ) -> ClassMember {
        ClassMember::ClassProp(
            self.class_prop(span, key, value, is_static, decorators)
                .into(),
        )
    }
    #[inline]
    pub fn class_member_private_prop(
        &mut self,
        span: Span,
        key: PrivateName,
        value: Option<Expr>,
        is_static: bool,
        decorators: TypedSubRange<Decorator>,
    ) -> ClassMember {
        ClassMember::PrivateProp(
            self.private_prop(span, key, value, is_static, decorators)
                .into(),
        )
    }
    #[inline]
    pub fn class_member_empty_stmt(&mut self, span: Span) -> ClassMember {
        ClassMember::Empty(self.empty_stmt(span).into())
    }
    #[inline]
    pub fn class_member_static_block(&mut self, span: Span, body: BlockStmt) -> ClassMember {
        ClassMember::StaticBlock(self.static_block(span, body).into())
    }
    #[inline]
    pub fn class_member_auto_accessor(
        &mut self,
        span: Span,
        key: Key,
        value: Option<Expr>,
        is_static: bool,
        decorators: TypedSubRange<Decorator>,
    ) -> ClassMember {
        ClassMember::AutoAccessor(
            self.auto_accessor(span, key, value, is_static, decorators)
                .into(),
        )
    }
    #[inline]
    pub fn class_prop(
        &mut self,
        span: Span,
        key: PropName,
        value: Option<Expr>,
        is_static: bool,
        decorators: TypedSubRange<Decorator>,
    ) -> ClassProp {
        let _f0 = self.add_extra(key.to_extra_data(self));
        let _f1 = self.add_extra(value.to_extra_data(self));
        let _f2 = self.add_extra(is_static.to_extra_data(self));
        ClassProp(self.add_node(
            span,
            NodeKind::ClassProp,
            NodePayload::partial(0u64 | unsafe { decorators.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn private_prop(
        &mut self,
        span: Span,
        key: PrivateName,
        value: Option<Expr>,
        is_static: bool,
        decorators: TypedSubRange<Decorator>,
    ) -> PrivateProp {
        let _f0 = self.add_extra(key.to_extra_data(self));
        let _f1 = self.add_extra(value.to_extra_data(self));
        let _f2 = self.add_extra(is_static.to_extra_data(self));
        PrivateProp(self.add_node(
            span,
            NodeKind::PrivateProp,
            NodePayload::partial(0u64 | unsafe { decorators.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn class_method(
        &mut self,
        span: Span,
        key: PropName,
        function: Function,
        kind: MethodKind,
        is_static: bool,
    ) -> ClassMethod {
        ClassMethod(self.add_node(
            span,
            NodeKind::ClassMethod,
            NodePayload::inline(
                0u64 | (key.node_id().index() as u32 as u64)
                    | ((function.node_id().index() as u32 as u64) << 32usize),
                0u32 | kind as u32 | ((is_static as u32) << 8usize),
            ),
        ))
    }
    #[inline]
    pub fn private_method(
        &mut self,
        span: Span,
        key: PrivateName,
        function: Function,
        kind: MethodKind,
        is_static: bool,
    ) -> PrivateMethod {
        PrivateMethod(self.add_node(
            span,
            NodeKind::PrivateMethod,
            NodePayload::inline(
                0u64 | (key.node_id().index() as u32 as u64)
                    | ((function.node_id().index() as u32 as u64) << 32usize),
                0u32 | kind as u32 | ((is_static as u32) << 8usize),
            ),
        ))
    }
    #[inline]
    pub fn constructor(
        &mut self,
        span: Span,
        key: PropName,
        params: TypedSubRange<ParamOrTsParamProp>,
        body: Option<BlockStmt>,
    ) -> Constructor {
        let _f0 = self.add_extra(key.to_extra_data(self));
        let _f1 = self.add_extra(body.to_extra_data(self));
        Constructor(self.add_node(
            span,
            NodeKind::Constructor,
            NodePayload::partial(0u64 | unsafe { params.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn decorator(&mut self, span: Span, expr: Expr) -> Decorator {
        Decorator(self.add_node(
            span,
            NodeKind::Decorator,
            NodePayload::inline(0u64 | (expr.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn static_block(&mut self, span: Span, body: BlockStmt) -> StaticBlock {
        StaticBlock(self.add_node(
            span,
            NodeKind::StaticBlock,
            NodePayload::inline(0u64 | (body.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn key_private_name(&mut self, span: Span, name: Atom<'a>) -> Key {
        Key::Private(self.private_name(span, name).into())
    }
    #[inline]
    pub fn key_prop_name_ident_name(&mut self, span: Span, sym: Atom<'a>) -> Key {
        Key::Public(PropName::Ident(self.ident_name(span, sym).into()))
    }
    #[inline]
    pub fn key_prop_name_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Key {
        Key::Public(PropName::Str(self.str(span, value, raw).into()))
    }
    #[inline]
    pub fn key_prop_name_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Key {
        Key::Public(PropName::Num(self.number(span, value, raw).into()))
    }
    #[inline]
    pub fn key_prop_name_computed_prop_name(&mut self, span: Span, expr: Expr) -> Key {
        Key::Public(PropName::Computed(
            self.computed_prop_name(span, expr).into(),
        ))
    }
    #[inline]
    pub fn key_prop_name_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Key {
        Key::Public(PropName::BigInt(self.big_int(span, value, raw).into()))
    }
    #[inline]
    pub fn auto_accessor(
        &mut self,
        span: Span,
        key: Key,
        value: Option<Expr>,
        is_static: bool,
        decorators: TypedSubRange<Decorator>,
    ) -> AutoAccessor {
        let _f0 = self.add_extra(key.to_extra_data(self));
        let _f1 = self.add_extra(value.to_extra_data(self));
        let _f2 = self.add_extra(is_static.to_extra_data(self));
        AutoAccessor(self.add_node(
            span,
            NodeKind::AutoAccessor,
            NodePayload::partial(0u64 | unsafe { decorators.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn prop_ident(&mut self, span: Span, sym: Atom<'a>) -> Prop {
        Prop::Shorthand(self.ident(span, sym).into())
    }
    #[inline]
    pub fn prop_key_value_prop(&mut self, span: Span, key: PropName, value: Expr) -> Prop {
        Prop::KeyValue(self.key_value_prop(span, key, value).into())
    }
    #[inline]
    pub fn prop_assign_prop(&mut self, span: Span, key: Ident, value: Expr) -> Prop {
        Prop::Assign(self.assign_prop(span, key, value).into())
    }
    #[inline]
    pub fn prop_getter_prop(&mut self, span: Span, key: PropName, function: Function) -> Prop {
        Prop::Getter(self.getter_prop(span, key, function).into())
    }
    #[inline]
    pub fn prop_setter_prop(&mut self, span: Span, key: PropName, function: Function) -> Prop {
        Prop::Setter(self.setter_prop(span, key, function).into())
    }
    #[inline]
    pub fn prop_method_prop(&mut self, span: Span, key: PropName, function: Function) -> Prop {
        Prop::Method(self.method_prop(span, key, function).into())
    }
    #[inline]
    pub fn key_value_prop(&mut self, span: Span, key: PropName, value: Expr) -> KeyValueProp {
        KeyValueProp(self.add_node(
            span,
            NodeKind::KeyValueProp,
            NodePayload::inline(
                0u64 | (key.node_id().index() as u32 as u64)
                    | ((value.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn assign_prop(&mut self, span: Span, key: Ident, value: Expr) -> AssignProp {
        AssignProp(self.add_node(
            span,
            NodeKind::AssignProp,
            NodePayload::inline(
                0u64 | (key.node_id().index() as u32 as u64)
                    | ((value.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn getter_prop(&mut self, span: Span, key: PropName, function: Function) -> GetterProp {
        GetterProp(self.add_node(
            span,
            NodeKind::GetterProp,
            NodePayload::inline(
                0u64 | (key.node_id().index() as u32 as u64)
                    | ((function.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn setter_prop(&mut self, span: Span, key: PropName, function: Function) -> SetterProp {
        SetterProp(self.add_node(
            span,
            NodeKind::SetterProp,
            NodePayload::inline(
                0u64 | (key.node_id().index() as u32 as u64)
                    | ((function.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn method_prop(&mut self, span: Span, key: PropName, function: Function) -> MethodProp {
        MethodProp(self.add_node(
            span,
            NodeKind::MethodProp,
            NodePayload::inline(
                0u64 | (key.node_id().index() as u32 as u64)
                    | ((function.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn prop_name_ident_name(&mut self, span: Span, sym: Atom<'a>) -> PropName {
        PropName::Ident(self.ident_name(span, sym).into())
    }
    #[inline]
    pub fn prop_name_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> PropName {
        PropName::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn prop_name_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> PropName {
        PropName::Num(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn prop_name_computed_prop_name(&mut self, span: Span, expr: Expr) -> PropName {
        PropName::Computed(self.computed_prop_name(span, expr).into())
    }
    #[inline]
    pub fn prop_name_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> PropName {
        PropName::BigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn computed_prop_name(&mut self, span: Span, expr: Expr) -> ComputedPropName {
        ComputedPropName(self.add_node(
            span,
            NodeKind::ComputedPropName,
            NodePayload::inline(0u64 | (expr.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn pat_binding_ident(&mut self, span: Span, id: Ident) -> Pat {
        Pat::Ident(self.binding_ident(span, id).into())
    }
    #[inline]
    pub fn pat_array_pat(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<Pat>>,
        optional: bool,
    ) -> Pat {
        Pat::Array(self.array_pat(span, elems, optional).into())
    }
    #[inline]
    pub fn pat_rest_pat(&mut self, span: Span, dot3_token: Span, arg: Pat) -> Pat {
        Pat::Rest(self.rest_pat(span, dot3_token, arg).into())
    }
    #[inline]
    pub fn pat_object_pat(
        &mut self,
        span: Span,
        props: TypedSubRange<ObjectPatProp>,
        optional: bool,
    ) -> Pat {
        Pat::Object(self.object_pat(span, props, optional).into())
    }
    #[inline]
    pub fn pat_assign_pat(&mut self, span: Span, left: Pat, right: Expr) -> Pat {
        Pat::Assign(self.assign_pat(span, left, right).into())
    }
    #[inline]
    pub fn pat_invalid(&mut self, span: Span) -> Pat {
        Pat::Invalid(self.invalid(span).into())
    }
    #[inline]
    pub fn pat_expr_this_expr(&mut self, span: Span) -> Pat {
        Pat::Expr(Expr::This(self.this_expr(span).into()))
    }
    #[inline]
    pub fn pat_expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> Pat {
        Pat::Expr(Expr::Array(self.array_lit(span, elems).into()))
    }
    #[inline]
    pub fn pat_expr_object_lit(&mut self, span: Span, props: TypedSubRange<PropOrSpread>) -> Pat {
        Pat::Expr(Expr::Object(self.object_lit(span, props).into()))
    }
    #[inline]
    pub fn pat_expr_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> Pat {
        Pat::Expr(Expr::Fn(self.fn_expr(span, ident, function).into()))
    }
    #[inline]
    pub fn pat_expr_unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> Pat {
        Pat::Expr(Expr::Unary(self.unary_expr(span, op, arg).into()))
    }
    #[inline]
    pub fn pat_expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> Pat {
        Pat::Expr(Expr::Update(self.update_expr(span, op, prefix, arg).into()))
    }
    #[inline]
    pub fn pat_expr_bin_expr(&mut self, span: Span, op: BinaryOp, left: Expr, right: Expr) -> Pat {
        Pat::Expr(Expr::Bin(self.bin_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn pat_expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> Pat {
        Pat::Expr(Expr::Assign(self.assign_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn pat_expr_member_expr(&mut self, span: Span, obj: Expr, prop: MemberProp) -> Pat {
        Pat::Expr(Expr::Member(self.member_expr(span, obj, prop).into()))
    }
    #[inline]
    pub fn pat_expr_super_prop_expr(&mut self, span: Span, obj: Super, prop: SuperProp) -> Pat {
        Pat::Expr(Expr::SuperProp(
            self.super_prop_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn pat_expr_cond_expr(&mut self, span: Span, test: Expr, cons: Expr, alt: Expr) -> Pat {
        Pat::Expr(Expr::Cond(self.cond_expr(span, test, cons, alt).into()))
    }
    #[inline]
    pub fn pat_expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> Pat {
        Pat::Expr(Expr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        ))
    }
    #[inline]
    pub fn pat_expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Pat {
        Pat::Expr(Expr::Call(self.call_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn pat_expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Pat {
        Pat::Expr(Expr::New(self.new_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn pat_expr_seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> Pat {
        Pat::Expr(Expr::Seq(self.seq_expr(span, exprs).into()))
    }
    #[inline]
    pub fn pat_expr_ident(&mut self, span: Span, sym: Atom<'a>) -> Pat {
        Pat::Expr(Expr::Ident(self.ident(span, sym).into()))
    }
    #[inline]
    pub fn pat_expr_lit_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Pat {
        Pat::Expr(Expr::Lit(Lit::Str(self.str(span, value, raw).into())))
    }
    #[inline]
    pub fn pat_expr_lit_bool(&mut self, span: Span, value: bool) -> Pat {
        Pat::Expr(Expr::Lit(Lit::Bool(self.bool(span, value).into())))
    }
    #[inline]
    pub fn pat_expr_lit_null(&mut self, span: Span) -> Pat {
        Pat::Expr(Expr::Lit(Lit::Null(self.null(span).into())))
    }
    #[inline]
    pub fn pat_expr_lit_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Pat {
        Pat::Expr(Expr::Lit(Lit::Num(self.number(span, value, raw).into())))
    }
    #[inline]
    pub fn pat_expr_lit_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> Pat {
        Pat::Expr(Expr::Lit(Lit::BigInt(
            self.big_int(span, value, raw).into(),
        )))
    }
    #[inline]
    pub fn pat_expr_lit_regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Pat {
        Pat::Expr(Expr::Lit(Lit::Regex(self.regex(span, exp, flags).into())))
    }
    #[inline]
    pub fn pat_expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> Pat {
        Pat::Expr(Expr::Tpl(self.tpl(span, exprs, quasis).into()))
    }
    #[inline]
    pub fn pat_expr_tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> Pat {
        Pat::Expr(Expr::TaggedTpl(self.tagged_tpl(span, tag, tpl).into()))
    }
    #[inline]
    pub fn pat_expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> Pat {
        Pat::Expr(Expr::Arrow(
            self.arrow_expr(span, params, body, is_async).into(),
        ))
    }
    #[inline]
    pub fn pat_expr_class_expr(&mut self, span: Span, ident: Option<Ident>, class: Class) -> Pat {
        Pat::Expr(Expr::Class(self.class_expr(span, ident, class).into()))
    }
    #[inline]
    pub fn pat_expr_yield_expr(&mut self, span: Span, arg: Option<Expr>, delegate: bool) -> Pat {
        Pat::Expr(Expr::Yield(self.yield_expr(span, arg, delegate).into()))
    }
    #[inline]
    pub fn pat_expr_meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> Pat {
        Pat::Expr(Expr::MetaProp(self.meta_prop_expr(span, kind).into()))
    }
    #[inline]
    pub fn pat_expr_await_expr(&mut self, span: Span, arg: Expr) -> Pat {
        Pat::Expr(Expr::Await(self.await_expr(span, arg).into()))
    }
    #[inline]
    pub fn pat_expr_paren_expr(&mut self, span: Span, expr: Expr) -> Pat {
        Pat::Expr(Expr::Paren(self.paren_expr(span, expr).into()))
    }
    #[inline]
    pub fn pat_expr_jsx_member_expr(&mut self, span: Span, obj: JSXObject, prop: IdentName) -> Pat {
        Pat::Expr(Expr::JSXMember(
            self.jsx_member_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn pat_expr_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> Pat {
        Pat::Expr(Expr::JSXNamespacedName(
            self.jsx_namespaced_name(span, ns, name).into(),
        ))
    }
    #[inline]
    pub fn pat_expr_jsx_empty_expr(&mut self, span: Span) -> Pat {
        Pat::Expr(Expr::JSXEmpty(self.jsx_empty_expr(span).into()))
    }
    #[inline]
    pub fn pat_expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> Pat {
        Pat::Expr(Expr::JSXElement(
            self.jsx_element(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn pat_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> Pat {
        Pat::Expr(Expr::JSXFragment(
            self.jsx_fragment(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn pat_expr_private_name(&mut self, span: Span, name: Atom<'a>) -> Pat {
        Pat::Expr(Expr::PrivateName(self.private_name(span, name).into()))
    }
    #[inline]
    pub fn pat_expr_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> Pat {
        Pat::Expr(Expr::OptChain(
            self.opt_chain_expr(span, optional, base).into(),
        ))
    }
    #[inline]
    pub fn pat_expr_invalid(&mut self, span: Span) -> Pat {
        Pat::Expr(Expr::Invalid(self.invalid(span).into()))
    }
    #[inline]
    pub fn array_pat(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<Pat>>,
        optional: bool,
    ) -> ArrayPat {
        ArrayPat(self.add_node(
            span,
            NodeKind::ArrayPat,
            NodePayload::inline(
                0u64 | unsafe { elems.to_extra_data(self).other },
                0u32 | optional as u32,
            ),
        ))
    }
    #[inline]
    pub fn object_pat(
        &mut self,
        span: Span,
        props: TypedSubRange<ObjectPatProp>,
        optional: bool,
    ) -> ObjectPat {
        ObjectPat(self.add_node(
            span,
            NodeKind::ObjectPat,
            NodePayload::inline(
                0u64 | unsafe { props.to_extra_data(self).other },
                0u32 | optional as u32,
            ),
        ))
    }
    #[inline]
    pub fn assign_pat(&mut self, span: Span, left: Pat, right: Expr) -> AssignPat {
        AssignPat(self.add_node(
            span,
            NodeKind::AssignPat,
            NodePayload::inline(
                0u64 | (left.node_id().index() as u32 as u64)
                    | ((right.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn rest_pat(&mut self, span: Span, dot3_token: Span, arg: Pat) -> RestPat {
        RestPat(self.add_node(
            span,
            NodeKind::RestPat,
            NodePayload::inline(
                0u64 | unsafe { dot3_token.to_extra_data(self).other },
                0u32 | arg.node_id().index() as u32,
            ),
        ))
    }
    #[inline]
    pub fn object_pat_prop_key_value_pat_prop(
        &mut self,
        span: Span,
        key: PropName,
        value: Pat,
    ) -> ObjectPatProp {
        ObjectPatProp::KeyValue(self.key_value_pat_prop(span, key, value).into())
    }
    #[inline]
    pub fn object_pat_prop_assign_pat_prop(
        &mut self,
        span: Span,
        key: BindingIdent,
        value: Option<Expr>,
    ) -> ObjectPatProp {
        ObjectPatProp::Assign(self.assign_pat_prop(span, key, value).into())
    }
    #[inline]
    pub fn object_pat_prop_rest_pat(
        &mut self,
        span: Span,
        dot3_token: Span,
        arg: Pat,
    ) -> ObjectPatProp {
        ObjectPatProp::Rest(self.rest_pat(span, dot3_token, arg).into())
    }
    #[inline]
    pub fn key_value_pat_prop(&mut self, span: Span, key: PropName, value: Pat) -> KeyValuePatProp {
        KeyValuePatProp(self.add_node(
            span,
            NodeKind::KeyValuePatProp,
            NodePayload::inline(
                0u64 | (key.node_id().index() as u32 as u64)
                    | ((value.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn assign_pat_prop(
        &mut self,
        span: Span,
        key: BindingIdent,
        value: Option<Expr>,
    ) -> AssignPatProp {
        AssignPatProp(self.add_node(
            span,
            NodeKind::AssignPatProp,
            NodePayload::inline(
                0u64 | (key.node_id().index() as u32 as u64)
                    | ((crate::OptionalNodeId::from(value.map(|n| n.node_id())).into_raw() as u64)
                        << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn ident(&mut self, span: Span, sym: Atom<'a>) -> Ident {
        let _inline_default_1 = <Option<SymbolId>>::default();
        Ident(self.add_node(
            span,
            NodeKind::Ident,
            NodePayload::inline(
                0u64 | unsafe { sym.to_extra_data(self).other },
                0u32 | _inline_default_1.map_or(0, |id| id.index() as u32 + 1),
            ),
        ))
    }
    #[inline]
    pub fn ident_name(&mut self, span: Span, sym: Atom<'a>) -> IdentName {
        IdentName(self.add_node(
            span,
            NodeKind::IdentName,
            NodePayload::inline(0u64 | unsafe { sym.to_extra_data(self).other }, 0u32),
        ))
    }
    #[inline]
    pub fn private_name(&mut self, span: Span, name: Atom<'a>) -> PrivateName {
        PrivateName(self.add_node(
            span,
            NodeKind::PrivateName,
            NodePayload::inline(0u64 | unsafe { name.to_extra_data(self).other }, 0u32),
        ))
    }
    #[inline]
    pub fn binding_ident(&mut self, span: Span, id: Ident) -> BindingIdent {
        BindingIdent(self.add_node(
            span,
            NodeKind::BindingIdent,
            NodePayload::inline(0u64 | (id.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn lit_str(&mut self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Lit {
        Lit::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn lit_bool(&mut self, span: Span, value: bool) -> Lit {
        Lit::Bool(self.bool(span, value).into())
    }
    #[inline]
    pub fn lit_null(&mut self, span: Span) -> Lit {
        Lit::Null(self.null(span).into())
    }
    #[inline]
    pub fn lit_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Lit {
        Lit::Num(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn lit_big_int(&mut self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> Lit {
        Lit::BigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn lit_regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Lit {
        Lit::Regex(self.regex(span, exp, flags).into())
    }
    #[inline]
    pub fn str(&mut self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Str {
        let _f0 = self.add_extra(raw.to_extra_data(self));
        Str(self.add_node(
            span,
            NodeKind::Str,
            NodePayload::partial(0u64 | unsafe { value.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn bool(&mut self, span: Span, value: bool) -> Bool {
        Bool(self.add_node(
            span,
            NodeKind::Bool,
            NodePayload::inline(0u64 | (value as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn null(&mut self, span: Span) -> Null {
        Null(self.add_node(span, NodeKind::Null, NodePayload::inline(0, 0)))
    }
    #[inline]
    pub fn number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Number {
        let _f0 = self.add_extra(raw.to_extra_data(self));
        Number(self.add_node(
            span,
            NodeKind::Number,
            NodePayload::partial(0u64 | unsafe { value.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn big_int(&mut self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> BigInt {
        let _f0 = self.add_extra(raw.to_extra_data(self));
        BigInt(self.add_node(
            span,
            NodeKind::BigInt,
            NodePayload::partial(0u64 | unsafe { value.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Regex {
        let _f0 = self.add_extra(flags.to_extra_data(self));
        Regex(self.add_node(
            span,
            NodeKind::Regex,
            NodePayload::partial(0u64 | unsafe { exp.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn jsx_object_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> JSXObject {
        JSXObject::JSXMemberExpr(self.jsx_member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn jsx_object_ident(&mut self, span: Span, sym: Atom<'a>) -> JSXObject {
        JSXObject::Ident(self.ident(span, sym).into())
    }
    #[inline]
    pub fn jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> JSXMemberExpr {
        JSXMemberExpr(self.add_node(
            span,
            NodeKind::JSXMemberExpr,
            NodePayload::inline(
                0u64 | (obj.node_id().index() as u32 as u64)
                    | ((prop.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> JSXNamespacedName {
        JSXNamespacedName(self.add_node(
            span,
            NodeKind::JSXNamespacedName,
            NodePayload::inline(
                0u64 | (ns.node_id().index() as u32 as u64)
                    | ((name.node_id().index() as u32 as u64) << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn jsx_empty_expr(&mut self, span: Span) -> JSXEmptyExpr {
        JSXEmptyExpr(self.add_node(span, NodeKind::JSXEmptyExpr, NodePayload::inline(0, 0)))
    }
    #[inline]
    pub fn jsx_expr_container(&mut self, span: Span, expr: JSXExpr) -> JSXExprContainer {
        JSXExprContainer(self.add_node(
            span,
            NodeKind::JSXExprContainer,
            NodePayload::inline(0u64 | (expr.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn jsx_expr_jsx_empty_expr(&mut self, span: Span) -> JSXExpr {
        JSXExpr::JSXEmptyExpr(self.jsx_empty_expr(span).into())
    }
    #[inline]
    pub fn jsx_expr_expr_this_expr(&mut self, span: Span) -> JSXExpr {
        JSXExpr::Expr(Expr::This(self.this_expr(span).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Array(self.array_lit(span, elems).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_object_lit(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Object(self.object_lit(span, props).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Fn(self.fn_expr(span, ident, function).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> JSXExpr {
        JSXExpr::Expr(Expr::Unary(self.unary_expr(span, op, arg).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Update(self.update_expr(span, op, prefix, arg).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_bin_expr(
        &mut self,
        span: Span,
        op: BinaryOp,
        left: Expr,
        right: Expr,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Bin(self.bin_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Assign(self.assign_expr(span, op, left, right).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Member(self.member_expr(span, obj, prop).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::SuperProp(
            self.super_prop_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn jsx_expr_expr_cond_expr(
        &mut self,
        span: Span,
        test: Expr,
        cons: Expr,
        alt: Expr,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Cond(self.cond_expr(span, test, cons, alt).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expr_expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Call(self.call_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::New(self.new_expr(span, callee, args).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> JSXExpr {
        JSXExpr::Expr(Expr::Seq(self.seq_expr(span, exprs).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_ident(&mut self, span: Span, sym: Atom<'a>) -> JSXExpr {
        JSXExpr::Expr(Expr::Ident(self.ident(span, sym).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_lit_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Lit(Lit::Str(self.str(span, value, raw).into())))
    }
    #[inline]
    pub fn jsx_expr_expr_lit_bool(&mut self, span: Span, value: bool) -> JSXExpr {
        JSXExpr::Expr(Expr::Lit(Lit::Bool(self.bool(span, value).into())))
    }
    #[inline]
    pub fn jsx_expr_expr_lit_null(&mut self, span: Span) -> JSXExpr {
        JSXExpr::Expr(Expr::Lit(Lit::Null(self.null(span).into())))
    }
    #[inline]
    pub fn jsx_expr_expr_lit_number(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Lit(Lit::Num(self.number(span, value, raw).into())))
    }
    #[inline]
    pub fn jsx_expr_expr_lit_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Lit(Lit::BigInt(
            self.big_int(span, value, raw).into(),
        )))
    }
    #[inline]
    pub fn jsx_expr_expr_lit_regex(
        &mut self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Lit(Lit::Regex(self.regex(span, exp, flags).into())))
    }
    #[inline]
    pub fn jsx_expr_expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Tpl(self.tpl(span, exprs, quasis).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> JSXExpr {
        JSXExpr::Expr(Expr::TaggedTpl(self.tagged_tpl(span, tag, tpl).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Arrow(
            self.arrow_expr(span, params, body, is_async).into(),
        ))
    }
    #[inline]
    pub fn jsx_expr_expr_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Class(self.class_expr(span, ident, class).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_yield_expr(
        &mut self,
        span: Span,
        arg: Option<Expr>,
        delegate: bool,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::Yield(self.yield_expr(span, arg, delegate).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> JSXExpr {
        JSXExpr::Expr(Expr::MetaProp(self.meta_prop_expr(span, kind).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_await_expr(&mut self, span: Span, arg: Expr) -> JSXExpr {
        JSXExpr::Expr(Expr::Await(self.await_expr(span, arg).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_paren_expr(&mut self, span: Span, expr: Expr) -> JSXExpr {
        JSXExpr::Expr(Expr::Paren(self.paren_expr(span, expr).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::JSXMember(
            self.jsx_member_expr(span, obj, prop).into(),
        ))
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::JSXNamespacedName(
            self.jsx_namespaced_name(span, ns, name).into(),
        ))
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_empty_expr(&mut self, span: Span) -> JSXExpr {
        JSXExpr::Expr(Expr::JSXEmpty(self.jsx_empty_expr(span).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::JSXElement(
            self.jsx_element(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn jsx_expr_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::JSXFragment(
            self.jsx_fragment(span, opening, children, closing).into(),
        ))
    }
    #[inline]
    pub fn jsx_expr_expr_private_name(&mut self, span: Span, name: Atom<'a>) -> JSXExpr {
        JSXExpr::Expr(Expr::PrivateName(self.private_name(span, name).into()))
    }
    #[inline]
    pub fn jsx_expr_expr_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> JSXExpr {
        JSXExpr::Expr(Expr::OptChain(
            self.opt_chain_expr(span, optional, base).into(),
        ))
    }
    #[inline]
    pub fn jsx_expr_expr_invalid(&mut self, span: Span) -> JSXExpr {
        JSXExpr::Expr(Expr::Invalid(self.invalid(span).into()))
    }
    #[inline]
    pub fn jsx_spread_child(&mut self, span: Span, expr: Expr) -> JSXSpreadChild {
        JSXSpreadChild(self.add_node(
            span,
            NodeKind::JSXSpreadChild,
            NodePayload::inline(0u64 | (expr.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn jsx_element_name_ident(&mut self, span: Span, sym: Atom<'a>) -> JSXElementName {
        JSXElementName::Ident(self.ident(span, sym).into())
    }
    #[inline]
    pub fn jsx_element_name_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> JSXElementName {
        JSXElementName::JSXMemberExpr(self.jsx_member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn jsx_element_name_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> JSXElementName {
        JSXElementName::JSXNamespacedName(self.jsx_namespaced_name(span, ns, name).into())
    }
    #[inline]
    pub fn jsx_opening_element(
        &mut self,
        span: Span,
        name: JSXElementName,
        attrs: TypedSubRange<JSXAttrOrSpread>,
        self_closing: bool,
    ) -> JSXOpeningElement {
        let _f0 = self.add_extra(name.to_extra_data(self));
        let _f1 = self.add_extra(self_closing.to_extra_data(self));
        JSXOpeningElement(self.add_node(
            span,
            NodeKind::JSXOpeningElement,
            NodePayload::partial(0u64 | unsafe { attrs.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn jsx_attr_or_spread_jsx_attr(
        &mut self,
        span: Span,
        name: JSXAttrName,
        value: Option<JSXAttrValue>,
    ) -> JSXAttrOrSpread {
        JSXAttrOrSpread::JSXAttr(self.jsx_attr(span, name, value).into())
    }
    #[inline]
    pub fn jsx_attr_or_spread_spread_element(
        &mut self,
        span: Span,
        dot3_token: Span,
        expr: Expr,
    ) -> JSXAttrOrSpread {
        JSXAttrOrSpread::SpreadElement(self.spread_element(span, dot3_token, expr).into())
    }
    #[inline]
    pub fn jsx_closing_element(&mut self, span: Span, name: JSXElementName) -> JSXClosingElement {
        JSXClosingElement(self.add_node(
            span,
            NodeKind::JSXClosingElement,
            NodePayload::inline(0u64 | (name.node_id().index() as u32 as u64), 0u32),
        ))
    }
    #[inline]
    pub fn jsx_attr(
        &mut self,
        span: Span,
        name: JSXAttrName,
        value: Option<JSXAttrValue>,
    ) -> JSXAttr {
        JSXAttr(self.add_node(
            span,
            NodeKind::JSXAttr,
            NodePayload::inline(
                0u64 | (name.node_id().index() as u32 as u64)
                    | ((crate::OptionalNodeId::from(value.map(|n| n.node_id())).into_raw() as u64)
                        << 32usize),
                0u32,
            ),
        ))
    }
    #[inline]
    pub fn jsx_attr_name_ident_name(&mut self, span: Span, sym: Atom<'a>) -> JSXAttrName {
        JSXAttrName::Ident(self.ident_name(span, sym).into())
    }
    #[inline]
    pub fn jsx_attr_name_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> JSXAttrName {
        JSXAttrName::JSXNamespacedName(self.jsx_namespaced_name(span, ns, name).into())
    }
    #[inline]
    pub fn jsx_attr_value_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> JSXAttrValue {
        JSXAttrValue::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn jsx_attr_value_jsx_expr_container(&mut self, span: Span, expr: JSXExpr) -> JSXAttrValue {
        JSXAttrValue::JSXExprContainer(self.jsx_expr_container(span, expr).into())
    }
    #[inline]
    pub fn jsx_attr_value_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> JSXAttrValue {
        JSXAttrValue::JSXElement(self.jsx_element(span, opening, children, closing).into())
    }
    #[inline]
    pub fn jsx_attr_value_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> JSXAttrValue {
        JSXAttrValue::JSXFragment(self.jsx_fragment(span, opening, children, closing).into())
    }
    #[inline]
    pub fn jsx_text(&mut self, span: Span, value: Atom<'a>, raw: Atom<'a>) -> JSXText {
        let _f0 = self.add_extra(raw.to_extra_data(self));
        JSXText(self.add_node(
            span,
            NodeKind::JSXText,
            NodePayload::partial(0u64 | unsafe { value.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> JSXElement {
        let _f0 = self.add_extra(opening.to_extra_data(self));
        let _f1 = self.add_extra(closing.to_extra_data(self));
        JSXElement(self.add_node(
            span,
            NodeKind::JSXElement,
            NodePayload::partial(0u64 | unsafe { children.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn jsx_element_child_jsx_text(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Atom<'a>,
    ) -> JSXElementChild {
        JSXElementChild::JSXText(self.jsx_text(span, value, raw).into())
    }
    #[inline]
    pub fn jsx_element_child_jsx_expr_container(
        &mut self,
        span: Span,
        expr: JSXExpr,
    ) -> JSXElementChild {
        JSXElementChild::JSXExprContainer(self.jsx_expr_container(span, expr).into())
    }
    #[inline]
    pub fn jsx_element_child_jsx_spread_child(
        &mut self,
        span: Span,
        expr: Expr,
    ) -> JSXElementChild {
        JSXElementChild::JSXSpreadChild(self.jsx_spread_child(span, expr).into())
    }
    #[inline]
    pub fn jsx_element_child_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> JSXElementChild {
        JSXElementChild::JSXElement(self.jsx_element(span, opening, children, closing).into())
    }
    #[inline]
    pub fn jsx_element_child_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> JSXElementChild {
        JSXElementChild::JSXFragment(self.jsx_fragment(span, opening, children, closing).into())
    }
    #[inline]
    pub fn jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> JSXFragment {
        let _f0 = self.add_extra(opening.to_extra_data(self));
        let _f1 = self.add_extra(closing.to_extra_data(self));
        JSXFragment(self.add_node(
            span,
            NodeKind::JSXFragment,
            NodePayload::partial(0u64 | unsafe { children.to_extra_data(self).other }, _f0),
        ))
    }
    #[inline]
    pub fn jsx_opening_fragment(&mut self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment(self.add_node(
            span,
            NodeKind::JSXOpeningFragment,
            NodePayload::inline(0, 0),
        ))
    }
    #[inline]
    pub fn jsx_closing_fragment(&mut self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment(self.add_node(
            span,
            NodeKind::JSXClosingFragment,
            NodePayload::inline(0, 0),
        ))
    }
}
