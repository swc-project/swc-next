use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, ParenExpr, Visit, VisitWith};
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

            let allocator = Allocator::new();
            let (mut root, mut ast) = match parse(&allocator, case) {
                ParseResult::Succ(ret, ast) => (ret, ast),
                _ => return None,
            };

            remove_paren::remove_paren(&mut root, &mut ast, None);
            let mut collector = ParenCollector {
                ast: &ast,
                count: 0,
            };
            root.visit_with(&mut collector);
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

struct ParenCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    count: usize,
}

impl<'ast> Visit<'ast> for ParenCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_paren_expr(&mut self, _node: ParenExpr) {
        self.count += 1;
    }
}
