use swc_experimental_ast_macros::ast;

use crate::{
    Number,
    ast::{BigInt, Ident, IdentName, Str},
};

#[ast]
pub enum Prop {
    Shorthand(Ident),
    KeyValue(KeyValueProp),
    Assign(AssignProp),
    Getter(GetterProp),
    Setter(SetterProp),
    Method(MethodProp),
}

#[ast]
pub struct KeyValueProp {
    key: PropName,
    value: Expr,
}

#[ast]
pub struct AssignProp {
    key: Ident,
    value: Expr,
}

#[ast]
pub struct GetterProp {
    key: PropName,
    function: Function,
}
#[ast]
pub struct SetterProp {
    key: PropName,
    function: Function,
}
#[ast]
pub struct MethodProp {
    key: PropName,
    function: Function,
}

#[ast]
pub enum PropName {
    Ident(IdentName),
    Str(Str),
    Num(Number),
    Computed(ComputedPropName),
    BigInt(BigInt),
}

#[ast]
pub struct ComputedPropName {
    expr: Expr,
}
