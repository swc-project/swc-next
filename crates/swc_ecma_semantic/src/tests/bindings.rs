use super::*;

#[test]
fn hoisting_variables_keep_lexical_bindings() {
    let module = Options::default();
    let ts_module = ts_module_options();

    let source = "{ var f; function f() {} f; } f;";
    assert_separate_lexical_binding(source, module);

    let source = "namespace N { { var f; type f = number; let value: f; } f; }";
    assert_separate_lexical_binding(source, ts_module);
}

#[track_caller]
fn assert_separate_lexical_binding(source: &str, options: Options) {
    let allocator = Allocator::new();
    let parse = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parse.diagnostics.is_empty());

    let check = |options| {
        let result = analyze(&parse.ast, options);
        let semantic = &result.semantic;
        let references = reference_symbols(&parse.ast, semantic, b"f");
        assert_eq!(references.len(), 2);
        let lexical = references[0].unwrap();
        let hoisted = references[1].unwrap();
        assert_ne!(lexical, hoisted);
        let lexical_scope = semantic.scope_of_symbol(lexical);
        let hoist_scope = semantic.scope_of_symbol(hoisted);
        assert_ne!(lexical_scope, hoist_scope);
        assert_eq!(semantic.binding(lexical_scope, b"f"), Some(lexical));
        assert_eq!(semantic.binding(hoist_scope, b"f"), Some(hoisted));
        assert_eq!(semantic.declarations(lexical).len(), 1);
        assert_eq!(semantic.declarations(hoisted).len(), 1);
        assert_eq!(
            semantic.resolve_name(hoist_scope, b"f", ReferenceSpace::Type),
            NameResolution::Unresolved
        );
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    });
}

#[test]
fn annex_b_if_bindings() {
    let allocator = Allocator::new();
    let script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };
    let checked = AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    };

    let source = "if (condition) function f() { return f; }";
    let (_, semantic) = analyze_with_options(&allocator, source, script, checked);
    let name = expression_name_symbol(&semantic, SymbolFlags::ANNEX_B_IF_FUNCTION);
    let outer = semantic.binding(ScopeId::ROOT, b"f").unwrap();
    assert_ne!(name, outer);
    assert!(
        semantic
            .symbol(outer)
            .flags
            .contains(SymbolFlags::FUNCTION_SCOPED_VAR)
    );
    let scope = semantic.scope_of_symbol(name);
    assert!(
        semantic
            .symbol(name)
            .flags
            .contains(SymbolFlags::ANNEX_B_IF_FUNCTION)
    );
    assert_eq!(
        semantic.scope(scope).kind,
        crate::scope::ScopeKind::Function
    );
    assert_eq!(semantic.scope(scope).parent, Some(ScopeId::ROOT));
    assert_eq!(semantic.iter_scopes().count(), 2);
    assert_eq!(semantic.binding(scope, b"f"), Some(name));
    assert_eq!(
        semantic.symbol_of(semantic.declarations(outer).next().unwrap().node),
        Some(name)
    );

    let source = "let f = 1; if (condition) function f() { 'use strict'; return f; } f;";
    let (ast, semantic) = analyze_with_options(&allocator, source, script, checked);
    let name = expression_name_symbol(&semantic, SymbolFlags::ANNEX_B_IF_FUNCTION);
    let outer = semantic.binding(ScopeId::ROOT, b"f").unwrap();
    assert_ne!(name, outer);
    assert_eq!(
        reference_symbols(&ast, &semantic, b"f"),
        [Some(name), Some(outer)]
    );
    assert!(semantic.scope(semantic.scope_of_symbol(name)).flags.strict);
    assert!(!semantic.scope(ScopeId::ROOT).flags.strict);

    let source = "if (condition) function f(f) { return f; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, script, checked);
    let name = expression_name_symbol(&semantic, SymbolFlags::ANNEX_B_IF_FUNCTION);
    let parameter = reference_symbol(&ast, &semantic, b"f").unwrap();
    assert_ne!(name, parameter);
    assert_eq!(
        semantic.scope_of_symbol(name),
        semantic.scope_of_symbol(parameter)
    );
    assert_eq!(semantic.declarations(name).len(), 1);
    assert_eq!(semantic.declarations(parameter).len(), 1);
    assert!(semantic.uses(name).is_empty());

    let source = "if (condition) function f() { var f; return f; }";
    let (ast, semantic) = analyze_with_options(&allocator, source, script, checked);
    let name = expression_name_symbol(&semantic, SymbolFlags::ANNEX_B_IF_FUNCTION);
    let local = reference_symbol(&ast, &semantic, b"f").unwrap();
    assert_ne!(name, local);
    assert_eq!(
        semantic.scope_of_symbol(name),
        semantic.scope_of_symbol(local)
    );
    assert!(
        !semantic
            .symbol(local)
            .flags
            .contains(SymbolFlags::ANNEX_B_IF_FUNCTION)
    );
}

