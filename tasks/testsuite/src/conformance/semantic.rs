use std::collections::{HashMap, HashSet};

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
use swc_experimental_ecma_ast::{BlockStmt, Ident, Program, ScopeId, Visit, VisitWith};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};

use crate::{
    AppArgs,
    cases::Case,
    legacy,
    runner::{ParseResult, parse},
    suite::TestResult,
};

type LegacySymbolSpans = HashMap<(LegacyAtom, SyntaxContext), Vec<SymbolSpan>>;
type LegacyBlockSpans = HashMap<SyntaxContext, Vec<SymbolSpan>>;
type ExperimentalSymbolSpans<'a> = HashMap<(ExperimentalAtom<'a>, ScopeId), Vec<SymbolSpan>>;
type ExperimentalBlockSpans = HashMap<ScopeId, Vec<SymbolSpan>>;
type SymbolGroups = HashMap<String, Vec<Vec<SymbolSpan>>>;
type ScopeGroups = Vec<Vec<SymbolSpan>>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct SymbolSpan {
    start: u32,
    end: u32,
}

pub struct SemanticConformanceRunner;

impl SemanticConformanceRunner {
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
    let Some(legacy_semantics) = collect_legacy_semantics(case) else {
        return Ok(false);
    };
    let comparable_symbol_spans = legacy_semantics
        .symbols
        .values()
        .flat_map(|spans| spans.iter().copied())
        .collect::<HashSet<_>>();
    let legacy_symbols = normalize_symbol_groups(
        legacy_semantics
            .symbols
            .into_iter()
            .map(|((sym, _), spans)| (sym.to_string(), spans)),
    );
    let legacy_blocks = normalize_scope_groups(legacy_semantics.blocks.into_values());

    let mut experimental_collector = ExperimentalSymbolsCollector {
        semantic,
        comparable_symbol_spans: &comparable_symbol_spans,
        symbols: HashMap::default(),
        blocks: HashMap::default(),
    };
    root.visit_with(&mut experimental_collector);
    let experimental_symbols = normalize_symbol_groups(
        experimental_collector
            .symbols
            .into_iter()
            .map(|((sym, _), spans)| (sym.to_string(), spans)),
    );
    let experimental_blocks = normalize_scope_groups(experimental_collector.blocks.into_values());

    if legacy_symbols == experimental_symbols && legacy_blocks == experimental_blocks {
        return Ok(true);
    }

    let mut symbol_mismatches = Vec::new();
    for sym in legacy_symbols.keys().chain(experimental_symbols.keys()) {
        let legacy_counts = legacy_symbols.get(sym).cloned().unwrap_or_default();
        let experimental_counts = experimental_symbols.get(sym).cloned().unwrap_or_default();
        if legacy_counts != experimental_counts {
            symbol_mismatches.push((sym.clone(), legacy_counts, experimental_counts));
        }
    }
    symbol_mismatches.sort_by(|a, b| a.0.cmp(&b.0));
    symbol_mismatches.dedup_by(|a, b| a.0 == b.0);

    let mut error = String::from("Semantic conformance mismatch");
    if !symbol_mismatches.is_empty() {
        error.push_str("\nSymbols:");
        for (sym, legacy_groups, experimental_groups) in symbol_mismatches.iter().take(20) {
            error.push_str(&format!(
                "\n  {sym}: swc_core={}, swc_experimental={}",
                format_span_groups(legacy_groups),
                format_span_groups(experimental_groups),
            ));
        }
        if symbol_mismatches.len() > 20 {
            error.push_str(&format!(
                "\n  ... and {} more symbol mismatches",
                symbol_mismatches.len() - 20
            ));
        }
    }

    if legacy_blocks != experimental_blocks {
        error.push_str(&format!(
            "\nBlock scopes:\n  swc_core={}\n  swc_experimental={}",
            format_span_groups(&legacy_blocks),
            format_span_groups(&experimental_blocks),
        ));
    }

    Err(error)
}

fn collect_legacy_semantics<C: Case>(case: &C) -> Option<LegacySemantics> {
    GLOBALS.set(&Globals::new(), || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        debug_assert_eq!(unresolved_mark.as_u32(), 1);
        debug_assert_eq!(top_level_mark.as_u32(), 2);

        let mut program = match legacy::parse(case) {
            legacy::ParseResult::Succ(program) => program,
            _ => return None,
        };

        program.visit_mut_with(&mut legacy_resolver(unresolved_mark, top_level_mark, false));

        let mut collector = LegacySymbolsCollector::default();
        LegacyVisitWith::visit_with(&program, &mut collector);

        Some(LegacySemantics {
            symbols: collector.symbols,
            blocks: collector.blocks,
        })
    })
}

struct LegacySemantics {
    symbols: LegacySymbolSpans,
    blocks: LegacyBlockSpans,
}

#[derive(Default)]
struct LegacySymbolsCollector {
    symbols: LegacySymbolSpans,
    blocks: LegacyBlockSpans,
}

impl LegacyVisit for LegacySymbolsCollector {
    fn visit_block_stmt(&mut self, node: &legacy_ast::BlockStmt) {
        if node.ctxt != SyntaxContext::empty() {
            self.blocks.entry(node.ctxt).or_default().push(SymbolSpan {
                start: node.span.lo.0,
                end: node.span.hi.0,
            });
        }

        node.visit_children_with(self);
    }

    fn visit_ident(&mut self, node: &legacy_ast::Ident) {
        if node.ctxt == SyntaxContext::empty() {
            return;
        }

        self.symbols
            .entry((node.sym.clone(), node.ctxt))
            .or_default()
            .push(SymbolSpan {
                start: node.span.lo.0,
                end: node.span.hi.0,
            });
    }
}

struct ExperimentalSymbolsCollector<'semantic, 'ast> {
    semantic: &'semantic Semantic,
    comparable_symbol_spans: &'semantic HashSet<SymbolSpan>,
    symbols: ExperimentalSymbolSpans<'ast>,
    blocks: ExperimentalBlockSpans,
}

impl<'a> Visit<'a> for ExperimentalSymbolsCollector<'_, 'a> {
    fn visit_block_stmt(&mut self, node: &BlockStmt<'a>) {
        if let Some(scope_id) = node.scope_id.get() {
            self.blocks.entry(scope_id).or_default().push(SymbolSpan {
                start: node.span.start,
                end: node.span.end,
            });
        }

        node.visit_children_with(self);
    }

    fn visit_ident(&mut self, node: &Ident<'a>) {
        let span = SymbolSpan {
            start: node.span.start,
            end: node.span.end,
        };

        if !self.comparable_symbol_spans.contains(&span) {
            return;
        }

        if node.symbol_id.get().is_none() {
            return;
        }

        self.symbols
            .entry((node.sym, self.semantic.node_scope(node)))
            .or_default()
            .push(span);
    }
}

fn normalize_scope_groups<I>(groups: I) -> ScopeGroups
where
    I: IntoIterator<Item = Vec<SymbolSpan>>,
{
    let mut groups = groups.into_iter().collect::<Vec<_>>();
    for group in &mut groups {
        group.sort_unstable();
    }
    groups.sort_unstable();
    groups
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

fn format_span_groups(groups: &[Vec<SymbolSpan>]) -> String {
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
