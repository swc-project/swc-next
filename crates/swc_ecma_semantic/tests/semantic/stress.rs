use std::fmt::Write;

use super::support::{analysis::with_semantic, options::TestOptions};
use swc_next_ecma_ast::{Ast, Lang, ScopeId, SourceType, SymbolId};
use swc_next_ecma_semantic::{NameResolution, ReferenceSpace, Semantic, SymbolFlags, SyntaxChecks};

fn check(source: &str, inspect: impl Fn(&Ast<'_>, &Semantic<'_>)) {
    let module = TestOptions::new(Lang::Ts, SourceType::Module);
    for options in [module, module.syntax(SyntaxChecks::empty())] {
        with_semantic(source, options, |ast, semantic| inspect(ast, &semantic));
    }
}

fn symbol(ast: &Ast<'_>, semantic: &Semantic<'_>, name: &str) -> SymbolId {
    let mut matches = semantic
        .iter_symbols()
        .filter(|s| ast.get_wtf8(s.symbol.name).as_bytes() == name.as_bytes());
    let id = matches.next().unwrap().id;
    assert!(matches.next().is_none());
    id
}

fn assert_alias_chain(source: &str, name: &str) {
    check(source, |ast, semantic| {
        let alias = semantic.binding(ScopeId::MODULE, name.as_bytes()).unwrap();
        let terminal = symbol(ast, semantic, "A20000");
        assert_eq!(semantic.ts_import_alias_target(alias), Some(terminal));
        let [id] = *semantic.uses(alias) else {
            panic!("expected one alias use")
        };
        let reference = semantic.reference(id);
        assert_eq!(reference.resolution(), NameResolution::Resolved(alias));
        assert_eq!(reference.flags.space, ReferenceSpace::Namespace);
        assert!(!reference.flags.is_read());
        assert!(!reference.flags.is_write());
        assert_eq!(
            semantic.lookup(reference.scope, name.as_bytes(), reference.flags.space),
            Some(alias)
        );
        assert!(semantic.reference_candidates(id).eq([alias]));
        assert!(semantic.reference_value_targets(ast, id).next().is_none());
    });
}

#[test]
fn long_import_alias_chains() {
    let mut source = String::new();
    for i in 0..20_000 {
        writeln!(source, "import A{i} = A{};", i + 1).unwrap();
    }
    source.push_str("namespace A20000 { export interface T {} }\ntype Result = A0.T;");
    assert_alias_chain(&source, "A0");

    let mut source = String::from("namespace N {\n");
    for i in 0..20_000 {
        writeln!(source, "export import A{i} = N.A{};", i + 1).unwrap();
    }
    source.push_str("export namespace A20000 { export interface T {} } }\nimport Root = N.A0;\ntype Result = Root.T;");
    assert_alias_chain(&source, "Root");
}

// Check every occurrence through the re-export chain, including its reverse use index.
fn uses(
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    name: &str,
    target: SymbolId,
    value: Option<SymbolId>,
) -> Vec<(ReferenceSpace, bool)> {
    let mut access = Vec::new();
    for entry in semantic
        .iter_references()
        .filter(|r| ast.get_utf8(r.reference.name) == name)
    {
        let reference = entry.reference;
        assert_eq!(reference.resolution(), NameResolution::Resolved(target));
        assert_eq!(
            semantic.lookup(reference.scope, name.as_bytes(), reference.flags.space),
            Some(target)
        );
        assert!(semantic.reference_candidates(entry.id).eq([target]));
        assert!(
            semantic
                .reference_value_targets(ast, entry.id)
                .eq(value.map(NameResolution::Resolved))
        );
        assert!(semantic.uses(target).contains(&entry.id));
        assert!(!reference.flags.is_write());
        access.push((reference.flags.space, reference.flags.is_read()));
    }
    access
}

