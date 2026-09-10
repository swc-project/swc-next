use super::*;

#[test]
fn module_records_link_imports_and_exports() {
    let allocator = Allocator::new();
    let js_module = Options::default();
    let module_options = AnalyzeOptions {
        build_module_record: true,
        ..AnalyzeOptions::default()
    };

    let source = "export const value = 1;";
    let (_, semantic) = analyze_source(&allocator, source, js_module);
    assert!(semantic.module_record().exports().is_empty());

    let source = "export { value }; const value = 1;";
    let (ast, semantic) = analyze_with_options(&allocator, source, js_module, module_options);
    let body = ast.root_program().body(&ast);
    let statement = body.get_node(&ast, 0).unwrap();
    let declaration = statement.as_export_named_declaration(&ast).unwrap();
    let local_specifier = declaration.specifiers(&ast).get_node(&ast, 0).unwrap();
    let local_node = local_specifier.local(&ast).node_id();

    let local_reference = semantic.reference_of(local_node).unwrap();
    let local_reference_data = semantic.reference(local_reference);
    let local_symbol = local_reference_data.symbol.unwrap();
    assert_eq!(local_reference_data.flags.space, ReferenceSpace::Any);

    let local_export = semantic
        .module_record()
        .exports()
        .iter()
        .find(|entry| entry.node == local_specifier.node_id())
        .unwrap();
    assert_eq!(local_export.origin, ExportOrigin::LocalSpecifier);
    assert_eq!(
        local_export.target,
        ExportTarget::Local {
            symbol: local_symbol,
            reference: Some(local_reference),
        }
    );

    let source = "export { external as alias } from 'dep';";
    let (ast, semantic) = analyze_with_options(&allocator, source, js_module, module_options);
    let body = ast.root_program().body(&ast);
    let statement = body.get_node(&ast, 0).unwrap();
    let declaration = statement.as_export_named_declaration(&ast).unwrap();
    let re_export_specifier = declaration.specifiers(&ast).get_node(&ast, 0).unwrap();
    let re_export_node = re_export_specifier.local(&ast).node_id();
    assert_eq!(semantic.reference_of(re_export_node), None);

    let re_export = semantic
        .module_record()
        .exports()
        .iter()
        .find(|entry| entry.node == re_export_specifier.node_id())
        .unwrap();
    assert_eq!(re_export.origin, ExportOrigin::ReExportSpecifier);
    assert!(matches!(re_export.target, ExportTarget::External { .. }));
}

