use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Program, Severity};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::analyze;
use swc_next_ecma_transformer::{Transformer, TransformerOptions};

use crate::{
    cases::{Case, babel::BabelTransformCase},
    runner::Runner,
    suite::TestResult,
};

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

fn run_case(case: &BabelTransformCase) -> Result<(), String> {
    let actual = transform_input(case)?;
    let expected = normalize_expected_output(case)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{}: transformed output differs from output.js",
            case.name
        ))
    }
}

fn transform_input(case: &BabelTransformCase) -> Result<Vec<u8>, String> {
    let source = case.input.as_bytes();
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = parse_program(&mut ast, case)?;
    let semantic = analyze(program, &mut ast);
    if let Some(diagnostic) = semantic
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "semantic: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }

    let options: TransformerOptions = case.options.clone().into();
    Transformer::transform(program, &mut ast, &semantic, options);
    Ok(print(&allocator, &ast, program, CodegenOptions::default())
        .code
        .to_vec())
}

fn normalize_expected_output(case: &BabelTransformCase) -> Result<Vec<u8>, String> {
    let source = case.output.as_bytes();
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = parse_program(&mut ast, case)?;
    Ok(print(&allocator, &ast, program, CodegenOptions::default())
        .code
        .to_vec())
}

fn parse_program(ast: &mut Ast<'_>, case: &BabelTransformCase) -> Result<Program, String> {
    let options = ParserOptions {
        source_type: case.source_type(),
        lang: case.lang(),
        ..ParserOptions::default()
    };
    let (program, error) = {
        let result = Parser::init(ast, options, NoTokenParserConfig).parse();
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
        (result.program, error)
    };

    if let Some(error) = error {
        return Err(error);
    }
    Ok(program)
}
