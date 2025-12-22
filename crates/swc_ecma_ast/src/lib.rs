mod assert_layout;
mod ast;
mod ast_list;
mod common;
mod derive;
mod node_id;
mod string_allocator;
mod visit;

mod generated {
    mod ast_builder;
    mod ast_clone_in;
    mod ast_node_id;
    mod ast_property;
    pub(crate) mod ast_visitor;
}

use std::marker::PhantomData;
use swc_core::atoms::wtf8::Wtf8;
use swc_core::common::BytePos;

use num_bigint::BigInt as BigIntValue;
use oxc_index::IndexVec;

pub use ast::*;
pub use common::*;
pub use derive::*;
pub use generated::ast_visitor::*;
pub use node_id::{
    BigIntId, ExtraDataId, NodeId, NodeIdTrait, OptionalNodeId, OptionalUtf8Ref, OptionalWtf8Ref,
    SubRange, TypedSubRange, Utf8Ref, Wtf8Ref,
};

use crate::{ast_list::NodeList, node_id::OptionalSubRange, string_allocator::StringAllocator};

/// AST context that stores everything about the flattening AST.
pub struct Ast {
    /// Flattening AST nodes.
    nodes: NodeList,

    /// Flattening AST fields.
    extra_data: IndexVec<ExtraDataId, ExtraData>,

    /// Mapping from BigIntId to BigIntValue.
    /// This is used to reduce the size of [BigIntValue] to store its Id in the AST.
    bigint: IndexVec<BigIntId, BigIntValue>,

    /// The AST doesn't directly store strings (UTF-8 and WTF-8).
    /// Instead, it stores the start and the end index of the string in the string allocator.
    /// See: [StringAllocator]
    string_allocator: StringAllocator,
}

/// A 24-bit unsigned integer stored as 3 bytes.
/// Used for inline storage in AstNode.
#[repr(transparent)]
#[derive(Clone, Copy, Default)]
pub struct U24([u8; 3]);

impl From<u32> for U24 {
    /// Create a U24 from the lower 24 bits of a u32.
    #[inline]
    fn from(val: u32) -> Self {
        Self([
            (val & 0xFF) as u8,
            ((val >> 8) & 0xFF) as u8,
            ((val >> 16) & 0xFF) as u8,
        ])
    }
}

impl From<U24> for u32 {
    /// Zero-extend U24 to u32.
    #[inline]
    fn from(val: U24) -> u32 {
        (val.0[0] as u32) | ((val.0[1] as u32) << 8) | ((val.0[2] as u32) << 16)
    }
}

/// Untyped AST node
#[derive(Clone)]
pub struct AstNode {
    span: Span,
    kind: NodeKind,
    inline_data: U24,
    data: NodeData,
}

impl AstNode {
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    #[inline]
    pub fn set_span(&mut self, span: Span) {
        self.span = span;
    }

    #[inline]
    pub fn kind(&self) -> NodeKind {
        self.kind
    }
}

/// Node data is the start index of [Ast::extra_data], if the node is not freed, which indicates the first field of the AST node.
/// Otherwise it's the next free node id, forming a free list of AST.
///
/// We use union here to eliminate the tag cost of enum.
///
/// # Safety:
/// It is only access the by the property accesses of typed AST node, and the construction of typed AST is
/// another safety guarantee.
#[derive(Clone, Copy)]
pub union NodeData {
    empty: (),
    extra_data_start: ExtraDataId,
    inline_data: u32,
    next_free: OptionalNodeId,
}

/// The extra data is used to represent the field of AST node.
/// It's usally another Id to point to the data structures in [Ast],
/// or it can be a primitive value, which is small enough to store in 8 bytes.
///
/// # Safety:
/// It is only access the by the property accesses of typed AST node, and the construction of typed AST is
/// another safety guarantee.
pub union ExtraData {
    // Ids
    node: NodeId,
    bigint: BigIntId,
    utf8: Utf8Ref,
    wtf8: Wtf8Ref,
    optional_node: OptionalNodeId,
    optional_utf8: OptionalUtf8Ref,
    optional_wtf8: OptionalWtf8Ref,

    // Primitives
    span: Span,
    bool: bool,
    number: f64,
    sub_range: SubRange,
    optional_sub_range: OptionalSubRange,

    // Typed enums
    expr: Expr,
    stmt: Stmt,
    decl: Decl,
    pat: Pat,
    default_decl: DefaultDecl,
    opt_chain_base: OptChainBase,
    module_export_name: ModuleExportName,
    for_head: ForHead,
    member_prop: MemberProp,
    super_prop: SuperProp,
    callee: Callee,
    block_stmt_or_expr: BlockStmtOrExpr,
    prop_name: PropName,
    key: Key,
    jsx_object: JSXObject,
    jsx_element_name: JSXElementName,
    jsx_attr_name: JSXAttrName,
    jsx_expr: JSXExpr,

