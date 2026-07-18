mod assert_layout;
mod ast;
mod comment;
mod common;
mod derive;
mod multi_index_vec;
mod node_id;
mod semantic;
mod span;
mod strings;
mod visit;

mod generated {
    mod ast_builder;
    mod ast_clone_in;
    pub(crate) mod ast_extra_compact;
    mod ast_node_id;
    mod ast_property;
    pub(crate) mod ast_visitor;
}

use oxc_index::Idx;
use std::marker::PhantomData;
use swc_experimental_allocator::{Allocator, vec::Vec as ArenaVec};

pub use ast::*;
pub use comment::*;
pub use common::*;
pub use derive::*;
pub use generated::ast_visitor::*;
pub use node_id::{ExtraDataCompact, ExtraDataId, NodeId, SubRange, TypedSubRange};
pub use semantic::{ScopeId, SymbolId};
pub use span::{DUMMY_SP, Span};
pub use swc_experimental_allocator::atom::{Atom, Wtf8Atom};

use crate::multi_index_vec::multi_index_vec;
use crate::node_id::OptionalSubRange;
use crate::span::OptionalSpan;
use crate::strings::{AtomId, AtomStore, Wtf8AtomId};

/// AST context that stores everything about the flattening AST.
pub struct Ast<'a> {
    allocator: &'a Allocator,

    /// Flattening AST nodes.
    pub(crate) nodes: NodeData<'a>,

    /// Flattening AST fields.
    extra_data: ArenaVec<'a, ExtraData>,

    /// Borrowed atoms are kept out of `ExtraData` so it remains eight bytes.
    atoms: AtomStore<'a>,
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

include!("generated/ast_extra_data.rs");

/// The NodeKind is one-to-one mapping to the typed AST node declared with struct keyword (not enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    // module.rs
    Module,
    Script,

    // module_decl.rs
    ImportDecl,
    ExportDecl,
    NamedExport,
    ImportDefaultSpecifier,
    ImportStarAsSpecifier,
    ImportNamedSpecifier,
    ExportNamespaceSpecifier,
    ExportDefaultSpecifier,
    ExportNamedSpecifier,
    ExportDefaultDecl,
    ExportDefaultExpr,
    ExportAll,

    // expr.rs
    ThisExpr,
    ArrayLit,
    ObjectLit,
    SpreadElement,
    ExprOrSpread,
    UnaryExpr,
    BinExpr,
    UpdateExpr,
    FnExpr,
    AssignExpr,
    MemberExpr,
    Super,
    SuperPropExpr,
    CondExpr,
    CallExpr,
    NewExpr,
    SeqExpr,
    Tpl,
    TaggedTpl,
    TplElement,
    ArrowExpr,
    ClassExpr,
    YieldExpr,
    MetaPropExpr,
    AwaitExpr,
    ParenExpr,
    OptChainExpr,
    OptCall,
    ImportExpr,
    Invalid,

    // stmt.rs
    BlockStmt,
    EmptyStmt,
    DebuggerStmt,
    WithStmt,
    ReturnStmt,
    LabeledStmt,
    BreakStmt,
    ContinueStmt,
    IfStmt,
    SwitchStmt,
    ThrowStmt,
    TryStmt,
    WhileStmt,
    DoWhileStmt,
    ForStmt,
    ForInStmt,
    ForOfStmt,
    SwitchCase,
    CatchClause,
    ExprStmt,

    // class.rs
    Class,
    ClassProp,
    PrivateProp,
    ClassMethod,
    PrivateMethod,
    Constructor,
    Decorator,
    StaticBlock,
    AutoAccessor,

    // prop.rs
    KeyValueProp,
    AssignProp,
    GetterProp,
    SetterProp,
    MethodProp,
    ComputedPropName,

    // decl.rs
    ClassDecl,
    FnDecl,
    VarDecl,
    UsingDecl,
    VarDeclarator,

    // function.rs
    Function,
    Param,

    // lit.rs
    BigInt,
    Str,
    Bool,
    Null,
    Regex,
    Number,

    // pat.rs
    ArrayPat,
    ObjectPat,
    AssignPat,
    RestPat,
    KeyValuePatProp,
    AssignPatProp,

    // ident.rs
    Ident,
    IdentName,
    BindingIdent,
    PrivateName,

    // jsx.rs
    JSXMemberExpr,
    JSXNamespacedName,
    JSXEmptyExpr,
    JSXExprContainer,
    JSXSpreadChild,
    JSXElementName,
    JSXOpeningElement,
    JSXClosingElement,
    JSXAttr,
    JSXText,
    JSXElement,
    JSXFragment,
    JSXOpeningFragment,
    JSXClosingFragment,

    // typescript.rs
    TsTypeAnn,
    TsTypeParamDecl,
    TsTypeParam,
    TsTypeParamInstantiation,
    TsParamProp,
    TsQualifiedName,
    TsCallSignatureDecl,
    TsConstructSignatureDecl,
    TsPropertySignature,
    TsGetterSignature,
    TsSetterSignature,
    TsMethodSignature,
    TsIndexSignature,
    TsKeywordType,
    TsThisType,
    TsFnType,
    TsConstructorType,
    TsTypeRef,
    TsTypePredicate,
    TsTypeQuery,
    TsImportCallOptions,
    TsImportType,
    TsTypeLit,
    TsArrayType,
    TsTupleType,
    TsTupleElement,
    TsOptionalType,
    TsRestType,
    TsUnionType,
    TsIntersectionType,
    TsConditionalType,
    TsInferType,
    TsParenthesizedType,
    TsTypeOperator,
    TsIndexedAccessType,
    TsMappedType,
    TsLitType,
    TsTplLitType,
    TsInterfaceDecl,
    TsInterfaceBody,
    TsExprWithTypeArgs,
    TsTypeAliasDecl,
    TsEnumDecl,
    TsEnumMember,
    TsModuleDecl,
    TsModuleBlock,
    TsNamespaceDecl,
    TsImportEqualsDecl,
    TsExternalModuleRef,
    TsExportAssginment,
    TsNamespaceExportDecl,
    TsAsExpr,
    TsTypeAssertion,
    TsNonNullExpr,
    TsSatisfiesExpr,
    TsConstAssertion,
    TsInstantiation,
}

