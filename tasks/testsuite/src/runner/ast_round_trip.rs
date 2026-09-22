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

pub struct AstRoundTripRunner<const PRESERVE_PARENS: bool>;

impl<C: Case, const PRESERVE_PARENS: bool> Runner<C> for AstRoundTripRunner<PRESERVE_PARENS> {
    const NAME: &'static str = if PRESERVE_PARENS {
        "ast_round_trip (preserve_parens=true)"
    } else {
        "ast_round_trip (preserve_parens=false)"
    };

    fn should_run(case: &C) -> bool {
        !case.expects_error()
    }

    fn run(case: &C) -> TestResult {
        let result = catch_unwind(AssertUnwindSafe(|| run_case(case, PRESERVE_PARENS)));
        match result {
            Ok(result) => result,
            Err(_) => TestResult::Panic {
                path: case.relative_path().to_path_buf(),
                negative: false,
            },
        }
    }
}

fn run_case<C: Case>(case: &C, preserve_parens: bool) -> TestResult {
    let path = case.relative_path().to_path_buf();
    let allocator = Allocator::new();
    let options = ParserOptions {
        source_type: case.source_type(),
        lang: case.lang(),
        preserve_parens,
        ..ParserOptions::default()
    };
    let ast = match parse_with_options(&allocator, case.code(), options) {
        Ok(ast) => ast,
        Err(reason) => {
            return TestResult::Skipped {
                path,
                reason,
                negative: false,
            };
        }
    };
    let output = print(
        &allocator,
        &ast,
        CodegenOptions {
            minify: false,
            ascii_only: false,
            ..CodegenOptions::default()
        },
    )
    .code;
    // Once the input parses, an invalid output is a round-trip failure too.
    let result = match parse_with_options(&allocator, output, options) {
        Ok(reparsed) => ast_compare::compare(&ast, &reparsed),
        Err(error) => Err(format!("output {error}")),
    };
    match result {
        Ok(()) => TestResult::Passed {
            path,
            negative: false,
        },
        Err(error) => TestResult::Failed {
            path,
            error,
            negative: false,
        },
    }
}
