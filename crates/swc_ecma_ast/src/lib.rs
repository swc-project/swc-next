mod assert_layout;
mod ast;
mod comment;
mod derive;
mod diagnostic;
pub mod ffi;
mod node_id;
mod semantic;
mod strings;
mod syntax;
mod visit;

mod generated {
    mod ast_builder;
    mod ast_clone_in;
    pub mod ast_node;
    pub(crate) mod ast_payload;
    mod ast_property;
    pub(crate) mod ast_visitor;
}

use generated::ast_payload::NodePayload;
use node_id::ExtraData;
use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};

pub use ast::*;
pub use comment::*;
pub use derive::*;
pub use diagnostic::*;
pub use generated::ast_node::*;
pub use generated::ast_visitor::*;
pub use node_id::{AstNode, ExtraDataId, ExtraDataValue, NodeId, SubRange};
pub use oxc_index::Idx;
pub use semantic::{ReferenceId, ScopeId, SymbolId};
pub use strings::{AstUtf8, AstWtf8};
pub use syntax::*;
pub use visit::{TraverseAction, TraverseCtx, TraverseMutCtx};

use swc_next_allocator::multi_index_vec;

use crate::strings::StringPool;

/// AST context that stores everything about the flattening AST.
pub struct Ast<'a> {
    allocator: &'a Allocator,

    root: Option<NodeId>,

    source_type: SourceType,

    lang: Lang,

    attached_comments: ArenaVec<'a, Comment>,

    /// Flattening AST nodes.
    pub(crate) nodes: NodeData<'a>,

    /// Flattening AST fields.
    extra_data: ArenaVec<'a, ExtraData>,

    /// Source-backed and decoded AST strings.
    strings: StringPool<'a>,
}

multi_index_vec! {
    /// Arena-backed structure-of-arrays storage for flattening AST nodes.
    pub(crate) struct NodeData<'a, NodeId> {
        pub(crate) span => span_mut => span_unchecked => span_mut_unchecked: Span,
        pub(crate) kind => kind_mut => kind_unchecked => kind_mut_unchecked: NodeKind,
        pub(crate) data => data_mut => data_unchecked => data_mut_unchecked: NodePayload,
    }
}

impl<'a> Ast<'a> {
    #[inline]
    fn root_node_id(&self) -> NodeId {
        self.root.expect("AST has no root")
    }

    /// Returns the AST's root program.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not a program.
    #[inline]
    pub fn root_program(&self) -> Program {
        let root = self.root_node_id();
        assert_eq!(
            self.node_kind(root),
            NodeKind::Program,
            "AST root is not a program"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Program::from_node_id_unchecked(root) }
    }

    /// Returns the AST's root expression.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not an expression.
    #[inline]
    pub fn root_expression(&self) -> Expr {
        let root = self.root_node_id();
        assert!(
            self.node_kind(root).is_expr(),
            "AST root is not an expression"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Expr::from_node_id_unchecked(root) }
    }

    /// Returns the AST's root statement.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not a statement.
    #[inline]
    pub fn root_statement(&self) -> Stmt {
        let root = self.root_node_id();
        assert!(
            self.node_kind(root).is_stmt(),
            "AST root is not a statement"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Stmt::from_node_id_unchecked(root) }
    }

    #[inline]
    fn add_node(&mut self, span: Span, kind: NodeKind, data: NodePayload) -> NodeId {
        self.nodes.push(span, kind, data)
    }

    /// Replace `old`'s node data with `new`'s while preserving `old`'s node ID.
    ///
    /// # Safety
    ///
    /// Replacing the data may change the concrete node kind stored at `old`,
    /// invalidating typed handles and AST edges that refer to it.
    /// The caller must ensure that every typed handle and every AST edge used
    /// after this call accepts the replacement kind.
    /// In particular, a handle for `old`'s previous concrete kind must never be
    /// used to access node-specific data after the replacement.
    #[inline]
    pub unsafe fn set_data(&mut self, old: NodeId, new: NodeId) {
        let span = *self.nodes.span(new);
        let kind = *self.nodes.kind(new);
        let data = *self.nodes.data(new);
        *self.nodes.span_mut(old) = span;
        *self.nodes.kind_mut(old) = kind;
        *self.nodes.data_mut(old) = data;
    }

