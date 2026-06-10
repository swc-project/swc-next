use std::collections::HashMap;

use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_core::{
    atoms::Atom as LegacyAtom,
    common::{BytePos, GLOBALS, Globals, Mark, SyntaxContext},
    ecma::{
        ast as legacy_ast,
        parser::{
            EsSyntax as LegacyEsSyntax, Parser as LegacyParser, StringInput,
            Syntax as LegacySyntax, lexer::Lexer,
        },
        transforms::base::resolver as legacy_resolver,
        visit::{
            Visit as LegacyVisit, VisitMutWith as LegacyVisitMutWith, VisitWith as LegacyVisitWith,
        },
    },
};
use swc_experimental_allocator::Allocator;
use swc_experimental_allocator::atom::Atom as ExperimentalAtom;
use swc_experimental_ecma_ast::{
    BreakStmt, ContinueStmt, Ident, ImportNamedSpecifier, LabeledStmt, NamedExport, Program,
    ScopeId, Visit, VisitWith,
};
use swc_experimental_ecma_parser::Syntax;
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};

use crate::{
    AppArgs,
    cases::{Case, IsModule},
    runner::{ParseResult, parse},
    suite::TestResult,
};

type LegacySymbolCounts = HashMap<(LegacyAtom, SyntaxContext), u32>;
type ExperimentalSymbolCounts<'a> = HashMap<(ExperimentalAtom<'a>, ScopeId), u32>;

pub struct SemanticConformanceRunner;

impl SemanticConformanceRunner {
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

            if case.syntax().export_default_from() {
                return None;
            }

            let allocator = Allocator::new();
            let root = match parse(&allocator, case) {
                ParseResult::Succ(ret) => ret,
                _ => return None,
            };
            let semantic = resolver(&root);

            match check_symbol_conformance(case, &root, &semantic) {
                Ok(true) => Some(TestResult::Passed {
                    path: case.path().to_owned(),
                }),
                Ok(false) => None,
                Err(error) => Some(TestResult::Failed {
                    path: case.relative_path().to_owned(),
                    error,
                }),
            }
        })
        .collect()
    }
}

fn check_symbol_conformance<'a, C: Case>(
    case: &C,
    root: &Program<'a>,
    semantic: &Semantic,
) -> Result<bool, String> {
    let mut legacy_symbols = HashMap::<String, Vec<u32>>::default();
    let Some(legacy_symbol_counts) = collect_legacy_symbols(case) else {
        return Ok(false);
    };
    for ((sym, _), count) in legacy_symbol_counts {
        legacy_symbols
            .entry(sym.to_string())
            .or_default()
            .push(count);
    }
    for counts in legacy_symbols.values_mut() {
        counts.sort_unstable();
    }

    let mut experimental_collector = ExperimentalSymbolsCollector {
        semantic,
        symbols: HashMap::default(),
    };
    root.visit_with(&mut experimental_collector);
    let mut experimental_symbols = HashMap::<String, Vec<u32>>::default();
    for ((sym, _), count) in experimental_collector.symbols {
        experimental_symbols
            .entry(sym.to_string())
            .or_default()
            .push(count);
    }
    for counts in experimental_symbols.values_mut() {
        counts.sort_unstable();
    }

    if legacy_symbols == experimental_symbols {
        return Ok(true);
    }

    let mut mismatches = Vec::new();
    for sym in legacy_symbols.keys().chain(experimental_symbols.keys()) {
        let legacy_counts = legacy_symbols.get(sym).cloned().unwrap_or_default();
        let experimental_counts = experimental_symbols.get(sym).cloned().unwrap_or_default();
        if legacy_counts != experimental_counts {
            mismatches.push((sym.clone(), legacy_counts, experimental_counts));
        }
    }
    mismatches.sort_by(|a, b| a.0.cmp(&b.0));
    mismatches.dedup_by(|a, b| a.0 == b.0);

    let mut error = String::from("Semantic symbol conformance mismatch");
    for (sym, legacy_counts, experimental_counts) in mismatches.iter().take(20) {
        error.push_str(&format!(
            "\n  {sym}: swc_core={legacy_counts:?}, swc_experimental={experimental_counts:?}"
        ));
    }
    if mismatches.len() > 20 {
        error.push_str(&format!(
            "\n  ... and {} more mismatches",
            mismatches.len() - 20
        ));
    }

    Err(error)
}

