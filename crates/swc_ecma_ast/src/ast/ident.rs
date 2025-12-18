use swc_experimental_ast_macros::ast;

#[ast]
pub struct Ident {
    sym: Utf8Ref,
    optional: bool,
}

#[ast]
pub struct IdentName {
    sym: Utf8Ref,
}

#[ast]
pub struct PrivateName {
    name: Utf8Ref,
}

#[ast]
pub struct BindingIdent {
    id: Ident,
    // pub type_ann: Option<Box<TsTypeAnn>>,
}

#[inline]
#[rustfmt::skip]
pub fn is_reserved(name: &str) -> bool {
    matches!(name,
        | "break" | "case" | "catch" | "class" | "const" | "continue" | "debugger" | "default"
        | "delete" | "do" | "else" | "enum" | "export" | "extends" | "false" | "finally"
        | "for" | "function" | "if" | "import" | "in" | "instanceof" | "new" | "null"
        | "package" | "return" | "super" | "switch" | "this" | "throw" | "true" | "try"
        | "typeof" | "var" | "void" | "while" | "with"
    )
}

#[inline]
#[rustfmt::skip]
pub fn is_reserved_in_strict_mode(name: &str, is_module: bool) -> bool {
    match name {
        "await" if is_module => true,
        "implements" | "interface" | "let" | "package" | "private" | "protected" | "public"
        | "static" | "yield" => true,
        _ => false,
    }
}

#[inline]
pub fn is_reserved_in_strict_bind(name: &str) -> bool {
    matches!(name, "eval" | "arguments")
}

#[inline]
#[rustfmt::skip]
pub fn is_reserved_in_es3(name: &str) -> bool {
    matches!(name,
        | "abstract" | "boolean" | "byte" | "char" | "double" | "final" | "float" | "goto"
        | "int" | "long" | "native" | "short" | "synchronized" | "throws" | "transient"
        | "volatile"
    )
}

#[inline]
pub fn is_reserved_in_any(name: &str) -> bool {
    is_reserved(name)
        || is_reserved_in_strict_mode(name, false)
        || is_reserved_in_strict_bind(name)
        || is_reserved_in_es3(name)
}
