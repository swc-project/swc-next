mod class_and_fn;
mod expr;
mod ident;
mod module_item;
mod object;
mod pat;
mod stmt;

use swc_experimental_ecma_ast::*;

pub(crate) fn is_not_this(p: &Param<'_>) -> bool {
    let Pat::Ident(ident) = &p.pat else {
        return true;
    };

    ident.id.sym != "this"
}
