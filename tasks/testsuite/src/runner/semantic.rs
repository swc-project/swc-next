use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{
    BlockStmt, BreakStmt, ContinueStmt, ExportSpecifier, Ident, ImportNamedSpecifier, LabeledStmt,
    Visit, VisitWith,
};
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

            let allocator = Allocator::new();
            let root = match parse(&allocator, case) {
                ParseResult::Succ(ret) => ret,
                _ => return None,
            };

            let _semantic = resolver(&root);

            // check semantic
            if !case.syntax().export_default_from() {
                let mut collector = SemanticIdsCollector::default();
                root.visit_with(&mut collector);
                if !collector.errors.is_empty() {
                    return Some(TestResult::Failed {
                        path: case.relative_path().to_owned(),
                        error: collector.errors.join("\n"),
                    });
                }
            }

            Some(TestResult::Passed {
                path: case.path().to_owned(),
            })
        })
        .collect()
    }
}

#[derive(Default)]
struct SemanticIdsCollector {
    errors: Vec<String>,
}

impl<'a> Visit<'a> for SemanticIdsCollector {
    fn visit_break_stmt(&mut self, _: &BreakStmt<'a>) {}

    fn visit_continue_stmt(&mut self, _: &ContinueStmt<'a>) {}

    fn visit_import_named_specifier(&mut self, node: &ImportNamedSpecifier<'a>) {
        if node.imported.is_some() {
            node.local.visit_with(self);
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_export_specifier(&mut self, node: &ExportSpecifier<'a>) {
        if let ExportSpecifier::Default(export_default_specifier) = node {
            export_default_specifier.visit_with(self);
        }
    }

    fn visit_labeled_stmt(&mut self, node: &LabeledStmt<'a>) {
        node.body.visit_with(self);
    }

    fn visit_block_stmt(&mut self, node: &BlockStmt<'a>) {
        if node.scope_id.get().is_none() {
            self.errors
                .push(format!("Missing ScopeId for BlockStmt: {:?}", node.span));
        }

        node.visit_children_with(self);
    }

    fn visit_ident(&mut self, node: &Ident<'a>) {
        if node.symbol_id.get().is_none() {
            self.errors.push(format!(
                "Missing SymbolId for Ident {:?}: {:?}",
                node.sym, node.span
            ));
        }
    }
}