impl<'a> Ast<'a> {
    /// Create a flattening AST whose atom storage is backed by `allocator`.
    pub fn new(_source_len: usize, allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            nodes: NodeData::new_in(allocator),
            extra_data: ArenaVec::with_capacity_in(0, allocator),
            atoms: AtomStore::new_in(allocator),
        }
    }

    #[inline]
    pub(crate) fn add_node(&mut self, span: Span, kind: NodeKind, data: NodePayload) -> NodeId {
        self.nodes.push(span, kind, data)
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

    #[inline]
    pub fn add_atom(&self, value: &str) -> Atom<'a> {
        Atom::new_in(value, self.allocator)
    }

    #[inline]
    pub fn add_wtf8_atom(&self, value: &swc_experimental_allocator::wtf8::Wtf8) -> Wtf8Atom<'a> {
        Wtf8Atom::new_in(value, self.allocator)
    }

    #[inline]
    pub fn empty_atom(&self) -> Atom<'a> {
        Atom::empty()
    }

    #[inline]
    pub(crate) fn store_atom(&self, atom: Atom<'a>) -> AtomId {
        self.atoms.store_atom(atom)
    }

    #[inline]
    pub(crate) fn stored_atom(&self, id: AtomId) -> Atom<'a> {
        self.atoms.atom(id)
    }

    #[inline]
    pub(crate) fn store_wtf8_atom(&self, atom: Wtf8Atom<'a>) -> Wtf8AtomId {
        self.atoms.store_wtf8_atom(atom)
    }

    #[inline]
    pub(crate) fn stored_wtf8_atom(&self, id: Wtf8AtomId) -> Wtf8Atom<'a> {
        self.atoms.wtf8_atom(id)
    }

    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
