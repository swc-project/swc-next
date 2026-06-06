use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
use swc_experimental_ast_macros::ast;

use crate::{
    Span,
    ast::{BindingIdent, Expr, Invalid, PropName},
};

#[ast]
#[derive(Debug)]
pub enum Pat<'a> {
    Ident(Box<'a, BindingIdent<'a>>),
    Array(Box<'a, ArrayPat<'a>>),
    Rest(Box<'a, RestPat<'a>>),
    Object(Box<'a, ObjectPat<'a>>),
    Assign(Box<'a, AssignPat<'a>>),
    Invalid(Box<'a, Invalid>),
    Expr(Box<'a, Expr<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct ArrayPat<'a> {
    pub span: Span,
    pub elems: Vec<'a, Option<Pat<'a>>>,
    pub optional: bool,
    // type_ann: Option<TsTypeAnn>,
}

#[ast]
#[derive(Debug)]
pub struct ObjectPat<'a> {
    pub span: Span,
    pub props: Vec<'a, ObjectPatProp<'a>>,
    pub optional: bool,
    // pub type_ann: Option<Box<TsTypeAnn>>,
}

#[ast]
#[derive(Debug)]
pub struct AssignPat<'a> {
    pub span: Span,
    pub left: Pat<'a>,
    pub right: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct RestPat<'a> {
    pub span: Span,
    pub dot3_token: Span,
    pub arg: Pat<'a>,
    // type_ann: Option<Box<TsTypeAnn>>,
}

#[ast]
#[derive(Debug)]
pub enum ObjectPatProp<'a> {
    KeyValue(Box<'a, KeyValuePatProp<'a>>),
    Assign(Box<'a, AssignPatProp<'a>>),
    Rest(Box<'a, RestPat<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct KeyValuePatProp<'a> {
    #[span(lo)]
    pub key: PropName<'a>,
    #[span(hi)]
    pub value: Pat<'a>,
}
#[ast]
#[derive(Debug)]
pub struct AssignPatProp<'a> {
    pub span: Span,
    pub key: Box<'a, BindingIdent<'a>>,
    pub value: Option<Expr<'a>>,
}
