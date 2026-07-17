use swc_experimental_ast_macros::ast;

use crate::ast::{BindingIdent, Expr, Invalid};

#[ast]
pub enum Pat {
    Ident(BindingIdent),
    Array(ArrayPat),
    Rest(RestPat),
    Object(ObjectPat),
    Assign(AssignPat),
    Invalid(Invalid),
    Expr(Expr),
}

#[ast]
pub struct ArrayPat {
    elems: Vec<Option<Pat>>,
    optional: bool,
    // type_ann: Option<TsTypeAnn>,
}

#[ast]
pub struct ObjectPat {
    pub props: Vec<ObjectPatProp>,
    pub optional: bool,
    // pub type_ann: Option<Box<TsTypeAnn>>,
}

#[ast]
pub struct AssignPat {
    pub left: Pat,
    pub right: Expr,
}

#[ast]
pub struct RestPat {
    dot3_token: Span,
    arg: Pat,
    // type_ann: Option<Box<TsTypeAnn>>,
}

#[ast]
pub enum ObjectPatProp {
    KeyValue(KeyValuePatProp),
    Assign(AssignPatProp),
    Rest(RestPat),
}

#[ast]
pub struct KeyValuePatProp {
    pub key: PropName,
    pub value: Pat,
}
#[ast]
pub struct AssignPatProp {
    pub key: BindingIdent,
    pub value: Option<Expr>,
}
