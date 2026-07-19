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
    pub fn is_module(self, ast: &Ast<'_>) -> bool {
        matches!(ast.program_data(self), ProgramData::Module(_))
    }
    #[inline]
    pub fn is_script(self, ast: &Ast<'_>) -> bool {
        matches!(ast.program_data(self), ProgramData::Script(_))
    }
    #[inline]
    pub fn as_module(self, ast: &Ast<'_>) -> Option<Module> {
        match ast.program_data(self) {
            ProgramData::Module(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_script(self, ast: &Ast<'_>) -> Option<Script> {
        match ast.program_data(self) {
            ProgramData::Script(it) => Some(it),
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
    pub fn is_module_decl(self, ast: &Ast<'_>) -> bool {
        matches!(ast.module_item_data(self), ModuleItemData::ModuleDecl(_))
    }
    #[inline]
    pub fn is_stmt(self, ast: &Ast<'_>) -> bool {
        matches!(ast.module_item_data(self), ModuleItemData::Stmt(_))
    }
    #[inline]
    pub fn as_module_decl(self, ast: &Ast<'_>) -> Option<ModuleDecl> {
        match ast.module_item_data(self) {
            ModuleItemData::ModuleDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt(self, ast: &Ast<'_>) -> Option<Stmt> {
        match ast.module_item_data(self) {
            ModuleItemData::Stmt(it) => Some(it),
            _ => None,
        }
    }
}
impl ModuleDecl {
    #[inline]
    pub fn is_import(self, ast: &Ast<'_>) -> bool {
        matches!(ast.module_decl_data(self), ModuleDeclData::Import(_))
    }
    #[inline]
    pub fn is_export_decl(self, ast: &Ast<'_>) -> bool {
        matches!(ast.module_decl_data(self), ModuleDeclData::ExportDecl(_))
    }
    #[inline]
    pub fn is_export_named(self, ast: &Ast<'_>) -> bool {
        matches!(ast.module_decl_data(self), ModuleDeclData::ExportNamed(_))
    }
    #[inline]
    pub fn is_export_default_decl(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.module_decl_data(self),
            ModuleDeclData::ExportDefaultDecl(_)
        )
    }
    #[inline]
    pub fn is_export_default_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.module_decl_data(self),
            ModuleDeclData::ExportDefaultExpr(_)
        )
    }
    #[inline]
    pub fn is_export_all(self, ast: &Ast<'_>) -> bool {
        matches!(ast.module_decl_data(self), ModuleDeclData::ExportAll(_))
    }
    #[inline]
    pub fn as_import(self, ast: &Ast<'_>) -> Option<ImportDecl> {
        match ast.module_decl_data(self) {
            ModuleDeclData::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_decl(self, ast: &Ast<'_>) -> Option<ExportDecl> {
        match ast.module_decl_data(self) {
            ModuleDeclData::ExportDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_named(self, ast: &Ast<'_>) -> Option<NamedExport> {
        match ast.module_decl_data(self) {
            ModuleDeclData::ExportNamed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_default_decl(self, ast: &Ast<'_>) -> Option<ExportDefaultDecl> {
        match ast.module_decl_data(self) {
            ModuleDeclData::ExportDefaultDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_default_expr(self, ast: &Ast<'_>) -> Option<ExportDefaultExpr> {
        match ast.module_decl_data(self) {
            ModuleDeclData::ExportDefaultExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_all(self, ast: &Ast<'_>) -> Option<ExportAll> {
        match ast.module_decl_data(self) {
            ModuleDeclData::ExportAll(it) => Some(it),
            _ => None,
        }
    }
}
impl ImportDecl {
    #[inline]
    pub fn src<'a>(self, ast: &crate::Ast<'a>) -> Str {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 4294967295u32;
        Str(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn with<'a>(self, ast: &crate::Ast<'a>) -> Option<ObjectLit> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 48usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| ObjectLit(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> ImportPhase {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ImportPhase>(raw as u8) }
    }
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ImportSpecifier> {
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
    pub fn set_src<'a>(self, ast: &mut crate::Ast<'a>, src: Str) {
        let field_val: u32 = src.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374325956791566335u128)
            | (((field_val as u128) & 4294967295u128) << 16usize);
    }
    #[inline]
    pub fn set_type_only<'a>(self, ast: &mut crate::Ast<'a>, type_only: bool) {
        let field_val: u32 = type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_with<'a>(self, ast: &mut crate::Ast<'a>, with: Option<ObjectLit>) {
        let field_val: u32 = with.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937254537555274277570215935u128)
            | (((field_val as u128) & 4294967295u128) << 48usize);
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: ImportPhase) {
        let field_val: u32 = phase as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        specifiers: TypedSubRange<ImportSpecifier>,
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
                .get_unchecked_mut(offset.index()) = specifiers.to_extra_data(ast);
        };
    }
}
impl ImportSpecifier {
    #[inline]
    pub fn is_named(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.import_specifier_data(self),
            ImportSpecifierData::Named(_)
        )
    }
    #[inline]
    pub fn is_default(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.import_specifier_data(self),
            ImportSpecifierData::Default(_)
        )
    }
    #[inline]
    pub fn is_namespace(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.import_specifier_data(self),
            ImportSpecifierData::Namespace(_)
        )
    }
    #[inline]
    pub fn as_named(self, ast: &Ast<'_>) -> Option<ImportNamedSpecifier> {
        match ast.import_specifier_data(self) {
            ImportSpecifierData::Named(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_default(self, ast: &Ast<'_>) -> Option<ImportDefaultSpecifier> {
        match ast.import_specifier_data(self) {
            ImportSpecifierData::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_namespace(self, ast: &Ast<'_>) -> Option<ImportStarAsSpecifier> {
        match ast.import_specifier_data(self) {
            ImportSpecifierData::Namespace(it) => Some(it),
            _ => None,
        }
    }
}
impl ImportNamedSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn imported<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        ModuleExportName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn is_type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: Ident) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_imported<'a>(self, ast: &mut crate::Ast<'a>, imported: ModuleExportName) {
        let field_val: u32 = imported.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_is_type_only<'a>(self, ast: &mut crate::Ast<'a>, is_type_only: bool) {
        let field_val: u32 = is_type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Decl(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_decl<'a>(self, ast: &mut crate::Ast<'a>, decl: Decl) {
        let field_val: u32 = decl.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl NamedExport {
    #[inline]
    pub fn src<'a>(self, ast: &crate::Ast<'a>) -> Option<Str> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Str(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn with<'a>(self, ast: &crate::Ast<'a>) -> Option<ObjectLit> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| ObjectLit(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExportSpecifier> {
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
    pub fn set_src<'a>(self, ast: &mut crate::Ast<'a>, src: Option<Str>) {
        let field_val: u32 = src.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_type_only<'a>(self, ast: &mut crate::Ast<'a>, type_only: bool) {
        let field_val: u32 = type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_with<'a>(self, ast: &mut crate::Ast<'a>, with: Option<ObjectLit>) {
        let field_val: u32 = with.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        specifiers: TypedSubRange<ExportSpecifier>,
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
                .get_unchecked_mut(offset.index()) = specifiers.to_extra_data(ast);
        };
    }
}
impl ExportSpecifier {
    #[inline]
    pub fn is_namespace(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.export_specifier_data(self),
            ExportSpecifierData::Namespace(_)
        )
    }
    #[inline]
    pub fn is_default(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.export_specifier_data(self),
            ExportSpecifierData::Default(_)
        )
    }
    #[inline]
    pub fn is_named(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.export_specifier_data(self),
            ExportSpecifierData::Named(_)
        )
    }
    #[inline]
    pub fn as_namespace(self, ast: &Ast<'_>) -> Option<ExportNamespaceSpecifier> {
        match ast.export_specifier_data(self) {
            ExportSpecifierData::Namespace(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_default(self, ast: &Ast<'_>) -> Option<ExportDefaultSpecifier> {
        match ast.export_specifier_data(self) {
            ExportSpecifierData::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_named(self, ast: &Ast<'_>) -> Option<ExportNamedSpecifier> {
        match ast.export_specifier_data(self) {
            ExportSpecifierData::Named(it) => Some(it),
            _ => None,
        }
    }
}
impl ExportNamespaceSpecifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ModuleExportName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: ModuleExportName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ModuleExportName {
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.module_export_name_data(self),
            ModuleExportNameData::Ident(_)
        )
    }
    #[inline]
    pub fn is_str(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.module_export_name_data(self),
            ModuleExportNameData::Str(_)
        )
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<Ident> {
        match ast.module_export_name_data(self) {
            ModuleExportNameData::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self, ast: &Ast<'_>) -> Option<Str> {
        match ast.module_export_name_data(self) {
            ModuleExportNameData::Str(it) => Some(it),
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ModuleExportName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        ModuleExportName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn is_type_only<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_orig<'a>(self, ast: &mut crate::Ast<'a>, orig: ModuleExportName) {
        let field_val: u32 = orig.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_exported<'a>(self, ast: &mut crate::Ast<'a>, exported: ModuleExportName) {
        let field_val: u32 = exported.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_is_type_only<'a>(self, ast: &mut crate::Ast<'a>, is_type_only: bool) {
        let field_val: u32 = is_type_only as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl ExportDefaultDecl {
    #[inline]
    pub fn decl<'a>(self, ast: &crate::Ast<'a>) -> DefaultDecl {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        DefaultDecl(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_decl<'a>(self, ast: &mut crate::Ast<'a>, decl: DefaultDecl) {
        let field_val: u32 = decl.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl DefaultDecl {
    #[inline]
    pub fn is_class(self, ast: &Ast<'_>) -> bool {
        matches!(ast.default_decl_data(self), DefaultDeclData::Class(_))
    }
    #[inline]
    pub fn is_fn(self, ast: &Ast<'_>) -> bool {
        matches!(ast.default_decl_data(self), DefaultDeclData::Fn(_))
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_>) -> Option<ClassExpr> {
        match ast.default_decl_data(self) {
            DefaultDeclData::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self, ast: &Ast<'_>) -> Option<FnExpr> {
        match ast.default_decl_data(self) {
            DefaultDeclData::Fn(it) => Some(it),
            _ => None,
        }
    }
}
impl ExportDefaultExpr {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
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
    pub fn is_block(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Block(_))
    }
    #[inline]
    pub fn is_empty(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Empty(_))
    }
    #[inline]
    pub fn is_debugger(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Debugger(_))
    }
    #[inline]
    pub fn is_with(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::With(_))
    }
    #[inline]
    pub fn is_return(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Return(_))
    }
    #[inline]
    pub fn is_labeled(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Labeled(_))
    }
    #[inline]
    pub fn is_break(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Break(_))
    }
    #[inline]
    pub fn is_continue(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Continue(_))
    }
    #[inline]
    pub fn is_if(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::If(_))
    }
    #[inline]
    pub fn is_switch(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Switch(_))
    }
    #[inline]
    pub fn is_throw(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Throw(_))
    }
    #[inline]
    pub fn is_try(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Try(_))
    }
    #[inline]
    pub fn is_while(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::While(_))
    }
    #[inline]
    pub fn is_do_while(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::DoWhile(_))
    }
    #[inline]
    pub fn is_for(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::For(_))
    }
    #[inline]
    pub fn is_for_in(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::ForIn(_))
    }
    #[inline]
    pub fn is_for_of(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::ForOf(_))
    }
    #[inline]
    pub fn is_decl(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Decl(_))
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), StmtData::Expr(_))
    }
    #[inline]
    pub fn as_block(self, ast: &Ast<'_>) -> Option<BlockStmt> {
        match ast.stmt_data(self) {
            StmtData::Block(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_empty(self, ast: &Ast<'_>) -> Option<EmptyStmt> {
        match ast.stmt_data(self) {
            StmtData::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_debugger(self, ast: &Ast<'_>) -> Option<DebuggerStmt> {
        match ast.stmt_data(self) {
            StmtData::Debugger(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_with(self, ast: &Ast<'_>) -> Option<WithStmt> {
        match ast.stmt_data(self) {
            StmtData::With(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_return(self, ast: &Ast<'_>) -> Option<ReturnStmt> {
        match ast.stmt_data(self) {
            StmtData::Return(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_labeled(self, ast: &Ast<'_>) -> Option<LabeledStmt> {
        match ast.stmt_data(self) {
            StmtData::Labeled(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_break(self, ast: &Ast<'_>) -> Option<BreakStmt> {
        match ast.stmt_data(self) {
            StmtData::Break(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_continue(self, ast: &Ast<'_>) -> Option<ContinueStmt> {
        match ast.stmt_data(self) {
            StmtData::Continue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_if(self, ast: &Ast<'_>) -> Option<IfStmt> {
        match ast.stmt_data(self) {
            StmtData::If(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_switch(self, ast: &Ast<'_>) -> Option<SwitchStmt> {
        match ast.stmt_data(self) {
            StmtData::Switch(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_throw(self, ast: &Ast<'_>) -> Option<ThrowStmt> {
        match ast.stmt_data(self) {
            StmtData::Throw(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_try(self, ast: &Ast<'_>) -> Option<TryStmt> {
        match ast.stmt_data(self) {
            StmtData::Try(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_while(self, ast: &Ast<'_>) -> Option<WhileStmt> {
        match ast.stmt_data(self) {
            StmtData::While(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_do_while(self, ast: &Ast<'_>) -> Option<DoWhileStmt> {
        match ast.stmt_data(self) {
            StmtData::DoWhile(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for(self, ast: &Ast<'_>) -> Option<ForStmt> {
        match ast.stmt_data(self) {
            StmtData::For(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_in(self, ast: &Ast<'_>) -> Option<ForInStmt> {
        match ast.stmt_data(self) {
            StmtData::ForIn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_of(self, ast: &Ast<'_>) -> Option<ForOfStmt> {
        match ast.stmt_data(self) {
            StmtData::ForOf(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_decl(self, ast: &Ast<'_>) -> Option<Decl> {
        match ast.stmt_data(self) {
            StmtData::Decl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<ExprStmt> {
        match ast.stmt_data(self) {
            StmtData::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl ExprStmt {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl EmptyStmt {}
impl DebuggerStmt {}
impl WithStmt {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: Expr) {
        let field_val: u32 = obj.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl ReturnStmt {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Option<Expr>) {
        let field_val: u32 = arg.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl LabeledStmt {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Ident) {
        let field_val: u32 = label.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn cons<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn alt<'a>(self, ast: &crate::Ast<'a>) -> Option<Stmt> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Stmt(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_cons<'a>(self, ast: &mut crate::Ast<'a>, cons: Stmt) {
        let field_val: u32 = cons.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_alt<'a>(self, ast: &mut crate::Ast<'a>, alt: Option<Stmt>) {
        let field_val: u32 = alt.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl SwitchStmt {
    #[inline]
    pub fn discriminant<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn cases<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<SwitchCase> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_discriminant<'a>(self, ast: &mut crate::Ast<'a>, discriminant: Expr) {
        let field_val: u32 = discriminant.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_cases<'a>(self, ast: &mut crate::Ast<'a>, cases: TypedSubRange<SwitchCase>) {
        let field_val: u64 = unsafe { cases.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl ThrowStmt {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Expr) {
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl DoWhileStmt {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl ForStmt {
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<VarDeclOrExpr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| VarDeclOrExpr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn update<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<VarDeclOrExpr>) {
        let field_val: u32 = init.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u32 = test.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_update<'a>(self, ast: &mut crate::Ast<'a>, update: Option<Expr>) {
        let field_val: u32 = update.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl ForInStmt {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForHead {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ForHead(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: ForHead) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl ForOfStmt {
    #[inline]
    pub fn is_await<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForHead {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ForHead(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_is_await<'a>(self, ast: &mut crate::Ast<'a>, is_await: bool) {
        let field_val: u32 = is_await as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: ForHead) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl SwitchCase {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn cons<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u32 = test.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_cons<'a>(self, ast: &mut crate::Ast<'a>, cons: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { cons.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl CatchClause {
    #[inline]
    pub fn param<'a>(self, ast: &crate::Ast<'a>) -> Option<Pat> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Pat(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        BlockStmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_param<'a>(self, ast: &mut crate::Ast<'a>, param: Option<Pat>) {
        let field_val: u32 = param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl ForHead {
    #[inline]
    pub fn is_var_decl(self, ast: &Ast<'_>) -> bool {
        matches!(ast.for_head_data(self), ForHeadData::VarDecl(_))
    }
    #[inline]
    pub fn is_using_decl(self, ast: &Ast<'_>) -> bool {
        matches!(ast.for_head_data(self), ForHeadData::UsingDecl(_))
    }
    #[inline]
    pub fn is_pat(self, ast: &Ast<'_>) -> bool {
        matches!(ast.for_head_data(self), ForHeadData::Pat(_))
    }
    #[inline]
    pub fn as_var_decl(self, ast: &Ast<'_>) -> Option<VarDecl> {
        match ast.for_head_data(self) {
            ForHeadData::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_using_decl(self, ast: &Ast<'_>) -> Option<UsingDecl> {
        match ast.for_head_data(self) {
            ForHeadData::UsingDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pat(self, ast: &Ast<'_>) -> Option<Pat> {
        match ast.for_head_data(self) {
            ForHeadData::Pat(it) => Some(it),
            _ => None,
        }
    }
}
impl VarDeclOrExpr {
    #[inline]
    pub fn is_var_decl(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.var_decl_or_expr_data(self),
            VarDeclOrExprData::VarDecl(_)
        )
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(ast.var_decl_or_expr_data(self), VarDeclOrExprData::Expr(_))
    }
    #[inline]
    pub fn as_var_decl(self, ast: &Ast<'_>) -> Option<VarDecl> {
        match ast.var_decl_or_expr_data(self) {
            VarDeclOrExprData::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.var_decl_or_expr_data(self) {
            VarDeclOrExprData::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl Decl {
    #[inline]
    pub fn is_class(self, ast: &Ast<'_>) -> bool {
        matches!(ast.decl_data(self), DeclData::Class(_))
    }
    #[inline]
    pub fn is_fn(self, ast: &Ast<'_>) -> bool {
        matches!(ast.decl_data(self), DeclData::Fn(_))
    }
    #[inline]
    pub fn is_var(self, ast: &Ast<'_>) -> bool {
        matches!(ast.decl_data(self), DeclData::Var(_))
    }
    #[inline]
    pub fn is_using(self, ast: &Ast<'_>) -> bool {
        matches!(ast.decl_data(self), DeclData::Using(_))
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_>) -> Option<ClassDecl> {
        match ast.decl_data(self) {
            DeclData::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self, ast: &Ast<'_>) -> Option<FnDecl> {
        match ast.decl_data(self) {
            DeclData::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_var(self, ast: &Ast<'_>) -> Option<VarDecl> {
        match ast.decl_data(self) {
            DeclData::Var(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_using(self, ast: &Ast<'_>) -> Option<UsingDecl> {
        match ast.decl_data(self) {
            DeclData::Using(it) => Some(it),
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Pat(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: Pat) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<Expr>) {
        let field_val: u32 = init.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
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
    pub fn is_this(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::This(_))
    }
    #[inline]
    pub fn is_array(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Array(_))
    }
    #[inline]
    pub fn is_object(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Object(_))
    }
    #[inline]
    pub fn is_fn(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Fn(_))
    }
    #[inline]
    pub fn is_unary(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Unary(_))
    }
    #[inline]
    pub fn is_update(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Update(_))
    }
    #[inline]
    pub fn is_bin(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Bin(_))
    }
    #[inline]
    pub fn is_assign(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Assign(_))
    }
    #[inline]
    pub fn is_member(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Member(_))
    }
    #[inline]
    pub fn is_super_prop(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::SuperProp(_))
    }
    #[inline]
    pub fn is_cond(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Cond(_))
    }
    #[inline]
    pub fn is_import(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Import(_))
    }
    #[inline]
    pub fn is_call(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Call(_))
    }
    #[inline]
    pub fn is_new(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::New(_))
    }
    #[inline]
    pub fn is_seq(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Seq(_))
    }
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Ident(_))
    }
    #[inline]
    pub fn is_lit(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Lit(_))
    }
    #[inline]
    pub fn is_tpl(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Tpl(_))
    }
    #[inline]
    pub fn is_tagged_tpl(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::TaggedTpl(_))
    }
    #[inline]
    pub fn is_arrow(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Arrow(_))
    }
    #[inline]
    pub fn is_class(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Class(_))
    }
    #[inline]
    pub fn is_yield(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Yield(_))
    }
    #[inline]
    pub fn is_meta_prop(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::MetaProp(_))
    }
    #[inline]
    pub fn is_await(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Await(_))
    }
    #[inline]
    pub fn is_paren(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Paren(_))
    }
    #[inline]
    pub fn is_jsx_member(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::JSXMember(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::JSXNamespacedName(_))
    }
    #[inline]
    pub fn is_jsx_empty(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::JSXEmpty(_))
    }
    #[inline]
    pub fn is_jsx_element(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::JSXFragment(_))
    }
    #[inline]
    pub fn is_private_name(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::PrivateName(_))
    }
    #[inline]
    pub fn is_opt_chain(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::OptChain(_))
    }
    #[inline]
    pub fn is_invalid(self, ast: &Ast<'_>) -> bool {
        matches!(ast.expr_data(self), ExprData::Invalid(_))
    }
    #[inline]
    pub fn as_this(self, ast: &Ast<'_>) -> Option<ThisExpr> {
        match ast.expr_data(self) {
            ExprData::This(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(self, ast: &Ast<'_>) -> Option<ArrayLit> {
        match ast.expr_data(self) {
            ExprData::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self, ast: &Ast<'_>) -> Option<ObjectLit> {
        match ast.expr_data(self) {
            ExprData::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(self, ast: &Ast<'_>) -> Option<FnExpr> {
        match ast.expr_data(self) {
            ExprData::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary(self, ast: &Ast<'_>) -> Option<UnaryExpr> {
        match ast.expr_data(self) {
            ExprData::Unary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_update(self, ast: &Ast<'_>) -> Option<UpdateExpr> {
        match ast.expr_data(self) {
            ExprData::Update(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bin(self, ast: &Ast<'_>) -> Option<BinExpr> {
        match ast.expr_data(self) {
            ExprData::Bin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self, ast: &Ast<'_>) -> Option<AssignExpr> {
        match ast.expr_data(self) {
            ExprData::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(self, ast: &Ast<'_>) -> Option<MemberExpr> {
        match ast.expr_data(self) {
            ExprData::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(self, ast: &Ast<'_>) -> Option<SuperPropExpr> {
        match ast.expr_data(self) {
            ExprData::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_cond(self, ast: &Ast<'_>) -> Option<CondExpr> {
        match ast.expr_data(self) {
            ExprData::Cond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import(self, ast: &Ast<'_>) -> Option<ImportExpr> {
        match ast.expr_data(self) {
            ExprData::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(self, ast: &Ast<'_>) -> Option<CallExpr> {
        match ast.expr_data(self) {
            ExprData::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_new(self, ast: &Ast<'_>) -> Option<NewExpr> {
        match ast.expr_data(self) {
            ExprData::New(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_seq(self, ast: &Ast<'_>) -> Option<SeqExpr> {
        match ast.expr_data(self) {
            ExprData::Seq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<Ident> {
        match ast.expr_data(self) {
            ExprData::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_lit(self, ast: &Ast<'_>) -> Option<Lit> {
        match ast.expr_data(self) {
            ExprData::Lit(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tpl(self, ast: &Ast<'_>) -> Option<Tpl> {
        match ast.expr_data(self) {
            ExprData::Tpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tagged_tpl(self, ast: &Ast<'_>) -> Option<TaggedTpl> {
        match ast.expr_data(self) {
            ExprData::TaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_arrow(self, ast: &Ast<'_>) -> Option<ArrowExpr> {
        match ast.expr_data(self) {
            ExprData::Arrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_>) -> Option<ClassExpr> {
        match ast.expr_data(self) {
            ExprData::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_yield(self, ast: &Ast<'_>) -> Option<YieldExpr> {
        match ast.expr_data(self) {
            ExprData::Yield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_meta_prop(self, ast: &Ast<'_>) -> Option<MetaPropExpr> {
        match ast.expr_data(self) {
            ExprData::MetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_await(self, ast: &Ast<'_>) -> Option<AwaitExpr> {
        match ast.expr_data(self) {
            ExprData::Await(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(self, ast: &Ast<'_>) -> Option<ParenExpr> {
        match ast.expr_data(self) {
            ExprData::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member(self, ast: &Ast<'_>) -> Option<JSXMemberExpr> {
        match ast.expr_data(self) {
            ExprData::JSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self, ast: &Ast<'_>) -> Option<JSXNamespacedName> {
        match ast.expr_data(self) {
            ExprData::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty(self, ast: &Ast<'_>) -> Option<JSXEmptyExpr> {
        match ast.expr_data(self) {
            ExprData::JSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_>) -> Option<JSXElement> {
        match ast.expr_data(self) {
            ExprData::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_>) -> Option<JSXFragment> {
        match ast.expr_data(self) {
            ExprData::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(self, ast: &Ast<'_>) -> Option<PrivateName> {
        match ast.expr_data(self) {
            ExprData::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(self, ast: &Ast<'_>) -> Option<OptChainExpr> {
        match ast.expr_data(self) {
            ExprData::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self, ast: &Ast<'_>) -> Option<Invalid> {
        match ast.expr_data(self) {
            ExprData::Invalid(it) => Some(it),
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
    pub fn is_spread(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_or_spread_data(self), PropOrSpreadData::Spread(_))
    }
    #[inline]
    pub fn is_prop(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_or_spread_data(self), PropOrSpreadData::Prop(_))
    }
    #[inline]
    pub fn as_spread(self, ast: &Ast<'_>) -> Option<SpreadElement> {
        match ast.prop_or_spread_data(self) {
            PropOrSpreadData::Spread(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_prop(self, ast: &Ast<'_>) -> Option<Prop> {
        match ast.prop_or_spread_data(self) {
            PropOrSpreadData::Prop(it) => Some(it),
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
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
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl UnaryExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> UnaryOp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, UnaryOp>(raw as u8) }
    }
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: UnaryOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Expr) {
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl UpdateExpr {
    #[inline]
    pub fn op<'a>(self, ast: &crate::Ast<'a>) -> UpdateOp {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, UpdateOp>(raw as u8) }
    }
    #[inline]
    pub fn prefix<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> SimpleAssignTarget {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        SimpleAssignTarget(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_op<'a>(self, ast: &mut crate::Ast<'a>, op: UpdateOp) {
        let field_val: u32 = op as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
    #[inline]
    pub fn set_prefix<'a>(self, ast: &mut crate::Ast<'a>, prefix: bool) {
        let field_val: u32 = prefix as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374327056303128575u128)
            | (((field_val as u128) & 255u128) << 40usize);
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: SimpleAssignTarget) {
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
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
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        AssignTarget(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
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
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl MemberExpr {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn prop<'a>(self, ast: &crate::Ast<'a>) -> MemberProp {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        MemberProp(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: Expr) {
        let field_val: u32 = obj.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_prop<'a>(self, ast: &mut crate::Ast<'a>, prop: MemberProp) {
        let field_val: u32 = prop.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl MemberProp {
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(ast.member_prop_data(self), MemberPropData::Ident(_))
    }
    #[inline]
    pub fn is_private_name(self, ast: &Ast<'_>) -> bool {
        matches!(ast.member_prop_data(self), MemberPropData::PrivateName(_))
    }
    #[inline]
    pub fn is_computed(self, ast: &Ast<'_>) -> bool {
        matches!(ast.member_prop_data(self), MemberPropData::Computed(_))
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<IdentName> {
        match ast.member_prop_data(self) {
            MemberPropData::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(self, ast: &Ast<'_>) -> Option<PrivateName> {
        match ast.member_prop_data(self) {
            MemberPropData::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(self, ast: &Ast<'_>) -> Option<ComputedPropName> {
        match ast.member_prop_data(self) {
            MemberPropData::Computed(it) => Some(it),
            _ => None,
        }
    }
}
impl SuperPropExpr {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> Super {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Super(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn prop<'a>(self, ast: &crate::Ast<'a>) -> SuperProp {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        SuperProp(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: Super) {
        let field_val: u32 = obj.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_prop<'a>(self, ast: &mut crate::Ast<'a>, prop: SuperProp) {
        let field_val: u32 = prop.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl SuperProp {
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(ast.super_prop_data(self), SuperPropData::Ident(_))
    }
    #[inline]
    pub fn is_computed(self, ast: &Ast<'_>) -> bool {
        matches!(ast.super_prop_data(self), SuperPropData::Computed(_))
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<IdentName> {
        match ast.super_prop_data(self) {
            SuperPropData::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(self, ast: &Ast<'_>) -> Option<ComputedPropName> {
        match ast.super_prop_data(self) {
            SuperPropData::Computed(it) => Some(it),
            _ => None,
        }
    }
}
impl CondExpr {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn cons<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn alt<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_cons<'a>(self, ast: &mut crate::Ast<'a>, cons: Expr) {
        let field_val: u32 = cons.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_alt<'a>(self, ast: &mut crate::Ast<'a>, alt: Expr) {
        let field_val: u32 = alt.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl ImportExpr {
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn options<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> ImportPhase {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ImportPhase>(raw as u8) }
    }
    #[inline]
    pub fn import_span<'a>(self, ast: &crate::Ast<'a>) -> Span {
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
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: Expr) {
        let field_val: u32 = source.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_options<'a>(self, ast: &mut crate::Ast<'a>, options: Option<Expr>) {
        let field_val: u32 = options.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: ImportPhase) {
        let field_val: u32 = phase as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_import_span<'a>(self, ast: &mut crate::Ast<'a>, import_span: Span) {
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
                .get_unchecked_mut(offset.index()) = import_span.to_extra_data(ast);
        };
    }
}
impl CallExpr {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Callee {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Callee(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn args<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExprOrSpread> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Callee) {
        let field_val: u32 = callee.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_args<'a>(self, ast: &mut crate::Ast<'a>, args: TypedSubRange<ExprOrSpread>) {
        let field_val: u64 = unsafe { args.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl NewExpr {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn args<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExprOrSpread> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        let field_val: u32 = callee.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_args<'a>(self, ast: &mut crate::Ast<'a>, args: TypedSubRange<ExprOrSpread>) {
        let field_val: u64 = unsafe { args.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
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
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmtOrExpr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        BlockStmtOrExpr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn is_async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: TypedSubRange<Pat>) {
        let field_val: u64 = unsafe { params.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmtOrExpr) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_is_async<'a>(self, ast: &mut crate::Ast<'a>, is_async: bool) {
        let field_val: u32 = is_async as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
}
impl YieldExpr {
    #[inline]
    pub fn arg<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn delegate<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Option<Expr>) {
        let field_val: u32 = arg.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_delegate<'a>(self, ast: &mut crate::Ast<'a>, delegate: bool) {
        let field_val: u32 = delegate as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_arg<'a>(self, ast: &mut crate::Ast<'a>, arg: Expr) {
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn tpl<'a>(self, ast: &crate::Ast<'a>) -> Tpl {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Tpl(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_tag<'a>(self, ast: &mut crate::Ast<'a>, tag: Expr) {
        let field_val: u32 = tag.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_tpl<'a>(self, ast: &mut crate::Ast<'a>, tpl: Tpl) {
        let field_val: u32 = tpl.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl Callee {
    #[inline]
    pub fn is_super(self, ast: &Ast<'_>) -> bool {
        matches!(ast.callee_data(self), CalleeData::Super(_))
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(ast.callee_data(self), CalleeData::Expr(_))
    }
    #[inline]
    pub fn as_super(self, ast: &Ast<'_>) -> Option<Super> {
        match ast.callee_data(self) {
            CalleeData::Super(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.callee_data(self) {
            CalleeData::Expr(it) => Some(it),
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
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
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl BlockStmtOrExpr {
    #[inline]
    pub fn is_block_stmt(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.block_stmt_or_expr_data(self),
            BlockStmtOrExprData::BlockStmt(_)
        )
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.block_stmt_or_expr_data(self),
            BlockStmtOrExprData::Expr(_)
        )
    }
    #[inline]
    pub fn as_block_stmt(self, ast: &Ast<'_>) -> Option<BlockStmt> {
        match ast.block_stmt_or_expr_data(self) {
            BlockStmtOrExprData::BlockStmt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.block_stmt_or_expr_data(self) {
            BlockStmtOrExprData::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl AssignTarget {
    #[inline]
    pub fn is_simple(self, ast: &Ast<'_>) -> bool {
        matches!(ast.assign_target_data(self), AssignTargetData::Simple(_))
    }
    #[inline]
    pub fn is_pat(self, ast: &Ast<'_>) -> bool {
        matches!(ast.assign_target_data(self), AssignTargetData::Pat(_))
    }
    #[inline]
    pub fn as_simple(self, ast: &Ast<'_>) -> Option<SimpleAssignTarget> {
        match ast.assign_target_data(self) {
            AssignTargetData::Simple(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pat(self, ast: &Ast<'_>) -> Option<AssignTargetPat> {
        match ast.assign_target_data(self) {
            AssignTargetData::Pat(it) => Some(it),
            _ => None,
        }
    }
}
impl AssignTargetPat {
    #[inline]
    pub fn is_array(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.assign_target_pat_data(self),
            AssignTargetPatData::Array(_)
        )
    }
    #[inline]
    pub fn is_object(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.assign_target_pat_data(self),
            AssignTargetPatData::Object(_)
        )
    }
    #[inline]
    pub fn is_invalid(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.assign_target_pat_data(self),
            AssignTargetPatData::Invalid(_)
        )
    }
    #[inline]
    pub fn as_array(self, ast: &Ast<'_>) -> Option<ArrayPat> {
        match ast.assign_target_pat_data(self) {
            AssignTargetPatData::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self, ast: &Ast<'_>) -> Option<ObjectPat> {
        match ast.assign_target_pat_data(self) {
            AssignTargetPatData::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self, ast: &Ast<'_>) -> Option<Invalid> {
        match ast.assign_target_pat_data(self) {
            AssignTargetPatData::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl SimpleAssignTarget {
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.simple_assign_target_data(self),
            SimpleAssignTargetData::Ident(_)
        )
    }
    #[inline]
    pub fn is_member(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.simple_assign_target_data(self),
            SimpleAssignTargetData::Member(_)
        )
    }
    #[inline]
    pub fn is_super_prop(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.simple_assign_target_data(self),
            SimpleAssignTargetData::SuperProp(_)
        )
    }
    #[inline]
    pub fn is_paren(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.simple_assign_target_data(self),
            SimpleAssignTargetData::Paren(_)
        )
    }
    #[inline]
    pub fn is_opt_chain(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.simple_assign_target_data(self),
            SimpleAssignTargetData::OptChain(_)
        )
    }
    #[inline]
    pub fn is_invalid(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.simple_assign_target_data(self),
            SimpleAssignTargetData::Invalid(_)
        )
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<BindingIdent> {
        match ast.simple_assign_target_data(self) {
            SimpleAssignTargetData::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(self, ast: &Ast<'_>) -> Option<MemberExpr> {
        match ast.simple_assign_target_data(self) {
            SimpleAssignTargetData::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(self, ast: &Ast<'_>) -> Option<SuperPropExpr> {
        match ast.simple_assign_target_data(self) {
            SimpleAssignTargetData::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(self, ast: &Ast<'_>) -> Option<ParenExpr> {
        match ast.simple_assign_target_data(self) {
            SimpleAssignTargetData::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(self, ast: &Ast<'_>) -> Option<OptChainExpr> {
        match ast.simple_assign_target_data(self) {
            SimpleAssignTargetData::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self, ast: &Ast<'_>) -> Option<Invalid> {
        match ast.simple_assign_target_data(self) {
            SimpleAssignTargetData::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl OptChainExpr {
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn base<'a>(self, ast: &crate::Ast<'a>) -> OptChainBase {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        OptChainBase(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
    #[inline]
    pub fn set_base<'a>(self, ast: &mut crate::Ast<'a>, base: OptChainBase) {
        let field_val: u32 = base.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl OptChainBase {
    #[inline]
    pub fn is_member(self, ast: &Ast<'_>) -> bool {
        matches!(ast.opt_chain_base_data(self), OptChainBaseData::Member(_))
    }
    #[inline]
    pub fn is_call(self, ast: &Ast<'_>) -> bool {
        matches!(ast.opt_chain_base_data(self), OptChainBaseData::Call(_))
    }
    #[inline]
    pub fn as_member(self, ast: &Ast<'_>) -> Option<MemberExpr> {
        match ast.opt_chain_base_data(self) {
            OptChainBaseData::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(self, ast: &Ast<'_>) -> Option<OptCall> {
        match ast.opt_chain_base_data(self) {
            OptChainBaseData::Call(it) => Some(it),
            _ => None,
        }
    }
}
impl OptCall {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn args<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExprOrSpread> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        let field_val: u32 = callee.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_args<'a>(self, ast: &mut crate::Ast<'a>, args: TypedSubRange<ExprOrSpread>) {
        let field_val: u64 = unsafe { args.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl Invalid {}
impl Function {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 4294967295u32;
        BlockStmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn is_generator<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn is_async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Param> {
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
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374325956791566335u128)
            | (((field_val as u128) & 4294967295u128) << 16usize);
    }
    #[inline]
    pub fn set_is_generator<'a>(self, ast: &mut crate::Ast<'a>, is_generator: bool) {
        let field_val: u32 = is_generator as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_is_async<'a>(self, ast: &mut crate::Ast<'a>, is_async: bool) {
        let field_val: u32 = is_async as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: TypedSubRange<Param>) {
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
impl Param {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn pat<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Pat(NodeId::from_raw_unchecked(raw))
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
        let field_val: u32 = pat.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl ParamOrTsParamProp {
    #[inline]
    pub fn is_param(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.param_or_ts_param_prop_data(self),
            ParamOrTsParamPropData::Param(_)
        )
    }
    #[inline]
    pub fn as_param(self, ast: &Ast<'_>) -> Option<Param> {
        match ast.param_or_ts_param_prop_data(self) {
            ParamOrTsParamPropData::Param(it) => Some(it),
            _ => None,
        }
    }
}
impl Class {
    #[inline]
    pub fn super_class<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn is_abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
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
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ClassMember> {
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
    pub fn set_super_class<'a>(self, ast: &mut crate::Ast<'a>, super_class: Option<Expr>) {
        let field_val: u32 = super_class.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_is_abstract<'a>(self, ast: &mut crate::Ast<'a>, is_abstract: bool) {
        let field_val: u32 = is_abstract as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
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
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<ClassMember>) {
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
impl ClassMember {
    #[inline]
    pub fn is_constructor(self, ast: &Ast<'_>) -> bool {
        matches!(ast.class_member_data(self), ClassMemberData::Constructor(_))
    }
    #[inline]
    pub fn is_method(self, ast: &Ast<'_>) -> bool {
        matches!(ast.class_member_data(self), ClassMemberData::Method(_))
    }
    #[inline]
    pub fn is_private_method(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.class_member_data(self),
            ClassMemberData::PrivateMethod(_)
        )
    }
    #[inline]
    pub fn is_class_prop(self, ast: &Ast<'_>) -> bool {
        matches!(ast.class_member_data(self), ClassMemberData::ClassProp(_))
    }
    #[inline]
    pub fn is_private_prop(self, ast: &Ast<'_>) -> bool {
        matches!(ast.class_member_data(self), ClassMemberData::PrivateProp(_))
    }
    #[inline]
    pub fn is_empty(self, ast: &Ast<'_>) -> bool {
        matches!(ast.class_member_data(self), ClassMemberData::Empty(_))
    }
    #[inline]
    pub fn is_static_block(self, ast: &Ast<'_>) -> bool {
        matches!(ast.class_member_data(self), ClassMemberData::StaticBlock(_))
    }
    #[inline]
    pub fn is_auto_accessor(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.class_member_data(self),
            ClassMemberData::AutoAccessor(_)
        )
    }
    #[inline]
    pub fn as_constructor(self, ast: &Ast<'_>) -> Option<Constructor> {
        match ast.class_member_data(self) {
            ClassMemberData::Constructor(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_method(self, ast: &Ast<'_>) -> Option<ClassMethod> {
        match ast.class_member_data(self) {
            ClassMemberData::Method(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_method(self, ast: &Ast<'_>) -> Option<PrivateMethod> {
        match ast.class_member_data(self) {
            ClassMemberData::PrivateMethod(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class_prop(self, ast: &Ast<'_>) -> Option<ClassProp> {
        match ast.class_member_data(self) {
            ClassMemberData::ClassProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_prop(self, ast: &Ast<'_>) -> Option<PrivateProp> {
        match ast.class_member_data(self) {
            ClassMemberData::PrivateProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_empty(self, ast: &Ast<'_>) -> Option<EmptyStmt> {
        match ast.class_member_data(self) {
            ClassMemberData::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_static_block(self, ast: &Ast<'_>) -> Option<StaticBlock> {
        match ast.class_member_data(self) {
            ClassMemberData::StaticBlock(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_auto_accessor(self, ast: &Ast<'_>) -> Option<AutoAccessor> {
        match ast.class_member_data(self) {
            ClassMemberData::AutoAccessor(it) => Some(it),
            _ => None,
        }
    }
}
impl ClassProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        PropName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
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
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let field_val: u32 = value.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let field_val: u32 = is_static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
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
}
impl PrivateProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PrivateName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        PrivateName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
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
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PrivateName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let field_val: u32 = value.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let field_val: u32 = is_static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
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
}
impl ClassMethod {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PropName(NodeId::from_raw_unchecked(raw))
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
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        PropName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ParamOrTsParamProp> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Option<BlockStmt> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BlockStmt(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
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
        let field_val: u64 = unsafe { params.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Option<BlockStmt>) {
        let field_val: u32 = body.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl Decorator {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
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
    pub fn is_private(self, ast: &Ast<'_>) -> bool {
        matches!(ast.key_data(self), KeyData::Private(_))
    }
    #[inline]
    pub fn is_public(self, ast: &Ast<'_>) -> bool {
        matches!(ast.key_data(self), KeyData::Public(_))
    }
    #[inline]
    pub fn as_private(self, ast: &Ast<'_>) -> Option<PrivateName> {
        match ast.key_data(self) {
            KeyData::Private(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_public(self, ast: &Ast<'_>) -> Option<PropName> {
        match ast.key_data(self) {
            KeyData::Public(it) => Some(it),
            _ => None,
        }
    }
}
impl AutoAccessor {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> Key {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        Key(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn is_static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
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
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: Key) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let field_val: u32 = value.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_is_static<'a>(self, ast: &mut crate::Ast<'a>, is_static: bool) {
        let field_val: u32 = is_static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
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
}
impl Prop {
    #[inline]
    pub fn is_shorthand(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_data(self), PropData::Shorthand(_))
    }
    #[inline]
    pub fn is_key_value(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_data(self), PropData::KeyValue(_))
    }
    #[inline]
    pub fn is_assign(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_data(self), PropData::Assign(_))
    }
    #[inline]
    pub fn is_getter(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_data(self), PropData::Getter(_))
    }
    #[inline]
    pub fn is_setter(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_data(self), PropData::Setter(_))
    }
    #[inline]
    pub fn is_method(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_data(self), PropData::Method(_))
    }
    #[inline]
    pub fn as_shorthand(self, ast: &Ast<'_>) -> Option<Ident> {
        match ast.prop_data(self) {
            PropData::Shorthand(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_key_value(self, ast: &Ast<'_>) -> Option<KeyValueProp> {
        match ast.prop_data(self) {
            PropData::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self, ast: &Ast<'_>) -> Option<AssignProp> {
        match ast.prop_data(self) {
            PropData::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_getter(self, ast: &Ast<'_>) -> Option<GetterProp> {
        match ast.prop_data(self) {
            PropData::Getter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_setter(self, ast: &Ast<'_>) -> Option<SetterProp> {
        match ast.prop_data(self) {
            PropData::Setter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_method(self, ast: &Ast<'_>) -> Option<MethodProp> {
        match ast.prop_data(self) {
            PropData::Method(it) => Some(it),
            _ => None,
        }
    }
}
impl KeyValueProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PropName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Expr) {
        let field_val: u32 = value.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl AssignProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> Ident {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Ident(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: Ident) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Expr) {
        let field_val: u32 = value.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl GetterProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PropName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
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
}
impl SetterProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PropName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
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
}
impl MethodProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PropName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn function<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Function(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
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
}
impl PropName {
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_name_data(self), PropNameData::Ident(_))
    }
    #[inline]
    pub fn is_str(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_name_data(self), PropNameData::Str(_))
    }
    #[inline]
    pub fn is_num(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_name_data(self), PropNameData::Num(_))
    }
    #[inline]
    pub fn is_computed(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_name_data(self), PropNameData::Computed(_))
    }
    #[inline]
    pub fn is_big_int(self, ast: &Ast<'_>) -> bool {
        matches!(ast.prop_name_data(self), PropNameData::BigInt(_))
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<IdentName> {
        match ast.prop_name_data(self) {
            PropNameData::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(self, ast: &Ast<'_>) -> Option<Str> {
        match ast.prop_name_data(self) {
            PropNameData::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self, ast: &Ast<'_>) -> Option<Number> {
        match ast.prop_name_data(self) {
            PropNameData::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(self, ast: &Ast<'_>) -> Option<ComputedPropName> {
        match ast.prop_name_data(self) {
            PropNameData::Computed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self, ast: &Ast<'_>) -> Option<BigInt> {
        match ast.prop_name_data(self) {
            PropNameData::BigInt(it) => Some(it),
            _ => None,
        }
    }
}
impl ComputedPropName {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl Pat {
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(ast.pat_data(self), PatData::Ident(_))
    }
    #[inline]
    pub fn is_array(self, ast: &Ast<'_>) -> bool {
        matches!(ast.pat_data(self), PatData::Array(_))
    }
    #[inline]
    pub fn is_rest(self, ast: &Ast<'_>) -> bool {
        matches!(ast.pat_data(self), PatData::Rest(_))
    }
    #[inline]
    pub fn is_object(self, ast: &Ast<'_>) -> bool {
        matches!(ast.pat_data(self), PatData::Object(_))
    }
    #[inline]
    pub fn is_assign(self, ast: &Ast<'_>) -> bool {
        matches!(ast.pat_data(self), PatData::Assign(_))
    }
    #[inline]
    pub fn is_invalid(self, ast: &Ast<'_>) -> bool {
        matches!(ast.pat_data(self), PatData::Invalid(_))
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(ast.pat_data(self), PatData::Expr(_))
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<BindingIdent> {
        match ast.pat_data(self) {
            PatData::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(self, ast: &Ast<'_>) -> Option<ArrayPat> {
        match ast.pat_data(self) {
            PatData::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_rest(self, ast: &Ast<'_>) -> Option<RestPat> {
        match ast.pat_data(self) {
            PatData::Rest(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(self, ast: &Ast<'_>) -> Option<ObjectPat> {
        match ast.pat_data(self) {
            PatData::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self, ast: &Ast<'_>) -> Option<AssignPat> {
        match ast.pat_data(self) {
            PatData::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(self, ast: &Ast<'_>) -> Option<Invalid> {
        match ast.pat_data(self) {
            PatData::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.pat_data(self) {
            PatData::Expr(it) => Some(it),
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Pat(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: Pat) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Pat(NodeId::from_raw_unchecked(raw))
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
        let field_val: u32 = arg.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl ObjectPatProp {
    #[inline]
    pub fn is_key_value(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.object_pat_prop_data(self),
            ObjectPatPropData::KeyValue(_)
        )
    }
    #[inline]
    pub fn is_assign(self, ast: &Ast<'_>) -> bool {
        matches!(ast.object_pat_prop_data(self), ObjectPatPropData::Assign(_))
    }
    #[inline]
    pub fn is_rest(self, ast: &Ast<'_>) -> bool {
        matches!(ast.object_pat_prop_data(self), ObjectPatPropData::Rest(_))
    }
    #[inline]
    pub fn as_key_value(self, ast: &Ast<'_>) -> Option<KeyValuePatProp> {
        match ast.object_pat_prop_data(self) {
            ObjectPatPropData::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(self, ast: &Ast<'_>) -> Option<AssignPatProp> {
        match ast.object_pat_prop_data(self) {
            ObjectPatPropData::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_rest(self, ast: &Ast<'_>) -> Option<RestPat> {
        match ast.object_pat_prop_data(self) {
            ObjectPatPropData::Rest(it) => Some(it),
            _ => None,
        }
    }
}
impl KeyValuePatProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PropName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Pat {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Pat(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Pat) {
        let field_val: u32 = value.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl AssignPatProp {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> BindingIdent {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingIdent(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: BindingIdent) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let field_val: u32 = value.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
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
    pub fn is_str(self, ast: &Ast<'_>) -> bool {
        matches!(ast.lit_data(self), LitData::Str(_))
    }
    #[inline]
    pub fn is_bool(self, ast: &Ast<'_>) -> bool {
        matches!(ast.lit_data(self), LitData::Bool(_))
    }
    #[inline]
    pub fn is_null(self, ast: &Ast<'_>) -> bool {
        matches!(ast.lit_data(self), LitData::Null(_))
    }
    #[inline]
    pub fn is_num(self, ast: &Ast<'_>) -> bool {
        matches!(ast.lit_data(self), LitData::Num(_))
    }
    #[inline]
    pub fn is_big_int(self, ast: &Ast<'_>) -> bool {
        matches!(ast.lit_data(self), LitData::BigInt(_))
    }
    #[inline]
    pub fn is_regex(self, ast: &Ast<'_>) -> bool {
        matches!(ast.lit_data(self), LitData::Regex(_))
    }
    #[inline]
    pub fn as_str(self, ast: &Ast<'_>) -> Option<Str> {
        match ast.lit_data(self) {
            LitData::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(self, ast: &Ast<'_>) -> Option<Bool> {
        match ast.lit_data(self) {
            LitData::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null(self, ast: &Ast<'_>) -> Option<Null> {
        match ast.lit_data(self) {
            LitData::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(self, ast: &Ast<'_>) -> Option<Number> {
        match ast.lit_data(self) {
            LitData::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(self, ast: &Ast<'_>) -> Option<BigInt> {
        match ast.lit_data(self) {
            LitData::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_regex(self, ast: &Ast<'_>) -> Option<Regex> {
        match ast.lit_data(self) {
            LitData::Regex(it) => Some(it),
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
    pub fn is_jsx_member_expr(self, ast: &Ast<'_>) -> bool {
        matches!(ast.jsx_object_data(self), JSXObjectData::JSXMemberExpr(_))
    }
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(ast.jsx_object_data(self), JSXObjectData::Ident(_))
    }
    #[inline]
    pub fn as_jsx_member_expr(self, ast: &Ast<'_>) -> Option<JSXMemberExpr> {
        match ast.jsx_object_data(self) {
            JSXObjectData::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<Ident> {
        match ast.jsx_object_data(self) {
            JSXObjectData::Ident(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXMemberExpr {
    #[inline]
    pub fn obj<'a>(self, ast: &crate::Ast<'a>) -> JSXObject {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        JSXObject(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn prop<'a>(self, ast: &crate::Ast<'a>) -> IdentName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        IdentName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_obj<'a>(self, ast: &mut crate::Ast<'a>, obj: JSXObject) {
        let field_val: u32 = obj.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_prop<'a>(self, ast: &mut crate::Ast<'a>, prop: IdentName) {
        let field_val: u32 = prop.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        JSXExpr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: JSXExpr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl JSXExpr {
    #[inline]
    pub fn is_jsx_empty_expr(self, ast: &Ast<'_>) -> bool {
        matches!(ast.jsx_expr_data(self), JSXExprData::JSXEmptyExpr(_))
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(ast.jsx_expr_data(self), JSXExprData::Expr(_))
    }
    #[inline]
    pub fn as_jsx_empty_expr(self, ast: &Ast<'_>) -> Option<JSXEmptyExpr> {
        match ast.jsx_expr_data(self) {
            JSXExprData::JSXEmptyExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.jsx_expr_data(self) {
            JSXExprData::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXSpreadChild {
    #[inline]
    pub fn expr<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expr<'a>(self, ast: &mut crate::Ast<'a>, expr: Expr) {
        let field_val: u32 = expr.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl JSXElementName {
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_element_name_data(self),
            JSXElementNameData::Ident(_)
        )
    }
    #[inline]
    pub fn is_jsx_member_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_element_name_data(self),
            JSXElementNameData::JSXMemberExpr(_)
        )
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_element_name_data(self),
            JSXElementNameData::JSXNamespacedName(_)
        )
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<Ident> {
        match ast.jsx_element_name_data(self) {
            JSXElementNameData::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member_expr(self, ast: &Ast<'_>) -> Option<JSXMemberExpr> {
        match ast.jsx_element_name_data(self) {
            JSXElementNameData::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self, ast: &Ast<'_>) -> Option<JSXNamespacedName> {
        match ast.jsx_element_name_data(self) {
            JSXElementNameData::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXOpeningElement {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JSXElementName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        JSXElementName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn attrs<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JSXAttrOrSpread> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn self_closing<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JSXElementName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_attrs<'a>(self, ast: &mut crate::Ast<'a>, attrs: TypedSubRange<JSXAttrOrSpread>) {
        let field_val: u64 = unsafe { attrs.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_self_closing<'a>(self, ast: &mut crate::Ast<'a>, self_closing: bool) {
        let field_val: u32 = self_closing as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
}
impl JSXAttrOrSpread {
    #[inline]
    pub fn is_jsx_attr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_attr_or_spread_data(self),
            JSXAttrOrSpreadData::JSXAttr(_)
        )
    }
    #[inline]
    pub fn is_spread_element(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_attr_or_spread_data(self),
            JSXAttrOrSpreadData::SpreadElement(_)
        )
    }
    #[inline]
    pub fn as_jsx_attr(self, ast: &Ast<'_>) -> Option<JSXAttr> {
        match ast.jsx_attr_or_spread_data(self) {
            JSXAttrOrSpreadData::JSXAttr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_spread_element(self, ast: &Ast<'_>) -> Option<SpreadElement> {
        match ast.jsx_attr_or_spread_data(self) {
            JSXAttrOrSpreadData::SpreadElement(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXClosingElement {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JSXElementName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        JSXElementName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JSXElementName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl JSXAttr {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JSXAttrName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        JSXAttrName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<JSXAttrValue> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| JSXAttrValue(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JSXAttrName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<JSXAttrValue>) {
        let field_val: u32 = value.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl JSXAttrName {
    #[inline]
    pub fn is_ident(self, ast: &Ast<'_>) -> bool {
        matches!(ast.jsx_attr_name_data(self), JSXAttrNameData::Ident(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_attr_name_data(self),
            JSXAttrNameData::JSXNamespacedName(_)
        )
    }
    #[inline]
    pub fn as_ident(self, ast: &Ast<'_>) -> Option<IdentName> {
        match ast.jsx_attr_name_data(self) {
            JSXAttrNameData::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self, ast: &Ast<'_>) -> Option<JSXNamespacedName> {
        match ast.jsx_attr_name_data(self) {
            JSXAttrNameData::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXAttrValue {
    #[inline]
    pub fn is_str(self, ast: &Ast<'_>) -> bool {
        matches!(ast.jsx_attr_value_data(self), JSXAttrValueData::Str(_))
    }
    #[inline]
    pub fn is_jsx_expr_container(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_attr_value_data(self),
            JSXAttrValueData::JSXExprContainer(_)
        )
    }
    #[inline]
    pub fn is_jsx_element(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_attr_value_data(self),
            JSXAttrValueData::JSXElement(_)
        )
    }
    #[inline]
    pub fn is_jsx_fragment(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_attr_value_data(self),
            JSXAttrValueData::JSXFragment(_)
        )
    }
    #[inline]
    pub fn as_str(self, ast: &Ast<'_>) -> Option<Str> {
        match ast.jsx_attr_value_data(self) {
            JSXAttrValueData::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expr_container(self, ast: &Ast<'_>) -> Option<JSXExprContainer> {
        match ast.jsx_attr_value_data(self) {
            JSXAttrValueData::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_>) -> Option<JSXElement> {
        match ast.jsx_attr_value_data(self) {
            JSXAttrValueData::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_>) -> Option<JSXFragment> {
        match ast.jsx_attr_value_data(self) {
            JSXAttrValueData::JSXFragment(it) => Some(it),
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
    pub fn is_jsx_text(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_element_child_data(self),
            JSXElementChildData::JSXText(_)
        )
    }
    #[inline]
    pub fn is_jsx_expr_container(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_element_child_data(self),
            JSXElementChildData::JSXExprContainer(_)
        )
    }
    #[inline]
    pub fn is_jsx_spread_child(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_element_child_data(self),
            JSXElementChildData::JSXSpreadChild(_)
        )
    }
    #[inline]
    pub fn is_jsx_element(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_element_child_data(self),
            JSXElementChildData::JSXElement(_)
        )
    }
    #[inline]
    pub fn is_jsx_fragment(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_element_child_data(self),
            JSXElementChildData::JSXFragment(_)
        )
    }
    #[inline]
    pub fn as_jsx_text(self, ast: &Ast<'_>) -> Option<JSXText> {
        match ast.jsx_element_child_data(self) {
            JSXElementChildData::JSXText(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expr_container(self, ast: &Ast<'_>) -> Option<JSXExprContainer> {
        match ast.jsx_element_child_data(self) {
            JSXElementChildData::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_spread_child(self, ast: &Ast<'_>) -> Option<JSXSpreadChild> {
        match ast.jsx_element_child_data(self) {
            JSXElementChildData::JSXSpreadChild(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_>) -> Option<JSXElement> {
        match ast.jsx_element_child_data(self) {
            JSXElementChildData::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_>) -> Option<JSXFragment> {
        match ast.jsx_element_child_data(self) {
            JSXElementChildData::JSXFragment(it) => Some(it),
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
