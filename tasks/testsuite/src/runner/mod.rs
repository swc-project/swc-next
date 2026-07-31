use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, Program, SourceType};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

use crate::{cases::Case, suite::TestResult};

pub mod codegen;
pub mod parser;

pub trait Runner {
    const NAME: &'static str;

    fn should_run<C: Case>(_case: &C) -> bool {
        true
    }

    fn run<C: Case>(case: &C, debug: bool) -> TestResult;
}

fn parse_case<'a, C: Case>(
    allocator: &'a Allocator,
    case: &'a C,
) -> Result<(Program, Ast<'a>), String> {
    let source = case.code();
    let source_type = case.source_type();
    let lang = case.lang();
    parse(allocator, source.as_bytes(), source_type, lang)
}

fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a [u8],
    source_type: SourceType,
    lang: Lang,
) -> Result<(Program, Ast<'a>), String> {
    let mut ast = Ast::new(source, &allocator);
    let result = Parser::init(
        &mut ast,
        Options {
            source_type,
            lang,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    if let Some(diagnostic) = result.diagnostics.first() {
        return Err(format!(
            "parser: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }

    Ok((result.program, ast))
}