#[test]
fn annex_b_companion_bindings() {
    for lang in [Lang::Js, Lang::Ts] {
        let script = Options {
            lang,
            source_type: SourceType::Script,
            ..Options::default()
        };

        assert_annex_b_dual_binding("if (condition) function f() { return f; } f;", script);

        assert_annex_b_dual_binding(
            "if (condition) function f() { return f; } const g = f; f = 1; g();",
            script,
        );

        assert_annex_b_dual_binding("if (false) function f() { return f; } f;", script);

        assert_annex_b_dual_binding(
            "if (condition) function f() { 'use strict'; return f; } f;",
            script,
        );

        assert_annex_b_dual_binding(
            "if (condition) {} else function f() { return f; } f;",
            script,
        );

        assert_annex_b_dual_binding("{ function f() { return f; } } f;", script);

        assert_annex_b_dual_binding(
            "{ function f() {} { function f() { return f; } } } f;",
            script,
        );

        assert_annex_b_dual_binding("{ function f() {} function f() { return f; } } f;", script);

        assert_annex_b_dual_binding(
            "switch (value) { case 0: function f() { return f; } } f;",
            script,
        );

        assert_annex_b_dual_binding("try {} catch { function f() { return f; } } f;", script);

        assert_annex_b_dual_binding(
            "try {} catch ([value = 1]) { function f() { return f; } } f;",
            script,
        );

        assert_annex_b_dual_binding(
            "try {} catch (f) { if (condition) function f() { return f; } } f;",
            script,
        );

        assert_annex_b_dual_binding(
            "function outer() { if (condition) function f() { return f; } return f; }",
            script,
        );

        assert_annex_b_dual_binding(
            "function outer(value = 1) { if (condition) function f() { return f; } return f; }",
            script,
        );

        assert_annex_b_dual_binding(
            "var f; if (condition) function f() { return f; } f;",
            script,
        );

        assert_annex_b_dual_binding(
            "if (condition) function f() { return f; } var f; f;",
            script,
        );

        assert_annex_b_dual_binding(
            "function f() {} if (condition) function f() { return f; } f;",
            script,
        );

        assert_annex_b_dual_binding(
            "const value = function f() { if (condition) function f() { return f; } return f; };",
            script,
        );

        assert_annex_b_dual_binding(
            "let f; function outer() { if (condition) function f() { return f; } return f; }",
            script,
        );

        let allocator = Allocator::new();
        let source = "if (condition) function f() {} { var g; } g;";
        for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
            let options = AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            };
            let (ast, semantic) = analyze_with_options(&allocator, source, script, options);
            let variable = reference_symbol(&ast, &semantic, b"g").unwrap();
            let block = semantic
                .iter_scopes()
                .find(|entry| entry.scope.kind == crate::scope::ScopeKind::Block)
                .unwrap()
                .id;
            assert_eq!(semantic.binding(block, b"g"), Some(variable));
        }
    }

    let ts_script = Options {
        lang: Lang::Ts,
        source_type: SourceType::Script,
        ..Options::default()
    };

    assert_annex_b_dual_binding(
        "namespace N { export var f; if (condition) function f() { return f; } f; }",
        ts_script,
    );

    assert_annex_b_dual_binding(
        "namespace N { if (condition) function f() { return f; } export var f; f; }",
        ts_script,
    );

    assert_annex_b_dual_binding(
        "namespace N { export function f(value) {} if (condition) function f() { return f; } f; }",
        ts_script,
    );

    assert_annex_b_dual_binding(
        "namespace N { type f = string; export var f; if (condition) function f() { return f; } f; }",
        ts_script,
    );
}

