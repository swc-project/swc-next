use swc_experimental_ast_macros::ast;

use crate::JSXText;

#[ast]
pub enum Lit {
    Str(Str),
    Bool(Bool),
    Null(Null),
    Num(Number),
    BigInt(BigInt),
    Regex(Regex),
    JSXText(JSXText),
}

#[ast]
pub struct Str {
    value: Wtf8Ref,
    raw: OptionalUtf8Ref,
}

#[ast]
pub struct Bool {
    value: bool,
}

#[ast]
pub struct Null {}

#[ast]
pub struct Number {
    value: f64,
    raw: OptionalUtf8Ref,
}

#[ast]
pub struct BigInt {
    value: BigIntId,
    raw: OptionalUtf8Ref,
}

#[ast]
pub struct Regex {
    exp: Utf8Ref,
    flags: Utf8Ref,
}
