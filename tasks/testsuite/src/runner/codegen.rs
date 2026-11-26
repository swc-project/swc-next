use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
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
    let normalized_options = ParserOptions {
        source_type: case.source_type(),
        lang: case.lang(),
        preserve_parens: false,
        ..ParserOptions::default()
    };
    let original = parse_with_options(&allocator, case.code(), normalized_options)
        .map_err(|error| format!("preserve_parens=false: input {error}"))?;

    for preserve_parens in [false, true] {
        let options = ParserOptions {
            preserve_parens,
            ..normalized_options
        };
        let preserved = preserve_parens
            .then(|| parse_with_options(&allocator, case.code(), options))
            .transpose()
            .map_err(|error| format!("preserve_parens={preserve_parens}: input {error}"))?;
        let ast = preserved.as_ref().unwrap_or(&original);
        let output = generate(ast, GeneratorOptions::default()).code;
        let reparsed = parse_with_options(&allocator, &output, options)
            .map_err(|error| format!("preserve_parens={preserve_parens}: output {error}"))?;
        // Grammar groups may be added to preserved input. Compare every AST
        // field after removing parentheses, and check printing stability below.
        let normalized = preserve_parens
            .then(|| parse_with_options(&allocator, &output, normalized_options))
            .transpose()
            .map_err(|error| format!("preserve_parens={preserve_parens}: output {error}"))?;
        if let Err(error) =
            ast_compare::compare(&original, normalized.as_ref().unwrap_or(&reparsed))
        {
            errors.push(format!("preserve_parens={preserve_parens}: {error}"));
        }
        let output2 = generate(&reparsed, GeneratorOptions::default()).code;
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
