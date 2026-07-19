mod class_and_fn;
mod expr;
mod ident;
mod module_item;
mod object;
mod pat;
mod stmt;

use swc_experimental_ecma_ast::*;

pub(crate) fn is_not_this(ast: &Ast, p: Param) -> bool {
    let PatData::Ident(ident) = ast.pat_data(p.pat(ast)) else {
        return true;
    };

    ident.id(ast).sym(ast).as_str() != "this"
}
