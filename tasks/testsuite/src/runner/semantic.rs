use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{
    Ast, BlockStmt, ExportSpecifier, GetSpan, Ident, ImportNamedSpecifier, Visit, VisitWith,
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
            let (root, mut ast) = match parse(&allocator, case) {
                ParseResult::Succ(ret, ast) => (ret, ast),
                _ => return None,
            };

            let _semantic = resolver(root, &mut ast);

            // check semantic
            if !case.syntax().export_default_from() {
                let mut collector = SemanticIdsCollector {
                    ast: &ast,
                    errors: Vec::new(),
                };
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

struct SemanticIdsCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    errors: Vec<String>,
}

impl<'ast> Visit<'ast> for SemanticIdsCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_import_named_specifier(&mut self, node: ImportNamedSpecifier) {
        // swc_core only resolves the local binding for named import specifiers;
        // semantic conformance keeps this smoke test aligned with that behavior.
        node.local(self.ast).visit_with(self);
    }

    fn visit_export_specifier(&mut self, node: ExportSpecifier) {
        // swc_core does not treat every export specifier name as a local semantic
        // identifier; semantic conformance checks the exact swc_core boundary.
        if let ExportSpecifier::Default(export_default_specifier) = node {
            export_default_specifier.visit_with(self);
        }
    }

    fn visit_block_stmt(&mut self, node: BlockStmt) {
        if node.scope_id(self.ast).is_none() {
            self.errors.push(format!(
                "Missing ScopeId for BlockStmt: {:?}",
                node.span(self.ast)
            ));
        }

        node.visit_children_with(self);
    }

    fn visit_ident(&mut self, node: Ident) {
        if node.symbol_id(self.ast).is_none() {
            self.errors.push(format!(
                "Missing SymbolId for Ident {:?}: {:?}",
                node.sym(self.ast),
                node.span(self.ast)
            ));
        }
    }
}
