use super::*;

#[test]
fn namespace_merging() {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();

    let source =
        "namespace N { export const first = 1; } namespace N { export const second = first; }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert_eq!(
        reference_symbol(&ast, &semantic, b"first"),
        Some(symbol_named(&ast, &semantic, b"first"))
    );

    let source = "namespace Root.Child { export const first = 1; } namespace Root.Child { export const second = first; }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert_eq!(
        reference_symbol(&ast, &semantic, b"first"),
        Some(symbol_named(&ast, &semantic, b"first"))
    );

    let source = "namespace A { export const x = 1 } namespace A.B { export const y = x } namespace A.B.C { export const z = x + y }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let x_references = reference_symbols(&ast, &semantic, b"x");
    let y_references = reference_symbols(&ast, &semantic, b"y");
    assert_eq!(x_references.len(), 2);
    assert_eq!(x_references, [Some(symbol_named(&ast, &semantic, b"x")); 2]);
    assert_eq!(y_references.len(), 1);
    assert_eq!(y_references, [Some(symbol_named(&ast, &semantic, b"y"))]);

    let source =
        "function F() {} namespace F { export const x = 1 } namespace F { export const y = x }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert_eq!(
        reference_symbol(&ast, &semantic, b"x"),
        Some(symbol_named(&ast, &semantic, b"x"))
    );

    let source = "class C {} namespace C { export const x = 1 } namespace C { export const y = x }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert_eq!(
        reference_symbol(&ast, &semantic, b"x"),
        Some(symbol_named(&ast, &semantic, b"x"))
    );

    let source =
        "enum E { A } namespace E { export const x = 1 } namespace E { export const y = x }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert_eq!(
        reference_symbol(&ast, &semantic, b"x"),
        Some(symbol_named(&ast, &semantic, b"x"))
    );
}

#[test]
fn namespace_instantiation() {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();

    let source = "namespace N { \"use strict\"; }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    assert_ts_module_state(&semantic, namespace, TsModuleState::Instantiated);

    let source = "namespace N { namespace M { \"use strict\"; } }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    assert_ts_module_state(&semantic, namespace, TsModuleState::Instantiated);

    let source = "namespace N { export const enum E { A } } const value = N.E.A";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    assert_ts_module_state(&semantic, namespace, TsModuleState::ConstEnumOnly);
    assert_eq!(reference_symbol(&ast, &semantic, b"N"), Some(namespace));

    let source = "namespace A { export const x = 1 } namespace N { export import y = A.x } const value = N.y";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    assert_ts_module_state(&semantic, namespace, TsModuleState::Instantiated);

    let source = "namespace N { export const enum E { A } } namespace N { export const value = 1 }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    assert_ts_module_state(&semantic, namespace, TsModuleState::Instantiated);

    let source = "namespace N { export { E }; const enum E { A } } const value = N.E.A";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    assert_ts_module_state(&semantic, namespace, TsModuleState::ConstEnumOnly);

    let source = "interface T {} namespace N { export { T } }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    assert_ts_module_state(&semantic, namespace, TsModuleState::NonInstantiated);

    let source = "declare namespace Outer { namespace A { const X: number; namespace B { export { Sibling } } const bad: typeof B } namespace Sibling { export { X } } interface X {} }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let outer = symbol_named(&ast, &semantic, b"Outer");
    let a = semantic.ts_namespace_member(outer, b"A").unwrap();
    let b = semantic.ts_namespace_member(a, b"B").unwrap();
    assert_ts_module_state(&semantic, b, TsModuleState::NonInstantiated);
    assert_eq!(reference_symbol(&ast, &semantic, b"B"), None);

    let source = "export default interface T {} namespace N { export { T } }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"N");
    assert_ts_module_state(&semantic, namespace, TsModuleState::NonInstantiated);
}

#[track_caller]
fn assert_namespace_type_export_does_not_shadow(source: &str) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions { check_syntax },
        );
        let outer = semantic.binding(ScopeId::MODULE, b"Inner").unwrap();
        let container = symbol_named(&ast, &semantic, b"Outer");
        let inner = semantic.ts_namespace_member(container, b"Inner").unwrap();
        assert!(!semantic.symbol(inner).flags.in_value_space());
        assert_eq!(reference_symbol(&ast, &semantic, b"Inner"), Some(outer));
        let reference = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"Inner")
            .unwrap()
            .reference;
        assert_eq!(
            semantic.lookup(reference.scope, b"Inner", ReferenceSpace::Value),
            Some(outer)
        );
    }
}

