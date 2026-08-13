use swc_next_ecma_ast::{
    Ast, ObjectProperty, Program, RegExpLiteral, Traverse, TraverseAction, TraverseCtx,
    TraverseWith,
};
use swc_next_ecma_semantic::Semantic;

use crate::{
    assumptions::Assumptions,
    normalize::Normalizer,
    state::{TransformerCtx, TransformerState},
};

pub mod assumptions;
pub mod es2015;
pub mod normalize;
mod state;

pub struct Transformer {}

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2015: es2015::ES2015Options,
}

pub struct TransformerImpl<'a> {
    semantic: &'a Semantic<'a>,
    x0_es2015: es2015::ES2015,
}

impl Transformer {
    pub fn transform<'a>(ast: &mut Ast<'a>, semantic: &Semantic, options: TransformerOptions) {
        let program = ast.root_program();

        Normalizer::normalize(program, ast);
        let mut transformer = TransformerImpl {
            semantic,
            x0_es2015: es2015::ES2015::new(options.es2015),
        };

        let state = TransformerState {
            assumptions: options.assumptions,
        };
        let _ = program.traverse_with(&mut transformer, &mut TraverseCtx::new(ast, state));
    }
}

impl<'a> Traverse<'a, TransformerState> for TransformerImpl<'_> {
    fn enter_program(
        &mut self,
        node: Program,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) -> TraverseAction<Program> {
        self.x0_es2015.transform(node, self.semantic, ctx);
        TraverseAction::Proceed
    }

    fn exit_object_property(&mut self, node: ObjectProperty, ctx: &mut TransformerCtx<'_, 'a>) {
        self.x0_es2015.exit_object_property(node, ctx);
    }

    fn exit_reg_exp_literal(&mut self, node: RegExpLiteral, ctx: &mut TransformerCtx<'_, 'a>) {
        self.x0_es2015.exit_reg_exp_literal(node, ctx);
    }
}
