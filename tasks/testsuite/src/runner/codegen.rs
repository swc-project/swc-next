use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_ecma_codegen::{Options as CodegenOptions, print};

use crate::{
    cases::Case,
    runner::{Runner, parse, parse_case},
    suite::TestResult,
};

pub struct CodegenRunner;

impl<C: Case> Runner<C> for CodegenRunner {
    const NAME: &'static str = "codegen";

    fn should_run(case: &C) -> bool {
        !case.expects_error()
    }

    fn run(case: &C) -> TestResult {
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
    }
}

fn run_case<C: Case>(case: &C) -> Result<(), String> {
    let allocator = Allocator::new();

    let ast = parse_case(&allocator, case)?;
    let codegen1 = print(&allocator, &ast, CodegenOptions::default()).code;

    let ast = parse(&allocator, codegen1, case.source_type(), case.lang())?;
    let codegen2 = print(&allocator, &ast, CodegenOptions::default()).code;

    if codegen1 == codegen2 {
        return Ok(());
    }

    Err(format!(
        "{}: codegen output changed after reparse ",
        case.name(),
    ))
}
