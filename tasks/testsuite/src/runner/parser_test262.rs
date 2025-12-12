use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_core::common::comments::SingleThreadedComments;
use swc_experimental_ecma_ast::Program;
use swc_experimental_ecma_parser::{Lexer, Parser, StringSource};

use crate::{AppArgs, cases::Case, suite::TestResult};

pub struct Test262Runner;

impl Test262Runner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        cases
            .par_iter()
            .map(|case| {
                if args.debug {
                    println!("[{}] {:?}", "Debug".green(), case.relative_path());
                }

                let filename = case.filename();

                let input = StringSource::new(case.code());
                let comments = SingleThreadedComments::default();
                let lexer = Lexer::new(case.syntax(), Default::default(), input, Some(&comments));
                let parser = Parser::new_from(lexer);
                let ret = if filename.ends_with(".module.js") {
                    parser
                        .parse_module()
                        .map(|ret| ret.map_root(Program::Module))
                } else {
                    parser
                        .parse_script()
                        .map(|ret| ret.map_root(Program::Script))
                };

                let errors = match ret {
                    Ok(ret) => ret.errors,
                    Err(e) => vec![e],
                };
                match (case.should_fail(), !errors.is_empty()) {
                    (true, false) => TestResult::Failed {
                        path: case.relative_path().to_path_buf(),
                        error: "Expected failure, but parsed successfully".to_string(),
                    },
                    (true, true) => TestResult::Passed {
                        path: case.relative_path().to_path_buf(),
                    },
                    (false, false) => TestResult::Passed {
                        path: case.relative_path().to_path_buf(),
                    },
                    (false, true) => TestResult::Failed {
                        path: case.relative_path().to_path_buf(),
                        error: format!("{:?}", errors),
                    },
                }
            })
            .collect()
    }
}
