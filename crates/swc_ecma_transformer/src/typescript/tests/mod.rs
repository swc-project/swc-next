use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

use super::{TypeScriptOptions, transform};
use crate::normalize::Normalizer;

mod modules;
mod syntax;

pub(super) fn transformed_code(source: &str) -> String {
    transformed_code_with_options(source, TypeScriptOptions::default())
}

pub(super) fn transformed_code_with_options(
    source: &str,
    transform_options: TypeScriptOptions,
) -> String {
    transformed_code_with_pipeline(source, transform_options, false)
}

fn transformed_code_with_pipeline(
    source: &str,
    transform_options: TypeScriptOptions,
    arrows: bool,
) -> String {
    let allocator = Allocator::new();
    let options = ParserOptions {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..ParserOptions::default()
    };
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    crate::Transformer::transform(
        &mut ast,
        &semantic,
        crate::TransformerOptions {
            typescript: Some(transform_options),
            es2015: crate::es2015::ES2015Options {
                arrow_functions: arrows
                    .then_some(crate::es2015::arrow::ArrowFunctionsOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    swc_next_ecma_codegen::print(&allocator, &ast, Default::default())
        .code
        .to_owned()
}
