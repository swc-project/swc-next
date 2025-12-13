use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_ecma_ast::NodeKind;
use swc_experimental_ecma_transforms_base::remove_paren;

use crate::{
    AppArgs,
    cases::Case,
    runner::{ParseResult, parse},
    suite::TestResult,
};

pub struct RemoveParenRunner;

impl RemoveParenRunner {
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

                let (root, mut ast) = match parse(case) {
                    ParseResult::Succ(ret) => ret,
                    _ => return None,
                };

                remove_paren::remove_paren(root, &mut ast, None);
                for (_, node_id) in ast.nodes() {
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
