use super::*;
use swc_next_ecma_semantic::ScopeModel;

#[test]
fn catch_linked_symbols() {
    let script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };
    let ts_script = Options {
        lang: Lang::Ts,
        ..script
    };
    for options in [script, ts_script] {
        assert_linked_symbols(
            "var e; try {} catch (e) { var e = 1; e; } e;",
            options,
            b"e",
            &[2],
        );

        assert_linked_symbols("try {} catch (e) { var e; } var e; e;", options, b"e", &[2]);

        assert_linked_symbols(
            "var e; try {} catch (e) { var e; var e; }",
            options,
            b"e",
            &[2],
        );

        assert_linked_symbols(
            "var e; try {} catch (e) { try {} catch (e) { var e; } }",
            options,
            b"e",
            &[3],
        );

        assert_linked_symbols(
            "try {} catch (e) { var e; } try {} catch (e) { var e; }",
            options,
            b"e",
            &[3],
        );

        assert_linked_symbols(
            "try {} catch (e) { for (var e of values) e; } e;",
            options,
            b"e",
            &[2],
        );

        assert_linked_symbols(
            "try {} catch (e) { var [e] = values; } e;",
            options,
            b"e",
            &[2],
        );

        assert_linked_symbols("var e; try {} catch (e) { e; }", options, b"e", &[1, 1]);

        assert_linked_symbols(
            "var e; try {} catch (e) { function f() { var e; } }",
            options,
            b"e",
            &[1, 1, 1],
        );
    }
}

#[test]
fn parameter_linked_symbols() {
    let module = Options::default();
    let ts_module = ts_module_options();
    for options in [module, ts_module] {
        assert_linked_symbols(
            "function f(a = 1) { var a; return a; }",
            options,
            b"a",
            &[2],
        );

        assert_linked_symbols(
            "function f(a = 1, read = () => a) { var a; a = 2; return read(); }",
            options,
            b"a",
            &[2],
        );

        assert_linked_symbols(
            "function f(a = 1) { try {} catch (a) { var a; } return a; }",
            options,
            b"a",
            &[3],
        );

        assert_linked_symbols("function f(a) { var a; return a; }", options, b"a", &[1]);

        assert_linked_symbols(
            "function f(a = 1) { function g() { var a; } }",
            options,
            b"a",
            &[1, 1],
        );
    }

    assert_linked_symbols(
        "function f(a = 1) { type a = number; var a; }",
        ts_module,
        b"a",
        &[2],
    );

    assert_linked_symbols(
        "function f(a = 1) { var a; function a() {} return a; }",
        module,
        b"a",
        &[1, 1],
    );

    assert_linked_symbols(
        "function f(a = 1) { function a() {} var a; return a; }",
        module,
        b"a",
        &[1, 1],
    );
}

#[test]
fn annex_b_linked_symbols() {
    let script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };
    let ts_script = Options {
        lang: Lang::Ts,
        ..script
    };
    for options in [script, ts_script] {
        assert_linked_symbols(
            "var f; { function f() { return f; } } f;",
            options,
            b"f",
            &[2],
        );

        assert_linked_symbols("{ function f() { return f; } } f;", options, b"f", &[2]);

        assert_linked_symbols(
            "var f; { function f() {} } { function f() {} } f;",
            options,
            b"f",
            &[3],
        );

        assert_linked_symbols(
            "if (condition) function f() { return f; } f;",
            options,
            b"f",
            &[2],
        );

        assert_linked_symbols(
            "try {} catch (f) { var f; } { function f() {} } f;",
            options,
            b"f",
            &[3],
        );

        assert_linked_symbols(
            "'use strict'; var f; { function f() {} } f;",
            options,
            b"f",
            &[1, 1],
        );

        assert_linked_symbols("let f; { function f() {} } f;", options, b"f", &[1, 1]);

        assert_linked_symbols(
            "function outer(f = 1) { { function f() {} } return f; }",
            options,
            b"f",
            &[1, 1],
        );
    }
}

