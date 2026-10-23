use super::*;

#[test]
fn contextual_expression_recovery() {
    fn analyze_recovered_source<'a>(
        allocator: &'a Allocator,
        source: &'a str,
        options: Options,
    ) -> (Ast<'a>, Semantic<'a>) {
        let parsed = Parser::init(allocator, source, options, NoTokenParserConfig).parse();
        assert_eq!(parsed.diagnostics.len(), 1);
        let ast = parsed.ast;
        let result = analyze(&ast, AnalyzeOptions::default());
        assert!(result.diagnostics.is_empty());
        (ast, result.semantic)
    }

    let allocator = Allocator::new();
    let ts_module = ts_module_options();
    let js_script = Options {
        lang: Lang::Js,
        source_type: SourceType::Script,
        ..Options::default()
    };

    let source = "const value = 1; function read() { await value; } read();";
    let (ast, semantic) = analyze_recovered_source(&allocator, source, ts_module);
    assert_eq!(
        reference_symbol(&ast, &semantic, b"value"),
        Some(symbol_named(&ast, &semantic, b"value"))
    );

    let source = "const value = 1; function read() { yield value; } read();";
    let (ast, semantic) = analyze_recovered_source(&allocator, source, ts_module);
    assert_eq!(
        reference_symbol(&ast, &semantic, b"value"),
        Some(symbol_named(&ast, &semantic, b"value"))
    );

    let source = "function read() { await value; }";
    let (_, semantic) = analyze_recovered_source(&allocator, source, js_script);
    assert_eq!(semantic.iter_references().count(), 1);

    let source = "function read() { yield value; }";
    let (_, semantic) = analyze_recovered_source(&allocator, source, js_script);
    assert_eq!(semantic.iter_references().count(), 1);

    let source = "function read(await, yield) { await + yield; await(value); yield[value]; }";
    let valid = has_syntax_diagnostics(&allocator, source, js_script);
    assert!(!valid);

    let source = "function read(await, yield) { await\nvalue; yield\nvalue; }";
    let valid = has_syntax_diagnostics(&allocator, source, js_script);
    assert!(!valid);

    let source = "async function read() { await value; }";
    let valid = has_syntax_diagnostics(&allocator, source, ts_module);
    assert!(!valid);

    let valid = has_syntax_diagnostics(&allocator, "function* read() { yield value; }", ts_module);
    assert!(!valid);
}

#[test]
fn access_flags_follow_reference_syntax() {
    let allocator = Allocator::new();
    let js_script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };
    let ts_module = Options {
        lang: Lang::Ts,
        ..Options::default()
    };

    let source = "let value; value;";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let reference = semantic.iter_references().next().unwrap().reference;
    assert!(reference.flags.is_read_only());

    let source = "let value; value = 1;";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let reference = semantic.iter_references().next().unwrap().reference;
    assert!(reference.flags.is_write_only());

    let source = "let value; value++;";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let reference = semantic.iter_references().next().unwrap().reference;
    assert!(reference.flags.is_read_write());

    let source = "let value; delete value;";
    let (_, semantic) = analyze_source(&allocator, source, js_script);
    let reference = semantic.iter_references().next().unwrap().reference;
    assert!(!reference.flags.is_read());
    assert!(!reference.flags.is_write());

    let source = "import type { T } from 'mod'; type U = T;";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let symbol = semantic
        .iter_symbols()
        .find(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == b"T")
        .unwrap();
    assert!(symbol.symbol.flags.contains(SymbolFlags::TYPE_IMPORT));

    let reference = semantic
        .iter_references()
        .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"T")
        .unwrap()
        .reference;
    assert_eq!(reference.flags.space, ReferenceSpace::Type);
    assert_eq!(reference.symbol, Some(symbol.id));
    assert!(!reference.flags.is_read());

    let source = "import type { T } from 'mod'; T;";
    let (ast, semantic) = analyze_source(&allocator, source, ts_module);
    let reference = semantic
        .iter_references()
        .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == b"T")
        .unwrap()
        .reference;
    assert_eq!(reference.flags.space, ReferenceSpace::Value);
    assert!(reference.symbol.is_none());
    assert!(reference.flags.is_read_only());
}

