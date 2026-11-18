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
use swc_next_ecma_semantic2::{Semantic, SymbolFlags};

#[derive(Serialize)]
struct TestSymbolSnapshot {
    symbols: BTreeMap<String, Vec<TestSymbol>>,
    #[serde(skip_serializing_if = "TestScopes::is_empty")]
    scopes: TestScopes,
}

#[derive(Serialize)]
struct TestSymbol {
    id: usize,
    flags: String,
    declarations: Vec<TestDeclaration>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bindings: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    uses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    export_symbol: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    members: Option<BTreeMap<String, Option<TestMember>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<TestAlias>,
    #[serde(skip_serializing_if = "Option::is_none")]
    import: Option<TestImport>,
}

/// Missing flags inherit the symbol's flags. A missing node_symbol means the
/// queried symbol; explicit null means that the declaration node has no symbol.
#[derive(Serialize)]
struct TestDeclaration {
    span: String,
    scope: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_symbol: Option<Option<usize>>,
    /// Present when this symbol shares a declaration owned by another symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<usize>,
}

#[derive(Serialize)]
struct TestMember {
    symbol: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    enum_flags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace_flags: Option<String>,
}

#[derive(Serialize)]
struct TestImport {
    initializer: Option<String>,
}

struct SymbolSnapshotBuilder<'a, 's> {
    ast: &'s Ast<'a>,
    semantic: &'s Semantic<'a>,
    imports: TestImportEquals<'a, 's>,
    scopes: TestScopes,
}

impl SymbolSnapshotBuilder<'_, '_> {
    fn symbol(&mut self, id: SymbolId, exports: &[String]) -> TestSymbol {
        let semantic = self.semantic;
        let ast = self.ast;
        let symbol = semantic.symbol(id);
        let import = self
            .imports
            .entries
            .iter()
            .find(|(symbol, _)| *symbol == Some(id))
            .map(|(_, reference)| TestImport {
                initializer: reference
                    .map(|reference| span(ast, semantic.reference(reference).node)),
            });
        let name = ast.get_wtf8(symbol.name).as_bytes();
        for scope in semantic.iter_scopes() {
            if semantic.ts_container_owner(scope.id) == Some(id) {
                self.scopes.insert(semantic, scope.id);
            }
        }
        let mut members = semantic
            .ts_container_members(id)
            .map(|(name, member)| {
                let member = TestMember {
                    symbol: member.symbol.index(),
                    enum_flags: member.enum_flags.map(flags),
                    namespace_flags: member.namespace_flags.map(flags),
                };
                (String::from_utf8_lossy(name).into_owned(), Some(member))
            })
            .collect::<BTreeMap<_, _>>();
        let is_container = symbol.flags.intersects(
            SymbolFlags::NAMESPACE_MODULE
                | SymbolFlags::VALUE_MODULE
                | SymbolFlags::REGULAR_ENUM
                | SymbolFlags::CONST_ENUM,
        );
        for name in exports {
            members.entry(name.clone()).or_insert(None);
        }
        TestSymbol {
            id: id.index(),
            flags: flags(symbol.flags),
            declarations: semantic
                .declaration_ids(id)
                .map(|declaration_id| {
                    let declaration = semantic.declaration(declaration_id);
                    if declaration.symbol != id {
                        assert!(
                            semantic
                                .declaration_ids(declaration.symbol)
                                .any(|id| id == declaration_id)
                        );
                    }
                    let node_symbol = semantic.symbol_of(declaration.node).map(Idx::index);
                    TestDeclaration {
                        span: span(ast, declaration.node),
                        scope: self.scopes.insert(semantic, declaration.scope),
                        flags: (declaration.flags != symbol.flags)
                            .then(|| flags(declaration.flags)),
                        node_symbol: if node_symbol == Some(id.index()) {
                            None
                        } else {
                            Some(node_symbol)
                        },
                        owner: (declaration.symbol != id).then(|| declaration.symbol.index()),
                    }
                })
                .collect(),
            bindings: semantic
                .iter_scopes()
                .filter(|scope| semantic.binding(scope.id, name) == Some(id))
                .map(|scope| scope.id.index())
                .collect(),
            uses: semantic
                .uses(id)
                .iter()
                .map(|&reference| span(ast, semantic.reference(reference).node))
                .collect(),
            export_symbol: semantic.ts_export_symbol(id).map(Idx::index),
            members: (is_container || !members.is_empty()).then_some(members),
            alias: semantic.ts_import_alias(id).map(TestAlias::from),
            import,
        }
    }
}

fn symbols(source: &str, options: TestOptions, query: &FixtureOptions) -> TestSymbolSnapshot {
    with_semantic(source, options, |ast, semantic| {
        let semantic = &semantic;
        let mut builder = SymbolSnapshotBuilder {
            ast,
            semantic,
            imports: TestImportEquals::new(ast, semantic),
            scopes: TestScopes::default(),
        };
        let symbols = query
            .names
            .iter()
            .map(|name| {
                let exports = query.exports.get(name).map_or(&[][..], Vec::as_slice);
                let symbols = semantic
                    .iter_symbols()
                    .filter(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == name.as_bytes())
                    .map(|entry| builder.symbol(entry.id, exports))
                    .collect();
                (name.clone(), symbols)
            })
            .collect();
        TestSymbolSnapshot {
            symbols,
            scopes: builder.scopes,
        }
    })
}

#[test]
fn symbol() {
    fixtures::run("symbol", symbols);
}
