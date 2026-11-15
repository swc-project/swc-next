use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_core::js::{CommentsOption, JsTransformApiOptions, transformer::transform_js};
use swc_next_ecma_ast::{AstCtx, Diagnostic, Lang, Severity};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};

use crate::{cases::TransformCase, runner::Runner, suite::TestResult, util::workspace_root};

pub struct TransformerRunner;

impl<C: TransformCase> Runner<C> for TransformerRunner {
    const NAME: &'static str = "transform";

    fn run(case: &C) -> TestResult {
        let root = workspace_root().unwrap();
        let path = case.path().strip_prefix(root).unwrap().to_path_buf();
        if !case.supports_options() {
            return TestResult::Skipped { path };
        }
        match catch_unwind(AssertUnwindSafe(|| run_case(case))) {
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
}

fn run_case<C: TransformCase>(case: &C) -> Result<bool, String> {
    let mut options = case.options();
    let parser = options.parser.unwrap_or_default();
    let lang: Lang = parser.lang.map(Into::into).unwrap_or_default();
    let output = transform_js::<true>(case.code(), options.clone());
    if let Err(error) = check_diagnostics(&output.diagnostics) {
        return if case.expected_output().is_some() {
            Err(error)
        } else {
            Ok(false)
        };
    }

    if let Some(expected) = case.expected_output() {
        let allocator = Allocator::default();
        let Some(ast) = parse_program(&allocator, &options, expected, true) else {
            return Ok(false);
        };
        let expected = generate(&ast, generator_options(&options)).code;
        let Some(actual) = normalize_output(&options, &output.code) else {
            return Ok(false);
        };
        let Some(expected) = normalize_output(&options, &expected) else {
            return Ok(false);
        };
        if actual != expected {
            return Err("transformed output differs from expected output".into());
        }
    } else {
        if lang == Lang::Dts {
            if !output.code.is_empty() {
                return Err("declaration input emitted JavaScript".into());
            }
            return Ok(true);
        }
        let output_lang = if lang == Lang::Tsx {
            Lang::Jsx
        } else {
            Lang::Js
        };
        options.parser.get_or_insert_with(Default::default).lang = Some(output_lang.into());
        let retransformed = transform_js::<true>(&output.code, options);
        if check_diagnostics(&retransformed.diagnostics).is_err() {
            return Ok(false);
        }
        if output.code != retransformed.code {
            return Err("transformed output is not idempotent".into());
        }
    }
    Ok(true)
}

fn check_diagnostics(diagnostics: &[Diagnostic]) -> Result<(), String> {
    if let Some(diagnostic) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "transform: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }
    Ok(())
}

fn parse_program<'a>(
    allocator: &'a Allocator,
    options: &JsTransformApiOptions,
    source: &'a str,
    preserve_parens: bool,
) -> Option<AstCtx<'a>> {
    let parser = options.parser.unwrap_or_default();
    let result = Parser::init(
        allocator,
        source,
        ParserOptions {
            preserve_parens,
            comments: CommentMode::Flat,
            ..parser.into()
        },
        NoTokenParserConfig,
    )
    .parse();
    check_diagnostics(&result.diagnostics).ok()?;
    Some(result.ast)
}

/// Compare output with comments while ignoring redundant parentheses and formatting.
fn normalize_output(options: &JsTransformApiOptions, source: &str) -> Option<String> {
    let allocator = Allocator::default();
    let ast = parse_program(&allocator, options, source, false)?;
    let mut generator = generator_options(options);
    generator.minify = true;
    generator.comments = true;
    Some(generate(&ast, generator).code)
}

fn generator_options(options: &JsTransformApiOptions) -> GeneratorOptions {
    let parser = options.parser.unwrap_or_default();
    let comments = options.comments.unwrap_or(CommentsOption::All);
    let parser_comments = parser.comments.unwrap_or(CommentsOption::All);
    let mut generator = GeneratorOptions::from(options.generator.unwrap_or_default());
    generator.comments = comments == CommentsOption::All && parser_comments == CommentsOption::All;
    generator
}