    /// Replace `old` with a newly built `new` node, discard the temporary node,
    /// and return a typed handle that preserves `old`'s node ID.
    ///
    /// # Safety
    ///
    /// Replacing the data may change the concrete node kind stored at `old`,
    /// invalidating typed handles and AST edges that refer to it. The caller
    /// must ensure that every typed handle and every AST edge used after this
    /// call accepts `New`'s concrete kind. In particular, a handle for `old`'s
    /// previous concrete kind must never be used to access node-specific data
    /// after the replacement.
    ///
    /// `new` must be an unattached temporary node at the end of the node table.
    /// No handle or AST edge may refer to `new` after this call, and the moved
    /// payload must not contain a reference to `new`, because its slot is
    /// removed before this function returns.
    #[inline]
    pub unsafe fn replace_data<Old: AstNode, New: AstNode>(&mut self, old: Old, new: New) -> New {
        let old = old.node_id();
        let new = new.node_id();
        assert_ne!(old, new);
        assert_eq!(new.index() + 1, self.nodes.len());

        let span = *self.nodes.span(new);
        let kind = *self.nodes.kind(new);
        let data = *self.nodes.data(new);
        *self.nodes.span_mut(old) = span;
        *self.nodes.kind_mut(old) = kind;
        *self.nodes.data_mut(old) = data;
        self.nodes.truncate(new.index());

        unsafe { New::from_node_id_unchecked(old) }
    }

    #[inline]
    pub fn add_sub_range<N: ExtraDataValue, I: IntoIterator<Item = N>>(
        &mut self,
        iter: I,
    ) -> SubRange<N> {
        let start = self.next_extra_data_id();
        for node in iter {
            let data = node.into_extra_data();
            self.extra_data.push(data);
        }
        SubRange::new(start, self.next_extra_data_id())
    }

    /// Append pre-encoded extra data and return its starting index.
    fn add_extra_data_slice(&mut self, data: &[ExtraData]) -> ExtraDataId {
        let start = self.next_extra_data_id();
        self.extra_data.extend_from_slice_copy(data);
        start
    }

    #[inline]
    pub fn allocator(&self) -> &'a Allocator {
        self.allocator
    }

    /// Creates a map grouping attached comments by attachment offset.
    pub fn create_comments_map(&self) -> ArenaHashMap<'a, u32, ArenaVec<'a, Comment>> {
        let mut attached_comments =
            ArenaHashMap::with_capacity_in(self.attached_comments.len(), self.allocator);
        let allocator = self.allocator;
        for &comment in &self.attached_comments {
            let comments = attached_comments.entry(comment.attached_to);
            let comments = comments.or_insert_with(|| ArenaVec::new_in(allocator));
            comments.push(comment);
        }
        attached_comments
    }

    /// Returns attached comments in source order.
    pub fn comments(&self) -> &[Comment] {
        &self.attached_comments
    }

    /// Retargets comment attachment offsets while preserving source order.
    ///
    /// The callback receives each comment's position and current attachment
    /// offset. Returning `Some` replaces the offset, while `None` leaves it
    /// unchanged.
    pub fn retarget_comments(
        &mut self,
        mut retarget: impl FnMut(CommentPosition, u32) -> Option<u32>,
    ) {
        for comment in &mut self.attached_comments {
            if let Some(attached_to) = retarget(comment.position, comment.attached_to) {
                comment.attached_to = attached_to;
            }
        }
    }

    #[inline]
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    #[inline]
    pub fn lang(&self) -> Lang {
        self.lang
    }

    #[inline]
    pub fn is_ts(&self) -> bool {
        self.lang.is_typescript()
    }

    #[inline]
    pub fn is_jsx(&self) -> bool {
        self.lang.is_jsx()
    }

    #[inline]
    pub fn is_module(&self) -> bool {
        self.source_type == SourceType::Module
    }

    #[inline]
    pub fn source(&self) -> &'a str {
        self.strings.source
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice(&self, start: u32, end: u32) -> AstUtf8 {
        self.strings.source_slice(start, end)
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice_wtf8(&self, start: u32, end: u32) -> AstWtf8 {
        self.strings.source_slice_wtf8(start, end)
    }

    #[inline]
    pub fn add_utf8(&mut self, value: &str) -> AstUtf8 {
        self.strings.add_utf8(value)
    }

    #[inline]
    pub fn add_wtf8(&mut self, value: &swc_next_allocator::wtf8::Wtf8) -> AstWtf8 {
        self.strings.add_wtf8(value)
    }

    /// Borrows immutable string bytes for the source/allocator lifetime.
    /// The reference remains valid when strings are added or the AST is dropped.
    #[inline]
    pub fn get_utf8(&self, id: AstUtf8) -> &'a str {
        self.strings.get_utf8(id)
    }

    /// Borrows immutable string bytes for the source/allocator lifetime.
    /// The reference remains valid when strings are added or the AST is dropped.
    #[inline]
    pub fn get_wtf8(&self, id: AstWtf8) -> &'a swc_next_allocator::wtf8::Wtf8 {
        self.strings.get_wtf8(id)
    }

    #[inline]
    pub fn empty_utf8(&self) -> AstUtf8 {
        AstUtf8::EMPTY
    }

    #[inline]
    pub fn empty_wtf8(&self) -> AstWtf8 {
        AstWtf8::EMPTY
    }

    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of entries in the out-of-line field table.
    #[inline]
    pub fn extra_data_count(&self) -> usize {
        self.extra_data.len()
    }

    /// Returns the concrete type stored at `node_id`.
    #[inline]
    pub fn node_kind(&self, node_id: NodeId) -> NodeKind {
        *self.nodes.kind(node_id)
    }

    /// Returns the source span stored for `node_id`.
    #[inline]
    pub fn span(&self, node_id: NodeId) -> Span {
        *self.nodes.span(node_id)
    }
}

