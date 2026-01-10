use colored::Colorize;
use rayon::prelude::*;

use crate::{
    AppArgs,
    cases::Case,
    runner::{ParseResult, parse},
    suite::TestResult,
};

pub struct ParserRunner;

impl ParserRunner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        let run_case = |case: &C| {
            if args.debug {
                println!("[{}] {:?}", "Debug".green(), case.relative_path());
            }

            if case.should_ignore() {
                return TestResult::Ignored {
                    path: case.relative_path().to_path_buf(),
                };
            }

            match (case.should_fail(), parse(case)) {
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
            }
        };

        #[cfg(not(miri))]
        if args.debug {
            cases.iter().map(run_case).collect()
        } else {
            cases.par_iter().map(run_case).collect()
        }

        #[cfg(miri)]
        cases.iter().map(run_case).collect()
    }
}