fn assert_namespace_scopes(semantic: &Semantic<'_>, depth: usize) {
    let scopes: Vec<_> = semantic
        .iter_scopes()
        .filter(|entry| entry.scope.kind == swc_next_ecma_semantic::scope::ScopeKind::TsModule)
        .collect();
    // The source opens the namespace chain, then reopens each outer namespace
    // in reverse order to forward X. The innermost namespace exports Y.
    assert_eq!(scopes.len(), depth * 2 + 1);
    let (opened, reopened) = scopes.split_at(depth + 1);
    let mut parent = ScopeId::MODULE;
    for entry in opened {
        assert_eq!(entry.scope.parent, Some(parent));
        parent = entry.id;
    }
    for (again, first) in reopened.iter().zip(opened[..depth].iter().rev()) {
        assert_eq!(again.scope.parent, first.scope.parent);
    }
    for entry in scopes {
        assert!(entry.scope.flags.strict);
        assert!(!entry.scope.flags.arguments_barrier);
        assert_eq!(entry.scope.hoist_target, entry.id);
        assert_eq!(entry.scope.function_body, None);
    }
}

#[test]
fn deeply_nested_namespace_exports() {
    use ReferenceSpace::{Any, Namespace, Type, Typeof, Value};

    let mut nested = String::from("export declare namespace Inner { export { X as Y }; }");
    let mut path = String::from("N");
    for i in (1..64).rev() {
        nested = format!(
            "export namespace N{i} {{ {nested} }}\nexport declare namespace N{i} {{ export {{ X }}; }}"
        );
    }
    for i in 1..64 {
        write!(path, ".N{i}").unwrap();
    }
    let source = format!(
        "export {{}}; namespace N0 {{ {nested} }}\ndeclare namespace N0 {{ interface T {{ right: true }} export {{ T as X }}; }}"
    );
    check(&source, |ast, semantic| {
        assert_namespace_scopes(semantic, 64);
        let ty = symbol(ast, semantic, "T");
        assert_eq!(uses(ast, semantic, "T", ty, None), [(Any, false)]);
        assert_eq!(uses(ast, semantic, "X", ty, None), [(Any, false); 64]);
        assert_eq!(semantic.uses(ty).len(), 65);
    });

    let source = format!(
        "export {{}}; namespace N {{ {nested} }}\ndeclare namespace N {{ export interface X {{ right: true }} const original: 1; export {{ original as X }}; }}\nimport Alias = {path}.Inner.Y; const value: 1 = Alias; const typed: Alias = {{ right: true }}; const queried: typeof Alias = 1;"
    );
    check(&source, |ast, semantic| {
        assert_namespace_scopes(semantic, 64);
        let root = symbol(ast, semantic, "N");
        let ty = symbol(ast, semantic, "X");
        let original = symbol(ast, semantic, "original");
        let alias = symbol(ast, semantic, "Alias");
        assert_eq!(
            semantic.symbol(ty).flags,
            SymbolFlags::INTERFACE | SymbolFlags::AMBIENT | SymbolFlags::EXPORTED
        );
        assert_eq!(
            semantic.symbol(ty).flags,
            semantic.declaration(semantic.symbol(ty).declaration).flags
        );
        assert!(semantic.ts_namespace_has_export_aliases(root));
        assert_eq!(semantic.ts_namespace_member(root, b"X"), Some(ty));
        assert!(
            semantic
                .ts_namespace_value_targets(root, b"X")
                .eq([NameResolution::Resolved(original)])
        );
        assert_eq!(semantic.ts_namespace_member(root, b"original"), None);
        assert!(
            semantic
                .ts_namespace_value_targets(root, b"original")
                .next()
                .is_none()
        );
        assert_eq!(semantic.ts_import_alias_target(alias), None);
        assert_eq!(
            uses(ast, semantic, "N", root, Some(root)),
            [(Namespace, true)]
        );
        assert_eq!(
            uses(ast, semantic, "Alias", alias, Some(alias)),
            [(Value, true), (Type, false), (Typeof, false)]
        );
        assert_eq!(
            uses(ast, semantic, "X", ty, Some(original)),
            [(Any, false); 64]
        );
        assert_eq!(
            uses(ast, semantic, "original", original, Some(original)),
            [(Any, false)]
        );
    });
}