fn reference_facts(
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    name: &str,
) -> Vec<(ReferenceSpace, bool, bool, Option<u32>)> {
    semantic
        .iter_references()
        .filter(|entry| ast.get_utf8(entry.reference.name) == name)
        .map(|entry| {
            let reference = entry.reference;
            let declaration = reference.symbol.map(|symbol| {
                let declaration = semantic.declaration(semantic.symbol(symbol).declaration);
                ast.span(declaration.node).start
            });
            (
                reference.flags.space,
                reference.flags.is_read(),
                reference.flags.is_write(),
                declaration,
            )
        })
        .collect()
}

macro_rules! assert_references {
    (
        $parser:expr, $check:expr,
        input: $source:expr,
        $($name:literal => [$(($flags:expr, $declaration:literal)),+ $(,)?]),+ $(,)?
    ) => {{
        let source = $source;
        let allocator = Allocator::new();
        let (ast, semantic) = analyze_with_options(&allocator, source, $parser, $check);
        $(
            let actual = reference_facts(&ast, &semantic, $name);
            let expected = [
                $({
                    let (space, read, write) = $flags;
                    let declaration = Some(source.find($declaration).unwrap() as u32);
                    (space, read, write, declaration)
                }),+
            ];
            assert_eq!(actual, expected);
        )+
    }};
}

