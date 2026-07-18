#![allow(unused, clippy::useless_conversion, clippy::identity_op)]
use crate::*;
use oxc_index::Idx;
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
        Module(self.add_node(
            span,
            NodeKind::Module,
            NodePayload(
                0u128
                    | ((unsafe { body.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { shebang.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn script(
        &mut self,
        span: Span,
        body: TypedSubRange<Stmt>,
        shebang: Option<Atom<'a>>,
    ) -> Script {
        Script(self.add_node(
            span,
            NodeKind::Script,
            NodePayload(
                0u128
                    | ((unsafe { body.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { shebang.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn module_item_import_decl(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ImportSpecifier>,
        src: Str,
        type_only: bool,
        with: Option<ObjectLit>,
        phase: ImportPhase,
    ) -> ModuleItem {
        ModuleItem::ModuleDeclImport(
            self.import_decl(span, specifiers, src, type_only, with, phase)
                .into(),
        )
    }
    #[inline]
    pub fn module_item_export_decl(&mut self, span: Span, decl: Decl) -> ModuleItem {
        ModuleItem::ModuleDeclExportDecl(self.export_decl(span, decl).into())
    }
    #[inline]
    pub fn module_item_named_export(
        &mut self,
        span: Span,
        specifiers: TypedSubRange<ExportSpecifier>,
        src: Option<Str>,
        type_only: bool,
        with: Option<ObjectLit>,
    ) -> ModuleItem {
        ModuleItem::ModuleDeclExportNamed(
            self.named_export(span, specifiers, src, type_only, with)
                .into(),
        )
    }
    #[inline]
    pub fn module_item_export_default_decl(&mut self, span: Span, decl: DefaultDecl) -> ModuleItem {
        ModuleItem::ModuleDeclExportDefaultDecl(self.export_default_decl(span, decl).into())
    }
    #[inline]
    pub fn module_item_export_default_expr(&mut self, span: Span, expr: Expr) -> ModuleItem {
        ModuleItem::ModuleDeclExportDefaultExpr(self.export_default_expr(span, expr).into())
    }
    #[inline]
    pub fn module_item_export_all(
        &mut self,
        span: Span,
        src: Str,
        type_only: bool,
        with: Option<ObjectLit>,
    ) -> ModuleItem {
        ModuleItem::ModuleDeclExportAll(self.export_all(span, src, type_only, with).into())
    }
    #[inline]
    pub fn module_item_block_stmt(&mut self, span: Span, stmts: TypedSubRange<Stmt>) -> ModuleItem {
        ModuleItem::StmtBlock(self.block_stmt(span, stmts).into())
    }
    #[inline]
    pub fn module_item_empty_stmt(&mut self, span: Span) -> ModuleItem {
        ModuleItem::StmtEmpty(self.empty_stmt(span).into())
    }
    #[inline]
    pub fn module_item_debugger_stmt(&mut self, span: Span) -> ModuleItem {
        ModuleItem::StmtDebugger(self.debugger_stmt(span).into())
    }
    #[inline]
    pub fn module_item_with_stmt(&mut self, span: Span, obj: Expr, body: Stmt) -> ModuleItem {
        ModuleItem::StmtWith(self.with_stmt(span, obj, body).into())
    }
    #[inline]
    pub fn module_item_return_stmt(&mut self, span: Span, arg: Option<Expr>) -> ModuleItem {
        ModuleItem::StmtReturn(self.return_stmt(span, arg).into())
    }
    #[inline]
    pub fn module_item_labeled_stmt(&mut self, span: Span, label: Ident, body: Stmt) -> ModuleItem {
        ModuleItem::StmtLabeled(self.labeled_stmt(span, label, body).into())
    }
    #[inline]
    pub fn module_item_break_stmt(&mut self, span: Span, label: Option<Ident>) -> ModuleItem {
        ModuleItem::StmtBreak(self.break_stmt(span, label).into())
    }
    #[inline]
    pub fn module_item_continue_stmt(&mut self, span: Span, label: Option<Ident>) -> ModuleItem {
        ModuleItem::StmtContinue(self.continue_stmt(span, label).into())
    }
    #[inline]
    pub fn module_item_if_stmt(
        &mut self,
        span: Span,
        test: Expr,
        cons: Stmt,
        alt: Option<Stmt>,
    ) -> ModuleItem {
        ModuleItem::StmtIf(self.if_stmt(span, test, cons, alt).into())
    }
    #[inline]
    pub fn module_item_switch_stmt(
        &mut self,
        span: Span,
        discriminant: Expr,
        cases: TypedSubRange<SwitchCase>,
    ) -> ModuleItem {
        ModuleItem::StmtSwitch(self.switch_stmt(span, discriminant, cases).into())
    }
    #[inline]
    pub fn module_item_throw_stmt(&mut self, span: Span, arg: Expr) -> ModuleItem {
        ModuleItem::StmtThrow(self.throw_stmt(span, arg).into())
    }
    #[inline]
    pub fn module_item_try_stmt(
        &mut self,
        span: Span,
        block: BlockStmt,
        handler: Option<CatchClause>,
        finalizer: Option<BlockStmt>,
    ) -> ModuleItem {
        ModuleItem::StmtTry(self.try_stmt(span, block, handler, finalizer).into())
    }
    #[inline]
    pub fn module_item_while_stmt(&mut self, span: Span, test: Expr, body: Stmt) -> ModuleItem {
        ModuleItem::StmtWhile(self.while_stmt(span, test, body).into())
    }
    #[inline]
    pub fn module_item_do_while_stmt(&mut self, span: Span, test: Expr, body: Stmt) -> ModuleItem {
        ModuleItem::StmtDoWhile(self.do_while_stmt(span, test, body).into())
    }
    #[inline]
    pub fn module_item_for_stmt(
        &mut self,
        span: Span,
        init: Option<VarDeclOrExpr>,
        test: Option<Expr>,
        update: Option<Expr>,
        body: Stmt,
    ) -> ModuleItem {
        ModuleItem::StmtFor(self.for_stmt(span, init, test, update, body).into())
    }
    #[inline]
    pub fn module_item_for_in_stmt(
        &mut self,
        span: Span,
        left: ForHead,
        right: Expr,
        body: Stmt,
    ) -> ModuleItem {
        ModuleItem::StmtForIn(self.for_in_stmt(span, left, right, body).into())
    }
    #[inline]
    pub fn module_item_for_of_stmt(
        &mut self,
        span: Span,
        is_await: bool,
        left: ForHead,
        right: Expr,
        body: Stmt,
    ) -> ModuleItem {
        ModuleItem::StmtForOf(self.for_of_stmt(span, is_await, left, right, body).into())
    }
    #[inline]
    pub fn module_item_class_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        class: Class,
    ) -> ModuleItem {
        ModuleItem::StmtDeclClass(self.class_decl(span, ident, declare, class).into())
    }
    #[inline]
    pub fn module_item_fn_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        function: Function,
    ) -> ModuleItem {
        ModuleItem::StmtDeclFn(self.fn_decl(span, ident, declare, function).into())
    }
    #[inline]
    pub fn module_item_var_decl(
        &mut self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> ModuleItem {
        ModuleItem::StmtDeclVar(self.var_decl(span, kind, declare, decls).into())
    }
    #[inline]
    pub fn module_item_using_decl(
        &mut self,
        span: Span,
        is_await: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> ModuleItem {
        ModuleItem::StmtDeclUsing(self.using_decl(span, is_await, decls).into())
    }
    #[inline]
    pub fn module_item_expr_stmt(&mut self, span: Span, expr: Expr) -> ModuleItem {
        ModuleItem::StmtExpr(self.expr_stmt(span, expr).into())
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
        let _extra_data = [
            type_only.to_extra_data(self),
            with.to_extra_data(self),
            phase.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ImportDecl(self.add_node(
            span,
            NodeKind::ImportDecl,
            NodePayload(
                0u128
                    | ((unsafe { specifiers.to_extra_data(self).other } as u128) << 0usize)
                    | ((src.node_id().index() as u32 as u128) << 64usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
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
            NodePayload(
                0u128
                    | ((local.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { imported.to_extra_data(self).other } as u128) << 0usize)
                    | ((is_type_only as u32 as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn import_default_specifier(&mut self, span: Span, local: Ident) -> ImportDefaultSpecifier {
        ImportDefaultSpecifier(self.add_node(
            span,
            NodeKind::ImportDefaultSpecifier,
            NodePayload(0u128 | ((local.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn import_star_as_specifier(&mut self, span: Span, local: Ident) -> ImportStarAsSpecifier {
        ImportStarAsSpecifier(self.add_node(
            span,
            NodeKind::ImportStarAsSpecifier,
            NodePayload(0u128 | ((local.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn export_decl(&mut self, span: Span, decl: Decl) -> ExportDecl {
        ExportDecl(self.add_node(
            span,
            NodeKind::ExportDecl,
            NodePayload(0u128 | ((unsafe { decl.to_extra_data(self).other } as u128) << 0usize)),
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
        let _extra_data = [type_only.to_extra_data(self), with.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        NamedExport(self.add_node(
            span,
            NodeKind::NamedExport,
            NodePayload(
                0u128
                    | ((unsafe { specifiers.to_extra_data(self).other } as u128) << 0usize)
                    | ((src.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
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
            NodePayload(0u128 | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)),
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
            NodePayload(0u128 | ((exported.node_id().index() as u32 as u128) << 0usize)),
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
        let _f0 = self.add_extra(exported.to_extra_data(self));
        ExportNamedSpecifier(self.add_node(
            span,
            NodeKind::ExportNamedSpecifier,
            NodePayload(
                0u128
                    | ((unsafe { orig.to_extra_data(self).other } as u128) << 0usize)
                    | ((is_type_only as u32 as u128) << 64usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn export_default_decl(&mut self, span: Span, decl: DefaultDecl) -> ExportDefaultDecl {
        ExportDefaultDecl(self.add_node(
            span,
            NodeKind::ExportDefaultDecl,
            NodePayload(0u128 | ((unsafe { decl.to_extra_data(self).other } as u128) << 0usize)),
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
            NodePayload(0u128 | ((unsafe { expr.to_extra_data(self).other } as u128) << 0usize)),
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
            NodePayload(
                0u128
                    | ((src.node_id().index() as u32 as u128) << 0usize)
                    | ((type_only as u32 as u128) << 64usize)
                    | ((with.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn block_stmt(&mut self, span: Span, stmts: TypedSubRange<Stmt>) -> BlockStmt {
        let _inline_default_1 = <Option<ScopeId>>::default();
        BlockStmt(self.add_node(
            span,
            NodeKind::BlockStmt,
            NodePayload(
                0u128
                    | ((unsafe { stmts.to_extra_data(self).other } as u128) << 0usize)
                    | ((_inline_default_1.map_or(0, |id| id.index() as u32 + 1) as u128)
                        << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn stmt_class_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        class: Class,
    ) -> Stmt {
        Stmt::Class(self.class_decl(span, ident, declare, class).into())
    }
    #[inline]
    pub fn stmt_fn_decl(
        &mut self,
        span: Span,
        ident: Ident,
        declare: bool,
        function: Function,
    ) -> Stmt {
        Stmt::Fn(self.fn_decl(span, ident, declare, function).into())
    }
    #[inline]
    pub fn stmt_var_decl(
        &mut self,
        span: Span,
        kind: VarDeclKind,
        declare: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> Stmt {
        Stmt::Var(self.var_decl(span, kind, declare, decls).into())
    }
    #[inline]
    pub fn stmt_using_decl(
        &mut self,
        span: Span,
        is_await: bool,
        decls: TypedSubRange<VarDeclarator>,
    ) -> Stmt {
        Stmt::Using(self.using_decl(span, is_await, decls).into())
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
    pub fn stmt_expr_stmt(&mut self, span: Span, expr: Expr) -> Stmt {
        Stmt::Expr(self.expr_stmt(span, expr).into())
    }
    #[inline]
    pub fn expr_stmt(&mut self, span: Span, expr: Expr) -> ExprStmt {
        ExprStmt(self.add_node(
            span,
            NodeKind::ExprStmt,
            NodePayload(0u128 | ((unsafe { expr.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn empty_stmt(&mut self, span: Span) -> EmptyStmt {
        EmptyStmt(self.add_node(span, NodeKind::EmptyStmt, NodePayload(0)))
    }
    #[inline]
    pub fn debugger_stmt(&mut self, span: Span) -> DebuggerStmt {
        DebuggerStmt(self.add_node(span, NodeKind::DebuggerStmt, NodePayload(0)))
    }
    #[inline]
    pub fn with_stmt(&mut self, span: Span, obj: Expr, body: Stmt) -> WithStmt {
        WithStmt(self.add_node(
            span,
            NodeKind::WithStmt,
            NodePayload(
                0u128
                    | ((unsafe { obj.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { body.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn return_stmt(&mut self, span: Span, arg: Option<Expr>) -> ReturnStmt {
        ReturnStmt(self.add_node(
            span,
            NodeKind::ReturnStmt,
            NodePayload(0u128 | ((unsafe { arg.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn labeled_stmt(&mut self, span: Span, label: Ident, body: Stmt) -> LabeledStmt {
        LabeledStmt(self.add_node(
            span,
            NodeKind::LabeledStmt,
            NodePayload(
                0u128
                    | ((label.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { body.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn break_stmt(&mut self, span: Span, label: Option<Ident>) -> BreakStmt {
        BreakStmt(self.add_node(
            span,
            NodeKind::BreakStmt,
            NodePayload(
                0u128
                    | ((label.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn continue_stmt(&mut self, span: Span, label: Option<Ident>) -> ContinueStmt {
        ContinueStmt(self.add_node(
            span,
            NodeKind::ContinueStmt,
            NodePayload(
                0u128
                    | ((label.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn if_stmt(&mut self, span: Span, test: Expr, cons: Stmt, alt: Option<Stmt>) -> IfStmt {
        let _extra_data = [cons.to_extra_data(self), alt.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        IfStmt(self.add_node(
            span,
            NodeKind::IfStmt,
            NodePayload(
                0u128
                    | ((unsafe { test.to_extra_data(self).other } as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
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
            NodePayload(
                0u128
                    | ((unsafe { discriminant.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { cases.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn throw_stmt(&mut self, span: Span, arg: Expr) -> ThrowStmt {
        ThrowStmt(self.add_node(
            span,
            NodeKind::ThrowStmt,
            NodePayload(0u128 | ((unsafe { arg.to_extra_data(self).other } as u128) << 0usize)),
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
            NodePayload(
                0u128
                    | ((block.node_id().index() as u32 as u128) << 0usize)
                    | ((handler.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 32usize)
                    | ((finalizer.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn while_stmt(&mut self, span: Span, test: Expr, body: Stmt) -> WhileStmt {
        WhileStmt(self.add_node(
            span,
            NodeKind::WhileStmt,
            NodePayload(
                0u128
                    | ((unsafe { test.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { body.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn do_while_stmt(&mut self, span: Span, test: Expr, body: Stmt) -> DoWhileStmt {
        DoWhileStmt(self.add_node(
            span,
            NodeKind::DoWhileStmt,
            NodePayload(
                0u128
                    | ((unsafe { test.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { body.to_extra_data(self).other } as u128) << 64usize),
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
        let _extra_data = [
            test.to_extra_data(self),
            update.to_extra_data(self),
            body.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ForStmt(self.add_node(
            span,
            NodeKind::ForStmt,
            NodePayload(
                0u128
                    | ((unsafe { init.to_extra_data(self).other } as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn for_in_stmt(&mut self, span: Span, left: ForHead, right: Expr, body: Stmt) -> ForInStmt {
        let _extra_data = [right.to_extra_data(self), body.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ForInStmt(self.add_node(
            span,
            NodeKind::ForInStmt,
            NodePayload(
                0u128
                    | ((unsafe { left.to_extra_data(self).other } as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
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
        let _extra_data = [right.to_extra_data(self), body.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ForOfStmt(self.add_node(
            span,
            NodeKind::ForOfStmt,
            NodePayload(
                0u128
                    | ((is_await as u32 as u128) << 64usize)
                    | ((unsafe { left.to_extra_data(self).other } as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
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
            NodePayload(
                0u128
                    | ((unsafe { test.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { cons.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn catch_clause(&mut self, span: Span, param: Option<Pat>, body: BlockStmt) -> CatchClause {
        CatchClause(self.add_node(
            span,
            NodeKind::CatchClause,
            NodePayload(
                0u128
                    | ((unsafe { param.to_extra_data(self).other } as u128) << 0usize)
                    | ((body.node_id().index() as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn for_head_this_expr(&mut self, span: Span) -> ForHead {
        ForHead::ExprThis(self.this_expr(span).into())
    }
    #[inline]
    pub fn for_head_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> ForHead {
        ForHead::ExprArray(self.array_lit(span, elems).into())
    }
    #[inline]
    pub fn for_head_object_lit(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
    ) -> ForHead {
        ForHead::ExprObject(self.object_lit(span, props).into())
    }
    #[inline]
    pub fn for_head_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> ForHead {
        ForHead::ExprFn(self.fn_expr(span, ident, function).into())
    }
    #[inline]
    pub fn for_head_unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> ForHead {
        ForHead::ExprUnary(self.unary_expr(span, op, arg).into())
    }
    #[inline]
    pub fn for_head_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> ForHead {
        ForHead::ExprUpdate(self.update_expr(span, op, prefix, arg).into())
    }
    #[inline]
    pub fn for_head_bin_expr(
        &mut self,
        span: Span,
        op: BinaryOp,
        left: Expr,
        right: Expr,
    ) -> ForHead {
        ForHead::ExprBin(self.bin_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn for_head_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> ForHead {
        ForHead::ExprAssign(self.assign_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn for_head_member_expr(&mut self, span: Span, obj: Expr, prop: MemberProp) -> ForHead {
        ForHead::ExprMember(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn for_head_super_prop_expr(&mut self, span: Span, obj: Super, prop: SuperProp) -> ForHead {
        ForHead::ExprSuperProp(self.super_prop_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn for_head_cond_expr(&mut self, span: Span, test: Expr, cons: Expr, alt: Expr) -> ForHead {
        ForHead::ExprCond(self.cond_expr(span, test, cons, alt).into())
    }
    #[inline]
    pub fn for_head_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> ForHead {
        ForHead::ExprImport(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        )
    }
    #[inline]
    pub fn for_head_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> ForHead {
        ForHead::ExprCall(self.call_expr(span, callee, args).into())
    }
    #[inline]
    pub fn for_head_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> ForHead {
        ForHead::ExprNew(self.new_expr(span, callee, args).into())
    }
    #[inline]
    pub fn for_head_seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> ForHead {
        ForHead::ExprSeq(self.seq_expr(span, exprs).into())
    }
    #[inline]
    pub fn for_head_ident(&mut self, span: Span, sym: Atom<'a>) -> ForHead {
        ForHead::ExprIdent(self.ident(span, sym).into())
    }
    #[inline]
    pub fn for_head_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> ForHead {
        ForHead::ExprStr(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn for_head_bool(&mut self, span: Span, value: bool) -> ForHead {
        ForHead::ExprBool(self.bool(span, value).into())
    }
    #[inline]
    pub fn for_head_null(&mut self, span: Span) -> ForHead {
        ForHead::ExprNull(self.null(span).into())
    }
    #[inline]
    pub fn for_head_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> ForHead {
        ForHead::ExprNum(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn for_head_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> ForHead {
        ForHead::ExprBigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn for_head_regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> ForHead {
        ForHead::ExprRegex(self.regex(span, exp, flags).into())
    }
    #[inline]
    pub fn for_head_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> ForHead {
        ForHead::ExprTpl(self.tpl(span, exprs, quasis).into())
    }
    #[inline]
    pub fn for_head_tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> ForHead {
        ForHead::ExprTaggedTpl(self.tagged_tpl(span, tag, tpl).into())
    }
    #[inline]
    pub fn for_head_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> ForHead {
        ForHead::ExprArrow(self.arrow_expr(span, params, body, is_async).into())
    }
    #[inline]
    pub fn for_head_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> ForHead {
        ForHead::ExprClass(self.class_expr(span, ident, class).into())
    }
    #[inline]
    pub fn for_head_yield_expr(
        &mut self,
        span: Span,
        arg: Option<Expr>,
        delegate: bool,
    ) -> ForHead {
        ForHead::ExprYield(self.yield_expr(span, arg, delegate).into())
    }
    #[inline]
    pub fn for_head_meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> ForHead {
        ForHead::ExprMetaProp(self.meta_prop_expr(span, kind).into())
    }
    #[inline]
    pub fn for_head_await_expr(&mut self, span: Span, arg: Expr) -> ForHead {
        ForHead::ExprAwait(self.await_expr(span, arg).into())
    }
    #[inline]
    pub fn for_head_paren_expr(&mut self, span: Span, expr: Expr) -> ForHead {
        ForHead::ExprParen(self.paren_expr(span, expr).into())
    }
    #[inline]
    pub fn for_head_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> ForHead {
        ForHead::ExprJSXMember(self.jsx_member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn for_head_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> ForHead {
        ForHead::ExprJSXNamespacedName(self.jsx_namespaced_name(span, ns, name).into())
    }
    #[inline]
    pub fn for_head_jsx_empty_expr(&mut self, span: Span) -> ForHead {
        ForHead::ExprJSXEmpty(self.jsx_empty_expr(span).into())
    }
    #[inline]
    pub fn for_head_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> ForHead {
        ForHead::ExprJSXElement(self.jsx_element(span, opening, children, closing).into())
    }
    #[inline]
    pub fn for_head_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> ForHead {
        ForHead::ExprJSXFragment(self.jsx_fragment(span, opening, children, closing).into())
    }
    #[inline]
    pub fn for_head_private_name(&mut self, span: Span, name: Atom<'a>) -> ForHead {
        ForHead::ExprPrivateName(self.private_name(span, name).into())
    }
    #[inline]
    pub fn for_head_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> ForHead {
        ForHead::ExprOptChain(self.opt_chain_expr(span, optional, base).into())
    }
    #[inline]
    pub fn for_head_expr_invalid(&mut self, span: Span) -> ForHead {
        ForHead::ExprInvalid(self.invalid(span).into())
    }
    #[inline]
    pub fn for_head_binding_ident(&mut self, span: Span, id: Ident) -> ForHead {
        ForHead::Ident(self.binding_ident(span, id).into())
    }
    #[inline]
    pub fn for_head_array_pat(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<Pat>>,
        optional: bool,
    ) -> ForHead {
        ForHead::Array(self.array_pat(span, elems, optional).into())
    }
    #[inline]
    pub fn for_head_rest_pat(&mut self, span: Span, dot3_token: Span, arg: Pat) -> ForHead {
        ForHead::Rest(self.rest_pat(span, dot3_token, arg).into())
    }
    #[inline]
    pub fn for_head_object_pat(
        &mut self,
        span: Span,
        props: TypedSubRange<ObjectPatProp>,
        optional: bool,
    ) -> ForHead {
        ForHead::Object(self.object_pat(span, props, optional).into())
    }
    #[inline]
    pub fn for_head_assign_pat(&mut self, span: Span, left: Pat, right: Expr) -> ForHead {
        ForHead::Assign(self.assign_pat(span, left, right).into())
    }
    #[inline]
    pub fn for_head_invalid(&mut self, span: Span) -> ForHead {
        ForHead::Invalid(self.invalid(span).into())
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
    pub fn var_decl_or_expr_this_expr(&mut self, span: Span) -> VarDeclOrExpr {
        VarDeclOrExpr::This(self.this_expr(span).into())
    }
    #[inline]
    pub fn var_decl_or_expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Array(self.array_lit(span, elems).into())
    }
    #[inline]
    pub fn var_decl_or_expr_object_lit(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Object(self.object_lit(span, props).into())
    }
    #[inline]
    pub fn var_decl_or_expr_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Fn(self.fn_expr(span, ident, function).into())
    }
    #[inline]
    pub fn var_decl_or_expr_unary_expr(
        &mut self,
        span: Span,
        op: UnaryOp,
        arg: Expr,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Unary(self.unary_expr(span, op, arg).into())
    }
    #[inline]
    pub fn var_decl_or_expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Update(self.update_expr(span, op, prefix, arg).into())
    }
    #[inline]
    pub fn var_decl_or_expr_bin_expr(
        &mut self,
        span: Span,
        op: BinaryOp,
        left: Expr,
        right: Expr,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Bin(self.bin_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn var_decl_or_expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Assign(self.assign_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn var_decl_or_expr_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Member(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn var_decl_or_expr_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::SuperProp(self.super_prop_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn var_decl_or_expr_cond_expr(
        &mut self,
        span: Span,
        test: Expr,
        cons: Expr,
        alt: Expr,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Cond(self.cond_expr(span, test, cons, alt).into())
    }
    #[inline]
    pub fn var_decl_or_expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        )
    }
    #[inline]
    pub fn var_decl_or_expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Call(self.call_expr(span, callee, args).into())
    }
    #[inline]
    pub fn var_decl_or_expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::New(self.new_expr(span, callee, args).into())
    }
    #[inline]
    pub fn var_decl_or_expr_seq_expr(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Seq(self.seq_expr(span, exprs).into())
    }
    #[inline]
    pub fn var_decl_or_expr_ident(&mut self, span: Span, sym: Atom<'a>) -> VarDeclOrExpr {
        VarDeclOrExpr::Ident(self.ident(span, sym).into())
    }
    #[inline]
    pub fn var_decl_or_expr_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn var_decl_or_expr_bool(&mut self, span: Span, value: bool) -> VarDeclOrExpr {
        VarDeclOrExpr::Bool(self.bool(span, value).into())
    }
    #[inline]
    pub fn var_decl_or_expr_null(&mut self, span: Span) -> VarDeclOrExpr {
        VarDeclOrExpr::Null(self.null(span).into())
    }
    #[inline]
    pub fn var_decl_or_expr_number(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Num(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn var_decl_or_expr_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::BigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn var_decl_or_expr_regex(
        &mut self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Regex(self.regex(span, exp, flags).into())
    }
    #[inline]
    pub fn var_decl_or_expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Tpl(self.tpl(span, exprs, quasis).into())
    }
    #[inline]
    pub fn var_decl_or_expr_tagged_tpl(
        &mut self,
        span: Span,
        tag: Expr,
        tpl: Tpl,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::TaggedTpl(self.tagged_tpl(span, tag, tpl).into())
    }
    #[inline]
    pub fn var_decl_or_expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Arrow(self.arrow_expr(span, params, body, is_async).into())
    }
    #[inline]
    pub fn var_decl_or_expr_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Class(self.class_expr(span, ident, class).into())
    }
    #[inline]
    pub fn var_decl_or_expr_yield_expr(
        &mut self,
        span: Span,
        arg: Option<Expr>,
        delegate: bool,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::Yield(self.yield_expr(span, arg, delegate).into())
    }
    #[inline]
    pub fn var_decl_or_expr_meta_prop_expr(
        &mut self,
        span: Span,
        kind: MetaPropKind,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::MetaProp(self.meta_prop_expr(span, kind).into())
    }
    #[inline]
    pub fn var_decl_or_expr_await_expr(&mut self, span: Span, arg: Expr) -> VarDeclOrExpr {
        VarDeclOrExpr::Await(self.await_expr(span, arg).into())
    }
    #[inline]
    pub fn var_decl_or_expr_paren_expr(&mut self, span: Span, expr: Expr) -> VarDeclOrExpr {
        VarDeclOrExpr::Paren(self.paren_expr(span, expr).into())
    }
    #[inline]
    pub fn var_decl_or_expr_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::JSXMember(self.jsx_member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn var_decl_or_expr_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::JSXNamespacedName(self.jsx_namespaced_name(span, ns, name).into())
    }
    #[inline]
    pub fn var_decl_or_expr_jsx_empty_expr(&mut self, span: Span) -> VarDeclOrExpr {
        VarDeclOrExpr::JSXEmpty(self.jsx_empty_expr(span).into())
    }
    #[inline]
    pub fn var_decl_or_expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::JSXElement(self.jsx_element(span, opening, children, closing).into())
    }
    #[inline]
    pub fn var_decl_or_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::JSXFragment(self.jsx_fragment(span, opening, children, closing).into())
    }
    #[inline]
    pub fn var_decl_or_expr_private_name(&mut self, span: Span, name: Atom<'a>) -> VarDeclOrExpr {
        VarDeclOrExpr::PrivateName(self.private_name(span, name).into())
    }
    #[inline]
    pub fn var_decl_or_expr_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> VarDeclOrExpr {
        VarDeclOrExpr::OptChain(self.opt_chain_expr(span, optional, base).into())
    }
    #[inline]
    pub fn var_decl_or_expr_invalid(&mut self, span: Span) -> VarDeclOrExpr {
        VarDeclOrExpr::Invalid(self.invalid(span).into())
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
            NodePayload(
                0u128
                    | ((ident.node_id().index() as u32 as u128) << 0usize)
                    | ((declare as u32 as u128) << 64usize)
                    | ((function.node_id().index() as u32 as u128) << 32usize),
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
            NodePayload(
                0u128
                    | ((ident.node_id().index() as u32 as u128) << 0usize)
                    | ((declare as u32 as u128) << 64usize)
                    | ((class.node_id().index() as u32 as u128) << 32usize),
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
            NodePayload(
                0u128
                    | ((kind as u32 as u128) << 64usize)
                    | ((declare as u32 as u128) << 72usize)
                    | ((unsafe { decls.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn var_declarator(&mut self, span: Span, name: Pat, init: Option<Expr>) -> VarDeclarator {
        VarDeclarator(self.add_node(
            span,
            NodeKind::VarDeclarator,
            NodePayload(
                0u128
                    | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { init.to_extra_data(self).other } as u128) << 64usize),
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
            NodePayload(
                0u128
                    | ((is_await as u32 as u128) << 64usize)
                    | ((unsafe { decls.to_extra_data(self).other } as u128) << 0usize),
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
    pub fn expr_str(&mut self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Expr {
        Expr::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn expr_bool(&mut self, span: Span, value: bool) -> Expr {
        Expr::Bool(self.bool(span, value).into())
    }
    #[inline]
    pub fn expr_null(&mut self, span: Span) -> Expr {
        Expr::Null(self.null(span).into())
    }
    #[inline]
    pub fn expr_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Expr {
        Expr::Num(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn expr_big_int(&mut self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> Expr {
        Expr::BigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn expr_regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Expr {
        Expr::Regex(self.regex(span, exp, flags).into())
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
        ThisExpr(self.add_node(span, NodeKind::ThisExpr, NodePayload(0)))
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
            NodePayload(0u128 | ((unsafe { elems.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn object_lit(&mut self, span: Span, props: TypedSubRange<PropOrSpread>) -> ObjectLit {
        ObjectLit(self.add_node(
            span,
            NodeKind::ObjectLit,
            NodePayload(0u128 | ((unsafe { props.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn prop_or_spread_ident(&mut self, span: Span, sym: Atom<'a>) -> PropOrSpread {
        PropOrSpread::Shorthand(self.ident(span, sym).into())
    }
    #[inline]
    pub fn prop_or_spread_key_value_prop(
        &mut self,
        span: Span,
        key: PropName,
        value: Expr,
    ) -> PropOrSpread {
        PropOrSpread::KeyValue(self.key_value_prop(span, key, value).into())
    }
    #[inline]
    pub fn prop_or_spread_assign_prop(
        &mut self,
        span: Span,
        key: Ident,
        value: Expr,
    ) -> PropOrSpread {
        PropOrSpread::Assign(self.assign_prop(span, key, value).into())
    }
    #[inline]
    pub fn prop_or_spread_getter_prop(
        &mut self,
        span: Span,
        key: PropName,
        function: Function,
    ) -> PropOrSpread {
        PropOrSpread::Getter(self.getter_prop(span, key, function).into())
    }
    #[inline]
    pub fn prop_or_spread_setter_prop(
        &mut self,
        span: Span,
        key: PropName,
        function: Function,
    ) -> PropOrSpread {
        PropOrSpread::Setter(self.setter_prop(span, key, function).into())
    }
    #[inline]
    pub fn prop_or_spread_method_prop(
        &mut self,
        span: Span,
        key: PropName,
        function: Function,
    ) -> PropOrSpread {
        PropOrSpread::Method(self.method_prop(span, key, function).into())
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
    pub fn spread_element(&mut self, span: Span, dot3_token: Span, expr: Expr) -> SpreadElement {
        SpreadElement(self.add_node(
            span,
            NodeKind::SpreadElement,
            NodePayload(
                0u128
                    | ((unsafe { dot3_token.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { expr.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> UnaryExpr {
        UnaryExpr(self.add_node(
            span,
            NodeKind::UnaryExpr,
            NodePayload(
                0u128
                    | ((op as u32 as u128) << 64usize)
                    | ((unsafe { arg.to_extra_data(self).other } as u128) << 0usize),
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
            NodePayload(
                0u128
                    | ((op as u32 as u128) << 64usize)
                    | ((prefix as u32 as u128) << 72usize)
                    | ((unsafe { arg.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn bin_expr(&mut self, span: Span, op: BinaryOp, left: Expr, right: Expr) -> BinExpr {
        let _f0 = self.add_extra(right.to_extra_data(self));
        BinExpr(self.add_node(
            span,
            NodeKind::BinExpr,
            NodePayload(
                0u128
                    | ((op as u32 as u128) << 64usize)
                    | ((unsafe { left.to_extra_data(self).other } as u128) << 0usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn fn_expr(&mut self, span: Span, ident: Option<Ident>, function: Function) -> FnExpr {
        FnExpr(self.add_node(
            span,
            NodeKind::FnExpr,
            NodePayload(
                0u128
                    | ((ident.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize)
                    | ((function.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn class_expr(&mut self, span: Span, ident: Option<Ident>, class: Class) -> ClassExpr {
        ClassExpr(self.add_node(
            span,
            NodeKind::ClassExpr,
            NodePayload(
                0u128
                    | ((ident.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 0usize)
                    | ((class.node_id().index() as u32 as u128) << 32usize),
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
        let _f0 = self.add_extra(right.to_extra_data(self));
        AssignExpr(self.add_node(
            span,
            NodeKind::AssignExpr,
            NodePayload(
                0u128
                    | ((op as u32 as u128) << 64usize)
                    | ((unsafe { left.to_extra_data(self).other } as u128) << 0usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn member_expr(&mut self, span: Span, obj: Expr, prop: MemberProp) -> MemberExpr {
        MemberExpr(self.add_node(
            span,
            NodeKind::MemberExpr,
            NodePayload(
                0u128
                    | ((unsafe { obj.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { prop.to_extra_data(self).other } as u128) << 64usize),
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
            NodePayload(
                0u128
                    | ((obj.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { prop.to_extra_data(self).other } as u128) << 0usize),
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
        let _extra_data = [cons.to_extra_data(self), alt.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        CondExpr(self.add_node(
            span,
            NodeKind::CondExpr,
            NodePayload(
                0u128
                    | ((unsafe { test.to_extra_data(self).other } as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
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
        let _extra_data = [source.to_extra_data(self), options.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ImportExpr(self.add_node(
            span,
            NodeKind::ImportExpr,
            NodePayload(
                0u128
                    | ((unsafe { import_span.to_extra_data(self).other } as u128) << 0usize)
                    | ((phase as u32 as u128) << 64usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
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
            NodePayload(
                0u128
                    | ((unsafe { callee.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { args.to_extra_data(self).other } as u128) << 64usize),
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
            NodePayload(
                0u128
                    | ((unsafe { callee.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { args.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> SeqExpr {
        SeqExpr(self.add_node(
            span,
            NodeKind::SeqExpr,
            NodePayload(0u128 | ((unsafe { exprs.to_extra_data(self).other } as u128) << 0usize)),
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
        ArrowExpr(self.add_node(
            span,
            NodeKind::ArrowExpr,
            NodePayload(
                0u128
                    | ((unsafe { params.to_extra_data(self).other } as u128) << 0usize)
                    | ((is_async as u32 as u128) << 64usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn yield_expr(&mut self, span: Span, arg: Option<Expr>, delegate: bool) -> YieldExpr {
        YieldExpr(self.add_node(
            span,
            NodeKind::YieldExpr,
            NodePayload(
                0u128
                    | ((unsafe { arg.to_extra_data(self).other } as u128) << 0usize)
                    | ((delegate as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> MetaPropExpr {
        MetaPropExpr(self.add_node(
            span,
            NodeKind::MetaPropExpr,
            NodePayload(0u128 | ((kind as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn await_expr(&mut self, span: Span, arg: Expr) -> AwaitExpr {
        AwaitExpr(self.add_node(
            span,
            NodeKind::AwaitExpr,
            NodePayload(0u128 | ((unsafe { arg.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> Tpl {
        Tpl(self.add_node(
            span,
            NodeKind::Tpl,
            NodePayload(
                0u128
                    | ((unsafe { exprs.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { quasis.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> TaggedTpl {
        TaggedTpl(self.add_node(
            span,
            NodeKind::TaggedTpl,
            NodePayload(
                0u128
                    | ((unsafe { tag.to_extra_data(self).other } as u128) << 0usize)
                    | ((tpl.node_id().index() as u32 as u128) << 64usize),
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
        let _f0 = self.add_extra(raw.to_extra_data(self));
        TplElement(self.add_node(
            span,
            NodeKind::TplElement,
            NodePayload(
                0u128
                    | ((tail as u32 as u128) << 64usize)
                    | ((unsafe { cooked.to_extra_data(self).other } as u128) << 0usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn paren_expr(&mut self, span: Span, expr: Expr) -> ParenExpr {
        ParenExpr(self.add_node(
            span,
            NodeKind::ParenExpr,
            NodePayload(0u128 | ((unsafe { expr.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn callee_this_expr(&mut self, span: Span) -> Callee {
        Callee::This(self.this_expr(span).into())
    }
    #[inline]
    pub fn callee_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> Callee {
        Callee::Array(self.array_lit(span, elems).into())
    }
    #[inline]
    pub fn callee_object_lit(&mut self, span: Span, props: TypedSubRange<PropOrSpread>) -> Callee {
        Callee::Object(self.object_lit(span, props).into())
    }
    #[inline]
    pub fn callee_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> Callee {
        Callee::Fn(self.fn_expr(span, ident, function).into())
    }
    #[inline]
    pub fn callee_unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> Callee {
        Callee::Unary(self.unary_expr(span, op, arg).into())
    }
    #[inline]
    pub fn callee_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> Callee {
        Callee::Update(self.update_expr(span, op, prefix, arg).into())
    }
    #[inline]
    pub fn callee_bin_expr(&mut self, span: Span, op: BinaryOp, left: Expr, right: Expr) -> Callee {
        Callee::Bin(self.bin_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn callee_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> Callee {
        Callee::Assign(self.assign_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn callee_member_expr(&mut self, span: Span, obj: Expr, prop: MemberProp) -> Callee {
        Callee::Member(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn callee_super_prop_expr(&mut self, span: Span, obj: Super, prop: SuperProp) -> Callee {
        Callee::SuperProp(self.super_prop_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn callee_cond_expr(&mut self, span: Span, test: Expr, cons: Expr, alt: Expr) -> Callee {
        Callee::Cond(self.cond_expr(span, test, cons, alt).into())
    }
    #[inline]
    pub fn callee_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> Callee {
        Callee::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        )
    }
    #[inline]
    pub fn callee_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Callee {
        Callee::Call(self.call_expr(span, callee, args).into())
    }
    #[inline]
    pub fn callee_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Callee {
        Callee::New(self.new_expr(span, callee, args).into())
    }
    #[inline]
    pub fn callee_seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> Callee {
        Callee::Seq(self.seq_expr(span, exprs).into())
    }
    #[inline]
    pub fn callee_ident(&mut self, span: Span, sym: Atom<'a>) -> Callee {
        Callee::Ident(self.ident(span, sym).into())
    }
    #[inline]
    pub fn callee_str(&mut self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Callee {
        Callee::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn callee_bool(&mut self, span: Span, value: bool) -> Callee {
        Callee::Bool(self.bool(span, value).into())
    }
    #[inline]
    pub fn callee_null(&mut self, span: Span) -> Callee {
        Callee::Null(self.null(span).into())
    }
    #[inline]
    pub fn callee_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Callee {
        Callee::Num(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn callee_big_int(&mut self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> Callee {
        Callee::BigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn callee_regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Callee {
        Callee::Regex(self.regex(span, exp, flags).into())
    }
    #[inline]
    pub fn callee_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> Callee {
        Callee::Tpl(self.tpl(span, exprs, quasis).into())
    }
    #[inline]
    pub fn callee_tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> Callee {
        Callee::TaggedTpl(self.tagged_tpl(span, tag, tpl).into())
    }
    #[inline]
    pub fn callee_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> Callee {
        Callee::Arrow(self.arrow_expr(span, params, body, is_async).into())
    }
    #[inline]
    pub fn callee_class_expr(&mut self, span: Span, ident: Option<Ident>, class: Class) -> Callee {
        Callee::Class(self.class_expr(span, ident, class).into())
    }
    #[inline]
    pub fn callee_yield_expr(&mut self, span: Span, arg: Option<Expr>, delegate: bool) -> Callee {
        Callee::Yield(self.yield_expr(span, arg, delegate).into())
    }
    #[inline]
    pub fn callee_meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> Callee {
        Callee::MetaProp(self.meta_prop_expr(span, kind).into())
    }
    #[inline]
    pub fn callee_await_expr(&mut self, span: Span, arg: Expr) -> Callee {
        Callee::Await(self.await_expr(span, arg).into())
    }
    #[inline]
    pub fn callee_paren_expr(&mut self, span: Span, expr: Expr) -> Callee {
        Callee::Paren(self.paren_expr(span, expr).into())
    }
    #[inline]
    pub fn callee_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> Callee {
        Callee::JSXMember(self.jsx_member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn callee_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> Callee {
        Callee::JSXNamespacedName(self.jsx_namespaced_name(span, ns, name).into())
    }
    #[inline]
    pub fn callee_jsx_empty_expr(&mut self, span: Span) -> Callee {
        Callee::JSXEmpty(self.jsx_empty_expr(span).into())
    }
    #[inline]
    pub fn callee_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> Callee {
        Callee::JSXElement(self.jsx_element(span, opening, children, closing).into())
    }
    #[inline]
    pub fn callee_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> Callee {
        Callee::JSXFragment(self.jsx_fragment(span, opening, children, closing).into())
    }
    #[inline]
    pub fn callee_private_name(&mut self, span: Span, name: Atom<'a>) -> Callee {
        Callee::PrivateName(self.private_name(span, name).into())
    }
    #[inline]
    pub fn callee_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> Callee {
        Callee::OptChain(self.opt_chain_expr(span, optional, base).into())
    }
    #[inline]
    pub fn callee_invalid(&mut self, span: Span) -> Callee {
        Callee::Invalid(self.invalid(span).into())
    }
    #[inline]
    pub fn callee_super(&mut self, span: Span) -> Callee {
        Callee::Super(self.super_(span).into())
    }
    #[inline]
    pub fn super_(&mut self, span: Span) -> Super {
        Super(self.add_node(span, NodeKind::Super, NodePayload(0)))
    }
    #[inline]
    pub fn expr_or_spread(&mut self, span: Span, spread: Option<Span>, expr: Expr) -> ExprOrSpread {
        ExprOrSpread(self.add_node(
            span,
            NodeKind::ExprOrSpread,
            NodePayload(
                0u128
                    | ((unsafe { spread.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { expr.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn block_stmt_or_expr_this_expr(&mut self, span: Span) -> BlockStmtOrExpr {
        BlockStmtOrExpr::This(self.this_expr(span).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Array(self.array_lit(span, elems).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_object_lit(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Object(self.object_lit(span, props).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Fn(self.fn_expr(span, ident, function).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_unary_expr(
        &mut self,
        span: Span,
        op: UnaryOp,
        arg: Expr,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Unary(self.unary_expr(span, op, arg).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Update(self.update_expr(span, op, prefix, arg).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_bin_expr(
        &mut self,
        span: Span,
        op: BinaryOp,
        left: Expr,
        right: Expr,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Bin(self.bin_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Assign(self.assign_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Member(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::SuperProp(self.super_prop_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_cond_expr(
        &mut self,
        span: Span,
        test: Expr,
        cons: Expr,
        alt: Expr,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Cond(self.cond_expr(span, test, cons, alt).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        )
    }
    #[inline]
    pub fn block_stmt_or_expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Call(self.call_expr(span, callee, args).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::New(self.new_expr(span, callee, args).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_seq_expr(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Seq(self.seq_expr(span, exprs).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_ident(&mut self, span: Span, sym: Atom<'a>) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Ident(self.ident(span, sym).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_bool(&mut self, span: Span, value: bool) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Bool(self.bool(span, value).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_null(&mut self, span: Span) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Null(self.null(span).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_number(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<Atom<'a>>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Num(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::BigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_regex(
        &mut self,
        span: Span,
        exp: Atom<'a>,
        flags: Atom<'a>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Regex(self.regex(span, exp, flags).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Tpl(self.tpl(span, exprs, quasis).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_tagged_tpl(
        &mut self,
        span: Span,
        tag: Expr,
        tpl: Tpl,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::TaggedTpl(self.tagged_tpl(span, tag, tpl).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Arrow(self.arrow_expr(span, params, body, is_async).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Class(self.class_expr(span, ident, class).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_yield_expr(
        &mut self,
        span: Span,
        arg: Option<Expr>,
        delegate: bool,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Yield(self.yield_expr(span, arg, delegate).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_meta_prop_expr(
        &mut self,
        span: Span,
        kind: MetaPropKind,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::MetaProp(self.meta_prop_expr(span, kind).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_await_expr(&mut self, span: Span, arg: Expr) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Await(self.await_expr(span, arg).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_paren_expr(&mut self, span: Span, expr: Expr) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Paren(self.paren_expr(span, expr).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::JSXMember(self.jsx_member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::JSXNamespacedName(self.jsx_namespaced_name(span, ns, name).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_jsx_empty_expr(&mut self, span: Span) -> BlockStmtOrExpr {
        BlockStmtOrExpr::JSXEmpty(self.jsx_empty_expr(span).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::JSXElement(self.jsx_element(span, opening, children, closing).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::JSXFragment(self.jsx_fragment(span, opening, children, closing).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_private_name(
        &mut self,
        span: Span,
        name: Atom<'a>,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::PrivateName(self.private_name(span, name).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> BlockStmtOrExpr {
        BlockStmtOrExpr::OptChain(self.opt_chain_expr(span, optional, base).into())
    }
    #[inline]
    pub fn block_stmt_or_expr_invalid(&mut self, span: Span) -> BlockStmtOrExpr {
        BlockStmtOrExpr::Invalid(self.invalid(span).into())
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
    pub fn assign_target_binding_ident(&mut self, span: Span, id: Ident) -> AssignTarget {
        AssignTarget::SimpleIdent(self.binding_ident(span, id).into())
    }
    #[inline]
    pub fn assign_target_member_expr(
        &mut self,
        span: Span,
        obj: Expr,
        prop: MemberProp,
    ) -> AssignTarget {
        AssignTarget::SimpleMember(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn assign_target_super_prop_expr(
        &mut self,
        span: Span,
        obj: Super,
        prop: SuperProp,
    ) -> AssignTarget {
        AssignTarget::SimpleSuperProp(self.super_prop_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn assign_target_paren_expr(&mut self, span: Span, expr: Expr) -> AssignTarget {
        AssignTarget::SimpleParen(self.paren_expr(span, expr).into())
    }
    #[inline]
    pub fn assign_target_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> AssignTarget {
        AssignTarget::SimpleOptChain(self.opt_chain_expr(span, optional, base).into())
    }
    #[inline]
    pub fn assign_target_simple_invalid(&mut self, span: Span) -> AssignTarget {
        AssignTarget::SimpleInvalid(self.invalid(span).into())
    }
    #[inline]
    pub fn assign_target_array_pat(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<Pat>>,
        optional: bool,
    ) -> AssignTarget {
        AssignTarget::PatArray(self.array_pat(span, elems, optional).into())
    }
    #[inline]
    pub fn assign_target_object_pat(
        &mut self,
        span: Span,
        props: TypedSubRange<ObjectPatProp>,
        optional: bool,
    ) -> AssignTarget {
        AssignTarget::PatObject(self.object_pat(span, props, optional).into())
    }
    #[inline]
    pub fn assign_target_pattern_invalid(&mut self, span: Span) -> AssignTarget {
        AssignTarget::PatternInvalid(self.invalid(span).into())
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
            NodePayload(
                0u128
                    | ((optional as u32 as u128) << 64usize)
                    | ((unsafe { base.to_extra_data(self).other } as u128) << 0usize),
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
            NodePayload(
                0u128
                    | ((unsafe { callee.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { args.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn invalid(&mut self, span: Span) -> Invalid {
        Invalid(self.add_node(span, NodeKind::Invalid, NodePayload(0)))
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
        let _extra_data = [
            decorators.to_extra_data(self),
            is_generator.to_extra_data(self),
            is_async.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        Function(self.add_node(
            span,
            NodeKind::Function,
            NodePayload(
                0u128
                    | ((unsafe { params.to_extra_data(self).other } as u128) << 0usize)
                    | ((body.node_id().index() as u32 as u128) << 64usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn param(&mut self, span: Span, decorators: TypedSubRange<Decorator>, pat: Pat) -> Param {
        Param(self.add_node(
            span,
            NodeKind::Param,
            NodePayload(
                0u128
                    | ((unsafe { decorators.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { pat.to_extra_data(self).other } as u128) << 64usize),
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
        let _extra_data = [body.to_extra_data(self), super_class.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        Class(self.add_node(
            span,
            NodeKind::Class,
            NodePayload(
                0u128
                    | ((unsafe { decorators.to_extra_data(self).other } as u128) << 0usize)
                    | ((is_abstract as u32 as u128) << 64usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
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
        let _extra_data = [value.to_extra_data(self), decorators.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        ClassProp(self.add_node(
            span,
            NodeKind::ClassProp,
            NodePayload(
                0u128
                    | ((unsafe { key.to_extra_data(self).other } as u128) << 0usize)
                    | ((is_static as u32 as u128) << 64usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
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
        let _extra_data = [
            is_static.to_extra_data(self),
            decorators.to_extra_data(self),
        ];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        PrivateProp(self.add_node(
            span,
            NodeKind::PrivateProp,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
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
            NodePayload(
                0u128
                    | ((unsafe { key.to_extra_data(self).other } as u128) << 0usize)
                    | ((function.node_id().index() as u32 as u128) << 64usize)
                    | ((kind as u32 as u128) << 96usize)
                    | ((is_static as u32 as u128) << 104usize),
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
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 0usize)
                    | ((function.node_id().index() as u32 as u128) << 32usize)
                    | ((kind as u32 as u128) << 64usize)
                    | ((is_static as u32 as u128) << 72usize),
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
        let _f0 = self.add_extra(params.to_extra_data(self));
        Constructor(self.add_node(
            span,
            NodeKind::Constructor,
            NodePayload(
                0u128
                    | ((unsafe { key.to_extra_data(self).other } as u128) << 0usize)
                    | ((body.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 64usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn decorator(&mut self, span: Span, expr: Expr) -> Decorator {
        Decorator(self.add_node(
            span,
            NodeKind::Decorator,
            NodePayload(0u128 | ((unsafe { expr.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn static_block(&mut self, span: Span, body: BlockStmt) -> StaticBlock {
        StaticBlock(self.add_node(
            span,
            NodeKind::StaticBlock,
            NodePayload(0u128 | ((body.node_id().index() as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn key_ident_name(&mut self, span: Span, sym: Atom<'a>) -> Key {
        Key::Ident(self.ident_name(span, sym).into())
    }
    #[inline]
    pub fn key_str(&mut self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Key {
        Key::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn key_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Key {
        Key::Num(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn key_computed_prop_name(&mut self, span: Span, expr: Expr) -> Key {
        Key::Computed(self.computed_prop_name(span, expr).into())
    }
    #[inline]
    pub fn key_big_int(&mut self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> Key {
        Key::BigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn key_private_name(&mut self, span: Span, name: Atom<'a>) -> Key {
        Key::Private(self.private_name(span, name).into())
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
        let _extra_data = [value.to_extra_data(self), decorators.to_extra_data(self)];
        let _extra_data_range = self.add_extra_data_sub_range(&_extra_data);
        AutoAccessor(self.add_node(
            span,
            NodeKind::AutoAccessor,
            NodePayload(
                0u128
                    | ((unsafe { key.to_extra_data(self).other } as u128) << 0usize)
                    | ((is_static as u32 as u128) << 64usize)
                    | ((_extra_data_range.start.index() as u128) << 96usize),
            ),
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
            NodePayload(
                0u128
                    | ((unsafe { key.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn assign_prop(&mut self, span: Span, key: Ident, value: Expr) -> AssignProp {
        AssignProp(self.add_node(
            span,
            NodeKind::AssignProp,
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn getter_prop(&mut self, span: Span, key: PropName, function: Function) -> GetterProp {
        GetterProp(self.add_node(
            span,
            NodeKind::GetterProp,
            NodePayload(
                0u128
                    | ((unsafe { key.to_extra_data(self).other } as u128) << 0usize)
                    | ((function.node_id().index() as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn setter_prop(&mut self, span: Span, key: PropName, function: Function) -> SetterProp {
        SetterProp(self.add_node(
            span,
            NodeKind::SetterProp,
            NodePayload(
                0u128
                    | ((unsafe { key.to_extra_data(self).other } as u128) << 0usize)
                    | ((function.node_id().index() as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn method_prop(&mut self, span: Span, key: PropName, function: Function) -> MethodProp {
        MethodProp(self.add_node(
            span,
            NodeKind::MethodProp,
            NodePayload(
                0u128
                    | ((unsafe { key.to_extra_data(self).other } as u128) << 0usize)
                    | ((function.node_id().index() as u32 as u128) << 64usize),
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
            NodePayload(0u128 | ((unsafe { expr.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn pat_this_expr(&mut self, span: Span) -> Pat {
        Pat::ExprThis(self.this_expr(span).into())
    }
    #[inline]
    pub fn pat_array_lit(&mut self, span: Span, elems: TypedSubRange<Option<ExprOrSpread>>) -> Pat {
        Pat::ExprArray(self.array_lit(span, elems).into())
    }
    #[inline]
    pub fn pat_object_lit(&mut self, span: Span, props: TypedSubRange<PropOrSpread>) -> Pat {
        Pat::ExprObject(self.object_lit(span, props).into())
    }
    #[inline]
    pub fn pat_fn_expr(&mut self, span: Span, ident: Option<Ident>, function: Function) -> Pat {
        Pat::ExprFn(self.fn_expr(span, ident, function).into())
    }
    #[inline]
    pub fn pat_unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> Pat {
        Pat::ExprUnary(self.unary_expr(span, op, arg).into())
    }
    #[inline]
    pub fn pat_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> Pat {
        Pat::ExprUpdate(self.update_expr(span, op, prefix, arg).into())
    }
    #[inline]
    pub fn pat_bin_expr(&mut self, span: Span, op: BinaryOp, left: Expr, right: Expr) -> Pat {
        Pat::ExprBin(self.bin_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn pat_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> Pat {
        Pat::ExprAssign(self.assign_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn pat_member_expr(&mut self, span: Span, obj: Expr, prop: MemberProp) -> Pat {
        Pat::ExprMember(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn pat_super_prop_expr(&mut self, span: Span, obj: Super, prop: SuperProp) -> Pat {
        Pat::ExprSuperProp(self.super_prop_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn pat_cond_expr(&mut self, span: Span, test: Expr, cons: Expr, alt: Expr) -> Pat {
        Pat::ExprCond(self.cond_expr(span, test, cons, alt).into())
    }
    #[inline]
    pub fn pat_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> Pat {
        Pat::ExprImport(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        )
    }
    #[inline]
    pub fn pat_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Pat {
        Pat::ExprCall(self.call_expr(span, callee, args).into())
    }
    #[inline]
    pub fn pat_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> Pat {
        Pat::ExprNew(self.new_expr(span, callee, args).into())
    }
    #[inline]
    pub fn pat_seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> Pat {
        Pat::ExprSeq(self.seq_expr(span, exprs).into())
    }
    #[inline]
    pub fn pat_ident(&mut self, span: Span, sym: Atom<'a>) -> Pat {
        Pat::ExprIdent(self.ident(span, sym).into())
    }
    #[inline]
    pub fn pat_str(&mut self, span: Span, value: Wtf8Atom<'a>, raw: Option<Atom<'a>>) -> Pat {
        Pat::ExprStr(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn pat_bool(&mut self, span: Span, value: bool) -> Pat {
        Pat::ExprBool(self.bool(span, value).into())
    }
    #[inline]
    pub fn pat_null(&mut self, span: Span) -> Pat {
        Pat::ExprNull(self.null(span).into())
    }
    #[inline]
    pub fn pat_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Pat {
        Pat::ExprNum(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn pat_big_int(&mut self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> Pat {
        Pat::ExprBigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn pat_regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Pat {
        Pat::ExprRegex(self.regex(span, exp, flags).into())
    }
    #[inline]
    pub fn pat_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> Pat {
        Pat::ExprTpl(self.tpl(span, exprs, quasis).into())
    }
    #[inline]
    pub fn pat_tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> Pat {
        Pat::ExprTaggedTpl(self.tagged_tpl(span, tag, tpl).into())
    }
    #[inline]
    pub fn pat_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> Pat {
        Pat::ExprArrow(self.arrow_expr(span, params, body, is_async).into())
    }
    #[inline]
    pub fn pat_class_expr(&mut self, span: Span, ident: Option<Ident>, class: Class) -> Pat {
        Pat::ExprClass(self.class_expr(span, ident, class).into())
    }
    #[inline]
    pub fn pat_yield_expr(&mut self, span: Span, arg: Option<Expr>, delegate: bool) -> Pat {
        Pat::ExprYield(self.yield_expr(span, arg, delegate).into())
    }
    #[inline]
    pub fn pat_meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> Pat {
        Pat::ExprMetaProp(self.meta_prop_expr(span, kind).into())
    }
    #[inline]
    pub fn pat_await_expr(&mut self, span: Span, arg: Expr) -> Pat {
        Pat::ExprAwait(self.await_expr(span, arg).into())
    }
    #[inline]
    pub fn pat_paren_expr(&mut self, span: Span, expr: Expr) -> Pat {
        Pat::ExprParen(self.paren_expr(span, expr).into())
    }
    #[inline]
    pub fn pat_jsx_member_expr(&mut self, span: Span, obj: JSXObject, prop: IdentName) -> Pat {
        Pat::ExprJSXMember(self.jsx_member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn pat_jsx_namespaced_name(&mut self, span: Span, ns: IdentName, name: IdentName) -> Pat {
        Pat::ExprJSXNamespacedName(self.jsx_namespaced_name(span, ns, name).into())
    }
    #[inline]
    pub fn pat_jsx_empty_expr(&mut self, span: Span) -> Pat {
        Pat::ExprJSXEmpty(self.jsx_empty_expr(span).into())
    }
    #[inline]
    pub fn pat_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> Pat {
        Pat::ExprJSXElement(self.jsx_element(span, opening, children, closing).into())
    }
    #[inline]
    pub fn pat_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> Pat {
        Pat::ExprJSXFragment(self.jsx_fragment(span, opening, children, closing).into())
    }
    #[inline]
    pub fn pat_private_name(&mut self, span: Span, name: Atom<'a>) -> Pat {
        Pat::ExprPrivateName(self.private_name(span, name).into())
    }
    #[inline]
    pub fn pat_opt_chain_expr(&mut self, span: Span, optional: bool, base: OptChainBase) -> Pat {
        Pat::ExprOptChain(self.opt_chain_expr(span, optional, base).into())
    }
    #[inline]
    pub fn pat_expr_invalid(&mut self, span: Span) -> Pat {
        Pat::ExprInvalid(self.invalid(span).into())
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
    pub fn array_pat(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<Pat>>,
        optional: bool,
    ) -> ArrayPat {
        ArrayPat(self.add_node(
            span,
            NodeKind::ArrayPat,
            NodePayload(
                0u128
                    | ((unsafe { elems.to_extra_data(self).other } as u128) << 0usize)
                    | ((optional as u32 as u128) << 64usize),
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
            NodePayload(
                0u128
                    | ((unsafe { props.to_extra_data(self).other } as u128) << 0usize)
                    | ((optional as u32 as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn assign_pat(&mut self, span: Span, left: Pat, right: Expr) -> AssignPat {
        AssignPat(self.add_node(
            span,
            NodeKind::AssignPat,
            NodePayload(
                0u128
                    | ((unsafe { left.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { right.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn rest_pat(&mut self, span: Span, dot3_token: Span, arg: Pat) -> RestPat {
        RestPat(self.add_node(
            span,
            NodeKind::RestPat,
            NodePayload(
                0u128
                    | ((unsafe { dot3_token.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { arg.to_extra_data(self).other } as u128) << 64usize),
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
            NodePayload(
                0u128
                    | ((unsafe { key.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 64usize),
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
            NodePayload(
                0u128
                    | ((key.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize),
            ),
        ))
    }
    #[inline]
    pub fn ident(&mut self, span: Span, sym: Atom<'a>) -> Ident {
        let _inline_default_1 = <Option<SymbolId>>::default();
        Ident(self.add_node(
            span,
            NodeKind::Ident,
            NodePayload(
                0u128
                    | ((unsafe { sym.to_extra_data(self).other } as u128) << 0usize)
                    | ((_inline_default_1.map_or(0, |id| id.index() as u32 + 1) as u128)
                        << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn ident_name(&mut self, span: Span, sym: Atom<'a>) -> IdentName {
        IdentName(self.add_node(
            span,
            NodeKind::IdentName,
            NodePayload(0u128 | ((unsafe { sym.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn private_name(&mut self, span: Span, name: Atom<'a>) -> PrivateName {
        PrivateName(self.add_node(
            span,
            NodeKind::PrivateName,
            NodePayload(0u128 | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn binding_ident(&mut self, span: Span, id: Ident) -> BindingIdent {
        BindingIdent(self.add_node(
            span,
            NodeKind::BindingIdent,
            NodePayload(0u128 | ((id.node_id().index() as u32 as u128) << 0usize)),
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
        Str(self.add_node(
            span,
            NodeKind::Str,
            NodePayload(
                0u128
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { raw.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn bool(&mut self, span: Span, value: bool) -> Bool {
        Bool(self.add_node(
            span,
            NodeKind::Bool,
            NodePayload(0u128 | ((value as u32 as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn null(&mut self, span: Span) -> Null {
        Null(self.add_node(span, NodeKind::Null, NodePayload(0)))
    }
    #[inline]
    pub fn number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> Number {
        Number(self.add_node(
            span,
            NodeKind::Number,
            NodePayload(
                0u128
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { raw.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn big_int(&mut self, span: Span, value: Atom<'a>, raw: Option<Atom<'a>>) -> BigInt {
        BigInt(self.add_node(
            span,
            NodeKind::BigInt,
            NodePayload(
                0u128
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { raw.to_extra_data(self).other } as u128) << 64usize),
            ),
        ))
    }
    #[inline]
    pub fn regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> Regex {
        Regex(self.add_node(
            span,
            NodeKind::Regex,
            NodePayload(
                0u128
                    | ((unsafe { exp.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { flags.to_extra_data(self).other } as u128) << 64usize),
            ),
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
            NodePayload(
                0u128
                    | ((unsafe { obj.to_extra_data(self).other } as u128) << 0usize)
                    | ((prop.node_id().index() as u32 as u128) << 64usize),
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
            NodePayload(
                0u128
                    | ((ns.node_id().index() as u32 as u128) << 0usize)
                    | ((name.node_id().index() as u32 as u128) << 32usize),
            ),
        ))
    }
    #[inline]
    pub fn jsx_empty_expr(&mut self, span: Span) -> JSXEmptyExpr {
        JSXEmptyExpr(self.add_node(span, NodeKind::JSXEmptyExpr, NodePayload(0)))
    }
    #[inline]
    pub fn jsx_expr_container(&mut self, span: Span, expr: JSXExpr) -> JSXExprContainer {
        JSXExprContainer(self.add_node(
            span,
            NodeKind::JSXExprContainer,
            NodePayload(0u128 | ((unsafe { expr.to_extra_data(self).other } as u128) << 0usize)),
        ))
    }
    #[inline]
    pub fn jsx_expr_this_expr(&mut self, span: Span) -> JSXExpr {
        JSXExpr::This(self.this_expr(span).into())
    }
    #[inline]
    pub fn jsx_expr_array_lit(
        &mut self,
        span: Span,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) -> JSXExpr {
        JSXExpr::Array(self.array_lit(span, elems).into())
    }
    #[inline]
    pub fn jsx_expr_object_lit(
        &mut self,
        span: Span,
        props: TypedSubRange<PropOrSpread>,
    ) -> JSXExpr {
        JSXExpr::Object(self.object_lit(span, props).into())
    }
    #[inline]
    pub fn jsx_expr_fn_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        function: Function,
    ) -> JSXExpr {
        JSXExpr::Fn(self.fn_expr(span, ident, function).into())
    }
    #[inline]
    pub fn jsx_expr_unary_expr(&mut self, span: Span, op: UnaryOp, arg: Expr) -> JSXExpr {
        JSXExpr::Unary(self.unary_expr(span, op, arg).into())
    }
    #[inline]
    pub fn jsx_expr_update_expr(
        &mut self,
        span: Span,
        op: UpdateOp,
        prefix: bool,
        arg: SimpleAssignTarget,
    ) -> JSXExpr {
        JSXExpr::Update(self.update_expr(span, op, prefix, arg).into())
    }
    #[inline]
    pub fn jsx_expr_bin_expr(
        &mut self,
        span: Span,
        op: BinaryOp,
        left: Expr,
        right: Expr,
    ) -> JSXExpr {
        JSXExpr::Bin(self.bin_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn jsx_expr_assign_expr(
        &mut self,
        span: Span,
        op: AssignOp,
        left: AssignTarget,
        right: Expr,
    ) -> JSXExpr {
        JSXExpr::Assign(self.assign_expr(span, op, left, right).into())
    }
    #[inline]
    pub fn jsx_expr_member_expr(&mut self, span: Span, obj: Expr, prop: MemberProp) -> JSXExpr {
        JSXExpr::Member(self.member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn jsx_expr_super_prop_expr(&mut self, span: Span, obj: Super, prop: SuperProp) -> JSXExpr {
        JSXExpr::SuperProp(self.super_prop_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn jsx_expr_cond_expr(&mut self, span: Span, test: Expr, cons: Expr, alt: Expr) -> JSXExpr {
        JSXExpr::Cond(self.cond_expr(span, test, cons, alt).into())
    }
    #[inline]
    pub fn jsx_expr_import_expr(
        &mut self,
        span: Span,
        import_span: Span,
        source: Expr,
        options: Option<Expr>,
        phase: ImportPhase,
    ) -> JSXExpr {
        JSXExpr::Import(
            self.import_expr(span, import_span, source, options, phase)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expr_call_expr(
        &mut self,
        span: Span,
        callee: Callee,
        args: TypedSubRange<ExprOrSpread>,
    ) -> JSXExpr {
        JSXExpr::Call(self.call_expr(span, callee, args).into())
    }
    #[inline]
    pub fn jsx_expr_new_expr(
        &mut self,
        span: Span,
        callee: Expr,
        args: TypedSubRange<ExprOrSpread>,
    ) -> JSXExpr {
        JSXExpr::New(self.new_expr(span, callee, args).into())
    }
    #[inline]
    pub fn jsx_expr_seq_expr(&mut self, span: Span, exprs: TypedSubRange<Expr>) -> JSXExpr {
        JSXExpr::Seq(self.seq_expr(span, exprs).into())
    }
    #[inline]
    pub fn jsx_expr_ident(&mut self, span: Span, sym: Atom<'a>) -> JSXExpr {
        JSXExpr::Ident(self.ident(span, sym).into())
    }
    #[inline]
    pub fn jsx_expr_str(
        &mut self,
        span: Span,
        value: Wtf8Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> JSXExpr {
        JSXExpr::Str(self.str(span, value, raw).into())
    }
    #[inline]
    pub fn jsx_expr_bool(&mut self, span: Span, value: bool) -> JSXExpr {
        JSXExpr::Bool(self.bool(span, value).into())
    }
    #[inline]
    pub fn jsx_expr_null(&mut self, span: Span) -> JSXExpr {
        JSXExpr::Null(self.null(span).into())
    }
    #[inline]
    pub fn jsx_expr_number(&mut self, span: Span, value: f64, raw: Option<Atom<'a>>) -> JSXExpr {
        JSXExpr::Num(self.number(span, value, raw).into())
    }
    #[inline]
    pub fn jsx_expr_big_int(
        &mut self,
        span: Span,
        value: Atom<'a>,
        raw: Option<Atom<'a>>,
    ) -> JSXExpr {
        JSXExpr::BigInt(self.big_int(span, value, raw).into())
    }
    #[inline]
    pub fn jsx_expr_regex(&mut self, span: Span, exp: Atom<'a>, flags: Atom<'a>) -> JSXExpr {
        JSXExpr::Regex(self.regex(span, exp, flags).into())
    }
    #[inline]
    pub fn jsx_expr_tpl(
        &mut self,
        span: Span,
        exprs: TypedSubRange<Expr>,
        quasis: TypedSubRange<TplElement>,
    ) -> JSXExpr {
        JSXExpr::Tpl(self.tpl(span, exprs, quasis).into())
    }
    #[inline]
    pub fn jsx_expr_tagged_tpl(&mut self, span: Span, tag: Expr, tpl: Tpl) -> JSXExpr {
        JSXExpr::TaggedTpl(self.tagged_tpl(span, tag, tpl).into())
    }
    #[inline]
    pub fn jsx_expr_arrow_expr(
        &mut self,
        span: Span,
        params: TypedSubRange<Pat>,
        body: BlockStmtOrExpr,
        is_async: bool,
    ) -> JSXExpr {
        JSXExpr::Arrow(self.arrow_expr(span, params, body, is_async).into())
    }
    #[inline]
    pub fn jsx_expr_class_expr(
        &mut self,
        span: Span,
        ident: Option<Ident>,
        class: Class,
    ) -> JSXExpr {
        JSXExpr::Class(self.class_expr(span, ident, class).into())
    }
    #[inline]
    pub fn jsx_expr_yield_expr(
        &mut self,
        span: Span,
        arg: Option<Expr>,
        delegate: bool,
    ) -> JSXExpr {
        JSXExpr::Yield(self.yield_expr(span, arg, delegate).into())
    }
    #[inline]
    pub fn jsx_expr_meta_prop_expr(&mut self, span: Span, kind: MetaPropKind) -> JSXExpr {
        JSXExpr::MetaProp(self.meta_prop_expr(span, kind).into())
    }
    #[inline]
    pub fn jsx_expr_await_expr(&mut self, span: Span, arg: Expr) -> JSXExpr {
        JSXExpr::Await(self.await_expr(span, arg).into())
    }
    #[inline]
    pub fn jsx_expr_paren_expr(&mut self, span: Span, expr: Expr) -> JSXExpr {
        JSXExpr::Paren(self.paren_expr(span, expr).into())
    }
    #[inline]
    pub fn jsx_expr_jsx_member_expr(
        &mut self,
        span: Span,
        obj: JSXObject,
        prop: IdentName,
    ) -> JSXExpr {
        JSXExpr::JSXMember(self.jsx_member_expr(span, obj, prop).into())
    }
    #[inline]
    pub fn jsx_expr_jsx_namespaced_name(
        &mut self,
        span: Span,
        ns: IdentName,
        name: IdentName,
    ) -> JSXExpr {
        JSXExpr::JSXNamespacedName(self.jsx_namespaced_name(span, ns, name).into())
    }
    #[inline]
    pub fn jsx_expr_jsx_empty(&mut self, span: Span) -> JSXExpr {
        JSXExpr::JSXEmpty(self.jsx_empty_expr(span).into())
    }
    #[inline]
    pub fn jsx_expr_jsx_element(
        &mut self,
        span: Span,
        opening: JSXOpeningElement,
        children: TypedSubRange<JSXElementChild>,
        closing: Option<JSXClosingElement>,
    ) -> JSXExpr {
        JSXExpr::JSXElement(self.jsx_element(span, opening, children, closing).into())
    }
    #[inline]
    pub fn jsx_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening: JSXOpeningFragment,
        children: TypedSubRange<JSXElementChild>,
        closing: JSXClosingFragment,
    ) -> JSXExpr {
        JSXExpr::JSXFragment(self.jsx_fragment(span, opening, children, closing).into())
    }
    #[inline]
    pub fn jsx_expr_private_name(&mut self, span: Span, name: Atom<'a>) -> JSXExpr {
        JSXExpr::PrivateName(self.private_name(span, name).into())
    }
    #[inline]
    pub fn jsx_expr_opt_chain_expr(
        &mut self,
        span: Span,
        optional: bool,
        base: OptChainBase,
    ) -> JSXExpr {
        JSXExpr::OptChain(self.opt_chain_expr(span, optional, base).into())
    }
    #[inline]
    pub fn jsx_expr_invalid(&mut self, span: Span) -> JSXExpr {
        JSXExpr::Invalid(self.invalid(span).into())
    }
    #[inline]
    pub fn jsx_expr_jsx_empty_expr(&mut self, span: Span) -> JSXExpr {
        JSXExpr::JSXEmptyExpr(self.jsx_empty_expr(span).into())
    }
    #[inline]
    pub fn jsx_spread_child(&mut self, span: Span, expr: Expr) -> JSXSpreadChild {
        JSXSpreadChild(self.add_node(
            span,
            NodeKind::JSXSpreadChild,
            NodePayload(0u128 | ((unsafe { expr.to_extra_data(self).other } as u128) << 0usize)),
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
        let _f0 = self.add_extra(attrs.to_extra_data(self));
        JSXOpeningElement(self.add_node(
            span,
            NodeKind::JSXOpeningElement,
            NodePayload(
                0u128
                    | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)
                    | ((self_closing as u32 as u128) << 64usize)
                    | ((_f0.index() as u128) << 96usize),
            ),
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
            NodePayload(0u128 | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)),
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
            NodePayload(
                0u128
                    | ((unsafe { name.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 64usize),
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
        JSXText(self.add_node(
            span,
            NodeKind::JSXText,
            NodePayload(
                0u128
                    | ((unsafe { value.to_extra_data(self).other } as u128) << 0usize)
                    | ((unsafe { raw.to_extra_data(self).other } as u128) << 64usize),
            ),
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
        JSXElement(self.add_node(
            span,
            NodeKind::JSXElement,
            NodePayload(
                0u128
                    | ((opening.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { children.to_extra_data(self).other } as u128) << 0usize)
                    | ((closing.map_or(u32::MAX, |node| node.node_id().index() as u32) as u128)
                        << 96usize),
            ),
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
        JSXFragment(self.add_node(
            span,
            NodeKind::JSXFragment,
            NodePayload(
                0u128
                    | ((opening.node_id().index() as u32 as u128) << 64usize)
                    | ((unsafe { children.to_extra_data(self).other } as u128) << 0usize)
                    | ((closing.node_id().index() as u32 as u128) << 96usize),
            ),
        ))
    }
    #[inline]
    pub fn jsx_opening_fragment(&mut self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment(self.add_node(span, NodeKind::JSXOpeningFragment, NodePayload(0)))
    }
    #[inline]
    pub fn jsx_closing_fragment(&mut self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment(self.add_node(span, NodeKind::JSXClosingFragment, NodePayload(0)))
    }
}