#[track_caller]
fn assert_annex_b_dual_binding(source: &str, options: Options) {
    let allocator = Allocator::new();
    let parse = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parse.diagnostics.is_empty());
    let mut previous = None;
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let result = analyze(
            &parse.ast,
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        assert!(result.diagnostics.is_empty());
        let semantic = result.semantic;
        let references = reference_symbols(&parse.ast, &semantic, b"f");
        assert!(references.len() >= 2);
        let lexical = references[0].unwrap();
        let outer = references[1].unwrap();
        assert_ne!(lexical, outer);
        assert!(
            references[1..]
                .iter()
                .all(|&reference| reference == Some(outer))
        );
        assert!(
            semantic
                .symbol(lexical)
                .flags
                .contains(SymbolFlags::FUNCTION_IMPLEMENTATION)
        );
        assert!(
            semantic
                .symbol(outer)
                .flags
                .contains(SymbolFlags::FUNCTION_SCOPED_VAR)
        );
        let companion = semantic
            .declarations(outer)
            .find(|declaration| semantic.symbol_of(declaration.node) == Some(lexical))
            .unwrap();
        assert_eq!(companion.flags, SymbolFlags::FUNCTION_SCOPED_VAR);
        assert_eq!(
            semantic.lookup(companion.scope, b"f", ReferenceSpace::Value),
            Some(outer)
        );
        let starts = semantic
            .iter_declarations()
            .map(|entry| parse.ast.span(entry.declaration.node).start)
            .collect::<Vec<_>>();
        assert!(starts.is_sorted());
        let facts = (
            semantic
                .iter_symbols()
                .map(|entry| (entry.id, entry.symbol.flags, entry.symbol.declaration))
                .collect::<Vec<_>>(),
            semantic
                .iter_declarations()
                .map(|entry| {
                    (
                        entry.id,
                        entry.declaration.node,
                        entry.declaration.symbol,
                        entry.declaration.scope,
                        entry.declaration.flags,
                    )
                })
                .collect::<Vec<_>>(),
            semantic
                .iter_references()
                .map(|entry| {
                    (
                        entry.id,
                        entry.reference.scope,
                        entry.reference.symbol,
                        entry.reference.flags,
                    )
                })
                .collect::<Vec<_>>(),
        );
        if let Some(previous) = previous.as_ref() {
            assert_eq!(&facts, previous);
        }
        previous = Some(facts);
    }
}

#[test]
fn expression_name_bindings() {
    let allocator = Allocator::new();
    let js = Options::default();
    let checked = AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    };

    let source = "const f = function n(n) { return n; };";
    let (ast, semantic) = analyze_with_options(&allocator, source, js, checked);
    let name = expression_name_symbol(&semantic, SymbolFlags::FUNCTION_EXPRESSION);
    let parameter = reference_symbol(&ast, &semantic, b"n").unwrap();
    assert_ne!(name, parameter);
    assert_eq!(
        semantic.scope_of_symbol(name),
        semantic.scope_of_symbol(parameter)
    );
    assert_eq!(
        semantic.binding(semantic.scope_of_symbol(name), b"n"),
        Some(parameter)
    );
    assert!(semantic.uses(name).is_empty());
    assert_eq!(semantic.declarations(name).len(), 1);
    assert_eq!(semantic.declarations(parameter).len(), 1);
    assert_eq!(semantic.iter_scopes().count(), 3);

    let source = "const f = function n() { var n; return n; };";
    let (ast, semantic) = analyze_with_options(&allocator, source, js, checked);
    let name = expression_name_symbol(&semantic, SymbolFlags::FUNCTION_EXPRESSION);
    let local = reference_symbol(&ast, &semantic, b"n").unwrap();
    assert_ne!(name, local);
    assert_eq!(
        semantic.scope_of_symbol(name),
        semantic.scope_of_symbol(local)
    );
    assert!(
        !semantic
            .symbol(local)
            .flags
            .contains(SymbolFlags::FUNCTION_EXPRESSION)
    );

    let source = "const f = function n() { let n; return n; };";
    let (ast, semantic) = analyze_with_options(&allocator, source, js, checked);
    let name = expression_name_symbol(&semantic, SymbolFlags::FUNCTION_EXPRESSION);
    let local = reference_symbol(&ast, &semantic, b"n").unwrap();
    assert_ne!(name, local);
    assert_eq!(
        semantic.scope_of_symbol(name),
        semantic.scope_of_symbol(local)
    );

    let source = "const C = class Inner { self = Inner; };";
    let (ast, semantic) = analyze_with_options(&allocator, source, js, checked);
    let name = expression_name_symbol(&semantic, SymbolFlags::CLASS_EXPRESSION);
    assert_eq!(
        semantic.scope(semantic.scope_of_symbol(name)).kind,
        crate::scope::ScopeKind::Class
    );
    assert_eq!(reference_symbol(&ast, &semantic, b"Inner"), Some(name));
    assert!(semantic.binding(ScopeId::MODULE, b"Inner").is_none());
    assert_eq!(semantic.iter_scopes().count(), 3);
}