#[test]
fn default_exports() {
    let allocator = Allocator::new();
    let checked = AnalyzeOptions {
        check_syntax: true,
        build_module_record: true,
        ..AnalyzeOptions::default()
    };
    let ts = ts_module_options();
    let dts = dts_module_options();

    let source = "export default function f(value: string): string; type F = typeof f;";
    let (ast, semantic) = analyze_with_options(&allocator, source, dts, checked);
    let symbol = default_export_symbol(&semantic);
    assert_eq!(reference_symbol(&ast, &semantic, b"f"), Some(symbol));

    let source = "export default function f(value: string): string; export default function f(value: number): number; export default function f(value: string | number) { return value; }";
    let (_, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let symbol = default_export_symbol(&semantic);
    assert_eq!(semantic.declarations(symbol).len(), 3);
    assert_eq!(semantic.module_record().exports().len(), 3);
    assert!(
        semantic
            .module_record()
            .exports()
            .iter()
            .all(|entry| entry.target
                == ExportTarget::Local {
                    symbol,
                    reference: None
                })
    );

    let source = "export default function (value: string): string;";
    let (_, semantic) = analyze_with_options(&allocator, source, dts, checked);
    assert!(matches!(
        semantic.module_record().exports()[0].target,
        ExportTarget::AnonymousDeclaration { .. }
    ));
}

fn default_export_symbol(semantic: &Semantic<'_>) -> SymbolId {
    let ExportTarget::Local {
        symbol,
        reference: None,
    } = semantic.module_record().exports()[0].target
    else {
        panic!("expected a named default declaration")
    };
    symbol
}

fn import_alias_symbol(semantic: &Semantic<'_>) -> SymbolId {
    semantic
        .iter_symbols()
        .find(|entry| entry.symbol.flags.contains(SymbolFlags::IMPORT))
        .unwrap()
        .id
}

#[test]
fn default_binding_spaces() {
    let allocator = Allocator::new();
    let ts = ts_module_options();
    let checked = AnalyzeOptions {
        check_syntax: true,
        build_module_record: true,
        ..AnalyzeOptions::default()
    };

    let source = "export default function f() {} export default interface I { value: string } let item: f; const fn = f;";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let symbol = default_export_symbol(&semantic);
    assert_eq!(
        reference_symbols(&ast, &semantic, b"f"),
        [Some(symbol), Some(symbol)]
    );
    assert_eq!(semantic.declarations(symbol).len(), 2);
    assert_eq!(semantic.uses(symbol).len(), 2);
    assert!(
        semantic
            .declarations(symbol)
            .all(|declaration| semantic.symbol_of(declaration.node) == Some(symbol))
    );

    let source = "export default interface I { value: string } export default function f() {} let item: f; const fn = f;";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let symbol = default_export_symbol(&semantic);
    assert_eq!(
        reference_symbols(&ast, &semantic, b"f"),
        [Some(symbol), Some(symbol)]
    );

    let source = "export default function f() {} export default interface I { value: string } type f = number; const value: f = 1; const fn = f;";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let symbol = default_export_symbol(&semantic);
    let local = semantic.binding(ScopeId::MODULE, b"f").unwrap();
    assert_ne!(local, symbol);
    assert_eq!(
        reference_symbols(&ast, &semantic, b"f"),
        [Some(local), Some(symbol)]
    );

    let source = "export default interface I<T> { extra: T } export default class C<T> { value!: T } const I = 1; const value = I;";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let local = semantic.binding(ScopeId::MODULE, b"I").unwrap();
    assert_ne!(local, default_export_symbol(&semantic));
    assert_eq!(reference_symbol(&ast, &semantic, b"I"), Some(local));
    assert_merged_type_parameter(&ast, &semantic, b"T");
}

#[test]
fn default_namespace_aliases() {
    let allocator = Allocator::new();
    let ts = ts_module_options();
    let checked = AnalyzeOptions {
        check_syntax: true,
        build_module_record: true,
        ..AnalyzeOptions::default()
    };

    let source = "export default function f() {} namespace f { export interface Item {} } const A = 1; type A = string; namespace Test { import A = f; export const fn: typeof A = A; export let text: A; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let alias = import_alias_symbol(&semantic);
    let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    assert_eq!(
        reference_symbols(&ast, &semantic, b"A"),
        [Some(alias), Some(alias), Some(outer)]
    );

    let source = "export default class C {} namespace C { export interface Item {} } namespace A { export interface Item {} } namespace Test { import A = C; export let value: A; export let ctor: typeof A; export let item: A.Item; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let alias = import_alias_symbol(&semantic);
    let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    assert_eq!(
        reference_symbols(&ast, &semantic, b"A"),
        [Some(alias), Some(alias), Some(outer)]
    );

    let source = "export default interface I { value: string } namespace I { export interface Item {} } type A = number; const A = 1; namespace Test { import A = I; export let item: A.Item; export let value: A; export let scalar: typeof A; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let alias = import_alias_symbol(&semantic);
    let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    assert_eq!(
        reference_symbols(&ast, &semantic, b"A"),
        [Some(alias), Some(outer), Some(outer)]
    );

    let source = "export default function f() {} namespace f { export interface Item {} } namespace Test { import A = f; const A = 1; export const value = A; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let alias = import_alias_symbol(&semantic);
    assert_eq!(reference_symbol(&ast, &semantic, b"A"), Some(alias));
    assert_eq!(semantic.declarations(alias).len(), 2);
}

#[test]
fn source_exports() {
    let allocator = Allocator::new();
    let ts = ts_module_options();
    let dts = dts_module_options();
    let checked = AnalyzeOptions {
        check_syntax: true,
        build_module_record: true,
        ..AnalyzeOptions::default()
    };

    let source = "export default function f() {} export default interface I { value: string } export type f = number; let item: f;";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let default = default_export_symbol(&semantic);
    let module = semantic.symbol_of(ast.root_program().node_id()).unwrap();
    let named = semantic.ts_namespace_member(module, b"f").unwrap();
    assert_ne!(named, default);
    assert_eq!(
        semantic.ts_namespace_member(module, b"default"),
        Some(default)
    );
    assert_eq!(reference_symbol(&ast, &semantic, b"f"), Some(default));

    let source = "export default function f() {} export type f = number; namespace f { export interface Item {} } type A = string; const A = 1; namespace Test { import A = f; export let item: A; export const scalar: typeof A = A; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let alias = import_alias_symbol(&semantic);
    let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    assert_eq!(
        reference_symbols(&ast, &semantic, b"A"),
        [Some(alias), Some(outer), Some(outer)]
    );

    let source = "export default interface I { value: string } export const I = 1; namespace I { export interface Item {} } type A = string; const A = 1; namespace Test { import A = I; export let item: A; export const scalar: typeof A = A; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let alias = import_alias_symbol(&semantic);
    let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    assert_eq!(
        reference_symbols(&ast, &semantic, b"A"),
        [Some(outer), Some(alias), Some(alias)]
    );

    let source = "export default function f(): void; type f = number; declare namespace f { interface Item {} } type A = string; declare const A: 1; declare namespace Test { import A = f; const value: A; const scalar: typeof A; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, dts, checked);
    let alias = import_alias_symbol(&semantic);
    let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    assert_eq!(
        reference_symbols(&ast, &semantic, b"A"),
        [Some(alias), Some(outer)]
    );

    let source = "export {}; export default function f(): void; type f = number; declare namespace f { interface Item {} } type A = string; declare const A: 1; declare namespace Test { import A = f; const value: A; const scalar: typeof A; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, dts, checked);
    let alias = import_alias_symbol(&semantic);
    let outer = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    assert_eq!(
        reference_symbols(&ast, &semantic, b"A"),
        [Some(outer), Some(alias)]
    );
}

#[track_caller]
fn assert_forward_namespace_export(source: &str, target_name: &[u8]) {
    let allocator = Allocator::new();
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let target = symbol_named(&ast, &semantic, target_name);
        let reference = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"X")
            .unwrap()
            .reference;
        assert_eq!(reference.symbol, Some(target));
        assert_eq!(
            semantic.lookup(reference.scope, b"X", reference.flags.space),
            Some(target)
        );
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: true,
        ..AnalyzeOptions::default()
    });
}

