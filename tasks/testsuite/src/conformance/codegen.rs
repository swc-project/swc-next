use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_allocator::Allocator;

use crate::legacy;
use crate::{
    AppArgs,
    cases::Case,
    runner::{
        ParseResult,
        codegen::{format_code_diff, generate_code},
        parse,
    },
    suite::TestResult,
};

pub struct CodegenConformanceRunner;

impl CodegenConformanceRunner {
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
            let experimental_root = match parse(&allocator, case) {
                ParseResult::Succ(ret) => ret,
                _ => return None,
            };
            let legacy_root = match legacy::parse(case) {
                legacy::ParseResult::Succ(ret) => ret,
                _ => return None,
            };

            let experimental_code = match generate_code(experimental_root) {
                Ok(ret) => ret,
                Err(()) => {
                    return Some(TestResult::Panic {
                        path: case.relative_path().to_owned(),
                    });
                }
            };
            let legacy_code = swc_core::ecma::codegen::to_code(&legacy_root);

            if experimental_code != legacy_code {
                return Some(TestResult::Failed {
                    path: case.relative_path().to_owned(),
                    error: format!(
                        "Codegen conformance mismatch\n{}",
                        format_code_diff(
                            "swc_core",
                            &legacy_code,
                            "swc_experimental",
                            &experimental_code
                        )
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
