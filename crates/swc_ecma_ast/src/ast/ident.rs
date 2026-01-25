use swc_experimental_ast_macros::ast;

use crate::{
    Ast, Utf8Ref,
    atom::{
        UTF8_ABSTRACT, UTF8_ARGUMENTS, UTF8_AWAIT, UTF8_BOOLEAN, UTF8_BREAK, UTF8_BYTE, UTF8_CASE,
        UTF8_CATCH, UTF8_CHAR, UTF8_CLASS, UTF8_CONST, UTF8_CONTINUE, UTF8_DEBUGGER, UTF8_DEFAULT,
        UTF8_DELETE, UTF8_DO, UTF8_DOUBLE, UTF8_ELSE, UTF8_ENUM, UTF8_EVAL, UTF8_EXPORT,
        UTF8_EXTENDS, UTF8_FALSE, UTF8_FINAL, UTF8_FINALLY, UTF8_FLOAT, UTF8_FOR, UTF8_FUNCTION,
        UTF8_GOTO, UTF8_IF, UTF8_IMPLEMENTS, UTF8_IMPORT, UTF8_IN, UTF8_INSTANCEOF, UTF8_INT,
        UTF8_INTERFACE, UTF8_LET, UTF8_LONG, UTF8_NATIVE, UTF8_NEW, UTF8_NULL, UTF8_PACKAGE,
        UTF8_PRIVATE, UTF8_PROTECTED, UTF8_PUBLIC, UTF8_RETURN, UTF8_SHORT, UTF8_STATIC,
        UTF8_SUPER, UTF8_SWITCH, UTF8_SYNCHRONIZED, UTF8_THIS, UTF8_THROW, UTF8_THROWS,
        UTF8_TRANSIENT, UTF8_TRUE, UTF8_TRY, UTF8_TYPEOF, UTF8_VAR, UTF8_VOID, UTF8_VOLATILE,
        UTF8_WHILE, UTF8_WITH, UTF8_YIELD,
    },
};

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

pub trait EsReserved {
    fn as_utf8_ref(&self, ast: &Ast) -> Utf8Ref;

    #[inline]
    #[rustfmt::skip]
    fn is_reserved(&self, ast: &Ast) -> bool {
        const RESERVED: &[Utf8Ref] = &[
            UTF8_BREAK, UTF8_CASE, UTF8_CATCH, UTF8_CLASS, UTF8_CONST, UTF8_CONTINUE,
            UTF8_DEBUGGER, UTF8_DEFAULT, UTF8_DELETE, UTF8_DO, UTF8_ELSE, UTF8_ENUM, UTF8_EXPORT,
            UTF8_EXTENDS, UTF8_FALSE, UTF8_FINALLY, UTF8_FOR, UTF8_FUNCTION, UTF8_IF, UTF8_IMPORT,
            UTF8_IN, UTF8_INSTANCEOF, UTF8_NEW, UTF8_NULL, UTF8_PACKAGE, UTF8_RETURN, UTF8_SUPER,
            UTF8_SWITCH, UTF8_THIS, UTF8_THROW, UTF8_TRUE, UTF8_TRY, UTF8_TYPEOF, UTF8_VAR,
            UTF8_VOID, UTF8_WHILE, UTF8_WITH,
        ];
        RESERVED.contains(&self.as_utf8_ref(ast))
    }

    #[inline]
    #[rustfmt::skip]
    fn is_reserved_in_strict_mode(&self, ast: &Ast, is_module: bool) -> bool {
        const RESERVED: &[Utf8Ref] = &[
            UTF8_IMPLEMENTS, UTF8_INTERFACE, UTF8_LET, UTF8_PACKAGE, UTF8_PRIVATE,
            UTF8_PROTECTED, UTF8_PUBLIC, UTF8_STATIC, UTF8_YIELD,
        ];

        let sym = self.as_utf8_ref(ast);

        if is_module && sym == UTF8_AWAIT {
            return true;
        }

        RESERVED.contains(&sym)
    }

    #[inline]
    fn is_reserved_in_strict_bind(&self, ast: &Ast) -> bool {
        const RESERVED: &[Utf8Ref] = &[UTF8_EVAL, UTF8_ARGUMENTS];
        RESERVED.contains(&self.as_utf8_ref(ast))
    }

    #[inline]
    #[rustfmt::skip]
    fn is_reserved_in_es3(&self, ast: &Ast) -> bool {
        const RESERVED: &[Utf8Ref] = &[
            UTF8_ABSTRACT, UTF8_BOOLEAN, UTF8_BYTE, UTF8_CHAR, UTF8_DOUBLE,
            UTF8_FINAL, UTF8_FLOAT, UTF8_GOTO, UTF8_INT, UTF8_LONG, UTF8_NATIVE,
            UTF8_SHORT, UTF8_SYNCHRONIZED, UTF8_THROWS, UTF8_TRANSIENT, UTF8_VOLATILE,
        ];
        RESERVED.contains(&self.as_utf8_ref(ast))
    }

    #[inline]
    fn is_reserved_in_any(&self, ast: &Ast) -> bool {
        self.is_reserved(ast)
            || self.is_reserved_in_strict_mode(ast, false)
            || self.is_reserved_in_strict_bind(ast)
            || self.is_reserved_in_es3(ast)
    }
}

impl EsReserved for Utf8Ref {
    fn as_utf8_ref(&self, _ast: &Ast) -> Utf8Ref {
        *self
    }
}

impl EsReserved for IdentName {
    fn as_utf8_ref(&self, ast: &Ast) -> Utf8Ref {
        self.sym(ast)
    }
}

impl EsReserved for Ident {
    fn as_utf8_ref(&self, ast: &Ast) -> Utf8Ref {
        self.sym(ast)
    }
}

impl EsReserved for BindingIdent {
    fn as_utf8_ref(&self, ast: &Ast) -> Utf8Ref {
        self.id(ast).sym(ast)
    }
}
