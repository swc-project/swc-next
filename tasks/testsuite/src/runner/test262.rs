use std::panic::{AssertUnwindSafe, catch_unwind};

use colored::Colorize;
use rayon::prelude::*;
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, SourceType};
use swc_experimental_ecma_parser::{Options, Parser};
use swc_experimental_ecma_semantic::analyze;

use crate::{
    AppArgs,
    cases::{Case, test262::Test262Case},
    suite::TestResult,
};

pub struct Test262Runner;

impl Test262Runner {
    pub fn run(args: &AppArgs, cases: &[Test262Case]) -> Vec<TestResult> {
        cases
            .par_iter()
            .map(|case| {
                if args.debug {
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

fn run_case(case: &Test262Case) -> Result<(), String> {
    if case.is_module() {
        return run_source(
            case.code().as_bytes(),
            SourceType::Module,
            case.expects_error(),
            "module",
        );
    }
    if case.runs_sloppy() {
        run_source(
            case.code().as_bytes(),
            SourceType::Script,
            case.expects_error(),
            "sloppy",
        )?;
    }
    if case.runs_strict() {
        let source = format!("\"use strict\";\n{}", case.code());
        run_source(
            source.as_bytes(),
            SourceType::Script,
            case.expects_error(),
            "strict",
        )?;
    }
    Ok(())
}

fn run_source(
    source: &[u8],
    source_type: SourceType,
    expects_error: bool,
    variant: &str,
) -> Result<(), String> {
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let options = Options {
        source_type,
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

    let actual_error = match parse_result {
        Err(error) => Some(format!("parser: {error:?}")),
        Ok(_) if parser_diagnostic.is_some() => parser_diagnostic,
        Ok(program) => {
            let semantic = analyze(program, &mut ast);
            semantic.diagnostics().first().map(|diagnostic| {
                format!(
                    "semantic: {} at {}..{}",
                    diagnostic.message, diagnostic.span.start, diagnostic.span.end
                )
            })
        }
    };

    match (expects_error, actual_error) {
        (true, Some(_)) | (false, None) => Ok(()),
        (true, None) => Err(format!(
            "expected parse or early error in {variant} mode, but pipeline succeeded"
        )),
        (false, Some(error)) => Err(format!("{variant}: {error}")),
    }
}
