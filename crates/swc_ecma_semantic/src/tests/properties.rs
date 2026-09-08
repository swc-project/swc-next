use super::*;

#[track_caller]
fn assert_static_alias_spaces(source: &str) {
    let allocator = Allocator::new();
    for check_syntax in [false, true] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
        let alias = semantic
            .iter_symbols()
            .find(|entry| {
                ast.get_wtf8(entry.symbol.name).as_bytes() == b"A"
                    && entry.symbol.flags.contains(SymbolFlags::IMPORT)
            })
            .unwrap()
            .id;
        assert_ne!(outer, alias);
        let mut seen = [false; 4];
        for entry in semantic.iter_references() {
            let reference = entry.reference;
            if ast.get_utf8(reference.name).as_bytes() != b"A" {
                continue;
            }
            let (index, expected) = match reference.flags.space {
                ReferenceSpace::Value => (0, alias),
                ReferenceSpace::Typeof => (1, alias),
                ReferenceSpace::Type => (2, outer),
                ReferenceSpace::Namespace => (3, outer),
                ReferenceSpace::Any => panic!("the fixture has no unqualified alias exports"),
            };
            seen[index] = true;
            assert_eq!(reference.symbol, Some(expected));
            assert_eq!(
                semantic.lookup(reference.scope, b"A", reference.flags.space),
                Some(expected)
            );
            assert_eq!(
                reference.flags.is_read(),
                reference.flags.space == ReferenceSpace::Value
            );
            assert!(!reference.flags.is_write());
        }
        assert!(seen.into_iter().all(|seen| seen));
        let targets = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"C")
            .collect::<Vec<_>>();
        assert!(!targets.is_empty());
        assert!(targets.iter().all(|entry| entry.reference.flags.is_read()));
    }
}

#[test]
fn property_name_parentheses() {
    let ts_module = ts_module_options();
    // Intentionally use unparenthesized semantics for class names, unlike
    // TS7's syntax-sensitive late-binding gate for computed methods.
    assert_named_declaration_targets(
        "class C { static ['x']() {} } namespace C { export interface x {} } const A = 0; namespace Use { import A = C.x; const result = A; }",
        ts_module,
        b"A",
        &[Some("A = C.x")],
    );

    assert_named_declaration_targets(
        "class C { static [(('x'))]() {} } namespace C { export interface x {} } const A = 0; namespace Use { import A = C.x; const result = A; }",
        ts_module,
        b"A",
        &[Some("A = C.x")],
    );

    assert_named_declaration_targets(
        "class C { static [('other')]() {} } namespace C { export interface x {} } const A = 0; namespace Use { import A = C.x; const result = A; }",
        ts_module,
        b"A",
        &[Some("A = 0")],
    );

    assert_named_declaration_targets(
        "function C() {} C[('other')] = 1; namespace C { export interface x {} } const A = 0; namespace Use { import A = C.x; const result = A; }",
        ts_module,
        b"A",
        &[Some("A = 0")],
    );

    assert_uncertain_alias_targets(
        "const key = 'other'; class C { static [(key)]() {} } namespace C { export interface x {} } const A = 0; namespace Use { import A = C.x; const result = A; }",
        &["A = C.x", "A = 0"],
    );
}

