#![allow(
    unused,
    clippy::useless_conversion,
    clippy::identity_op,
    clippy::erasing_op,
    clippy::let_and_return
)]
use crate::*;
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
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn shebang<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<ModuleItem>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_shebang<'a>(self, ast: &mut crate::Ast<'a>, shebang: Option<Atom<'a>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = shebang.to_extra_data(ast);
        };
    }
}
impl Script {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn shebang<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_shebang<'a>(self, ast: &mut crate::Ast<'a>, shebang: Option<Atom<'a>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = shebang.to_extra_data(ast);
        };
    }
}
impl ModuleItem {
    #[inline]
    pub fn is_module_decl(self) -> bool {
        matches!(self, Self::ModuleDecl(_))
    }
    #[inline]
    pub fn is_stmt(self) -> bool {
        matches!(self, Self::Stmt(_))
    }
    #[inline]
    pub fn as_module_decl(self) -> Option<ModuleDecl> {
        match self {
            Self::ModuleDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt(self) -> Option<Stmt> {
        match self {
            Self::Stmt(it) => Some(it),
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
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn src<'a>(self, ast: &crate::Ast<'a>) -> Str {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn with<'a>(self, ast: &crate::Ast<'a>) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> ImportPhase {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(3usize),
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
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_src<'a>(self, ast: &mut crate::Ast<'a>, src: Str) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = src.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_type_only<'a>(self, ast: &mut crate::Ast<'a>, type_only: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(2usize),
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(3usize),
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn imported<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe {
            ModuleExportName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast)
        }
    }
    #[inline]
    pub fn is_type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: Ident) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_imported<'a>(self, ast: &mut crate::Ast<'a>, imported: ModuleExportName) {
        let field_val: u32 = imported.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_is_type_only<'a>(self, ast: &mut crate::Ast<'a>, is_type_only: bool) {
        let field_val: u32 = is_type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
}
impl ImportDefaultSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: Ident) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl ImportStarAsSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: Ident) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl ExportDecl {
    #[inline]
    pub fn decl<'a>(self, ast: &crate::Ast<'a>) -> Decl {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Decl::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_decl<'a>(self, ast: &mut crate::Ast<'a>, decl: Decl) {
        let field_val: u32 = decl.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl NamedExport {
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExportSpecifier> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn src<'a>(self, ast: &crate::Ast<'a>) -> Option<Str> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn with<'a>(self, ast: &crate::Ast<'a>) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
        specifiers: TypedSubRange<ExportSpecifier>,
    ) {
        let field_val: u64 = unsafe { specifiers.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_src<'a>(self, ast: &mut crate::Ast<'a>, src: Option<Str>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = src.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_type_only<'a>(self, ast: &mut crate::Ast<'a>, type_only: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(2usize),
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe {
            ModuleExportName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast)
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: ModuleExportName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_exported<'a>(self, ast: &mut crate::Ast<'a>, exported: Ident) {
        let field_val: u32 = exported.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl ExportNamedSpecifier {
    #[inline]
    pub fn orig<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe {
            ModuleExportName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast)
        }
    }
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe {
            ModuleExportName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast)
        }
    }
    #[inline]
    pub fn is_type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_orig<'a>(self, ast: &mut crate::Ast<'a>, orig: ModuleExportName) {
        let field_val: u32 = orig.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_exported<'a>(self, ast: &mut crate::Ast<'a>, exported: ModuleExportName) {
        let field_val: u32 = exported.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_is_type_only<'a>(self, ast: &mut crate::Ast<'a>, is_type_only: bool) {
        let field_val: u32 = is_type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
}
impl ExportDefaultDecl {
    #[inline]
    pub fn decl<'a>(self, ast: &crate::Ast<'a>) -> DefaultDecl {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { DefaultDecl::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_decl<'a>(self, ast: &mut crate::Ast<'a>, decl: DefaultDecl) {
        let field_val: u32 = decl.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl ExportAll {
    #[inline]
    pub fn src<'a>(self, ast: &crate::Ast<'a>) -> Str {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Str::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn with<'a>(self, ast: &crate::Ast<'a>) -> Option<ObjectLit> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { ObjectLit::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_src<'a>(self, ast: &mut crate::Ast<'a>, src: Str) {
        let field_val: u32 = src.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_type_only<'a>(self, ast: &mut crate::Ast<'a>, type_only: bool) {
        let field_val: u32 = type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_with<'a>(self, ast: &mut crate::Ast<'a>, with: Option<ObjectLit>) {
        let field_val: u32 = crate::OptionalNodeId::from(with.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl BlockStmt {
    #[inline]
    pub fn stmts<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn scope_id<'a>(self, ast: &crate::Ast<'a>) -> Option<ScopeId> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        (raw != 0).then(|| unsafe {
            <ScopeId as oxc_index::Idx>::from_usize_unchecked(raw.wrapping_sub(1) as usize)
        })
    }
    #[inline]
    pub fn set_stmts<'a>(self, ast: &mut crate::Ast<'a>, stmts: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { stmts.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_scope_id<'a>(self, ast: &mut crate::Ast<'a>, scope_id: Option<ScopeId>) {
        let field_val: u32 = scope_id.map_or(0, |id| id.index() as u32 + 1);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
}
impl Stmt {
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
    pub fn is_decl(self) -> bool {
        matches!(self, Self::Decl(_))
    }
    #[inline]
    pub fn is_expr(self) -> bool {
        matches!(self, Self::Expr(_))
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
    pub fn as_decl(self) -> Option<Decl> {
        match self {
            Self::Decl(it) => Some(it),
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl EmptyStmt {}
impl DebuggerStmt {}
impl WithStmt {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Stmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: Expr) {
        let field_val: u32 = obj.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl ReturnStmt {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Expr::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Option<Expr>) {
        let field_val: u32 = crate::OptionalNodeId::from(arg.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl LabeledStmt {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Stmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Ident) {
        let field_val: u32 = label.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl BreakStmt {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Option<Ident> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Option<Ident>) {
        let field_val: u32 = crate::OptionalNodeId::from(label.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl ContinueStmt {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Option<Ident> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Option<Ident>) {
        let field_val: u32 = crate::OptionalNodeId::from(label.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl IfStmt {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn cons<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Stmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn alt<'a>(self, ast: &crate::Ast<'a>) -> Option<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Stmt::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_cons<'a>(self, ast: &mut crate::Ast<'a>, cons: Stmt) {
        let field_val: u32 = cons.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_alt<'a>(self, ast: &mut crate::Ast<'a>, alt: Option<Stmt>) {
        let field_val: u32 = crate::OptionalNodeId::from(alt.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
}
impl SwitchStmt {
    #[inline]
    pub fn discriminant<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn cases<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<SwitchCase> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_discriminant<'a>(self, ast: &mut crate::Ast<'a>, discriminant: Expr) {
        let field_val: u32 = discriminant.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_cases<'a>(self, ast: &mut crate::Ast<'a>, cases: TypedSubRange<SwitchCase>) {
        let field_val: u64 = unsafe { cases.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl ThrowStmt {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Expr) {
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl TryStmt {
    #[inline]
    pub fn block<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { BlockStmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn handler<'a>(self, ast: &crate::Ast<'a>) -> Option<CatchClause> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { CatchClause::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn finalizer<'a>(self, ast: &crate::Ast<'a>) -> Option<BlockStmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { BlockStmt::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_block<'a>(self, ast: &mut crate::Ast<'a>, block: BlockStmt) {
        let field_val: u32 = block.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_handler<'a>(self, ast: &mut crate::Ast<'a>, handler: Option<CatchClause>) {
        let field_val: u32 = crate::OptionalNodeId::from(handler.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_finalizer<'a>(self, ast: &mut crate::Ast<'a>, finalizer: Option<BlockStmt>) {
        let field_val: u32 = crate::OptionalNodeId::from(finalizer.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
}
impl WhileStmt {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Stmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl DoWhileStmt {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Stmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl ForStmt {
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<VarDeclOrExpr> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { VarDeclOrExpr::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Expr::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn update<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<VarDeclOrExpr>) {
        let field_val: u32 = crate::OptionalNodeId::from(init.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u32 = crate::OptionalNodeId::from(test.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_update<'a>(self, ast: &mut crate::Ast<'a>, update: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
impl ForInStmt {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForHead {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { ForHead::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Stmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: ForHead) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
}
impl ForOfStmt {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForHead {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { ForHead::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn is_await<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_is_await<'a>(self, ast: &mut crate::Ast<'a>, is_await: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = is_await.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Expr::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn cons<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u32 = crate::OptionalNodeId::from(test.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_cons<'a>(self, ast: &mut crate::Ast<'a>, cons: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { cons.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl CatchClause {
    #[inline]
    pub fn param<'a>(self, ast: &crate::Ast<'a>) -> Option<Pat> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Pat::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { BlockStmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_param<'a>(self, ast: &mut crate::Ast<'a>, param: Option<Pat>) {
        let field_val: u32 = crate::OptionalNodeId::from(param.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl ForHead {
    #[inline]
    pub fn is_var_decl(self) -> bool {
        matches!(self, Self::VarDecl(_))
    }
    #[inline]
    pub fn is_using_decl(self) -> bool {
        matches!(self, Self::UsingDecl(_))
    }
    #[inline]
    pub fn is_pat(self) -> bool {
        matches!(self, Self::Pat(_))
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
    #[inline]
    pub fn as_pat(self) -> Option<Pat> {
        match self {
            Self::Pat(it) => Some(it),
            _ => None,
        }
    }
}
impl VarDeclOrExpr {
    #[inline]
    pub fn is_var_decl(self) -> bool {
        matches!(self, Self::VarDecl(_))
    }
    #[inline]
    pub fn is_expr(self) -> bool {
        matches!(self, Self::Expr(_))
    }
    #[inline]
    pub fn as_var_decl(self) -> Option<VarDecl> {
        match self {
            Self::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self) -> Option<Expr> {
        match self {
            Self::Expr(it) => Some(it),
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Function::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_ident<'a>(self, ast: &mut crate::Ast<'a>, ident: Ident) {
        let field_val: u32 = ident.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl ClassDecl {
    #[inline]
    pub fn ident<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn class<'a>(self, ast: &crate::Ast<'a>) -> Class {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Class::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_ident<'a>(self, ast: &mut crate::Ast<'a>, ident: Ident) {
        let field_val: u32 = ident.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_class<'a>(self, ast: &mut crate::Ast<'a>, class: Class) {
        let field_val: u32 = class.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl VarDecl {
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> VarDeclKind {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        unsafe { std::mem::transmute::<u8, VarDeclKind>(raw as u8) }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 8usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn decls<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<VarDeclarator> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: VarDeclKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294902015u32) | ((field_val & 255u32) << 8usize));
        }
    }
    #[inline]
    pub fn set_decls<'a>(self, ast: &mut crate::Ast<'a>, decls: TypedSubRange<VarDeclarator>) {
        let field_val: u64 = unsafe { decls.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl VarDeclarator {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Pat::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Expr::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: Pat) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<Expr>) {
        let field_val: u32 = crate::OptionalNodeId::from(init.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl UsingDecl {
    #[inline]
    pub fn is_await<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn decls<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<VarDeclarator> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_is_await<'a>(self, ast: &mut crate::Ast<'a>, is_await: bool) {
        let field_val: u32 = is_await as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_decls<'a>(self, ast: &mut crate::Ast<'a>, decls: TypedSubRange<VarDeclarator>) {
        let field_val: u64 = unsafe { decls.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
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
    pub fn is_lit(self) -> bool {
        matches!(self, Self::Lit(_))
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
    pub fn as_lit(self) -> Option<Lit> {
        match self {
            Self::Lit(it) => Some(it),
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
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_elems<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        elems: TypedSubRange<Option<ExprOrSpread>>,
    ) {
        let field_val: u64 = unsafe { elems.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl ObjectLit {
    #[inline]
    pub fn props<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<PropOrSpread> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_props<'a>(self, ast: &mut crate::Ast<'a>, props: TypedSubRange<PropOrSpread>) {
        let field_val: u64 = unsafe { props.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl PropOrSpread {
    #[inline]
    pub fn is_spread(self) -> bool {
        matches!(self, Self::Spread(_))
    }
    #[inline]
    pub fn is_prop(self) -> bool {
        matches!(self, Self::Prop(_))
    }
    #[inline]
    pub fn as_spread(self) -> Option<SpreadElement> {
        match self {
            Self::Spread(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_prop(self) -> Option<Prop> {
        match self {
            Self::Prop(it) => Some(it),
            _ => None,
        }
    }
}
impl SpreadElement {
    #[inline]
    pub fn dot_3_token<'a>(self, ast: &crate::Ast<'a>) -> Span {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_dot3_token<'a>(self, ast: &mut crate::Ast<'a>, dot3_token: Span) {
        let field_val: u64 = unsafe { dot3_token.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
}
impl UnaryExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> UnaryOp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 255u32;
        unsafe { std::mem::transmute::<u8, UnaryOp>(raw as u8) }
    }
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: UnaryOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446742978492891135u64) | (((field_val as u64) & 255u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Expr) {
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl UpdateExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> UpdateOp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 255u32;
        unsafe { std::mem::transmute::<u8, UpdateOp>(raw as u8) }
    }
    #[inline]
    pub fn prefix<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 40usize) as u32)
            & 255u32;
        raw != 0
    }
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> SimpleAssignTarget {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe {
            SimpleAssignTarget::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast)
        }
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: UpdateOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446742978492891135u64) | (((field_val as u64) & 255u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_prefix<'a>(self, ast: &mut crate::Ast<'a>, prefix: bool) {
        let field_val: u32 = prefix as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446463698244468735u64) | (((field_val as u64) & 255u64) << 40usize),
        );
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: SimpleAssignTarget) {
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl BinExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> BinaryOp {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        unsafe { std::mem::transmute::<u8, BinaryOp>(raw as u8) }
    }
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: BinaryOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: Expr) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl FnExpr {
    #[inline]
    pub fn ident<'a>(self, ast: &crate::Ast<'a>) -> Option<Ident> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Function::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_ident<'a>(self, ast: &mut crate::Ast<'a>, ident: Option<Ident>) {
        let field_val: u32 = crate::OptionalNodeId::from(ident.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl ClassExpr {
    #[inline]
    pub fn ident<'a>(self, ast: &crate::Ast<'a>) -> Option<Ident> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn class<'a>(self, ast: &crate::Ast<'a>) -> Class {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Class::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_ident<'a>(self, ast: &mut crate::Ast<'a>, ident: Option<Ident>) {
        let field_val: u32 = crate::OptionalNodeId::from(ident.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_class<'a>(self, ast: &mut crate::Ast<'a>, class: Class) {
        let field_val: u32 = class.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl AssignExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> AssignOp {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        unsafe { std::mem::transmute::<u8, AssignOp>(raw as u8) }
    }
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> AssignTarget {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { AssignTarget::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: AssignOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: AssignTarget) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl MemberExpr {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn prop<'a>(self, ast: &crate::Ast<'a>) -> MemberProp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { MemberProp::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: Expr) {
        let field_val: u32 = obj.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_prop<'a>(self, ast: &mut crate::Ast<'a>, prop: MemberProp) {
        let field_val: u32 = prop.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Super::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn prop<'a>(self, ast: &crate::Ast<'a>) -> SuperProp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { SuperProp::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: Super) {
        let field_val: u32 = obj.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_prop<'a>(self, ast: &mut crate::Ast<'a>, prop: SuperProp) {
        let field_val: u32 = prop.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn cons<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn alt<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_cons<'a>(self, ast: &mut crate::Ast<'a>, cons: Expr) {
        let field_val: u32 = cons.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_alt<'a>(self, ast: &mut crate::Ast<'a>, alt: Expr) {
        let field_val: u32 = alt.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
}
impl ImportExpr {
    #[inline]
    pub fn import_span<'a>(self, ast: &crate::Ast<'a>) -> Span {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_import_span<'a>(self, ast: &mut crate::Ast<'a>, import_span: Span) {
        let field_val: u64 = unsafe { import_span.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: ImportPhase) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
impl CallExpr {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Callee {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Callee::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn args<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExprOrSpread> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Callee) {
        let field_val: u32 = callee.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_args<'a>(self, ast: &mut crate::Ast<'a>, args: TypedSubRange<ExprOrSpread>) {
        let field_val: u64 = unsafe { args.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl NewExpr {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn args<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExprOrSpread> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        let field_val: u32 = callee.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_args<'a>(self, ast: &mut crate::Ast<'a>, args: TypedSubRange<ExprOrSpread>) {
        let field_val: u64 = unsafe { args.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl SeqExpr {
    #[inline]
    pub fn exprs<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_exprs<'a>(self, ast: &mut crate::Ast<'a>, exprs: TypedSubRange<Expr>) {
        let field_val: u64 = unsafe { exprs.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl ArrowExpr {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Pat> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmtOrExpr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn is_async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: TypedSubRange<Pat>) {
        let field_val: u64 = unsafe { params.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmtOrExpr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_is_async<'a>(self, ast: &mut crate::Ast<'a>, is_async: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
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
impl YieldExpr {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Expr::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn delegate<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Option<Expr>) {
        let field_val: u32 = crate::OptionalNodeId::from(arg.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_delegate<'a>(self, ast: &mut crate::Ast<'a>, delegate: bool) {
        let field_val: u32 = delegate as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446742978492891135u64) | (((field_val as u64) & 255u64) << 32usize),
        );
    }
}
impl MetaPropExpr {
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MetaPropKind {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, MetaPropKind>(raw as u8) }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MetaPropKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744073709551360u64) | (((field_val as u64) & 255u64) << 0usize),
        );
    }
}
impl AwaitExpr {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Expr) {
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl Tpl {
    #[inline]
    pub fn exprs<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn quasis<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TplElement> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_exprs<'a>(self, ast: &mut crate::Ast<'a>, exprs: TypedSubRange<Expr>) {
        let field_val: u64 = unsafe { exprs.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_quasis<'a>(self, ast: &mut crate::Ast<'a>, quasis: TypedSubRange<TplElement>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = quasis.to_extra_data(ast);
        };
    }
}
impl TaggedTpl {
    #[inline]
    pub fn tag<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn tpl<'a>(self, ast: &crate::Ast<'a>) -> Tpl {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Tpl::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_tag<'a>(self, ast: &mut crate::Ast<'a>, tag: Expr) {
        let field_val: u32 = tag.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_tpl<'a>(self, ast: &mut crate::Ast<'a>, tpl: Tpl) {
        let field_val: u32 = tpl.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl TplElement {
    #[inline]
    pub fn cooked<'a>(self, ast: &crate::Ast<'a>) -> Option<Wtf8Atom<'a>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn tail<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_cooked<'a>(self, ast: &mut crate::Ast<'a>, cooked: Option<Wtf8Atom<'a>>) {
        let field_val: u64 = unsafe { cooked.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_tail<'a>(self, ast: &mut crate::Ast<'a>, tail: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = tail.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Atom<'a>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data(ast);
        };
    }
}
impl ParenExpr {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl Callee {
    #[inline]
    pub fn is_super(self) -> bool {
        matches!(self, Self::Super(_))
    }
    #[inline]
    pub fn is_expr(self) -> bool {
        matches!(self, Self::Expr(_))
    }
    #[inline]
    pub fn as_super(self) -> Option<Super> {
        match self {
            Self::Super(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self) -> Option<Expr> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl Super {}
impl ExprOrSpread {
    #[inline]
    pub fn spread<'a>(self, ast: &crate::Ast<'a>) -> Option<Span> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_spread<'a>(self, ast: &mut crate::Ast<'a>, spread: Option<Span>) {
        let field_val: u64 = unsafe { spread.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
}
impl BlockStmtOrExpr {
    #[inline]
    pub fn is_block_stmt(self) -> bool {
        matches!(self, Self::BlockStmt(_))
    }
    #[inline]
    pub fn is_expr(self) -> bool {
        matches!(self, Self::Expr(_))
    }
    #[inline]
    pub fn as_block_stmt(self) -> Option<BlockStmt> {
        match self {
            Self::BlockStmt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self) -> Option<Expr> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl AssignTarget {
    #[inline]
    pub fn is_simple(self) -> bool {
        matches!(self, Self::Simple(_))
    }
    #[inline]
    pub fn is_pat(self) -> bool {
        matches!(self, Self::Pat(_))
    }
    #[inline]
    pub fn as_simple(self) -> Option<SimpleAssignTarget> {
        match self {
            Self::Simple(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pat(self) -> Option<AssignTargetPat> {
        match self {
            Self::Pat(it) => Some(it),
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 255u32;
        raw != 0
    }
    #[inline]
    pub fn base<'a>(self, ast: &crate::Ast<'a>) -> OptChainBase {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { OptChainBase::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446742978492891135u64) | (((field_val as u64) & 255u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_base<'a>(self, ast: &mut crate::Ast<'a>, base: OptChainBase) {
        let field_val: u32 = base.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn args<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExprOrSpread> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        let field_val: u32 = callee.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_args<'a>(self, ast: &mut crate::Ast<'a>, args: TypedSubRange<ExprOrSpread>) {
        let field_val: u64 = unsafe { args.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl Invalid {}
impl Function {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Param> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn is_generator<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn is_async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(3usize),
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
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    #[inline]
    pub fn set_is_generator<'a>(self, ast: &mut crate::Ast<'a>, is_generator: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(2usize),
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(3usize),
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
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn pat<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Pat::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let field_val: u64 = unsafe { decorators.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_pat<'a>(self, ast: &mut crate::Ast<'a>, pat: Pat) {
        let field_val: u32 = pat.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
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
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ClassMember> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn is_abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let field_val: u64 = unsafe { decorators.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<ClassMember>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    #[inline]
    pub fn set_is_abstract<'a>(self, ast: &mut crate::Ast<'a>, is_abstract: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = is_abstract.to_extra_data(ast);
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
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let field_val: u64 = unsafe { decorators.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = key.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
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
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = is_static.to_extra_data(ast);
        };
    }
}
impl PrivateProp {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PrivateName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let field_val: u64 = unsafe { decorators.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PrivateName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = key.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
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
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = is_static.to_extra_data(ast);
        };
    }
}
impl ClassMethod {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { PropName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Function::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MethodKind {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodKind>(raw as u8) }
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 8usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MethodKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let field_val: u32 = is_static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294902015u32) | ((field_val & 255u32) << 8usize));
        }
    }
}
impl PrivateMethod {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PrivateName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { PrivateName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Function::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MethodKind {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodKind>(raw as u8) }
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 8usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PrivateName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MethodKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let field_val: u32 = is_static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294902015u32) | ((field_val & 255u32) << 8usize));
        }
    }
}
impl Constructor {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ParamOrTsParamProp> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_params<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        params: TypedSubRange<ParamOrTsParamProp>,
    ) {
        let field_val: u64 = unsafe { params.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = key.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
impl Decorator {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl StaticBlock {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { BlockStmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl Key {
    #[inline]
    pub fn is_private(self) -> bool {
        matches!(self, Self::Private(_))
    }
    #[inline]
    pub fn is_public(self) -> bool {
        matches!(self, Self::Public(_))
    }
    #[inline]
    pub fn as_private(self) -> Option<PrivateName> {
        match self {
            Self::Private(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_public(self) -> Option<PropName> {
        match self {
            Self::Public(it) => Some(it),
            _ => None,
        }
    }
}
impl AutoAccessor {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> Key {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let field_val: u64 = unsafe { decorators.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: Key) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = key.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
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
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = is_static.to_extra_data(ast);
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { PropName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Expr) {
        let field_val: u32 = value.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl AssignProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: Ident) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Expr) {
        let field_val: u32 = value.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl GetterProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { PropName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Function::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl SetterProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { PropName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Function::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl MethodProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { PropName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Function::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_function<'a>(self, ast: &mut crate::Ast<'a>, function: Function) {
        let field_val: u32 = function.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl Pat {
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
    pub fn is_expr(self) -> bool {
        matches!(self, Self::Expr(_))
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
    pub fn as_expr(self) -> Option<Expr> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl ArrayPat {
    #[inline]
    pub fn elems<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Option<Pat>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_elems<'a>(self, ast: &mut crate::Ast<'a>, elems: TypedSubRange<Option<Pat>>) {
        let field_val: u64 = unsafe { elems.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
}
impl ObjectPat {
    #[inline]
    pub fn props<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ObjectPatProp> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_props<'a>(self, ast: &mut crate::Ast<'a>, props: TypedSubRange<ObjectPatProp>) {
        let field_val: u64 = unsafe { props.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 4294967040u32) | ((field_val & 255u32) << 0usize));
        }
    }
}
impl AssignPat {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Pat::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: Pat) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl RestPat {
    #[inline]
    pub fn dot_3_token<'a>(self, ast: &crate::Ast<'a>) -> Span {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        unsafe { Pat::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_dot3_token<'a>(self, ast: &mut crate::Ast<'a>, dot3_token: Span) {
        let field_val: u64 = unsafe { dot3_token.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Pat) {
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { PropName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { Pat::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Pat) {
        let field_val: u32 = value.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl AssignPatProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> BindingIdent {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { BindingIdent::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Expr::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: BindingIdent) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let field_val: u32 = crate::OptionalNodeId::from(value.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl Ident {
    #[inline]
    pub fn sym<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn symbol_id<'a>(self, ast: &crate::Ast<'a>) -> Option<SymbolId> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> 0usize)
            & 4294967295u32;
        (raw != 0).then(|| unsafe {
            <SymbolId as oxc_index::Idx>::from_usize_unchecked(raw.wrapping_sub(1) as usize)
        })
    }
    #[inline]
    pub fn set_sym<'a>(self, ast: &mut crate::Ast<'a>, sym: Atom<'a>) {
        let field_val: u64 = unsafe { sym.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_symbol_id<'a>(self, ast: &mut crate::Ast<'a>, symbol_id: Option<SymbolId>) {
        let field_val: u32 = symbol_id.map_or(0, |id| id.index() as u32 + 1);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = unsafe { payload.trailing_inline_data() };
        unsafe {
            payload
                .set_trailing_inline_data((old & 0u32) | ((field_val & 4294967295u32) << 0usize));
        }
    }
}
impl IdentName {
    #[inline]
    pub fn sym<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_sym<'a>(self, ast: &mut crate::Ast<'a>, sym: Atom<'a>) {
        let field_val: u64 = unsafe { sym.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl PrivateName {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: Atom<'a>) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
}
impl BindingIdent {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: Ident) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
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
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Wtf8Atom<'a>) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Option<Atom<'a>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data(ast);
        };
    }
}
impl Bool {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: bool) {
        let field_val: u32 = value as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744073709551360u64) | (((field_val as u64) & 255u64) << 0usize),
        );
    }
}
impl Null {}
impl Number {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> f64 {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: f64) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Option<Atom<'a>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data(ast);
        };
    }
}
impl BigInt {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Option<Atom<'a>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Atom<'a>) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Option<Atom<'a>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data(ast);
        };
    }
}
impl Regex {
    #[inline]
    pub fn exp<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn flags<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_exp<'a>(self, ast: &mut crate::Ast<'a>, exp: Atom<'a>) {
        let field_val: u64 = unsafe { exp.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_flags<'a>(self, ast: &mut crate::Ast<'a>, flags: Atom<'a>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = flags.to_extra_data(ast);
        };
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { JSXObject::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn prop<'a>(self, ast: &crate::Ast<'a>) -> IdentName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { IdentName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: JSXObject) {
        let field_val: u32 = obj.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_prop<'a>(self, ast: &mut crate::Ast<'a>, prop: IdentName) {
        let field_val: u32 = prop.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl JSXNamespacedName {
    #[inline]
    pub fn ns<'a>(self, ast: &crate::Ast<'a>) -> IdentName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { IdentName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> IdentName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        unsafe { IdentName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_ns<'a>(self, ast: &mut crate::Ast<'a>, ns: IdentName) {
        let field_val: u32 = ns.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: IdentName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
    }
}
impl JSXEmptyExpr {}
impl JSXExprContainer {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> JSXExpr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { JSXExpr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: JSXExpr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl JSXExpr {
    #[inline]
    pub fn is_jsx_empty_expr(self) -> bool {
        matches!(self, Self::JSXEmptyExpr(_))
    }
    #[inline]
    pub fn is_expr(self) -> bool {
        matches!(self, Self::Expr(_))
    }
    #[inline]
    pub fn as_jsx_empty_expr(self) -> Option<JSXEmptyExpr> {
        match self {
            Self::JSXEmptyExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self) -> Option<Expr> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXSpreadChild {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { Expr::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
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
    pub fn attrs<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JSXAttrOrSpread> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JSXElementName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn self_closing<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_attrs<'a>(self, ast: &mut crate::Ast<'a>, attrs: TypedSubRange<JSXAttrOrSpread>) {
        let field_val: u64 = unsafe { attrs.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JSXElementName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = name.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_self_closing<'a>(self, ast: &mut crate::Ast<'a>, self_closing: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = self_closing.to_extra_data(ast);
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe {
            JSXElementName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast)
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JSXElementName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
}
impl JSXAttr {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JSXAttrName {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 0usize) as u32)
            & 4294967295u32;
        unsafe { JSXAttrName::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<JSXAttrValue> {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> 32usize) as u32)
            & 4294967295u32;
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { JSXAttrValue::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JSXAttrName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 18446744069414584320u64) | (((field_val as u64) & 4294967295u64) << 0usize),
        );
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<JSXAttrValue>) {
        let field_val: u32 = crate::OptionalNodeId::from(value.map(|n| n.node_id())).into_raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        let old = payload.inline_data();
        payload.set_inline_data(
            (old & 4294967295u64) | (((field_val as u64) & 4294967295u64) << 32usize),
        );
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
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Atom<'a> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Atom<'a>) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Atom<'a>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data(ast);
        };
    }
}
impl JSXElement {
    #[inline]
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JSXElementChild> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn opening<'a>(self, ast: &crate::Ast<'a>) -> JSXOpeningElement {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn closing<'a>(self, ast: &crate::Ast<'a>) -> Option<JSXClosingElement> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_children<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        children: TypedSubRange<JSXElementChild>,
    ) {
        let field_val: u64 = unsafe { children.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_opening<'a>(self, ast: &mut crate::Ast<'a>, opening: JSXOpeningElement) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = opening.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_closing<'a>(self, ast: &mut crate::Ast<'a>, closing: Option<JSXClosingElement>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = closing.to_extra_data(ast);
        };
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
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JSXElementChild> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn opening<'a>(self, ast: &crate::Ast<'a>) -> JSXOpeningFragment {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn closing<'a>(self, ast: &crate::Ast<'a>) -> JSXClosingFragment {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
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
    pub fn set_children<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        children: TypedSubRange<JSXElementChild>,
    ) {
        let field_val: u64 = unsafe { children.to_extra_data(ast).other };
        unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
    }
    #[inline]
    pub fn set_opening<'a>(self, ast: &mut crate::Ast<'a>, opening: JSXOpeningFragment) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = opening.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_closing<'a>(self, ast: &mut crate::Ast<'a>, closing: JSXClosingFragment) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .data_unchecked(self.0)
                    .extra_data_start()
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = closing.to_extra_data(ast);
        };
    }
}
impl JSXOpeningFragment {}
impl JSXClosingFragment {}