#[test]
fn namespace_export_dependencies() {
    assert_forward_namespace_export(
        "export {};\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ndeclare namespace N { interface T { right: true } export { T as X }; }\ntype Result = N.Inner.Y;\nconst result: Result = { right: true };\n",
        b"T",
    );

    assert_forward_namespace_export(
        "export {};\ndeclare namespace N { interface T { right: true } export { T as X }; }\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ntype Result = N.Inner.Y;\nconst result: Result = { right: true };\n",
        b"T",
    );

    assert_forward_namespace_export(
        "export {};\ninterface X { wrong: true }\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ndeclare namespace N { interface T { right: true } export { T as X }; }\ntype Result = N.Inner.Y;\nconst result: Result = { right: true };\n",
        b"T",
    );

    assert_forward_namespace_export(
        "export {};\ninterface X { wrong: true }\ndeclare namespace N { interface T { right: true } export { T as X }; }\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ntype Result = N.Inner.Y;\nconst result: Result = { right: true };\n",
        b"T",
    );

    assert_forward_namespace_export(
        "export {};\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ndeclare namespace N { const original: 'right'; export { original as X }; }\ntype Result = typeof N.Inner.Y;\nconst result: Result = 'right';\n",
        b"original",
    );

    assert_forward_namespace_export(
        "export {};\ndeclare namespace N { const original: 'right'; export { original as X }; }\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ntype Result = typeof N.Inner.Y;\nconst result: Result = 'right';\n",
        b"original",
    );

    assert_forward_namespace_export(
        "export {};\nconst X = 'wrong';\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ndeclare namespace N { const original: 'right'; export { original as X }; }\ntype Result = typeof N.Inner.Y;\nconst result: Result = 'right';\n",
        b"original",
    );

    assert_forward_namespace_export(
        "export {};\nconst X = 'wrong';\ndeclare namespace N { const original: 'right'; export { original as X }; }\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ntype Result = typeof N.Inner.Y;\nconst result: Result = 'right';\n",
        b"original",
    );
    let source = nested_namespace_exports();

    let source = format!(
        "export {{}}; namespace N0 {{ {source} }}\ndeclare namespace N0 {{ interface T {{ right: true }} export {{ T as X }}; }}"
    );
    assert_forward_namespace_export(&source, b"T");
}

#[test]
fn public_alias_conflict_meanings() {
    assert_public_alias_conflict_meanings(
        "export {};\ndeclare namespace N {\n    export const X: 1;\n    interface X {}\n    export namespace Y {}\n    export { X as Y };\n    export namespace Inner {\n        export interface Z {}\n        export { Y as Z };\n    }\n}\n",
    );

    assert_public_alias_conflict_meanings(
        "export {};\ndeclare namespace N {\n    export const X: 1;\n    interface X {}\n    export namespace Y {}\n    export { X as Y };\n}\nimport Alias = N.Y;\ninterface Alias { own: true }\n",
    );

    assert_public_alias_conflict_meanings(
        "export {};\ndeclare namespace N {\n    export const X: 1;\n    interface X {}\n    export namespace Y {}\n    export { X as Y };\n    export namespace Inner {\n        export interface Z { own: true }\n        export { Y as Z };\n    }\n}\nimport Alias = N.Inner.Z;\nconst value: 1 = Alias;\nconst typed: Alias = { own: true };\n",
    );

    assert_public_alias_conflict_meanings(
        "export {};\ndeclare namespace N {\n    export const X: 1;\n    interface X {}\n    export { X as Y };\n}\nimport Alias = N.Y;\ndeclare const Alias: string;\n",
    );
}

#[track_caller]
fn assert_public_alias_conflict_meanings(source: &str) {
    let allocator = Allocator::new();
    for check_syntax in [false, true] {
        let _ = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
    }
}

#[test]
fn static_composed_exports() {
    let allocator = Allocator::new();
    let source = "export {}; class C { static x = 1; } declare namespace C { export namespace Y { interface T { t: true; } } interface Original { own: true; } export { Original as Y }; export { Y as x }; } import Alias = C.x; const value: number = Alias; const typed: Alias = { own: true }; const nested: Alias.T = { t: true };\n";
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
        for entry in semantic.iter_references() {
            let reference = entry.reference;
            if ast.get_utf8(reference.name).as_bytes() == b"Alias" {
                assert_eq!(
                    semantic.lookup(reference.scope, b"Alias", reference.flags.space),
                    Some(alias)
                );
            }
        }
        let root = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"C")
            .unwrap()
            .reference;
        assert!(root.flags.is_read());
    }
}