#[track_caller]
fn assert_linked_symbols(source: &str, options: Options, name: &[u8], group_sizes: &[usize]) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let (ast, semantic) =
            analyze_with_options(&allocator, source, options, AnalyzeOptions { check_syntax });
        let symbols = semantic
            .iter_symbols()
            .filter(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == name)
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        assert_eq!(symbols.len(), group_sizes.iter().sum::<usize>());
        let mut groups = Vec::new();
        for &symbol in &symbols {
            let mut group = semantic.linked_symbols(symbol).collect::<Vec<_>>();
            group.sort_unstable();
            assert!(group.contains(&symbol));
            assert!(group.windows(2).all(|pair| pair[0] != pair[1]));
            assert!(group.iter().all(|member| symbols.contains(member)));
            groups.push(group);
        }
        let mut unique = groups.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.iter().map(Vec::len).sum::<usize>(), symbols.len());
        let mut sizes = unique.iter().map(Vec::len).collect::<Vec<_>>();
        sizes.sort_unstable();
        assert_eq!(sizes, group_sizes);
        let model = ScopeModel::new(semantic);
        for (symbol, expected) in symbols.into_iter().zip(groups) {
            let mut group = model.linked_symbols(symbol).collect::<Vec<_>>();
            group.sort_unstable();
            assert_eq!(group, expected);
        }
    }
}

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
            semantic.lookup(hoist_scope, b"f", ReferenceSpace::Type),
            None
        );
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
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
        swc_next_ecma_semantic::scope::ScopeKind::Function
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

        assert_annex_b_companions(
            "{ function f() {} { function f(value) {} } } f;",
            script,
            &["f() {}"],
        );

        assert_annex_b_companions(
            "{ { function f(value) {} } function f() {} } f;",
            script,
            &["f() {}"],
        );

        assert_annex_b_companions(
            "{ function f() {} function f(value) {} } f;",
            script,
            &["f() {}", "f(value) {}"],
        );

        assert_annex_b_companions(
            "{ function f() {} if (condition) function f(value) {} } f;",
            script,
            &["f() {}"],
        );

        assert_annex_b_companions(
            "switch (value) { case 0: function f() {} { function f(value) {} } } f;",
            script,
            &["f() {}"],
        );

        assert_annex_b_companions(
            "{ function f() {} } { function f(value) {} } f;",
            script,
            &["f() {}", "f(value) {}"],
        );

        assert_annex_b_companions(
            "{ function f() {} function g() { { function f(value) {} } return f; } }",
            script,
            &["f(value) {}"],
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
            let options = AnalyzeOptions { check_syntax };
            let (ast, semantic) = analyze_with_options(&allocator, source, script, options);
            let variable = reference_symbol(&ast, &semantic, b"g").unwrap();
            let block = semantic
                .iter_scopes()
                .find(|entry| entry.scope.kind == swc_next_ecma_semantic::scope::ScopeKind::Block)
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
fn assert_annex_b_companions(source: &str, options: Options, declarations: &[&str]) {
    let allocator = Allocator::new();
    let expected = declarations
        .iter()
        .map(|declaration| source.find(declaration).unwrap() as u32)
        .collect::<Vec<_>>();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let analyze_options = AnalyzeOptions { check_syntax };
        let (ast, semantic) = analyze_with_options(&allocator, source, options, analyze_options);
        let outer = reference_symbols(&ast, &semantic, b"f")
            .pop()
            .unwrap()
            .unwrap();
        let actual = semantic
            .declarations(outer)
            .map(|declaration| {
                assert_eq!(declaration.flags, SymbolFlags::FUNCTION_SCOPED_VAR);
                ast.span(declaration.node).start
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
}

#[track_caller]
fn assert_annex_b_dual_binding(source: &str, options: Options) {
    let allocator = Allocator::new();
    let parse = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parse.diagnostics.is_empty());
    let mut previous = None;
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let result = analyze(&parse.ast, AnalyzeOptions { check_syntax });
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
        swc_next_ecma_semantic::scope::ScopeKind::Class
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

#[test]
fn function_declarations_annex_b() {
    let allocator = Allocator::new();
    let js_module = Options {
        lang: Lang::Js,
        source_type: SourceType::Module,
        ..Options::default()
    };
    let js_script = Options {
        lang: Lang::Js,
        source_type: SourceType::Script,
        ..Options::default()
    };
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

    for options in [js_script, ts_script] {
        let errors = |source| has_syntax_diagnostics(&allocator, source, options);

        let has_errors = errors("let f = 1;\nif (true) function f() { \"use strict\"; }");
        assert!(!has_errors);

        let has_errors = errors("if (true) function f() { \"use strict\"; }");
        assert!(!has_errors);

        let has_errors = errors("if (true) function eval() { \"use strict\"; }");
        assert!(has_errors);

        let has_errors = errors("if (true) function f(value = 1) { \"use strict\"; }");
        assert!(has_errors);

        let has_errors = errors("label: function f() { \"use strict\"; }");
        assert!(!has_errors);

        let has_errors =
            errors("function outer() {\n  if (true) function f() { \"use strict\"; }\n}");
        assert!(!has_errors);

        let has_errors = errors("\"use strict\";\nif (true) function f() {}");
        assert!(has_errors);

        let has_errors = errors("\"use strict\";\nlabel: function f() {}");
        assert!(has_errors);
    }

    for options in [js_module, ts_module] {
        let errors = |source| has_syntax_diagnostics(&allocator, source, options);

        let has_errors = errors("if (true) function f() { \"use strict\"; }");
        assert!(has_errors);

        let has_errors = errors("label: function f() { \"use strict\"; }");
        assert!(has_errors);
    }
}

#[test]
fn function_declarations_arrow_body() {
    let allocator = Allocator::new();
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let has_errors = errors("() => {\n  function f() {}\n  var f;\n};");
        assert!(has_errors);

        let has_errors = errors("() => {\n  var f;\n  function f() {}\n};");
        assert!(has_errors);
    }
}

#[test]
fn function_declarations_block() {
    let allocator = Allocator::new();
    let js_script = Options {
        lang: Lang::Js,
        source_type: SourceType::Script,
        ..Options::default()
    };
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
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let has_errors = errors("{\n  async function f() {}\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  class f {}\n  function f() {}\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  async function f() {}\n  async function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  function* f() {}\n  function* f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  function f() {}\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  function f() {}\n  async function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  function f() {}\n  function* f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  function f() {}\n  var f;\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  function* f() {}\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors =
            errors("{\n  function f(): void;\n  function f() {}\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  function f(): void;\n  function f() {}\n}");
        assert!(!has_errors);

        let has_errors = errors("{\n  var f;\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  var f;\n  function f(): void;\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  var f;\n  interface f {}\n  function f() {}\n}");
        assert!(has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_script);

        let has_errors = errors("{\n  function f() {}\n  function f() {}\n}");
        assert!(!has_errors);

        let has_errors = errors("\"use strict\";\n{ function f() {} function f() {} }");
        assert!(has_errors);
    }

    for options in [js_script, ts_script] {
        let errors = |source| has_syntax_diagnostics(&allocator, source, options);

        let has_errors =
            errors("{\n  function f() { \"use strict\"; }\n  function f() { \"use strict\"; }\n}");
        assert!(!has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, js_script);

        let has_errors = errors("function f() {}\n{\n  var f;\n  function f() {}\n}");
        assert!(has_errors);
    }
}

#[test]
fn function_declarations_catch() {
    let allocator = Allocator::new();
    let js_module = Options {
        lang: Lang::Js,
        source_type: SourceType::Module,
        ..Options::default()
    };
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
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let has_errors = errors("try {} catch {\n  function f() {}\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("try {} catch {\n  function f() {}\n  var f;\n}");
        assert!(has_errors);

        let has_errors = errors("try {} catch {\n  function f(): void;\n  function f() {}\n}");
        assert!(!has_errors);

        let has_errors = errors("try {} catch (f) {\n  function f(): void;\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("try {} catch ([error = 1]) {\n  function f() {}\n  var f;\n}");
        assert!(has_errors);

        let has_errors =
            errors("try {} catch ([error = 1]) {\n  function f(): void;\n  function f() {}\n}");
        assert!(!has_errors);

        let has_errors =
            errors("try {} catch ([f = 1]) {\n  function f(): void;\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("try {} catch ([f = 1]) {\n  function f(): void;\n}");
        assert!(has_errors);

        let has_errors = errors("try {} catch ([error = 1]) {\n  var f;\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("try {} catch {\n  var f;\n  function f() {}\n}");
        assert!(has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_script);

        let has_errors = errors("try {} catch {\n  function f() {}\n  function f() {}\n}");
        assert!(!has_errors);
    }

    for options in [js_module, ts_module] {
        let errors = |source| has_syntax_diagnostics(&allocator, source, options);

        let has_errors = errors("try {} catch ([f = 1]) {\n  {\n    function f() {}\n  }\n}");
        assert!(!has_errors);

        let has_errors = errors("try {} catch ([f = 1]) {\n  function f() {}\n}");
        assert!(has_errors);
    }
}

#[test]
fn function_declarations_function_body() {
    let allocator = Allocator::new();
    let js_module = Options {
        lang: Lang::Js,
        source_type: SourceType::Module,
        ..Options::default()
    };
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let has_errors = errors("function outer() {\n  function f() {}\n  var f;\n}");
        assert!(has_errors);

        let has_errors = errors("function outer(f: unknown) {\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("function outer(f = 1) {\n  function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("function outer() {\n  var f;\n  function f() {}\n}");
        assert!(has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, js_module);

        let has_errors = errors("function outer(f) {\n  function f() {}\n}");
        assert!(!has_errors);

        let has_errors = errors("function outer(f = 1) {\n  function f() {}\n}");
        assert!(!has_errors);
    }
}

#[test]
fn function_declarations_namespace() {
    let allocator = Allocator::new();
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let source = "namespace N {\n  export function f() {}\n}\nnamespace N {\n  export var f: unknown;\n}";
        let has_errors = errors(source);
        assert!(has_errors);

        let source = "namespace N {\n  export var f: unknown;\n}\nnamespace N {\n  export function f() {}\n}";
        let has_errors = errors(source);
        assert!(has_errors);
    }
}

#[test]
fn function_declarations_program() {
    let allocator = Allocator::new();
    let dts_module = Options {
        lang: Lang::Dts,
        source_type: SourceType::Module,
        ..Options::default()
    };
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };
    let ts_script = Options {
        source_type: SourceType::Script,
        ..ts_module
    };

    for options in [ts_module, ts_script] {
        let errors = |source| has_syntax_diagnostics(&allocator, source, options);

        let has_errors = errors("declare class f {}\nfunction f(): void;\nfunction f() {}");
        assert!(!has_errors);

        let has_errors = errors("function f() {}\ndeclare class f {}");
        assert!(!has_errors);

        let has_errors = errors("function f() {}\nvar f;");
        assert!(has_errors);

        let has_errors = errors("var f;\nfunction f() {}");
        assert!(has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, dts_module);

        let has_errors = errors("declare function f(): void;\ndeclare var f: unknown;");
        assert!(has_errors);

        let has_errors = errors("declare var f: unknown;\ndeclare function f(): void;");
        assert!(has_errors);
    }
}

#[test]
fn function_declarations_static_block() {
    let allocator = Allocator::new();
    let js_module = Options {
        lang: Lang::Js,
        source_type: SourceType::Module,
        ..Options::default()
    };
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let source =
            "class C {\n  static {\n    async function f() {}\n    async function f() {}\n  }\n}";
        let has_errors = errors(source);
        assert!(!has_errors);

        let has_errors =
            errors("class C {\n  static {\n    function* f() {}\n    function* f() {}\n  }\n}");
        assert!(!has_errors);

        let has_errors = errors("class C {\n  static {\n    function f() {}\n    var f;\n  }\n}");
        assert!(has_errors);

        let has_errors =
            errors("class C {\n  static {\n    function f(): void;\n    function f() {}\n  }\n}");
        assert!(!has_errors);

        let has_errors = errors("class C {\n  static {\n    var f;\n    function f() {}\n  }\n}");
        assert!(has_errors);
    }

    for options in [js_module, ts_module] {
        let errors = |source| has_syntax_diagnostics(&allocator, source, options);

        let has_errors =
            errors("class C {\n  static {\n    function f() {}\n    function f() {}\n  }\n}");
        assert!(!has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, js_module);

        let has_errors = errors("class C {\n  static {\n    function f() {}\n    var f;\n  }\n}");
        assert!(!has_errors);

        let has_errors = errors("class C {\n  static {\n    var f;\n    function f() {}\n  }\n}");
        assert!(!has_errors);
    }
}

#[test]
fn function_declarations_switch() {
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
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let has_errors =
            errors("switch (0) {\n  case 0:\n    function f() {}\n    function f() {}\n}");
        assert!(has_errors);

        let has_errors = errors("switch (0) {\n  case 0:\n    function f() {}\n    var f;\n}");
        assert!(has_errors);

        let has_errors =
            errors("switch (0) {\n  case 0:\n    function f(): void;\n    function f() {}\n}");
        assert!(!has_errors);

        let has_errors = errors("switch (0) {\n  case 0:\n    var f;\n    function f() {}\n}");
        assert!(has_errors);
    }

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_script);

        let has_errors =
            errors("switch (0) {\n  case 0:\n    function f() {}\n    function f() {}\n}");
        assert!(!has_errors);
    }
}

#[test]
fn hoisting_variables() {
    let allocator = Allocator::new();
    let ts_module = Options {
        lang: Lang::Ts,
        source_type: SourceType::Module,
        ..Options::default()
    };

    {
        let errors = |source| has_syntax_diagnostics(&allocator, source, ts_module);

        let has_errors = errors("{\n  const enum E {}\n  var E;\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  enum E {}\n  var E;\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  var E;\n  const enum E {}\n}");
        assert!(has_errors);

        let has_errors = errors("{\n  var E;\n  enum E {}\n}");
        assert!(has_errors);
    }
}
