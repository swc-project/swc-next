use swc_experimental_ecma_ast::{
    Ast, ObjectProperty, Program, RegExpLiteral, Traverse, TraverseAction, TraverseCtx,
    TraverseWith,
};
use swc_experimental_ecma_semantic::Semantic;

mod es2015;

pub struct Transformer {}

#[derive(Default)]
pub struct TransformerOptions {
    pub es2015: es2015::ES2015Options,
}

pub struct TransformerImpl<'a> {
    semantic: &'a Semantic,
    x0_es2015: es2015::ES2015,
}

impl Transformer {
    pub fn transform<'a>(
        program: Program,
        ast: &mut Ast<'a>,
        semantic: &Semantic,
        options: TransformerOptions,
    ) {
        let mut transformer = TransformerImpl {
            semantic,
            x0_es2015: es2015::ES2015::new(options.es2015),
        };
        let _ = program.traverse_with(&mut transformer, &mut TraverseCtx::new(ast));
    }
}

impl<'a> Traverse<'a> for TransformerImpl<'_> {
    fn enter_program(&mut self, node: Program, ctx: &mut TraverseCtx<'_, 'a>) -> TraverseAction {
        self.x0_es2015.transform(node, self.semantic, ctx);
        TraverseAction::Proceed
    }

    fn exit_object_property(&mut self, node: ObjectProperty, ctx: &mut TraverseCtx<'_, 'a>) {
        self.x0_es2015.exit_object_property(node, ctx);
    }

    fn exit_reg_exp_literal(&mut self, node: RegExpLiteral, ctx: &mut TraverseCtx<'_, 'a>) {
        self.x0_es2015.exit_reg_exp_literal(node, ctx);
    }
}