#[test]
fn static_values_do_not_become_namespace_locals() {
    let source = "export {};\nconst x = 2;\nclass C { static x = 1; }\nnamespace C { export const y = x; }\n";
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
        let x = semantic.binding(ScopeId::MODULE, b"x").unwrap();
        assert_eq!(reference_symbol(&ast, &semantic, b"x"), Some(x));
        let c = symbol_named(&ast, &semantic, b"C");
        assert_eq!(semantic.ts_namespace_member(c, b"x"), None);
    }
}

#[test]
fn composed_namespace_export_recovery() {
    assert_composed_namespace_export_recovery(
        "export {}; declare namespace N { export const X: 1; interface Original { right: true; } export type { Original as X }; } namespace N { export declare namespace Inner { export { X as Y }; } } import Alias = N.Inner.Y; const value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;",
        true,
    );

    assert_composed_namespace_export_recovery(
        "export {}; declare namespace N { export interface X { right: true; } const original: 1; export type { original as X }; } namespace N { export declare namespace Inner { export { X as Y }; } } import Alias = N.Inner.Y; const typed: Alias = { right: true }; const queried: typeof Alias = 1;",
        false,
    );
}

#[track_caller]
fn assert_composed_namespace_export_recovery(source: &str, runtime_read: bool) {
    let allocator = Allocator::new();
    for check_syntax in [false, true] {
        let parse =
            Parser::init(&allocator, source, ts_module_options(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty());
        let result = analyze(
            &parse.ast,
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        // These inputs can produce TS1379. Test recovery semantics,
        // without requiring either the presence or absence of diagnostics.
        let ast = &parse.ast;
        let semantic = &result.semantic;
        let alias = symbol_named(ast, semantic, b"Alias");
        let references = reference_symbols(ast, semantic, b"Alias");
        assert!(!references.is_empty());
        assert!(references.iter().all(|reference| *reference == Some(alias)));
        let root = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"N")
            .unwrap()
            .reference;
        assert_eq!(root.flags.is_read(), runtime_read);
        assert!(!root.flags.is_write());
    }
}

#[track_caller]
fn assert_composed_namespace_export(source: &str) {
    let allocator = Allocator::new();
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let alias = symbol_named(&ast, &semantic, b"Alias");
        let references = semantic
            .iter_references()
            .map(|entry| entry.reference)
            .filter(|reference| ast.get_utf8(reference.name).as_bytes() == b"Alias")
            .collect::<Vec<_>>();
        assert_eq!(references.len(), 3);
        for reference in references {
            assert_eq!(reference.symbol, Some(alias));
            assert_eq!(
                semantic.lookup(reference.scope, b"Alias", reference.flags.space),
                Some(alias)
            );
        }
        let root = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"N")
            .unwrap()
            .reference;
        assert!(root.flags.is_read());
        let direct = semantic.symbol(symbol_named(&ast, &semantic, b"X"));
        assert_eq!(direct.flags, semantic.declaration(direct.declaration).flags);
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: true,
        ..AnalyzeOptions::default()
    });
}

