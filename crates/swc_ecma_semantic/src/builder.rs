use swc_next_ecma_ast::Program;

use crate::{Semantic, checker::Checker};

pub struct SemanticBuilder<'a> {
    checker: Checker<'a>,
}

impl<'a> SemanticBuilder<'a> {
    pub fn build(root: Program, checker: &mut Checker<'a>) -> Semantic<'a> {
        todo!()
    }
}
