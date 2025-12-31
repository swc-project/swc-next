use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_ecma_ast::{Ast, Visit};
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
        #[cfg(not(miri))]
        let iter = cases.par_iter();

        #[cfg(miri)]
        let iter = cases.iter();

        iter.filter_map(|case| {
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
            let mut collector = ParenCollector {
                ast: &ast,
                count: 0,
            };
            collector.visit_program(root);
            if collector.count > 0 {
                return Some(TestResult::Failed {
                    path: case.relative_path().to_owned(),
                    error: format!("ParenExpr is detected {}", collector.count),
                });
            }

            Some(TestResult::Passed {
                path: case.relative_path().to_owned(),
            })
        })
        .collect()
    }
}

struct ParenCollector<'a> {
    ast: &'a Ast,
    count: usize,
}

impl<'a> Visit for ParenCollector<'a> {
    fn ast(&self) -> &Ast {
        self.ast
    }

    fn visit_paren_expr(&mut self, _node: swc_experimental_ecma_ast::ParenExpr) {
        self.count += 1;
    }
}
