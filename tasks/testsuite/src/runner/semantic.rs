use std::{
    collections::{BTreeMap, HashMap},
    fmt::Write,
    panic::{AssertUnwindSafe, catch_unwind},
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, NodeKindData};
use swc_next_ecma_semantic::{AnalyzeOptions, Semantic, analyze};

use crate::{
    cases::{Case, swc_resolver::SwcResolverCase},
    runner::{Runner, parse_case},
    suite::TestResult,
};

pub struct SemanticRunner;

impl Runner<SwcResolverCase> for SemanticRunner {
    const NAME: &'static str = "semantic";

    fn run(case: &SwcResolverCase) -> TestResult {
        let path = case.relative_path().to_path_buf();
        let result = catch_unwind(AssertUnwindSafe(|| run_case(case)));
        match result {
            Ok(Ok(())) => TestResult::Passed {
                path,
                negative: false,
            },
            Ok(Err(error)) => TestResult::Failed {
                path,
                error,
                negative: false,
            },
            Err(_) => TestResult::Panic {
                path,
                negative: false,
            },
        }
    }
}

fn run_case(case: &SwcResolverCase) -> Result<(), String> {
    let allocator = Allocator::new();
    let ast = parse_case(&allocator, case)?;
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let actual = semantic_snapshot(&ast, &semantic);

    if actual == case.expected() {
        Ok(())
    } else {
        Err(format!("{}: name resolution differs", case.name()))
    }
}

// Identifier end offsets make ordering independent of AST traversal and scope allocation.
// Zero denotes a name without an explicit binding in the semantic model.
fn semantic_snapshot(ast: &Ast<'_>, semantic: &Semantic<'_>) -> String {
    let mut identifiers = BTreeMap::new();
    for entry in semantic.iter_declarations() {
        let node = entry.declaration.node;
        if let NodeKindData::BindingIdentifier(_) = ast.kind_data(node) {
            let span = ast.span(node);
            // Annex B companions share their syntax node with the lexical binding.
            identifiers.insert(span.end, semantic.symbol_of(node));
        }
    }
    for entry in semantic.iter_references() {
        let span = ast.span(entry.reference.node);
        identifiers.insert(span.end, entry.reference.symbol);
    }

    let mut bindings = HashMap::new();
    let mut output = String::new();
    for (end, symbol) in identifiers {
        let target = symbol.map_or(0, |symbol| {
            let next = bindings.len() + 1;
            *bindings.entry(symbol).or_insert(next)
        });
        writeln!(output, "{end} -> {target}").unwrap();
    }
    output
}
