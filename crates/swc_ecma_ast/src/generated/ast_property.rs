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
    pub fn is_module(&self) -> bool {
        matches!(self, Self::Module(_))
    }
    #[inline]
    pub fn is_script(&self) -> bool {
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
    pub fn body(&self, ast: &crate::Ast) -> TypedSubRange<ModuleItem> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn shebang(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: TypedSubRange<ModuleItem>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
    #[inline]
    pub fn set_shebang(&self, ast: &mut crate::Ast, shebang: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = shebang.to_extra_data();
        };
    }
}
impl Script {
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> TypedSubRange<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn shebang(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: TypedSubRange<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
    #[inline]
    pub fn set_shebang(&self, ast: &mut crate::Ast, shebang: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = shebang.to_extra_data();
        };
    }
}
impl ModuleItem {
    #[inline]
    pub fn is_module_decl(&self) -> bool {
        matches!(self, Self::ModuleDecl(_))
    }
    #[inline]
    pub fn is_stmt(&self) -> bool {
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
    pub fn is_import(&self) -> bool {
        matches!(self, Self::Import(_))
    }
    #[inline]
    pub fn is_export_decl(&self) -> bool {
        matches!(self, Self::ExportDecl(_))
    }
    #[inline]
    pub fn is_export_named(&self) -> bool {
        matches!(self, Self::ExportNamed(_))
    }
    #[inline]
    pub fn is_export_default_decl(&self) -> bool {
        matches!(self, Self::ExportDefaultDecl(_))
    }
    #[inline]
    pub fn is_export_default_expr(&self) -> bool {
        matches!(self, Self::ExportDefaultExpr(_))
    }
    #[inline]
    pub fn is_export_all(&self) -> bool {
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
    pub fn specifiers(&self, ast: &crate::Ast) -> TypedSubRange<ImportSpecifier> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn src(&self, ast: &crate::Ast) -> Str {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn type_only(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn with(&self, ast: &crate::Ast) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn phase(&self, ast: &crate::Ast) -> ImportPhase {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_specifiers(&self, ast: &mut crate::Ast, specifiers: TypedSubRange<ImportSpecifier>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = specifiers.to_extra_data();
        };
    }
    #[inline]
    pub fn set_src(&self, ast: &mut crate::Ast, src: Str) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = src.to_extra_data();
        };
    }
    #[inline]
    pub fn set_type_only(&self, ast: &mut crate::Ast, type_only: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = type_only.to_extra_data();
        };
    }
    #[inline]
    pub fn set_with(&self, ast: &mut crate::Ast, with: Option<ObjectLit>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = with.to_extra_data();
        };
    }
    #[inline]
    pub fn set_phase(&self, ast: &mut crate::Ast, phase: ImportPhase) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = phase.to_extra_data();
        };
    }
}
impl ImportSpecifier {
    #[inline]
    pub fn is_named(&self) -> bool {
        matches!(self, Self::Named(_))
    }
    #[inline]
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }
    #[inline]
    pub fn is_namespace(&self) -> bool {
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
    pub fn is_type_only(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn local(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn imported(&self, ast: &crate::Ast) -> Option<ModuleExportName> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_is_type_only(&self, ast: &mut crate::Ast, is_type_only: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = is_type_only as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_local(&self, ast: &mut crate::Ast, local: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = local.to_extra_data();
        };
    }
    #[inline]
    pub fn set_imported(&self, ast: &mut crate::Ast, imported: Option<ModuleExportName>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = imported.to_extra_data();
        };
    }
}
impl ImportDefaultSpecifier {
    #[inline]
    pub fn local(&self, ast: &crate::Ast) -> Ident {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_local(&self, ast: &mut crate::Ast, local: Ident) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = local.node_id().index() as u32;
    }
}
impl ImportStarAsSpecifier {
    #[inline]
    pub fn local(&self, ast: &crate::Ast) -> Ident {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_local(&self, ast: &mut crate::Ast, local: Ident) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = local.node_id().index() as u32;
    }
}
impl ExportDecl {
    #[inline]
    pub fn decl(&self, ast: &crate::Ast) -> Decl {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_decl(&self, ast: &mut crate::Ast, decl: Decl) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decl.to_extra_data();
        };
    }
}
impl NamedExport {
    #[inline]
    pub fn specifiers(&self, ast: &crate::Ast) -> TypedSubRange<ExportSpecifier> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn src(&self, ast: &crate::Ast) -> Option<Str> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn type_only(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn with(&self, ast: &crate::Ast) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_specifiers(&self, ast: &mut crate::Ast, specifiers: TypedSubRange<ExportSpecifier>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = specifiers.to_extra_data();
        };
    }
    #[inline]
    pub fn set_src(&self, ast: &mut crate::Ast, src: Option<Str>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = src.to_extra_data();
        };
    }
    #[inline]
    pub fn set_type_only(&self, ast: &mut crate::Ast, type_only: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = type_only.to_extra_data();
        };
    }
    #[inline]
    pub fn set_with(&self, ast: &mut crate::Ast, with: Option<ObjectLit>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = with.to_extra_data();
        };
    }
}
impl ExportSpecifier {
    #[inline]
    pub fn is_namespace(&self) -> bool {
        matches!(self, Self::Namespace(_))
    }
    #[inline]
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }
    #[inline]
    pub fn is_named(&self) -> bool {
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
    pub fn name(&self, ast: &crate::Ast) -> ModuleExportName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: ModuleExportName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = name.to_extra_data();
        };
    }
}
impl ModuleExportName {
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(&self) -> bool {
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
    pub fn exported(&self, ast: &crate::Ast) -> Ident {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_exported(&self, ast: &mut crate::Ast, exported: Ident) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = exported.node_id().index() as u32;
    }
}
impl ExportNamedSpecifier {
    #[inline]
    pub fn is_type_only(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn orig(&self, ast: &crate::Ast) -> ModuleExportName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn exported(&self, ast: &crate::Ast) -> Option<ModuleExportName> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_is_type_only(&self, ast: &mut crate::Ast, is_type_only: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = is_type_only as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_orig(&self, ast: &mut crate::Ast, orig: ModuleExportName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = orig.to_extra_data();
        };
    }
    #[inline]
    pub fn set_exported(&self, ast: &mut crate::Ast, exported: Option<ModuleExportName>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = exported.to_extra_data();
        };
    }
}
impl ExportDefaultDecl {
    #[inline]
    pub fn decl(&self, ast: &crate::Ast) -> DefaultDecl {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_decl(&self, ast: &mut crate::Ast, decl: DefaultDecl) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decl.to_extra_data();
        };
    }
}
impl DefaultDecl {
    #[inline]
    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_fn(&self) -> bool {
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
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = expr.to_extra_data();
        };
    }
}
impl ExportAll {
    #[inline]
    pub fn type_only(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn src(&self, ast: &crate::Ast) -> Str {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn with(&self, ast: &crate::Ast) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_type_only(&self, ast: &mut crate::Ast, type_only: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = type_only as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_src(&self, ast: &mut crate::Ast, src: Str) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = src.to_extra_data();
        };
    }
    #[inline]
    pub fn set_with(&self, ast: &mut crate::Ast, with: Option<ObjectLit>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = with.to_extra_data();
        };
    }
}
impl BlockStmt {
    #[inline]
    pub fn stmts(&self, ast: &crate::Ast) -> TypedSubRange<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_stmts(&self, ast: &mut crate::Ast, stmts: TypedSubRange<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = stmts.to_extra_data();
        };
    }
}
impl Stmt {
    #[inline]
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block(_))
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty(_))
    }
    #[inline]
    pub fn is_debugger(&self) -> bool {
        matches!(self, Self::Debugger(_))
    }
    #[inline]
    pub fn is_with(&self) -> bool {
        matches!(self, Self::With(_))
    }
    #[inline]
    pub fn is_return(&self) -> bool {
        matches!(self, Self::Return(_))
    }
    #[inline]
    pub fn is_labeled(&self) -> bool {
        matches!(self, Self::Labeled(_))
    }
    #[inline]
    pub fn is_break(&self) -> bool {
        matches!(self, Self::Break(_))
    }
    #[inline]
    pub fn is_continue(&self) -> bool {
        matches!(self, Self::Continue(_))
    }
    #[inline]
    pub fn is_if(&self) -> bool {
        matches!(self, Self::If(_))
    }
    #[inline]
    pub fn is_switch(&self) -> bool {
        matches!(self, Self::Switch(_))
    }
    #[inline]
    pub fn is_throw(&self) -> bool {
        matches!(self, Self::Throw(_))
    }
    #[inline]
    pub fn is_try(&self) -> bool {
        matches!(self, Self::Try(_))
    }
    #[inline]
    pub fn is_while(&self) -> bool {
        matches!(self, Self::While(_))
    }
    #[inline]
    pub fn is_do_while(&self) -> bool {
        matches!(self, Self::DoWhile(_))
    }
    #[inline]
    pub fn is_for(&self) -> bool {
        matches!(self, Self::For(_))
    }
    #[inline]
    pub fn is_for_in(&self) -> bool {
        matches!(self, Self::ForIn(_))
    }
    #[inline]
    pub fn is_for_of(&self) -> bool {
        matches!(self, Self::ForOf(_))
    }
    #[inline]
    pub fn is_decl(&self) -> bool {
        matches!(self, Self::Decl(_))
    }
    #[inline]
    pub fn is_expr(&self) -> bool {
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
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = expr.to_extra_data();
        };
    }
}
impl EmptyStmt {}
impl DebuggerStmt {}
impl WithStmt {
    #[inline]
    pub fn obj(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_obj(&self, ast: &mut crate::Ast, obj: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = obj.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl ReturnStmt {
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = arg.to_extra_data();
        };
    }
}
impl LabeledStmt {
    #[inline]
    pub fn label(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_label(&self, ast: &mut crate::Ast, label: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = label.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl BreakStmt {
    #[inline]
    pub fn label(&self, ast: &crate::Ast) -> Option<Ident> {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_label(&self, ast: &mut crate::Ast, label: Option<Ident>) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = crate::OptionalNodeId::from(label.map(|n| n.node_id())).into_raw();
    }
}
impl ContinueStmt {
    #[inline]
    pub fn label(&self, ast: &crate::Ast) -> Option<Ident> {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_label(&self, ast: &mut crate::Ast, label: Option<Ident>) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = crate::OptionalNodeId::from(label.map(|n| n.node_id())).into_raw();
    }
}
impl IfStmt {
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn cons(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn alt(&self, ast: &crate::Ast) -> Option<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = test.to_extra_data();
        };
    }
    #[inline]
    pub fn set_cons(&self, ast: &mut crate::Ast, cons: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = cons.to_extra_data();
        };
    }
    #[inline]
    pub fn set_alt(&self, ast: &mut crate::Ast, alt: Option<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = alt.to_extra_data();
        };
    }
}
impl SwitchStmt {
    #[inline]
    pub fn discriminant(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn cases(&self, ast: &crate::Ast) -> TypedSubRange<SwitchCase> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_discriminant(&self, ast: &mut crate::Ast, discriminant: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = discriminant.to_extra_data();
        };
    }
    #[inline]
    pub fn set_cases(&self, ast: &mut crate::Ast, cases: TypedSubRange<SwitchCase>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = cases.to_extra_data();
        };
    }
}
impl ThrowStmt {
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = arg.to_extra_data();
        };
    }
}
impl TryStmt {
    #[inline]
    pub fn block(&self, ast: &crate::Ast) -> BlockStmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn handler(&self, ast: &crate::Ast) -> Option<CatchClause> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn finalizer(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_block(&self, ast: &mut crate::Ast, block: BlockStmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = block.to_extra_data();
        };
    }
    #[inline]
    pub fn set_handler(&self, ast: &mut crate::Ast, handler: Option<CatchClause>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = handler.to_extra_data();
        };
    }
    #[inline]
    pub fn set_finalizer(&self, ast: &mut crate::Ast, finalizer: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = finalizer.to_extra_data();
        };
    }
}
impl WhileStmt {
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = test.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl DoWhileStmt {
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = test.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl ForStmt {
    #[inline]
    pub fn init(&self, ast: &crate::Ast) -> Option<VarDeclOrExpr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn update(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_init(&self, ast: &mut crate::Ast, init: Option<VarDeclOrExpr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = init.to_extra_data();
        };
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = test.to_extra_data();
        };
    }
    #[inline]
    pub fn set_update(&self, ast: &mut crate::Ast, update: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = update.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl ForInStmt {
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> ForHead {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: ForHead) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = left.to_extra_data();
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = right.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl ForOfStmt {
    #[inline]
    pub fn is_await(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> ForHead {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_is_await(&self, ast: &mut crate::Ast, is_await: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = is_await as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: ForHead) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = left.to_extra_data();
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = right.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl SwitchCase {
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn cons(&self, ast: &crate::Ast) -> TypedSubRange<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = test.to_extra_data();
        };
    }
    #[inline]
    pub fn set_cons(&self, ast: &mut crate::Ast, cons: TypedSubRange<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = cons.to_extra_data();
        };
    }
}
impl CatchClause {
    #[inline]
    pub fn param(&self, ast: &crate::Ast) -> Option<Pat> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> BlockStmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_param(&self, ast: &mut crate::Ast, param: Option<Pat>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = param.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: BlockStmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl ForHead {
    #[inline]
    pub fn is_var_decl(&self) -> bool {
        matches!(self, Self::VarDecl(_))
    }
    #[inline]
    pub fn is_using_decl(&self) -> bool {
        matches!(self, Self::UsingDecl(_))
    }
    #[inline]
    pub fn is_pat(&self) -> bool {
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
    pub fn is_var_decl(&self) -> bool {
        matches!(self, Self::VarDecl(_))
    }
    #[inline]
    pub fn is_expr(&self) -> bool {
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
    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_fn(&self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
    }
    #[inline]
    pub fn is_using(&self) -> bool {
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
    pub fn declare(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn ident(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_declare(&self, ast: &mut crate::Ast, declare: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = declare as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_ident(&self, ast: &mut crate::Ast, ident: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = ident.to_extra_data();
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = function.to_extra_data();
        };
    }
}
impl ClassDecl {
    #[inline]
    pub fn declare(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn ident(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn class(&self, ast: &crate::Ast) -> Class {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_declare(&self, ast: &mut crate::Ast, declare: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = declare as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_ident(&self, ast: &mut crate::Ast, ident: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = ident.to_extra_data();
        };
    }
    #[inline]
    pub fn set_class(&self, ast: &mut crate::Ast, class: Class) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = class.to_extra_data();
        };
    }
}
impl VarDecl {
    #[inline]
    pub fn kind(&self, ast: &crate::Ast) -> VarDeclKind {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn declare(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decls(&self, ast: &crate::Ast) -> TypedSubRange<VarDeclarator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_kind(&self, ast: &mut crate::Ast, kind: VarDeclKind) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = kind.to_extra_data();
        };
    }
    #[inline]
    pub fn set_declare(&self, ast: &mut crate::Ast, declare: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = declare.to_extra_data();
        };
    }
    #[inline]
    pub fn set_decls(&self, ast: &mut crate::Ast, decls: TypedSubRange<VarDeclarator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decls.to_extra_data();
        };
    }
}
impl VarDeclarator {
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn init(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = name.to_extra_data();
        };
    }
    #[inline]
    pub fn set_init(&self, ast: &mut crate::Ast, init: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = init.to_extra_data();
        };
    }
}
impl UsingDecl {
    #[inline]
    pub fn is_await(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decls(&self, ast: &crate::Ast) -> TypedSubRange<VarDeclarator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_is_await(&self, ast: &mut crate::Ast, is_await: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = is_await.to_extra_data();
        };
    }
    #[inline]
    pub fn set_decls(&self, ast: &mut crate::Ast, decls: TypedSubRange<VarDeclarator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decls.to_extra_data();
        };
    }
}
impl Expr {
    #[inline]
    pub fn is_this(&self) -> bool {
        matches!(self, Self::This(_))
    }
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_fn(&self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn is_unary(&self) -> bool {
        matches!(self, Self::Unary(_))
    }
    #[inline]
    pub fn is_update(&self) -> bool {
        matches!(self, Self::Update(_))
    }
    #[inline]
    pub fn is_bin(&self) -> bool {
        matches!(self, Self::Bin(_))
    }
    #[inline]
    pub fn is_assign(&self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_member(&self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_super_prop(&self) -> bool {
        matches!(self, Self::SuperProp(_))
    }
    #[inline]
    pub fn is_cond(&self) -> bool {
        matches!(self, Self::Cond(_))
    }
    #[inline]
    pub fn is_call(&self) -> bool {
        matches!(self, Self::Call(_))
    }
    #[inline]
    pub fn is_new(&self) -> bool {
        matches!(self, Self::New(_))
    }
    #[inline]
    pub fn is_seq(&self) -> bool {
        matches!(self, Self::Seq(_))
    }
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_lit(&self) -> bool {
        matches!(self, Self::Lit(_))
    }
    #[inline]
    pub fn is_tpl(&self) -> bool {
        matches!(self, Self::Tpl(_))
    }
    #[inline]
    pub fn is_tagged_tpl(&self) -> bool {
        matches!(self, Self::TaggedTpl(_))
    }
    #[inline]
    pub fn is_arrow(&self) -> bool {
        matches!(self, Self::Arrow(_))
    }
    #[inline]
    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_yield(&self) -> bool {
        matches!(self, Self::Yield(_))
    }
    #[inline]
    pub fn is_meta_prop(&self) -> bool {
        matches!(self, Self::MetaProp(_))
    }
    #[inline]
    pub fn is_await(&self) -> bool {
        matches!(self, Self::Await(_))
    }
    #[inline]
    pub fn is_paren(&self) -> bool {
        matches!(self, Self::Paren(_))
    }
    #[inline]
    pub fn is_jsx_member(&self) -> bool {
        matches!(self, Self::JSXMember(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(&self) -> bool {
        matches!(self, Self::JSXNamespacedName(_))
    }
    #[inline]
    pub fn is_jsx_empty(&self) -> bool {
        matches!(self, Self::JSXEmpty(_))
    }
    #[inline]
    pub fn is_jsx_element(&self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(&self) -> bool {
        matches!(self, Self::JSXFragment(_))
    }
    #[inline]
    pub fn is_private_name(&self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
    #[inline]
    pub fn is_opt_chain(&self) -> bool {
        matches!(self, Self::OptChain(_))
    }
    #[inline]
    pub fn is_invalid(&self) -> bool {
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
    pub fn elems(&self, ast: &crate::Ast) -> TypedSubRange<Option<ExprOrSpread>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_elems(&self, ast: &mut crate::Ast, elems: TypedSubRange<Option<ExprOrSpread>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = elems.to_extra_data();
        };
    }
}
impl ObjectLit {
    #[inline]
    pub fn props(&self, ast: &crate::Ast) -> TypedSubRange<PropOrSpread> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_props(&self, ast: &mut crate::Ast, props: TypedSubRange<PropOrSpread>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = props.to_extra_data();
        };
    }
}
impl PropOrSpread {
    #[inline]
    pub fn is_spread_element(&self) -> bool {
        matches!(self, Self::SpreadElement(_))
    }
    #[inline]
    pub fn is_prop(&self) -> bool {
        matches!(self, Self::Prop(_))
    }
    #[inline]
    pub fn as_spread_element(self) -> Option<SpreadElement> {
        match self {
            Self::SpreadElement(it) => Some(it),
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
    pub fn dot_3_token(&self, ast: &crate::Ast) -> Span {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_dot3_token(&self, ast: &mut crate::Ast, dot3_token: Span) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = dot3_token.to_extra_data();
        };
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = expr.to_extra_data();
        };
    }
}
impl UnaryExpr {
    #[inline]
    pub fn op(&self, ast: &crate::Ast) -> UnaryOp {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, UnaryOp>(raw as u8) }
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_op(&self, ast: &mut crate::Ast, op: UnaryOp) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = op as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = arg.to_extra_data();
        };
    }
}
impl UpdateExpr {
    #[inline]
    pub fn op(&self, ast: &crate::Ast) -> UpdateOp {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, UpdateOp>(raw as u8) }
    }
    #[inline]
    pub fn prefix(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = (u32::from(node.inline_data) >> 8usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_op(&self, ast: &mut crate::Ast, op: UpdateOp) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = op as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_prefix(&self, ast: &mut crate::Ast, prefix: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = prefix as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16711935u32) | ((field_val & 255u32) << 8usize)).into();
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = arg.to_extra_data();
        };
    }
}
impl BinExpr {
    #[inline]
    pub fn op(&self, ast: &crate::Ast) -> BinaryOp {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, BinaryOp>(raw as u8) }
    }
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_op(&self, ast: &mut crate::Ast, op: BinaryOp) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = op as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = left.to_extra_data();
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = right.to_extra_data();
        };
    }
}
impl FnExpr {
    #[inline]
    pub fn ident(&self, ast: &crate::Ast) -> Option<Ident> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_ident(&self, ast: &mut crate::Ast, ident: Option<Ident>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = ident.to_extra_data();
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = function.to_extra_data();
        };
    }
}
impl ClassExpr {
    #[inline]
    pub fn ident(&self, ast: &crate::Ast) -> Option<Ident> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn class(&self, ast: &crate::Ast) -> Class {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_ident(&self, ast: &mut crate::Ast, ident: Option<Ident>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = ident.to_extra_data();
        };
    }
    #[inline]
    pub fn set_class(&self, ast: &mut crate::Ast, class: Class) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = class.to_extra_data();
        };
    }
}
impl AssignExpr {
    #[inline]
    pub fn op(&self, ast: &crate::Ast) -> AssignOp {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, AssignOp>(raw as u8) }
    }
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> AssignTarget {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_op(&self, ast: &mut crate::Ast, op: AssignOp) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = op as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: AssignTarget) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = left.to_extra_data();
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = right.to_extra_data();
        };
    }
}
impl MemberExpr {
    #[inline]
    pub fn obj(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn prop(&self, ast: &crate::Ast) -> MemberProp {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_obj(&self, ast: &mut crate::Ast, obj: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = obj.to_extra_data();
        };
    }
    #[inline]
    pub fn set_prop(&self, ast: &mut crate::Ast, prop: MemberProp) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = prop.to_extra_data();
        };
    }
}
impl MemberProp {
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_private_name(&self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
    #[inline]
    pub fn is_computed(&self) -> bool {
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
    pub fn obj(&self, ast: &crate::Ast) -> Super {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn prop(&self, ast: &crate::Ast) -> SuperProp {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_obj(&self, ast: &mut crate::Ast, obj: Super) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = obj.to_extra_data();
        };
    }
    #[inline]
    pub fn set_prop(&self, ast: &mut crate::Ast, prop: SuperProp) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = prop.to_extra_data();
        };
    }
}
impl SuperProp {
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_computed(&self) -> bool {
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
    pub fn test(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn cons(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn alt(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = test.to_extra_data();
        };
    }
    #[inline]
    pub fn set_cons(&self, ast: &mut crate::Ast, cons: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = cons.to_extra_data();
        };
    }
    #[inline]
    pub fn set_alt(&self, ast: &mut crate::Ast, alt: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = alt.to_extra_data();
        };
    }
}
impl CallExpr {
    #[inline]
    pub fn callee(&self, ast: &crate::Ast) -> Callee {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn args(&self, ast: &crate::Ast) -> TypedSubRange<ExprOrSpread> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_callee(&self, ast: &mut crate::Ast, callee: Callee) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = callee.to_extra_data();
        };
    }
    #[inline]
    pub fn set_args(&self, ast: &mut crate::Ast, args: TypedSubRange<ExprOrSpread>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = args.to_extra_data();
        };
    }
}
impl NewExpr {
    #[inline]
    pub fn callee(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn args(&self, ast: &crate::Ast) -> Option<TypedSubRange<ExprOrSpread>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_callee(&self, ast: &mut crate::Ast, callee: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = callee.to_extra_data();
        };
    }
    #[inline]
    pub fn set_args(&self, ast: &mut crate::Ast, args: Option<TypedSubRange<ExprOrSpread>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = args.to_extra_data();
        };
    }
}
impl SeqExpr {
    #[inline]
    pub fn exprs(&self, ast: &crate::Ast) -> TypedSubRange<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_exprs(&self, ast: &mut crate::Ast, exprs: TypedSubRange<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = exprs.to_extra_data();
        };
    }
}
impl ArrowExpr {
    #[inline]
    pub fn params(&self, ast: &crate::Ast) -> TypedSubRange<Pat> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> BlockStmtOrExpr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_async(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_generator(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_params(&self, ast: &mut crate::Ast, params: TypedSubRange<Pat>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = params.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: BlockStmtOrExpr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
    #[inline]
    pub fn set_is_async(&self, ast: &mut crate::Ast, is_async: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = is_async.to_extra_data();
        };
    }
    #[inline]
    pub fn set_is_generator(&self, ast: &mut crate::Ast, is_generator: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = is_generator.to_extra_data();
        };
    }
}
impl YieldExpr {
    #[inline]
    pub fn delegate(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_delegate(&self, ast: &mut crate::Ast, delegate: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = delegate as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = arg.to_extra_data();
        };
    }
}
impl MetaPropExpr {
    #[inline]
    pub fn kind(&self, ast: &crate::Ast) -> MetaPropKind {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { std::mem::transmute::<u8, MetaPropKind>(raw as u8) }
    }
    #[inline]
    pub fn set_kind(&self, ast: &mut crate::Ast, kind: MetaPropKind) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = kind as u32;
    }
}
impl AwaitExpr {
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = arg.to_extra_data();
        };
    }
}
impl Tpl {
    #[inline]
    pub fn exprs(&self, ast: &crate::Ast) -> TypedSubRange<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn quasis(&self, ast: &crate::Ast) -> TypedSubRange<TplElement> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_exprs(&self, ast: &mut crate::Ast, exprs: TypedSubRange<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = exprs.to_extra_data();
        };
    }
    #[inline]
    pub fn set_quasis(&self, ast: &mut crate::Ast, quasis: TypedSubRange<TplElement>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = quasis.to_extra_data();
        };
    }
}
impl TaggedTpl {
    #[inline]
    pub fn tag(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn tpl(&self, ast: &crate::Ast) -> Tpl {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_tag(&self, ast: &mut crate::Ast, tag: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = tag.to_extra_data();
        };
    }
    #[inline]
    pub fn set_tpl(&self, ast: &mut crate::Ast, tpl: Tpl) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = tpl.to_extra_data();
        };
    }
}
impl TplElement {
    #[inline]
    pub fn tail(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn cooked(&self, ast: &crate::Ast) -> OptionalWtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_tail(&self, ast: &mut crate::Ast, tail: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = tail as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_cooked(&self, ast: &mut crate::Ast, cooked: OptionalWtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = cooked.to_extra_data();
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data();
        };
    }
}
impl ParenExpr {
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = expr.to_extra_data();
        };
    }
}
impl Callee {
    #[inline]
    pub fn is_super(&self) -> bool {
        matches!(self, Self::Super(_))
    }
    #[inline]
    pub fn is_import(&self) -> bool {
        matches!(self, Self::Import(_))
    }
    #[inline]
    pub fn is_expr(&self) -> bool {
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
    pub fn as_import(self) -> Option<Import> {
        match self {
            Self::Import(it) => Some(it),
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
impl Import {
    #[inline]
    pub fn phase(&self, ast: &crate::Ast) -> ImportPhase {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { std::mem::transmute::<u8, ImportPhase>(raw as u8) }
    }
    #[inline]
    pub fn set_phase(&self, ast: &mut crate::Ast, phase: ImportPhase) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = phase as u32;
    }
}
impl ExprOrSpread {
    #[inline]
    pub fn spread(&self, ast: &crate::Ast) -> Option<SpreadDot3Token> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_spread(&self, ast: &mut crate::Ast, spread: Option<SpreadDot3Token>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = spread.to_extra_data();
        };
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = expr.to_extra_data();
        };
    }
}
impl SpreadDot3Token {}
impl BlockStmtOrExpr {
    #[inline]
    pub fn is_block_stmt(&self) -> bool {
        matches!(self, Self::BlockStmt(_))
    }
    #[inline]
    pub fn is_expr(&self) -> bool {
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
    pub fn is_simple(&self) -> bool {
        matches!(self, Self::Simple(_))
    }
    #[inline]
    pub fn is_pat(&self) -> bool {
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
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_invalid(&self) -> bool {
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
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_member(&self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_super_prop(&self) -> bool {
        matches!(self, Self::SuperProp(_))
    }
    #[inline]
    pub fn is_paren(&self) -> bool {
        matches!(self, Self::Paren(_))
    }
    #[inline]
    pub fn is_opt_chain(&self) -> bool {
        matches!(self, Self::OptChain(_))
    }
    #[inline]
    pub fn is_invalid(&self) -> bool {
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
    pub fn optional(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn base(&self, ast: &crate::Ast) -> OptChainBase {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_optional(&self, ast: &mut crate::Ast, optional: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = optional as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_base(&self, ast: &mut crate::Ast, base: OptChainBase) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = base.to_extra_data();
        };
    }
}
impl OptChainBase {
    #[inline]
    pub fn is_member(&self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_call(&self) -> bool {
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
    pub fn callee(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn args(&self, ast: &crate::Ast) -> TypedSubRange<ExprOrSpread> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_callee(&self, ast: &mut crate::Ast, callee: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = callee.to_extra_data();
        };
    }
    #[inline]
    pub fn set_args(&self, ast: &mut crate::Ast, args: TypedSubRange<ExprOrSpread>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = args.to_extra_data();
        };
    }
}
impl Invalid {}
impl Function {
    #[inline]
    pub fn params(&self, ast: &crate::Ast) -> TypedSubRange<Param> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_generator(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_async(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_params(&self, ast: &mut crate::Ast, params: TypedSubRange<Param>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = params.to_extra_data();
        };
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
    #[inline]
    pub fn set_is_generator(&self, ast: &mut crate::Ast, is_generator: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = is_generator.to_extra_data();
        };
    }
    #[inline]
    pub fn set_is_async(&self, ast: &mut crate::Ast, is_async: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = is_async.to_extra_data();
        };
    }
}
impl Param {
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn pat(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data();
        };
    }
    #[inline]
    pub fn set_pat(&self, ast: &mut crate::Ast, pat: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = pat.to_extra_data();
        };
    }
}
impl ParamOrTsParamProp {
    #[inline]
    pub fn is_param(&self) -> bool {
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
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> TypedSubRange<ClassMember> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn super_class(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_abstract(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: TypedSubRange<ClassMember>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
    #[inline]
    pub fn set_super_class(&self, ast: &mut crate::Ast, super_class: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = super_class.to_extra_data();
        };
    }
    #[inline]
    pub fn set_is_abstract(&self, ast: &mut crate::Ast, is_abstract: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = is_abstract.to_extra_data();
        };
    }
}
impl ClassMember {
    #[inline]
    pub fn is_constructor(&self) -> bool {
        matches!(self, Self::Constructor(_))
    }
    #[inline]
    pub fn is_method(&self) -> bool {
        matches!(self, Self::Method(_))
    }
    #[inline]
    pub fn is_private_method(&self) -> bool {
        matches!(self, Self::PrivateMethod(_))
    }
    #[inline]
    pub fn is_class_prop(&self) -> bool {
        matches!(self, Self::ClassProp(_))
    }
    #[inline]
    pub fn is_private_prop(&self) -> bool {
        matches!(self, Self::PrivateProp(_))
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty(_))
    }
    #[inline]
    pub fn is_static_block(&self) -> bool {
        matches!(self, Self::StaticBlock(_))
    }
    #[inline]
    pub fn is_auto_accessor(&self) -> bool {
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
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = is_static.to_extra_data();
        };
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data();
        };
    }
}
impl PrivateProp {
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PrivateName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PrivateName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = is_static.to_extra_data();
        };
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data();
        };
    }
}
impl ClassMethod {
    #[inline]
    pub fn kind(&self, ast: &crate::Ast) -> MethodKind {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodKind>(raw as u8) }
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = (u32::from(node.inline_data) >> 8usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_kind(&self, ast: &mut crate::Ast, kind: MethodKind) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = kind as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = is_static as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16711935u32) | ((field_val & 255u32) << 8usize)).into();
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = function.to_extra_data();
        };
    }
}
impl PrivateMethod {
    #[inline]
    pub fn kind(&self, ast: &crate::Ast) -> MethodKind {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodKind>(raw as u8) }
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = (u32::from(node.inline_data) >> 8usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PrivateName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_kind(&self, ast: &mut crate::Ast, kind: MethodKind) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = kind as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = is_static as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16711935u32) | ((field_val & 255u32) << 8usize)).into();
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PrivateName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = function.to_extra_data();
        };
    }
}
impl Constructor {
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn params(&self, ast: &crate::Ast) -> TypedSubRange<ParamOrTsParamProp> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_params(&self, ast: &mut crate::Ast, params: TypedSubRange<ParamOrTsParamProp>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = params.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl Decorator {
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = expr.to_extra_data();
        };
    }
}
impl StaticBlock {
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> BlockStmt {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { BlockStmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: BlockStmt) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = body.node_id().index() as u32;
    }
}
impl Key {
    #[inline]
    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private(_))
    }
    #[inline]
    pub fn is_public(&self) -> bool {
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
    pub fn key(&self, ast: &crate::Ast) -> Key {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: Key) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = is_static.to_extra_data();
        };
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data();
        };
    }
}
impl Prop {
    #[inline]
    pub fn is_shorthand(&self) -> bool {
        matches!(self, Self::Shorthand(_))
    }
    #[inline]
    pub fn is_key_value(&self) -> bool {
        matches!(self, Self::KeyValue(_))
    }
    #[inline]
    pub fn is_assign(&self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_getter(&self) -> bool {
        matches!(self, Self::Getter(_))
    }
    #[inline]
    pub fn is_setter(&self) -> bool {
        matches!(self, Self::Setter(_))
    }
    #[inline]
    pub fn is_method(&self) -> bool {
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
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
}
impl AssignProp {
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
}
impl GetterProp {
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl SetterProp {
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn this_param(&self, ast: &crate::Ast) -> Option<Pat> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn param(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_this_param(&self, ast: &mut crate::Ast, this_param: Option<Pat>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = this_param.to_extra_data();
        };
    }
    #[inline]
    pub fn set_param(&self, ast: &mut crate::Ast, param: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = param.to_extra_data();
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = body.to_extra_data();
        };
    }
}
impl MethodProp {
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = function.to_extra_data();
        };
    }
}
impl PropName {
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_num(&self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_computed(&self) -> bool {
        matches!(self, Self::Computed(_))
    }
    #[inline]
    pub fn is_big_int(&self) -> bool {
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
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = expr.to_extra_data();
        };
    }
}
impl Pat {
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }
    #[inline]
    pub fn is_rest(&self) -> bool {
        matches!(self, Self::Rest(_))
    }
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }
    #[inline]
    pub fn is_assign(&self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn is_expr(&self) -> bool {
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
    pub fn elems(&self, ast: &crate::Ast) -> TypedSubRange<Option<Pat>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn optional(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_elems(&self, ast: &mut crate::Ast, elems: TypedSubRange<Option<Pat>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = elems.to_extra_data();
        };
    }
    #[inline]
    pub fn set_optional(&self, ast: &mut crate::Ast, optional: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = optional.to_extra_data();
        };
    }
}
impl ObjectPat {
    #[inline]
    pub fn props(&self, ast: &crate::Ast) -> TypedSubRange<ObjectPatProp> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn optional(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_props(&self, ast: &mut crate::Ast, props: TypedSubRange<ObjectPatProp>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = props.to_extra_data();
        };
    }
    #[inline]
    pub fn set_optional(&self, ast: &mut crate::Ast, optional: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = optional.to_extra_data();
        };
    }
}
impl AssignPat {
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = left.to_extra_data();
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = right.to_extra_data();
        };
    }
}
impl RestPat {
    #[inline]
    pub fn dot_3_token(&self, ast: &crate::Ast) -> Span {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_dot3_token(&self, ast: &mut crate::Ast, dot3_token: Span) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = dot3_token.to_extra_data();
        };
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = arg.to_extra_data();
        };
    }
}
impl ObjectPatProp {
    #[inline]
    pub fn is_key_value(&self) -> bool {
        matches!(self, Self::KeyValue(_))
    }
    #[inline]
    pub fn is_assign(&self) -> bool {
        matches!(self, Self::Assign(_))
    }
    #[inline]
    pub fn is_rest(&self) -> bool {
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
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
}
impl AssignPatProp {
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> BindingIdent {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: BindingIdent) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = key.to_extra_data();
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
}
impl Ident {
    #[inline]
    pub fn optional(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn sym(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_optional(&self, ast: &mut crate::Ast, optional: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        let field_val: u32 = optional as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_sym(&self, ast: &mut crate::Ast, sym: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = sym.to_extra_data();
        };
    }
}
impl IdentName {
    #[inline]
    pub fn sym(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_sym(&self, ast: &mut crate::Ast, sym: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = sym.to_extra_data();
        };
    }
}
impl PrivateName {
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = name.to_extra_data();
        };
    }
}
impl BindingIdent {
    #[inline]
    pub fn id(&self, ast: &crate::Ast) -> Ident {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_id(&self, ast: &mut crate::Ast, id: Ident) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = id.node_id().index() as u32;
    }
}
impl Lit {
    #[inline]
    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null(_))
    }
    #[inline]
    pub fn is_num(&self) -> bool {
        matches!(self, Self::Num(_))
    }
    #[inline]
    pub fn is_big_int(&self) -> bool {
        matches!(self, Self::BigInt(_))
    }
    #[inline]
    pub fn is_regex(&self) -> bool {
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
    pub fn value(&self, ast: &crate::Ast) -> Wtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Wtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data();
        };
    }
}
impl Bool {
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.get_node_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        raw != 0
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: bool) {
        let node = unsafe { ast.get_node_unchecked_mut(self.0) };
        node.data.inline_data = value as u32;
    }
}
impl Null {}
impl Number {
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> f64 {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: f64) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data();
        };
    }
}
impl BigInt {
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> BigIntId {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: BigIntId) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data();
        };
    }
}
impl Regex {
    #[inline]
    pub fn exp(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn flags(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_exp(&self, ast: &mut crate::Ast, exp: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = exp.to_extra_data();
        };
    }
    #[inline]
    pub fn set_flags(&self, ast: &mut crate::Ast, flags: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = flags.to_extra_data();
        };
    }
}
impl JSXObject {
    #[inline]
    pub fn is_jsx_member_expr(&self) -> bool {
        matches!(self, Self::JSXMemberExpr(_))
    }
    #[inline]
    pub fn is_ident(&self) -> bool {
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
    pub fn obj(&self, ast: &crate::Ast) -> JSXObject {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn prop(&self, ast: &crate::Ast) -> IdentName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_obj(&self, ast: &mut crate::Ast, obj: JSXObject) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = obj.to_extra_data();
        };
    }
    #[inline]
    pub fn set_prop(&self, ast: &mut crate::Ast, prop: IdentName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = prop.to_extra_data();
        };
    }
}
impl JSXNamespacedName {
    #[inline]
    pub fn ns(&self, ast: &crate::Ast) -> IdentName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> IdentName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_ns(&self, ast: &mut crate::Ast, ns: IdentName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = ns.to_extra_data();
        };
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: IdentName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = name.to_extra_data();
        };
    }
}
impl JSXEmptyExpr {}
impl JSXExprContainer {
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> JSXExpr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: JSXExpr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = expr.to_extra_data();
        };
    }
}
impl JSXExpr {
    #[inline]
    pub fn is_jsx_empty_expr(&self) -> bool {
        matches!(self, Self::JSXEmptyExpr(_))
    }
    #[inline]
    pub fn is_expr(&self) -> bool {
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
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = expr.to_extra_data();
        };
    }
}
impl JSXElementName {
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_jsx_member_expr(&self) -> bool {
        matches!(self, Self::JSXMemberExpr(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(&self) -> bool {
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
    pub fn name(&self, ast: &crate::Ast) -> JSXElementName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn attrs(&self, ast: &crate::Ast) -> TypedSubRange<JSXAttrOrSpread> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn self_closing(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: JSXElementName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = name.to_extra_data();
        };
    }
    #[inline]
    pub fn set_attrs(&self, ast: &mut crate::Ast, attrs: TypedSubRange<JSXAttrOrSpread>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = attrs.to_extra_data();
        };
    }
    #[inline]
    pub fn set_self_closing(&self, ast: &mut crate::Ast, self_closing: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = self_closing.to_extra_data();
        };
    }
}
impl JSXAttrOrSpread {
    #[inline]
    pub fn is_jsx_attr(&self) -> bool {
        matches!(self, Self::JSXAttr(_))
    }
    #[inline]
    pub fn is_spread_element(&self) -> bool {
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
    pub fn name(&self, ast: &crate::Ast) -> JSXElementName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: JSXElementName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = name.to_extra_data();
        };
    }
}
impl JSXAttr {
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> JSXAttrName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<JSXAttrValue> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: JSXAttrName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = name.to_extra_data();
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<JSXAttrValue>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
}
impl JSXAttrName {
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(&self) -> bool {
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
    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn is_jsx_expr_container(&self) -> bool {
        matches!(self, Self::JSXExprContainer(_))
    }
    #[inline]
    pub fn is_jsx_element(&self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(&self) -> bool {
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
    pub fn value(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = value.to_extra_data();
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data();
        };
    }
}
impl JSXElement {
    #[inline]
    pub fn opening(&self, ast: &crate::Ast) -> JSXOpeningElement {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn children(&self, ast: &crate::Ast) -> TypedSubRange<JSXElementChild> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn closing(&self, ast: &crate::Ast) -> Option<JSXClosingElement> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_opening(&self, ast: &mut crate::Ast, opening: JSXOpeningElement) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = opening.to_extra_data();
        };
    }
    #[inline]
    pub fn set_children(&self, ast: &mut crate::Ast, children: TypedSubRange<JSXElementChild>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = children.to_extra_data();
        };
    }
    #[inline]
    pub fn set_closing(&self, ast: &mut crate::Ast, closing: Option<JSXClosingElement>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = closing.to_extra_data();
        };
    }
}
impl JSXElementChild {
    #[inline]
    pub fn is_jsx_text(&self) -> bool {
        matches!(self, Self::JSXText(_))
    }
    #[inline]
    pub fn is_jsx_expr_container(&self) -> bool {
        matches!(self, Self::JSXExprContainer(_))
    }
    #[inline]
    pub fn is_jsx_spread_child(&self) -> bool {
        matches!(self, Self::JSXSpreadChild(_))
    }
    #[inline]
    pub fn is_jsx_element(&self) -> bool {
        matches!(self, Self::JSXElement(_))
    }
    #[inline]
    pub fn is_jsx_fragment(&self) -> bool {
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
    pub fn opening(&self, ast: &crate::Ast) -> JSXOpeningFragment {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn children(&self, ast: &crate::Ast) -> TypedSubRange<JSXElementChild> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn closing(&self, ast: &crate::Ast) -> JSXClosingFragment {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_opening(&self, ast: &mut crate::Ast, opening: JSXOpeningFragment) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = opening.to_extra_data();
        };
    }
    #[inline]
    pub fn set_children(&self, ast: &mut crate::Ast, children: TypedSubRange<JSXElementChild>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = children.to_extra_data();
        };
    }
    #[inline]
    pub fn set_closing(&self, ast: &mut crate::Ast, closing: JSXClosingFragment) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.get_node_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index()) = closing.to_extra_data();
        };
    }
}
impl JSXOpeningFragment {}
impl JSXClosingFragment {}