#[test]
fn composed_namespace_exports() {
    assert_composed_namespace_export(
        "export {};\ndeclare namespace N { export interface X { right: true; } const original: 1; export { original as X }; }\nimport Alias = N.X;\nconst value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;\n",
    );

    assert_composed_namespace_export(
        "export {};\ndeclare namespace N { export interface X { right: true; } const original: 1; export { original as X }; }\nnamespace N { export declare namespace Inner { export { X as Y }; } }\nimport Alias = N.Inner.Y;\nconst value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;\n",
    );

    assert_composed_namespace_export(
        "export {};\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ndeclare namespace N { export interface X { right: true; } const original: 1; export { original as X }; }\nimport Alias = N.Inner.Y;\nconst value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;\n",
    );

    assert_composed_namespace_export(
        "export {};\ndeclare namespace N { export const X: 1; interface Original { right: true; } export { Original as X }; }\nimport Alias = N.X;\nconst value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;\n",
    );

    assert_composed_namespace_export(
        "export {};\ndeclare namespace N { export const X: 1; interface Original { right: true; } export { Original as X }; }\nnamespace N { export declare namespace Inner { export { X as Y }; } }\nimport Alias = N.Inner.Y;\nconst value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;\n",
    );

    assert_composed_namespace_export(
        "export {};\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ndeclare namespace N { export const X: 1; interface Original { right: true; } export { Original as X }; }\nimport Alias = N.Inner.Y;\nconst value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;\n",
    );

    assert_composed_namespace_export(
        "export {};\ndeclare namespace N { export interface X { right: true; } namespace Original { const v: 1; interface T { ok: true; } } export { Original as X }; }\nimport Alias = N.X;\nconst value: 1 = Alias.v; const typed: Alias = { right: true }; const nested: Alias.T = { ok: true };\n",
    );

    assert_composed_namespace_export(
        "export {};\ndeclare namespace N { export interface X { right: true; } namespace Original { const v: 1; interface T { ok: true; } } export { Original as X }; }\nnamespace N { export declare namespace Inner { export { X as Y }; } }\nimport Alias = N.Inner.Y;\nconst value: 1 = Alias.v; const typed: Alias = { right: true }; const nested: Alias.T = { ok: true };\n",
    );

    assert_composed_namespace_export(
        "export {};\nnamespace N { export declare namespace Inner { export { X as Y }; } }\ndeclare namespace N { export interface X { right: true; } namespace Original { const v: 1; interface T { ok: true; } } export { Original as X }; }\nimport Alias = N.Inner.Y;\nconst value: 1 = Alias.v; const typed: Alias = { right: true }; const nested: Alias.T = { ok: true };\n",
    );

    assert_composed_namespace_export(
        "export {}; declare namespace N { export interface X { right: true; } const original: 1; export { original as X }; } declare namespace N { namespace Inner { export { X as Y }; } } import Alias = N.Inner.Y; const value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;",
    );

    assert_composed_namespace_export(
        "export {}; declare namespace N { export interface X { right: true; } const original: 1; export { original as X }; } namespace N { export declare namespace Inner { export namespace Y { interface T { ok: true } } export { X as Y }; } } import Alias = N.Inner.Y; const value: 1 = Alias; const typed: Alias = { right: true }; const nested: Alias.T = { ok: true };",
    );

    assert_composed_namespace_export(
        "export {}; declare namespace Values { const v: 1; } declare namespace N { export interface X { right: true; } import original = Values.v; export { original as X }; } namespace N { export declare namespace Inner { export { X as Y }; } } import Alias = N.Inner.Y; const value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;",
    );

    assert_composed_namespace_export(
        "export {}; declare namespace N { export interface X { right: true; } const original: 1; export { original as X }; export namespace Inner { export { X as Y }; } } import Alias = N.Inner.Y; const value: 1 = Alias; const typed: Alias = { right: true }; const queried: typeof Alias = 1;",
    );

    assert_composed_namespace_export(
        "export {}; declare namespace Values { namespace Original { const v: 1; interface T { ok: true; } } } declare namespace N { export interface X { right: true; } import original = Values.Original; export { original as X }; } namespace N { export declare namespace Inner { export { X as Y }; } } import Alias = N.Inner.Y; const value: 1 = Alias.v; const typed: Alias = { right: true }; const nested: Alias.T = { ok: true };",
    );

    assert_namespace_export_reference(
        "export {}; declare namespace N { export interface X { right: true; } const original: 1; export { original as X }; } namespace N { export declare namespace Inner { export { X as Y }; } } namespace N.Inner { export const value: 1 = Y; export const typed: Y = { right: true }; export const queried: typeof Y = 1; }",
        b"Y",
        b"original",
        ReferenceSpace::Value,
        true,
    );

    assert_namespace_export_reference(
        "export {}; declare namespace N { export interface X { right: true; } const original: 1; export { original as X }; } namespace N { export declare namespace Inner { export { X as Y }; } } namespace N.Inner { export const value: 1 = Y; export const typed: Y = { right: true }; export const queried: typeof Y = 1; }",
        b"Y",
        b"X",
        ReferenceSpace::Type,
        false,
    );

    assert_namespace_export_reference(
        "export {}; declare namespace N { export interface X { right: true; } const original: 1; export { original as X }; } namespace N { export declare namespace Inner { export { X as Y }; } } namespace N.Inner { export const value: 1 = Y; export const typed: Y = { right: true }; export const queried: typeof Y = 1; }",
        b"Y",
        b"original",
        ReferenceSpace::Typeof,
        false,
    );

    let source = nested_namespace_exports();
    let mut target = String::from("N");
    for index in 1..64 {
        target.push_str(&format!(".N{index}"));
    }
    let source = format!(
        "export {{}}; namespace N {{ {source} }}\ndeclare namespace N {{ export interface X {{ right: true; }} const original: 1; export {{ original as X }}; }}\nimport Alias = {target}.Inner.Y; const value: 1 = Alias; const typed: Alias = {{ right: true }}; const queried: typeof Alias = 1;"
    );
    assert_composed_namespace_export(&source);
}

#[track_caller]
fn assert_namespace_export_reference(
    source: &str,
    name: &[u8],
    target_name: &[u8],
    space: ReferenceSpace,
    read: bool,
) {
    let allocator = Allocator::new();
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let target = symbol_named(&ast, &semantic, target_name);
        let reference = semantic
            .iter_references()
            .find(|entry| {
                ast.get_utf8(entry.reference.name).as_bytes() == name
                    && entry.reference.flags.space == space
            })
            .unwrap()
            .reference;
        assert_eq!(reference.symbol, Some(target));
        assert_eq!(reference.flags.is_read(), read);
        assert!(!reference.flags.is_write());
        assert_eq!(semantic.lookup(reference.scope, name, space), Some(target));
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: true,
        ..AnalyzeOptions::default()
    });
}

