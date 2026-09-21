use swc_next_ecma_ast::{
    Ast, CallExpression, Diagnostic, NodeId, ObjectProperty, RegExpLiteral, TraverseMut,
    TraverseMutCtx, TraverseMutWith,
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
    pub react_display_name: bool,
    pub react_jsx_self: bool,
    pub react_jsx_source: bool,
    pub assumptions: Assumptions,
    pub es2015: es2015::ES2015Options,
    pub jsx: Option<jsx::JsxOptions>,
    pub typescript: Option<typescript::TypeScriptOptions>,
}

pub struct TransformerImpl {
    es2015: es2015::TraverseTransforms,
    display_name: Option<jsx::DisplayName>,
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
            react_display_name,
            react_jsx_self,
            react_jsx_source,
            assumptions,
            es2015,
            jsx,
            typescript,
        } = options;

        let typescript_enabled = typescript.is_some();
        if typescript_enabled {
            Normalizer::normalize_runtime(program, ast);
        } else {
            Normalizer::normalize(program, ast);
        }
        let refresh = jsx.as_ref().and_then(|options| options.refresh.as_ref());
        let refresh_semantic = refresh.map(|refresh| {
            jsx::transform_refresh(program, ast, semantic, refresh);
            analyze(
                ast,
                AnalyzeOptions {
                    check_syntax: false,
                    build_module_record: false,
                },
            )
            .semantic
        });
        let semantic = refresh_semantic.as_ref().unwrap_or(semantic);
        let jsx = jsx.map(|mut options| {
            options.jsx_self |= react_jsx_self;
            options.jsx_source |= react_jsx_source;
            jsx::resolve_options(program, ast, options)
        });
        let jsx_enabled = jsx.is_some();
        let mut syntax_result = SyntaxResult::default();
        if !jsx_enabled && (react_jsx_self || react_jsx_source) {
            syntax_result.generated_prelude = jsx::transform_development_attributes(
                program,
                ast,
                semantic,
                react_jsx_self,
                react_jsx_source,
                source_path.as_deref(),
            );
        }
        syntax_result = match (typescript, jsx) {
            (Some(typescript), Some(jsx)) => {
                let mut jsx = jsx::JsxLowerer::new(&jsx, ast, semantic, source_path.as_deref());
                typescript::transform_with_following(program, ast, semantic, typescript, &mut jsx);
                jsx.take_result()
            }
            (Some(typescript), None) => {
                typescript::transform(program, ast, semantic, typescript);
                syntax_result
            }
            (None, Some(jsx)) => {
                jsx::transform_resolved(program, ast, semantic, jsx, source_path.as_deref())
            }
            (None, None) => syntax_result,
        };

        let (semantic_program_transforms, traverse) = es2015::ES2015::new(es2015).into_phases();
        if let Some(mut transforms) = semantic_program_transforms {
            let syntax_transformed = typescript_enabled || jsx_enabled;
            let refreshed_semantic = syntax_transformed.then(|| {
                let options = AnalyzeOptions {
                    check_syntax: false,
                    build_module_record: false,
                };
                analyze(ast, options).semantic
            });
            let semantic = refreshed_semantic.as_ref().unwrap_or(semantic);
            transforms.transform(ast, semantic, syntax_result.generated_prelude);
        }

        if react_display_name || traverse.is_enabled() {
            let mut transformer = TransformerImpl {
                es2015: traverse,
                display_name: react_display_name.then(|| jsx::DisplayName::new(source_path)),
            };
            let state = TransformerState { assumptions };
            program.traverse_mut_with(&mut transformer, &mut TraverseMutCtx::new(ast, state));
        }

        TransformerReturn {
            diagnostics: syntax_result.diagnostics,
        }
    }
}

impl<'a> TraverseMut<'a, TransformerState> for TransformerImpl {
    fn enter_node(&mut self, node: NodeId, _ctx: &mut TransformerCtx<'_, 'a>) {
        if let Some(display_name) = &mut self.display_name {
            display_name.enter_node(node);
        }
    }

    fn exit_node(&mut self, node: NodeId, _ctx: &mut TransformerCtx<'_, 'a>) {
        if let Some(display_name) = &mut self.display_name {
            display_name.exit_node(node);
        }
    }

    fn exit_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState>,
    ) {
        if let Some(display_name) = &self.display_name {
            display_name.exit_call_expression(node, ctx);
        }
    }

    fn exit_object_property(&mut self, node: ObjectProperty, ctx: &mut TransformerCtx<'_, 'a>) {
        self.es2015.exit_object_property(node, ctx);
    }

    fn exit_reg_exp_literal(&mut self, node: RegExpLiteral, ctx: &mut TransformerCtx<'_, 'a>) {
        self.es2015.exit_reg_exp_literal(node, ctx);
    }
}