fn expression_name_symbol(semantic: &Semantic<'_>, flag: SymbolFlags) -> SymbolId {
    let mut names = semantic
        .iter_symbols()
        .filter(|entry| entry.symbol.flags.contains(flag));
    let name = names.next().unwrap().id;
    assert!(names.next().is_none());
    name
}

#[test]
fn class_function_conflicts() {
    let allocator = Allocator::new();
    let script = Options {
        lang: Lang::Ts,
        source_type: SourceType::Script,
        ..Options::default()
    };

    let source = "{ class f {} function f() {} function f() {} }";
    let (_, result) = analyze_syntax_source(&allocator, source, script);
    assert_eq!(result.diagnostics.len(), 2);
    assert_eq!(
        result.diagnostics[1].span.start as usize,
        source.rfind("f()").unwrap()
    );

    let source = "{ function f() {} class f {} function f() {} }";
    let (_, result) = analyze_syntax_source(&allocator, source, script);
    assert_eq!(result.diagnostics.len(), 2);
    assert_eq!(
        result.diagnostics[1].span.start as usize,
        source.rfind("f()").unwrap()
    );
}

#[test]
fn declarations_share_canonical_symbols() {
    let allocator = Allocator::new();
    let ts_script = Options {
        lang: Lang::Ts,
        source_type: SourceType::Script,
        ..Options::default()
    };

    let source = "interface A {} interface B {} interface A {} interface A {}";
    let (ast, semantic) = analyze_source(&allocator, source, ts_script);
    let a = semantic.binding(ScopeId::ROOT, b"A").unwrap();
    let b = semantic.binding(ScopeId::ROOT, b"B").unwrap();
    let mut a_declarations = semantic.declaration_ids(a);
    assert_eq!(a_declarations.len(), 3);
    let first_a = a_declarations.next().unwrap();
    let last_a = a_declarations.next_back().unwrap();
    assert!(
        ast.span(semantic.declaration(first_a).node).start
            < ast.span(semantic.declaration(last_a).node).start
    );

    let mut declarations = semantic.iter_declarations();
    let first = declarations.next().unwrap();
    let second = declarations.next().unwrap();
    let third = declarations.next().unwrap();
    let fourth = declarations.next().unwrap();
    assert!(declarations.next().is_none());
    assert_eq!(
        [
            first.declaration.symbol,
            second.declaration.symbol,
            third.declaration.symbol,
            fourth.declaration.symbol,
        ],
        [a, b, a, a]
    );
    assert_eq!(semantic.symbol(a).declaration, first.id);
    assert_eq!(
        semantic.declaration_ids(a).collect::<Vec<_>>(),
        [first.id, third.id, fourth.id]
    );
}

#[test]
fn merged_type_parameters_share_identity() {
    let allocator = Allocator::new();
    let ts = ts_module_options();

    let source = "interface Box<T> { value: T } interface Box<T> { extra: T }";
    let (ast, semantic) = analyze_source(&allocator, source, ts);
    assert_merged_type_parameter(&ast, &semantic, b"T");

    let source = "class Box<T> { value!: T } interface Box<T> { extra: T }";
    let (ast, semantic) = analyze_source(&allocator, source, ts);
    assert_merged_type_parameter(&ast, &semantic, b"T");

    let source = "interface Box<T> { extra: T } class Box<T> { value!: T }";
    let (ast, semantic) = analyze_source(&allocator, source, ts);
    assert_merged_type_parameter(&ast, &semantic, b"T");

    let source = "namespace N { export interface Box<T> { value: T } } namespace N { export interface Box<T> { extra: T } }";
    let (ast, semantic) = analyze_source(&allocator, source, ts);
    assert_merged_type_parameter(&ast, &semantic, b"T");

    let source = "export {}; declare global { interface Box<T> { value: T } } declare global { interface Box<T> { extra: T } }";
    let (ast, semantic) = analyze_source(&allocator, source, ts);
    assert_merged_type_parameter(&ast, &semantic, b"T");
}

