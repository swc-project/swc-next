use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, SourceType};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

use crate::{cases::Case, suite::TestResult};

pub mod babel_transform;
pub mod codegen;
pub mod parser;
pub mod resolver;
pub mod swc_transform;

pub trait Runner<C: Case> {
    const NAME: &'static str;

    fn should_run(_case: &C) -> bool {
        true
    }

    fn run(case: &C) -> TestResult;
}

fn parse_case<'a, C: Case>(allocator: &'a Allocator, case: &'a C) -> Result<Ast<'a>, String> {
    let source = case.code();
    let source_type = case.source_type();
    let lang = case.lang();
    parse(allocator, source, source_type, lang)
}

fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
    lang: Lang,
) -> Result<Ast<'a>, String> {
    parse_with_options(
        allocator,
        source,
        Options {
            source_type,
            lang,
            ..Options::default()
        },
    )
}

fn parse_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: Options,
) -> Result<Ast<'a>, String> {
    let result = Parser::init(allocator, source, options, NoTokenParserConfig).parse();
    if let Some(diagnostic) = result.diagnostics.first() {
        return Err(format!(
            "parser: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }

    Ok(result.ast)
}
