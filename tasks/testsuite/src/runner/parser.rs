use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Diagnostic;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze, check_syntax};

use crate::{cases::Case, runner::Runner, suite::TestResult};

pub struct ParserRunner;

impl<C: Case> Runner<C> for ParserRunner {
    const NAME: &'static str = "parser";

    fn run(case: &C) -> TestResult {
        let path = case.relative_path().to_path_buf();
        let result = catch_unwind(AssertUnwindSafe(|| {
            let allocator = Allocator::new();
            let options = Options {
                source_type: case.source_type(),
                lang: case.lang(),
                ..Options::default()
            };
            let parsed =
                Parser::init(&allocator, case.code(), options, NoTokenParserConfig).parse();
            if let Some(error) = parsed.diagnostics.first() {
                if case.expects_error() {
                    return check_error_spans(case, &parsed.diagnostics);
                }
                return Err(format!(
                    "{}: parser: {} at {}..{}",
                    case.name(),
                    error.message,
                    error.span.start,
                    error.span.end
                ));
            }
            let ast = parsed.ast;
            if !case.expects_error()
                && let Some(expected) = case.expected_source_type()
            {
                let actual = ast.source_type();
                if actual != expected {
                    return Err(format!(
                        "{}: expected source type {expected:?}, but parser produced {actual:?}",
                        case.name()
                    ));
                }
            }
            let semantic_ret = analyze(
                &ast,
                AnalyzeOptions {
                    check_syntax: SyntaxChecks::all(),
                },
            );
            let diagnostics = check_syntax(&ast, SyntaxChecks::all());
            if diagnostics != semantic_ret.diagnostics {
                return Err(format!(
                    "{}: syntax checking and semantic analysis diagnostics differ",
                    case.name()
                ));
            }
            match (case.expects_error(), diagnostics.first()) {
                (true, Some(_)) => check_error_spans(case, &diagnostics),
                (false, None) => Ok(()),
                (false, Some(_)) if !case.asserts_semantic_success() => Ok(()),
                (true, None) => Err(format!(
                    "{}: expected a parse or early error, but pipeline succeeded",
                    case.name()
                )),
                (false, Some(error)) => Err(format!(
                    "{}: semantic: {} at {}..{}",
                    case.name(),
                    error.message,
                    error.span.start,
                    error.span.end
                )),
            }
        }));

        let negative = case.expects_error();
        match result {
            Ok(Ok(())) => TestResult::Passed { path, negative },
            Ok(Err(error)) => TestResult::Failed {
                path,
                error,
                negative,
            },
            Err(_) => TestResult::Panic { path, negative },
        }
    }
}

fn check_error_spans<C: Case>(case: &C, diagnostics: &[Diagnostic<'_>]) -> Result<(), String> {
    let Some(expected) = case.expected_error_spans() else {
        return Ok(());
    };
    for diagnostic in diagnostics {
        let actual = diagnostic.span;
        let contained = actual.start <= actual.end
            && expected
                .iter()
                .any(|span| span.start <= actual.start && actual.end <= span.end);
        if !contained {
            return Err(format!(
                "{}: diagnostic span {actual:?} is not contained in reference spans {expected:?}",
                case.name()
            ));
        }
    }
    Ok(())
}
