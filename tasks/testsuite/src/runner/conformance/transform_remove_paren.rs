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
    cases::Case,
    runner::{
        ParseResult,
        conformance::{
            LegacyParseResult, collect_legacy_node_spans, compat_experimental_program,
            format_node_span_mismatch, parse_legacy,
        },
        parse,
    },
    suite::TestResult,
};

pub struct RemoveParenConformanceRunner;

impl RemoveParenConformanceRunner {
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

            if case.should_fail() {
                return None;
            }

            let allocator = Allocator::new();
            let mut experimental_root = match parse(&allocator, case) {
                ParseResult::Succ(ret) => ret,
                _ => return None,
            };
            let mut legacy_root = match parse_legacy(case) {
                LegacyParseResult::Succ(ret) => ret,
                _ => return None,
            };

            remove_paren::remove_paren(&mut experimental_root, &allocator, None);
            legacy_root.visit_mut_with(&mut legacy_paren_remover(None));
            let experimental_root = compat_experimental_program(experimental_root);

            let legacy_nodes = collect_legacy_node_spans(&legacy_root);
            let experimental_nodes = collect_legacy_node_spans(&experimental_root);
            if let Some(error) = format_node_span_mismatch(
                "Remove paren node conformance mismatch",
                &legacy_nodes,
                &experimental_nodes,
            ) {
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
