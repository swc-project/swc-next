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

/// Empty binding/use lists are omitted. Domain-specific views are present only
/// for the relevant symbol kinds, including empty namespaces and enums.
#[derive(Serialize)]
struct TestSymbol {
    id: usize,
    flags: String,
    declarations: Vec<TestDeclaration>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bindings: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    uses: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    roles: Vec<TestSymbolRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<TestNamespace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#enum: Option<TestEnum>,
    #[serde(skip_serializing_if = "Option::is_none")]
    import: Option<TestImport>,
}

/// Missing flags inherit the symbol's flags. A missing node_symbol means the
/// owning symbol; explicit null means that the declaration node has no symbol.
#[derive(Serialize)]
struct TestDeclaration {
    span: String,
    scope: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_symbol: Option<Option<usize>>,
}

#[derive(Serialize)]
enum TestSymbolRole {
    SourceModule,
    DefaultExport,
}

#[derive(Serialize)]
struct TestNamespace {
    members: BTreeMap<String, TestMember>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    export_aliases: bool,
}

#[derive(Serialize)]
struct TestEnum {
    members: BTreeMap<String, TestMember>,
}

/// Alias-only exports and negative queries have no declared member symbol.
/// Empty value targets remain explicit: an exported type need not be a value.
#[derive(Serialize)]
struct TestMember {
    symbol: Option<usize>,
    value_symbols: Vec<TestResolution>,
}

#[derive(Serialize)]
struct TestImport {
    target: Option<usize>,
    initializer: Option<String>,
}

fn member(
    semantic: &Semantic<'_>,
    owner: SymbolId,
    name: &str,
    symbol: Option<SymbolId>,
) -> TestMember {
    TestMember {
        symbol: symbol.map(Idx::index),
        value_symbols: semantic
            .ts_namespace_value_targets(owner, name.as_bytes())
            .map(TestResolution::from)
            .collect(),
    }
}

fn member_name(ast: &Ast<'_>, semantic: &Semantic<'_>, owner: SymbolId, id: SymbolId) -> String {
    if semantic.ts_source_module_symbol() == Some(owner)
        && semantic.ts_default_export_symbol() == Some(id)
    {
        "default".to_owned()
    } else {
        String::from_utf8_lossy(ast.get_wtf8(semantic.symbol(id).name).as_bytes()).into_owned()
    }
}

struct SymbolSnapshotBuilder<'a, 's> {
    ast: &'s Ast<'a>,
    semantic: &'s Semantic<'a>,
    nodes: TestNodes<'a, 's>,
    scopes: TestScopes,
}

impl SymbolSnapshotBuilder<'_, '_> {
    fn members(
        &self,
        owner: SymbolId,
        symbols: impl Iterator<Item = SymbolId>,
    ) -> BTreeMap<String, TestMember> {
        symbols
            .map(|symbol| {
                let name = member_name(self.ast, self.semantic, owner, symbol);
                let member = member(self.semantic, owner, &name, Some(symbol));
                (name, member)
            })
            .collect()
    }

    fn query_members<'q>(
        &self,
        owner: SymbolId,
        members: &mut BTreeMap<String, TestMember>,
        names: impl Iterator<Item = &'q str>,
        lookup: impl Fn(&[u8]) -> Option<SymbolId>,
    ) {
        for name in names {
            members
                .entry(name.to_owned())
                .or_insert_with(|| member(self.semantic, owner, name, lookup(name.as_bytes())));
        }
    }

    fn containers(
        &self,
        id: SymbolId,
        exports: &[String],
    ) -> (Option<TestNamespace>, Option<TestEnum>) {
        let semantic = self.semantic;
        let flags = semantic.symbol(id).flags;
        let mut namespace_members = self.members(id, semantic.ts_namespace_members(id));
        let mut enum_members = self.members(id, semantic.ts_enum_members(id));
        let export_aliases = semantic.ts_namespace_has_export_aliases(id);
        let is_namespace = semantic.ts_source_module_symbol() == Some(id)
            || flags.contains(SymbolFlags::NAMESPACE_MODULE)
            || !namespace_members.is_empty()
            || export_aliases;
        let is_enum = flags.intersects(SymbolFlags::REGULAR_ENUM | SymbolFlags::CONST_ENUM)
            || !enum_members.is_empty();
        let namespace_exports = if is_namespace || !is_enum {
            exports
        } else {
            &[]
        };
        let mut names = self
            .nodes
            .export_names
            .get(&id)
            .cloned()
            .unwrap_or_default();
        names.extend(namespace_exports.iter().cloned());
        self.query_members(
            id,
            &mut namespace_members,
            names.iter().map(String::as_str),
            |name| semantic.ts_namespace_member(id, name),
        );
        if is_enum && !is_namespace {
            self.query_members(
                id,
                &mut enum_members,
                exports.iter().map(String::as_str),
                |name| semantic.ts_enum_member(id, name),
            );
        }
        (
            (is_namespace || !namespace_members.is_empty()).then_some(TestNamespace {
                members: namespace_members,
                export_aliases,
            }),
            is_enum.then_some(TestEnum {
                members: enum_members,
            }),
        )
    }

    fn symbol(&mut self, id: SymbolId, exports: &[String]) -> TestSymbol {
        let semantic = self.semantic;
        let ast = self.ast;
        let symbol = semantic.symbol(id);
        let (namespace, enum_view) = self.containers(id, exports);
        let mut roles = Vec::new();
        if semantic.ts_source_module_symbol() == Some(id) {
            roles.push(TestSymbolRole::SourceModule);
        }
        if semantic.ts_default_export_symbol() == Some(id) {
            roles.push(TestSymbolRole::DefaultExport);
        }
        let alias_target = semantic.ts_import_alias_target(id);
        let initializer = self
            .nodes
            .imports
            .iter()
            .find(|(symbol, _)| *symbol == Some(id));
        let import = (symbol.flags.contains(SymbolFlags::IMPORT_EQUALS)
            || alias_target.is_some()
            || initializer.is_some())
        .then(|| TestImport {
            target: alias_target.map(Idx::index),
            initializer: initializer.and_then(|(_, reference)| {
                reference.map(|reference| span(ast, semantic.reference(reference).node))
            }),
        });
        let name = ast.get_wtf8(symbol.name).as_bytes();
        TestSymbol {
            id: id.index(),
            flags: flags(symbol.flags),
            declarations: semantic
                .declarations(id)
                .map(|declaration| {
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
            roles,
            namespace,
            r#enum: enum_view,
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
            nodes: TestNodes::new(ast, semantic),
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
