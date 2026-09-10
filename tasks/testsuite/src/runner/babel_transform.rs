use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Severity};
use swc_next_ecma_codegen::print;
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_ecma_transformer::Transformer;

use crate::{
    cases::{babel::BabelTransformCase, transform::TransformCase},
    suite::TestResult,
    util::crate_root,
};

use super::Runner;

pub struct BabelTransformRunner;

impl Runner<BabelTransformCase> for BabelTransformRunner {
    const NAME: &'static str = "babel_transform";

    fn should_run(case: &BabelTransformCase) -> bool {
        case.options.plugins.errors.is_empty()
            && case.options.plugins.unsupported.is_empty()
            && case.options.presets.errors.is_empty()
            && case.options.presets.unsupported.is_empty()
    }

    fn run(case: &BabelTransformCase) -> TestResult {
        run_transform(case, Self::NAME)
    }
}

pub fn output_root(runner_name: &str) -> PathBuf {
    crate_root().join("outputs").join(runner_name)
}

pub(super) fn run_transform<C: TransformCase>(case: &C, runner_name: &str) -> TestResult {
    let path = case.relative_path().to_path_buf();
    let result = catch_unwind(AssertUnwindSafe(|| run_case(case, runner_name)));
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

fn run_case<C: TransformCase>(case: &C, runner_name: &str) -> Result<(), String> {
    let actual = transform_input(case)?;
    let expected = normalize_expected_output(case)?;
    write_comparison_outputs(case, runner_name, &actual, &expected)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{}: transformed output differs from output.js",
            case.name()
        ))
    }
}

fn write_comparison_outputs<C: TransformCase>(
    case: &C,
    runner_name: &str,
    actual: &str,
    expected: &str,
) -> Result<(), String> {
    let relative_path = case
        .output_path()
        .strip_prefix(crate_root())
        .map_err(|error| format!("output path: {error}"))?;
    let output_root = output_root(runner_name);

    for (kind, output) in [("actual", actual), ("expected", expected)] {
        let output_path = output_root.join(kind).join(relative_path);
        let parent = output_path
            .parent()
            .ok_or_else(|| format!("output path has no parent: {}", output_path.display()))?;
        fs::create_dir_all(parent)
            .and_then(|()| fs::write(&output_path, output))
            .map_err(|error| format!("write {}: {error}", output_path.display()))?;
    }

    Ok(())
}

fn transform_input<C: TransformCase>(case: &C) -> Result<String, String> {
    let source = case.code();
    let allocator = Allocator::new();
    let mut ast = parse_program(source, &allocator, case)?;
    let options = case.transformer_options();
    let mut semantic_ret = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: true,
            features: options.semantic_features(),
            ..AnalyzeOptions::default()
        },
    );
    if let Some(diagnostic) = semantic_ret
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "semantic: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }

    Transformer::transform(&mut ast, &mut semantic_ret.semantic, options);
    Ok(print(&allocator, &ast, case.codegen_options())
        .code
        .to_owned())
}

fn normalize_expected_output<C: TransformCase>(case: &C) -> Result<String, String> {
    let source = case.output();
    let allocator = Allocator::new();
    let ast = parse_program(source, &allocator, case)?;
    Ok(print(&allocator, &ast, case.codegen_options())
        .code
        .to_owned())
}

fn parse_program<'a, C: TransformCase>(
    source: &'a str,
    allocator: &'a Allocator,
    case: &C,
) -> Result<Ast<'a>, String> {
    let options = ParserOptions {
        source_type: case.source_type(),
        lang: case.lang(),
        ..ParserOptions::default()
    };
    let (ast, error) = {
        let result = Parser::init(allocator, source, options, NoTokenParserConfig).parse();
        let error = result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| {
                format!(
                    "parse: {} at {}..{}",
                    diagnostic.message, diagnostic.span.start, diagnostic.span.end
                )
            });
        (result.ast, error)
    };

    if let Some(error) = error {
        return Err(error);
    }
    Ok(ast)
}
