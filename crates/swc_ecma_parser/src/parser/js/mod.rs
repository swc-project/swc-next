mod class_and_fn;
mod expr;
mod ident;
mod module_item;
mod object;
mod pat;
mod stmt;

use swc_experimental_ecma_ast::*;

use crate::{Parser, input::Tokens};

impl<I: Tokens> Parser<I> {
    fn is_not_this(&self, p: Param) -> bool {
        let Pat::Ident(ident) = p.pat(&self.ast) else {
            return true;
        };
        self.input
            .iter
            .read_utf8(ident.id(&self.ast).sym(&self.ast))
            != "this"
    }
}
