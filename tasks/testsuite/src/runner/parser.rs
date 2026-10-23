use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze, check_syntax};

use crate::{
    cases::Case,
    runner::{Runner, parse_case},
    suite::TestResult,
};

pub struct ParserRunner;

impl<C: Case> Runner<C> for ParserRunner {
    const NAME: &'static str = "parser";

    fn run(case: &C) -> TestResult {
        let path = case.relative_path().to_path_buf();
        let result = catch_unwind(AssertUnwindSafe(|| {
            let allocator = Allocator::new();
            let ast = match parse_case(&allocator, case) {
                Ok(result) => result,
                Err(_) if case.expects_error() => return Ok(()),
                Err(error) => return Err(format!("{}: {error}", case.name())),
            };
            if !case.expects_error()
                && let Some(expected) = case.expected_source_type()
            {
                let actual = ast.source_type();
                if actual != expected {
                    return Err(format!(
                        "{}: expected source type {expected:?}, but parser produced {actual:?}",
                        case.name()
                    ));
                }
            }
            let semantic_ret = analyze(
                &ast,
                AnalyzeOptions {
                    check_syntax: SyntaxChecks::all(),
                },
            );
            let diagnostics = check_syntax(&ast, SyntaxChecks::all());
            if diagnostics != semantic_ret.diagnostics {
                return Err(format!(
                    "{}: syntax checking and semantic analysis diagnostics differ",
                    case.name()
                ));
            }
            match (case.expects_error(), diagnostics.first()) {
                (true, Some(_)) | (false, None) => Ok(()),
                (false, Some(_)) if !case.asserts_semantic_success() => Ok(()),
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