    optional_expr: Option<Expr>,
    optional_pat: Option<Pat>,
    optional_stmt: Option<Stmt>,
    optional_module_export_name: Option<ModuleExportName>,
    optional_var_decl_or_expr: Option<VarDeclOrExpr>,
    optional_jsx_attr_value: Option<JSXAttrValue>,

    /// Any other data (usually enum) that can be representated within 8 bytes
    other: u64,
}

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
    SpreadDot3Token,
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
    Import,
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

    /// This is a special node kind to mark a node as freed.
    __FREED,
}

impl Ast {
    /// Create a new AST with the given source length.
    /// The source length is used to pre-allocate memory for the string allocator.
    pub fn new(source_len: usize) -> Self {
        Self {
            nodes: NodeList::default(),
            extra_data: IndexVec::new(),
            bigint: IndexVec::new(),
            string_allocator: StringAllocator::new(source_len),
        }
    }

    #[inline]
    fn add_node(&mut self, node: AstNode) -> NodeId {
        self.nodes.add_node(node)
    }

    #[inline]
    fn add_extra(&mut self, extra: ExtraData) -> ExtraDataId {
        self.extra_data.push(extra)
    }

    #[inline]
    pub fn add_typed_sub_range<N: NodeIdTrait>(&mut self, range: &[NodeId]) -> TypedSubRange<N> {
        let start = self.extra_data.next_idx();
        self.extra_data
            .extend(range.iter().map(|n| ExtraData { node: *n }));
        TypedSubRange {
            inner: SubRange {
                start,
                end: self.extra_data.next_idx(),
            },
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn add_typed_opt_sub_range<N: NodeIdTrait>(
        &mut self,
        range: &[OptionalNodeId],
    ) -> TypedSubRange<Option<N>> {
        let start = self.extra_data.next_idx();
        self.extra_data
            .extend(range.iter().map(|n| ExtraData { optional_node: *n }));
        TypedSubRange {
            inner: SubRange {
                start,
                end: self.extra_data.next_idx(),
            },
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn add_utf8(&mut self, s: &str) -> Utf8Ref {
        self.string_allocator.add_utf8(s)
    }

    #[inline]
    pub fn add_optional_utf8(&mut self, s: Option<&str>) -> OptionalUtf8Ref {
        self.string_allocator.add_optional_utf8(s)
    }

    #[inline]
    pub fn add_wtf8(&mut self, s: &Wtf8) -> Wtf8Ref {
        self.string_allocator.add_wtf8(s)
    }

    #[inline]
    pub fn add_optional_wtf8(&mut self, s: Option<&Wtf8>) -> OptionalWtf8Ref {
        self.string_allocator.add_optional_wtf8(s)
    }

    #[inline]
    pub fn get_utf8(&self, id: Utf8Ref) -> &str {
        self.string_allocator.get_utf8(id)
    }

    #[inline]
    pub fn get_optional_utf8(&self, id: OptionalUtf8Ref) -> Option<&str> {
        self.string_allocator.get_optional_utf8(id)
    }

    #[inline]
    pub fn get_wtf8(&self, id: Wtf8Ref) -> &Wtf8 {
        self.string_allocator.get_wtf8(id)
    }

    #[inline]
    pub fn get_optional_wtf8(&self, id: OptionalWtf8Ref) -> Option<&Wtf8> {
        self.string_allocator.get_optional_wtf8(id)
    }

    #[inline]
    pub fn add_bigint(&mut self, big_int: BigIntValue) -> BigIntId {
        self.bigint.push(big_int)
    }

    #[inline]
    pub fn get_big_int(&self, big_int_id: BigIntId) -> &BigIntValue {
        &self.bigint[big_int_id]
    }

    #[inline]
    pub fn into_string_allocator(self) -> StringAllocator {
        self.string_allocator
    }
}

impl Ast {
    #[inline]
    pub fn free_node(&mut self, node_id: NodeId) {
        self.nodes.free_node(node_id);
    }

    #[inline]
    pub fn replace_node<T: NodeIdTrait>(&mut self, dest: T, source: T) {
        if source.node_id() != dest.node_id() {
            self.nodes.replace_node(dest.node_id(), source.node_id());
        }
    }

    #[inline]
    pub fn get_node(&self, node_id: NodeId) -> &AstNode {
        &self.nodes[node_id]
    }

    #[inline]
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &AstNode)> {
        self.nodes.iter()
    }

    #[inline]
    pub fn nodes_len(&self) -> u32 {
        self.nodes.len()
    }

    #[inline]
    pub fn nodes_capacity(&self) -> usize {
        self.nodes.capacity()
    }
}
