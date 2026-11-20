#[path = "semantic/stress.rs"]
mod stress;
mod support;

use std::collections::BTreeMap;

use serde::Serialize;
use support::{
    analysis::analyze_fixture,
    fixtures,
    model::{TestNodes, TestResolution, flags, span},
};
use swc_next_allocator::Idx;
use swc_next_ecma_ast::{Ast, SymbolId};
use swc_next_ecma_semantic::{ReferenceEntry, ScopeEntry, Semantic, SymbolEntry, SymbolFlags};

#[derive(Serialize)]
struct Symbol {
    id: usize,
    name: String,
    flags: String,
    declarations: Vec<Declaration>,
    bindings: Vec<usize>,
    uses: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    linked_symbols: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    roles: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    members: Option<BTreeMap<String, Member>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    export_aliases: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    import: Option<Import>,
}

#[derive(Serialize)]
struct Declaration {
    span: String,
    scope: usize,
    // Equal flags and node symbols inherit the enclosing symbol's values.
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_symbol: Option<Option<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<usize>,
}

#[derive(Serialize)]
struct Import {
    target: Option<usize>,
    initializers: Vec<Option<usize>>,
}

#[derive(Serialize)]
struct Member {
    symbol: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace_flags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enum_flags: Option<String>,
    value_symbols: Vec<TestResolution>,
}

#[derive(Serialize)]
struct Reference {
    id: usize,
    name: String,
    span: String,
    scope: usize,
    space: String,
    access: &'static str,
    resolution: TestResolution,
    candidates: Vec<usize>,
    value_symbols: Vec<TestResolution>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    import_equals: bool,
}

#[derive(Serialize)]
struct Scope {
    id: usize,
    span: String,
    kind: String,
    parent: Option<usize>,
    hoist_target: usize,
    bindings: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_body: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    flags: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_owner: Option<usize>,
}

struct Snapshot<'a, 's> {
    ast: &'s Ast<'a>,
    semantic: &'s Semantic<'a>,
    nodes: TestNodes<'a, 's>,
}

impl Snapshot<'_, '_> {
    fn members(&self, owner: SymbolId) -> Option<BTreeMap<String, Member>> {
        let semantic = self.semantic;
        // Export specifiers may introduce names that have no declared member.
        // Include their local spellings to exercise private and type-only lookups.
        let mut names = self
            .nodes
            .export_names
            .get(&owner)
            .cloned()
            .unwrap_or_default();
        let member_name = |id| {
            if semantic.ts_source_module_symbol() == Some(owner)
                && semantic.ts_default_export_symbol() == Some(id)
            {
                "default".to_owned()
            } else {
                String::from_utf8_lossy(self.ast.get_wtf8(semantic.symbol(id).name).as_bytes())
                    .into_owned()
            }
        };
        for id in semantic.ts_namespace_members(owner) {
            let name = member_name(id);
            assert_eq!(
                semantic.ts_namespace_member(owner, name.as_bytes()),
                Some(id)
            );
            names.insert(name);
        }
        for id in semantic.ts_enum_members(owner) {
            let name = member_name(id);
            assert_eq!(semantic.ts_enum_member(owner, name.as_bytes()), Some(id));
            names.insert(name);
        }
        let is_container = semantic.ts_source_module_symbol() == Some(owner)
            || semantic.symbol(owner).flags.intersects(
                SymbolFlags::NAMESPACE_MODULE
                    | SymbolFlags::VALUE_MODULE
                    | SymbolFlags::REGULAR_ENUM
                    | SymbolFlags::CONST_ENUM,
            )
            || semantic.ts_namespace_has_export_aliases(owner);
        if names.is_empty() && !is_container {
            return None;
        }
        Some(
            names
                .into_iter()
                .map(|name| {
                    let member = self.member(owner, &name);
                    (name, member)
                })
                .collect(),
        )
    }

    fn member(&self, owner: SymbolId, name: &str) -> Member {
        let semantic = self.semantic;
        let namespace = semantic.ts_namespace_member(owner, name.as_bytes());
        let enum_member = semantic.ts_enum_member(owner, name.as_bytes());
        if let (Some(namespace), Some(enum_member)) = (namespace, enum_member) {
            assert_eq!(namespace, enum_member);
        }
        let member_flags = |id, is_enum| {
            flags(
                semantic
                    .declarations(id)
                    .filter(|declaration| {
                        declaration.flags.contains(SymbolFlags::ENUM_MEMBER) == is_enum
                    })
                    .fold(SymbolFlags::empty(), |flags, declaration| {
                        flags | declaration.flags
                    }),
            )
        };
        Member {
            symbol: namespace.or(enum_member).map(Idx::index),
            enum_flags: enum_member.map(|id| member_flags(id, true)),
            namespace_flags: namespace.map(|id| member_flags(id, false)),
            value_symbols: semantic
                .ts_namespace_value_targets(owner, name.as_bytes())
                .map(Into::into)
                .collect(),
        }
    }

