use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_ecma_semantic::resolver::resolver;

use crate::{
    AppArgs,
    cases::Case,
    runner::{ParseResult, parse},
    suite::TestResult,
};

pub struct SemanticRunner;

impl SemanticRunner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        cases
            .par_iter()
            .filter_map(|case| {
                if args.debug {
                    println!("[{}] {:?}", "Debug".green(), case.relative_path());
                }

                if case.should_ignore() {
                    return Some(TestResult::Ignored {
                        path: case.relative_path().to_owned(),
                    });
                }

                let (root, ast) = match parse(case) {
                    ParseResult::Succ(ret) => ret,
                    _ => return None,
                };

                let _semantic = resolver(root, &ast);
                Some(TestResult::Passed {
                    path: case.path().to_owned(),
                })
            })
            .collect()
    }
}