#[test]
fn property_value_uncertainty() {
    assert_static_alias_spaces(
        "export {}; C.x = 1; function C() {} namespace C { export interface T {} } type A = number; namespace A { export interface T {} } namespace N { import A = C.x; export const typed: A = 1; export const runtime = A; export type Queried = typeof A; export type Nested = A.T; }",
    );

    assert_static_alias_spaces(
        "export {}; const key = 'x'; class C { static [key] = 1; } namespace C { export interface T {} } type A = number; namespace A { export interface T {} } namespace N { import A = C.x; export const typed: A = 1; export const runtime = A; export type Queried = typeof A; export type Nested = A.T; }",
    );

    assert_static_alias_spaces(
        "export {}; type Key = 'x'; const key: Key = 'x'; class C { static [key] = 1; } namespace C { export interface T {} } type A = number; namespace A { export interface T {} } namespace N { import A = C.x; export const typed: A = 1; export const runtime = A; export type Queried = typeof A; export type Nested = A.T; }",
    );

    assert_static_alias_spaces(
        "export {}; function C() {} C.x = 1; namespace C { export interface T {} } type A = number; namespace A { export interface T {} } namespace N { import A = C.x; export const typed: A = 1; export const runtime = A; export type Queried = typeof A; export type Nested = A.T; }",
    );

    assert_static_alias_spaces(
        "export {}; function C() {} C['x'] = 1; namespace C { export interface T {} } type A = number; namespace A { export interface T {} } namespace N { import A = C.x; export const typed: A = 1; export const runtime = A; export type Queried = typeof A; export type Nested = A.T; }",
    );

    assert_static_alias_spaces(
        "export {}; const key = 'x'; function C() {} C[key] = 1; namespace C { export interface T {} } type A = number; namespace A { export interface T {} } namespace N { import A = C.x; export const typed: A = 1; export const runtime = A; export type Queried = typeof A; export type Nested = A.T; }",
    );
}

#[test]
fn static_alias_space_shadowing() {
    assert_static_alias_spaces(
        "export {}; class C {} namespace C { export interface T {} } type A = number; namespace A { export interface T {} } namespace N { import A = C.prototype; export const typed: A = 1; export const runtime = A; export type Queried = typeof A; export type Nested = A.T; }",
    );

    assert_static_alias_spaces(
        "export {}; class C { static [`x`] = 1; } namespace C { export interface T {} } type A = number; namespace A { export interface T {} } namespace N { import A = C.x; export const typed: A = 1; export const runtime = A; export type Queried = typeof A; export type Nested = A.T; }",
    );

    assert_static_alias_spaces(
        "export {};\ndeclare class C { static x(): number; }\nnamespace C { export interface T {} }\ntype A = number;\nnamespace A { export interface T {} }\nnamespace N { import A = C.x; export const typed: A = 1; export const runtime = A; export type Queried = typeof A; export type Nested = A.T; }",
    );

    assert_static_alias_spaces(
        "export {};\nclass C { static x = 1; }\nnamespace C { export interface T {} }\ntype A = number;\nnamespace A { export interface T {} }\nnamespace N {\n    import A = C.x;\n    export const typed: A = 1;\n    export const runtime = A;\n    export type Queried = typeof A;\n    export type Nested = A.T;\n}\n",
    );

    assert_static_alias_spaces(
        "export {};\nclass C { static 'x' = 1; }\nnamespace C { export interface T {} }\ntype A = number;\nnamespace A { export interface T {} }\nnamespace N {\n    import A = C.x;\n    export const typed: A = 1;\n    export const runtime = A;\n    export type Queried = typeof A;\n    export type Nested = A.T;\n}\n",
    );

    assert_static_alias_spaces(
        "export {};\nclass C { static ['x'] = 1; }\nnamespace C { export interface T {} }\ntype A = number;\nnamespace A { export interface T {} }\nnamespace N {\n    import A = C.x;\n    export const typed: A = 1;\n    export const runtime = A;\n    export type Queried = typeof A;\n    export type Nested = A.T;\n}\n",
    );

    assert_static_alias_spaces(
        "export {};\nclass C { static x() { return 1; } }\nnamespace C { export interface T {} }\ntype A = number;\nnamespace A { export interface T {} }\nnamespace N {\n    import A = C.x;\n    export const typed: A = 1;\n    export const runtime = A;\n    export type Queried = typeof A;\n    export type Nested = A.T;\n}\n",
    );

    assert_static_alias_spaces(
        "export {};\nclass C { static get x() { return 1; } }\nnamespace C { export interface T {} }\ntype A = number;\nnamespace A { export interface T {} }\nnamespace N {\n    import A = C.x;\n    export const typed: A = 1;\n    export const runtime = A;\n    export type Queried = typeof A;\n    export type Nested = A.T;\n}\n",
    );

    assert_static_alias_spaces(
        "export {};\nclass C { static x = 1; }\nnamespace C { export interface T {} }\nnamespace Holder { export import Value = C.x; }\ntype A = number;\nnamespace A { export interface T {} }\nnamespace N {\n    import A = Holder.Value;\n    export const typed: A = 1;\n    export const runtime = A;\n    export type Queried = typeof A;\n    export type Nested = A.T;\n}\n",
    );
}

