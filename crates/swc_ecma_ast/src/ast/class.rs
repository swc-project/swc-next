use crate::Span;
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ast_macros::ast;

use crate::{BlockStmt, EmptyStmt, Expr, Function, ParamOrTsParamProp, PrivateName, PropName};

#[ast]
#[derive(Debug)]
pub struct Class<'a> {
    pub span: Span,
    pub decorators: Vec<'a, Decorator<'a>>,
    pub body: Vec<'a, ClassMember<'a>>,
    pub super_class: Option<Expr<'a>>,
    pub is_abstract: bool,
    // type_params: Option<Box<TsTypeParamDecl>>,
    // super_type_params: Option<Box<TsTypeParamInstantiation>>,
    // implements: Vec<TsExprWithTypeArgs>,
}

#[ast]
#[derive(Debug)]
pub enum ClassMember<'a> {
    Constructor(Box<'a, Constructor<'a>>),
    Method(Box<'a, ClassMethod<'a>>),
    PrivateMethod(Box<'a, PrivateMethod<'a>>),
    ClassProp(Box<'a, ClassProp<'a>>),
    PrivateProp(Box<'a, PrivateProp<'a>>),
    Empty(Box<'a, EmptyStmt>),
    StaticBlock(Box<'a, StaticBlock<'a>>),
    AutoAccessor(Box<'a, AutoAccessor<'a>>),
    // TsIndexSignature(TsIndexSignature),
}

#[ast]
#[derive(Debug)]
pub struct ClassProp<'a> {
    pub span: Span,
    pub key: PropName<'a>,
    pub value: Option<Expr<'a>>,
    // type_ann: Option<Box<TsTypeAnn>>,
    pub is_static: bool,
    pub decorators: Vec<'a, Decorator<'a>>,
    // accessibility: Option<Accessibility>,
    // is_abstract: bool,
    // is_optional: bool,
    // is_override: bool,
    // readonly: bool,
    // declare: bool,
    // definite: bool,
}

#[ast]
#[derive(Debug)]
pub struct PrivateProp<'a> {
    pub span: Span,
    pub key: Box<'a, PrivateName<'a>>,
    pub value: Option<Expr<'a>>,
    // type_ann: Option<TsTypeAnn>,
    pub is_static: bool,
    pub decorators: Vec<'a, Decorator<'a>>,
    // accessibility: Option<Accessibility>,
    // is_optional: bool,
    // is_override: bool,
    // readonly: bool,
    // definite: bool,
}

#[ast]
#[derive(Debug)]
pub struct ClassMethod<'a> {
    pub span: Span,
    pub key: PropName<'a>,
    pub function: Box<'a, Function<'a>>,
    pub kind: MethodKind,
    pub is_static: bool,
    // accessibility: Option<Accessibility>,
    // is_abstract: bool,
    // is_optional: bool,
    // is_override: bool,
}

#[ast]
#[derive(Debug)]
pub struct PrivateMethod<'a> {
    pub span: Span,
    pub key: Box<'a, PrivateName<'a>>,
    pub function: Box<'a, Function<'a>>,
    pub kind: MethodKind,
    pub is_static: bool,
    // accessibility: Option<Accessibility>,
    // is_abstract: bool,
    // is_optional: bool,
    // is_override: bool,
}

#[ast]
#[derive(Debug)]
pub struct Constructor<'a> {
    pub span: Span,
    pub key: PropName<'a>,
    pub params: Vec<'a, ParamOrTsParamProp<'a>>,
    pub body: Option<Box<'a, BlockStmt<'a>>>,
    // accessibility: Option<Accessibility>,
    // is_optional: bool,
}

#[ast]
#[derive(Debug)]
pub struct Decorator<'a> {
    pub span: Span,
    pub expr: Expr<'a>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodKind {
    Method,
    Getter,
    Setter,
}

#[ast]
#[derive(Debug)]
pub struct StaticBlock<'a> {
    pub span: Span,
    pub body: Box<'a, BlockStmt<'a>>,
}

#[ast]
#[derive(Debug)]
pub enum Key<'a> {
    Private(Box<'a, PrivateName<'a>>),
    Public(Box<'a, PropName<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct AutoAccessor<'a> {
    pub span: Span,
    pub key: Key<'a>,
    pub value: Option<Expr<'a>>,
    // type_ann: Option<TsTypeAnn>,
    pub is_static: bool,
    pub decorators: Vec<'a, Decorator<'a>>,
    // accessibility: Option<Accessibility>,
    // is_abstract: bool,
    // is_override: bool,
    // definite: bool,
}
