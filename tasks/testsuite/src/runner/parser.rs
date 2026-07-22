use std::panic::{AssertUnwindSafe, catch_unwind};

use colored::Colorize;
use rayon::prelude::*;
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, SourceType};
use swc_experimental_ecma_parser::{Options, Parser};
use swc_experimental_ecma_semantic::analyze;

use crate::{cases::Case, runner::Runner, suite::TestResult};

pub struct ParserRunner;

impl Runner for ParserRunner {
    const NAME: &'static str = "parser";

    fn run<C: Case>(cases: &[C], debug: bool) -> Vec<TestResult> {
        cases
            .par_iter()
            .map(|case| {
                if debug {
                    println!("[{}] {:?}", "Debug".green(), case.relative_path());
                }

                let path = case.relative_path().to_path_buf();
                let result = catch_unwind(AssertUnwindSafe(|| run_case(case)));
                match result {
                    Ok(Ok(())) => TestResult::Passed { path },
                    Ok(Err(error)) => TestResult::Failed { path, error },
                    Err(_) => TestResult::Panic { path },
                }
            })
            .collect()
    }
}

fn run_case<C: Case>(case: &C) -> Result<(), String> {
    let error = run_source(case.code().as_bytes(), case.source_type(), case.lang());
    match (case.expects_error(), error) {
        (true, Some(_)) | (false, None) => Ok(()),
        (true, None) => Err(format!(
            "{}: expected a parse or early error, but pipeline succeeded",
            case.name()
        )),
        (false, Some(error)) => Err(format!("{}: {error}", case.name())),
    }
}

fn run_source(
    source: &[u8],
    source_type: SourceType,
    lang: swc_experimental_ecma_ast::Lang,
) -> Option<String> {
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let options = Options {
        source_type,
        lang,
        ..Options::default()
    };

    let (parse_result, parser_diagnostic) = {
        let mut parser = Parser::init(&mut ast, source, options);
        let result = parser.parse();
        let diagnostic = parser.diagnostics.first().map(|diagnostic| {
            format!(
                "parser: {} at {}..{}",
                diagnostic.message, diagnostic.span.start, diagnostic.span.end
            )
        });
        (result, diagnostic)
    };

    match parse_result {
        None => parser_diagnostic
            .or_else(|| Some("parser: parsing stopped without producing a diagnostic".to_string())),
        Some(_) if parser_diagnostic.is_some() => parser_diagnostic,
        Some(program) => {
            let semantic = analyze(program, &mut ast);
            semantic.diagnostics().first().map(|diagnostic| {
                format!(
                    "semantic: {} at {}..{}",
                    diagnostic.message, diagnostic.span.start, diagnostic.span.end
                )
            })
        }
    }
}
