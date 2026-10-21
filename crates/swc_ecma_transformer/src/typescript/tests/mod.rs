use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};

use super::{TypeScriptOptions, transform};
use crate::normalize::Normalizer;

mod enums;
mod jsx;
mod modules;
mod namespaces;
mod parameters;
mod syntax;

pub(super) fn transformed_code(source: &str) -> String {
    transformed_code_with_options(source, TypeScriptOptions::default())
}

pub(super) fn transformed_code_with_options(
    source: &str,
    transform_options: TypeScriptOptions,
) -> String {
    transformed_code_with_language(source, transform_options, Lang::Ts)
}

fn transformed_code_with_language(
    source: &str,
    transform_options: TypeScriptOptions,
    lang: Lang,
) -> String {
    transformed_code_with_source_type(source, transform_options, lang, SourceType::Module)
}

fn transformed_code_with_source_type(
    source: &str,
    transform_options: TypeScriptOptions,
    lang: Lang,
    source_type: SourceType,
) -> String {
    let allocator = Allocator::new();
    let options = ParserOptions {
        lang,
        source_type,
        ..ParserOptions::default()
    };
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    transform(ast.root_program(), &mut ast, &semantic, transform_options);
    swc_next_ecma_codegen::print(&ast, Default::default()).code
}

/// Analyze printed JavaScript, so assertions cover the bindings consumers see.
pub(super) fn with_output_semantic(
    source: &str,
    check: impl FnOnce(&swc_next_ecma_ast::Ast<'_>, &swc_next_ecma_semantic::Semantic<'_>),
) {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions::default(),
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let analyzed = analyze(
        &parsed.ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
            build_module_record: false,
        },
    );
    assert!(analyzed.diagnostics.is_empty());
    check(&parsed.ast, &analyzed.semantic);
}