#[test]
fn namespace_export_visibility() {
    assert_namespace_export_reference(
        "export {}; type Item = { outer: true }; declare namespace N { interface Local { inner: true } export { Local as Item }; } declare namespace N { const value: Item; }",
        b"Item",
        b"Item",
        ReferenceSpace::Type,
        false,
    );

    assert_namespace_export_reference(
        "export {}; const pick = 'outer'; declare namespace N { const original: 'inner'; export { original as pick }; } declare namespace N { const result: typeof pick; }",
        b"pick",
        b"pick",
        ReferenceSpace::Typeof,
        false,
    );

    assert_namespace_export_reference(
        "export {}; type Item = { outer: true }; declare namespace N { interface Local { inner: true } export { Local as Item }; } namespace N { export let value: Item; }",
        b"Item",
        b"Local",
        ReferenceSpace::Type,
        false,
    );

    assert_namespace_export_reference(
        "export {}; const pick = 'outer'; declare namespace N { const original: 'inner'; export { original as pick }; } namespace N { export const result = pick; }",
        b"pick",
        b"original",
        ReferenceSpace::Value,
        true,
    );

    assert_namespace_export_reference(
        "export {}; type Item = { outer: true }; declare namespace N { interface Local { inner: true } export { Local as Item }; export const Item: 0; } declare namespace N { const value: Item; }",
        b"Item",
        b"Local",
        ReferenceSpace::Type,
        false,
    );

    assert_namespace_export_reference(
        "export {}; const pick = 'outer'; declare namespace N { const original: 'inner'; export { original as pick }; export interface pick { tag: true } } declare namespace N { const result: typeof pick; }",
        b"pick",
        b"original",
        ReferenceSpace::Typeof,
        false,
    );

    assert_namespace_export_reference(
        "export {}; type Item = { outer: true }; declare namespace N { interface Local { inner: true } export { Local as Item }; export const Item: 0; } namespace N { export let value: Item; }",
        b"Item",
        b"Local",
        ReferenceSpace::Type,
        false,
    );

    assert_namespace_export_reference(
        "export {}; const pick = 'outer'; declare namespace N { const original: 'inner'; export { original as pick }; export interface pick { tag: true } } namespace N { export const result = pick; }",
        b"pick",
        b"original",
        ReferenceSpace::Value,
        true,
    );
    assert_namespace_export_reference(
        "export {}; namespace Item { export interface T { outer: true } } declare namespace N { namespace Local { export interface T { inner: true } } export { Local as Item }; export interface Item { direct: true } } declare namespace N { export import A = Item; const value: A.T; const direct: A; }",
        b"Item",
        b"Local",
        ReferenceSpace::Namespace,
        false,
    );

    assert_namespace_export_reference(
        "export {}; namespace Item { export interface T { outer: true } } declare namespace N { namespace Local { export interface T { inner: true } } export { Local as Item }; export interface Item { direct: true } } declare namespace N { export import A = Item; const value: A.T; const direct: A; }",
        b"A",
        b"A",
        ReferenceSpace::Type,
        false,
    );

    assert_namespace_export_reference(
        "export {}; namespace Item { export const value = 'outer'; } declare namespace N { namespace Local { const value: 'inner'; } export { Local as Item }; } namespace N { import A = Item; export const result = A.value; }",
        b"Item",
        b"Local",
        ReferenceSpace::Namespace,
        true,
    );

    assert_namespace_export_reference(
        "export {}; const pick = 'outer'; declare namespace N { const original: 'inner'; export { original as pick }; export namespace Inner {} } namespace N.Inner { export const result = pick; }",
        b"pick",
        b"original",
        ReferenceSpace::Value,
        true,
    );

    assert_namespace_export_reference(
        "export {}; declare namespace N { const original: 'inner'; export { original as pick }; } namespace N { const pick = 'local'; export const result = pick; }",
        b"pick",
        b"pick",
        ReferenceSpace::Value,
        true,
    );

    assert_namespace_export_reference(
        "export {}; declare namespace N { interface Local { inner: true } export { Local as Item }; export const Item: 0; } namespace N { export const result = Item; }",
        b"Item",
        b"Item",
        ReferenceSpace::Value,
        true,
    );

    assert_namespace_export_reference(
        "export {}; const pick = 'outer'; declare namespace N { namespace Types { interface T {} } import original = Types.T; export { original as pick }; } namespace N { export const result = pick; }",
        b"pick",
        b"pick",
        ReferenceSpace::Value,
        true,
    );

    assert_namespace_export_reference(
        "export {}; const pick = 'outer'; declare namespace N { const original: 'inner'; export { original as pick }; } namespace N { export const result = pick; } namespace Unrelated {} import Unused = Unrelated;",
        b"pick",
        b"original",
        ReferenceSpace::Value,
        true,
    );
}

