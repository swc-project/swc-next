use std::panic::{AssertUnwindSafe, catch_unwind};

use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::Program;
use swc_experimental_ecma_codegen::{Codegen, CodegenOptions};
use swc_experimental_ecma_semantic::resolver::resolver;

use crate::{
    AppArgs,
    cases::Case,
    runner::{ParseResult, parse, parse_code},
    suite::TestResult,
};

pub struct CodegenRunner;

impl CodegenRunner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        let iter = cases.par_iter();
        iter.filter_map(|case| {
            if args.debug {
                println!("[{}] {:?}", "Debug".green(), case.relative_path());
            }

            if case.should_ignore() {
                return Some(TestResult::Ignored {
                    path: case.relative_path().to_owned(),
                });
            }

            if case.should_fail() {
                return None;
            }

            if case.syntax().export_default_from() {
                return None;
            }

            let allocator = Allocator::new();
            let root = match parse(&allocator, case) {
                ParseResult::Succ(ret) => ret,
                _ => return None,
            };

            let first = match generate_code(root) {
                Ok(ret) => ret,
                Err(()) => {
                    return Some(TestResult::Panic {
                        path: case.relative_path().to_owned(),
                    });
                }
            };

            let second_allocator = Allocator::new();
            let reparsed =
                match parse_code(&second_allocator, &first, case.syntax(), case.is_module()) {
                    ParseResult::Succ(ret) => ret,
                    ParseResult::Fail(errors) => {
                        return Some(TestResult::Failed {
                            path: case.relative_path().to_owned(),
                            error: format!("Generated code failed to parse: {:?}", errors),
                        });
                    }
                    ParseResult::Panic => {
                        return Some(TestResult::Panic {
                            path: case.relative_path().to_owned(),
                        });
                    }
                    ParseResult::Ignore => return None,
                };

            let second = match generate_code(reparsed) {
                Ok(ret) => ret,
                Err(()) => {
                    return Some(TestResult::Panic {
                        path: case.relative_path().to_owned(),
                    });
                }
            };

            if first != second {
                return Some(TestResult::Failed {
                    path: case.relative_path().to_owned(),
                    error: format!(
                        "Codegen is not stable after reparsing\n{}",
                        format_code_diff("first", &first, "second", &second)
                    ),
                });
            }

            Some(TestResult::Passed {
                path: case.relative_path().to_owned(),
            })
        })
        .collect()
    }
}

pub fn generate_code(program: Program) -> Result<String, ()> {
    catch_unwind(AssertUnwindSafe(|| {
        let semantic = resolver(&program);
        Codegen::new(CodegenOptions::default()).build(program, &semantic)
    }))
    .map_err(|_| ())
}

pub fn format_code_diff(old_label: &str, old: &str, new_label: &str, new: &str) -> String {
    similar::TextDiff::from_lines(old, new)
        .unified_diff()
        .header(old_label, new_label)
        .to_string()
}
