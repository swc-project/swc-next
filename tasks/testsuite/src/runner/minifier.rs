use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, Severity, SourceType};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_minifier::{Minifier, MinifierOptions};
use swc_next_ecma_parser::{CommentMode, Options as ParserOptions};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};

use crate::{
    cases::MinifierCase,
    runner::{Runner, parse_with_options},
    suite::TestResult,
};

pub struct MinifierRunner;

impl<C: MinifierCase> Runner<C> for MinifierRunner {
    const NAME: &'static str = "minifier";

    fn run(case: &C) -> TestResult {
        let path = case.relative_path().to_path_buf();
        match catch_unwind(AssertUnwindSafe(|| run_case(case))) {
            Ok(None) => TestResult::Skipped { path },
            Ok(Some((negative, Ok(())))) => TestResult::Passed { path, negative },
            Ok(Some((negative, Err(error)))) => TestResult::Failed {
                path,
                error,
                negative,
            },
            Err(_) => TestResult::Panic {
                path,
                negative: false,
            },
        }
    }
}

/// Minifies with the fixture options, then checks that the output is valid,
/// that compression is stable, and that the output matches the reference
/// output except for formatting and redundant parentheses.
///
/// Returns `None` for a skipped case. Otherwise, the result is negative when
/// the reference output equals the input, which checks that the minifier keeps
/// the code unchanged.
fn run_case<C: MinifierCase>(case: &C) -> Option<(bool, Result<(), String>)> {
    let source_type = case.source_type();
    // Inputs that are not valid JavaScript, such as JSX, are out of scope, and
    // a reference output that the parser rejects cannot be compared.
    let input = normalize(case.code(), source_type).ok()?;
    let expected = normalize(case.expected_output()?, source_type).ok()?;
    let options = case.minifier_options();
    let first = minify(case.code(), source_type, options.clone()).ok()?;
    Some((input == expected, check_output(&first, &expected, options)))
}

fn check_output(first: &Minified, expected: &str, options: MinifierOptions) -> Result<(), String> {
    // Names are planned before compression removes references, so a second
    // pass can choose different names. Only compression must be stable.
    let source_type = if first.module {
        SourceType::Module
    } else {
        SourceType::Script
    };
    let repeat_options = MinifierOptions {
        mangle: false,
        ..options
    };
    let second = minify(&first.code, source_type, repeat_options)
        .map_err(|error| format!("output: {error}"))?;
    if first.code != second.code {
        return Err("minification is not stable".into());
    }

    let actual = normalize(&first.code, source_type).map_err(|error| format!("output: {error}"))?;
    if actual != expected {
        return Err("output differs from reference output".into());
    }
    Ok(())
}

/// Prints `source` compactly without comments or redundant parentheses.
fn normalize(source: &str, source_type: SourceType) -> Result<String, String> {
    let allocator = Allocator::new();
    let ast = parse_with_options(&allocator, source, parser_options(source_type))?;
    Ok(generate(&ast, generator_options()).code)
}

fn parser_options(source_type: SourceType) -> ParserOptions {
    ParserOptions {
        source_type,
        lang: Lang::Js,
        preserve_parens: false,
        comments: CommentMode::None,
    }
}

fn generator_options() -> GeneratorOptions {
    GeneratorOptions {
        minify: true,
        comments: false,
        ..GeneratorOptions::default()
    }
}

struct Minified {
    code: String,
    module: bool,
}

fn minify(
    source: &str,
    source_type: SourceType,
    options: MinifierOptions,
) -> Result<Minified, String> {
    let allocator = Allocator::new();
    let mut ast = parse_with_options(&allocator, source, parser_options(source_type))?;
    let analysis = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
        },
    );
    if let Some(error) = analysis
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "semantic: {} at {}..{}",
            error.message, error.span.start, error.span.end
        ));
    }
    drop(analysis);

    Minifier::minify(&mut ast, options);
    let code = generate(&ast, generator_options()).code;
    Ok(Minified {
        code,
        module: ast.is_module(),
    })
}