#[test]
fn namespace_instantiation_names() {
    assert_namespace_type_export_does_not_shadow(
        "export {}; const Inner = 1; namespace Outer { const { T } = { T: 0 }; interface T {} export declare namespace Inner { export { T }; } export const selected = Inner; }\n",
    );

    assert_namespace_type_export_does_not_shadow(
        "export {}; const Inner = 1; namespace Outer { const { fn }: { fn: <T>(x: T) => T } = { fn: x => x }; interface T {} export declare namespace Inner { export { T }; } export const selected = Inner; }\n",
    );

    assert_namespace_type_export_does_not_shadow(
        "export {}; const Inner = 1; namespace Outer { const { fn }: { fn: <U>(x: U) => U } = { fn: x => x }; interface T {} export declare namespace Inner { export { T }; } export const selected = Inner; }\n",
    );
}

#[test]
fn global_export_aliases() {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let source = "export as namespace Foo; export declare const Foo: number; export type Value = typeof Foo;";
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            dts_module_options(),
            AnalyzeOptions { check_syntax },
        );
        let local = semantic.binding(ScopeId::MODULE, b"Foo").unwrap();
        let mut globals = semantic
            .iter_symbols()
            .filter(|entry| entry.symbol.flags.contains(SymbolFlags::UMD_EXPORT));
        let global = globals.next().unwrap().id;
        assert!(globals.next().is_none());
        assert_ne!(global, local);
        let declaration = semantic.declaration(semantic.symbol(global).declaration);
        assert_eq!(semantic.symbol_of(declaration.node), Some(global));
        assert_eq!(
            ast.get_wtf8(semantic.symbol(global).name).as_bytes(),
            b"Foo"
        );
        assert_eq!(reference_symbol(&ast, &semantic, b"Foo"), Some(local));
        assert_eq!(
            semantic.lookup(ScopeId::MODULE, b"Foo", ReferenceSpace::Namespace),
            None
        );
        assert!(
            !semantic
                .bindings(ScopeId::MODULE)
                .any(|symbol| symbol == global)
        );

        let source = "export as namespace Global; export declare const value: number;";
        let (_, semantic) = analyze_with_options(
            &allocator,
            source,
            dts_module_options(),
            AnalyzeOptions { check_syntax },
        );
        assert_eq!(
            semantic
                .iter_symbols()
                .filter(|entry| entry.symbol.flags.contains(SymbolFlags::UMD_EXPORT))
                .count(),
            1
        );
        assert_eq!(semantic.binding(ScopeId::MODULE, b"Global"), None);
        assert_eq!(
            semantic.lookup(ScopeId::MODULE, b"Global", ReferenceSpace::Any),
            None
        );
    }
}

#[test]
fn ambient_namespace_merging() {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();
    let dts_module = dts_module_options();

    let source =
        "declare namespace N { const x: number } declare namespace N { const y: typeof x }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"x").is_some());

    let source = "namespace N { const x: number } namespace N { const y: typeof x }";
    let (ast, semantic) = analyze_source(&allocator, source, dts_module);
    assert!(reference_symbol(&ast, &semantic, b"x").is_some());

    let source = "declare namespace N { const hidden: number; export const shown: number } declare namespace N { const use: typeof hidden }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"hidden").is_some());
}