    fn symbol(&self, SymbolEntry { id, symbol }: SymbolEntry) -> Symbol {
        let semantic = self.semantic;
        let name = self.ast.get_wtf8(symbol.name).as_bytes();
        assert_eq!(
            semantic.scope_of_symbol(id),
            semantic.declaration(symbol.declaration).scope
        );
        let mut roles = Vec::new();
        if semantic.ts_source_module_symbol() == Some(id) {
            roles.push("SourceModule");
        }
        if semantic.ts_default_export_symbol() == Some(id) {
            roles.push("DefaultExport");
        }
        let target = semantic.ts_import_alias_target(id);
        let initializers = self.nodes.initializers.get(&id);
        let import = (initializers.is_some()
            || target.is_some()
            || symbol.flags.contains(SymbolFlags::IMPORT_EQUALS))
        .then(|| Import {
            target: target.map(Idx::index),
            initializers: initializers
                .into_iter()
                .flatten()
                .map(|r| r.map(Idx::index))
                .collect(),
        });
        let mut linked_symbols: Vec<_> = semantic
            .linked_symbols(id)
            .filter(|other| *other != id)
            .map(Idx::index)
            .collect();
        linked_symbols.sort_unstable();
        Symbol {
            id: id.index(),
            name: String::from_utf8_lossy(name).into_owned(),
            flags: flags(symbol.flags),
            declarations: semantic
                .declarations(id)
                .map(|d| {
                    let node_symbol = semantic.symbol_of(d.node);
                    Declaration {
                        span: span(self.ast, d.node),
                        scope: d.scope.index(),
                        flags: (d.flags != symbol.flags).then(|| flags(d.flags)),
                        node_symbol: (node_symbol != Some(id)).then(|| node_symbol.map(Idx::index)),
                        owner: (d.symbol != id).then(|| d.symbol.index()),
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
                .map(|&reference| {
                    assert!(
                        semantic
                            .reference_candidates(reference)
                            .any(|candidate| candidate == id)
                    );
                    reference.index()
                })
                .collect(),
            roles,
            linked_symbols,
            members: self.members(id),
            import,
            export_aliases: semantic.ts_namespace_has_export_aliases(id),
        }
    }

    fn reference(&self, ReferenceEntry { id, reference }: ReferenceEntry) -> Reference {
        let semantic = self.semantic;
        let name = self.ast.get_utf8(reference.name);
        let resolution = reference.resolution();
        let flags = reference.flags;
        assert_eq!(flags.is_read_only(), flags.is_read() && !flags.is_write());
        assert_eq!(flags.is_write_only(), !flags.is_read() && flags.is_write());
        assert_eq!(flags.is_read_write(), flags.is_read() && flags.is_write());
        assert_eq!(semantic.reference_of(reference.node), Some(id));
        assert_eq!(
            semantic.lookup(reference.scope, name.as_bytes(), reference.flags.space),
            resolution.candidate()
        );
        let candidates = semantic
            .reference_candidates(id)
            .map(|candidate| {
                assert_eq!(
                    semantic
                        .uses(candidate)
                        .iter()
                        .filter(|&&r| r == id)
                        .count(),
                    1
                );
                candidate.index()
            })
            .collect();
        Reference {
            id: id.index(),
            name: name.to_owned(),
            span: span(self.ast, reference.node),
            scope: reference.scope.index(),
            space: format!("{:?}", reference.flags.space),
            access: match (reference.flags.is_read(), reference.flags.is_write()) {
                (false, false) => "None",
                (true, false) => "Read",
                (false, true) => "Write",
                (true, true) => "ReadWrite",
            },
            resolution: resolution.into(),
            candidates,
            value_symbols: semantic
                .reference_value_targets(self.ast, id)
                .map(Into::into)
                .collect(),
            import_equals: self.nodes.import_references.contains(&id),
        }
    }

    fn scope(&self, ScopeEntry { id, scope }: ScopeEntry) -> Scope {
        let mut flags = Vec::new();
        if scope.flags.strict {
            flags.push("Strict");
        }
        if scope.flags.arguments_barrier {
            flags.push("ArgumentsBarrier");
        }
        let mut bindings: Vec<_> = self.semantic.bindings(id).map(Idx::index).collect();
        bindings.sort_unstable();
        Scope {
            id: id.index(),
            span: span(self.ast, scope.node),
            kind: format!("{:?}", scope.kind),
            parent: scope.parent.map(Idx::index),
            hoist_target: scope.hoist_target.index(),
            bindings,
            function_body: scope.function_body.map(Idx::index),
            flags,
            container_owner: self
                .nodes
                .container_owners
                .get(&id)
                .copied()
                .map(Idx::index),
        }
    }
}

// Keep one record per line so adding a symbol or reference produces a local diff.
fn table<T: Serialize>(rows: impl Iterator<Item = T>) -> String {
    let rows: Vec<_> = rows
        .map(|row| serde_json::to_string(&row).unwrap())
        .collect();
    if rows.is_empty() {
        "[]".to_owned()
    } else {
        format!("[\n    {}\n  ]", rows.join(",\n    "))
    }
}

#[test]
fn semantic() {
    fixtures::run(|input, options| {
        analyze_fixture(input, options, |ast, semantic| {
            let snapshot = Snapshot {
                ast,
                semantic: &semantic,
                nodes: TestNodes::new(ast, &semantic),
            };
            let symbols = table(semantic.iter_symbols().map(|s| snapshot.symbol(s)));
            let references = table(semantic.iter_references().map(|r| snapshot.reference(r)));
            let scopes = table(semantic.iter_scopes().map(|s| snapshot.scope(s)));
            format!(
                "{{\n  \"symbols\": {symbols},\n  \"references\": {references},\n  \"scopes\": {scopes}\n}}"
            )
        })
    });
}
