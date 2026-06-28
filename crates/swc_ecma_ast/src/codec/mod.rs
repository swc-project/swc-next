use std::mem;

use crate::{Encoder, codec::decode::Decoder};

pub mod generated {
    pub mod decode;
    pub mod encode;
    pub mod measure;
}

pub mod decode;
pub mod encode;
pub mod measure;
pub mod utf8_to_utf16;

const HEADER_SIZE: usize = mem::size_of::<Header>();
const NODE_SIZE: usize = mem::size_of::<PackedNode>();
const EXTRA_SIZE: usize = mem::size_of::<u32>();
const NONE: u32 = u32::MAX;

pub(crate) trait AstEncode {
    fn encode(&self, encoder: &mut Encoder) -> u32;
}

pub(crate) trait AstDecode<'a>: Sized {
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self;
}

pub(crate) trait AstMeasure {
    fn measure(&self, header: &mut Header);
}

#[repr(C)]
#[derive(Debug)]
pub struct Header {
    node_count: u32,
    extra_count: u32,
    string_pool_size: u32,
}

impl Header {
    pub fn size(&self) -> usize {
        HEADER_SIZE
            + (self.node_count as usize) * NODE_SIZE
            + (self.extra_count as usize) * EXTRA_SIZE
            + (self.string_pool_size as usize)
    }
}

#[repr(C)]
#[derive(Default)]
struct PackedNode {
    kind: NodeKind,
    _pad0: u8,
    flags: u16,
    span_lo: u32,
    span_hi: u32,
    field0: u32,
    field1: u32,
    field2: u32,
    field3: u32,
    field4: u32,
}

#[repr(u8)]
#[derive(Default)]
enum NodeKind {
    #[default]
    Unknown = 0,
    ArrayLit,
    ArrayPat,
    ArrowExpr,
    AssignExpr,
    AssignPat,
    AssignPatProp,
    AssignProp,
    AutoAccessor,
    AwaitExpr,
    BigInt,
    BinExpr,
    BindingIdent,
    BlockStmt,
    Bool,
    BreakStmt,
    CallExpr,
    CatchClause,
    Class,
    ClassDecl,
    ClassExpr,
    ClassMethod,
    ClassProp,
    ComputedPropName,
    CondExpr,
    Constructor,
    ContinueStmt,
    DebuggerStmt,
    Decorator,
    DoWhileStmt,
    EmptyStmt,
    ExportAll,
    ExportDecl,
    ExportDefaultDecl,
    ExportDefaultExpr,
    ExportDefaultSpecifier,
    ExportNamedSpecifier,
    ExportNamespaceSpecifier,
    ExprOrSpread,
    ExprStmt,
    FnDecl,
    FnExpr,
    ForInStmt,
    ForOfStmt,
    ForStmt,
    Function,
    GetterProp,
    Ident,
    IdentName,
    IfStmt,
    ImportDecl,
    ImportDefaultSpecifier,
    ImportExpr,
    ImportNamedSpecifier,
    ImportStarAsSpecifier,
    Invalid,
    JSXAttr,
    JSXClosingElement,
    JSXClosingFragment,
    JSXElement,
    JSXEmptyExpr,
    JSXExprContainer,
    JSXFragment,
    JSXMemberExpr,
    JSXNamespacedName,
    JSXOpeningElement,
    JSXOpeningFragment,
    JSXSpreadChild,
    JSXText,
    KeyValuePatProp,
    KeyValueProp,
    LabeledStmt,
    MemberExpr,
    MetaPropExpr,
    MethodProp,
    Module,
    NamedExport,
    NewExpr,
    Null,
    Number,
    ObjectLit,
    ObjectPat,
    OptCall,
    OptChainExpr,
    Param,
    ParenExpr,
    PrivateMethod,
    PrivateName,
    PrivateProp,
    Regex,
    RestPat,
    ReturnStmt,
    Script,
    SeqExpr,
    SetterProp,
    SpreadElement,
    StaticBlock,
    Str,
    Super,
    SuperPropExpr,
    SwitchCase,
    SwitchStmt,
    TaggedTpl,
    ThisExpr,
    ThrowStmt,
    Tpl,
    TplElement,
    TryStmt,
    UnaryExpr,
    UpdateExpr,
    UsingDecl,
    VarDecl,
    VarDeclarator,
    WhileStmt,
    WithStmt,
    YieldExpr,
}
