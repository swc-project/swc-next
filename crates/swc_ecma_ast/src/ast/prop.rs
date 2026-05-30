use crate::Span;
use swc_experimental_allocator::boxed::Box;
use swc_experimental_ast_macros::ast;

use crate::ast::{BigInt, BlockStmt, Expr, Function, Ident, IdentName, Number, Pat, Str};

#[ast]
#[derive(Debug)]
pub enum Prop<'a> {
    Shorthand(Box<'a, Ident<'a>>),
    KeyValue(Box<'a, KeyValueProp<'a>>),
    Assign(Box<'a, AssignProp<'a>>),
    Getter(Box<'a, GetterProp<'a>>),
    Setter(Box<'a, SetterProp<'a>>),
    Method(Box<'a, MethodProp<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct KeyValueProp<'a> {
    pub key: PropName<'a>,
    pub value: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct AssignProp<'a> {
    pub span: Span,
    pub key: Box<'a, Ident<'a>>,
    pub value: Expr<'a>,
}

#[ast]
#[derive(Debug)]
pub struct GetterProp<'a> {
    pub span: Span,
    pub key: PropName<'a>,
    // type_ann: Option<TsTypeAnn>,
    pub body: Option<Box<'a, BlockStmt<'a>>>,
}
#[ast]
#[derive(Debug)]
pub struct SetterProp<'a> {
    pub span: Span,
    pub key: PropName<'a>,
    pub this_param: Option<Pat<'a>>,
    pub param: Pat<'a>,
    pub body: Option<Box<'a, BlockStmt<'a>>>,
}
#[ast]
#[derive(Debug)]
pub struct MethodProp<'a> {
    pub key: PropName<'a>,
    pub function: Box<'a, Function<'a>>,
}

#[ast]
#[derive(Debug)]
pub enum PropName<'a> {
    Ident(Box<'a, IdentName<'a>>),
    Str(Box<'a, Str<'a>>),
    Num(Box<'a, Number<'a>>),
    Computed(Box<'a, ComputedPropName<'a>>),
    BigInt(Box<'a, BigInt<'a>>),
}

#[ast]
#[derive(Debug)]
pub struct ComputedPropName<'a> {
    pub span: Span,
    pub expr: Expr<'a>,
}
