use crate::Span;
use swc_experimental_allocator::atom::{Atom, Wtf8Atom};
use swc_experimental_allocator::boxed::Box;
use swc_experimental_ast_macros::ast;

#[ast]
#[derive(Debug)]
pub enum Lit<'a> {
    Str(Box<'a, Str<'a>>),
    Bool(Box<'a, Bool>),
    Null(Box<'a, Null>),
    Num(Box<'a, Number<'a>>),
    BigInt(Box<'a, BigInt<'a>>),
    Regex(Box<'a, Regex<'a>>),
    // JSXText(JSXText),
}

#[ast]
#[derive(Debug)]
pub struct Str<'a> {
    pub span: Span,
    pub value: Wtf8Atom<'a>,
    pub raw: Option<Atom<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct Bool {
    pub span: Span,
    pub value: bool,
}

#[ast]
#[derive(Debug)]
pub struct Null {
    pub span: Span,
}

#[ast]
#[derive(Debug)]
pub struct Number<'a> {
    pub span: Span,
    pub value: f64,
    pub raw: Option<Atom<'a>>,
}

#[ast]
#[derive(Debug)]
pub struct BigInt<'a> {
    pub span: Span,
    pub value: Atom<'a>,
    pub raw: Option<Atom<'a>>,
}

impl BigInt<'_> {
    pub fn to_bigint(&self) -> num_bigint::BigInt {
        num_bigint::BigInt::parse_bytes(self.value.as_bytes(), 10).unwrap()
    }
}

#[ast]
#[derive(Debug)]
pub struct Regex<'a> {
    pub span: Span,
    pub exp: Atom<'a>,
    pub flags: Atom<'a>,
}

// #[ast]
// pub struct JSXText {
//     value: Utf8Ref,
//     raw: Utf8Ref,
// }
