use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};

use super::TypeScriptOptions;
use crate::{MultiPassTransformer, normalize::Normalizer};

mod modules;
mod namespaces;
mod parameters;
mod syntax;

pub(super) fn transformed_code(source: &str) -> String {
    let allocator = Allocator::new();
    let options = ParserOptions {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..ParserOptions::default()
    };
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ctx = parsed.ast;
    Normalizer::normalize_runtime(&mut ctx);
    let analyzed = analyze(&mut ctx, AnalyzeOptions::default()).semantic;
    let mut transformer = MultiPassTransformer::new(analyzed);
    transformer.typescript(&TypeScriptOptions::default());
    swc_next_ecma_codegen::generate(transformer.ctx, Default::default()).code
}

/// Analyze printed JavaScript, so assertions cover the bindings consumers see.
pub(super) fn with_output_semantic(
    source: &str,
    check: impl FnOnce(&swc_next_ecma_ast::Ast<'_>, &swc_next_ecma_semantic::Semantic<'_>),
) {
    let allocator = Allocator::new();
    let options = ParserOptions::default();
    let parser = Parser::init(&allocator, source, options, NoTokenParserConfig);
    let mut parsed = parser.parse();
    assert!(parsed.diagnostics.is_empty());
    let analyzed = analyze(
        &mut parsed.ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
        },
    );
    assert!(analyzed.diagnostics.is_empty());
    check(
        unsafe { analyzed.semantic.ctx().ast() },
        analyzed.semantic.semantic(),
    );
}
