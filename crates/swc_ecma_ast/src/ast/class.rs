use std::mem;

use swc_experimental_ast_macros::ast;

use crate::{Ast, ExtraData, ast::*, node_id::ExtraDataCompact};

#[ast]
pub struct Class {
    decorators: Vec<Decorator>,
    body: Vec<ClassMember>,
    super_class: Option<Expr>,
    is_abstract: bool,
    // type_params: Option<Box<TsTypeParamDecl>>,
    // super_type_params: Option<Box<TsTypeParamInstantiation>>,
    // implements: Vec<TsExprWithTypeArgs>,
}

#[ast]
pub enum ClassMember {
    Constructor(Constructor),
    Method(ClassMethod),
    PrivateMethod(PrivateMethod),
    ClassProp(ClassProp),
    PrivateProp(PrivateProp),
    Empty(EmptyStmt),
    StaticBlock(StaticBlock),
    AutoAccessor(AutoAccessor),
    // TsIndexSignature(TsIndexSignature),
}

#[ast]
pub struct ClassProp {
    key: PropName,
    value: Option<Expr>,
    // type_ann: Option<Box<TsTypeAnn>>,
    is_static: bool,
    decorators: Vec<Decorator>,
    // accessibility: Option<Accessibility>,
    // is_abstract: bool,
    // is_optional: bool,
    // is_override: bool,
    // readonly: bool,
    // declare: bool,
    // definite: bool,
}

#[ast]
pub struct PrivateProp {
    key: PrivateName,
    value: Option<Expr>,
    // type_ann: Option<TsTypeAnn>,
    is_static: bool,
    decorators: Vec<Decorator>,
    // accessibility: Option<Accessibility>,
    // is_optional: bool,
    // is_override: bool,
    // readonly: bool,
    // definite: bool,
}

#[ast]
pub struct ClassMethod {
    key: PropName,
    function: Function,
    kind: MethodKind,
    is_static: bool,
    // accessibility: Option<Accessibility>,
    // is_abstract: bool,
    // is_optional: bool,
    // is_override: bool,
}

#[ast]
pub struct PrivateMethod {
    key: PrivateName,
    function: Function,
    kind: MethodKind,
    is_static: bool,
    // accessibility: Option<Accessibility>,
    // is_abstract: bool,
    // is_optional: bool,
    // is_override: bool,
}

#[ast]
pub struct Constructor {
    key: PropName,
    params: Vec<ParamOrTsParamProp>,
    body: Option<BlockStmt>,
    // accessibility: Option<Accessibility>,
    // is_optional: bool,
}

#[ast]
pub struct Decorator {
    expr: Expr,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodKind {
    Method,
    Getter,
    Setter,
}

impl ExtraDataCompact for MethodKind {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[ast]
pub struct StaticBlock {
    body: BlockStmt,
}

#[ast]
pub enum Key {
    Private(PrivateName),
    Public(PropName),
}

#[ast]
pub struct AutoAccessor {
    key: Key,
    value: Option<Expr>,
    // type_ann: Option<TsTypeAnn>,
    is_static: bool,
    decorators: Vec<Decorator>,
    // accessibility: Option<Accessibility>,
    // is_abstract: bool,
    // is_override: bool,
    // definite: bool,
}
