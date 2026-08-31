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

#[allow(clippy::diverging_sub_expression)]
mod generated {
    mod ast_builder;
    mod ast_clone_in;
    pub(crate) mod ast_extra_compact;
    pub mod ast_node;
    mod ast_property;
    pub(crate) mod ast_visitor;
}

use oxc_index::Idx;
use std::marker::PhantomData;
use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};

pub use ast::*;
pub use comment::*;
pub use derive::*;
pub use diagnostic::*;
pub use generated::ast_node::{NodeKind, NodeKindData};
pub use generated::ast_visitor::*;
pub use node_id::{AstNode, ExtraDataCompact, ExtraDataId, NodeId, SubRange, TypedSubRange};
pub use semantic::{ReferenceId, ScopeId, SymbolId};
pub use strings::{
    AstInternedUtf8, AstUtf8, AstWtf8, GeneratedName, GeneratedUtf8State, Utf8Chunks,
};
pub use syntax::*;
pub use visit::{TraverseAction, TraverseCtx};

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
    pub struct NodeData<'a, NodeId> {
        pub(crate) span => span_mut => span_unchecked => span_mut_unchecked: Span,
        pub(crate) kind => kind_mut => kind_unchecked => kind_mut_unchecked: NodeKind,
        pub(crate) data => data_mut => data_unchecked => data_mut_unchecked: NodePayload,
    }
}

/// Compact 128-bit payload for a node.
///
/// Fully inlined nodes use all 128 bits. Nodes with out-of-line fields use the
/// low 96 bits for inline data and the high 32 bits for the start index in
/// [Ast::extra_data].
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct NodePayload(pub(crate) u128);

/// Compact values stored in the flattening AST's extra-data table.
#[derive(Clone, Copy)]
pub union ExtraData {
    node_id: NodeId,

    sub_range: SubRange,

    /// Any scalar enum or other value representable within eight bytes.
    other: u64,
}

impl ExtraData {
    /// Construct a fully initialized extra-data word containing a node ID.
    ///
    /// `ExtraData` is copied byte-for-byte across the JavaScript FFI. Writing
    /// the four-byte union field directly would leave the high four bytes
    /// uninitialized.
    #[inline]
    pub(crate) fn from_node_id(node_id: NodeId) -> Self {
        let mut data = Self { other: 0 };
        data.node_id = node_id;
        data
    }
}

