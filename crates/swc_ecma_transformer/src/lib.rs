use swc_next_ecma_ast::{
    Ast, ObjectProperty, RegExpLiteral, TraverseMut, TraverseMutCtx, TraverseMutWith,
};
use swc_next_ecma_semantic::{AnalyzeOptions, Semantic, analyze};

use crate::{
    assumptions::Assumptions,
    normalize::Normalizer,
    state::{TransformerCtx, TransformerState},
};

pub mod assumptions;
mod comments;
pub mod es2015;
mod jsx_pragma;
pub mod normalize;
mod state;
pub mod typescript;
mod uid;

pub struct Transformer {}

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2015: es2015::ES2015Options,
    pub typescript: Option<typescript::TypeScriptOptions>,
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
            typescript,
        } = options;

        let typescript_enabled = typescript.is_some();
        if let Some(options) = typescript {
            Normalizer::normalize_runtime(program, ast);
            typescript::transform(program, ast, semantic, options);
        } else {
            Normalizer::normalize(program, ast);
        }

        let (semantic_program_transforms, traverse) = es2015::ES2015::new(es2015).into_phases();
        if let Some(mut transforms) = semantic_program_transforms {
            let refreshed_semantic = typescript_enabled.then(|| {
                let options = AnalyzeOptions {
                    check_syntax: false,
                    build_module_record: false,
                };
                analyze(ast, options).semantic
            });
            let semantic = refreshed_semantic.as_ref().unwrap_or(semantic);
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