#[test]
fn exported_static_alias_member_view() {
    assert_exported_static_alias_member_view(
        "export {};\nclass C { static x = 1; }\nnamespace C { export interface T {} }\nconst A = 2;\ntype A = number;\nnamespace A { export interface T {} }\nnamespace N { export import A = C.x; }\nnamespace N {\n    export const typed: A = 1;\n    export const runtime = A;\n    export type Queried = typeof A;\n    export type Nested = A.T;\n}\nimport Selected = N.A;\nconst selected = Selected;\n",
    );

    assert_exported_static_alias_member_view(
        "export {};\nclass C { static x = 1; }\nnamespace C { export interface T {} }\nconst A = 2;\ntype A = number;\nnamespace A { export interface T {} }\ndeclare namespace N { import local = C.x; export { local as A }; }\nnamespace N {\n    export const typed: A = 1;\n    export const runtime = A;\n    export type Queried = typeof A;\n    export type Nested = A.T;\n}\nimport Selected = N.A;\nconst selected = Selected;\n",
    );
}

#[track_caller]
fn assert_exported_static_alias_member_view(source: &str) {
    let allocator = Allocator::new();
    for check_syntax in [false, true] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
        let references = reference_symbols(&ast, &semantic, b"A");
        assert!(!references.is_empty());
        assert!(references.iter().all(|reference| *reference == Some(outer)));
        let selected = symbol_named(&ast, &semantic, b"Selected");
        assert_eq!(
            reference_symbol(&ast, &semantic, b"Selected"),
            Some(selected)
        );
    }
}

#[test]
fn static_alias_namespace_lookup() {
    let source = "export {};\nclass C { static x = 1; }\nnamespace C { export interface T {} }\nnamespace A { export interface T { outer: true } }\nnamespace N {\n    import A = C.x;\n    import B = A;\n    export const typed: B.T = { outer: true };\n    export const runtime = A;\n}\n";
    let allocator = Allocator::new();
    for check_syntax in [false, true] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
        let root = semantic
            .iter_references()
            .find(|entry| {
                ast.get_utf8(entry.reference.name).as_bytes() == b"A"
                    && entry.reference.flags.space == ReferenceSpace::Namespace
            })
            .unwrap()
            .reference;
        assert_eq!(root.symbol, Some(outer));
        assert!(!root.flags.is_read());
        let b = symbol_named(&ast, &semantic, b"B");
        assert_eq!(reference_symbol(&ast, &semantic, b"B"), Some(b));
    }
}

#[test]
fn static_values_preserve_type_contributions() {
    assert_static_values_preserve_type_contributions(
        "export {};\nclass C { static x = 1; }\nnamespace C { export interface x { own: true } }\nimport Alias = C.x;\nconst runtime = Alias;\nconst typed: Alias = { own: true };\nconst queried: typeof Alias = 1;\n",
    );

    assert_static_values_preserve_type_contributions(
        "export {};\nclass C { static x = 1; }\nnamespace C { export interface T {} }\nimport Alias = C.x;\ninterface Alias { own: true }\nconst runtime = Alias;\nconst typed: Alias = { own: true };\nconst queried: typeof Alias = 1;\n",
    );
}

