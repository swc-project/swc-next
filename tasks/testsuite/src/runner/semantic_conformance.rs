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
    ExportSpecifier, Ident, ImportNamedSpecifier, Program, ScopeId, Visit, VisitWith,
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
                Ok(()) => Some(TestResult::Passed {
                    path: case.path().to_owned(),
                }),
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
) -> Result<(), String> {
    let legacy_symbols = collect_legacy_symbols(case)?
        .into_iter()
        .map(|((sym, ctxt), count)| ((sym.to_string(), ctxt.as_u32()), count))
        .collect::<HashMap<_, _>>();

    let mut experimental_collector = ExperimentalSymbolsCollector {
        semantic,
        symbols: HashMap::default(),
    };
    root.visit_with(&mut experimental_collector);
    let experimental_symbols = experimental_collector
        .symbols
        .into_iter()
        .map(|((sym, scope), count)| ((sym.to_string(), scope.raw()), count))
        .collect::<HashMap<_, _>>();

    if legacy_symbols == experimental_symbols {
        return Ok(());
    }

    let mut mismatches = Vec::new();
    for key in legacy_symbols.keys().chain(experimental_symbols.keys()) {
        let legacy_count = legacy_symbols.get(key).copied().unwrap_or_default();
        let experimental_count = experimental_symbols.get(key).copied().unwrap_or_default();
        if legacy_count != experimental_count {
            mismatches.push((key.clone(), legacy_count, experimental_count));
        }
    }
    mismatches.sort_by(|a, b| a.0.cmp(&b.0));
    mismatches.dedup_by(|a, b| a.0 == b.0);

    let mut error = String::from("Semantic symbol conformance mismatch");
    for ((sym, scope), legacy_count, experimental_count) in mismatches.iter().take(20) {
        error.push_str(&format!(
            "\n  {sym}@{scope}: swc_core={legacy_count}, swc_experimental={experimental_count}"
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

fn collect_legacy_symbols<C: Case>(case: &C) -> Result<LegacySymbolCounts, String> {
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
        .map_err(|err| format!("Failed to parse with swc_core: {err:?}"))?;

        let errors = parser.take_errors();
        if !errors.is_empty() {
            return Err(format!("swc_core parser emitted errors: {errors:?}"));
        }

        program.visit_mut_with(&mut legacy_resolver(unresolved_mark, top_level_mark, false));

        let mut collector = LegacySymbolsCollector::default();
        LegacyVisitWith::visit_with(&program, &mut collector);

        Ok(collector.symbols)
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
    fn visit_ident(&mut self, node: &legacy_ast::Ident) {
        *self
            .symbols
            .entry((node.sym.clone(), node.ctxt))
            .or_default() += 1;
    }
}

struct ExperimentalSymbolsCollector<'semantic, 'ast> {
    semantic: &'semantic Semantic,
    symbols: ExperimentalSymbolCounts<'ast>,
}

impl<'a> Visit<'a> for ExperimentalSymbolsCollector<'_, 'a> {
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

    fn visit_ident(&mut self, node: &Ident<'a>) {
        *self
            .symbols
            .entry((node.sym, self.semantic.node_scope(node)))
            .or_default() += 1;
    }
}
