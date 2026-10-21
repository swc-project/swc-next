use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::Options as ParserOptions;

use crate::{
    ast_compare,
    cases::Case,
    runner::{Runner, parse_with_options},
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
    let mut errors = Vec::new();

    for preserve_parens in [false, true] {
        let options = ParserOptions {
            source_type: case.source_type(),
            lang: case.lang(),
            preserve_parens,
            ..ParserOptions::default()
        };
        let ast = parse_with_options(&allocator, case.code(), options)
            .map_err(|error| format!("preserve_parens={preserve_parens}: input {error}"))?;
        let output = print(&ast, CodegenOptions::default()).code;
        let reparsed = parse_with_options(&allocator, &output, options)
            .map_err(|error| format!("preserve_parens={preserve_parens}: output {error}"))?;
        if let Err(error) = ast_compare::compare(&ast, &reparsed) {
            errors.push(format!("preserve_parens={preserve_parens}: {error}"));
        }
        let output2 = print(&reparsed, CodegenOptions::default()).code;
        if output != output2 {
            errors.push(format!(
                "preserve_parens={preserve_parens}: codegen output changed after reparse"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