impl<'a> Ast<'a> {
    #[inline]
    pub(crate) fn root_node_id(&self) -> NodeId {
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
    pub(crate) fn add_node(&mut self, span: Span, kind: NodeKind, data: NodePayload) -> NodeId {
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
    fn add_extra(&mut self, extra: ExtraData) -> ExtraDataId {
        let id = self.next_extra_data_id();
        self.extra_data.push(extra);
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

    #[inline]
    pub fn add_typed_sub_range<N: ExtraDataCompact<'a>, I: IntoIterator<Item = N>>(
        &mut self,
        iter: I,
    ) -> TypedSubRange<N> {
        let start = self.next_extra_data_id();
        for node in iter {
            let data = node.to_extra_data(self);
            self.extra_data.push(data);
        }
        TypedSubRange {
            inner: SubRange {
                start,
                end: self.next_extra_data_id(),
            },
            _phantom: PhantomData,
        }
    }

    /// Append pre-encoded extra data as a single contiguous range.
    pub fn add_extra_data_sub_range(&mut self, data: &[ExtraData]) -> SubRange {
        let start = self.next_extra_data_id();
        let end = start.index() + data.len();
        self.extra_data.extend_from_slice_copy(data);
        SubRange {
            start,
            end: ExtraDataId::from_usize_unchecked(end),
        }
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
    pub fn source(&self) -> &str {
        self.strings.source()
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
    pub fn add_utf8(&mut self, value: &str) -> AstInternedUtf8 {
        self.strings.add_utf8(value)
    }

    /// Adds a UTF-8 string to the existing deduplicating string pool and
    /// marks the returned handle as canonical for identity-based lookup.
    #[inline]
    pub fn intern_utf8(&mut self, value: &str) -> AstInternedUtf8 {
        self.strings.add_utf8(value)
    }

    /// Canonicalizes an existing UTF-8 handle without copying its value into string storage.
    #[inline]
    pub fn intern_utf8_range(&mut self, value: AstUtf8) -> AstInternedUtf8 {
        self.strings.intern_utf8_range(value)
    }

    /// Canonicalizes an identifier and records it for generated-name collision
    /// checks without a second content lookup.
    #[inline]
    pub fn intern_name_range(&mut self, value: AstUtf8) -> AstInternedUtf8 {
        self.strings.intern_name_range(value)
    }

    #[inline]
    pub fn generated_utf8_state(&mut self, base: &str) -> GeneratedUtf8State {
        self.strings.generated_utf8_state(base)
    }

    #[inline]
    pub fn generated_utf8_base(state: &GeneratedUtf8State) -> AstInternedUtf8 {
        StringPool::generated_utf8_base(state)
    }

    #[inline]
    pub fn intern_generated_name_from_state(
        &mut self,
        state: &mut GeneratedUtf8State,
        suffix: u64,
    ) -> AstInternedUtf8 {
        self.strings.intern_generated_name_from_state(state, suffix)
    }

    #[inline]
    pub fn reserve_generated_name_from_state(
        &mut self,
        state: &mut GeneratedUtf8State,
        suffix: u64,
    ) -> (AstInternedUtf8, bool) {
        self.strings
            .reserve_generated_name_from_state(state, suffix)
    }

    #[inline]
    #[doc(hidden)]
    pub fn skip_observed_generated_suffixes(
        &mut self,
        state: &mut GeneratedUtf8State,
        suffix: u64,
    ) -> u64 {
        self.strings.skip_observed_generated_suffixes(state, suffix)
    }

    #[inline]
    pub fn reserve_name(&mut self, name: AstInternedUtf8) -> bool {
        self.strings.reserve_name(name)
    }

    #[inline]
    #[doc(hidden)]
    pub fn observe_name(&mut self, name: AstInternedUtf8) -> bool {
        self.strings.observe_name(name)
    }

    #[inline]
    pub fn is_name_reserved(&self, name: AstInternedUtf8) -> bool {
        self.strings.is_name_reserved(name)
    }

    /// Builds or reuses the physical prefix shared by one block of 16 generated names.
    #[inline]
    pub fn generated_name_prefix(&mut self, base: AstInternedUtf8, block: u64) -> Option<AstUtf8> {
        self.strings.generated_name_prefix(base, block)
    }

    /// Interns a generated name from a cached prefix and hexadecimal suffix.
    #[inline]
    pub fn intern_generated_name(
        &mut self,
        base: AstInternedUtf8,
        prefix: Option<AstUtf8>,
        suffix: u64,
    ) -> AstInternedUtf8 {
        self.strings.intern_generated_name(base, prefix, suffix)
    }

    #[inline]
    pub fn add_wtf8(&mut self, value: &swc_next_allocator::wtf8::Wtf8) -> AstWtf8 {
        self.strings.add_wtf8(value)
    }

    #[inline]
    pub fn add_wtf8_prefixed_utf8(&mut self, prefix: &[u8], value: AstUtf8) -> AstWtf8 {
        self.strings.add_wtf8_prefixed_utf8(prefix, value)
    }

    #[inline]
    pub fn utf8_to_wtf8(&mut self, value: AstUtf8) -> AstWtf8 {
        self.strings.utf8_to_wtf8(value)
    }

    /// Canonicalizes an existing WTF-8 handle without copying its value into string storage.
    #[inline]
    pub fn intern_wtf8_range(&mut self, value: AstWtf8) -> AstWtf8 {
        self.strings.intern_wtf8_range(value)
    }

    #[inline]
    pub fn get_flat_utf8(&self, id: AstUtf8) -> &str {
        self.strings.get_flat_utf8(id)
    }

    /// Returns a borrowed or materialized logical UTF-8 string.
    ///
    /// Test-only convenience API. Production callers should use the
    /// non-materializing string-pool operations instead.
    #[doc(hidden)]
    #[inline]
    pub fn get_utf8(&self, id: AstUtf8) -> std::borrow::Cow<'_, str> {
        self.strings.get_utf8(id)
    }

    /// Writes a logical UTF-8 handle without materializing segmented bytes.
    #[inline]
    pub fn write_utf8<W: std::fmt::Write>(&self, id: AstUtf8, writer: &mut W) -> std::fmt::Result {
        self.strings.write_utf8(id, writer)
    }

    /// Resolves a logical UTF-8 handle once and passes both segments to a consumer.
    #[inline(always)]
    pub fn consume_utf8<'s, R>(
        &'s self,
        id: AstUtf8,
        consumer: impl FnOnce(&'s str, Option<u8>) -> R,
    ) -> R {
        self.strings.consume_utf8(id, consumer)
    }

    /// Returns a non-allocating logical view of a flat or compact UTF-8 handle.
    #[inline]
    pub fn utf8_chunks(&self, id: AstUtf8) -> Utf8Chunks<'_> {
        self.strings.utf8_chunks(id)
    }

    #[inline]
    pub fn utf8_eq(&self, left: AstUtf8, right: AstUtf8) -> bool {
        self.strings.utf8_eq(left, right)
    }

    #[inline]
    pub fn utf8_eq_bytes(&self, id: AstUtf8, bytes: &[u8]) -> bool {
        self.strings.utf8_eq_bytes(id, bytes)
    }

    #[inline]
    pub fn utf8_eq_wtf8(&self, utf8: AstUtf8, wtf8: AstWtf8) -> bool {
        self.strings.utf8_eq_wtf8(utf8, wtf8)
    }

    #[inline]
    pub fn wtf8_eq(&self, left: AstWtf8, right: AstWtf8) -> bool {
        self.strings.wtf8_eq(left, right)
    }

    #[inline]
    pub fn wtf8_eq_bytes(&self, id: AstWtf8, bytes: &[u8]) -> bool {
        self.strings.wtf8_eq_bytes(id, bytes)
    }

    #[inline]
    pub fn utf8_contains_byte(&self, id: AstUtf8, byte: u8) -> bool {
        self.strings.utf8_contains_byte(id, byte)
    }

    #[inline]
    pub fn utf8_first_byte(&self, id: AstUtf8) -> Option<u8> {
        self.strings.utf8_first_byte(id)
    }

    #[inline]
    pub fn get_wtf8(&self, id: AstWtf8) -> &swc_next_allocator::wtf8::Wtf8 {
        self.strings.get_wtf8(id)
    }

    #[inline]
    pub fn as_utf8(&self, id: AstWtf8) -> Option<AstUtf8> {
        self.strings.as_utf8(id)
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
    pub fn new_copying(source: &str, allocator: &'a Allocator) -> Self {
        let mut copied = ArenaVec::with_capacity_in(source.len(), allocator);
        copied.extend_from_slice(source.as_bytes());
        // SAFETY: the buffer was copied from a valid `str`.
        let source = unsafe { std::str::from_utf8_unchecked_mut(copied.into_bump_slice_mut()) };
        Self::new(source, allocator)
    }

    /// Create a flattening AST backed by `source` and `allocator`.
    pub fn new(source: &'a mut str, allocator: &'a Allocator) -> Self {
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
