mod support;

use std::collections::BTreeMap;

use serde::Serialize;
use support::{
    analysis::with_semantic,
    fixtures::{self, FixtureOptions},
    model::*,
    options::TestOptions,
};
use swc_next_allocator::Idx;
use swc_next_ecma_ast::{Ast, SymbolId};
use swc_next_ecma_semantic2::Semantic;

#[derive(Serialize)]
struct TestReferenceSnapshot {
    references: BTreeMap<String, Vec<TestReference>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    symbols: BTreeMap<usize, TestSymbol>,
    #[serde(skip_serializing_if = "TestScopes::is_empty")]
    scopes: TestScopes,
}

#[derive(Serialize)]
struct TestReference {
    span: String,
    scope: usize,
    space: String,
    access: TestAccess,
    resolution: TestResolution,
    candidates: Vec<usize>,
    value_symbols: Vec<TestResolution>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    import_equals: bool,
}

#[derive(Serialize)]
enum TestAccess {
    None,
    Read,
    Write,
    ReadWrite,
}

#[derive(Serialize)]
struct TestSymbol {
    name: String,
    flags: String,
    declarations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias_target: Option<usize>,
}

/// Include directly referenced symbols and terminal alias targets, without
/// expanding all namespace members or intermediate import-alias chains.
fn add_symbol(
    symbols: &mut BTreeMap<usize, TestSymbol>,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    id: SymbolId,
) {
    let mut pending = vec![id];
    while let Some(id) = pending.pop() {
        if symbols.contains_key(&id.index()) {
            continue;
        }
        let symbol = semantic.symbol(id);
        let alias_target = semantic.ts_import_alias_target(id);
        symbols.insert(
            id.index(),
            TestSymbol {
                name: String::from_utf8_lossy(ast.get_wtf8(symbol.name).as_bytes()).into_owned(),
                flags: flags(symbol.flags),
                declarations: semantic
                    .declarations(id)
                    .map(|declaration| span(ast, declaration.node))
                    .collect(),
                alias_target: alias_target.map(Idx::index),
            },
        );
        if let Some(target) = alias_target {
            pending.push(target);
        }
    }
}

fn references(source: &str, options: TestOptions, query: &FixtureOptions) -> TestReferenceSnapshot {
    with_semantic(source, options, |ast, semantic| {
        let semantic = &semantic;
        let nodes = TestNodes::new(ast, semantic);
        let mut scopes = TestScopes::default();
        let mut symbols = BTreeMap::new();
        let references = query
            .names
            .iter()
            .map(|name| {
                let references = semantic
                    .iter_references()
                    .filter(|entry| ast.get_utf8(entry.reference.name) == name)
                    .map(|entry| {
                        let reference = entry.reference;
                        let resolution = reference.resolution();
                        let candidates =
                            semantic.reference_candidates(entry.id).collect::<Vec<_>>();
                        let value_symbols = semantic
                            .reference_value_targets(ast, entry.id)
                            .collect::<Vec<_>>();
                        for &id in &candidates {
                            add_symbol(&mut symbols, ast, semantic, id);
                        }
                        for resolution in
                            std::iter::once(resolution).chain(value_symbols.iter().copied())
                        {
                            match resolution {
                                NameResolution::Resolved(id)
                                | NameResolution::Uncertain { candidate: id } => {
                                    add_symbol(&mut symbols, ast, semantic, id)
                                }
                                NameResolution::Unresolved => {}
                            }
                        }
                        TestReference {
                            span: span(ast, reference.node),
                            scope: scopes.insert(semantic, reference.scope),
                            space: format!("{:?}", reference.flags.space),
                            access: match (reference.flags.is_read(), reference.flags.is_write()) {
                                (false, false) => TestAccess::None,
                                (true, false) => TestAccess::Read,
                                (false, true) => TestAccess::Write,
                                (true, true) => TestAccess::ReadWrite,
                            },
                            resolution: resolution.into(),
                            candidates: candidates.into_iter().map(Idx::index).collect(),
                            value_symbols: value_symbols
                                .into_iter()
                                .map(TestResolution::from)
                                .collect(),
                            import_equals: nodes
                                .imports
                                .iter()
                                .any(|(_, reference)| *reference == Some(entry.id)),
                        }
                    })
                    .collect();
                (name.clone(), references)
            })
            .collect();
        TestReferenceSnapshot {
            references,
            symbols,
            scopes,
        }
    })
}

#[test]
fn reference() {
    fixtures::run("reference", references);
}