#[track_caller]
fn assert_static_values_preserve_type_contributions(source: &str) {
    let allocator = Allocator::new();
    for check_syntax in [false, true] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let alias = symbol_named(&ast, &semantic, b"Alias");
        assert_eq!(
            reference_symbols(&ast, &semantic, b"Alias"),
            [Some(alias); 3]
        );
        let root = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"C")
            .unwrap()
            .reference;
        assert!(root.flags.is_read());
    }
}

#[test]
fn import_alias_value_shadowing() {
    let source = "class C {\n    static ['x']() { return 'x' }\n}\n\nnamespace C {\n    export interface x { }\n    export interface y { }\n}\n\nfunction A() {\n    return 1;\n}\n\nfunction B() {\n    return 2;\n}\n\nnamespace N {\n    import A = C.x;\n    import B = C.y;\n    console.log(A(), B());\n}";
    let module = ts_module_options();

    assert_named_declaration_targets(source, module, b"A", &[Some("A = C.x")]);
    assert_named_declaration_targets(source, module, b"B", &[Some("B() {\n")]);
}

#[track_caller]
fn assert_partial_property_alias(source: &str) {
    let allocator = Allocator::new();
    for check_syntax in [false, true] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let alias = symbol_named(&ast, &semantic, b"Alias");
        let references = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name) == "Alias")
            .map(|entry| entry.reference)
            .collect::<Vec<_>>();
        assert_eq!(references.len(), 3);
        for reference in references {
            assert_eq!(reference.symbol, Some(alias));
            assert_eq!(
                semantic.lookup(reference.scope, b"Alias", reference.flags.space),
                Some(alias)
            );
            assert_eq!(
                reference.flags.is_read(),
                reference.flags.space == ReferenceSpace::Value
            );
            assert!(!reference.flags.is_write());
        }
        let roots = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name) == "C")
            .collect::<Vec<_>>();
        assert!(!roots.is_empty());
        assert!(roots.iter().all(|entry| entry.reference.flags.is_read()));
    }
}

#[test]
fn numeric_property_contributions() {
    let module = ts_module_options();

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { class C { static 1e400 = 1; } namespace C { export interface Infinity {} } import Alias = C.Infinity; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = C.Infinity")],
    );

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { class C { static 1e400() {} } namespace C { export interface Infinity {} } import Alias = C.Infinity; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = C.Infinity")],
    );

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { declare class C { static 1e400(): void; } namespace C { export interface Infinity {} } import Alias = C.Infinity; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = C.Infinity")],
    );

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { class C { static [1e400] = 1; } namespace C { export interface Infinity {} } import Alias = C.Infinity; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = C.Infinity")],
    );

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { class C { static [123] = 1; } namespace C { export interface x {} } import Alias = C.x; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = 0")],
    );

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { function C() {} C[123] = 1; namespace C { export interface x {} } import Alias = C.x; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = 0")],
    );

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { class C { static [-1e400] = 1; } namespace C { export interface Infinity {} } import Alias = C.Infinity; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = 0")],
    );

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { class C { static [+1e400] = 1; } namespace C { export interface Infinity {} } import Alias = C.Infinity; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = 0")],
    );

    assert_partial_property_alias(
        "class C { static 1e400 = 1; } namespace C { export interface Infinity { own: true; } } import Alias = C.Infinity; const runtime = Alias; const typed: Alias = { own: true }; const queried: typeof Alias = 1;",
    );
}

#[test]
fn expando_ownership() {
    let module = ts_module_options();

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { const fn = () => {}; namespace fn { export interface x {} } fn.x = 1; import Alias = fn.x; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = 0")],
    );

    assert_named_declaration_targets(
        "const Alias = 0; namespace N { const fn = function() {}; namespace fn { export interface x {} } fn.x = 1; import Alias = fn.x; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = 0")],
    );
}

