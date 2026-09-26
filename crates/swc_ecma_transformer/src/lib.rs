use swc_next_ecma_ast::{
    Ast, ObjectProperty, RegExpLiteral, TraverseMut, TraverseMutCtx, TraverseMutWith,
};
use swc_next_ecma_semantic::Semantic;

use crate::{
    assumptions::Assumptions,
    normalize::Normalizer,
    state::{TransformerCtx, TransformerState},
};

pub mod assumptions;
mod comments;
pub mod es2015;
pub mod normalize;
mod state;
mod uid;

pub struct Transformer {}

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2015: es2015::ES2015Options,
}

pub struct TransformerImpl {
    es2015: es2015::TraverseTransforms,
}

impl Transformer {
    pub fn transform<'a>(ast: &mut Ast<'a>, semantic: &Semantic<'a>, options: TransformerOptions) {
        let program = ast.root_program();
        let TransformerOptions {
            assumptions,
            es2015,
        } = options;

        Normalizer::normalize(program, ast);

        let (semantic_program_transforms, traverse) = es2015::ES2015::new(es2015).into_phases();
        if let Some(mut transforms) = semantic_program_transforms {
            transforms.transform(ast, semantic);
        }

        if traverse.is_enabled() {
            let mut transformer = TransformerImpl { es2015: traverse };
            let state = TransformerState { assumptions };
            program.traverse_mut_with(&mut transformer, &mut TraverseMutCtx::new(ast, state));
        }
    }
}

impl<'a> TraverseMut<'a, TransformerState> for TransformerImpl {
    fn exit_object_property(&mut self, node: ObjectProperty, ctx: &mut TransformerCtx<'_, 'a>) {
        self.es2015.exit_object_property(node, ctx);
    }

    fn exit_reg_exp_literal(&mut self, node: RegExpLiteral, ctx: &mut TransformerCtx<'_, 'a>) {
        self.es2015.exit_reg_exp_literal(node, ctx);
    }
}
