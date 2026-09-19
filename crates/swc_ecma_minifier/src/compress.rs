use swc_next_ecma_ast::{Ast, VisitMut, VisitMutWith};

use crate::CompressOptions;

pub(super) fn compress(ast: &mut Ast<'_>, _options: CompressOptions) {
    ast.root_program().visit_mut_with(&mut Compressor { ast });
}

struct Compressor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
}

impl<'ast> VisitMut<'ast> for Compressor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
}
