use swc_next_ecma_ast::Ast;

use crate::Semantic;

pub struct Checker<'a> {
    pub tree: &'a mut Ast<'a>,
}

impl<'a> Checker<'a> {
    pub fn new(tree: &'a mut Ast<'a>) -> Self {
        Self { tree }
    }

    pub fn check_unresolved_exports(&mut self, sem: &Semantic<'_>) {
        todo!()
    }
}
