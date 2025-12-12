use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_core::common::comments::SingleThreadedComments;
use swc_experimental_ecma_ast::{NodeKind, Program};
use swc_experimental_ecma_parser::{Lexer, Parser, StringSource};
use swc_experimental_ecma_transforms_base::remove_paren;

use crate::{AppArgs, cases::Case, suite::TestResult};

pub struct RemoveParenRunner;

impl RemoveParenRunner {
    pub fn run<C: Case>(args: &AppArgs, cases: &[C]) -> Vec<TestResult> {
        cases
            .par_iter()
            .filter_map(|case| {
                if args.debug {
                    println!("[{}] {:?}", "Debug".green(), case.relative_path());
                }

                if case.should_fail() {
                    return None;
                }

                let syntax = case.syntax();
                let input = StringSource::new(case.code());
                let comments = SingleThreadedComments::default();
                let lexer = Lexer::new(syntax, Default::default(), input, Some(&comments));
                let parser = Parser::new_from(lexer);
                let mut ret = match case.ext().as_str() {
                    "js" | "jsx" => parser.parse_program(),
                    "cjs" => parser
                        .parse_script()
                        .map(|ret| ret.map_root(Program::Script)),
                    "mjs" => parser
                        .parse_module()
                        .map(|ret| ret.map_root(Program::Module)),
                    "ts" | "tsx" => {
                        return Some(TestResult::Ignored {
                            path: case.path().to_owned(),
                        });
                    }
                    _ => unreachable!(),
                }
                .expect("Failure cases are filtered");

                remove_paren::remove_paren(ret.root, &mut ret.ast, None);
                for (_, node_id) in ret.ast.nodes() {
                    if node_id.kind() == NodeKind::ParenExpr {
                        return Some(TestResult::Failed {
                            path: case.relative_path().to_owned(),
                            error: "ParenExpr is detected".to_string(),
                        });
                    }
                }

                Some(TestResult::Passed {
                    path: case.relative_path().to_owned(),
                })
            })
            .collect()
    }
}
