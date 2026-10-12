use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, Severity};
use swc_next_ecma_codegen::print;
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_ecma_transformer::{Transformer, normalize::Normalizer};

use crate::{
    cases::TransformCase,
    suite::TestResult,
    util::{crate_root, workspace_root},
};

use super::Runner;

pub struct TransformerRunner;

impl<C: TransformCase> Runner<C> for TransformerRunner {
    const NAME: &'static str = "transform";

    fn run(case: &C) -> TestResult {
        if !case.supports_options() {
            return TestResult::Skipped {
                path: case.relative_path().to_path_buf(),
            };
        }
        run_transform(case, <Self as Runner<C>>::NAME)
    }
}

pub fn output_root(runner_name: &str) -> PathBuf {
    crate_root().join("outputs").join(runner_name)
}

fn run_transform<C: TransformCase>(case: &C, runner_name: &str) -> TestResult {
    let path = case
        .path()
        .strip_prefix(workspace_root().expect("workspace root should be discoverable"))
        .expect("transform fixture should be inside the workspace")
        .to_path_buf();
    let result = catch_unwind(AssertUnwindSafe(|| run_case(case, runner_name)));
    match result {
        Ok(Ok(true)) => TestResult::Passed {
            path,
            negative: false,
        },
        Ok(Ok(false)) => TestResult::Skipped { path },
        Ok(Err(error)) => {
            let error = match case.options_label() {
                Some(options) => format!("[{options}]: {error}"),
                None => error,
            };
            let error = match case.virtual_file_name() {
                Some(name) => format!("virtual: {name} - {error}"),
                None => error,
            };
            TestResult::Failed {
                path,
                error,
                negative: false,
            }
        }
        Err(_) => TestResult::Panic {
            path,
            negative: false,
        },
    }
}

fn run_case<C: TransformCase>(case: &C, runner_name: &str) -> Result<bool, String> {
    let actual = match transform_input(case) {
        Ok(actual) => actual,
        // TypeScript conformance includes inputs with syntax and semantic errors.
        // As in main, these are outside the transform suite's valid-input contract.
        Err(_) if case.expected_output().is_none() => return Ok(false),
        Err(error) => return Err(error),
    };
    if case.expected_output().is_none() {
        if case.lang() == Lang::Dts {
            return if actual.is_empty() {
                Ok(true)
            } else {
                Err("declaration input emitted JavaScript".into())
            };
        }
        let lang = if case.lang() == Lang::Tsx {
            Lang::Jsx
        } else {
            Lang::Js
        };
        let output = match transform_source(&actual, lang, case) {
            Ok(output) => output,
            Err(_) => return Ok(false),
        };
        return if output == actual {
            Ok(true)
        } else {
            Err("transformed output is not idempotent".into())
        };
    }
    let expected = print_expected_output(case)?;
    write_comparison_outputs(case, runner_name, &actual, &expected)?;
    if normalize_output(&actual, case)? == normalize_output(&expected, case)? {
        Ok(true)
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
    let output_root = output_root(runner_name).join(C::NAME);

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
    transform_source(case.code(), case.lang(), case)
}

fn transform_source<C: TransformCase>(
    source: &str,
    lang: Lang,
    case: &C,
) -> Result<String, String> {
    let allocator = Allocator::new();
    let mut ast = parse_program_with_options(
        source,
        &allocator,
        ParserOptions {
            source_type: case.source_type(),
            lang,
            ..ParserOptions::default()
        },
    )?;
    let options = case.transformer_options();
    if options.typescript.is_some() {
        Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    } else {
        Normalizer::normalize(ast.root_program(), &mut ast);
    }
    let semantic_ret = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: true,
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

    Transformer::transform(&mut ast, semantic_ret.semantic, options);
    Ok(print(&allocator, &ast, case.codegen_options())
        .code
        .to_owned())
}

fn print_expected_output<C: TransformCase>(case: &C) -> Result<String, String> {
    let source = case.expected_output().expect("expected output fixture");
    let allocator = Allocator::new();
    let ast = parse_program(source, &allocator, case)?;
    Ok(print(&allocator, &ast, case.codegen_options())
        .code
        .to_owned())
}

/// Compare reparsed output with comments and without redundant sequence parentheses.
/// Keep the readable output files above for diagnosing remaining differences.
fn normalize_output<C: TransformCase>(source: &str, case: &C) -> Result<String, String> {
    let allocator = Allocator::new();
    let ast = parse_program_with_options(
        source,
        &allocator,
        ParserOptions {
            source_type: case.source_type(),
            lang: case.lang(),
            preserve_parens: false,
            comments: CommentMode::Flat,
        },
    )?;
    let mut options = case.codegen_options();
    options.minify = true;
    options.comments = true;
    Ok(print(&allocator, &ast, options).code.to_owned())
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
    parse_program_with_options(source, allocator, options)
}

fn parse_program_with_options<'a>(
    source: &'a str,
    allocator: &'a Allocator,
    options: ParserOptions,
) -> Result<Ast<'a>, String> {
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
