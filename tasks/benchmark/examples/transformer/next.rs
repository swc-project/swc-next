use std::hint::black_box;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{AstCtx, Lang};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SemanticAst, analyze};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions,
    jsx::{JsxOptions, JsxRuntime, ReactRefreshOptions},
    normalize::Normalizer,
    typescript::TypeScriptOptions,
};

pub fn run(source: &str) {
    let allocator = Allocator::new();
    let mut ast = run_parse(source, &allocator);
    Normalizer::normalize_runtime(&mut ast);
    let semantic = run_semantic(&mut ast);
    run_transform(semantic);
    black_box(ast);
}

#[inline(never)]
fn run_parse<'a>(source: &'a str, allocator: &'a Allocator) -> AstCtx<'a> {
    let result = Parser::init(
        allocator,
        source,
        Options {
            lang: Lang::Tsx,
            preserve_parens: false,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(result.diagnostics.is_empty());
    result.ast
}

#[inline(never)]
fn run_semantic<'ctx, 'a>(ast: &'ctx mut AstCtx<'a>) -> SemanticAst<'a, &'ctx mut AstCtx<'a>> {
    let result = analyze(ast, AnalyzeOptions::default());
    assert!(result.diagnostics.is_empty());
    result.semantic
}

#[inline(never)]
fn run_transform<'a>(semantic: SemanticAst<'a, &mut AstCtx<'a>>) {
    let options = TransformerOptions {
        filename: Some("App.tsx".to_owned()),
        typescript: Some(TypeScriptOptions {
            verbatim_module_syntax: false,
            preserve_const_enums: false,
            ..TypeScriptOptions::default()
        }),
        jsx: Some(JsxOptions {
            runtime: JsxRuntime::Automatic,
            development: true,
            source_self: true,
            display_name: true,
            refresh: Some(ReactRefreshOptions::default()),
            pure: Some(true),
            ..JsxOptions::default()
        }),
        ..TransformerOptions::default()
    };
    let result = Transformer::transform(semantic, options);
    assert!(result.diagnostics.is_empty());
    black_box(result);
}
