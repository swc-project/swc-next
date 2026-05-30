use std::cell::Cell;

use crate::Allocator;
use crate::Span;
use crate::semantic::SymbolId;
use swc_experimental_allocator::atom::Atom;
use swc_experimental_allocator::boxed::Box;
use swc_experimental_ast_macros::ast;

#[ast]
#[derive(Debug)]
pub struct Ident<'a> {
    pub span: Span,
    pub sym: Atom<'a>,
    pub optional: bool,
    pub symbol_id: Cell<Option<SymbolId>>,
}

#[ast]
#[derive(Debug)]
pub struct IdentName<'a> {
    pub span: Span,
    pub sym: Atom<'a>,
}

#[ast]
#[derive(Debug)]
pub struct PrivateName<'a> {
    pub span: Span,
    pub name: Atom<'a>,
}

#[ast]
#[derive(Debug)]
pub struct BindingIdent<'a> {
    pub id: Box<'a, Ident<'a>>,
    // pub type_ann: Option<Box<TsTypeAnn>>,
}

impl<'a> Ident<'a> {
    pub fn into_binding(self, allocator: &'a Allocator) -> BindingIdent<'a> {
        BindingIdent {
            id: allocator.boxed(self),
        }
    }
}

pub trait EsReserved {
    fn as_str(&self) -> &str;

    #[inline]
    #[rustfmt::skip]
    fn is_reserved(&self) -> bool {
        matches!(self.as_str(),
            | "break" | "case" | "catch" | "class" | "const" | "continue" | "debugger" | "default"
            | "delete" | "do" | "else" | "enum" | "export" | "extends" | "false" | "finally"
            | "for" | "function" | "if" | "import" | "in" | "instanceof" | "new" | "null"
            | "package" | "return" | "super" | "switch" | "this" | "throw" | "true" | "try"
            | "typeof" | "var" | "void" | "while" | "with"
        )
    }

    #[inline]
    #[rustfmt::skip]
    fn is_reserved_in_strict_mode(&self, is_module: bool) -> bool {
        match self.as_str() {
            "await" if is_module => true,
            "implements" | "interface" | "let" | "package" | "private" | "protected" | "public"
            | "static" | "yield" => true,
            _ => false,
        }
    }

    #[inline]
    fn is_reserved_in_strict_bind(&self) -> bool {
        matches!(self.as_str(), "eval" | "arguments")
    }

    #[inline]
    #[rustfmt::skip]
    fn is_reserved_in_es3(&self) -> bool {
        matches!(self.as_str(),
            | "abstract" | "boolean" | "byte" | "char" | "double" | "final" | "float" | "goto"
            | "int" | "long" | "native" | "short" | "synchronized" | "throws" | "transient"
            | "volatile"
        )
    }

    #[inline]
    fn is_reserved_in_any(&self) -> bool {
        self.is_reserved()
            || self.is_reserved_in_strict_mode(false)
            || self.is_reserved_in_strict_bind()
            || self.is_reserved_in_es3()
    }
}

impl EsReserved for Atom<'_> {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl EsReserved for IdentName<'_> {
    fn as_str(&self) -> &str {
        self.sym.as_str()
    }
}

impl EsReserved for Ident<'_> {
    fn as_str(&self) -> &str {
        self.sym.as_str()
    }
}

impl EsReserved for BindingIdent<'_> {
    fn as_str(&self) -> &str {
        self.id.sym.as_str()
    }
}
