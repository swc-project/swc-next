use swc_experimental_ast_macros::ast;

#[ast]
pub enum Lit {
    Str(Str),
    Bool(Bool),
    Null(Null),
    Num(Number),
    BigInt(BigInt),
    Regex(Regex),
    // JSXText(JSXText),
}

#[ast]
pub struct Str {
    value: Wtf8Atom,
    raw: Option<Atom>,
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
    raw: Option<Atom>,
}

#[ast]
pub struct BigInt {
    value: Atom,
    raw: Option<Atom>,
}

#[ast]
pub struct Regex {
    exp: Atom,
    flags: Atom,
}

impl BigInt {
    pub fn to_bigint(self, ast: &crate::Ast<'_>) -> num_bigint::BigInt {
        num_bigint::BigInt::parse_bytes(self.value(ast).as_bytes(ast), 10).unwrap()
    }
}
