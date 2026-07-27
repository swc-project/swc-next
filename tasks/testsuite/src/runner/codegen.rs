use std::panic::{AssertUnwindSafe, catch_unwind};

use colored::Colorize;
use rayon::prelude::*;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, Severity, SourceType};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{Options as ParserOptions, Parser};

use crate::{cases::Case, runner::Runner, suite::TestResult};

pub struct CodegenRunner;

impl Runner for CodegenRunner {
    const NAME: &'static str = "codegen";

    fn run<C: Case>(cases: &[C], debug: bool) -> Vec<TestResult> {
        cases
            .par_iter()
            .filter(|case| !case.expects_error())
            .map(|case| {
                if debug {
                    println!("[{}] {:?}", "Debug".green(), case.relative_path());
                }

                let path = case.relative_path().to_path_buf();
                let result = catch_unwind(AssertUnwindSafe(|| run_case(case)));
                match result {
                    Ok(Ok(())) => TestResult::Passed {
                        path,
                        negative: false,
                    },
                    Ok(Err(error)) => TestResult::Failed {
                        path,
                        error,
                        negative: false,
                    },
                    Err(_) => TestResult::Panic {
                        path,
                        negative: false,
                    },
                }
            })
            .collect()
    }
}

fn run_case<C: Case>(case: &C) -> Result<(), String> {
    let first = parse_and_codegen(
        case.code().as_bytes(),
        case.source_type(),
        case.lang(),
        "first parse",
    )
    .map_err(|error| format!("{}: {error}", case.name()))?;
    let second = parse_and_codegen(&first, case.source_type(), case.lang(), "second parse")
        .map_err(|error| format!("{}: {error}", case.name()))?;

    if first == second {
        return Ok(());
    }

    let mismatch = first
        .iter()
        .zip(&second)
        .position(|(first, second)| first != second)
        .unwrap_or_else(|| first.len().min(second.len()));
    Err(format!(
        "{}: codegen output changed after reparse at byte {mismatch} (first: {} bytes, second: {} bytes)",
        case.name(),
        first.len(),
        second.len(),
    ))
}

fn parse_and_codegen(
    source: &[u8],
    source_type: SourceType,
    lang: Lang,
    stage: &str,
) -> Result<Vec<u8>, String> {
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let options = ParserOptions {
        source_type,
        lang,
        ..ParserOptions::default()
    };

    let (program, error) = {
        let mut parser = Parser::init(&mut ast, source, options);
        let program = parser.parse();
        let error = parser
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| {
                format!(
                    "{stage}: {} at {}..{}",
                    diagnostic.message, diagnostic.span.start, diagnostic.span.end
                )
            });
        (program, error)
    };

    if let Some(error) = error {
        return Err(error);
    }
    let Some(program) = program else {
        return Err(format!(
            "{stage}: parsing stopped without producing an error diagnostic"
        ));
    };

    Ok(print(&allocator, &ast, program, CodegenOptions::default())
        .code
        .to_vec())
}