#[doc(hidden)]
impl<'a> Ast<'a> {
    /// Create a flattening AST backed by `source` and `allocator`.
    pub fn new(source: &'a str, allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            root: None,
            source_type: SourceType::Module,
            lang: Lang::Js,
            attached_comments: ArenaVec::new_in(allocator),
            nodes: NodeData::new_in(allocator),
            extra_data: ArenaVec::with_capacity_in(0, allocator),
            strings: StringPool::new_in(allocator, source),
        }
    }

    #[inline]
    pub fn set_source_metadata(&mut self, source_type: SourceType, lang: Lang) {
        self.source_type = source_type;
        self.lang = lang;
    }

    /// Replaces attached comments while preserving source order.
    pub fn set_attached_comments(&mut self, comments: ArenaVec<'a, Comment>) {
        self.attached_comments = comments;
    }

    /// Removes all attached comments.
    pub fn clear_attached_comments(&mut self) {
        self.attached_comments.clear();
    }

    /// Sets a program as the AST root.
    #[inline]
    pub fn set_root_program(&mut self, root: Program) {
        self.root = Some(root.node_id());
    }

    /// Sets an expression as the AST root.
    #[inline]
    pub fn set_root_expression(&mut self, root: Expr) {
        self.root = Some(root.node_id());
    }

    /// Sets a statement as the AST root.
    #[inline]
    pub fn set_root_statement(&mut self, root: Stmt) {
        self.root = Some(root.node_id());
    }

    /// Reserves storage for parser output without changing logical lengths.
    #[inline]
    pub fn reserve(&mut self, additional_nodes: usize, additional_extra_data: usize) {
        self.nodes.reserve(additional_nodes);
        self.extra_data.reserve(additional_extra_data);
    }

    /// Rewinds node and out-of-line field storage to an earlier checkpoint.
    #[inline]
    pub fn truncate(&mut self, nodes_len: usize, extra_data_len: usize) {
        assert!(nodes_len <= self.nodes.len());
        assert!(extra_data_len <= self.extra_data.len());
        self.nodes.truncate(nodes_len);
        self.extra_data.truncate(extra_data_len);
    }
}

#[doc(hidden)]
impl Ast<'_> {
    #[inline]
    fn add_extra<T: ExtraDataValue>(&mut self, value: T) -> ExtraDataId {
        let id = self.next_extra_data_id();
        self.extra_data.push(value.into_extra_data());
        id
    }

    #[inline]
    fn next_extra_data_id(&self) -> ExtraDataId {
        let index = self.extra_data.len();
        assert!(
            index <= ExtraDataId::MAX,
            "extra data index exceeds ExtraDataId range"
        );
        ExtraDataId::from_usize_unchecked(index)
    }
}

pub const DUMMY_SP: Span = Span { start: 0, end: 0 };

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn is_dummy(self) -> bool {
        self.start == 0 && self.end == 0
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl From<FormalParameterPattern> for BindingPattern {
    fn from(pattern: FormalParameterPattern) -> Self {
        // SAFETY: all formal parameter binding variants are also BindingPattern variants.
        unsafe { Self::from_node_id_unchecked(pattern.node_id()) }
    }
}