#[test]
fn namespace_value_targets() {
    let allocator = Allocator::new();
    let options = ts_module_options();

    let source = "declare namespace N { const original: number; export { original as value }; export type { original as erased }; export interface mixed {} export { original as mixed }; }";
    let (ast, semantic) = analyze_source(&allocator, source, options);
    let owner = symbol_named(&ast, &semantic, b"N");
    let original = symbol_named(&ast, &semantic, b"original");
    assert_eq!(
        semantic
            .ts_namespace_value_targets(owner, b"value")
            .collect::<Vec<_>>(),
        [NameResolution::Resolved(original)]
    );
    assert_eq!(
        semantic
            .ts_namespace_value_targets(owner, b"mixed")
            .collect::<Vec<_>>(),
        [NameResolution::Resolved(original)]
    );
    assert!(
        semantic
            .ts_namespace_value_targets(owner, b"erased")
            .next()
            .is_none()
    );
    assert!(
        semantic
            .ts_namespace_value_targets(owner, b"original")
            .next()
            .is_none()
    );

    let source = "enum E { A=42 }";
    let (ast, semantic) = analyze_source(&allocator, source, options);
    let owner = symbol_named(&ast, &semantic, b"E");
    let member = semantic.ts_enum_member(owner, b"A").unwrap();
    assert_eq!(
        semantic
            .ts_namespace_value_targets(owner, b"A")
            .collect::<Vec<_>>(),
        [NameResolution::Resolved(member)]
    );

    let source = "import * as remote from 'pkg'; namespace N { export import A=remote.Unknown; }";
    let (ast, semantic) = analyze_source(&allocator, source, options);
    let owner = symbol_named(&ast, &semantic, b"N");
    let alias = semantic.ts_namespace_member(owner, b"A").unwrap();
    assert_eq!(
        semantic
            .ts_namespace_value_targets(owner, b"A")
            .collect::<Vec<_>>(),
        [NameResolution::Uncertain { candidate: alias }]
    );

    let source = "import * as remote from 'pkg'; declare namespace N { import original=remote.value; export { original as value }; export type { original as erased }; } namespace N { export const result=value; }";
    let (ast, semantic) = analyze_source(&allocator, source, options);
    let owner = symbol_named(&ast, &semantic, b"N");
    let alias = symbol_named(&ast, &semantic, b"original");
    let reference = semantic
        .iter_references()
        .find(|entry| ast.get_utf8(entry.reference.name) == "value")
        .unwrap();
    assert_eq!(
        reference.reference.resolution(),
        NameResolution::Uncertain { candidate: alias }
    );
    assert_eq!(
        semantic
            .ts_namespace_value_targets(owner, b"value")
            .collect::<Vec<_>>(),
        [NameResolution::Uncertain { candidate: alias }]
    );
    assert!(
        semantic
            .ts_namespace_value_targets(owner, b"erased")
            .next()
            .is_none()
    );
}