#[test]
fn namespace_lexical_visibility() {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();

    let source =
        "namespace N { export const x = 1 } namespace N { const x = 2; export const y = x }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let local_x = reference_symbol(&ast, &semantic, b"x").unwrap();
    let namespace = symbol_named(&ast, &semantic, b"N");
    let exported_x = semantic.ts_namespace_member(namespace, b"x").unwrap();
    assert_ne!(local_x, exported_x);
    assert!(
        !semantic
            .symbol(local_x)
            .flags
            .contains(SymbolFlags::EXPORTED)
    );

    let source = "namespace N { export interface X { exported: true } } namespace N { interface X { local: true } export const X = 1; type T = X }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let local_x = reference_symbol(&ast, &semantic, b"X").unwrap();
    let namespace = symbol_named(&ast, &semantic, b"N");
    let exported_x = semantic.ts_namespace_member(namespace, b"X").unwrap();
    let local_flags = semantic.symbol(local_x).flags;
    let exported_flags = semantic.symbol(exported_x).flags;
    assert_ne!(local_x, exported_x);
    assert!(!local_flags.contains(SymbolFlags::EXPORTED));
    assert!(exported_flags.in_type_space());
    assert!(exported_flags.in_value_space());
    let exported_declarations = semantic.declarations(exported_x).collect::<Vec<_>>();
    assert_eq!(exported_declarations.len(), 2);
    assert_ne!(
        exported_declarations[0].scope,
        exported_declarations[1].scope
    );
    for declaration in exported_declarations {
        // Public binding API contract: member views are not lexical bindings.
        assert_ne!(semantic.binding(declaration.scope, b"X"), Some(exported_x));
    }

    let source = "namespace N { export const X = 1; interface X { local: true } type T = X }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let local_x = reference_symbol(&ast, &semantic, b"X").unwrap();
    let namespace = symbol_named(&ast, &semantic, b"N");
    let exported_x = semantic.ts_namespace_member(namespace, b"X").unwrap();
    let local_flags = semantic.symbol(local_x).flags;
    let exported_flags = semantic.symbol(exported_x).flags;
    assert_ne!(local_x, exported_x);
    assert!(!local_flags.contains(SymbolFlags::EXPORTED));
    assert!(!exported_flags.in_type_space());
    assert!(exported_flags.in_value_space());
    let local_declaration = semantic.declarations(local_x).next().unwrap();
    assert_eq!(
        semantic.binding(local_declaration.scope, b"X"),
        Some(local_x)
    );
    let exported_declaration = semantic.declarations(exported_x).next().unwrap();
    assert_eq!(exported_declaration.scope, local_declaration.scope);
    assert!(
        !semantic
            .bindings(local_declaration.scope)
            .any(|symbol| symbol == exported_x)
    );

    let source = "namespace N { export interface X {} } namespace N { const X = 1; type T = X }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let x = reference_symbol(&ast, &semantic, b"X").unwrap();
    assert!(semantic.symbol(x).flags.in_type_space());

    let source =
        "namespace N { export class X {} } namespace N { export interface X {}; const X = 1 }";
    assert!(!has_syntax_diagnostics(&allocator, source, ts_module));

    let source =
        "namespace N { export class X {} } namespace N { const X = 1; export interface X {} }";
    assert!(!has_syntax_diagnostics(&allocator, source, ts_module));

    let source = "namespace N { const hidden = 1 } namespace N { export const visible = hidden }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"hidden").is_none());

    let source =
        "declare namespace N { interface Box<T> {} } declare namespace N { type Invalid = T }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"T").is_none());

    let source = "declare namespace N { namespace Target { interface Type {} } export import PublicAlias = Target; import PrivateAlias = Target; } declare namespace N { interface UsesPublic extends PublicAlias.Type {} type UsesPrivate = PrivateAlias }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"PublicAlias").is_some());
    assert!(reference_symbol(&ast, &semantic, b"PrivateAlias").is_none());

    let source = "namespace AliasSpace { interface OnlyType {} export type Public = OnlyType } namespace AliasSpace { export type Good = Public; export const bad = Public }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let public_references = semantic
        .iter_references()
        .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"Public")
        .collect::<Vec<_>>();
    assert_eq!(public_references.len(), 2);
    assert!(public_references.iter().any(|entry| {
        entry.reference.flags.space == ReferenceSpace::Type && entry.reference.symbol.is_some()
    }));
    assert!(public_references.iter().any(|entry| {
        entry.reference.flags.space == ReferenceSpace::Value && entry.reference.symbol.is_none()
    }));

    let source = "namespace A { export const x = 2 } namespace A.B {} namespace A { const x = 1; namespace B { export const y = x } }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let x = reference_symbol(&ast, &semantic, b"x").unwrap();
    assert!(!semantic.symbol(x).flags.contains(SymbolFlags::EXPORTED));

    let source = "namespace A { export const x = 2 } namespace A { const x = 1; namespace B { export const y = x } } namespace A.B {}";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let x = reference_symbol(&ast, &semantic, b"x").unwrap();
    assert!(!semantic.symbol(x).flags.contains(SymbolFlags::EXPORTED));
}

#[test]
fn ambient_external_modules_are_lexical_only() {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();

    let source = "declare module 'foo' { const x: number; const y: typeof x }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"x").is_some());

    let source = "declare module 'foo' { export const x: number } declare module 'foo' { export const y: typeof x }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"x").is_none());
    assert!(
        semantic
            .iter_symbols()
            .all(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() != b"foo")
    );

    let source = "declare module 'foo' { export as namespace Umd }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(
        semantic
            .iter_symbols()
            .all(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() != b"Umd")
    );
}

