use std::collections::HashMap;

use colored::Colorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use swc_core::{
    atoms::Atom as LegacyAtom,
    common::{GLOBALS, Globals, Mark, SyntaxContext},
    ecma::{
        ast as legacy_ast,
        transforms::base::resolver as legacy_resolver,
        visit::{
            Visit as LegacyVisit, VisitMutWith as LegacyVisitMutWith, VisitWith as LegacyVisitWith,
        },
    },
};
use swc_experimental_allocator::Allocator;
use swc_experimental_allocator::atom::Atom as ExperimentalAtom;
use swc_experimental_ecma_ast::{
    Ident, ImportNamedSpecifier, NamedExport, Program, ScopeId, Visit, VisitWith,
};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};

use crate::{
    AppArgs,
    cases::Case,
    runner::{
        ParseResult,
        conformance::{LegacyParseResult, parse_legacy_with_current_globals},
        parse,
    },
    suite::TestResult,
};

type LegacySymbolSpans = HashMap<(LegacyAtom, SyntaxContext), Vec<SymbolSpan>>;
type ExperimentalSymbolSpans<'a> = HashMap<(ExperimentalAtom<'a>, ScopeId), Vec<SymbolSpan>>;
type SymbolGroups = HashMap<String, Vec<Vec<SymbolSpan>>>;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SymbolSpan {
    start: u32,
    end: u32,
}

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
    let Some(legacy_symbol_spans) = collect_legacy_symbols(case) else {
        return Ok(false);
    };
    let legacy_symbols = normalize_symbol_groups(
        legacy_symbol_spans
            .into_iter()
            .map(|((sym, _), spans)| (sym.to_string(), spans)),
    );

    let mut experimental_collector = ExperimentalSymbolsCollector {
        semantic,
        symbols: HashMap::default(),
    };
    root.visit_with(&mut experimental_collector);
    let experimental_symbols = normalize_symbol_groups(
        experimental_collector
            .symbols
            .into_iter()
            .map(|((sym, _), spans)| (sym.to_string(), spans)),
    );

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
    for (sym, legacy_groups, experimental_groups) in mismatches.iter().take(20) {
        error.push_str(&format!(
            "\n  {sym}: swc_core={}, swc_experimental={}",
            format_symbol_groups(legacy_groups),
            format_symbol_groups(experimental_groups),
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

fn collect_legacy_symbols<C: Case>(case: &C) -> Option<LegacySymbolSpans> {
    GLOBALS.set(&Globals::new(), || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        debug_assert_eq!(unresolved_mark.as_u32(), 1);
        debug_assert_eq!(top_level_mark.as_u32(), 2);

        let mut program = match parse_legacy_with_current_globals(case) {
            LegacyParseResult::Succ(program) => program,
            _ => return None,
        };

        program.visit_mut_with(&mut legacy_resolver(unresolved_mark, top_level_mark, false));

        let mut collector = LegacySymbolsCollector::default();
        LegacyVisitWith::visit_with(&program, &mut collector);

        Some(collector.symbols)
    })
}

#[derive(Default)]
struct LegacySymbolsCollector {
    symbols: LegacySymbolSpans,
}

impl LegacyVisit for LegacySymbolsCollector {
    fn visit_ident(&mut self, node: &legacy_ast::Ident) {
        self.symbols
            .entry((node.sym.clone(), node.ctxt))
            .or_default()
            .push(SymbolSpan {
                start: node.span.lo.0,
                end: node.span.hi.0,
            });
    }

    fn visit_import_named_specifier(&mut self, node: &legacy_ast::ImportNamedSpecifier) {
        LegacyVisitWith::visit_with(&node.local, self);
    }

    fn visit_named_export(&mut self, node: &legacy_ast::NamedExport) {
        if node.src.is_none() {
            node.visit_children_with(self);
        }
    }
}

struct ExperimentalSymbolsCollector<'semantic, 'ast> {
    semantic: &'semantic Semantic,
    symbols: ExperimentalSymbolSpans<'ast>,
}

impl<'a> Visit<'a> for ExperimentalSymbolsCollector<'_, 'a> {
    fn visit_import_named_specifier(&mut self, node: &ImportNamedSpecifier<'a>) {
        if node.imported.is_some() {
            node.local.visit_with(self);
            return;
        }
        node.visit_children_with(self);
    }

    fn visit_named_export(&mut self, node: &NamedExport<'a>) {
        if node.src.is_none() {
            node.visit_children_with(self);
        }
    }

    fn visit_ident(&mut self, node: &Ident<'a>) {
        node.symbol_id.get().unwrap();

        self.symbols
            .entry((node.sym, self.semantic.node_scope(node)))
            .or_default()
            .push(SymbolSpan {
                start: node.span.start,
                end: node.span.end,
            });
    }
}

fn normalize_symbol_groups<I>(groups: I) -> SymbolGroups
where
    I: IntoIterator<Item = (String, Vec<SymbolSpan>)>,
{
    let mut symbols = HashMap::<String, Vec<Vec<SymbolSpan>>>::default();
    for (sym, mut spans) in groups {
        spans.sort_unstable();
        symbols.entry(sym).or_default().push(spans);
    }
    for groups in symbols.values_mut() {
        groups.sort_unstable();
    }
    symbols
}

fn format_symbol_groups(groups: &[Vec<SymbolSpan>]) -> String {
    let mut ret = String::from("[");
    for (i, group) in groups.iter().take(6).enumerate() {
        if i > 0 {
            ret.push_str(", ");
        }
        ret.push('[');
        for (j, span) in group.iter().take(8).enumerate() {
            if j > 0 {
                ret.push_str(", ");
            }
            ret.push_str(&format!("{}..{}", span.start, span.end));
        }
        if group.len() > 8 {
            ret.push_str(&format!(", ... +{}", group.len() - 8));
        }
        ret.push(']');
    }
    if groups.len() > 6 {
        ret.push_str(&format!(", ... +{}", groups.len() - 6));
    }
    ret.push(']');
    ret
}
