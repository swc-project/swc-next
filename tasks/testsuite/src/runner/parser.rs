use colored::Colorize;
use rayon::prelude::*;
use swc_experimental_allocator::Allocator;

use crate::{
    AppArgs,
    cases::Case,
    runner::{ParseResult, parse},
    suite::TestResult,
};

pub struct ParserRunner;

impl ParserRunner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        #[cfg(not(miri))]
        let iter = cases.par_iter();

        #[cfg(miri)]
        let iter = cases.iter();

        iter.map(|case| {
            if args.debug {
                println!("[{}] {:?}", "Debug".green(), case.relative_path());
            }

            if case.should_ignore() {
                return TestResult::Ignored {
                    path: case.relative_path().to_path_buf(),
                };
            }

            let allocator = Allocator::new();
            match (case.should_fail(), parse(&allocator, case)) {
                (false, ParseResult::Succ(_)) => TestResult::Passed {
                    path: case.relative_path().to_owned(),
                },
                (true, ParseResult::Succ(_)) => TestResult::Failed {
                    path: case.relative_path().to_owned(),
                    error: "Expected failure, but parsed successfully".to_string(),
                },
                (false, ParseResult::Fail(errors)) => TestResult::Failed {
                    path: case.relative_path().to_owned(),
                    error: format!("{:?}", errors),
                },
                (true, ParseResult::Fail(_)) => TestResult::Passed {
                    path: case.relative_path().to_owned(),
                },
                (_, ParseResult::Panic) => TestResult::Panic {
                    path: case.relative_path().to_owned(),
                },
                (_, ParseResult::Ignore) => TestResult::Ignored {
                    path: case.relative_path().to_owned(),
                },
            }
        })
        .collect()
    }
}
