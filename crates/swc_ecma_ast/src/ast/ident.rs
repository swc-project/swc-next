use swc_experimental_ast_macros::ast;

use crate::{Ast, Atom, GetSpan};

#[ast]
pub struct Ident {
    sym: Atom,
    symbol_id: Cell<Option<SymbolId>>,
}

#[ast]
pub struct IdentName {
    sym: Atom,
}

#[ast]
pub struct PrivateName {
    name: Atom,
}

#[ast]
pub struct BindingIdent {
    id: Ident,
    // pub type_ann: Option<Box<TsTypeAnn>>,
}

impl Ident {
    pub fn into_binding(self, ast: &mut Ast<'_>) -> BindingIdent {
        ast.binding_ident(self.span(ast), self)
    }
}

pub trait EsReserved<'a>: Copy {
    fn atom(self, ast: &Ast<'a>) -> Atom<'a>;

    #[inline]
    #[rustfmt::skip]
    fn is_reserved(self, ast: &Ast<'a>) -> bool {
        matches!(self.atom(ast).as_str(),
            | "break" | "case" | "catch" | "class" | "const" | "continue" | "debugger" | "default"
            | "delete" | "do" | "else" | "enum" | "export" | "extends" | "false" | "finally"
            | "for" | "function" | "if" | "import" | "in" | "instanceof" | "new" | "null"
            | "package" | "return" | "super" | "switch" | "this" | "throw" | "true" | "try"
            | "typeof" | "var" | "void" | "while" | "with"
        )
    }

    #[inline]
    #[rustfmt::skip]
    fn is_reserved_in_strict_mode(self, ast: &Ast<'a>, is_module: bool) -> bool {
        match self.atom(ast).as_str() {
            "await" if is_module => true,
            "implements" | "interface" | "let" | "package" | "private" | "protected" | "public"
            | "static" | "yield" => true,
            _ => false,
        }
    }

    #[inline]
    fn is_reserved_in_strict_bind(self, ast: &Ast<'a>) -> bool {
        matches!(self.atom(ast).as_str(), "eval" | "arguments")
    }

    #[inline]
    #[rustfmt::skip]
    fn is_reserved_in_es3(self, ast: &Ast<'a>) -> bool {
        matches!(self.atom(ast).as_str(),
            | "abstract" | "boolean" | "byte" | "char" | "double" | "final" | "float" | "goto"
            | "int" | "long" | "native" | "short" | "synchronized" | "throws" | "transient"
            | "volatile"
        )
    }

    #[inline]
    fn is_reserved_in_any(self, ast: &Ast<'a>) -> bool {
        self.is_reserved(ast)
            || self.is_reserved_in_strict_mode(ast, false)
            || self.is_reserved_in_strict_bind(ast)
            || self.is_reserved_in_es3(ast)
    }
}

impl<'a> EsReserved<'a> for Atom<'a> {
    fn atom(self, _ast: &Ast<'a>) -> Atom<'a> {
        self
    }
}

impl<'a> EsReserved<'a> for IdentName {
    fn atom(self, ast: &Ast<'a>) -> Atom<'a> {
        self.sym(ast)
    }
}

impl<'a> EsReserved<'a> for Ident {
    fn atom(self, ast: &Ast<'a>) -> Atom<'a> {
        self.sym(ast)
    }
}

impl<'a> EsReserved<'a> for BindingIdent {
    fn atom(self, ast: &Ast<'a>) -> Atom<'a> {
        self.id(ast).sym(ast)
    }
}
