use swc_core::atoms::Atom;
use swc_experimental_ast_macros::ast;

use crate::{Ast, GetSpan};

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

impl Ident {
    pub fn into_binding(self, ast: &mut Ast) -> BindingIdent {
        ast.binding_ident(self.span(ast), self)
    }
}

pub trait EsReserved {
    fn as_str<'a>(&'a self, ast: &'a Ast) -> &'a str;

    #[inline]
    #[rustfmt::skip]
    fn is_reserved(&self, ast: &Ast) -> bool {
        matches!(self.as_str(ast),
            | "break" | "case" | "catch" | "class" | "const" | "continue" | "debugger" | "default"
            | "delete" | "do" | "else" | "enum" | "export" | "extends" | "false" | "finally"
            | "for" | "function" | "if" | "import" | "in" | "instanceof" | "new" | "null"
            | "package" | "return" | "super" | "switch" | "this" | "throw" | "true" | "try"
            | "typeof" | "var" | "void" | "while" | "with"
        )
    }

    #[inline]
    #[rustfmt::skip]
    fn is_reserved_in_strict_mode(&self, ast: &Ast, is_module: bool) -> bool {
        match self.as_str(ast) {
            "await" if is_module => true,
            "implements" | "interface" | "let" | "package" | "private" | "protected" | "public"
            | "static" | "yield" => true,
            _ => false,
        }
    }

    #[inline]
    fn is_reserved_in_strict_bind(&self, ast: &Ast) -> bool {
        matches!(self.as_str(ast), "eval" | "arguments")
    }

    #[inline]
    #[rustfmt::skip]
    fn is_reserved_in_es3(&self, ast: &Ast) -> bool {
        matches!(self.as_str(ast),
            | "abstract" | "boolean" | "byte" | "char" | "double" | "final" | "float" | "goto"
            | "int" | "long" | "native" | "short" | "synchronized" | "throws" | "transient"
            | "volatile"
        )
    }

    #[inline]
    fn is_reserved_in_any(&self, ast: &Ast) -> bool {
        self.is_reserved(ast)
            || self.is_reserved_in_strict_mode(ast, false)
            || self.is_reserved_in_strict_bind(ast)
            || self.is_reserved_in_es3(ast)
    }
}

impl EsReserved for Atom {
    fn as_str(&self, _ast: &Ast) -> &str {
        self.as_str()
    }
}

impl EsReserved for IdentName {
    fn as_str<'a>(&'a self, ast: &'a Ast) -> &'a str {
        ast.get_utf8(self.sym(ast))
    }
}

impl EsReserved for Ident {
    fn as_str<'a>(&'a self, ast: &'a Ast) -> &'a str {
        ast.get_utf8(self.sym(ast))
    }
}

impl EsReserved for BindingIdent {
    fn as_str<'a>(&'a self, ast: &'a Ast) -> &'a str {
        ast.get_utf8(self.id(ast).sym(ast))
    }
}