#[test]
fn merged_type_parameters_keep_occurrence_visibility() {
    let allocator = Allocator::new();
    let ts = ts_module_options();
    let checked = AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    };

    let source = "type U = number; interface Box<T> { value: T; outer: U } interface Box<T, U = string> { extra: T; inner: U }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    assert_merged_type_parameter(&ast, &semantic, b"T");
    let outer = semantic.binding(ScopeId::MODULE, b"U").unwrap();
    let references = reference_symbols(&ast, &semantic, b"U");
    assert_eq!(references[0], Some(outer));
    assert_ne!(references[1], Some(outer));
    assert!(references[1].is_some());

    let source =
        "const T = 1; class Box<T> { T!: T; method() { return T; } } interface Box<T> { value: T }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let outer = semantic.binding(ScopeId::MODULE, b"T").unwrap();
    let references = reference_symbols(&ast, &semantic, b"T");
    assert_eq!(references[0], references[2]);
    assert_ne!(references[0], Some(outer));
    assert_eq!(references[1], Some(outer));
    assert_eq!(semantic.declarations(references[0].unwrap()).len(), 2);

    let source = "class Box<T> { constructor(public T: T) { const value = T; } } interface Box<T> { extra: T }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    let references = reference_symbols(&ast, &semantic, b"T");
    assert_eq!(references[0], references[2]);
    assert_ne!(references[0], references[1]);
    let parameter = references[1].unwrap();
    assert!(
        semantic
            .symbol(parameter)
            .flags
            .contains(SymbolFlags::PARAMETER)
    );
    assert_eq!(semantic.declarations(parameter).len(), 1);
    assert_eq!(semantic.declarations(references[0].unwrap()).len(), 2);
}

#[test]
fn unrelated_type_parameters_keep_distinct_identities() {
    let allocator = Allocator::new();
    let ts = ts_module_options();

    let source = "namespace N { interface Box<T> { value: T } } namespace N { interface Box<T> { extra: T } }";
    let (ast, semantic) = analyze_source(&allocator, source, ts);
    assert_distinct_type_parameters(&ast, &semantic, b"T");

    let source = "interface Box<T> { value: T } const C = class Box<T> { value!: T };";
    let (ast, semantic) = analyze_source(&allocator, source, ts);
    assert_distinct_type_parameters(&ast, &semantic, b"T");

    let source = "declare function f<T>(value: T): void; declare function f<T>(value: T, extra: number): void;";
    let (ast, semantic) = analyze_source(&allocator, source, ts);
    assert_distinct_type_parameters(&ast, &semantic, b"T");

    let source = "interface Box<T> { value: T } interface Box<T> { method<T>(value: T): void }";
    let (ast, semantic) = analyze_source(&allocator, source, ts);
    let references = reference_symbols(&ast, &semantic, b"T");
    assert_ne!(references[0], references[1]);
    assert_eq!(semantic.declarations(references[0].unwrap()).len(), 2);
    assert_eq!(semantic.declarations(references[1].unwrap()).len(), 1);
}

#[track_caller]
fn assert_distinct_type_parameters(ast: &Ast<'_>, semantic: &Semantic<'_>, name: &[u8]) {
    let references = reference_symbols(ast, semantic, name);
    assert_eq!(references.len(), 2);
    let first = references[0].unwrap();
    let second = references[1].unwrap();
    assert_ne!(first, second);
    assert_eq!(semantic.declarations(first).len(), 1);
    assert_eq!(semantic.declarations(second).len(), 1);
}

#[test]
fn default_type_parameters() {
    let allocator = Allocator::new();
    let ts = ts_module_options();
    let checked = AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        build_module_record: true,
    };

    let source =
        "export default class C<T> { value!: T } export default interface I<T> { extra: T }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    assert_merged_type_parameter(&ast, &semantic, b"T");

    let source =
        "export default interface A<T> { value: T } export default interface B<T> { extra: T }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    assert_merged_type_parameter(&ast, &semantic, b"T");

    let source = "export default class<T> { value!: T } export default interface I<T> { extra: T } const ctor = I;";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    assert_merged_type_parameter(&ast, &semantic, b"T");
    let symbol = reference_symbol(&ast, &semantic, b"I").unwrap();
    assert!(semantic.symbol(symbol).flags.contains(SymbolFlags::CLASS));
    assert_eq!(semantic.declarations(symbol).len(), 2);

    let source = "export default function f<T>(value: T) { return value; } export default interface I<T> { value: T }";
    let (ast, semantic) = analyze_with_options(&allocator, source, ts, checked);
    assert_distinct_type_parameters(&ast, &semantic, b"T");
}
