use std::panic::{AssertUnwindSafe, catch_unwind};

use colored::Colorize;
use swc_next_allocator::Allocator;
use swc_next_ecma_semantic::analyze;

use crate::{
    cases::Case,
    runner::{Runner, parse_case},
    suite::TestResult,
};

pub struct ParserRunner;

impl Runner for ParserRunner {
    const NAME: &'static str = "parser";

    fn run<C: Case>(case: &C, debug: bool) -> TestResult {
        if debug {
            println!("[{}] {:?}", "Debug".green(), case.relative_path());
        }

        let path = case.relative_path().to_path_buf();
        let result = catch_unwind(AssertUnwindSafe(|| {
            let allocator = Allocator::new();
            let (program, mut ast) = match parse_case(&allocator, case) {
                Ok(result) => result,
                Err(_) if case.expects_error() => return Ok(()),
                Err(error) => return Err(format!("{}: {error}", case.name())),
            };
            let semantic = analyze(program, &mut ast);
            match (case.expects_error(), semantic.diagnostics().first()) {
                (true, Some(_)) | (false, None) => Ok(()),
                (true, None) => Err(format!(
                    "{}: expected a parse or early error, but pipeline succeeded",
                    case.name()
                )),
                (false, Some(error)) => Err(format!(
                    "{}: semantic: {} at {}..{}",
                    case.name(),
                    error.message,
                    error.span.start,
                    error.span.end
                )),
            }
        }));

        let negative = case.expects_error();
        match result {
            Ok(Ok(())) => TestResult::Passed { path, negative },
            Ok(Err(error)) => TestResult::Failed {
                path,
                error,
                negative,
            },
            Err(_) => TestResult::Panic { path, negative },
        }
    }
}