#[test]
fn namespace_export_aliases() {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();
    let dts_module = dts_module_options();

    let source = "declare namespace N { interface X {} export { X as Y } } declare namespace N { type T = Y } import A = N.Y;";
    let (ast, semantic) = analyze_source(&allocator, source, dts_module);
    let import = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    let target = semantic.ts_import_alias_target(import).unwrap();
    assert_eq!(reference_symbols(&ast, &semantic, b"X"), [Some(target)]);
    assert_eq!(reference_symbols(&ast, &semantic, b"Y"), [None]);

    let source = "interface X {} declare namespace N { export { X } }";
    let ts_script = Options {
        source_type: SourceType::Script,
        ..ts_module
    };
    assert!(has_syntax_diagnostics(&allocator, source, ts_script));

    let source = "declare namespace N { const a: number; export { a as x } } declare namespace N { const b: string; export { b as x } }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace N { export const x: number } declare namespace N { const y: string; export { y as x } }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace N { export const x: number } declare namespace N { interface Y {} export type { Y as x } }";
    assert!(!has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace N { export const X: number; interface X {} export { X as Y } } import A = N.Y;";
    let (ast, semantic) = analyze_source(&allocator, source, dts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    let exported = semantic.ts_namespace_member(namespace, b"X").unwrap();
    let import = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    let alias = semantic.ts_import_alias_target(import);
    assert_eq!(alias, Some(exported));
    assert!(semantic.symbol(exported).flags.in_value_space());
    assert!(!semantic.symbol(exported).flags.in_type_space());

    let source = "declare namespace N { export const X: number } declare namespace N { interface X {} export { X as Y } } import A = N.Y;";
    let (ast, semantic) = analyze_source(&allocator, source, dts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    let exported = semantic.ts_namespace_member(namespace, b"X").unwrap();
    let import = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    let target = semantic.ts_import_alias_target(import).unwrap();
    assert_ne!(target, exported);
    assert!(semantic.symbol(target).flags.in_type_space());
    assert!(!semantic.symbol(target).flags.in_value_space());

    let source = "declare namespace N { export interface X {} const X: number; export { X as Y } } import A = N.Y;";
    let (ast, semantic) = analyze_source(&allocator, source, dts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    let exported = semantic.ts_namespace_member(namespace, b"X").unwrap();
    let import = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    let target = semantic.ts_import_alias_target(import).unwrap();
    assert_ne!(target, exported);
    assert!(semantic.symbol(target).flags.in_value_space());
    assert!(!semantic.symbol(target).flags.in_type_space());

    let source = "declare namespace N { namespace A {} export import X = A; export { X } }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace N { namespace A { const value: number } import Y = A; export const X: number; export { Y as X } }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace N { export const X: number; interface X {} export interface Y {} export { X as Y } }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace N { export const X: number; interface X {} export const Y: string; export { X as Y } }";
    assert!(!has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace N { export const X: number; interface X {} export namespace Inner { export { X as Y } } } import A = N.Inner.Y;";
    let (ast, semantic) = analyze_source(&allocator, source, dts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    let exported = semantic.ts_namespace_member(namespace, b"X").unwrap();
    let import = semantic.binding(ScopeId::MODULE, b"A").unwrap();
    let alias = semantic.ts_import_alias_target(import);
    assert_eq!(alias, Some(exported));
}

#[test]
fn namespace_export_declaration_spaces() {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();
    let dts_module = dts_module_options();

    let source = "declare namespace A { interface T {} } declare namespace N { export import X = A.T; interface X {} }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace A { interface T {} } declare namespace N { interface X {} export import X = A.T }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace A { const T: number } declare namespace N { export import X = A.T; interface X {} }";
    assert!(!has_syntax_diagnostics(&allocator, source, dts_module));

    let source =
        "declare enum E { A } declare namespace E { interface X {} export type { X as A } }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace A { namespace T { interface U {} } export type { T as X } } declare namespace B { export import Y = A.X.U }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace N { interface T {} export type { T as x } } declare namespace M { export import y = N.x }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "declare namespace N { export interface x {} } declare namespace M { export import y = N.x; interface y {} }";
    assert!(has_syntax_diagnostics(&allocator, source, dts_module));

    let source = "namespace A {} namespace N { interface A {} import X = A; interface X {} }";
    assert!(!has_syntax_diagnostics(&allocator, source, ts_module));

    let source =
        "namespace A { export interface T {} } import B = A; import X = B.T; type X = number";
    assert!(has_syntax_diagnostics(&allocator, source, ts_module));

    let source = "namespace A { export interface T {} } namespace N { import X = A.T; export interface X {} }";
    assert!(has_syntax_diagnostics(&allocator, source, ts_module));

    let source = "namespace A { export interface T {} } namespace N { import X = A.T } namespace N { export interface X {} }";
    assert!(!has_syntax_diagnostics(&allocator, source, ts_module));

    let source = "declare namespace N { export const X: number; interface X {} export { X as Y } } import Alias = N.Y; declare const Alias: string";
    assert!(!has_syntax_diagnostics(&allocator, source, ts_module));

    let source = "declare namespace N { export const X: number; interface X {} export { X as Y } } import Alias = N.Y; interface Alias {}";
    assert!(has_syntax_diagnostics(&allocator, source, ts_module));

    let source = "namespace A { export interface T {} } namespace V { export const x = 1 } namespace N { export import A = V.x } namespace N { import B = A; const B = 0 }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    let member = semantic.ts_namespace_member(namespace, b"A").unwrap();
    let resolved = reference_symbol(&ast, &semantic, b"A").unwrap();
    assert_ne!(resolved, member);
    assert!(semantic.symbol(resolved).flags.in_namespace_space());

    let source = "type A = string; namespace V { export const x = 1 } namespace N { export import A = V.x } namespace N { export type Result = A }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    let member = semantic.ts_namespace_member(namespace, b"A").unwrap();
    let resolved = reference_symbol(&ast, &semantic, b"A").unwrap();
    assert_ne!(resolved, member);
    assert!(semantic.symbol(resolved).flags.in_type_space());
}

#[test]
fn global_augmentation_exports() {
    let source = "declare global { interface X {} } export { X as Y };";
    assert_global_augmentation_export(source, false);

    let source = "export type { X as Y }; declare global { interface X {} }";
    assert_global_augmentation_export(source, false);

    let source = "declare global { const X: number } export { X as Y };";
    assert_global_augmentation_export(source, true);

    let source = "declare global { class X {} } export { X as Y };";
    assert_global_augmentation_export(source, true);

    let source = "declare global { namespace X { interface T {} } } export { X as Y };";
    assert_global_augmentation_export(source, false);

    let source = "export {}; declare namespace N { export { X as Y } } declare global { interface X {} } import A = N.Y;";
    assert_global_augmentation_export(source, false);
}

#[track_caller]
fn assert_global_augmentation_export(source: &str, value: bool) {
    let allocator = Allocator::new();
    let check = |check_syntax, build_module_record| {
        let options = AnalyzeOptions {
            check_syntax,
            build_module_record,
            ..AnalyzeOptions::default()
        };
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let target = semantic.binding(ScopeId::ROOT, b"X").unwrap();
        assert_eq!(reference_symbols(&ast, &semantic, b"X"), [Some(target)]);
        assert_eq!(semantic.symbol(target).flags.in_value_space(), value);
        if semantic.binding(ScopeId::MODULE, b"N").is_some() {
            let import = semantic.binding(ScopeId::MODULE, b"A").unwrap();
            assert_eq!(semantic.ts_import_alias_target(import), Some(target));
        } else if build_module_record {
            let [export] = semantic.module_record().exports() else {
                panic!("expected one source export");
            };
            let ExportTarget::Local {
                symbol,
                reference: Some(reference),
            } = export.target
            else {
                panic!("expected a resolved export reference");
            };
            assert_eq!(symbol, target);
            assert!(semantic.uses(target).contains(&reference));
        }
    };
    check(false, false);
    check(true, false);
    check(false, true);
    check(true, true);
}

fn nested_namespace_exports() -> String {
    let mut source = String::from("export declare namespace Inner { export { X as Y }; }\n");
    for index in (1..64).rev() {
        source = format!(
            "export namespace N{index} {{ {source} }}\nexport declare namespace N{index} {{ export {{ X }}; }}\n"
        );
    }
    source
}
