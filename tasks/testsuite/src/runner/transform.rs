use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, Severity, SourceType};
use swc_next_ecma_codegen::print;
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};
use swc_next_ecma_transformer::{Transformer, normalize::Normalizer};

use crate::{cases::TransformCase, runner::Runner, suite::TestResult, util::workspace_root};

pub struct TransformerRunner;

impl<C: TransformCase> Runner<C> for TransformerRunner {
    const NAME: &'static str = "transform";

    fn run(case: &C) -> TestResult {
        let path = case
            .path()
            .strip_prefix(workspace_root().expect("workspace root should be discoverable"))
            .expect("transform fixture should be inside the workspace")
            .to_path_buf();
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
    let allocator = Allocator::new();
    let Some(ast) = transform_program(
        case,
        &allocator,
        case.code(),
        case.source_type(),
        case.lang(),
    )?
    else {
        return Ok(false);
    };

    let expected = case.expected_output();
    let output = print(&allocator, &ast, case.codegen_options()).code;
    if let Some(expected) = expected {
        let expected_allocator = Allocator::new();
        let Some(expected_ast) = parse_program(
            &expected_allocator,
            expected,
            case.source_type(),
            case.lang(),
            true,
        ) else {
            return Ok(false);
        };
        let expected = print(&expected_allocator, &expected_ast, case.codegen_options()).code;
        let Some(actual) = normalize_output(case, output) else {
            return Ok(false);
        };
        let Some(expected) = normalize_output(case, expected) else {
            return Ok(false);
        };
        if actual != expected {
            return Err("transformed output differs from expected output".into());
        }
    } else {
        if case.lang() == Lang::Dts {
            if !output.is_empty() {
                return Err("declaration input emitted JavaScript".into());
            }
            return Ok(true);
        }
        let output_lang = if case.lang() == Lang::Tsx {
            Lang::Jsx
        } else {
            Lang::Js
        };
        let output_allocator = Allocator::new();
        // TODO: Configure decorators and module transforms when those passes are supported.
        // Keep the input's parsing mode even when unambiguous parsing resolved its AST.
        let Some(output_ast) = transform_program(
            case,
            &output_allocator,
            output,
            case.source_type(),
            output_lang,
        )?
        else {
            return Ok(false);
        };
        let retransformed = print(&output_allocator, &output_ast, case.codegen_options()).code;
        if output != retransformed {
            return Err("transformed output is not idempotent".into());
        }
    }
    Ok(true)
}

fn transform_program<'a, C: TransformCase>(
    case: &C,
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
    lang: Lang,
) -> Result<Option<Ast<'a>>, String> {
    let Some(mut ast) = parse_program(allocator, source, source_type, lang, true) else {
        return Ok(None);
    };
    let options = case.transformer_options();
    if options.typescript.is_some() {
        Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    } else {
        Normalizer::normalize(ast.root_program(), &mut ast);
    }
    let semantic_ret = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
            build_module_record: false,
        },
    );
    if semantic_ret
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Ok(None);
    }
    let transformed = Transformer::transform(&mut ast, semantic_ret.semantic, options);
    if let Some(diagnostic) = transformed
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "transform: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }
    Ok(Some(ast))
}

fn parse_program<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
    lang: Lang,
    preserve_parens: bool,
) -> Option<Ast<'a>> {
    let result = Parser::init(
        allocator,
        source,
        ParserOptions {
            source_type,
            lang,
            preserve_parens,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    if result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return None;
    }
    Some(result.ast)
}

/// Compare output with comments while ignoring redundant parentheses and formatting.
fn normalize_output<C: TransformCase>(case: &C, source: &str) -> Option<String> {
    let allocator = Allocator::new();
    let ast = parse_program(&allocator, source, case.source_type(), case.lang(), false)?;
    let mut options = case.codegen_options();
    options.minify = true;
    options.comments = true;
    Some(print(&allocator, &ast, options).code.to_owned())
}