#[test]
fn augmentation_contexts() {
    let allocator = Allocator::new();
    let ts_commonjs = Options {
        source_type: SourceType::CommonJs,
        ..ts_module_options()
    };
    let ts_script = Options {
        source_type: SourceType::Script,
        ..ts_module_options()
    };
    let dts_script = Options {
        source_type: SourceType::Script,
        ..dts_module_options()
    };

    let invalid = has_syntax_diagnostics(&allocator, "declare global {}", ts_commonjs);
    assert!(!invalid);

    let source = "declare module 'pkg' { global {} }";
    let invalid = has_syntax_diagnostics(&allocator, source, ts_commonjs);
    assert!(invalid);

    let invalid = has_syntax_diagnostics(&allocator, "global {}", ts_commonjs);
    assert!(invalid);

    let source = "declare module 'outer' { module 'inner' {} }";
    let invalid = has_syntax_diagnostics(&allocator, source, ts_commonjs);
    assert!(invalid);

    let invalid = has_syntax_diagnostics(&allocator, "module 'pkg' { global {} }", dts_script);
    assert!(!invalid);

    let invalid = has_syntax_diagnostics(&allocator, "module 'pkg' { global {} }", ts_script);
    assert!(invalid);
}

#[test]
fn global_augmentation_recovery() {
    let allocator = Allocator::new();
    let ts_unambiguous = Options {
        source_type: SourceType::Unambiguous,
        ..ts_module_options()
    };

    let source = "export const value = 0; global { global { let copy: typeof value; } }";
    let (ast, checked) = analyze_syntax_source(&allocator, source, ts_unambiguous);
    assert_eq!(ast.source_type(), SourceType::Module);
    assert_eq!(checked.diagnostics.len(), 3);
    let outer = source.find("global").unwrap() as u32;
    let inner = source.rfind("global").unwrap() as u32;
    let spans = checked
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.span.start)
        .collect::<Vec<_>>();
    assert_eq!(spans, [inner, inner, outer]);
    let unchecked = analyze(&ast, AnalyzeOptions::default());
    assert!(unchecked.diagnostics.is_empty());
    let value = symbol_named(&ast, &checked.semantic, b"value");
    assert_eq!(
        reference_symbol(&ast, &checked.semantic, b"value"),
        Some(value)
    );
    assert_eq!(
        reference_symbol(&ast, &unchecked.semantic, b"value"),
        Some(value)
    );
    let reference = checked.semantic.iter_references().next().unwrap().reference;
    assert_eq!(reference.flags.space, ReferenceSpace::Typeof);
    assert!(!reference.flags.is_read());
    assert!(!reference.flags.is_write());

    let source = "export {}; declare global { global {} }";
    let (ast, checked) = analyze_syntax_source(&allocator, source, ts_unambiguous);
    assert_eq!(ast.source_type(), SourceType::Module);
    assert_eq!(checked.diagnostics.len(), 1);
}

#[test]
fn global_augmentation_merging() {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();

    let source = "export {}; declare global { const x: number } declare global { const y: typeof x } const z: typeof x = y";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let x_references = reference_symbols(&ast, &semantic, b"x");
    assert_eq!(x_references.len(), 2);
    assert!(x_references.iter().all(Option::is_some));
    let x = x_references[0].unwrap();
    assert!(x_references.iter().all(|reference| *reference == Some(x)));
    assert_eq!(semantic.binding(ScopeId::ROOT, b"x"), Some(x));

    let y = reference_symbol(&ast, &semantic, b"y").unwrap();
    assert_eq!(semantic.binding(ScopeId::ROOT, b"y"), Some(y));
}

