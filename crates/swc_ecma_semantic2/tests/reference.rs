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
use swc_next_ecma_semantic2::{Semantic, TsImportAlias};

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
    symbol: Option<usize>,
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
    export_symbol: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<TestAlias>,
}

fn add_symbols(
    symbols: &mut BTreeMap<usize, TestSymbol>,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    target: SymbolId,
) {
    let export = semantic.ts_export_symbol(target);
    let alias = match semantic.ts_import_alias(target) {
        Some(TsImportAlias::Target(target)) => Some(target),
        _ => None,
    };
    // Include direct links while keeping long alias chains out of the snapshot.
    for id in std::iter::once(target).chain(export).chain(alias) {
        symbols.entry(id.index()).or_insert_with(|| {
            let symbol = semantic.symbol(id);
            TestSymbol {
                name: String::from_utf8_lossy(ast.get_wtf8(symbol.name).as_bytes()).into_owned(),
                flags: flags(symbol.flags),
                declarations: semantic
                    .declarations(id)
                    .map(|declaration| span(ast, declaration.node))
                    .collect(),
                export_symbol: semantic.ts_export_symbol(id).map(Idx::index),
                alias: semantic.ts_import_alias(id).map(TestAlias::from),
            }
        });
    }
}

fn references(source: &str, options: TestOptions, query: &FixtureOptions) -> TestReferenceSnapshot {
    with_semantic(source, options, |ast, semantic| {
        let semantic = &semantic;
        let imports = TestImportEquals::new(ast, semantic);
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
                        if let Some(id) = reference.symbol {
                            let uses = semantic.uses(id);
                            assert_eq!(
                                uses.iter().filter(|&&use_id| use_id == entry.id).count(),
                                1
                            );
                            add_symbols(&mut symbols, ast, semantic, id);
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
                            symbol: reference.symbol.map(Idx::index),
                            import_equals: imports
                                .entries
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
