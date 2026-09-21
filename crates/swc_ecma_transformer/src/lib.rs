use swc_next_ecma_ast::{
    Ast, Diagnostic, ObjectProperty, RegExpLiteral, TraverseMut, TraverseMutCtx, TraverseMutWith,
};
use swc_next_ecma_semantic::{AnalyzeOptions, Semantic, analyze};

use crate::{
    assumptions::Assumptions,
    normalize::Normalizer,
    state::{TransformerCtx, TransformerState},
    syntax::SyntaxResult,
};

pub mod assumptions;
mod comments;
pub mod es2015;
pub mod jsx;
mod jsx_pragma;
pub mod normalize;
mod state;
mod syntax;
pub mod typescript;
mod uid;

pub struct Transformer {}

pub struct TransformerReturn {
    pub diagnostics: Vec<Diagnostic<'static>>,
}

#[derive(Default)]
pub struct TransformerOptions {
    pub source_path: Option<String>,
    pub react_jsx_self: bool,
    pub react_jsx_source: bool,
    pub assumptions: Assumptions,
    pub es2015: es2015::ES2015Options,
    pub typescript: Option<typescript::TypeScriptOptions>,
}

pub struct TransformerImpl {
    es2015: es2015::TraverseTransforms,
}

impl Transformer {
    pub fn transform<'a>(
        ast: &mut Ast<'a>,
        semantic: &Semantic<'a>,
        options: TransformerOptions,
    ) -> TransformerReturn {
        let program = ast.root_program();
        let TransformerOptions {
            source_path,
            react_jsx_self,
            react_jsx_source,
            assumptions,
            es2015,
            typescript,
        } = options;

        let typescript_enabled = typescript.is_some();
        if typescript_enabled {
            Normalizer::normalize_runtime(program, ast);
        } else {
            Normalizer::normalize(program, ast);
        }
        let mut syntax_result = SyntaxResult::default();
        if react_jsx_self || react_jsx_source {
            syntax_result.generated_prelude = jsx::transform_development_attributes(
                program,
                ast,
                semantic,
                react_jsx_self,
                react_jsx_source,
                source_path.as_deref(),
            );
        }
        if let Some(options) = typescript {
            typescript::transform(program, ast, semantic, options);
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
            transforms.transform(ast, semantic, syntax_result.generated_prelude);
        }

        if traverse.is_enabled() {
            let mut transformer = TransformerImpl { es2015: traverse };
            let state = TransformerState { assumptions };
            program.traverse_mut_with(&mut transformer, &mut TraverseMutCtx::new(ast, state));
        }
        TransformerReturn {
            diagnostics: syntax_result.diagnostics,
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