#[test]
fn exported_expandos() {
    let module = ts_module_options();

    assert_named_declaration_targets(
        "interface F {} export function F() {} export namespace F { export interface Marker {} } F.x = 1; type Alias = number; namespace Use { import Alias = F.x; const value = Alias; const typed: Alias = 0; }",
        module,
        b"Alias",
        &[Some("Alias = F.x"), Some("Alias = number")],
    );

    assert_named_declaration_targets(
        "export function F() {} type F = {}; export namespace F { export interface Marker {} } F.x = 1; type Alias = number; namespace Use { import Alias = F.x; const value = Alias; const typed: Alias = 0; }",
        module,
        b"Alias",
        &[Some("Alias = F.x"), Some("Alias = number")],
    );

    assert_named_declaration_targets(
        "namespace N { interface F {} export function F() {} export namespace F { export interface Marker {} } F.x = 1; type Alias = number; namespace Use { import Alias = F.x; const value = Alias; const typed: Alias = 0; } }",
        module,
        b"Alias",
        &[Some("Alias = F.x"), Some("Alias = number")],
    );

    assert_named_declaration_targets(
        "type F = {}; export function F() {} export namespace F { export interface Marker {} } F.x = 1; namespace N { export import Item = F.x; } import Alias = N.Item; const value = Alias;",
        module,
        b"Alias",
        &[Some("Alias = N.Item")],
    );
}

#[test]
fn qualified_expando_ownership() {
    let module = ts_module_options();

    assert_named_declaration_targets(
        "type Alias = number; function Root() {} namespace Root { export function child() {} export namespace child { export interface Marker {} } } Root.child.value = 1; namespace Use { import Alias = Root.child.value; export type T = Alias; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = number"), Some("Alias = Root.child.value")],
    );

    assert_named_declaration_targets(
        "type Alias = number; Root.child.leaf['value'] = 1; function Root() {} namespace Root { export function child() {} export namespace child { export function leaf() {} export namespace leaf { export interface Marker {} } } } namespace Use { import Alias = Root.child.leaf.value; export type T = Alias; export const result = Alias; }",
        module,
        b"Alias",
        &[
            Some("Alias = number"),
            Some("Alias = Root.child.leaf.value"),
        ],
    );

    assert_named_declaration_targets(
        "type Alias = number; function F() {} namespace F { export interface Marker {} } { { F.value = 1; } } namespace Use { import Alias = F.value; export type T = Alias; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = number"), Some("Alias = F.value")],
    );
}

#[track_caller]
fn assert_unbound_expando(source: &str) {
    let allocator = Allocator::new();
    for preserve_parens in [false, true] {
        for check_syntax in [false, true] {
            let options = Options {
                preserve_parens,
                ..ts_module_options()
            };
            let (ast, semantic) = analyze_with_options(
                &allocator,
                source,
                options,
                AnalyzeOptions {
                    check_syntax,
                    ..AnalyzeOptions::default()
                },
            );
            let reference = semantic
                .iter_references()
                .find(|entry| ast.get_utf8(entry.reference.name) == "Alias")
                .unwrap();
            assert!(matches!(
                reference.reference.resolution(),
                NameResolution::Uncertain { .. }
            ));
        }
    }
}

#[test]
fn expando_containers() {
    assert_unbound_expando(
        "function F() {} namespace F { export interface Marker {} } function write() { F.value = 1; } import Alias = F.value; type T = Alias;",
    );

    assert_unbound_expando(
        "function F() {} namespace F { export interface Marker {} } const write = () => { F.value = 1; }; import Alias = F.value; type T = Alias;",
    );

    assert_unbound_expando(
        "function F() {} namespace F { export interface Marker {} } namespace Write { F.value = 1; } import Alias = F.value; type T = Alias;",
    );

    assert_unbound_expando(
        "function F() {} namespace F { export interface Marker {} } const object = { value: F.value = 1 }; import Alias = F.value; type T = Alias;",
    );

    assert_unbound_expando(
        "function F() {} namespace F { export interface Marker {} } class C { value = F.value = 1; } import Alias = F.value; type T = Alias;",
    );

    assert_unbound_expando(
        "namespace Root { export function child() {} export namespace child { export interface Marker {} } } Root.child.value = 1; import Alias = Root.child.value; type T = Alias;",
    );

    assert_unbound_expando(
        "function Root() {} namespace Root { export function child() {} export namespace child { export interface Marker {} } } Root['child'].value = 1; import Alias = Root.child.value; type T = Alias;",
    );
}

