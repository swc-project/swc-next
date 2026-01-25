mod class_and_fn;
mod expr;
mod ident;
mod module_item;
mod object;
mod pat;
mod stmt;

use swc_experimental_ecma_ast::*;

pub(crate) fn is_not_this(ast: &Ast, p: Param) -> bool {
    let Pat::Ident(ident) = p.pat(ast) else {
        return true;
    };

    ast.get_utf8(ident.id(ast).sym(ast)) != "this"
}
