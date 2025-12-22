#![allow(
    unused,
    clippy::useless_conversion,
    clippy::identity_op,
    clippy::erasing_op,
    clippy::let_and_return
)]
use crate::{ast::*, node_id::*};
impl Program {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Module(it) => it.span(ast),
            Self::Script(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Module(it) => it.set_span(ast, span),
            Self::Script(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_module(&self) -> bool {
        matches!(self, Self::Module(_))
    }
    #[inline]
    pub fn is_script(&self) -> bool {
        matches!(self, Self::Script(_))
    }
    #[inline]
    pub fn as_module(&self) -> Option<&Module> {
        match self {
            Self::Module(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_script(&self) -> Option<&Script> {
        match self {
            Self::Script(it) => Some(it),
            _ => None,
        }
    }
}
impl Module {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> TypedSubRange<ModuleItem> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn shebang(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: TypedSubRange<ModuleItem>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = body.into()
        };
    }
    #[inline]
    pub fn set_shebang(&self, ast: &mut crate::Ast, shebang: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_utf8 = shebang.into()
        };
    }
}
impl Script {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> TypedSubRange<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn shebang(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: TypedSubRange<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = body.into()
        };
    }
    #[inline]
    pub fn set_shebang(&self, ast: &mut crate::Ast, shebang: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_utf8 = shebang.into()
        };
    }
}
impl ModuleItem {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::ModuleDecl(it) => it.span(ast),
            Self::Stmt(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::ModuleDecl(it) => it.set_span(ast, span),
            Self::Stmt(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_module_decl(&self) -> bool {
        matches!(self, Self::ModuleDecl(_))
    }
    #[inline]
    pub fn is_stmt(&self) -> bool {
        matches!(self, Self::Stmt(_))
    }
    #[inline]
    pub fn as_module_decl(&self) -> Option<&ModuleDecl> {
        match self {
            Self::ModuleDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_stmt(&self) -> Option<&Stmt> {
        match self {
            Self::Stmt(it) => Some(it),
            _ => None,
        }
    }
}
impl ModuleDecl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Import(it) => it.span(ast),
            Self::ExportDecl(it) => it.span(ast),
            Self::ExportNamed(it) => it.span(ast),
            Self::ExportDefaultDecl(it) => it.span(ast),
            Self::ExportDefaultExpr(it) => it.span(ast),
            Self::ExportAll(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Import(it) => it.set_span(ast, span),
            Self::ExportDecl(it) => it.set_span(ast, span),
            Self::ExportNamed(it) => it.set_span(ast, span),
            Self::ExportDefaultDecl(it) => it.set_span(ast, span),
            Self::ExportDefaultExpr(it) => it.set_span(ast, span),
            Self::ExportAll(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_import(&self) -> Option<&ImportDecl> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_decl(&self) -> Option<&ExportDecl> {
        match self {
            Self::ExportDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_named(&self) -> Option<&NamedExport> {
        match self {
            Self::ExportNamed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_default_decl(&self) -> Option<&ExportDefaultDecl> {
        match self {
            Self::ExportDefaultDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_default_expr(&self) -> Option<&ExportDefaultExpr> {
        match self {
            Self::ExportDefaultExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_all(&self) -> Option<&ExportAll> {
        match self {
            Self::ExportAll(it) => Some(it),
            _ => None,
        }
    }
}
impl ImportDecl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn specifiers(&self, ast: &crate::Ast) -> TypedSubRange<ImportSpecifier> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn src(&self, ast: &crate::Ast) -> Str {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Str::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn type_only(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn with(&self, ast: &crate::Ast) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { ObjectLit::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn phase(&self, ast: &crate::Ast) -> ImportPhase {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .other
        };
        ImportPhase::from_extra_data(ret)
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_specifiers(&self, ast: &mut crate::Ast, specifiers: TypedSubRange<ImportSpecifier>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = specifiers.into()
        };
    }
    #[inline]
    pub fn set_src(&self, ast: &mut crate::Ast, src: Str) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = src.node_id().into()
        };
    }
    #[inline]
    pub fn set_type_only(&self, ast: &mut crate::Ast, type_only: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = type_only.into()
        };
    }
    #[inline]
    pub fn set_with(&self, ast: &mut crate::Ast, with: Option<ObjectLit>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = with.map(|n| n.node_id()).into()
        };
    }
    #[inline]
    pub fn set_phase(&self, ast: &mut crate::Ast, phase: ImportPhase) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .other = phase.to_extra_data()
        };
    }
}
impl ImportSpecifier {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Named(it) => it.span(ast),
            Self::Default(it) => it.span(ast),
            Self::Namespace(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Named(it) => it.set_span(ast, span),
            Self::Default(it) => it.set_span(ast, span),
            Self::Namespace(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_named(&self) -> Option<&ImportNamedSpecifier> {
        match self {
            Self::Named(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_default(&self) -> Option<&ImportDefaultSpecifier> {
        match self {
            Self::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_namespace(&self) -> Option<&ImportStarAsSpecifier> {
        match self {
            Self::Namespace(it) => Some(it),
            _ => None,
        }
    }
}
impl ImportNamedSpecifier {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn is_type_only(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn local(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Ident::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn imported(&self, ast: &crate::Ast) -> Option<ModuleExportName> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_module_export_name
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_is_type_only(&self, ast: &mut crate::Ast, is_type_only: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = is_type_only as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_local(&self, ast: &mut crate::Ast, local: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = local.node_id().into()
        };
    }
    #[inline]
    pub fn set_imported(&self, ast: &mut crate::Ast, imported: Option<ModuleExportName>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_module_export_name = imported
        };
    }
}
impl ImportDefaultSpecifier {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn local(&self, ast: &crate::Ast) -> Ident {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_local(&self, ast: &mut crate::Ast, local: Ident) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = local.node_id().index() as u32;
    }
}
impl ImportStarAsSpecifier {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn local(&self, ast: &crate::Ast) -> Ident {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_local(&self, ast: &mut crate::Ast, local: Ident) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = local.node_id().index() as u32;
    }
}
impl ExportDecl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn decl(&self, ast: &crate::Ast) -> Decl {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .decl
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_decl(&self, ast: &mut crate::Ast, decl: Decl) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .decl = decl
        };
    }
}
impl NamedExport {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn specifiers(&self, ast: &crate::Ast) -> TypedSubRange<ExportSpecifier> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn src(&self, ast: &crate::Ast) -> Option<Str> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { Str::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn type_only(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn with(&self, ast: &crate::Ast) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { ObjectLit::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_specifiers(&self, ast: &mut crate::Ast, specifiers: TypedSubRange<ExportSpecifier>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = specifiers.into()
        };
    }
    #[inline]
    pub fn set_src(&self, ast: &mut crate::Ast, src: Option<Str>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = src.map(|n| n.node_id()).into()
        };
    }
    #[inline]
    pub fn set_type_only(&self, ast: &mut crate::Ast, type_only: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = type_only.into()
        };
    }
    #[inline]
    pub fn set_with(&self, ast: &mut crate::Ast, with: Option<ObjectLit>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = with.map(|n| n.node_id()).into()
        };
    }
}
impl ExportSpecifier {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Namespace(it) => it.span(ast),
            Self::Default(it) => it.span(ast),
            Self::Named(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Namespace(it) => it.set_span(ast, span),
            Self::Default(it) => it.set_span(ast, span),
            Self::Named(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_namespace(&self) -> Option<&ExportNamespaceSpecifier> {
        match self {
            Self::Namespace(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_default(&self) -> Option<&ExportDefaultSpecifier> {
        match self {
            Self::Default(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_named(&self) -> Option<&ExportNamedSpecifier> {
        match self {
            Self::Named(it) => Some(it),
            _ => None,
        }
    }
}
impl ExportNamespaceSpecifier {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> ModuleExportName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .module_export_name
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: ModuleExportName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .module_export_name = name
        };
    }
}
impl ModuleExportName {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Ident(it) => it.span(ast),
            Self::Str(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Ident(it) => it.set_span(ast, span),
            Self::Str(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(&self) -> Option<&Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
}
impl ExportDefaultSpecifier {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn exported(&self, ast: &crate::Ast) -> Ident {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_exported(&self, ast: &mut crate::Ast, exported: Ident) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = exported.node_id().index() as u32;
    }
}
impl ExportNamedSpecifier {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn is_type_only(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn orig(&self, ast: &crate::Ast) -> ModuleExportName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .module_export_name
        };
        ret
    }
    #[inline]
    pub fn exported(&self, ast: &crate::Ast) -> Option<ModuleExportName> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_module_export_name
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_is_type_only(&self, ast: &mut crate::Ast, is_type_only: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = is_type_only as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_orig(&self, ast: &mut crate::Ast, orig: ModuleExportName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .module_export_name = orig
        };
    }
    #[inline]
    pub fn set_exported(&self, ast: &mut crate::Ast, exported: Option<ModuleExportName>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_module_export_name = exported
        };
    }
}
impl ExportDefaultDecl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn decl(&self, ast: &crate::Ast) -> DefaultDecl {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .default_decl
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_decl(&self, ast: &mut crate::Ast, decl: DefaultDecl) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .default_decl = decl
        };
    }
}
impl DefaultDecl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Class(it) => it.span(ast),
            Self::Fn(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Class(it) => it.set_span(ast, span),
            Self::Fn(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }
    #[inline]
    pub fn is_fn(&self) -> bool {
        matches!(self, Self::Fn(_))
    }
    #[inline]
    pub fn as_class(&self) -> Option<&ClassExpr> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(&self) -> Option<&FnExpr> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
}
impl ExportDefaultExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = expr
        };
    }
}
impl ExportAll {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn type_only(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn src(&self, ast: &crate::Ast) -> Str {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Str::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn with(&self, ast: &crate::Ast) -> Option<ObjectLit> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { ObjectLit::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_type_only(&self, ast: &mut crate::Ast, type_only: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = type_only as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_src(&self, ast: &mut crate::Ast, src: Str) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = src.node_id().into()
        };
    }
    #[inline]
    pub fn set_with(&self, ast: &mut crate::Ast, with: Option<ObjectLit>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = with.map(|n| n.node_id()).into()
        };
    }
}
impl BlockStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn stmts(&self, ast: &crate::Ast) -> TypedSubRange<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_stmts(&self, ast: &mut crate::Ast, stmts: TypedSubRange<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = stmts.into()
        };
    }
}
impl Stmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Block(it) => it.span(ast),
            Self::Empty(it) => it.span(ast),
            Self::Debugger(it) => it.span(ast),
            Self::With(it) => it.span(ast),
            Self::Return(it) => it.span(ast),
            Self::Labeled(it) => it.span(ast),
            Self::Break(it) => it.span(ast),
            Self::Continue(it) => it.span(ast),
            Self::If(it) => it.span(ast),
            Self::Switch(it) => it.span(ast),
            Self::Throw(it) => it.span(ast),
            Self::Try(it) => it.span(ast),
            Self::While(it) => it.span(ast),
            Self::DoWhile(it) => it.span(ast),
            Self::For(it) => it.span(ast),
            Self::ForIn(it) => it.span(ast),
            Self::ForOf(it) => it.span(ast),
            Self::Decl(it) => it.span(ast),
            Self::Expr(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Block(it) => it.set_span(ast, span),
            Self::Empty(it) => it.set_span(ast, span),
            Self::Debugger(it) => it.set_span(ast, span),
            Self::With(it) => it.set_span(ast, span),
            Self::Return(it) => it.set_span(ast, span),
            Self::Labeled(it) => it.set_span(ast, span),
            Self::Break(it) => it.set_span(ast, span),
            Self::Continue(it) => it.set_span(ast, span),
            Self::If(it) => it.set_span(ast, span),
            Self::Switch(it) => it.set_span(ast, span),
            Self::Throw(it) => it.set_span(ast, span),
            Self::Try(it) => it.set_span(ast, span),
            Self::While(it) => it.set_span(ast, span),
            Self::DoWhile(it) => it.set_span(ast, span),
            Self::For(it) => it.set_span(ast, span),
            Self::ForIn(it) => it.set_span(ast, span),
            Self::ForOf(it) => it.set_span(ast, span),
            Self::Decl(it) => it.set_span(ast, span),
            Self::Expr(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_block(&self) -> Option<&BlockStmt> {
        match self {
            Self::Block(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_empty(&self) -> Option<&EmptyStmt> {
        match self {
            Self::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_debugger(&self) -> Option<&DebuggerStmt> {
        match self {
            Self::Debugger(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_with(&self) -> Option<&WithStmt> {
        match self {
            Self::With(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_return(&self) -> Option<&ReturnStmt> {
        match self {
            Self::Return(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_labeled(&self) -> Option<&LabeledStmt> {
        match self {
            Self::Labeled(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_break(&self) -> Option<&BreakStmt> {
        match self {
            Self::Break(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_continue(&self) -> Option<&ContinueStmt> {
        match self {
            Self::Continue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_if(&self) -> Option<&IfStmt> {
        match self {
            Self::If(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_switch(&self) -> Option<&SwitchStmt> {
        match self {
            Self::Switch(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_throw(&self) -> Option<&ThrowStmt> {
        match self {
            Self::Throw(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_try(&self) -> Option<&TryStmt> {
        match self {
            Self::Try(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_while(&self) -> Option<&WhileStmt> {
        match self {
            Self::While(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_do_while(&self) -> Option<&DoWhileStmt> {
        match self {
            Self::DoWhile(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for(&self) -> Option<&ForStmt> {
        match self {
            Self::For(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_in(&self) -> Option<&ForInStmt> {
        match self {
            Self::ForIn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_of(&self) -> Option<&ForOfStmt> {
        match self {
            Self::ForOf(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_decl(&self) -> Option<&Decl> {
        match self {
            Self::Decl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&ExprStmt> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl ExprStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = expr
        };
    }
}
impl EmptyStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
impl DebuggerStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
impl WithStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn obj(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .stmt
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_obj(&self, ast: &mut crate::Ast, obj: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = obj
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .stmt = body
        };
    }
}
impl ReturnStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = arg
        };
    }
}
impl LabeledStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn label(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Ident::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .stmt
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_label(&self, ast: &mut crate::Ast, label: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = label.node_id().into()
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .stmt = body
        };
    }
}
impl BreakStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn label(&self, ast: &crate::Ast) -> Option<Ident> {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_label(&self, ast: &mut crate::Ast, label: Option<Ident>) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = crate::OptionalNodeId::from(label.map(|n| n.node_id())).into_raw();
    }
}
impl ContinueStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn label(&self, ast: &crate::Ast) -> Option<Ident> {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        let opt = crate::OptionalNodeId::from_raw(raw);
        opt.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_label(&self, ast: &mut crate::Ast, label: Option<Ident>) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = crate::OptionalNodeId::from(label.map(|n| n.node_id())).into_raw();
    }
}
impl IfStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn cons(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .stmt
        };
        ret
    }
    #[inline]
    pub fn alt(&self, ast: &crate::Ast) -> Option<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_stmt
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = test
        };
    }
    #[inline]
    pub fn set_cons(&self, ast: &mut crate::Ast, cons: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .stmt = cons
        };
    }
    #[inline]
    pub fn set_alt(&self, ast: &mut crate::Ast, alt: Option<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_stmt = alt
        };
    }
}
impl SwitchStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn discriminant(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn cases(&self, ast: &crate::Ast) -> TypedSubRange<SwitchCase> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_discriminant(&self, ast: &mut crate::Ast, discriminant: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = discriminant
        };
    }
    #[inline]
    pub fn set_cases(&self, ast: &mut crate::Ast, cases: TypedSubRange<SwitchCase>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = cases.into()
        };
    }
}
impl ThrowStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = arg
        };
    }
}
impl TryStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn block(&self, ast: &crate::Ast) -> BlockStmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { BlockStmt::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn handler(&self, ast: &crate::Ast) -> Option<CatchClause> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { CatchClause::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn finalizer(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { BlockStmt::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_block(&self, ast: &mut crate::Ast, block: BlockStmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = block.node_id().into()
        };
    }
    #[inline]
    pub fn set_handler(&self, ast: &mut crate::Ast, handler: Option<CatchClause>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = handler.map(|n| n.node_id()).into()
        };
    }
    #[inline]
    pub fn set_finalizer(&self, ast: &mut crate::Ast, finalizer: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = finalizer.map(|n| n.node_id()).into()
        };
    }
}
impl WhileStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .stmt
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = test
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .stmt = body
        };
    }
}
impl DoWhileStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .stmt
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = test
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .stmt = body
        };
    }
}
impl ForStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn init(&self, ast: &crate::Ast) -> Option<VarDeclOrExpr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_var_decl_or_expr
        };
        ret
    }
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn update(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .stmt
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_init(&self, ast: &mut crate::Ast, init: Option<VarDeclOrExpr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_var_decl_or_expr = init
        };
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = test
        };
    }
    #[inline]
    pub fn set_update(&self, ast: &mut crate::Ast, update: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = update
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .stmt = body
        };
    }
}
impl ForInStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> ForHead {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .for_head
        };
        ret
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .stmt
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: ForHead) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .for_head = left
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = right
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .stmt = body
        };
    }
}
impl ForOfStmt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn is_await(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> ForHead {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .for_head
        };
        ret
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Stmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .stmt
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_is_await(&self, ast: &mut crate::Ast, is_await: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = is_await as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: ForHead) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .for_head = left
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = right
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Stmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .stmt = body
        };
    }
}
impl SwitchCase {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn cons(&self, ast: &crate::Ast) -> TypedSubRange<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = test
        };
    }
    #[inline]
    pub fn set_cons(&self, ast: &mut crate::Ast, cons: TypedSubRange<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = cons.into()
        };
    }
}
impl CatchClause {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn param(&self, ast: &crate::Ast) -> Option<Pat> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_pat
        };
        ret
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> BlockStmt {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { BlockStmt::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_param(&self, ast: &mut crate::Ast, param: Option<Pat>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_pat = param
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: BlockStmt) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = body.node_id().into()
        };
    }
}
impl ForHead {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::VarDecl(it) => it.span(ast),
            Self::UsingDecl(it) => it.span(ast),
            Self::Pat(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::VarDecl(it) => it.set_span(ast, span),
            Self::UsingDecl(it) => it.set_span(ast, span),
            Self::Pat(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_var_decl(&self) -> Option<&VarDecl> {
        match self {
            Self::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_using_decl(&self) -> Option<&UsingDecl> {
        match self {
            Self::UsingDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pat(&self) -> Option<&Pat> {
        match self {
            Self::Pat(it) => Some(it),
            _ => None,
        }
    }
}
impl VarDeclOrExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::VarDecl(it) => it.span(ast),
            Self::Expr(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::VarDecl(it) => it.set_span(ast, span),
            Self::Expr(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_var_decl(&self) -> bool {
        matches!(self, Self::VarDecl(_))
    }
    #[inline]
    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(_))
    }
    #[inline]
    pub fn as_var_decl(&self) -> Option<&VarDecl> {
        match self {
            Self::VarDecl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl Decl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Class(it) => it.span(ast),
            Self::Fn(it) => it.span(ast),
            Self::Var(it) => it.span(ast),
            Self::Using(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Class(it) => it.set_span(ast, span),
            Self::Fn(it) => it.set_span(ast, span),
            Self::Var(it) => it.set_span(ast, span),
            Self::Using(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_class(&self) -> Option<&ClassDecl> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(&self) -> Option<&FnDecl> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_var(&self) -> Option<&VarDecl> {
        match self {
            Self::Var(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_using(&self) -> Option<&UsingDecl> {
        match self {
            Self::Using(it) => Some(it),
            _ => None,
        }
    }
}
impl FnDecl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn declare(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn ident(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Ident::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Function::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_declare(&self, ast: &mut crate::Ast, declare: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = declare as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_ident(&self, ast: &mut crate::Ast, ident: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = ident.node_id().into()
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = function.node_id().into()
        };
    }
}
impl ClassDecl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn declare(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn ident(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Ident::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn class(&self, ast: &crate::Ast) -> Class {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Class::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_declare(&self, ast: &mut crate::Ast, declare: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = declare as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_ident(&self, ast: &mut crate::Ast, ident: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = ident.node_id().into()
        };
    }
    #[inline]
    pub fn set_class(&self, ast: &mut crate::Ast, class: Class) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = class.node_id().into()
        };
    }
}
impl VarDecl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn kind(&self, ast: &crate::Ast) -> VarDeclKind {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .other
        };
        VarDeclKind::from_extra_data(ret)
    }
    #[inline]
    pub fn declare(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn decls(&self, ast: &crate::Ast) -> TypedSubRange<VarDeclarator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_kind(&self, ast: &mut crate::Ast, kind: VarDeclKind) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .other = kind.to_extra_data()
        };
    }
    #[inline]
    pub fn set_declare(&self, ast: &mut crate::Ast, declare: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = declare.into()
        };
    }
    #[inline]
    pub fn set_decls(&self, ast: &mut crate::Ast, decls: TypedSubRange<VarDeclarator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = decls.into()
        };
    }
}
impl VarDeclarator {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .pat
        };
        ret
    }
    #[inline]
    pub fn init(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .pat = name
        };
    }
    #[inline]
    pub fn set_init(&self, ast: &mut crate::Ast, init: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = init
        };
    }
}
impl UsingDecl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn is_await(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn decls(&self, ast: &crate::Ast) -> TypedSubRange<VarDeclarator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_is_await(&self, ast: &mut crate::Ast, is_await: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = is_await.into()
        };
    }
    #[inline]
    pub fn set_decls(&self, ast: &mut crate::Ast, decls: TypedSubRange<VarDeclarator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = decls.into()
        };
    }
}
impl Expr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::This(it) => it.span(ast),
            Self::Array(it) => it.span(ast),
            Self::Object(it) => it.span(ast),
            Self::Fn(it) => it.span(ast),
            Self::Unary(it) => it.span(ast),
            Self::Update(it) => it.span(ast),
            Self::Bin(it) => it.span(ast),
            Self::Assign(it) => it.span(ast),
            Self::Member(it) => it.span(ast),
            Self::SuperProp(it) => it.span(ast),
            Self::Cond(it) => it.span(ast),
            Self::Call(it) => it.span(ast),
            Self::New(it) => it.span(ast),
            Self::Seq(it) => it.span(ast),
            Self::Ident(it) => it.span(ast),
            Self::Lit(it) => it.span(ast),
            Self::Tpl(it) => it.span(ast),
            Self::TaggedTpl(it) => it.span(ast),
            Self::Arrow(it) => it.span(ast),
            Self::Class(it) => it.span(ast),
            Self::Yield(it) => it.span(ast),
            Self::MetaProp(it) => it.span(ast),
            Self::Await(it) => it.span(ast),
            Self::Paren(it) => it.span(ast),
            Self::JSXMember(it) => it.span(ast),
            Self::JSXNamespacedName(it) => it.span(ast),
            Self::JSXEmpty(it) => it.span(ast),
            Self::JSXElement(it) => it.span(ast),
            Self::JSXFragment(it) => it.span(ast),
            Self::PrivateName(it) => it.span(ast),
            Self::OptChain(it) => it.span(ast),
            Self::Invalid(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::This(it) => it.set_span(ast, span),
            Self::Array(it) => it.set_span(ast, span),
            Self::Object(it) => it.set_span(ast, span),
            Self::Fn(it) => it.set_span(ast, span),
            Self::Unary(it) => it.set_span(ast, span),
            Self::Update(it) => it.set_span(ast, span),
            Self::Bin(it) => it.set_span(ast, span),
            Self::Assign(it) => it.set_span(ast, span),
            Self::Member(it) => it.set_span(ast, span),
            Self::SuperProp(it) => it.set_span(ast, span),
            Self::Cond(it) => it.set_span(ast, span),
            Self::Call(it) => it.set_span(ast, span),
            Self::New(it) => it.set_span(ast, span),
            Self::Seq(it) => it.set_span(ast, span),
            Self::Ident(it) => it.set_span(ast, span),
            Self::Lit(it) => it.set_span(ast, span),
            Self::Tpl(it) => it.set_span(ast, span),
            Self::TaggedTpl(it) => it.set_span(ast, span),
            Self::Arrow(it) => it.set_span(ast, span),
            Self::Class(it) => it.set_span(ast, span),
            Self::Yield(it) => it.set_span(ast, span),
            Self::MetaProp(it) => it.set_span(ast, span),
            Self::Await(it) => it.set_span(ast, span),
            Self::Paren(it) => it.set_span(ast, span),
            Self::JSXMember(it) => it.set_span(ast, span),
            Self::JSXNamespacedName(it) => it.set_span(ast, span),
            Self::JSXEmpty(it) => it.set_span(ast, span),
            Self::JSXElement(it) => it.set_span(ast, span),
            Self::JSXFragment(it) => it.set_span(ast, span),
            Self::PrivateName(it) => it.set_span(ast, span),
            Self::OptChain(it) => it.set_span(ast, span),
            Self::Invalid(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_this(&self) -> Option<&ThisExpr> {
        match self {
            Self::This(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(&self) -> Option<&ArrayLit> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(&self) -> Option<&ObjectLit> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_fn(&self) -> Option<&FnExpr> {
        match self {
            Self::Fn(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary(&self) -> Option<&UnaryExpr> {
        match self {
            Self::Unary(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_update(&self) -> Option<&UpdateExpr> {
        match self {
            Self::Update(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bin(&self) -> Option<&BinExpr> {
        match self {
            Self::Bin(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(&self) -> Option<&AssignExpr> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(&self) -> Option<&MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(&self) -> Option<&SuperPropExpr> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_cond(&self) -> Option<&CondExpr> {
        match self {
            Self::Cond(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(&self) -> Option<&CallExpr> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_new(&self) -> Option<&NewExpr> {
        match self {
            Self::New(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_seq(&self) -> Option<&SeqExpr> {
        match self {
            Self::Seq(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_lit(&self) -> Option<&Lit> {
        match self {
            Self::Lit(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tpl(&self) -> Option<&Tpl> {
        match self {
            Self::Tpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tagged_tpl(&self) -> Option<&TaggedTpl> {
        match self {
            Self::TaggedTpl(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_arrow(&self) -> Option<&ArrowExpr> {
        match self {
            Self::Arrow(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(&self) -> Option<&ClassExpr> {
        match self {
            Self::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_yield(&self) -> Option<&YieldExpr> {
        match self {
            Self::Yield(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_meta_prop(&self) -> Option<&MetaPropExpr> {
        match self {
            Self::MetaProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_await(&self) -> Option<&AwaitExpr> {
        match self {
            Self::Await(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(&self) -> Option<&ParenExpr> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member(&self) -> Option<&JSXMemberExpr> {
        match self {
            Self::JSXMember(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(&self) -> Option<&JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty(&self) -> Option<&JSXEmptyExpr> {
        match self {
            Self::JSXEmpty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(&self) -> Option<&JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(&self) -> Option<&JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(&self) -> Option<&PrivateName> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(&self) -> Option<&OptChainExpr> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(&self) -> Option<&Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl ThisExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
impl ArrayLit {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn elems(&self, ast: &crate::Ast) -> TypedSubRange<Option<ExprOrSpread>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_elems(&self, ast: &mut crate::Ast, elems: TypedSubRange<Option<ExprOrSpread>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = elems.into()
        };
    }
}
impl ObjectLit {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn props(&self, ast: &crate::Ast) -> TypedSubRange<PropOrSpread> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_props(&self, ast: &mut crate::Ast, props: TypedSubRange<PropOrSpread>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = props.into()
        };
    }
}
impl PropOrSpread {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::SpreadElement(it) => it.span(ast),
            Self::Prop(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::SpreadElement(it) => it.set_span(ast, span),
            Self::Prop(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_spread_element(&self) -> bool {
        matches!(self, Self::SpreadElement(_))
    }
    #[inline]
    pub fn is_prop(&self) -> bool {
        matches!(self, Self::Prop(_))
    }
    #[inline]
    pub fn as_spread_element(&self) -> Option<&SpreadElement> {
        match self {
            Self::SpreadElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_prop(&self) -> Option<&Prop> {
        match self {
            Self::Prop(it) => Some(it),
            _ => None,
        }
    }
}
impl SpreadElement {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn dot_3_token(&self, ast: &crate::Ast) -> Span {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .span
        };
        ret.into()
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_dot3_token(&self, ast: &mut crate::Ast, dot3_token: Span) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .span = dot3_token.into()
        };
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = expr
        };
    }
}
impl UnaryExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn op(&self, ast: &crate::Ast) -> UnaryOp {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, UnaryOp>(raw as u8) }
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_op(&self, ast: &mut crate::Ast, op: UnaryOp) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = op as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = arg
        };
    }
}
impl UpdateExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn op(&self, ast: &crate::Ast) -> UpdateOp {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, UpdateOp>(raw as u8) }
    }
    #[inline]
    pub fn prefix(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = (u32::from(node.inline_data) >> 8usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_op(&self, ast: &mut crate::Ast, op: UpdateOp) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = op as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_prefix(&self, ast: &mut crate::Ast, prefix: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = prefix as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16711935u32) | ((field_val & 255u32) << 8usize)).into();
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = arg
        };
    }
}
impl BinExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn op(&self, ast: &crate::Ast) -> BinaryOp {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, BinaryOp>(raw as u8) }
    }
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_op(&self, ast: &mut crate::Ast, op: BinaryOp) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = op as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = left
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = right
        };
    }
}
impl FnExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn ident(&self, ast: &crate::Ast) -> Option<Ident> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Function::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_ident(&self, ast: &mut crate::Ast, ident: Option<Ident>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = ident.map(|n| n.node_id()).into()
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = function.node_id().into()
        };
    }
}
impl ClassExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn ident(&self, ast: &crate::Ast) -> Option<Ident> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { Ident::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn class(&self, ast: &crate::Ast) -> Class {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Class::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_ident(&self, ast: &mut crate::Ast, ident: Option<Ident>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = ident.map(|n| n.node_id()).into()
        };
    }
    #[inline]
    pub fn set_class(&self, ast: &mut crate::Ast, class: Class) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = class.node_id().into()
        };
    }
}
impl AssignExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn op(&self, ast: &crate::Ast) -> AssignOp {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, AssignOp>(raw as u8) }
    }
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> AssignTarget {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { AssignTarget::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_op(&self, ast: &mut crate::Ast, op: AssignOp) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = op as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: AssignTarget) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = left.node_id().into()
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = right
        };
    }
}
impl MemberExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn obj(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn prop(&self, ast: &crate::Ast) -> MemberProp {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .member_prop
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_obj(&self, ast: &mut crate::Ast, obj: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = obj
        };
    }
    #[inline]
    pub fn set_prop(&self, ast: &mut crate::Ast, prop: MemberProp) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .member_prop = prop
        };
    }
}
impl MemberProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Ident(it) => it.span(ast),
            Self::PrivateName(it) => it.span(ast),
            Self::Computed(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Ident(it) => it.set_span(ast, span),
            Self::PrivateName(it) => it.set_span(ast, span),
            Self::Computed(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_ident(&self) -> Option<&IdentName> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_name(&self) -> Option<&PrivateName> {
        match self {
            Self::PrivateName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(&self) -> Option<&ComputedPropName> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
}
impl SuperPropExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn obj(&self, ast: &crate::Ast) -> Super {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Super::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn prop(&self, ast: &crate::Ast) -> SuperProp {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .super_prop
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_obj(&self, ast: &mut crate::Ast, obj: Super) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = obj.node_id().into()
        };
    }
    #[inline]
    pub fn set_prop(&self, ast: &mut crate::Ast, prop: SuperProp) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .super_prop = prop
        };
    }
}
impl SuperProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Ident(it) => it.span(ast),
            Self::Computed(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Ident(it) => it.set_span(ast, span),
            Self::Computed(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_computed(&self) -> bool {
        matches!(self, Self::Computed(_))
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&IdentName> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(&self) -> Option<&ComputedPropName> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
}
impl CondExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn test(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn cons(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn alt(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_test(&self, ast: &mut crate::Ast, test: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = test
        };
    }
    #[inline]
    pub fn set_cons(&self, ast: &mut crate::Ast, cons: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = cons
        };
    }
    #[inline]
    pub fn set_alt(&self, ast: &mut crate::Ast, alt: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = alt
        };
    }
}
impl CallExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn callee(&self, ast: &crate::Ast) -> Callee {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .callee
        };
        ret
    }
    #[inline]
    pub fn args(&self, ast: &crate::Ast) -> TypedSubRange<ExprOrSpread> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_callee(&self, ast: &mut crate::Ast, callee: Callee) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .callee = callee
        };
    }
    #[inline]
    pub fn set_args(&self, ast: &mut crate::Ast, args: TypedSubRange<ExprOrSpread>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = args.into()
        };
    }
}
impl NewExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn callee(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn args(&self, ast: &crate::Ast) -> Option<TypedSubRange<ExprOrSpread>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_sub_range
        };
        unsafe { ret.cast_to_typed().to_option() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_callee(&self, ast: &mut crate::Ast, callee: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = callee
        };
    }
    #[inline]
    pub fn set_args(&self, ast: &mut crate::Ast, args: Option<TypedSubRange<ExprOrSpread>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_sub_range = args.map(|n| n.inner).into()
        };
    }
}
impl SeqExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn exprs(&self, ast: &crate::Ast) -> TypedSubRange<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_exprs(&self, ast: &mut crate::Ast, exprs: TypedSubRange<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = exprs.into()
        };
    }
}
impl ArrowExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn params(&self, ast: &crate::Ast) -> TypedSubRange<Pat> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> BlockStmtOrExpr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .block_stmt_or_expr
        };
        ret
    }
    #[inline]
    pub fn is_async(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn is_generator(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_params(&self, ast: &mut crate::Ast, params: TypedSubRange<Pat>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = params.into()
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: BlockStmtOrExpr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .block_stmt_or_expr = body
        };
    }
    #[inline]
    pub fn set_is_async(&self, ast: &mut crate::Ast, is_async: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = is_async.into()
        };
    }
    #[inline]
    pub fn set_is_generator(&self, ast: &mut crate::Ast, is_generator: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = is_generator.into()
        };
    }
}
impl YieldExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn delegate(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_delegate(&self, ast: &mut crate::Ast, delegate: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = delegate as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = arg
        };
    }
}
impl MetaPropExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn kind(&self, ast: &crate::Ast) -> MetaPropKind {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { std::mem::transmute::<u8, MetaPropKind>(raw as u8) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_kind(&self, ast: &mut crate::Ast, kind: MetaPropKind) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = kind as u32;
    }
}
impl AwaitExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = arg
        };
    }
}
impl Tpl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn exprs(&self, ast: &crate::Ast) -> TypedSubRange<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn quasis(&self, ast: &crate::Ast) -> TypedSubRange<TplElement> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_exprs(&self, ast: &mut crate::Ast, exprs: TypedSubRange<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = exprs.into()
        };
    }
    #[inline]
    pub fn set_quasis(&self, ast: &mut crate::Ast, quasis: TypedSubRange<TplElement>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = quasis.into()
        };
    }
}
impl TaggedTpl {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn tag(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn tpl(&self, ast: &crate::Ast) -> Tpl {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Tpl::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_tag(&self, ast: &mut crate::Ast, tag: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = tag
        };
    }
    #[inline]
    pub fn set_tpl(&self, ast: &mut crate::Ast, tpl: Tpl) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = tpl.node_id().into()
        };
    }
}
impl TplElement {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn tail(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn cooked(&self, ast: &crate::Ast) -> OptionalWtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_wtf8
        };
        ret.into()
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_tail(&self, ast: &mut crate::Ast, tail: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = tail as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_cooked(&self, ast: &mut crate::Ast, cooked: OptionalWtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_wtf8 = cooked.into()
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .utf8 = raw.into()
        };
    }
}
impl ParenExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = expr
        };
    }
}
impl Callee {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Super(it) => it.span(ast),
            Self::Import(it) => it.span(ast),
            Self::Expr(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Super(it) => it.set_span(ast, span),
            Self::Import(it) => it.set_span(ast, span),
            Self::Expr(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_super(&self) -> Option<&Super> {
        match self {
            Self::Super(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import(&self) -> Option<&Import> {
        match self {
            Self::Import(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl Super {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
impl Import {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn phase(&self, ast: &crate::Ast) -> ImportPhase {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { std::mem::transmute::<u8, ImportPhase>(raw as u8) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_phase(&self, ast: &mut crate::Ast, phase: ImportPhase) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = phase as u32;
    }
}
impl ExprOrSpread {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn spread(&self, ast: &crate::Ast) -> Option<SpreadDot3Token> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { SpreadDot3Token::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_spread(&self, ast: &mut crate::Ast, spread: Option<SpreadDot3Token>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = spread.map(|n| n.node_id()).into()
        };
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = expr
        };
    }
}
impl SpreadDot3Token {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
impl BlockStmtOrExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::BlockStmt(it) => it.span(ast),
            Self::Expr(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::BlockStmt(it) => it.set_span(ast, span),
            Self::Expr(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_block_stmt(&self) -> bool {
        matches!(self, Self::BlockStmt(_))
    }
    #[inline]
    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(_))
    }
    #[inline]
    pub fn as_block_stmt(&self) -> Option<&BlockStmt> {
        match self {
            Self::BlockStmt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl AssignTarget {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Simple(it) => it.span(ast),
            Self::Pat(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Simple(it) => it.set_span(ast, span),
            Self::Pat(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_simple(&self) -> bool {
        matches!(self, Self::Simple(_))
    }
    #[inline]
    pub fn is_pat(&self) -> bool {
        matches!(self, Self::Pat(_))
    }
    #[inline]
    pub fn as_simple(&self) -> Option<&SimpleAssignTarget> {
        match self {
            Self::Simple(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_pat(&self) -> Option<&AssignTargetPat> {
        match self {
            Self::Pat(it) => Some(it),
            _ => None,
        }
    }
}
impl AssignTargetPat {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Array(it) => it.span(ast),
            Self::Object(it) => it.span(ast),
            Self::Invalid(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Array(it) => it.set_span(ast, span),
            Self::Object(it) => it.set_span(ast, span),
            Self::Invalid(it) => it.set_span(ast, span),
        }
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
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }
    #[inline]
    pub fn as_array(&self) -> Option<&ArrayPat> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(&self) -> Option<&ObjectPat> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(&self) -> Option<&Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl SimpleAssignTarget {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Ident(it) => it.span(ast),
            Self::Member(it) => it.span(ast),
            Self::SuperProp(it) => it.span(ast),
            Self::Paren(it) => it.span(ast),
            Self::OptChain(it) => it.span(ast),
            Self::Invalid(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Ident(it) => it.set_span(ast, span),
            Self::Member(it) => it.set_span(ast, span),
            Self::SuperProp(it) => it.set_span(ast, span),
            Self::Paren(it) => it.set_span(ast, span),
            Self::OptChain(it) => it.set_span(ast, span),
            Self::Invalid(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_ident(&self) -> Option<&BindingIdent> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member(&self) -> Option<&MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super_prop(&self) -> Option<&SuperPropExpr> {
        match self {
            Self::SuperProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_paren(&self) -> Option<&ParenExpr> {
        match self {
            Self::Paren(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_opt_chain(&self) -> Option<&OptChainExpr> {
        match self {
            Self::OptChain(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(&self) -> Option<&Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
}
impl OptChainExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn optional(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn base(&self, ast: &crate::Ast) -> OptChainBase {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .opt_chain_base
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_optional(&self, ast: &mut crate::Ast, optional: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = optional as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_base(&self, ast: &mut crate::Ast, base: OptChainBase) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .opt_chain_base = base
        };
    }
}
impl OptChainBase {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Member(it) => it.span(ast),
            Self::Call(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Member(it) => it.set_span(ast, span),
            Self::Call(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_member(&self) -> bool {
        matches!(self, Self::Member(_))
    }
    #[inline]
    pub fn is_call(&self) -> bool {
        matches!(self, Self::Call(_))
    }
    #[inline]
    pub fn as_member(&self) -> Option<&MemberExpr> {
        match self {
            Self::Member(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call(&self) -> Option<&OptCall> {
        match self {
            Self::Call(it) => Some(it),
            _ => None,
        }
    }
}
impl OptCall {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn callee(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn args(&self, ast: &crate::Ast) -> TypedSubRange<ExprOrSpread> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_callee(&self, ast: &mut crate::Ast, callee: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = callee
        };
    }
    #[inline]
    pub fn set_args(&self, ast: &mut crate::Ast, args: TypedSubRange<ExprOrSpread>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = args.into()
        };
    }
}
impl Invalid {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
impl Function {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn params(&self, ast: &crate::Ast) -> TypedSubRange<Param> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { BlockStmt::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn is_generator(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn is_async(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_params(&self, ast: &mut crate::Ast, params: TypedSubRange<Param>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = params.into()
        };
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = decorators.into()
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = body.map(|n| n.node_id()).into()
        };
    }
    #[inline]
    pub fn set_is_generator(&self, ast: &mut crate::Ast, is_generator: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = is_generator.into()
        };
    }
    #[inline]
    pub fn set_is_async(&self, ast: &mut crate::Ast, is_async: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = is_async.into()
        };
    }
}
impl Param {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn pat(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .pat
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = decorators.into()
        };
    }
    #[inline]
    pub fn set_pat(&self, ast: &mut crate::Ast, pat: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .pat = pat
        };
    }
}
impl ParamOrTsParamProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Param(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Param(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_param(&self) -> bool {
        matches!(self, Self::Param(_))
    }
    #[inline]
    pub fn as_param(&self) -> Option<&Param> {
        match self {
            Self::Param(it) => Some(it),
            _ => None,
        }
    }
}
impl Class {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> TypedSubRange<ClassMember> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn super_class(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn is_abstract(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = decorators.into()
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: TypedSubRange<ClassMember>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = body.into()
        };
    }
    #[inline]
    pub fn set_super_class(&self, ast: &mut crate::Ast, super_class: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = super_class
        };
    }
    #[inline]
    pub fn set_is_abstract(&self, ast: &mut crate::Ast, is_abstract: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = is_abstract.into()
        };
    }
}
impl ClassMember {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Constructor(it) => it.span(ast),
            Self::Method(it) => it.span(ast),
            Self::PrivateMethod(it) => it.span(ast),
            Self::ClassProp(it) => it.span(ast),
            Self::PrivateProp(it) => it.span(ast),
            Self::Empty(it) => it.span(ast),
            Self::StaticBlock(it) => it.span(ast),
            Self::AutoAccessor(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Constructor(it) => it.set_span(ast, span),
            Self::Method(it) => it.set_span(ast, span),
            Self::PrivateMethod(it) => it.set_span(ast, span),
            Self::ClassProp(it) => it.set_span(ast, span),
            Self::PrivateProp(it) => it.set_span(ast, span),
            Self::Empty(it) => it.set_span(ast, span),
            Self::StaticBlock(it) => it.set_span(ast, span),
            Self::AutoAccessor(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_constructor(&self) -> Option<&Constructor> {
        match self {
            Self::Constructor(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_method(&self) -> Option<&ClassMethod> {
        match self {
            Self::Method(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_method(&self) -> Option<&PrivateMethod> {
        match self {
            Self::PrivateMethod(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class_prop(&self) -> Option<&ClassProp> {
        match self {
            Self::ClassProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_prop(&self) -> Option<&PrivateProp> {
        match self {
            Self::PrivateProp(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_empty(&self) -> Option<&EmptyStmt> {
        match self {
            Self::Empty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_static_block(&self) -> Option<&StaticBlock> {
        match self {
            Self::StaticBlock(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_auto_accessor(&self) -> Option<&AutoAccessor> {
        match self {
            Self::AutoAccessor(it) => Some(it),
            _ => None,
        }
    }
}
impl ClassProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .prop_name
        };
        ret
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .prop_name = key
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = value
        };
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = is_static.into()
        };
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = decorators.into()
        };
    }
}
impl PrivateProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PrivateName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { PrivateName::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PrivateName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = key.node_id().into()
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = value
        };
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = is_static.into()
        };
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = decorators.into()
        };
    }
}
impl ClassMethod {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn kind(&self, ast: &crate::Ast) -> MethodKind {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodKind>(raw as u8) }
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = (u32::from(node.inline_data) >> 8usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .prop_name
        };
        ret
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Function::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_kind(&self, ast: &mut crate::Ast, kind: MethodKind) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = kind as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = is_static as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16711935u32) | ((field_val & 255u32) << 8usize)).into();
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .prop_name = key
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = function.node_id().into()
        };
    }
}
impl PrivateMethod {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn kind(&self, ast: &crate::Ast) -> MethodKind {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodKind>(raw as u8) }
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = (u32::from(node.inline_data) >> 8usize) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PrivateName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { PrivateName::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Function::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_kind(&self, ast: &mut crate::Ast, kind: MethodKind) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = kind as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = is_static as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16711935u32) | ((field_val & 255u32) << 8usize)).into();
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PrivateName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = key.node_id().into()
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = function.node_id().into()
        };
    }
}
impl Constructor {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .prop_name
        };
        ret
    }
    #[inline]
    pub fn params(&self, ast: &crate::Ast) -> TypedSubRange<ParamOrTsParamProp> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { BlockStmt::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .prop_name = key
        };
    }
    #[inline]
    pub fn set_params(&self, ast: &mut crate::Ast, params: TypedSubRange<ParamOrTsParamProp>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = params.into()
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = body.map(|n| n.node_id()).into()
        };
    }
}
impl Decorator {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = expr
        };
    }
}
impl StaticBlock {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> BlockStmt {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { BlockStmt::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: BlockStmt) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = body.node_id().index() as u32;
    }
}
impl Key {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Private(it) => it.span(ast),
            Self::Public(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Private(it) => it.set_span(ast, span),
            Self::Public(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private(_))
    }
    #[inline]
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public(_))
    }
    #[inline]
    pub fn as_private(&self) -> Option<&PrivateName> {
        match self {
            Self::Private(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_public(&self) -> Option<&PropName> {
        match self {
            Self::Public(it) => Some(it),
            _ => None,
        }
    }
}
impl AutoAccessor {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> Key {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .key
        };
        ret
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn is_static(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn decorators(&self, ast: &crate::Ast) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: Key) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .key = key
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = value
        };
    }
    #[inline]
    pub fn set_is_static(&self, ast: &mut crate::Ast, is_static: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = is_static.into()
        };
    }
    #[inline]
    pub fn set_decorators(&self, ast: &mut crate::Ast, decorators: TypedSubRange<Decorator>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = decorators.into()
        };
    }
}
impl Prop {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Shorthand(it) => it.span(ast),
            Self::KeyValue(it) => it.span(ast),
            Self::Assign(it) => it.span(ast),
            Self::Getter(it) => it.span(ast),
            Self::Setter(it) => it.span(ast),
            Self::Method(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Shorthand(it) => it.set_span(ast, span),
            Self::KeyValue(it) => it.set_span(ast, span),
            Self::Assign(it) => it.set_span(ast, span),
            Self::Getter(it) => it.set_span(ast, span),
            Self::Setter(it) => it.set_span(ast, span),
            Self::Method(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_shorthand(&self) -> Option<&Ident> {
        match self {
            Self::Shorthand(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_key_value(&self) -> Option<&KeyValueProp> {
        match self {
            Self::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(&self) -> Option<&AssignProp> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_getter(&self) -> Option<&GetterProp> {
        match self {
            Self::Getter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_setter(&self) -> Option<&SetterProp> {
        match self {
            Self::Setter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_method(&self) -> Option<&MethodProp> {
        match self {
            Self::Method(it) => Some(it),
            _ => None,
        }
    }
}
impl KeyValueProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .prop_name
        };
        ret
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .prop_name = key
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = value
        };
    }
}
impl AssignProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> Ident {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Ident::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: Ident) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = key.node_id().into()
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = value
        };
    }
}
impl GetterProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .prop_name
        };
        ret
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { BlockStmt::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .prop_name = key
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = body.map(|n| n.node_id()).into()
        };
    }
}
impl SetterProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .prop_name
        };
        ret
    }
    #[inline]
    pub fn this_param(&self, ast: &crate::Ast) -> Option<Pat> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_pat
        };
        ret
    }
    #[inline]
    pub fn param(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .pat
        };
        ret
    }
    #[inline]
    pub fn body(&self, ast: &crate::Ast) -> Option<BlockStmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { BlockStmt::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .prop_name = key
        };
    }
    #[inline]
    pub fn set_this_param(&self, ast: &mut crate::Ast, this_param: Option<Pat>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_pat = this_param
        };
    }
    #[inline]
    pub fn set_param(&self, ast: &mut crate::Ast, param: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .pat = param
        };
    }
    #[inline]
    pub fn set_body(&self, ast: &mut crate::Ast, body: Option<BlockStmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = body.map(|n| n.node_id()).into()
        };
    }
}
impl MethodProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .prop_name
        };
        ret
    }
    #[inline]
    pub fn function(&self, ast: &crate::Ast) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { Function::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .prop_name = key
        };
    }
    #[inline]
    pub fn set_function(&self, ast: &mut crate::Ast, function: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = function.node_id().into()
        };
    }
}
impl PropName {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Ident(it) => it.span(ast),
            Self::Str(it) => it.span(ast),
            Self::Num(it) => it.span(ast),
            Self::Computed(it) => it.span(ast),
            Self::BigInt(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Ident(it) => it.set_span(ast, span),
            Self::Str(it) => it.set_span(ast, span),
            Self::Num(it) => it.set_span(ast, span),
            Self::Computed(it) => it.set_span(ast, span),
            Self::BigInt(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_ident(&self) -> Option<&IdentName> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_str(&self) -> Option<&Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(&self) -> Option<&Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_computed(&self) -> Option<&ComputedPropName> {
        match self {
            Self::Computed(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(&self) -> Option<&BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
}
impl ComputedPropName {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = expr
        };
    }
}
impl Pat {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Ident(it) => it.span(ast),
            Self::Array(it) => it.span(ast),
            Self::Rest(it) => it.span(ast),
            Self::Object(it) => it.span(ast),
            Self::Assign(it) => it.span(ast),
            Self::Invalid(it) => it.span(ast),
            Self::Expr(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Ident(it) => it.set_span(ast, span),
            Self::Array(it) => it.set_span(ast, span),
            Self::Rest(it) => it.set_span(ast, span),
            Self::Object(it) => it.set_span(ast, span),
            Self::Assign(it) => it.set_span(ast, span),
            Self::Invalid(it) => it.set_span(ast, span),
            Self::Expr(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_ident(&self) -> Option<&BindingIdent> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(&self) -> Option<&ArrayPat> {
        match self {
            Self::Array(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_rest(&self) -> Option<&RestPat> {
        match self {
            Self::Rest(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object(&self) -> Option<&ObjectPat> {
        match self {
            Self::Object(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(&self) -> Option<&AssignPat> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_invalid(&self) -> Option<&Invalid> {
        match self {
            Self::Invalid(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl ArrayPat {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn elems(&self, ast: &crate::Ast) -> TypedSubRange<Option<Pat>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn optional(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_elems(&self, ast: &mut crate::Ast, elems: TypedSubRange<Option<Pat>>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = elems.into()
        };
    }
    #[inline]
    pub fn set_optional(&self, ast: &mut crate::Ast, optional: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = optional.into()
        };
    }
}
impl ObjectPat {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn props(&self, ast: &crate::Ast) -> TypedSubRange<ObjectPatProp> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn optional(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_props(&self, ast: &mut crate::Ast, props: TypedSubRange<ObjectPatProp>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = props.into()
        };
    }
    #[inline]
    pub fn set_optional(&self, ast: &mut crate::Ast, optional: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = optional.into()
        };
    }
}
impl AssignPat {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn left(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .pat
        };
        ret
    }
    #[inline]
    pub fn right(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_left(&self, ast: &mut crate::Ast, left: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .pat = left
        };
    }
    #[inline]
    pub fn set_right(&self, ast: &mut crate::Ast, right: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = right
        };
    }
}
impl RestPat {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn dot_3_token(&self, ast: &crate::Ast) -> Span {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .span
        };
        ret.into()
    }
    #[inline]
    pub fn arg(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .pat
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_dot3_token(&self, ast: &mut crate::Ast, dot3_token: Span) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .span = dot3_token.into()
        };
    }
    #[inline]
    pub fn set_arg(&self, ast: &mut crate::Ast, arg: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .pat = arg
        };
    }
}
impl ObjectPatProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::KeyValue(it) => it.span(ast),
            Self::Assign(it) => it.span(ast),
            Self::Rest(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::KeyValue(it) => it.set_span(ast, span),
            Self::Assign(it) => it.set_span(ast, span),
            Self::Rest(it) => it.set_span(ast, span),
        }
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
    pub fn is_rest(&self) -> bool {
        matches!(self, Self::Rest(_))
    }
    #[inline]
    pub fn as_key_value(&self) -> Option<&KeyValuePatProp> {
        match self {
            Self::KeyValue(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assign(&self) -> Option<&AssignPatProp> {
        match self {
            Self::Assign(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_rest(&self) -> Option<&RestPat> {
        match self {
            Self::Rest(it) => Some(it),
            _ => None,
        }
    }
}
impl KeyValuePatProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> PropName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .prop_name
        };
        ret
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Pat {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .pat
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: PropName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .prop_name = key
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Pat) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .pat = value
        };
    }
}
impl AssignPatProp {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn key(&self, ast: &crate::Ast) -> BindingIdent {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { BindingIdent::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_key(&self, ast: &mut crate::Ast, key: BindingIdent) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = key.node_id().into()
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_expr = value
        };
    }
}
impl Ident {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn optional(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = u32::from(node.inline_data) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn sym(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_optional(&self, ast: &mut crate::Ast, optional: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        let field_val: u32 = optional as u32;
        let old = u32::from(node.inline_data);
        node.inline_data = ((old & 16776960u32) | (field_val & 255u32)).into();
    }
    #[inline]
    pub fn set_sym(&self, ast: &mut crate::Ast, sym: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .utf8 = sym.into()
        };
    }
}
impl IdentName {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn sym(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_sym(&self, ast: &mut crate::Ast, sym: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .utf8 = sym.into()
        };
    }
}
impl PrivateName {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .utf8 = name.into()
        };
    }
}
impl BindingIdent {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn id(&self, ast: &crate::Ast) -> Ident {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        unsafe { Ident::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_id(&self, ast: &mut crate::Ast, id: Ident) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = id.node_id().index() as u32;
    }
}
impl Lit {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Str(it) => it.span(ast),
            Self::Bool(it) => it.span(ast),
            Self::Null(it) => it.span(ast),
            Self::Num(it) => it.span(ast),
            Self::BigInt(it) => it.span(ast),
            Self::Regex(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Str(it) => it.set_span(ast, span),
            Self::Bool(it) => it.set_span(ast, span),
            Self::Null(it) => it.set_span(ast, span),
            Self::Num(it) => it.set_span(ast, span),
            Self::BigInt(it) => it.set_span(ast, span),
            Self::Regex(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_str(&self) -> Option<&Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_bool(&self) -> Option<&Bool> {
        match self {
            Self::Bool(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null(&self) -> Option<&Null> {
        match self {
            Self::Null(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_num(&self) -> Option<&Number> {
        match self {
            Self::Num(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int(&self) -> Option<&BigInt> {
        match self {
            Self::BigInt(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_regex(&self) -> Option<&Regex> {
        match self {
            Self::Regex(it) => Some(it),
            _ => None,
        }
    }
}
impl Str {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Wtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .wtf8
        };
        ret.into()
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Wtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .wtf8 = value.into()
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_utf8 = raw.into()
        };
    }
}
impl Bool {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> bool {
        let node = unsafe { ast.nodes.get_unchecked(self.0) };
        let raw = unsafe { node.data.inline_data };
        raw != 0
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: bool) {
        let node = unsafe { ast.nodes.get_unchecked_mut(self.0) };
        node.data.inline_data = value as u32;
    }
}
impl Null {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
impl Number {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> f64 {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .number
        };
        ret.into()
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: f64) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .number = value.into()
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_utf8 = raw.into()
        };
    }
}
impl BigInt {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> BigIntId {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bigint
        };
        ret.into()
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> OptionalUtf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: BigIntId) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bigint = value.into()
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: OptionalUtf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_utf8 = raw.into()
        };
    }
}
impl Regex {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn exp(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .utf8
        };
        ret.into()
    }
    #[inline]
    pub fn flags(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_exp(&self, ast: &mut crate::Ast, exp: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .utf8 = exp.into()
        };
    }
    #[inline]
    pub fn set_flags(&self, ast: &mut crate::Ast, flags: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .utf8 = flags.into()
        };
    }
}
impl JSXObject {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::JSXMemberExpr(it) => it.span(ast),
            Self::Ident(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::JSXMemberExpr(it) => it.set_span(ast, span),
            Self::Ident(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_jsx_member_expr(&self) -> bool {
        matches!(self, Self::JSXMemberExpr(_))
    }
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn as_jsx_member_expr(&self) -> Option<&JSXMemberExpr> {
        match self {
            Self::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXMemberExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn obj(&self, ast: &crate::Ast) -> JSXObject {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .jsx_object
        };
        ret
    }
    #[inline]
    pub fn prop(&self, ast: &crate::Ast) -> IdentName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { IdentName::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_obj(&self, ast: &mut crate::Ast, obj: JSXObject) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .jsx_object = obj
        };
    }
    #[inline]
    pub fn set_prop(&self, ast: &mut crate::Ast, prop: IdentName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = prop.node_id().into()
        };
    }
}
impl JSXNamespacedName {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn ns(&self, ast: &crate::Ast) -> IdentName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { IdentName::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> IdentName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { IdentName::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_ns(&self, ast: &mut crate::Ast, ns: IdentName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = ns.node_id().into()
        };
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: IdentName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = name.node_id().into()
        };
    }
}
impl JSXEmptyExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
impl JSXExprContainer {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> JSXExpr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .jsx_expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: JSXExpr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .jsx_expr = expr
        };
    }
}
impl JSXExpr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::JSXEmptyExpr(it) => it.span(ast),
            Self::Expr(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::JSXEmptyExpr(it) => it.set_span(ast, span),
            Self::Expr(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_jsx_empty_expr(&self) -> bool {
        matches!(self, Self::JSXEmptyExpr(_))
    }
    #[inline]
    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(_))
    }
    #[inline]
    pub fn as_jsx_empty_expr(&self) -> Option<&JSXEmptyExpr> {
        match self {
            Self::JSXEmptyExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(&self) -> Option<&Expr> {
        match self {
            Self::Expr(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXSpreadChild {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn expr(&self, ast: &crate::Ast) -> Expr {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .expr
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_expr(&self, ast: &mut crate::Ast, expr: Expr) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .expr = expr
        };
    }
}
impl JSXElementName {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Ident(it) => it.span(ast),
            Self::JSXMemberExpr(it) => it.span(ast),
            Self::JSXNamespacedName(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Ident(it) => it.set_span(ast, span),
            Self::JSXMemberExpr(it) => it.set_span(ast, span),
            Self::JSXNamespacedName(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_ident(&self) -> Option<&Ident> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member_expr(&self) -> Option<&JSXMemberExpr> {
        match self {
            Self::JSXMemberExpr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(&self) -> Option<&JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXOpeningElement {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> JSXElementName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .jsx_element_name
        };
        ret
    }
    #[inline]
    pub fn attrs(&self, ast: &crate::Ast) -> TypedSubRange<JSXAttrOrSpread> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn self_closing(&self, ast: &crate::Ast) -> bool {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .bool
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: JSXElementName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .jsx_element_name = name
        };
    }
    #[inline]
    pub fn set_attrs(&self, ast: &mut crate::Ast, attrs: TypedSubRange<JSXAttrOrSpread>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = attrs.into()
        };
    }
    #[inline]
    pub fn set_self_closing(&self, ast: &mut crate::Ast, self_closing: bool) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .bool = self_closing.into()
        };
    }
}
impl JSXAttrOrSpread {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::JSXAttr(it) => it.span(ast),
            Self::SpreadElement(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::JSXAttr(it) => it.set_span(ast, span),
            Self::SpreadElement(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_jsx_attr(&self) -> bool {
        matches!(self, Self::JSXAttr(_))
    }
    #[inline]
    pub fn is_spread_element(&self) -> bool {
        matches!(self, Self::SpreadElement(_))
    }
    #[inline]
    pub fn as_jsx_attr(&self) -> Option<&JSXAttr> {
        match self {
            Self::JSXAttr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_spread_element(&self) -> Option<&SpreadElement> {
        match self {
            Self::SpreadElement(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXClosingElement {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> JSXElementName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .jsx_element_name
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: JSXElementName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .jsx_element_name = name
        };
    }
}
impl JSXAttr {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn name(&self, ast: &crate::Ast) -> JSXAttrName {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .jsx_attr_name
        };
        ret
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Option<JSXAttrValue> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_jsx_attr_value
        };
        ret
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_name(&self, ast: &mut crate::Ast, name: JSXAttrName) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .jsx_attr_name = name
        };
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Option<JSXAttrValue>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_jsx_attr_value = value
        };
    }
}
impl JSXAttrName {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Ident(it) => it.span(ast),
            Self::JSXNamespacedName(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Ident(it) => it.set_span(ast, span),
            Self::JSXNamespacedName(it) => it.set_span(ast, span),
        }
    }
    #[inline]
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }
    #[inline]
    pub fn is_jsx_namespaced_name(&self) -> bool {
        matches!(self, Self::JSXNamespacedName(_))
    }
    #[inline]
    pub fn as_ident(&self) -> Option<&IdentName> {
        match self {
            Self::Ident(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(&self) -> Option<&JSXNamespacedName> {
        match self {
            Self::JSXNamespacedName(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXAttrValue {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::Str(it) => it.span(ast),
            Self::JSXExprContainer(it) => it.span(ast),
            Self::JSXElement(it) => it.span(ast),
            Self::JSXFragment(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::Str(it) => it.set_span(ast, span),
            Self::JSXExprContainer(it) => it.set_span(ast, span),
            Self::JSXElement(it) => it.set_span(ast, span),
            Self::JSXFragment(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_str(&self) -> Option<&Str> {
        match self {
            Self::Str(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expr_container(&self) -> Option<&JSXExprContainer> {
        match self {
            Self::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(&self) -> Option<&JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(&self) -> Option<&JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXText {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn value(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .utf8
        };
        ret.into()
    }
    #[inline]
    pub fn raw(&self, ast: &crate::Ast) -> Utf8Ref {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .utf8
        };
        ret.into()
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_value(&self, ast: &mut crate::Ast, value: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .utf8 = value.into()
        };
    }
    #[inline]
    pub fn set_raw(&self, ast: &mut crate::Ast, raw: Utf8Ref) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .utf8 = raw.into()
        };
    }
}
impl JSXElement {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn opening(&self, ast: &crate::Ast) -> JSXOpeningElement {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { JSXOpeningElement::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn children(&self, ast: &crate::Ast) -> TypedSubRange<JSXElementChild> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn closing(&self, ast: &crate::Ast) -> Option<JSXClosingElement> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .optional_node
        };
        ret.map(|id| unsafe { JSXClosingElement::from_node_id_unchecked(id, ast) })
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_opening(&self, ast: &mut crate::Ast, opening: JSXOpeningElement) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = opening.node_id().into()
        };
    }
    #[inline]
    pub fn set_children(&self, ast: &mut crate::Ast, children: TypedSubRange<JSXElementChild>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = children.into()
        };
    }
    #[inline]
    pub fn set_closing(&self, ast: &mut crate::Ast, closing: Option<JSXClosingElement>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .optional_node = closing.map(|n| n.node_id()).into()
        };
    }
}
impl JSXElementChild {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        match self {
            Self::JSXText(it) => it.span(ast),
            Self::JSXExprContainer(it) => it.span(ast),
            Self::JSXSpreadChild(it) => it.span(ast),
            Self::JSXElement(it) => it.span(ast),
            Self::JSXFragment(it) => it.span(ast),
        }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        match self {
            Self::JSXText(it) => it.set_span(ast, span),
            Self::JSXExprContainer(it) => it.set_span(ast, span),
            Self::JSXSpreadChild(it) => it.set_span(ast, span),
            Self::JSXElement(it) => it.set_span(ast, span),
            Self::JSXFragment(it) => it.set_span(ast, span),
        }
    }
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
    pub fn as_jsx_text(&self) -> Option<&JSXText> {
        match self {
            Self::JSXText(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expr_container(&self) -> Option<&JSXExprContainer> {
        match self {
            Self::JSXExprContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_spread_child(&self) -> Option<&JSXSpreadChild> {
        match self {
            Self::JSXSpreadChild(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(&self) -> Option<&JSXElement> {
        match self {
            Self::JSXElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(&self) -> Option<&JSXFragment> {
        match self {
            Self::JSXFragment(it) => Some(it),
            _ => None,
        }
    }
}
impl JSXFragment {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn opening(&self, ast: &crate::Ast) -> JSXOpeningFragment {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { JSXOpeningFragment::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn children(&self, ast: &crate::Ast) -> TypedSubRange<JSXElementChild> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .sub_range
        };
        unsafe { ret.cast_to_typed() }
    }
    #[inline]
    pub fn closing(&self, ast: &crate::Ast) -> JSXClosingFragment {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        let ret = unsafe {
            ast.extra_data
                .as_raw_slice()
                .get_unchecked(offset.index())
                .node
        };
        unsafe { JSXClosingFragment::from_node_id_unchecked(ret, ast) }
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
    #[inline]
    pub fn set_opening(&self, ast: &mut crate::Ast, opening: JSXOpeningFragment) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = opening.node_id().into()
        };
    }
    #[inline]
    pub fn set_children(&self, ast: &mut crate::Ast, children: TypedSubRange<JSXElementChild>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .sub_range = children.into()
        };
    }
    #[inline]
    pub fn set_closing(&self, ast: &mut crate::Ast, closing: JSXClosingFragment) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ast.nodes
                    .get_unchecked(self.0)
                    .data
                    .extra_data_start
                    .index()
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset < ast.extra_data.len());
        unsafe {
            ast.extra_data
                .as_raw_slice_mut()
                .get_unchecked_mut(offset.index())
                .node = closing.node_id().into()
        };
    }
}
impl JSXOpeningFragment {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
impl JSXClosingFragment {
    #[inline]
    pub fn span(&self, ast: &crate::Ast) -> crate::Span {
        unsafe { ast.nodes.get_unchecked(self.0).span }
    }
    #[inline]
    pub fn span_lo(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).lo
    }
    #[inline]
    pub fn span_hi(&self, ast: &crate::Ast) -> crate::BytePos {
        self.span(ast).hi
    }
    #[inline]
    pub fn set_span(&self, ast: &mut crate::Ast, span: crate::Span) {
        unsafe {
            ast.nodes.get_unchecked_mut(self.0).span = span;
        }
    }
}
