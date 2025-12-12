use colored::Colorize;
use rayon::prelude::*;
use swc_core::common::comments::SingleThreadedComments;
use swc_experimental_ecma_ast::Program;
use swc_experimental_ecma_parser::{Lexer, Parser, StringSource};

use crate::{AppArgs, cases::Case, suite::TestResult};

pub struct ParserRunner;

impl ParserRunner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        cases
            .par_iter()
            .map(|case| {
                if args.debug {
                    println!("[{}] {:?}", "Debug".green(), case.relative_path());
                }

                if case.should_ignore() {
                    return TestResult::Ignored {
                        path: case.relative_path().to_path_buf(),
                    };
                }

                let syntax = case.syntax();
                let input = StringSource::new(case.code());
                let comments = SingleThreadedComments::default();
                let lexer = Lexer::new(syntax, Default::default(), input, Some(&comments));
                let parser = Parser::new_from(lexer);
                let ret = match case.ext().as_str() {
                    "cjs" => parser
                        .parse_script()
                        .map(|ret| ret.map_root(Program::Script)),
                    "mjs" => parser
                        .parse_module()
                        .map(|ret| ret.map_root(Program::Module)),
                    "js" => {
                        if case.filename().ends_with(".module.js") {
                            parser
                                .parse_module()
                                .map(|ret| ret.map_root(Program::Module))
                        } else {
                            parser
                                .parse_script()
                                .map(|ret| ret.map_root(Program::Script))
                        }
                    }
                    "jsx" => parser.parse_program(),
                    "ts" | "tsx" => {
                        return TestResult::Ignored {
                            path: case.path().to_owned(),
                        };
                    }
                    _ => unreachable!(),
                };

                let errors = match ret {
                    Ok(ret) => ret.errors,
                    Err(e) => vec![e],
                };
                match (case.should_fail(), !errors.is_empty()) {
                    (true, false) => TestResult::Failed {
                        path: case.relative_path().to_owned(),
                        error: "Expected failure, but parsed successfully".to_string(),
                    },
                    (true, true) => TestResult::Passed {
                        path: case.relative_path().to_owned(),
                    },
                    (false, false) => TestResult::Passed {
                        path: case.relative_path().to_owned(),
                    },
                    (false, true) => TestResult::Failed {
                        path: case.relative_path().to_owned(),
                        error: format!("{:?}", errors),
                    },
                }
            })
            .collect()
    }
}
