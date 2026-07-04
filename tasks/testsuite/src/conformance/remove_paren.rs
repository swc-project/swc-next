use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_core::ecma::{
    transforms::base::fixer::paren_remover as legacy_paren_remover,
    visit::VisitMutWith as LegacyVisitMutWith,
};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_transforms_base::remove_paren;

use crate::{
    AppArgs,
    ast_compare::ast_compare,
    cases::Case,
    conformance::convert_experimental_program,
    legacy,
    runner::{ParseResult, parse},
    suite::TestResult,
};

pub struct RemoveParenConformanceRunner;

impl RemoveParenConformanceRunner {
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
            let mut experimental_root = match parse(&allocator, case) {
                ParseResult::Succ(ret) => ret,
                _ => return None,
            };
            let mut legacy_root = match legacy::parse(case) {
                legacy::ParseResult::Succ(ret) => ret,
                _ => return None,
            };

            remove_paren::remove_paren(&mut experimental_root, &allocator, None);
            legacy_root.visit_mut_with(&mut legacy_paren_remover(None));
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