#[test]
fn computed_parameter_keys() {
    let parser = ts_module_options();
    let check = AnalyzeOptions::default();
    let queried = (ReferenceSpace::Typeof, false, false);

    assert_references!(
        parser, check,
        input: "const key = 'p'; type F = ({ [key]: v }: { p: number }) => typeof v;",
        "key" => [(queried, "key =")],
        "v" => [(queried, "v }")],
    );

    assert_references!(
        parser, check,
        input: "interface key {} const key = 'p'; type F = ({ [key]: v }) => void;",
        "key" => [(queried, "key {}")],
    );

    assert_references!(
        parser, check,
        input: "interface key {} namespace N { const key = 'p'; type F = ({ [key]: v }: { p: number }) => typeof v; }",
        "key" => [(queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "const key = 'p'; type F = ([{ nested: { [key]: v } }]: [{ nested: { p: number } }]) => typeof v;",
        "key" => [(queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "namespace key { export const p = 'p'; } type F = ({ [key.p]: v }) => void;",
        "key" => [(queried, "key {")],
    );

    assert_references!(
        parser, check,
        input: "const key = 'p'; type F = new ({ [key]: v }: { p: number }) => object;",
        "key" => [(queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "const key = 'p'; interface I { ({ [key]: v }: { p: number }): typeof v; }",
        "key" => [(queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "const key = 'p'; interface I { new ({ [key]: v }: { p: number }): object; }",
        "key" => [(queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "const key = 'p'; interface I { method({ [key]: v }: { p: number }): typeof v; }",
        "key" => [(queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "const key = 'p'; declare function f({ [key]: v }: { p: number }): typeof v;",
        "key" => [(queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "const key = 'p'; function f({ [key]: v }: { p: number }): typeof v;",
        "key" => [(queried, "key =")],
    );

    let allocator = Allocator::new();
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let source = "type F = ({ [(() => { const yield = \"prop\"; return yield; })()]: value }: { prop: number }) => typeof value;";
        let has_errors = errors(source);
        assert!(has_errors);

        let has_errors = errors("type F = ({ [\"\\1\"]: value }: object) => typeof value;");
        assert!(has_errors);

        let has_errors = errors("type F = ({ [010]: value }: { 8: number }) => typeof value;");
        assert!(has_errors);

        let has_errors = errors("type F = ({ [yield]: value }: object) => typeof value;");
        assert!(has_errors);
    }
}

#[test]
fn computed_key_expressions() {
    let parser = ts_module_options();
    let check = AnalyzeOptions::default();
    let queried = (ReferenceSpace::Typeof, false, false);

    assert_references!(
        parser, check,
        input: "const key = 'p'; type F = ({ [key + key]: v }: object) => void;",
        "key" => [(queried, "key ="), (queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "const key = 'p'; type F = ({ [key[key]]: v }: object) => void;",
        "key" => [(queried, "key ="), (queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "const key = (v: string) => v; type F = ({ [key(key)]: v }: object) => typeof v;",
        "key" => [(queried, "key ="), (queried, "key =")],
    );

    assert_references!(
        parser, check,
        input: "const key = 'p'; type T = { [key + key]: number };",
        "key" => [(queried, "key ="), (queried, "key =")],
    );
}

#[test]
fn computed_key_contexts() {
    let module = ts_module_options();
    let script = Options {
        source_type: SourceType::Script,
        ..module
    };
    let unchecked = AnalyzeOptions::default();
    let checked = AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
    };
    let queried = (ReferenceSpace::Typeof, false, false);
    let typed = (ReferenceSpace::Type, false, false);
    let runtime = (ReferenceSpace::Value, true, false);

    let source = "interface T {} const k = 'p'; type F = ({ [(k as T) + k]: v }) => void;";
    assert_references!(
        module, unchecked,
        input: source,
        "k" => [(queried, "k ="), (queried, "k =")],
        "T" => [(typed, "T {}")],
    );

    assert_references!(
        module, checked,
        input: source,
        "k" => [(queried, "k ="), (queried, "k =")],
    );

    assert_references!(
        script, checked,
        input: source,
        "k" => [(queried, "k ="), (queried, "k =")],
    );

    assert_references!(
        module, checked,
        input: "const key = 'p'; function f({ [key]: v }: { p: number }) { return v; }",
        "key" => [(runtime, "key =")],
    );

    assert_references!(
        module, unchecked,
        input: "const key = 'p'; class C { declare [key]: number; v = key; }",
        "key" => [(queried, "key ="), (runtime, "key =")],
    );
}

#[test]
fn uncertain_reference_targets() {
    // The bounded property analysis cannot distinguish these keys. Both
    // candidate sets must include the target chosen by TS7's type checker.
    assert_uncertain_alias_targets(
        "const key = 'other'; class C { static [key] = 1; } namespace C { export interface x {} } const A = 0; namespace Use { import A = C.x; const result = A; }",
        &["A = C.x", "A = 0"],
    );

    assert_uncertain_alias_targets(
        "const key = 'x'; class C { static [key] = 1; } namespace C { export interface x {} } const A = 0; namespace Use { import A = C.x; const result = A; }",
        &["A = C.x", "A = 0"],
    );

    assert_uncertain_alias_targets(
        "const key = 'other'; class C { static [key] = 1; } namespace C { export interface x {} } const A = 0; namespace Holder { export import Item = C.x; } namespace Use { import A = Holder.Item; const result = A; }",
        &["A = Holder.Item", "A = 0"],
    );

    assert_uncertain_alias_targets(
        "const key = 'other'; class C { static [key] = 1; } namespace C { export interface x {} } const A = 0; namespace Outer { import A = C.x; namespace Inner { import A = C.x; const result = A; } }",
        &["A = C.x; const result", "A = C.x; namespace Inner", "A = 0"],
    );

    assert_uncertain_alias_targets(
        "const key = 'other'; class C { static [key] = 1; } namespace C { export interface x {} } const A = 0; namespace Outer { const A = 1; namespace Inner { import A = C.x; const result = A; } }",
        &["A = C.x", "A = 1"],
    );

    assert_uncertain_alias_targets(
        "const key = 'x'; class C { static [key] = 1; } namespace C { export interface x {} } namespace Use { import A = C.x; const result = A; }",
        &["A = C.x"],
    );

    assert_uncertain_alias_targets(
        "const key = 'other'; class C { static [key] = 1; } namespace C { export interface x {} } const A = 0; namespace Use { import A = C.x; type Result = typeof A; }",
        &["A = C.x", "A = 0"],
    );
}

#[test]
fn known_reference_targets() {
    let ts_module = ts_module_options();
    assert_named_declaration_targets(
        "const key = 'other'; class C { static x = 1; static [key] = 2; } namespace C { export interface x {} } const A = 0; namespace Use { import A = C.x; const result = A; }",
        ts_module,
        b"A",
        &[Some("A = C.x")],
    );

    assert_named_declaration_targets(
        "const key = 'other'; class C { static [key] = 1; } namespace C { export interface x {} } namespace Use { import A = C.x; const typed: A = {}; }",
        ts_module,
        b"A",
        &[Some("A = C.x")],
    );

    assert_named_declaration_targets(
        "const key = 'other'; class C { static [key] = 1; } namespace C { export interface x {} } namespace A { export interface Inner {} } namespace Use { import A = C.x; type Result = A.Inner; }",
        ts_module,
        b"A",
        &[Some("A { export interface Inner")],
    );
}

#[track_caller]
fn assert_const_assertion_references(source: &str, count: usize) {
    let allocator = Allocator::new();
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let references = semantic.iter_references().collect::<Vec<_>>();
        assert_eq!(references.len(), count);
        for entry in references {
            assert_eq!(ast.get_utf8(entry.reference.name), "value");
            assert!(entry.reference.flags.is_read_only());
            assert!(entry.reference.resolution().symbol().is_some());
        }
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
    });
}

#[test]
fn const_assertions() {
    assert_const_assertion_references("const result = 1 as const;", 0);
    assert_const_assertion_references("const result = <const>1;", 0);
    assert_const_assertion_references("const value = 1; const result = { value } as const;", 1);
    assert_const_assertion_references("const value = 1; const result = <const>{ value };", 1);
}

#[track_caller]
fn assert_export_value_reference(source: &str, value: Option<&str>) {
    assert_export_reference(source, value, value.is_some());
}

#[track_caller]
fn assert_export_reference(source: &str, value: Option<&str>, read: bool) {
    let allocator = Allocator::new();
    let ts_module = ts_module_options();
    let ts_module_without_parens = Options {
        preserve_parens: false,
        ..ts_module
    };

    for parser_options in [ts_module, ts_module_without_parens] {
        for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
            let options = AnalyzeOptions { check_syntax };
            let (ast, semantic) = analyze_with_options(&allocator, source, parser_options, options);
            let entry = semantic
                .iter_references()
                .find(|entry| ast.get_utf8(entry.reference.name) == "X")
                .unwrap();
            assert!(matches!(
                entry.reference.resolution(),
                NameResolution::Resolved(_)
            ));
            assert_eq!(entry.reference.flags.is_read(), read);
            assert!(!entry.reference.flags.is_write());
            let canonical = entry.reference.symbol.unwrap();
            assert_eq!(
                semantic.reference_candidates(entry.id).collect::<Vec<_>>(),
                [canonical],
            );
            assert!(semantic.uses(canonical).contains(&entry.id));
            let target = value
                .map(|declaration| symbol_at_declaration(&ast, &semantic, source, declaration));
            let expected = target
                .map(NameResolution::Resolved)
                .into_iter()
                .collect::<Vec<_>>();
            assert_eq!(
                semantic
                    .reference_value_targets(&ast, entry.id)
                    .collect::<Vec<_>>(),
                expected
            );

            if value.is_some() {
                let local_type = semantic
                    .lookup(entry.reference.scope, b"X", ReferenceSpace::Type)
                    .unwrap();
                assert_ne!(Some(local_type), target);
                assert_eq!(entry.reference.symbol, Some(local_type));
            }
        }
    }
}

#[test]
fn export_value_references() {
    assert_export_value_reference(
        "interface X {} export const X = 1; export default X;",
        Some("X = 1"),
    );

    assert_export_value_reference(
        "export const X = 1; interface X {} export default X;",
        Some("X = 1"),
    );

    assert_export_value_reference(
        "interface X {} export const X = 1; export { X as Y };",
        Some("X = 1"),
    );

    assert_export_value_reference(
        "export const X = 1; interface X {} export { X as Y };",
        Some("X = 1"),
    );

    assert_export_value_reference(
        "interface X {} export const X = 1; export default (X);",
        Some("X = 1"),
    );

    assert_export_value_reference(
        "export const X = 1; interface X {} export default (X);",
        Some("X = 1"),
    );

    assert_export_value_reference("interface X {} export default X;", None);

    assert_export_value_reference("interface X {} export default (X);", None);

    assert_export_value_reference(
        "const value = 1; export interface X {} export { value as X }; export default X;",
        Some("value = 1"),
    );

    assert_export_value_reference(
        "const value = 1; export { value as X }; export interface X {} export { X as Y };",
        Some("value = 1"),
    );

    assert_export_reference(
        "const value = 1; export interface X {} export { value as X }; export type { X as Y };",
        Some("value = 1"),
        false,
    );

    assert_export_value_reference(
        "const value = 1; export interface X {} export type { value as X }; export default X;",
        None,
    );

    assert_export_value_reference(
        "const X = 1; declare namespace N { interface X {} export { X }; }",
        None,
    );

    assert_export_reference(
        "declare namespace N { export interface X {} const value: 1; export { value as X }; export { X as Y }; }",
        Some("value: 1"),
        false,
    );

    assert_export_value_reference(
        "const X = 1; declare namespace N { export interface X {} const value: 1; export type { value as X }; export { X as Y }; }",
        None,
    );
}

#[track_caller]
fn assert_alias_value_reference(source: &str, value: bool, read: bool) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let options = AnalyzeOptions { check_syntax };
        let (ast, semantic) =
            analyze_with_options(&allocator, source, ts_module_options(), options);
        let alias = symbol_named(&ast, &semantic, b"Alias");
        let entry = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name) == "Alias")
            .unwrap();
        assert_eq!(
            entry.reference.resolution(),
            NameResolution::Resolved(alias)
        );
        assert_eq!(entry.reference.flags.is_read(), read);
        let expected = if value {
            vec![NameResolution::Resolved(alias)]
        } else {
            vec![]
        };
        assert_eq!(
            semantic
                .reference_value_targets(&ast, entry.id)
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn alias_value_references() {
    assert_alias_value_reference(
        "namespace N { export class C {} } namespace N { export import Value = N.C; } import Alias = N.Value; export { Alias };",
        true,
        true,
    );

    assert_alias_value_reference(
        "namespace N { export import Value = N.C; } namespace N { export class C {} } import Alias = N.Value; export { Alias };",
        true,
        true,
    );

    assert_alias_value_reference(
        "namespace N { export interface C {} } namespace N { export import Value = N.C; } import Alias = N.Value; export { Alias };",
        false,
        false,
    );

    assert_alias_value_reference(
        "namespace N { export class C {} } import Alias = N.C; export type { Alias };",
        true,
        false,
    );

    assert_alias_value_reference(
        "namespace N { export class C {} } import Alias = N.C; export type T = typeof Alias;",
        true,
        false,
    );
}

#[track_caller]
fn assert_jsx_value_roots(source: &str, names: &[&[u8]]) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            Options {
                lang: Lang::Tsx,
                ..Options::default()
            },
            AnalyzeOptions { check_syntax },
        );
        let references = semantic
            .iter_references()
            .filter(|entry| entry.reference.flags.space == ReferenceSpace::Value)
            .map(|entry| entry.reference)
            .collect::<Vec<_>>();
        assert_eq!(references.len(), names.len());
        for (reference, &name) in references.iter().zip(names) {
            assert_eq!(ast.get_utf8(reference.name).as_bytes(), name);
            let symbol = symbol_named(&ast, &semantic, name);
            assert_eq!(reference.symbol, Some(symbol));
            assert_eq!(
                semantic.lookup(reference.scope, name, ReferenceSpace::Value),
                Some(symbol)
            );
            assert!(reference.flags.is_read());
            assert!(!reference.flags.is_write());
        }
    }
}

#[test]
fn jsx_tag_references() {
    assert_jsx_value_roots(
        "export {}; declare namespace JSX { interface Element {} interface IntrinsicElements { div: {}; 'A-B': {}; } } declare const ui: { Button: () => JSX.Element }; const view = <ui.Button />;\n",
        &[b"ui"],
    );

    assert_jsx_value_roots(
        "declare namespace JSX { interface Element {} interface IntrinsicElements { div: {}; 'A-B': {}; } } declare const _Button: () => JSX.Element; declare const $Button: () => JSX.Element; declare const Élement: () => JSX.Element; const views = [<_Button />, <$Button />, <Élement />, <div />, <A-B />];\n",
        &[b"_Button", b"$Button", "Élement".as_bytes()],
    );
}
