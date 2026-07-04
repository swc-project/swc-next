use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_allocator::Allocator;

use crate::{
    AppArgs,
    ast_compare::ast_compare,
    cases::Case,
    conformance::convert_experimental_program,
    legacy,
    runner::{ParseResult, parse},
    suite::TestResult,
};

pub struct ParserConformanceRunner;

impl ParserConformanceRunner {
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

            let allocator = Allocator::new();
            let experimental_root = match parse(&allocator, case) {
                ParseResult::Succ(ret) => ret,
                _ => return None,
            };
            let legacy_root = match legacy::parse(case) {
                legacy::ParseResult::Succ(ret) => ret,
                _ => return None,
            };
            let experimental_root = convert_experimental_program(experimental_root);
            if let Some(error) = ast_compare(&legacy_root, &experimental_root) {
                return Some(TestResult::Failed {
                    path: case.relative_path().to_owned(),
                    error,
                });
            }

            Some(TestResult::Passed {
                path: case.relative_path().to_owned(),
            })
        })
        .collect()
    }
}