#[test]
fn expando_parentheses() {
    let module = ts_module_options();

    assert_named_declaration_targets(
        "type Alias = number; function Root() {} namespace Root { export function child() {} export namespace child { export interface Marker {} } } (Root.child).value = 1; namespace Use { import Alias = Root.child.value; export type T = Alias; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = number"), Some("Alias = Root.child.value")],
    );

    assert_named_declaration_targets(
        "type Alias = number; function Root() {} namespace Root { export function child() {} export namespace child { export interface Marker {} } } ((Root).child.value) = 1; namespace Use { import Alias = Root.child.value; export type T = Alias; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = number"), Some("Alias = Root.child.value")],
    );

    assert_named_declaration_targets(
        "type Alias = number; function F() {} namespace F { export interface Marker {} } (F.value) = 1; namespace Use { import Alias = F.value; export type T = Alias; export const result = Alias; }",
        module,
        b"Alias",
        &[Some("Alias = number"), Some("Alias = F.value")],
    );
}

#[test]
fn partial_property_contributions() {
    assert_partial_property_alias(
        "export {}; type Key = 'x'; const key: Key = 'x'; class C { static [key] = 1; } namespace C { export interface x { own: true; } } import Alias = C.x; const runtime = Alias; const typed: Alias = { own: true }; const queried: typeof Alias = 1;",
    );

    assert_partial_property_alias(
        "export {}; const key = 'x'; class C { static [key] = 1; } namespace C { export interface x { own: true; } } import Alias = C.x; const runtime = Alias; const typed: Alias = { own: true }; const queried: typeof Alias = 1;",
    );

    assert_partial_property_alias(
        "export {}; const key = 'x'; class C { static [key] = 1; } namespace C { export interface x { own: true; } } namespace Holder { export import Item = C.x; } import Alias = Holder.Item; const runtime = Alias; const typed: Alias = { own: true }; const queried: typeof Alias = 1;",
    );
}

#[test]
fn property_exports_exclude_inherited_values() {
    let allocator = Allocator::new();
    for check_syntax in [false, true] {
        let source = "export {}; class Base { static x = 1; } class C extends Base {} namespace C { export interface x { own: true; } } import Alias = C.x; const typed: Alias = { own: true };";
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let alias = symbol_named(&ast, &semantic, b"Alias");
        assert_eq!(reference_symbol(&ast, &semantic, b"Alias"), Some(alias));
        let root = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name) == "C")
            .unwrap()
            .reference;
        assert!(!root.flags.is_read());
    }
}

#[test]
fn expando_receiver_identity() {
    let allocator = Allocator::new();
    for check_syntax in [false, true] {
        let source = "export {}; function C() {} namespace C { export interface x { own: true } } function write(C: { x: number }) { C.x = 1; } import Alias = C.x; const typed: Alias = { own: true };";
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let alias = symbol_named(&ast, &semantic, b"Alias");
        assert_eq!(reference_symbol(&ast, &semantic, b"Alias"), Some(alias));
        let root = semantic
            .iter_references()
            .find(|entry| {
                ast.get_utf8(entry.reference.name) == "C"
                    && entry.reference.flags.space == ReferenceSpace::Namespace
            })
            .unwrap()
            .reference;
        assert!(!root.flags.is_read());
        assert_eq!(root.symbol, semantic.binding(ScopeId::MODULE, b"C"));
    }
}