fn collect_legacy_symbols<C: Case>(case: &C) -> Option<LegacySymbolCounts> {
    GLOBALS.set(&Globals::new(), || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        debug_assert_eq!(unresolved_mark.as_u32(), 1);
        debug_assert_eq!(top_level_mark.as_u32(), 2);

        let input = StringInput::new(case.code(), BytePos(0), BytePos(case.code().len() as u32));
        let lexer = Lexer::new(
            legacy_syntax(case.syntax()),
            Default::default(),
            input,
            None,
        );
        let mut parser = LegacyParser::new_from(lexer);

        let mut program = match case.is_module() {
            IsModule::Script => parser.parse_script().map(legacy_ast::Program::Script),
            IsModule::Module => parser.parse_module().map(legacy_ast::Program::Module),
            IsModule::Unknown => parser.parse_program(),
            IsModule::Skip => {
                unreachable!("semantic runner ignores skipped cases before this point")
            }
        }
        .ok()?;

        let errors = parser.take_errors();
        if !errors.is_empty() {
            return None;
        }

        program.visit_mut_with(&mut legacy_resolver(unresolved_mark, top_level_mark, false));

        let mut collector = LegacySymbolsCollector::default();
        LegacyVisitWith::visit_with(&program, &mut collector);

        Some(collector.symbols)
    })
}

fn legacy_syntax(syntax: Syntax) -> LegacySyntax {
    match syntax {
        Syntax::Es(es) => LegacySyntax::Es(LegacyEsSyntax {
            jsx: es.jsx,
            fn_bind: es.fn_bind,
            decorators: es.decorators,
            decorators_before_export: es.decorators_before_export,
            export_default_from: es.export_default_from,
            import_attributes: es.import_attributes,
            allow_super_outside_method: es.allow_super_outside_method,
            allow_return_outside_function: es.allow_return_outside_function,
            auto_accessors: es.auto_accessors,
            explicit_resource_management: es.explicit_resource_management,
        }),
    }
}

#[derive(Default)]
struct LegacySymbolsCollector {
    symbols: LegacySymbolCounts,
}

impl LegacyVisit for LegacySymbolsCollector {
    fn visit_break_stmt(&mut self, _: &legacy_ast::BreakStmt) {}

    fn visit_continue_stmt(&mut self, _: &legacy_ast::ContinueStmt) {}

    fn visit_ident(&mut self, node: &legacy_ast::Ident) {
        *self
            .symbols
            .entry((node.sym.clone(), node.ctxt))
            .or_default() += 1;
    }

    fn visit_import_named_specifier(&mut self, node: &legacy_ast::ImportNamedSpecifier) {
        LegacyVisitWith::visit_with(&node.local, self);
    }

    fn visit_labeled_stmt(&mut self, node: &legacy_ast::LabeledStmt) {
        LegacyVisitWith::visit_with(&node.body, self);
    }

    fn visit_named_export(&mut self, node: &legacy_ast::NamedExport) {
        if node.src.is_none() {
            node.visit_children_with(self);
        }
    }
}

struct ExperimentalSymbolsCollector<'semantic, 'ast> {
    semantic: &'semantic Semantic,
    symbols: ExperimentalSymbolCounts<'ast>,
}

impl<'a> Visit<'a> for ExperimentalSymbolsCollector<'_, 'a> {
    fn visit_break_stmt(&mut self, _: &BreakStmt<'a>) {}

    fn visit_continue_stmt(&mut self, _: &ContinueStmt<'a>) {}

    fn visit_import_named_specifier(&mut self, node: &ImportNamedSpecifier<'a>) {
        if node.imported.is_some() {
            node.local.visit_with(self);
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_labeled_stmt(&mut self, node: &LabeledStmt<'a>) {
        node.body.visit_with(self);
    }

    fn visit_named_export(&mut self, node: &NamedExport<'a>) {
        if node.src.is_none() {
            node.visit_children_with(self);
        }
    }

    fn visit_ident(&mut self, node: &Ident<'a>) {
        if node.symbol_id.get().is_none() {
            return;
        }

        *self
            .symbols
            .entry((node.sym, self.semantic.node_scope(node)))
            .or_default() += 1;
    }
}
