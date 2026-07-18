#![allow(
    unused,
    clippy::useless_conversion,
    clippy::identity_op,
    clippy::erasing_op,
    clippy::let_and_return
)]
use crate::*;
use oxc_index::Idx;
impl Program {
    #[inline]
    pub fn is_module(self) -> bool {
        matches!(self, Self::Module(_))
    }
    #[inline]
    pub fn is_script(self) -> bool {
        matches!(self, Self::Script(_))
    }
    #[inline]
    pub fn as_module(self) -> Option<Module> {
        match self {
            Self::Module(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_script(self) -> Option<Script> {
        match self {
            Self::Script(it) => Some(it),
            _ => None,
        }
    }
}
impl Module {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ModuleItem> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn shebang<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<ModuleItem>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_shebang<'a>(self, ast: &mut crate::Ast<'a>, shebang: Option<Atom<'a>>) {
        let field_val: u32 = unsafe { shebang.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl Script {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn shebang<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_shebang<'a>(self, ast: &mut crate::Ast<'a>, shebang: Option<Atom<'a>>) {
        let field_val: u32 = unsafe { shebang.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl ModuleItem {
    #[inline]
    pub fn is_module_decl_import(self) -> bool {
        matches!(self, Self::ModuleDeclImport(_))
    }
    #[inline]
    pub fn is_module_decl_export_decl(self) -> bool {
        matches!(self, Self::ModuleDeclExportDecl(_))
    }
    #[inline]
    pub fn is_module_decl_export_named(self) -> bool {
        matches!(self, Self::ModuleDeclExportNamed(_))
    }
    #[inline]
    pub fn is_module_decl_export_default_decl(self) -> bool {
        matches!(self, Self::ModuleDeclExportDefaultDecl(_))
    }
    #[inline]
    pub fn is_module_decl_export_default_expr(self) -> bool {
        matches!(self, Self::ModuleDeclExportDefaultExpr(_))
    }
    #[inline]
    pub fn is_module_decl_export_all(self) -> bool {
        matches!(self, Self::ModuleDeclExportAll(_))
    }
    #[inline]
    pub fn is_stmt_block(self) -> bool {
        matches!(self, Self::StmtBlock(_))
    }
    #[inline]
    pub fn is_stmt_empty(self) -> bool {
        matches!(self, Self::StmtEmpty(_))
    }
    #[inline]
    pub fn is_stmt_debugger(self) -> bool {
        matches!(self, Self::StmtDebugger(_))
    }
    #[inline]
    pub fn is_stmt_with(self) -> bool {
        matches!(self, Self::StmtWith(_))
    }
    #[inline]
    pub fn is_stmt_return(self) -> bool {
        matches!(self, Self::StmtReturn(_))
    }
    #[inline]
    pub fn is_stmt_labeled(self) -> bool {
        matches!(self, Self::StmtLabeled(_))
    }
    #[inline]
    pub fn is_stmt_break(self) -> bool {
        matches!(self, Self::StmtBreak(_))
    }
    #[inline]
    pub fn is_stmt_continue(self) -> bool {
        matches!(self, Self::StmtContinue(_))
    }
    #[inline]
    pub fn is_stmt_if(self) -> bool {
        matches!(self, Self::StmtIf(_))
    }
    #[inline]
    pub fn is_stmt_switch(self) -> bool {
        matches!(self, Self::StmtSwitch(_))
    }
    #[inline]
    pub fn is_stmt_throw(self) -> bool {
        matches!(self, Self::StmtThrow(_))
    }
    #[inline]
    pub fn is_stmt_try(self) -> bool {
        matches!(self, Self::StmtTry(_))
    }
    #[inline]
    pub fn is_stmt_while(self) -> bool {
        matches!(self, Self::StmtWhile(_))
    }
    #[inline]
    pub fn is_stmt_do_while(self) -> bool {
        matches!(self, Self::StmtDoWhile(_))
    }
    #[inline]
    pub fn is_stmt_for(self) -> bool {
        matches!(self, Self::StmtFor(_))
    }
    #[inline]
    pub fn is_stmt_for_in(self) -> bool {
        matches!(self, Self::StmtForIn(_))
    }
    #[inline]
    pub fn is_stmt_for_of(self) -> bool {
        matches!(self, Self::StmtForOf(_))
    }
    #[inline]
    pub fn is_stmt_decl_class(self) -> bool {
        matches!(self, Self::StmtDeclClass(_))
    }
    #[inline]
    pub fn is_stmt_decl_fn(self) -> bool {
        matches!(self, Self::StmtDeclFn(_))
    }
    #[inline]
    pub fn is_stmt_decl_var(self) -> bool {
        matches!(self, Self::StmtDeclVar(_))
    }
    #[inline]
    pub fn is_stmt_decl_using(self) -> bool {
        matches!(self, Self::StmtDeclUsing(_))
    }
    #[inline]
    pub fn is_stmt_expr(self) -> bool {
        matches!(self, Self::StmtExpr(_))
    }
    #[inline]
    pub fn as_module_decl_import(self) -> Option<ImportDecl> {
        match self {
            Self::ModuleDeclImport(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_module_decl_export_decl(self) -> Option<ExportDecl> {
        match self {
            Self::ModuleDeclExportDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_module_decl_export_named(self) -> Option<NamedExport> {
        match self {
            Self::ModuleDeclExportNamed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_module_decl_export_default_decl(self) -> Option<ExportDefaultDecl> {
        match self {
            Self::ModuleDeclExportDefaultDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_module_decl_export_default_expr(self) -> Option<ExportDefaultExpr> {
        match self {
            Self::ModuleDeclExportDefaultExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_module_decl_export_all(self) -> Option<ExportAll> {
        match self {
            Self::ModuleDeclExportAll(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_block(self) -> Option<BlockStmt> {
        match self {
            Self::StmtBlock(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_empty(self) -> Option<EmptyStmt> {
        match self {
            Self::StmtEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_debugger(self) -> Option<DebuggerStmt> {
        match self {
            Self::StmtDebugger(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_with(self) -> Option<WithStmt> {
        match self {
            Self::StmtWith(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_return(self) -> Option<ReturnStmt> {
        match self {
            Self::StmtReturn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_labeled(self) -> Option<LabeledStmt> {
        match self {
            Self::StmtLabeled(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_break(self) -> Option<BreakStmt> {
        match self {
            Self::StmtBreak(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_continue(self) -> Option<ContinueStmt> {
        match self {
            Self::StmtContinue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_if(self) -> Option<IfStmt> {
        match self {
            Self::StmtIf(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_switch(self) -> Option<SwitchStmt> {
        match self {
            Self::StmtSwitch(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_throw(self) -> Option<ThrowStmt> {
        match self {
            Self::StmtThrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_try(self) -> Option<TryStmt> {
        match self {
            Self::StmtTry(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_while(self) -> Option<WhileStmt> {
        match self {
            Self::StmtWhile(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_do_while(self) -> Option<DoWhileStmt> {
        match self {
            Self::StmtDoWhile(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_for(self) -> Option<ForStmt> {
        match self {
            Self::StmtFor(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_for_in(self) -> Option<ForInStmt> {
        match self {
            Self::StmtForIn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_for_of(self) -> Option<ForOfStmt> {
        match self {
            Self::StmtForOf(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_decl_class(self) -> Option<ClassDecl> {
        match self {
            Self::StmtDeclClass(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_decl_fn(self) -> Option<FnDecl> {
        match self {
            Self::StmtDeclFn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_decl_var(self) -> Option<VarDecl> {
        match self {
            Self::StmtDeclVar(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_decl_using(self) -> Option<UsingDecl> {
        match self {
            Self::StmtDeclUsing(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt_expr(self) -> Option<ExprStmt> {
        match self {
            Self::StmtExpr(it) => Some(it),
            _ => None,
        }
    }
}
impl ModuleDecl {
    #[inline]
    pub fn is_import(self) -> bool {
        matches!(self, Self::Import(_))
    }
    #[inline]
    pub fn is_export_decl(self) -> bool {
        matches!(self, Self::ExportDecl(_))
    }
    #[inline]
    pub fn is_export_named(self) -> bool {
        matches!(self, Self::ExportNamed(_))
    }
    #[inline]
    pub fn is_export_default_decl(self) -> bool {
        matches!(self, Self::ExportDefaultDecl(_))
    }
    #[inline]
    pub fn is_export_default_expr(self) -> bool {
        matches!(self, Self::ExportDefaultExpr(_))
    }
    #[inline]
    pub fn is_export_all(self) -> bool {
        matches!(self, Self::ExportAll(_))
    }
    #[inline]
    pub fn as_import(self) -> Option<ImportDecl> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_decl(self) -> Option<ExportDecl> {
        match self {
            Self::ExportDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_named(self) -> Option<NamedExport> {
        match self {
            Self::ExportNamed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_default_decl(self) -> Option<ExportDefaultDecl> {
        match self {
            Self::ExportDefaultDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_default_expr(self) -> Option<ExportDefaultExpr> {
        match self {
            Self::ExportDefaultExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_all(self) -> Option<ExportAll> {
        match self {
            Self::ExportAll(it) => Some(it),
            _ => None,
        }
    }
}
impl ImportDecl {
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ImportSpecifier> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn src<'a>(self, ast: &crate::Ast<'a>) -> Str {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Str(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn with<'a>(self, ast: &crate::Ast<'a>) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> ImportPhase {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        specifiers: TypedSubRange<ImportSpecifier>,
    ) {
        let field_val: u64 = unsafe { specifiers.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_src<'a>(self, ast: &mut crate::Ast<'a>, src: Str) {
        let field_val: u32 = src.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_type_only<'a>(self, ast: &mut crate::Ast<'a>, type_only: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_only.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_with<'a>(self, ast: &mut crate::Ast<'a>, with: Option<ObjectLit>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = with.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: ImportPhase) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = phase.to_extra_data(ast);
        };
    }
}
impl ImportSpecifier {
    #[inline]
    pub fn is_named(self) -> bool {
        matches!(self, Self::Named(_))
    }
    #[inline]
    pub fn is_default(self) -> bool {
        matches!(self, Self::Default(_))
    }
    #[inline]
    pub fn is_namespace(self) -> bool {
        matches!(self, Self::Namespace(_))
    }
    #[inline]
    pub fn as_named(self) -> Option<ImportNamedSpecifier> {
        match self {
            Self::Named(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_default(self) -> Option<ImportDefaultSpecifier> {
        match self {
            Self::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_namespace(self) -> Option<ImportStarAsSpecifier> {
        match self {
            Self::Namespace(it) => Some(it),
            _ => None,
        }
    }
}
impl ImportNamedSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn imported<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn is_type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: Ident) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_imported<'a>(self, ast: &mut crate::Ast<'a>, imported: ModuleExportName) {
        let field_val: u64 = unsafe { imported.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_is_type_only<'a>(self, ast: &mut crate::Ast<'a>, is_type_only: bool) {
        let field_val: u32 = is_type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
}
impl ImportDefaultSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: Ident) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ImportStarAsSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: Ident) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ExportDecl {
    #[inline]
    pub fn decl<'a>(self, ast: &crate::Ast<'a>) -> Decl {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_decl<'a>(self, ast: &mut crate::Ast<'a>, decl: Decl) {
        let field_val: u64 = unsafe { decl.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl NamedExport {
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExportSpecifier> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn src<'a>(self, ast: &crate::Ast<'a>) -> Option<Str> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Str(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn with<'a>(self, ast: &crate::Ast<'a>) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        specifiers: TypedSubRange<ExportSpecifier>,
    ) {
        let field_val: u64 = unsafe { specifiers.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_src<'a>(self, ast: &mut crate::Ast<'a>, src: Option<Str>) {
        let field_val: u32 = src.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_type_only<'a>(self, ast: &mut crate::Ast<'a>, type_only: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_only.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_with<'a>(self, ast: &mut crate::Ast<'a>, with: Option<ObjectLit>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = with.to_extra_data(ast);
        };
    }
}
impl ExportSpecifier {
    #[inline]
    pub fn is_namespace(self) -> bool {
        matches!(self, Self::Namespace(_))
    }
    #[inline]
    pub fn is_default(self) -> bool {
        matches!(self, Self::Default(_))
    }
    #[inline]
    pub fn is_named(self) -> bool {
        matches!(self, Self::Named(_))
    }
    #[inline]
    pub fn as_namespace(self) -> Option<ExportNamespaceSpecifier> {
        match self {
            Self::Namespace(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_default(self) -> Option<ExportDefaultSpecifier> {
        match self {
            Self::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_named(self) -> Option<ExportNamedSpecifier> {
        match self {
            Self::Named(it) => Some(it),
            _ => None,
        }
    }
}
impl ExportNamespaceSpecifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: ModuleExportName) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl ModuleExportName {
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn as_ident(self) -> Option<Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
}
impl ExportDefaultSpecifier {
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_exported<'a>(self, ast: &mut crate::Ast<'a>, exported: Ident) {
        let field_val: u32 = exported.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ExportNamedSpecifier {
    #[inline]
    pub fn orig<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn is_type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_orig<'a>(self, ast: &mut crate::Ast<'a>, orig: ModuleExportName) {
        let field_val: u64 = unsafe { orig.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_is_type_only<'a>(self, ast: &mut crate::Ast<'a>, is_type_only: bool) {
        let field_val: u32 = is_type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_exported<'a>(self, ast: &mut crate::Ast<'a>, exported: ModuleExportName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = exported.to_extra_data(ast);
        };
    }
}
impl ExportDefaultDecl {
    #[inline]
    pub fn decl<'a>(self, ast: &crate::Ast<'a>) -> DefaultDecl {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_decl<'a>(self, ast: &mut crate::Ast<'a>, decl: DefaultDecl) {
        let field_val: u64 = unsafe { decl.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl DefaultDecl {
    #[inline]
    pub fn is_class(self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_fn(self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn as_class(self) -> Option<ClassExpr> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self) -> Option<FnExpr> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
}
impl ExportDefaultExpr {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u64 = unsafe { expr.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl ExportAll {
    #[inline]
    pub fn src<'a>(self, ast: &crate::Ast<'a>) -> Str {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Str(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn with<'a>(self, ast: &crate::Ast<'a>) -> Option<ObjectLit> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| ObjectLit(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_src<'a>(self, ast: &mut crate::Ast<'a>, src: Str) {
        let field_val: u32 = src.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_only<'a>(self, ast: &mut crate::Ast<'a>, type_only: bool) {
        let field_val: u32 = type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_with<'a>(self, ast: &mut crate::Ast<'a>, with: Option<ObjectLit>) {
        let field_val: u32 = with.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl BlockStmt {
    #[inline]
    pub fn stmts<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn scope_id<'a>(self, ast: &crate::Ast<'a>) -> Option<ScopeId> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != 0).then(|| unsafe {
            <ScopeId as oxc_index::Idx>::from_usize_unchecked(raw.wrapping_sub(1) as usize)
        })
    }
    #[inline]
    pub fn set_stmts<'a>(self, ast: &mut crate::Ast<'a>, stmts: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { stmts.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_scope_id<'a>(self, ast: &mut crate::Ast<'a>, scope_id: Option<ScopeId>) {
        let field_val: u32 = scope_id.map_or(0, |id| id.index() as u32 + 1);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl Stmt {
    #[inline]
    pub fn is_class(self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_fn(self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn is_var(self) -> bool {
        matches!(self, Self::Var(_))
    }
    #[inline]
    pub fn is_using(self) -> bool {
        matches!(self, Self::Using(_))
    }
    #[inline]
    pub fn is_block(self) -> bool {
        matches!(self, Self::Block(_))
    }
    #[inline]
    pub fn is_empty(self) -> bool {
        matches!(self, Self::Empty(_))
    }
    #[inline]
    pub fn is_debugger(self) -> bool {
        matches!(self, Self::Debugger(_))
    }
    #[inline]
    pub fn is_with(self) -> bool {
        matches!(self, Self::With(_))
    }
    #[inline]
    pub fn is_return(self) -> bool {
        matches!(self, Self::Return(_))
    }
    #[inline]
    pub fn is_labeled(self) -> bool {
        matches!(self, Self::Labeled(_))
    }
    #[inline]
    pub fn is_break(self) -> bool {
        matches!(self, Self::Break(_))
    }
    #[inline]
    pub fn is_continue(self) -> bool {
        matches!(self, Self::Continue(_))
    }
    #[inline]
    pub fn is_if(self) -> bool {
        matches!(self, Self::If(_))
    }
    #[inline]
    pub fn is_switch(self) -> bool {
        matches!(self, Self::Switch(_))
    }
    #[inline]
    pub fn is_throw(self) -> bool {
        matches!(self, Self::Throw(_))
    }
    #[inline]
    pub fn is_try(self) -> bool {
        matches!(self, Self::Try(_))
    }
    #[inline]
    pub fn is_while(self) -> bool {
        matches!(self, Self::While(_))
    }
    #[inline]
    pub fn is_do_while(self) -> bool {
        matches!(self, Self::DoWhile(_))
    }
    #[inline]
    pub fn is_for(self) -> bool {
        matches!(self, Self::For(_))
    }
    #[inline]
    pub fn is_for_in(self) -> bool {
        matches!(self, Self::ForIn(_))
    }
    #[inline]
    pub fn is_for_of(self) -> bool {
        matches!(self, Self::ForOf(_))
    }
    #[inline]
    pub fn is_expr(self) -> bool {
        matches!(self, Self::Expr(_))
    }
    #[inline]
    pub fn as_class(self) -> Option<ClassDecl> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self) -> Option<FnDecl> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_var(self) -> Option<VarDecl> {
        match self {
            Self::Var(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_using(self) -> Option<UsingDecl> {
        match self {
            Self::Using(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_block(self) -> Option<BlockStmt> {
        match self {
            Self::Block(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_empty(self) -> Option<EmptyStmt> {
        match self {
            Self::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_debugger(self) -> Option<DebuggerStmt> {
        match self {
            Self::Debugger(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_with(self) -> Option<WithStmt> {
        match self {
            Self::With(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_return(self) -> Option<ReturnStmt> {
        match self {
            Self::Return(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_labeled(self) -> Option<LabeledStmt> {
        match self {
            Self::Labeled(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_break(self) -> Option<BreakStmt> {
        match self {
            Self::Break(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_continue(self) -> Option<ContinueStmt> {
        match self {
            Self::Continue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_if(self) -> Option<IfStmt> {
        match self {
            Self::If(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_switch(self) -> Option<SwitchStmt> {
        match self {
            Self::Switch(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_throw(self) -> Option<ThrowStmt> {
        match self {
            Self::Throw(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_try(self) -> Option<TryStmt> {
        match self {
            Self::Try(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_while(self) -> Option<WhileStmt> {
        match self {
            Self::While(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_do_while(self) -> Option<DoWhileStmt> {
        match self {
            Self::DoWhile(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for(self) -> Option<ForStmt> {
        match self {
            Self::For(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_in(self) -> Option<ForInStmt> {
        match self {
            Self::ForIn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_of(self) -> Option<ForOfStmt> {
        match self {
            Self::ForOf(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self) -> Option<ExprStmt> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl ExprStmt {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u64 = unsafe { expr.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl EmptyStmt {}
impl DebuggerStmt {}
impl WithStmt {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: Expr) {
        let field_val: u64 = unsafe { obj.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl ReturnStmt {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Option<Expr>) {
        let field_val: u64 = unsafe { arg.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl LabeledStmt {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Ident) {
        let field_val: u32 = label.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl BreakStmt {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Option<Ident> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Ident(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Option<Ident>) {
        let field_val: u32 = label.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ContinueStmt {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Option<Ident> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Ident(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Option<Ident>) {
        let field_val: u32 = label.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl IfStmt {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn cons<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn alt<'a>(self, ast: &crate::Ast<'a>) -> Option<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u64 = unsafe { test.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_cons<'a>(self, ast: &mut crate::Ast<'a>, cons: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = cons.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_alt<'a>(self, ast: &mut crate::Ast<'a>, alt: Option<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = alt.to_extra_data(ast);
        };
    }
}
impl SwitchStmt {
    #[inline]
    pub fn discriminant<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn cases<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<SwitchCase> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_discriminant<'a>(self, ast: &mut crate::Ast<'a>, discriminant: Expr) {
        let field_val: u64 = unsafe { discriminant.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_cases<'a>(self, ast: &mut crate::Ast<'a>, cases: TypedSubRange<SwitchCase>) {
        let field_val: u64 = unsafe { cases.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl ThrowStmt {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Expr) {
        let field_val: u64 = unsafe { arg.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TryStmt {
    #[inline]
    pub fn block<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BlockStmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn handler<'a>(self, ast: &crate::Ast<'a>) -> Option<CatchClause> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| CatchClause(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn finalizer<'a>(self, ast: &crate::Ast<'a>) -> Option<BlockStmt> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BlockStmt(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_block<'a>(self, ast: &mut crate::Ast<'a>, block: BlockStmt) {
        let field_val: u32 = block.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_handler<'a>(self, ast: &mut crate::Ast<'a>, handler: Option<CatchClause>) {
        let field_val: u32 = handler.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_finalizer<'a>(self, ast: &mut crate::Ast<'a>, finalizer: Option<BlockStmt>) {
        let field_val: u32 = finalizer.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl WhileStmt {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u64 = unsafe { test.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl DoWhileStmt {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u64 = unsafe { test.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl ForStmt {
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<VarDeclOrExpr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn update<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<VarDeclOrExpr>) {
        let field_val: u64 = unsafe { init.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = test.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_update<'a>(self, ast: &mut crate::Ast<'a>, update: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = update.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.to_extra_data(ast);
        };
    }
}
impl ForInStmt {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForHead {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: ForHead) {
        let field_val: u64 = unsafe { left.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = right.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.to_extra_data(ast);
        };
    }
}
impl ForOfStmt {
    #[inline]
    pub fn is_await<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForHead {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_is_await<'a>(self, ast: &mut crate::Ast<'a>, is_await: bool) {
        let field_val: u32 = is_await as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: ForHead) {
        let field_val: u64 = unsafe { left.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = right.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.to_extra_data(ast);
        };
    }
}
impl SwitchCase {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn cons<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u64 = unsafe { test.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_cons<'a>(self, ast: &mut crate::Ast<'a>, cons: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { cons.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl CatchClause {
    #[inline]
    pub fn param<'a>(self, ast: &crate::Ast<'a>) -> Option<Pat> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        BlockStmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_param<'a>(self, ast: &mut crate::Ast<'a>, param: Option<Pat>) {
        let field_val: u64 = unsafe { param.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl ForHead {
    #[inline]
    pub fn is_expr_this(self) -> bool {
        matches!(self, Self::ExprThis(_))
    }
    #[inline]
    pub fn is_expr_array(self) -> bool {
        matches!(self, Self::ExprArray(_))
    }
    #[inline]
    pub fn is_expr_object(self) -> bool {
        matches!(self, Self::ExprObject(_))
    }
    #[inline]
    pub fn is_expr_fn(self) -> bool {
        matches!(self, Self::ExprFn(_))
    }
    #[inline]
    pub fn is_expr_unary(self) -> bool {
        matches!(self, Self::ExprUnary(_))
    }
    #[inline]
    pub fn is_expr_update(self) -> bool {
        matches!(self, Self::ExprUpdate(_))
    }
    #[inline]
    pub fn is_expr_bin(self) -> bool {
        matches!(self, Self::ExprBin(_))
    }
    #[inline]
    pub fn is_expr_assign(self) -> bool {
        matches!(self, Self::ExprAssign(_))
    }
    #[inline]
    pub fn is_expr_member(self) -> bool {
        matches!(self, Self::ExprMember(_))
    }
    #[inline]
    pub fn is_expr_super_prop(self) -> bool {
        matches!(self, Self::ExprSuperProp(_))
    }
    #[inline]
    pub fn is_expr_cond(self) -> bool {
        matches!(self, Self::ExprCond(_))
    }
    #[inline]
    pub fn is_expr_import(self) -> bool {
        matches!(self, Self::ExprImport(_))
    }
    #[inline]
    pub fn is_expr_call(self) -> bool {
        matches!(self, Self::ExprCall(_))
    }
    #[inline]
    pub fn is_expr_new(self) -> bool {
        matches!(self, Self::ExprNew(_))
    }
    #[inline]
    pub fn is_expr_seq(self) -> bool {
        matches!(self, Self::ExprSeq(_))
    }
    #[inline]
    pub fn is_expr_ident(self) -> bool {
        matches!(self, Self::ExprIdent(_))
    }
    #[inline]
    pub fn is_expr_str(self) -> bool {
        matches!(self, Self::ExprStr(_))
    }
    #[inline]
    pub fn is_expr_bool(self) -> bool {
        matches!(self, Self::ExprBool(_))
    }
    #[inline]
    pub fn is_expr_null(self) -> bool {
        matches!(self, Self::ExprNull(_))
    }
    #[inline]
    pub fn is_expr_num(self) -> bool {
        matches!(self, Self::ExprNum(_))
    }
    #[inline]
    pub fn is_expr_big_int(self) -> bool {
        matches!(self, Self::ExprBigInt(_))
    }
    #[inline]
    pub fn is_expr_regex(self) -> bool {
        matches!(self, Self::ExprRegex(_))
    }
    #[inline]
    pub fn is_expr_tpl(self) -> bool {
        matches!(self, Self::ExprTpl(_))
    }
    #[inline]
    pub fn is_expr_tagged_tpl(self) -> bool {
        matches!(self, Self::ExprTaggedTpl(_))
    }
    #[inline]
    pub fn is_expr_arrow(self) -> bool {
        matches!(self, Self::ExprArrow(_))
    }
    #[inline]
    pub fn is_expr_class(self) -> bool {
        matches!(self, Self::ExprClass(_))
    }
    #[inline]
    pub fn is_expr_yield(self) -> bool {
        matches!(self, Self::ExprYield(_))
    }
    #[inline]
    pub fn is_expr_meta_prop(self) -> bool {
        matches!(self, Self::ExprMetaProp(_))
    }
    #[inline]
    pub fn is_expr_await(self) -> bool {
        matches!(self, Self::ExprAwait(_))
    }
    #[inline]
    pub fn is_expr_paren(self) -> bool {
        matches!(self, Self::ExprParen(_))
    }
    #[inline]
    pub fn is_expr_jsx_member(self) -> bool {
        matches!(self, Self::ExprJSXMember(_))
    }
    #[inline]
    pub fn is_expr_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::ExprJSXNamespacedName(_))
    }
    #[inline]
    pub fn is_expr_jsx_empty(self) -> bool {
        matches!(self, Self::ExprJSXEmpty(_))
    }
    #[inline]
    pub fn is_expr_jsx_element(self) -> bool {
        matches!(self, Self::ExprJSXElement(_))
    }
    #[inline]
    pub fn is_expr_jsx_fragment(self) -> bool {
        matches!(self, Self::ExprJSXFragment(_))
    }
    #[inline]
    pub fn is_expr_private_name(self) -> bool {
        matches!(self, Self::ExprPrivateName(_))
    }
    #[inline]
    pub fn is_expr_opt_chain(self) -> bool {
        matches!(self, Self::ExprOptChain(_))
    }
    #[inline]
    pub fn is_expr_invalid(self) -> bool {
        matches!(self, Self::ExprInvalid(_))
    }
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_rest(self) -> bool {
        matches!(self, Self::Rest(_))
    }
    #[inline]
    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_invalid(self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn is_var_decl(self) -> bool {
        matches!(self, Self::VarDecl(_))
    }
    #[inline]
    pub fn is_using_decl(self) -> bool {
        matches!(self, Self::UsingDecl(_))
    }
    #[inline]
    pub fn as_expr_this(self) -> Option<ThisExpr> {
        match self {
            Self::ExprThis(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_array(self) -> Option<ArrayLit> {
        match self {
            Self::ExprArray(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_object(self) -> Option<ObjectLit> {
        match self {
            Self::ExprObject(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_fn(self) -> Option<FnExpr> {
        match self {
            Self::ExprFn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_unary(self) -> Option<UnaryExpr> {
        match self {
            Self::ExprUnary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_update(self) -> Option<UpdateExpr> {
        match self {
            Self::ExprUpdate(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_bin(self) -> Option<BinExpr> {
        match self {
            Self::ExprBin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_assign(self) -> Option<AssignExpr> {
        match self {
            Self::ExprAssign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_member(self) -> Option<MemberExpr> {
        match self {
            Self::ExprMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_super_prop(self) -> Option<SuperPropExpr> {
        match self {
            Self::ExprSuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_cond(self) -> Option<CondExpr> {
        match self {
            Self::ExprCond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_import(self) -> Option<ImportExpr> {
        match self {
            Self::ExprImport(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_call(self) -> Option<CallExpr> {
        match self {
            Self::ExprCall(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_new(self) -> Option<NewExpr> {
        match self {
            Self::ExprNew(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_seq(self) -> Option<SeqExpr> {
        match self {
            Self::ExprSeq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_ident(self) -> Option<Ident> {
        match self {
            Self::ExprIdent(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_str(self) -> Option<Str> {
        match self {
            Self::ExprStr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_bool(self) -> Option<Bool> {
        match self {
            Self::ExprBool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_null(self) -> Option<Null> {
        match self {
            Self::ExprNull(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_num(self) -> Option<Number> {
        match self {
            Self::ExprNum(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_big_int(self) -> Option<BigInt> {
        match self {
            Self::ExprBigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_regex(self) -> Option<Regex> {
        match self {
            Self::ExprRegex(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_tpl(self) -> Option<Tpl> {
        match self {
            Self::ExprTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_tagged_tpl(self) -> Option<TaggedTpl> {
        match self {
            Self::ExprTaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_arrow(self) -> Option<ArrowExpr> {
        match self {
            Self::ExprArrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_class(self) -> Option<ClassExpr> {
        match self {
            Self::ExprClass(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_yield(self) -> Option<YieldExpr> {
        match self {
            Self::ExprYield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_meta_prop(self) -> Option<MetaPropExpr> {
        match self {
            Self::ExprMetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_await(self) -> Option<AwaitExpr> {
        match self {
            Self::ExprAwait(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_paren(self) -> Option<ParenExpr> {
        match self {
            Self::ExprParen(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_member(self) -> Option<JSXMemberExpr> {
        match self {
            Self::ExprJSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_namespaced_name(self) -> Option<JSXNamespacedName> {
        match self {
            Self::ExprJSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_empty(self) -> Option<JSXEmptyExpr> {
        match self {
            Self::ExprJSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_element(self) -> Option<JSXElement> {
        match self {
            Self::ExprJSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_fragment(self) -> Option<JSXFragment> {
        match self {
            Self::ExprJSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_private_name(self) -> Option<PrivateName> {
        match self {
            Self::ExprPrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_opt_chain(self) -> Option<OptChainExpr> {
        match self {
            Self::ExprOptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_invalid(self) -> Option<Invalid> {
        match self {
            Self::ExprInvalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self) -> Option<BindingIdent> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(self) -> Option<ArrayPat> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_rest(self) -> Option<RestPat> {
        match self {
            Self::Rest(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self) -> Option<ObjectPat> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignPat> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self) -> Option<Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_var_decl(self) -> Option<VarDecl> {
        match self {
            Self::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_using_decl(self) -> Option<UsingDecl> {
        match self {
            Self::UsingDecl(it) => Some(it),
            _ => None,
        }
    }
}
impl VarDeclOrExpr {
    #[inline]
    pub fn is_this(self) -> bool {
        matches!(self, Self::This(_))
    }
    #[inline]
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_fn(self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn is_unary(self) -> bool {
        matches!(self, Self::Unary(_))
    }
    #[inline]
    pub fn is_update(self) -> bool {
        matches!(self, Self::Update(_))
    }
    #[inline]
    pub fn is_bin(self) -> bool {
        matches!(self, Self::Bin(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_member(self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_super_prop(self) -> bool {
        matches!(self, Self::SuperProp(_))
    }
    #[inline]
    pub fn is_cond(self) -> bool {
        matches!(self, Self::Cond(_))
    }
    #[inline]
    pub fn is_import(self) -> bool {
        matches!(self, Self::Import(_))
    }
    #[inline]
    pub fn is_call(self) -> bool {
        matches!(self, Self::Call(_))
    }
    #[inline]
    pub fn is_new(self) -> bool {
        matches!(self, Self::New(_))
    }
    #[inline]
    pub fn is_seq(self) -> bool {
        matches!(self, Self::Seq(_))
    }
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_bool(self) -> bool {
        matches!(self, Self::Bool(_))
    }
    #[inline]
    pub fn is_null(self) -> bool {
        matches!(self, Self::Null(_))
    }
    #[inline]
    pub fn is_num(self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_big_int(self) -> bool {
        matches!(self, Self::BigInt(_))
    }
    #[inline]
    pub fn is_regex(self) -> bool {
        matches!(self, Self::Regex(_))
    }
    #[inline]
    pub fn is_tpl(self) -> bool {
        matches!(self, Self::Tpl(_))
    }
    #[inline]
    pub fn is_tagged_tpl(self) -> bool {
        matches!(self, Self::TaggedTpl(_))
    }
    #[inline]
    pub fn is_arrow(self) -> bool {
        matches!(self, Self::Arrow(_))
    }
    #[inline]
    pub fn is_class(self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_yield(self) -> bool {
        matches!(self, Self::Yield(_))
    }
    #[inline]
    pub fn is_meta_prop(self) -> bool {
        matches!(self, Self::MetaProp(_))
    }
    #[inline]
    pub fn is_await(self) -> bool {
        matches!(self, Self::Await(_))
    }
    #[inline]
    pub fn is_paren(self) -> bool {
        matches!(self, Self::Paren(_))
    }
    #[inline]
    pub fn is_jsx_member(self) -> bool {
        matches!(self, Self::JSXMember(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::JSXNamespacedName(_))
    }
    #[inline]
    pub fn is_jsx_empty(self) -> bool {
        matches!(self, Self::JSXEmpty(_))
    }
    #[inline]
    pub fn is_jsx_element(self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(self) -> bool {
        matches!(self, Self::JSXFragment(_))
    }
    #[inline]
    pub fn is_private_name(self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
    #[inline]
    pub fn is_opt_chain(self) -> bool {
        matches!(self, Self::OptChain(_))
    }
    #[inline]
    pub fn is_invalid(self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn is_var_decl(self) -> bool {
        matches!(self, Self::VarDecl(_))
    }
    #[inline]
    pub fn as_this(self) -> Option<ThisExpr> {
        match self {
            Self::This(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(self) -> Option<ArrayLit> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self) -> Option<ObjectLit> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self) -> Option<FnExpr> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary(self) -> Option<UnaryExpr> {
        match self {
            Self::Unary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_update(self) -> Option<UpdateExpr> {
        match self {
            Self::Update(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bin(self) -> Option<BinExpr> {
        match self {
            Self::Bin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignExpr> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(self) -> Option<MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(self) -> Option<SuperPropExpr> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_cond(self) -> Option<CondExpr> {
        match self {
            Self::Cond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import(self) -> Option<ImportExpr> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(self) -> Option<CallExpr> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_new(self) -> Option<NewExpr> {
        match self {
            Self::New(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_seq(self) -> Option<SeqExpr> {
        match self {
            Self::Seq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self) -> Option<Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(self) -> Option<Bool> {
        match self {
            Self::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null(self) -> Option<Null> {
        match self {
            Self::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self) -> Option<Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self) -> Option<BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_regex(self) -> Option<Regex> {
        match self {
            Self::Regex(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tpl(self) -> Option<Tpl> {
        match self {
            Self::Tpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tagged_tpl(self) -> Option<TaggedTpl> {
        match self {
            Self::TaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_arrow(self) -> Option<ArrowExpr> {
        match self {
            Self::Arrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(self) -> Option<ClassExpr> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_yield(self) -> Option<YieldExpr> {
        match self {
            Self::Yield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_meta_prop(self) -> Option<MetaPropExpr> {
        match self {
            Self::MetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_await(self) -> Option<AwaitExpr> {
        match self {
            Self::Await(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(self) -> Option<ParenExpr> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member(self) -> Option<JSXMemberExpr> {
        match self {
            Self::JSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self) -> Option<JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty(self) -> Option<JSXEmptyExpr> {
        match self {
            Self::JSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self) -> Option<JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self) -> Option<JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(self) -> Option<PrivateName> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(self) -> Option<OptChainExpr> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self) -> Option<Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_var_decl(self) -> Option<VarDecl> {
        match self {
            Self::VarDecl(it) => Some(it),
            _ => None,
        }
    }
}
impl Decl {
    #[inline]
    pub fn is_class(self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_fn(self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn is_var(self) -> bool {
        matches!(self, Self::Var(_))
    }
    #[inline]
    pub fn is_using(self) -> bool {
        matches!(self, Self::Using(_))
    }
    #[inline]
    pub fn as_class(self) -> Option<ClassDecl> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self) -> Option<FnDecl> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_var(self) -> Option<VarDecl> {
        match self {
            Self::Var(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_using(self) -> Option<UsingDecl> {
        match self {
            Self::Using(it) => Some(it),
            _ => None,
        }
    }
}
impl FnDecl {
    #[inline]
    pub fn ident<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_ident<'a>(self, ast: &mut crate::Ast<'a>, ident: Ident) {
        let field_val: u32 = ident.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl ClassDecl {
    #[inline]
    pub fn ident<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn class<'a>(self, ast: &crate::Ast<'a>) -> Class {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Class(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_ident<'a>(self, ast: &mut crate::Ast<'a>, ident: Ident) {
        let field_val: u32 = ident.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_class<'a>(self, ast: &mut crate::Ast<'a>, class: Class) {
        let field_val: u32 = class.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl VarDecl {
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> VarDeclKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, VarDeclKind>(raw as u8) }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn decls<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<VarDeclarator> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: VarDeclKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
    #[inline]
    pub fn set_decls<'a>(self, ast: &mut crate::Ast<'a>, decls: TypedSubRange<VarDeclarator>) {
        let field_val: u64 = unsafe { decls.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl VarDeclarator {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: Pat) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<Expr>) {
        let field_val: u64 = unsafe { init.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl UsingDecl {
    #[inline]
    pub fn is_await<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn decls<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<VarDeclarator> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_is_await<'a>(self, ast: &mut crate::Ast<'a>, is_await: bool) {
        let field_val: u32 = is_await as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_decls<'a>(self, ast: &mut crate::Ast<'a>, decls: TypedSubRange<VarDeclarator>) {
        let field_val: u64 = unsafe { decls.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl Expr {
    #[inline]
    pub fn is_this(self) -> bool {
        matches!(self, Self::This(_))
    }
    #[inline]
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_fn(self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn is_unary(self) -> bool {
        matches!(self, Self::Unary(_))
    }
    #[inline]
    pub fn is_update(self) -> bool {
        matches!(self, Self::Update(_))
    }
    #[inline]
    pub fn is_bin(self) -> bool {
        matches!(self, Self::Bin(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_member(self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_super_prop(self) -> bool {
        matches!(self, Self::SuperProp(_))
    }
    #[inline]
    pub fn is_cond(self) -> bool {
        matches!(self, Self::Cond(_))
    }
    #[inline]
    pub fn is_import(self) -> bool {
        matches!(self, Self::Import(_))
    }
    #[inline]
    pub fn is_call(self) -> bool {
        matches!(self, Self::Call(_))
    }
    #[inline]
    pub fn is_new(self) -> bool {
        matches!(self, Self::New(_))
    }
    #[inline]
    pub fn is_seq(self) -> bool {
        matches!(self, Self::Seq(_))
    }
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_bool(self) -> bool {
        matches!(self, Self::Bool(_))
    }
    #[inline]
    pub fn is_null(self) -> bool {
        matches!(self, Self::Null(_))
    }
    #[inline]
    pub fn is_num(self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_big_int(self) -> bool {
        matches!(self, Self::BigInt(_))
    }
    #[inline]
    pub fn is_regex(self) -> bool {
        matches!(self, Self::Regex(_))
    }
    #[inline]
    pub fn is_tpl(self) -> bool {
        matches!(self, Self::Tpl(_))
    }
    #[inline]
    pub fn is_tagged_tpl(self) -> bool {
        matches!(self, Self::TaggedTpl(_))
    }
    #[inline]
    pub fn is_arrow(self) -> bool {
        matches!(self, Self::Arrow(_))
    }
    #[inline]
    pub fn is_class(self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_yield(self) -> bool {
        matches!(self, Self::Yield(_))
    }
    #[inline]
    pub fn is_meta_prop(self) -> bool {
        matches!(self, Self::MetaProp(_))
    }
    #[inline]
    pub fn is_await(self) -> bool {
        matches!(self, Self::Await(_))
    }
    #[inline]
    pub fn is_paren(self) -> bool {
        matches!(self, Self::Paren(_))
    }
    #[inline]
    pub fn is_jsx_member(self) -> bool {
        matches!(self, Self::JSXMember(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::JSXNamespacedName(_))
    }
    #[inline]
    pub fn is_jsx_empty(self) -> bool {
        matches!(self, Self::JSXEmpty(_))
    }
    #[inline]
    pub fn is_jsx_element(self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(self) -> bool {
        matches!(self, Self::JSXFragment(_))
    }
    #[inline]
    pub fn is_private_name(self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
    #[inline]
    pub fn is_opt_chain(self) -> bool {
        matches!(self, Self::OptChain(_))
    }
    #[inline]
    pub fn is_invalid(self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn as_this(self) -> Option<ThisExpr> {
        match self {
            Self::This(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(self) -> Option<ArrayLit> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self) -> Option<ObjectLit> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self) -> Option<FnExpr> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary(self) -> Option<UnaryExpr> {
        match self {
            Self::Unary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_update(self) -> Option<UpdateExpr> {
        match self {
            Self::Update(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bin(self) -> Option<BinExpr> {
        match self {
            Self::Bin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignExpr> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(self) -> Option<MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(self) -> Option<SuperPropExpr> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_cond(self) -> Option<CondExpr> {
        match self {
            Self::Cond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import(self) -> Option<ImportExpr> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(self) -> Option<CallExpr> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_new(self) -> Option<NewExpr> {
        match self {
            Self::New(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_seq(self) -> Option<SeqExpr> {
        match self {
            Self::Seq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self) -> Option<Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(self) -> Option<Bool> {
        match self {
            Self::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null(self) -> Option<Null> {
        match self {
            Self::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self) -> Option<Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self) -> Option<BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_regex(self) -> Option<Regex> {
        match self {
            Self::Regex(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tpl(self) -> Option<Tpl> {
        match self {
            Self::Tpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tagged_tpl(self) -> Option<TaggedTpl> {
        match self {
            Self::TaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_arrow(self) -> Option<ArrowExpr> {
        match self {
            Self::Arrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(self) -> Option<ClassExpr> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_yield(self) -> Option<YieldExpr> {
        match self {
            Self::Yield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_meta_prop(self) -> Option<MetaPropExpr> {
        match self {
            Self::MetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_await(self) -> Option<AwaitExpr> {
        match self {
            Self::Await(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(self) -> Option<ParenExpr> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member(self) -> Option<JSXMemberExpr> {
        match self {
            Self::JSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self) -> Option<JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty(self) -> Option<JSXEmptyExpr> {
        match self {
            Self::JSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self) -> Option<JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self) -> Option<JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(self) -> Option<PrivateName> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(self) -> Option<OptChainExpr> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self) -> Option<Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl ThisExpr {}
impl ArrayLit {
    #[inline]
    pub fn elems<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Option<ExprOrSpread>> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_elems<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) {
        let field_val: u64 = unsafe { elems.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl ObjectLit {
    #[inline]
    pub fn props<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<PropOrSpread> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_props<'a>(self, ast: &mut crate::Ast<'a>, props: TypedSubRange<PropOrSpread>) {
        let field_val: u64 = unsafe { props.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl PropOrSpread {
    #[inline]
    pub fn is_shorthand(self) -> bool {
        matches!(self, Self::Shorthand(_))
    }
    #[inline]
    pub fn is_key_value(self) -> bool {
        matches!(self, Self::KeyValue(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_getter(self) -> bool {
        matches!(self, Self::Getter(_))
    }
    #[inline]
    pub fn is_setter(self) -> bool {
        matches!(self, Self::Setter(_))
    }
    #[inline]
    pub fn is_method(self) -> bool {
        matches!(self, Self::Method(_))
    }
    #[inline]
    pub fn is_spread(self) -> bool {
        matches!(self, Self::Spread(_))
    }
    #[inline]
    pub fn as_shorthand(self) -> Option<Ident> {
        match self {
            Self::Shorthand(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_key_value(self) -> Option<KeyValueProp> {
        match self {
            Self::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignProp> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_getter(self) -> Option<GetterProp> {
        match self {
            Self::Getter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_setter(self) -> Option<SetterProp> {
        match self {
            Self::Setter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_method(self) -> Option<MethodProp> {
        match self {
            Self::Method(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_spread(self) -> Option<SpreadElement> {
        match self {
            Self::Spread(it) => Some(it),
            _ => None,
        }
    }
}
impl SpreadElement {
    #[inline]
    pub fn dot_3_token<'a>(self, ast: &crate::Ast<'a>) -> Span {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_dot3_token<'a>(self, ast: &mut crate::Ast<'a>, dot3_token: Span) {
        let field_val: u64 = unsafe { dot3_token.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u64 = unsafe { expr.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl UnaryExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> UnaryOp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, UnaryOp>(raw as u8) }
    }
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: UnaryOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Expr) {
        let field_val: u64 = unsafe { arg.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl UpdateExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> UpdateOp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, UpdateOp>(raw as u8) }
    }
    #[inline]
    pub fn prefix<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> SimpleAssignTarget {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: UpdateOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_prefix<'a>(self, ast: &mut crate::Ast<'a>, prefix: bool) {
        let field_val: u32 = prefix as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: SimpleAssignTarget) {
        let field_val: u64 = unsafe { arg.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl BinExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> BinaryOp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, BinaryOp>(raw as u8) }
    }
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: BinaryOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: Expr) {
        let field_val: u64 = unsafe { left.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = right.to_extra_data(ast);
        };
    }
}
impl FnExpr {
    #[inline]
    pub fn ident<'a>(self, ast: &crate::Ast<'a>) -> Option<Ident> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Ident(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_ident<'a>(self, ast: &mut crate::Ast<'a>, ident: Option<Ident>) {
        let field_val: u32 = ident.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl ClassExpr {
    #[inline]
    pub fn ident<'a>(self, ast: &crate::Ast<'a>) -> Option<Ident> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Ident(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn class<'a>(self, ast: &crate::Ast<'a>) -> Class {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Class(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_ident<'a>(self, ast: &mut crate::Ast<'a>, ident: Option<Ident>) {
        let field_val: u32 = ident.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_class<'a>(self, ast: &mut crate::Ast<'a>, class: Class) {
        let field_val: u32 = class.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl AssignExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> AssignOp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, AssignOp>(raw as u8) }
    }
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> AssignTarget {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: AssignOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: AssignTarget) {
        let field_val: u64 = unsafe { left.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = right.to_extra_data(ast);
        };
    }
}
impl MemberExpr {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn prop<'a>(self, ast: &crate::Ast<'a>) -> MemberProp {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: Expr) {
        let field_val: u64 = unsafe { obj.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_prop<'a>(self, ast: &mut crate::Ast<'a>, prop: MemberProp) {
        let field_val: u64 = unsafe { prop.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl MemberProp {
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_private_name(self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
    #[inline]
    pub fn is_computed(self) -> bool {
        matches!(self, Self::Computed(_))
    }
    #[inline]
    pub fn as_ident(self) -> Option<IdentName> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(self) -> Option<PrivateName> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(self) -> Option<ComputedPropName> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
}
impl SuperPropExpr {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> Super {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Super(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn prop<'a>(self, ast: &crate::Ast<'a>) -> SuperProp {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: Super) {
        let field_val: u32 = obj.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_prop<'a>(self, ast: &mut crate::Ast<'a>, prop: SuperProp) {
        let field_val: u64 = unsafe { prop.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl SuperProp {
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_computed(self) -> bool {
        matches!(self, Self::Computed(_))
    }
    #[inline]
    pub fn as_ident(self) -> Option<IdentName> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(self) -> Option<ComputedPropName> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
}
impl CondExpr {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn cons<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn alt<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u64 = unsafe { test.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_cons<'a>(self, ast: &mut crate::Ast<'a>, cons: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = cons.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_alt<'a>(self, ast: &mut crate::Ast<'a>, alt: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = alt.to_extra_data(ast);
        };
    }
}
impl ImportExpr {
    #[inline]
    pub fn import_span<'a>(self, ast: &crate::Ast<'a>) -> Span {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> ImportPhase {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ImportPhase>(raw as u8) }
    }
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn options<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_import_span<'a>(self, ast: &mut crate::Ast<'a>, import_span: Span) {
        let field_val: u64 = unsafe { import_span.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: ImportPhase) {
        let field_val: u32 = phase as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = source.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_options<'a>(self, ast: &mut crate::Ast<'a>, options: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = options.to_extra_data(ast);
        };
    }
}
impl CallExpr {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Callee {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn args<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExprOrSpread> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Callee) {
        let field_val: u64 = unsafe { callee.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_args<'a>(self, ast: &mut crate::Ast<'a>, args: TypedSubRange<ExprOrSpread>) {
        let field_val: u64 = unsafe { args.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl NewExpr {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn args<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExprOrSpread> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        let field_val: u64 = unsafe { callee.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_args<'a>(self, ast: &mut crate::Ast<'a>, args: TypedSubRange<ExprOrSpread>) {
        let field_val: u64 = unsafe { args.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl SeqExpr {
    #[inline]
    pub fn exprs<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_exprs<'a>(self, ast: &mut crate::Ast<'a>, exprs: TypedSubRange<Expr>) {
        let field_val: u64 = unsafe { exprs.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl ArrowExpr {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Pat> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn is_async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmtOrExpr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: TypedSubRange<Pat>) {
        let field_val: u64 = unsafe { params.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_is_async<'a>(self, ast: &mut crate::Ast<'a>, is_async: bool) {
        let field_val: u32 = is_async as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmtOrExpr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.to_extra_data(ast);
        };
    }
}
impl YieldExpr {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn delegate<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Option<Expr>) {
        let field_val: u64 = unsafe { arg.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_delegate<'a>(self, ast: &mut crate::Ast<'a>, delegate: bool) {
        let field_val: u32 = delegate as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl MetaPropExpr {
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MetaPropKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, MetaPropKind>(raw as u8) }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MetaPropKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
}
impl AwaitExpr {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Expr) {
        let field_val: u64 = unsafe { arg.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl Tpl {
    #[inline]
    pub fn exprs<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn quasis<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TplElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_exprs<'a>(self, ast: &mut crate::Ast<'a>, exprs: TypedSubRange<Expr>) {
        let field_val: u64 = unsafe { exprs.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_quasis<'a>(self, ast: &mut crate::Ast<'a>, quasis: TypedSubRange<TplElement>) {
        let field_val: u64 = unsafe { quasis.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl TaggedTpl {
    #[inline]
    pub fn tag<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn tpl<'a>(self, ast: &crate::Ast<'a>) -> Tpl {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Tpl(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_tag<'a>(self, ast: &mut crate::Ast<'a>, tag: Expr) {
        let field_val: u64 = unsafe { tag.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_tpl<'a>(self, ast: &mut crate::Ast<'a>, tpl: Tpl) {
        let field_val: u32 = tpl.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl TplElement {
    #[inline]
    pub fn tail<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn cooked<'a>(self, ast: &crate::Ast<'a>) -> Option<Wtf8Atom<'a>> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    wtf8_atom: Wtf8AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_tail<'a>(self, ast: &mut crate::Ast<'a>, tail: bool) {
        let field_val: u32 = tail as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_cooked<'a>(self, ast: &mut crate::Ast<'a>, cooked: Option<Wtf8Atom<'a>>) {
        let field_val: u32 = unsafe { cooked.to_extra_data(ast).wtf8_atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Atom<'a>) {
        let field_val: u32 = unsafe { raw.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl ParenExpr {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u64 = unsafe { expr.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl Callee {
    #[inline]
    pub fn is_this(self) -> bool {
        matches!(self, Self::This(_))
    }
    #[inline]
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_fn(self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn is_unary(self) -> bool {
        matches!(self, Self::Unary(_))
    }
    #[inline]
    pub fn is_update(self) -> bool {
        matches!(self, Self::Update(_))
    }
    #[inline]
    pub fn is_bin(self) -> bool {
        matches!(self, Self::Bin(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_member(self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_super_prop(self) -> bool {
        matches!(self, Self::SuperProp(_))
    }
    #[inline]
    pub fn is_cond(self) -> bool {
        matches!(self, Self::Cond(_))
    }
    #[inline]
    pub fn is_import(self) -> bool {
        matches!(self, Self::Import(_))
    }
    #[inline]
    pub fn is_call(self) -> bool {
        matches!(self, Self::Call(_))
    }
    #[inline]
    pub fn is_new(self) -> bool {
        matches!(self, Self::New(_))
    }
    #[inline]
    pub fn is_seq(self) -> bool {
        matches!(self, Self::Seq(_))
    }
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_bool(self) -> bool {
        matches!(self, Self::Bool(_))
    }
    #[inline]
    pub fn is_null(self) -> bool {
        matches!(self, Self::Null(_))
    }
    #[inline]
    pub fn is_num(self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_big_int(self) -> bool {
        matches!(self, Self::BigInt(_))
    }
    #[inline]
    pub fn is_regex(self) -> bool {
        matches!(self, Self::Regex(_))
    }
    #[inline]
    pub fn is_tpl(self) -> bool {
        matches!(self, Self::Tpl(_))
    }
    #[inline]
    pub fn is_tagged_tpl(self) -> bool {
        matches!(self, Self::TaggedTpl(_))
    }
    #[inline]
    pub fn is_arrow(self) -> bool {
        matches!(self, Self::Arrow(_))
    }
    #[inline]
    pub fn is_class(self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_yield(self) -> bool {
        matches!(self, Self::Yield(_))
    }
    #[inline]
    pub fn is_meta_prop(self) -> bool {
        matches!(self, Self::MetaProp(_))
    }
    #[inline]
    pub fn is_await(self) -> bool {
        matches!(self, Self::Await(_))
    }
    #[inline]
    pub fn is_paren(self) -> bool {
        matches!(self, Self::Paren(_))
    }
    #[inline]
    pub fn is_jsx_member(self) -> bool {
        matches!(self, Self::JSXMember(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::JSXNamespacedName(_))
    }
    #[inline]
    pub fn is_jsx_empty(self) -> bool {
        matches!(self, Self::JSXEmpty(_))
    }
    #[inline]
    pub fn is_jsx_element(self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(self) -> bool {
        matches!(self, Self::JSXFragment(_))
    }
    #[inline]
    pub fn is_private_name(self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
    #[inline]
    pub fn is_opt_chain(self) -> bool {
        matches!(self, Self::OptChain(_))
    }
    #[inline]
    pub fn is_invalid(self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn is_super(self) -> bool {
        matches!(self, Self::Super(_))
    }
    #[inline]
    pub fn as_this(self) -> Option<ThisExpr> {
        match self {
            Self::This(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(self) -> Option<ArrayLit> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self) -> Option<ObjectLit> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self) -> Option<FnExpr> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary(self) -> Option<UnaryExpr> {
        match self {
            Self::Unary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_update(self) -> Option<UpdateExpr> {
        match self {
            Self::Update(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bin(self) -> Option<BinExpr> {
        match self {
            Self::Bin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignExpr> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(self) -> Option<MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(self) -> Option<SuperPropExpr> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_cond(self) -> Option<CondExpr> {
        match self {
            Self::Cond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import(self) -> Option<ImportExpr> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(self) -> Option<CallExpr> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_new(self) -> Option<NewExpr> {
        match self {
            Self::New(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_seq(self) -> Option<SeqExpr> {
        match self {
            Self::Seq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self) -> Option<Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(self) -> Option<Bool> {
        match self {
            Self::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null(self) -> Option<Null> {
        match self {
            Self::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self) -> Option<Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self) -> Option<BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_regex(self) -> Option<Regex> {
        match self {
            Self::Regex(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tpl(self) -> Option<Tpl> {
        match self {
            Self::Tpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tagged_tpl(self) -> Option<TaggedTpl> {
        match self {
            Self::TaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_arrow(self) -> Option<ArrowExpr> {
        match self {
            Self::Arrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(self) -> Option<ClassExpr> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_yield(self) -> Option<YieldExpr> {
        match self {
            Self::Yield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_meta_prop(self) -> Option<MetaPropExpr> {
        match self {
            Self::MetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_await(self) -> Option<AwaitExpr> {
        match self {
            Self::Await(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(self) -> Option<ParenExpr> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member(self) -> Option<JSXMemberExpr> {
        match self {
            Self::JSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self) -> Option<JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty(self) -> Option<JSXEmptyExpr> {
        match self {
            Self::JSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self) -> Option<JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self) -> Option<JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(self) -> Option<PrivateName> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(self) -> Option<OptChainExpr> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self) -> Option<Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super(self) -> Option<Super> {
        match self {
            Self::Super(it) => Some(it),
            _ => None,
        }
    }
}
impl Super {}
impl ExprOrSpread {
    #[inline]
    pub fn spread<'a>(self, ast: &crate::Ast<'a>) -> Option<Span> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_spread<'a>(self, ast: &mut crate::Ast<'a>, spread: Option<Span>) {
        let field_val: u64 = unsafe { spread.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u64 = unsafe { expr.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl BlockStmtOrExpr {
    #[inline]
    pub fn is_this(self) -> bool {
        matches!(self, Self::This(_))
    }
    #[inline]
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_fn(self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn is_unary(self) -> bool {
        matches!(self, Self::Unary(_))
    }
    #[inline]
    pub fn is_update(self) -> bool {
        matches!(self, Self::Update(_))
    }
    #[inline]
    pub fn is_bin(self) -> bool {
        matches!(self, Self::Bin(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_member(self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_super_prop(self) -> bool {
        matches!(self, Self::SuperProp(_))
    }
    #[inline]
    pub fn is_cond(self) -> bool {
        matches!(self, Self::Cond(_))
    }
    #[inline]
    pub fn is_import(self) -> bool {
        matches!(self, Self::Import(_))
    }
    #[inline]
    pub fn is_call(self) -> bool {
        matches!(self, Self::Call(_))
    }
    #[inline]
    pub fn is_new(self) -> bool {
        matches!(self, Self::New(_))
    }
    #[inline]
    pub fn is_seq(self) -> bool {
        matches!(self, Self::Seq(_))
    }
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_bool(self) -> bool {
        matches!(self, Self::Bool(_))
    }
    #[inline]
    pub fn is_null(self) -> bool {
        matches!(self, Self::Null(_))
    }
    #[inline]
    pub fn is_num(self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_big_int(self) -> bool {
        matches!(self, Self::BigInt(_))
    }
    #[inline]
    pub fn is_regex(self) -> bool {
        matches!(self, Self::Regex(_))
    }
    #[inline]
    pub fn is_tpl(self) -> bool {
        matches!(self, Self::Tpl(_))
    }
    #[inline]
    pub fn is_tagged_tpl(self) -> bool {
        matches!(self, Self::TaggedTpl(_))
    }
    #[inline]
    pub fn is_arrow(self) -> bool {
        matches!(self, Self::Arrow(_))
    }
    #[inline]
    pub fn is_class(self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_yield(self) -> bool {
        matches!(self, Self::Yield(_))
    }
    #[inline]
    pub fn is_meta_prop(self) -> bool {
        matches!(self, Self::MetaProp(_))
    }
    #[inline]
    pub fn is_await(self) -> bool {
        matches!(self, Self::Await(_))
    }
    #[inline]
    pub fn is_paren(self) -> bool {
        matches!(self, Self::Paren(_))
    }
    #[inline]
    pub fn is_jsx_member(self) -> bool {
        matches!(self, Self::JSXMember(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::JSXNamespacedName(_))
    }
    #[inline]
    pub fn is_jsx_empty(self) -> bool {
        matches!(self, Self::JSXEmpty(_))
    }
    #[inline]
    pub fn is_jsx_element(self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(self) -> bool {
        matches!(self, Self::JSXFragment(_))
    }
    #[inline]
    pub fn is_private_name(self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
    #[inline]
    pub fn is_opt_chain(self) -> bool {
        matches!(self, Self::OptChain(_))
    }
    #[inline]
    pub fn is_invalid(self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn is_block_stmt(self) -> bool {
        matches!(self, Self::BlockStmt(_))
    }
    #[inline]
    pub fn as_this(self) -> Option<ThisExpr> {
        match self {
            Self::This(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(self) -> Option<ArrayLit> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self) -> Option<ObjectLit> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self) -> Option<FnExpr> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary(self) -> Option<UnaryExpr> {
        match self {
            Self::Unary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_update(self) -> Option<UpdateExpr> {
        match self {
            Self::Update(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bin(self) -> Option<BinExpr> {
        match self {
            Self::Bin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignExpr> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(self) -> Option<MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(self) -> Option<SuperPropExpr> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_cond(self) -> Option<CondExpr> {
        match self {
            Self::Cond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import(self) -> Option<ImportExpr> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(self) -> Option<CallExpr> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_new(self) -> Option<NewExpr> {
        match self {
            Self::New(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_seq(self) -> Option<SeqExpr> {
        match self {
            Self::Seq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self) -> Option<Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(self) -> Option<Bool> {
        match self {
            Self::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null(self) -> Option<Null> {
        match self {
            Self::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self) -> Option<Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self) -> Option<BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_regex(self) -> Option<Regex> {
        match self {
            Self::Regex(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tpl(self) -> Option<Tpl> {
        match self {
            Self::Tpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tagged_tpl(self) -> Option<TaggedTpl> {
        match self {
            Self::TaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_arrow(self) -> Option<ArrowExpr> {
        match self {
            Self::Arrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(self) -> Option<ClassExpr> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_yield(self) -> Option<YieldExpr> {
        match self {
            Self::Yield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_meta_prop(self) -> Option<MetaPropExpr> {
        match self {
            Self::MetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_await(self) -> Option<AwaitExpr> {
        match self {
            Self::Await(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(self) -> Option<ParenExpr> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member(self) -> Option<JSXMemberExpr> {
        match self {
            Self::JSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self) -> Option<JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty(self) -> Option<JSXEmptyExpr> {
        match self {
            Self::JSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self) -> Option<JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self) -> Option<JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(self) -> Option<PrivateName> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(self) -> Option<OptChainExpr> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self) -> Option<Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_block_stmt(self) -> Option<BlockStmt> {
        match self {
            Self::BlockStmt(it) => Some(it),
            _ => None,
        }
    }
}
impl AssignTarget {
    #[inline]
    pub fn is_simple_ident(self) -> bool {
        matches!(self, Self::SimpleIdent(_))
    }
    #[inline]
    pub fn is_simple_member(self) -> bool {
        matches!(self, Self::SimpleMember(_))
    }
    #[inline]
    pub fn is_simple_super_prop(self) -> bool {
        matches!(self, Self::SimpleSuperProp(_))
    }
    #[inline]
    pub fn is_simple_paren(self) -> bool {
        matches!(self, Self::SimpleParen(_))
    }
    #[inline]
    pub fn is_simple_opt_chain(self) -> bool {
        matches!(self, Self::SimpleOptChain(_))
    }
    #[inline]
    pub fn is_simple_invalid(self) -> bool {
        matches!(self, Self::SimpleInvalid(_))
    }
    #[inline]
    pub fn is_pat_array(self) -> bool {
        matches!(self, Self::PatArray(_))
    }
    #[inline]
    pub fn is_pat_object(self) -> bool {
        matches!(self, Self::PatObject(_))
    }
    #[inline]
    pub fn is_pattern_invalid(self) -> bool {
        matches!(self, Self::PatternInvalid(_))
    }
    #[inline]
    pub fn as_simple_ident(self) -> Option<BindingIdent> {
        match self {
            Self::SimpleIdent(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_simple_member(self) -> Option<MemberExpr> {
        match self {
            Self::SimpleMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_simple_super_prop(self) -> Option<SuperPropExpr> {
        match self {
            Self::SimpleSuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_simple_paren(self) -> Option<ParenExpr> {
        match self {
            Self::SimpleParen(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_simple_opt_chain(self) -> Option<OptChainExpr> {
        match self {
            Self::SimpleOptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_simple_invalid(self) -> Option<Invalid> {
        match self {
            Self::SimpleInvalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pat_array(self) -> Option<ArrayPat> {
        match self {
            Self::PatArray(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pat_object(self) -> Option<ObjectPat> {
        match self {
            Self::PatObject(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pattern_invalid(self) -> Option<Invalid> {
        match self {
            Self::PatternInvalid(it) => Some(it),
            _ => None,
        }
    }
}
impl AssignTargetPat {
    #[inline]
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_invalid(self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn as_array(self) -> Option<ArrayPat> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self) -> Option<ObjectPat> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self) -> Option<Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl SimpleAssignTarget {
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_member(self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_super_prop(self) -> bool {
        matches!(self, Self::SuperProp(_))
    }
    #[inline]
    pub fn is_paren(self) -> bool {
        matches!(self, Self::Paren(_))
    }
    #[inline]
    pub fn is_opt_chain(self) -> bool {
        matches!(self, Self::OptChain(_))
    }
    #[inline]
    pub fn is_invalid(self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn as_ident(self) -> Option<BindingIdent> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(self) -> Option<MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(self) -> Option<SuperPropExpr> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(self) -> Option<ParenExpr> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(self) -> Option<OptChainExpr> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self) -> Option<Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl OptChainExpr {
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn base<'a>(self, ast: &crate::Ast<'a>) -> OptChainBase {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_base<'a>(self, ast: &mut crate::Ast<'a>, base: OptChainBase) {
        let field_val: u64 = unsafe { base.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl OptChainBase {
    #[inline]
    pub fn is_member(self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_call(self) -> bool {
        matches!(self, Self::Call(_))
    }
    #[inline]
    pub fn as_member(self) -> Option<MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(self) -> Option<OptCall> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
}
impl OptCall {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn args<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExprOrSpread> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        let field_val: u64 = unsafe { callee.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_args<'a>(self, ast: &mut crate::Ast<'a>, args: TypedSubRange<ExprOrSpread>) {
        let field_val: u64 = unsafe { args.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl Invalid {}
impl Function {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Param> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        BlockStmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_generator<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: TypedSubRange<Param>) {
        let field_val: u64 = unsafe { params.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_is_generator<'a>(self, ast: &mut crate::Ast<'a>, is_generator: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = is_generator.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_is_async<'a>(self, ast: &mut crate::Ast<'a>, is_async: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = is_async.to_extra_data(ast);
        };
    }
}
impl Param {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn pat<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let field_val: u64 = unsafe { decorators.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_pat<'a>(self, ast: &mut crate::Ast<'a>, pat: Pat) {
        let field_val: u64 = unsafe { pat.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl ParamOrTsParamProp {
    #[inline]
    pub fn is_param(self) -> bool {
        matches!(self, Self::Param(_))
    }
    #[inline]
    pub fn as_param(self) -> Option<Param> {
        match self {
            Self::Param(it) => Some(it),
            _ => None,
        }
    }
}
impl Class {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn is_abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ClassMember> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn super_class<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let field_val: u64 = unsafe { decorators.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_is_abstract<'a>(self, ast: &mut crate::Ast<'a>, is_abstract: bool) {
        let field_val: u32 = is_abstract as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<ClassMember>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_super_class<'a>(self, ast: &mut crate::Ast<'a>, super_class: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = super_class.to_extra_data(ast);
        };
    }
}
impl ClassMember {
    #[inline]
    pub fn is_constructor(self) -> bool {
        matches!(self, Self::Constructor(_))
    }
    #[inline]
    pub fn is_method(self) -> bool {
        matches!(self, Self::Method(_))
    }
    #[inline]
    pub fn is_private_method(self) -> bool {
        matches!(self, Self::PrivateMethod(_))
    }
    #[inline]
    pub fn is_class_prop(self) -> bool {
        matches!(self, Self::ClassProp(_))
    }
    #[inline]
    pub fn is_private_prop(self) -> bool {
        matches!(self, Self::PrivateProp(_))
    }
    #[inline]
    pub fn is_empty(self) -> bool {
        matches!(self, Self::Empty(_))
    }
    #[inline]
    pub fn is_static_block(self) -> bool {
        matches!(self, Self::StaticBlock(_))
    }
    #[inline]
    pub fn is_auto_accessor(self) -> bool {
        matches!(self, Self::AutoAccessor(_))
    }
    #[inline]
    pub fn as_constructor(self) -> Option<Constructor> {
        match self {
            Self::Constructor(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_method(self) -> Option<ClassMethod> {
        match self {
            Self::Method(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_method(self) -> Option<PrivateMethod> {
        match self {
            Self::PrivateMethod(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class_prop(self) -> Option<ClassProp> {
        match self {
            Self::ClassProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_prop(self) -> Option<PrivateProp> {
        match self {
            Self::PrivateProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_empty(self) -> Option<EmptyStmt> {
        match self {
            Self::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_static_block(self) -> Option<StaticBlock> {
        match self {
            Self::StaticBlock(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_auto_accessor(self) -> Option<AutoAccessor> {
        match self {
            Self::AutoAccessor(it) => Some(it),
            _ => None,
        }
    }
}
impl ClassProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u64 = unsafe { key.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let field_val: u32 = is_static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
}
impl PrivateProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PrivateName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        PrivateName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PrivateName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = is_static.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
}
impl ClassMethod {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MethodKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodKind>(raw as u8) }
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 104usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u64 = unsafe { key.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MethodKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let field_val: u32 = is_static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340277194906489532287416500882690277375u128)
            | (((field_val as u128) & 255u128) << 104usize);
    }
}
impl PrivateMethod {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PrivateName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PrivateName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MethodKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodKind>(raw as u8) }
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PrivateName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MethodKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let field_val: u32 = is_static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
}
impl Constructor {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Option<BlockStmt> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BlockStmt(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ParamOrTsParamProp> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u64 = unsafe { key.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Option<BlockStmt>) {
        let field_val: u32 = body.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_params<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        params: TypedSubRange<ParamOrTsParamProp>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = params.to_extra_data(ast);
        };
    }
}
impl Decorator {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u64 = unsafe { expr.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl StaticBlock {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BlockStmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl Key {
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_num(self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_computed(self) -> bool {
        matches!(self, Self::Computed(_))
    }
    #[inline]
    pub fn is_big_int(self) -> bool {
        matches!(self, Self::BigInt(_))
    }
    #[inline]
    pub fn is_private(self) -> bool {
        matches!(self, Self::Private(_))
    }
    #[inline]
    pub fn as_ident(self) -> Option<IdentName> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self) -> Option<Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(self) -> Option<ComputedPropName> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self) -> Option<BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private(self) -> Option<PrivateName> {
        match self {
            Self::Private(it) => Some(it),
            _ => None,
        }
    }
}
impl AutoAccessor {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> Key {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: Key) {
        let field_val: u64 = unsafe { key.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let field_val: u32 = is_static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
}
impl Prop {
    #[inline]
    pub fn is_shorthand(self) -> bool {
        matches!(self, Self::Shorthand(_))
    }
    #[inline]
    pub fn is_key_value(self) -> bool {
        matches!(self, Self::KeyValue(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_getter(self) -> bool {
        matches!(self, Self::Getter(_))
    }
    #[inline]
    pub fn is_setter(self) -> bool {
        matches!(self, Self::Setter(_))
    }
    #[inline]
    pub fn is_method(self) -> bool {
        matches!(self, Self::Method(_))
    }
    #[inline]
    pub fn as_shorthand(self) -> Option<Ident> {
        match self {
            Self::Shorthand(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_key_value(self) -> Option<KeyValueProp> {
        match self {
            Self::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignProp> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_getter(self) -> Option<GetterProp> {
        match self {
            Self::Getter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_setter(self) -> Option<SetterProp> {
        match self {
            Self::Setter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_method(self) -> Option<MethodProp> {
        match self {
            Self::Method(it) => Some(it),
            _ => None,
        }
    }
}
impl KeyValueProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u64 = unsafe { key.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Expr) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl AssignProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: Ident) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Expr) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl GetterProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u64 = unsafe { key.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl SetterProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u64 = unsafe { key.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl MethodProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u64 = unsafe { key.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl PropName {
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_num(self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_computed(self) -> bool {
        matches!(self, Self::Computed(_))
    }
    #[inline]
    pub fn is_big_int(self) -> bool {
        matches!(self, Self::BigInt(_))
    }
    #[inline]
    pub fn as_ident(self) -> Option<IdentName> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self) -> Option<Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(self) -> Option<ComputedPropName> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self) -> Option<BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
}
impl ComputedPropName {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u64 = unsafe { expr.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl Pat {
    #[inline]
    pub fn is_expr_this(self) -> bool {
        matches!(self, Self::ExprThis(_))
    }
    #[inline]
    pub fn is_expr_array(self) -> bool {
        matches!(self, Self::ExprArray(_))
    }
    #[inline]
    pub fn is_expr_object(self) -> bool {
        matches!(self, Self::ExprObject(_))
    }
    #[inline]
    pub fn is_expr_fn(self) -> bool {
        matches!(self, Self::ExprFn(_))
    }
    #[inline]
    pub fn is_expr_unary(self) -> bool {
        matches!(self, Self::ExprUnary(_))
    }
    #[inline]
    pub fn is_expr_update(self) -> bool {
        matches!(self, Self::ExprUpdate(_))
    }
    #[inline]
    pub fn is_expr_bin(self) -> bool {
        matches!(self, Self::ExprBin(_))
    }
    #[inline]
    pub fn is_expr_assign(self) -> bool {
        matches!(self, Self::ExprAssign(_))
    }
    #[inline]
    pub fn is_expr_member(self) -> bool {
        matches!(self, Self::ExprMember(_))
    }
    #[inline]
    pub fn is_expr_super_prop(self) -> bool {
        matches!(self, Self::ExprSuperProp(_))
    }
    #[inline]
    pub fn is_expr_cond(self) -> bool {
        matches!(self, Self::ExprCond(_))
    }
    #[inline]
    pub fn is_expr_import(self) -> bool {
        matches!(self, Self::ExprImport(_))
    }
    #[inline]
    pub fn is_expr_call(self) -> bool {
        matches!(self, Self::ExprCall(_))
    }
    #[inline]
    pub fn is_expr_new(self) -> bool {
        matches!(self, Self::ExprNew(_))
    }
    #[inline]
    pub fn is_expr_seq(self) -> bool {
        matches!(self, Self::ExprSeq(_))
    }
    #[inline]
    pub fn is_expr_ident(self) -> bool {
        matches!(self, Self::ExprIdent(_))
    }
    #[inline]
    pub fn is_expr_str(self) -> bool {
        matches!(self, Self::ExprStr(_))
    }
    #[inline]
    pub fn is_expr_bool(self) -> bool {
        matches!(self, Self::ExprBool(_))
    }
    #[inline]
    pub fn is_expr_null(self) -> bool {
        matches!(self, Self::ExprNull(_))
    }
    #[inline]
    pub fn is_expr_num(self) -> bool {
        matches!(self, Self::ExprNum(_))
    }
    #[inline]
    pub fn is_expr_big_int(self) -> bool {
        matches!(self, Self::ExprBigInt(_))
    }
    #[inline]
    pub fn is_expr_regex(self) -> bool {
        matches!(self, Self::ExprRegex(_))
    }
    #[inline]
    pub fn is_expr_tpl(self) -> bool {
        matches!(self, Self::ExprTpl(_))
    }
    #[inline]
    pub fn is_expr_tagged_tpl(self) -> bool {
        matches!(self, Self::ExprTaggedTpl(_))
    }
    #[inline]
    pub fn is_expr_arrow(self) -> bool {
        matches!(self, Self::ExprArrow(_))
    }
    #[inline]
    pub fn is_expr_class(self) -> bool {
        matches!(self, Self::ExprClass(_))
    }
    #[inline]
    pub fn is_expr_yield(self) -> bool {
        matches!(self, Self::ExprYield(_))
    }
    #[inline]
    pub fn is_expr_meta_prop(self) -> bool {
        matches!(self, Self::ExprMetaProp(_))
    }
    #[inline]
    pub fn is_expr_await(self) -> bool {
        matches!(self, Self::ExprAwait(_))
    }
    #[inline]
    pub fn is_expr_paren(self) -> bool {
        matches!(self, Self::ExprParen(_))
    }
    #[inline]
    pub fn is_expr_jsx_member(self) -> bool {
        matches!(self, Self::ExprJSXMember(_))
    }
    #[inline]
    pub fn is_expr_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::ExprJSXNamespacedName(_))
    }
    #[inline]
    pub fn is_expr_jsx_empty(self) -> bool {
        matches!(self, Self::ExprJSXEmpty(_))
    }
    #[inline]
    pub fn is_expr_jsx_element(self) -> bool {
        matches!(self, Self::ExprJSXElement(_))
    }
    #[inline]
    pub fn is_expr_jsx_fragment(self) -> bool {
        matches!(self, Self::ExprJSXFragment(_))
    }
    #[inline]
    pub fn is_expr_private_name(self) -> bool {
        matches!(self, Self::ExprPrivateName(_))
    }
    #[inline]
    pub fn is_expr_opt_chain(self) -> bool {
        matches!(self, Self::ExprOptChain(_))
    }
    #[inline]
    pub fn is_expr_invalid(self) -> bool {
        matches!(self, Self::ExprInvalid(_))
    }
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_rest(self) -> bool {
        matches!(self, Self::Rest(_))
    }
    #[inline]
    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_invalid(self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn as_expr_this(self) -> Option<ThisExpr> {
        match self {
            Self::ExprThis(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_array(self) -> Option<ArrayLit> {
        match self {
            Self::ExprArray(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_object(self) -> Option<ObjectLit> {
        match self {
            Self::ExprObject(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_fn(self) -> Option<FnExpr> {
        match self {
            Self::ExprFn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_unary(self) -> Option<UnaryExpr> {
        match self {
            Self::ExprUnary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_update(self) -> Option<UpdateExpr> {
        match self {
            Self::ExprUpdate(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_bin(self) -> Option<BinExpr> {
        match self {
            Self::ExprBin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_assign(self) -> Option<AssignExpr> {
        match self {
            Self::ExprAssign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_member(self) -> Option<MemberExpr> {
        match self {
            Self::ExprMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_super_prop(self) -> Option<SuperPropExpr> {
        match self {
            Self::ExprSuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_cond(self) -> Option<CondExpr> {
        match self {
            Self::ExprCond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_import(self) -> Option<ImportExpr> {
        match self {
            Self::ExprImport(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_call(self) -> Option<CallExpr> {
        match self {
            Self::ExprCall(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_new(self) -> Option<NewExpr> {
        match self {
            Self::ExprNew(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_seq(self) -> Option<SeqExpr> {
        match self {
            Self::ExprSeq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_ident(self) -> Option<Ident> {
        match self {
            Self::ExprIdent(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_str(self) -> Option<Str> {
        match self {
            Self::ExprStr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_bool(self) -> Option<Bool> {
        match self {
            Self::ExprBool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_null(self) -> Option<Null> {
        match self {
            Self::ExprNull(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_num(self) -> Option<Number> {
        match self {
            Self::ExprNum(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_big_int(self) -> Option<BigInt> {
        match self {
            Self::ExprBigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_regex(self) -> Option<Regex> {
        match self {
            Self::ExprRegex(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_tpl(self) -> Option<Tpl> {
        match self {
            Self::ExprTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_tagged_tpl(self) -> Option<TaggedTpl> {
        match self {
            Self::ExprTaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_arrow(self) -> Option<ArrowExpr> {
        match self {
            Self::ExprArrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_class(self) -> Option<ClassExpr> {
        match self {
            Self::ExprClass(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_yield(self) -> Option<YieldExpr> {
        match self {
            Self::ExprYield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_meta_prop(self) -> Option<MetaPropExpr> {
        match self {
            Self::ExprMetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_await(self) -> Option<AwaitExpr> {
        match self {
            Self::ExprAwait(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_paren(self) -> Option<ParenExpr> {
        match self {
            Self::ExprParen(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_member(self) -> Option<JSXMemberExpr> {
        match self {
            Self::ExprJSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_namespaced_name(self) -> Option<JSXNamespacedName> {
        match self {
            Self::ExprJSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_empty(self) -> Option<JSXEmptyExpr> {
        match self {
            Self::ExprJSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_element(self) -> Option<JSXElement> {
        match self {
            Self::ExprJSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_jsx_fragment(self) -> Option<JSXFragment> {
        match self {
            Self::ExprJSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_private_name(self) -> Option<PrivateName> {
        match self {
            Self::ExprPrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_opt_chain(self) -> Option<OptChainExpr> {
        match self {
            Self::ExprOptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr_invalid(self) -> Option<Invalid> {
        match self {
            Self::ExprInvalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self) -> Option<BindingIdent> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(self) -> Option<ArrayPat> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_rest(self) -> Option<RestPat> {
        match self {
            Self::Rest(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self) -> Option<ObjectPat> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignPat> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self) -> Option<Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl ArrayPat {
    #[inline]
    pub fn elems<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Option<Pat>> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_elems<'a>(self, ast: &mut crate::Ast<'a>, elems: TypedSubRange<Option<Pat>>) {
        let field_val: u64 = unsafe { elems.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl ObjectPat {
    #[inline]
    pub fn props<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ObjectPatProp> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_props<'a>(self, ast: &mut crate::Ast<'a>, props: TypedSubRange<ObjectPatProp>) {
        let field_val: u64 = unsafe { props.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl AssignPat {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: Pat) {
        let field_val: u64 = unsafe { left.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u64 = unsafe { right.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl RestPat {
    #[inline]
    pub fn dot_3_token<'a>(self, ast: &crate::Ast<'a>) -> Span {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_dot3_token<'a>(self, ast: &mut crate::Ast<'a>, dot3_token: Span) {
        let field_val: u64 = unsafe { dot3_token.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Pat) {
        let field_val: u64 = unsafe { arg.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl ObjectPatProp {
    #[inline]
    pub fn is_key_value(self) -> bool {
        matches!(self, Self::KeyValue(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_rest(self) -> bool {
        matches!(self, Self::Rest(_))
    }
    #[inline]
    pub fn as_key_value(self) -> Option<KeyValuePatProp> {
        match self {
            Self::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignPatProp> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_rest(self) -> Option<RestPat> {
        match self {
            Self::Rest(it) => Some(it),
            _ => None,
        }
    }
}
impl KeyValuePatProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u64 = unsafe { key.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Pat) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl AssignPatProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> BindingIdent {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        BindingIdent(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: BindingIdent) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl Ident {
    #[inline]
    pub fn sym<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn symbol_id<'a>(self, ast: &crate::Ast<'a>) -> Option<SymbolId> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != 0).then(|| unsafe {
            <SymbolId as oxc_index::Idx>::from_usize_unchecked(raw.wrapping_sub(1) as usize)
        })
    }
    #[inline]
    pub fn set_sym<'a>(self, ast: &mut crate::Ast<'a>, sym: Atom<'a>) {
        let field_val: u32 = unsafe { sym.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_symbol_id<'a>(self, ast: &mut crate::Ast<'a>, symbol_id: Option<SymbolId>) {
        let field_val: u32 = symbol_id.map_or(0, |id| id.index() as u32 + 1);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl IdentName {
    #[inline]
    pub fn sym<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_sym<'a>(self, ast: &mut crate::Ast<'a>, sym: Atom<'a>) {
        let field_val: u32 = unsafe { sym.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl PrivateName {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: Atom<'a>) {
        let field_val: u32 = unsafe { name.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl BindingIdent {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: Ident) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl Lit {
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_bool(self) -> bool {
        matches!(self, Self::Bool(_))
    }
    #[inline]
    pub fn is_null(self) -> bool {
        matches!(self, Self::Null(_))
    }
    #[inline]
    pub fn is_num(self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_big_int(self) -> bool {
        matches!(self, Self::BigInt(_))
    }
    #[inline]
    pub fn is_regex(self) -> bool {
        matches!(self, Self::Regex(_))
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(self) -> Option<Bool> {
        match self {
            Self::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null(self) -> Option<Null> {
        match self {
            Self::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self) -> Option<Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self) -> Option<BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_regex(self) -> Option<Regex> {
        match self {
            Self::Regex(it) => Some(it),
            _ => None,
        }
    }
}
impl Str {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Wtf8Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    wtf8_atom: Wtf8AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Wtf8Atom<'a>) {
        let field_val: u32 = unsafe { value.to_extra_data(ast).wtf8_atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Option<Atom<'a>>) {
        let field_val: u32 = unsafe { raw.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl Bool {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: bool) {
        let field_val: u32 = value as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
}
impl Null {}
impl Number {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> f64 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: f64) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Option<Atom<'a>>) {
        let field_val: u32 = unsafe { raw.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl BigInt {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Atom<'a>) {
        let field_val: u32 = unsafe { value.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Option<Atom<'a>>) {
        let field_val: u32 = unsafe { raw.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl Regex {
    #[inline]
    pub fn exp<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn flags<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_exp<'a>(self, ast: &mut crate::Ast<'a>, exp: Atom<'a>) {
        let field_val: u32 = unsafe { exp.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_flags<'a>(self, ast: &mut crate::Ast<'a>, flags: Atom<'a>) {
        let field_val: u32 = unsafe { flags.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl JSXObject {
    #[inline]
    pub fn is_jsx_member_expr(self) -> bool {
        matches!(self, Self::JSXMemberExpr(_))
    }
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn as_jsx_member_expr(self) -> Option<JSXMemberExpr> {
        match self {
            Self::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self) -> Option<Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXMemberExpr {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> JSXObject {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn prop<'a>(self, ast: &crate::Ast<'a>) -> IdentName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        IdentName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: JSXObject) {
        let field_val: u64 = unsafe { obj.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_prop<'a>(self, ast: &mut crate::Ast<'a>, prop: IdentName) {
        let field_val: u32 = prop.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl JSXNamespacedName {
    #[inline]
    pub fn ns<'a>(self, ast: &crate::Ast<'a>) -> IdentName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        IdentName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> IdentName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        IdentName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_ns<'a>(self, ast: &mut crate::Ast<'a>, ns: IdentName) {
        let field_val: u32 = ns.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: IdentName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl JSXEmptyExpr {}
impl JSXExprContainer {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> JSXExpr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: JSXExpr) {
        let field_val: u64 = unsafe { expr.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl JSXExpr {
    #[inline]
    pub fn is_this(self) -> bool {
        matches!(self, Self::This(_))
    }
    #[inline]
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_fn(self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn is_unary(self) -> bool {
        matches!(self, Self::Unary(_))
    }
    #[inline]
    pub fn is_update(self) -> bool {
        matches!(self, Self::Update(_))
    }
    #[inline]
    pub fn is_bin(self) -> bool {
        matches!(self, Self::Bin(_))
    }
    #[inline]
    pub fn is_assign(self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_member(self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_super_prop(self) -> bool {
        matches!(self, Self::SuperProp(_))
    }
    #[inline]
    pub fn is_cond(self) -> bool {
        matches!(self, Self::Cond(_))
    }
    #[inline]
    pub fn is_import(self) -> bool {
        matches!(self, Self::Import(_))
    }
    #[inline]
    pub fn is_call(self) -> bool {
        matches!(self, Self::Call(_))
    }
    #[inline]
    pub fn is_new(self) -> bool {
        matches!(self, Self::New(_))
    }
    #[inline]
    pub fn is_seq(self) -> bool {
        matches!(self, Self::Seq(_))
    }
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_bool(self) -> bool {
        matches!(self, Self::Bool(_))
    }
    #[inline]
    pub fn is_null(self) -> bool {
        matches!(self, Self::Null(_))
    }
    #[inline]
    pub fn is_num(self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_big_int(self) -> bool {
        matches!(self, Self::BigInt(_))
    }
    #[inline]
    pub fn is_regex(self) -> bool {
        matches!(self, Self::Regex(_))
    }
    #[inline]
    pub fn is_tpl(self) -> bool {
        matches!(self, Self::Tpl(_))
    }
    #[inline]
    pub fn is_tagged_tpl(self) -> bool {
        matches!(self, Self::TaggedTpl(_))
    }
    #[inline]
    pub fn is_arrow(self) -> bool {
        matches!(self, Self::Arrow(_))
    }
    #[inline]
    pub fn is_class(self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_yield(self) -> bool {
        matches!(self, Self::Yield(_))
    }
    #[inline]
    pub fn is_meta_prop(self) -> bool {
        matches!(self, Self::MetaProp(_))
    }
    #[inline]
    pub fn is_await(self) -> bool {
        matches!(self, Self::Await(_))
    }
    #[inline]
    pub fn is_paren(self) -> bool {
        matches!(self, Self::Paren(_))
    }
    #[inline]
    pub fn is_jsx_member(self) -> bool {
        matches!(self, Self::JSXMember(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::JSXNamespacedName(_))
    }
    #[inline]
    pub fn is_jsx_empty(self) -> bool {
        matches!(self, Self::JSXEmpty(_))
    }
    #[inline]
    pub fn is_jsx_element(self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(self) -> bool {
        matches!(self, Self::JSXFragment(_))
    }
    #[inline]
    pub fn is_private_name(self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
    #[inline]
    pub fn is_opt_chain(self) -> bool {
        matches!(self, Self::OptChain(_))
    }
    #[inline]
    pub fn is_invalid(self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn is_jsx_empty_expr(self) -> bool {
        matches!(self, Self::JSXEmptyExpr(_))
    }
    #[inline]
    pub fn as_this(self) -> Option<ThisExpr> {
        match self {
            Self::This(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(self) -> Option<ArrayLit> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self) -> Option<ObjectLit> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self) -> Option<FnExpr> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary(self) -> Option<UnaryExpr> {
        match self {
            Self::Unary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_update(self) -> Option<UpdateExpr> {
        match self {
            Self::Update(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bin(self) -> Option<BinExpr> {
        match self {
            Self::Bin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self) -> Option<AssignExpr> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(self) -> Option<MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(self) -> Option<SuperPropExpr> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_cond(self) -> Option<CondExpr> {
        match self {
            Self::Cond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import(self) -> Option<ImportExpr> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(self) -> Option<CallExpr> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_new(self) -> Option<NewExpr> {
        match self {
            Self::New(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_seq(self) -> Option<SeqExpr> {
        match self {
            Self::Seq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self) -> Option<Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(self) -> Option<Bool> {
        match self {
            Self::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null(self) -> Option<Null> {
        match self {
            Self::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self) -> Option<Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self) -> Option<BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_regex(self) -> Option<Regex> {
        match self {
            Self::Regex(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tpl(self) -> Option<Tpl> {
        match self {
            Self::Tpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tagged_tpl(self) -> Option<TaggedTpl> {
        match self {
            Self::TaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_arrow(self) -> Option<ArrowExpr> {
        match self {
            Self::Arrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(self) -> Option<ClassExpr> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_yield(self) -> Option<YieldExpr> {
        match self {
            Self::Yield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_meta_prop(self) -> Option<MetaPropExpr> {
        match self {
            Self::MetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_await(self) -> Option<AwaitExpr> {
        match self {
            Self::Await(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(self) -> Option<ParenExpr> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member(self) -> Option<JSXMemberExpr> {
        match self {
            Self::JSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self) -> Option<JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty(self) -> Option<JSXEmptyExpr> {
        match self {
            Self::JSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self) -> Option<JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self) -> Option<JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(self) -> Option<PrivateName> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(self) -> Option<OptChainExpr> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self) -> Option<Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty_expr(self) -> Option<JSXEmptyExpr> {
        match self {
            Self::JSXEmptyExpr(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXSpreadChild {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u64 = unsafe { expr.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl JSXElementName {
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_jsx_member_expr(self) -> bool {
        matches!(self, Self::JSXMemberExpr(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::JSXNamespacedName(_))
    }
    #[inline]
    pub fn as_ident(self) -> Option<Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member_expr(self) -> Option<JSXMemberExpr> {
        match self {
            Self::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self) -> Option<JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXOpeningElement {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JSXElementName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn self_closing<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn attrs<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JSXAttrOrSpread> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JSXElementName) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_self_closing<'a>(self, ast: &mut crate::Ast<'a>, self_closing: bool) {
        let field_val: u32 = self_closing as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_attrs<'a>(self, ast: &mut crate::Ast<'a>, attrs: TypedSubRange<JSXAttrOrSpread>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attrs.to_extra_data(ast);
        };
    }
}
impl JSXAttrOrSpread {
    #[inline]
    pub fn is_jsx_attr(self) -> bool {
        matches!(self, Self::JSXAttr(_))
    }
    #[inline]
    pub fn is_spread_element(self) -> bool {
        matches!(self, Self::SpreadElement(_))
    }
    #[inline]
    pub fn as_jsx_attr(self) -> Option<JSXAttr> {
        match self {
            Self::JSXAttr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_spread_element(self) -> Option<SpreadElement> {
        match self {
            Self::SpreadElement(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXClosingElement {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JSXElementName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JSXElementName) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl JSXAttr {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JSXAttrName {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<JSXAttrValue> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JSXAttrName) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<JSXAttrValue>) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl JSXAttrName {
    #[inline]
    pub fn is_ident(self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::JSXNamespacedName(_))
    }
    #[inline]
    pub fn as_ident(self) -> Option<IdentName> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self) -> Option<JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXAttrValue {
    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_jsx_expr_container(self) -> bool {
        matches!(self, Self::JSXExprContainer(_))
    }
    #[inline]
    pub fn is_jsx_element(self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(self) -> bool {
        matches!(self, Self::JSXFragment(_))
    }
    #[inline]
    pub fn as_str(self) -> Option<Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expr_container(self) -> Option<JSXExprContainer> {
        match self {
            Self::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self) -> Option<JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self) -> Option<JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXText {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        unsafe {
            ExtraDataCompact::from_extra_data(
                ExtraData {
                    atom: AtomId::from_raw(raw),
                },
                ast,
            )
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Atom<'a>) {
        let field_val: u32 = unsafe { value.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Atom<'a>) {
        let field_val: u32 = unsafe { raw.to_extra_data(ast).atom }.raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl JSXElement {
    #[inline]
    pub fn opening<'a>(self, ast: &crate::Ast<'a>) -> JSXOpeningElement {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        JSXOpeningElement(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JSXElementChild> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn closing<'a>(self, ast: &crate::Ast<'a>) -> Option<JSXClosingElement> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| JSXClosingElement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_opening<'a>(self, ast: &mut crate::Ast<'a>, opening: JSXOpeningElement) {
        let field_val: u32 = opening.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_children<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        children: TypedSubRange<JSXElementChild>,
    ) {
        let field_val: u64 = unsafe { children.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_closing<'a>(self, ast: &mut crate::Ast<'a>, closing: Option<JSXClosingElement>) {
        let field_val: u32 = closing.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl JSXElementChild {
    #[inline]
    pub fn is_jsx_text(self) -> bool {
        matches!(self, Self::JSXText(_))
    }
    #[inline]
    pub fn is_jsx_expr_container(self) -> bool {
        matches!(self, Self::JSXExprContainer(_))
    }
    #[inline]
    pub fn is_jsx_spread_child(self) -> bool {
        matches!(self, Self::JSXSpreadChild(_))
    }
    #[inline]
    pub fn is_jsx_element(self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(self) -> bool {
        matches!(self, Self::JSXFragment(_))
    }
    #[inline]
    pub fn as_jsx_text(self) -> Option<JSXText> {
        match self {
            Self::JSXText(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expr_container(self) -> Option<JSXExprContainer> {
        match self {
            Self::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_spread_child(self) -> Option<JSXSpreadChild> {
        match self {
            Self::JSXSpreadChild(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self) -> Option<JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self) -> Option<JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXFragment {
    #[inline]
    pub fn opening<'a>(self, ast: &crate::Ast<'a>) -> JSXOpeningFragment {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        JSXOpeningFragment(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JSXElementChild> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn closing<'a>(self, ast: &crate::Ast<'a>) -> JSXClosingFragment {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        JSXClosingFragment(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_opening<'a>(self, ast: &mut crate::Ast<'a>, opening: JSXOpeningFragment) {
        let field_val: u32 = opening.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_children<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        children: TypedSubRange<JSXElementChild>,
    ) {
        let field_val: u64 = unsafe { children.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_closing<'a>(self, ast: &mut crate::Ast<'a>, closing: JSXClosingFragment) {
        let field_val: u32 = closing.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl JSXOpeningFragment {}
impl JSXClosingFragment {}