#[test]
fn enum_merging() {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();
    let syntax_check = AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
    };

    let source = "enum Outer { A = (() => { enum Inner { X }; return 0; })(), B = 1, C = B }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let outer = symbol_named(&ast, &semantic, b"Outer");
    let inner = symbol_named(&ast, &semantic, b"Inner");
    let b = semantic.ts_enum_member(outer, b"B").unwrap();
    assert_eq!(reference_symbol(&ast, &semantic, b"B"), Some(b));
    assert!(semantic.ts_enum_member(outer, b"C").is_some());
    assert!(semantic.ts_enum_member(inner, b"X").is_some());
    assert!(semantic.ts_enum_member(inner, b"B").is_none());
    assert!(semantic.ts_enum_member(inner, b"C").is_none());

    let source = "enum E { A = 1 } enum E { B = A }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"A").is_some());

    let source = "enum E { ['value'] = 1, Other = value }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"value").is_some());

    let source = "enum E { A = 1 } namespace E { export const B = A }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"A").is_none());

    let source = "namespace E { export const B = 1 } enum E { A = B }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    assert!(reference_symbol(&ast, &semantic, b"B").is_none());

    let source = "namespace E { export namespace A {} } enum E { A }";
    let _ = analyze_with_options(&allocator, source, ts_module, syntax_check);

    let source = "enum E { A } namespace E { export namespace A { export interface X {} } } namespace E { export type T = A.X }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"E");
    let namespace_a = semantic.ts_namespace_member(namespace, b"A").unwrap();
    assert_eq!(semantic.ts_enum_member(namespace, b"A"), Some(namespace_a));
    assert_eq!(reference_symbol(&ast, &semantic, b"A"), Some(namespace_a));

    let source = "const A = 0; type A = string; enum E { A } namespace E { export namespace A { export interface X {} } } namespace E { const value = A; type Type = A; type Qualified = A.X }";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let namespace = symbol_named(&ast, &semantic, b"E");
    let namespace_a = semantic.ts_namespace_member(namespace, b"A").unwrap();
    let references = reference_symbols(&ast, &semantic, b"A");
    assert_eq!(references.len(), 3);
    assert!(references[0].is_some());
    assert_eq!(references[0], references[1]);
    assert_ne!(references[0], Some(namespace_a));
    assert_eq!(references[2], Some(namespace_a));
}

#[test]
fn contextual_await() {
    let allocator = Allocator::new();
    let ts_module = Options {
        lang: Lang::Ts,
        ..Options::default()
    };
    let syntax = AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
    };

    let source = "enum E{await=1,A=await}";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts_module, syntax);
    assert!(reference_symbol(&ast, &semantic, b"await").is_some());

    let source = "namespace N{export let await=1,x=await}";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts_module, syntax);
    assert!(reference_symbol(&ast, &semantic, b"await").is_some());
}

#[test]
fn class_definition_super_calls() {
    let allocator = Allocator::new();
    let options = Options::default();

    let source =
        "class B {} class C extends B { constructor() { class Inner { [(super(), 'x')] = 0; } } }";
    let diagnostics = has_syntax_diagnostics(&allocator, source, options);
    assert!(!diagnostics);

    let source = "class B {} class C extends B { constructor() { class Inner { x = super(); } } }";
    let diagnostics = has_syntax_diagnostics(&allocator, source, options);
    assert!(diagnostics);
}

#[test]
fn global_augmentation() {
    let allocator = Allocator::new();
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };
    let ts_script = Options {
        lang: Lang::Ts,
        source_type: SourceType::Script,
        ..Options::default()
    };

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_script);

        let has_errors = errors("declare module \"pkg\" { global {} }");
        assert!(!has_errors);

        let has_errors = errors("declare namespace N { global {} }");
        assert!(has_errors);

        let has_errors = errors("{ declare global {} }");
        assert!(has_errors);

        let has_errors = errors("function f() { declare global {} }");
        assert!(has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let has_errors = errors("declare global {}");
        assert!(!has_errors);

        let has_errors = errors("global {}");
        assert!(has_errors);

        let has_errors = errors("declare module \"pkg\" { global {} }");
        assert!(has_errors);

        let has_errors = errors("declare global { global {} }");
        assert!(has_errors);
    }
}

#[test]
fn namespace_context() {
    let allocator = Allocator::new();
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };
    let ts_script = Options {
        lang: Lang::Ts,
        source_type: SourceType::Script,
        ..Options::default()
    };

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_script);

        let has_errors = errors("declare module \"outer\" { module \"inner\" {} }");
        assert!(!has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let has_errors = errors("declare module \"outer\" { module \"inner\" {} }");
        assert!(has_errors);
    }
}

#[test]
fn using_declarations() {
    let allocator = Allocator::new();
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };
    let ts_script = Options {
        lang: Lang::Ts,
        source_type: SourceType::Script,
        ..Options::default()
    };

    for options in [ts_module, ts_script] {
        let errors = |source| has_syntax_diagnostics(&allocator, source, options);

        let has_errors = errors("declare namespace N { using x: null; }");
        assert!(has_errors);

        let has_errors = errors("namespace N { using x = null; }");
        assert!(!has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_script);

        let has_errors = errors("namespace N.M { using x = null; }");
        assert!(!has_errors);

        let has_errors = errors("namespace N { { using x = null; } }");
        assert!(!has_errors);

        let has_errors = errors("namespace N { export namespace M { using x = null; } }");
        assert!(!has_errors);

        let has_errors = errors("using x = null;");
        assert!(has_errors);
    }
}
