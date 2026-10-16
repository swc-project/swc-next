use super::*;

#[test]
fn annex_b_if_visibility() {
    let script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };
    let ts_script = Options {
        lang: Lang::Ts,
        ..script
    };

    assert_named_declaration_targets(
        "if (condition) function f() { return f; } f;",
        script,
        b"f",
        &[Some("f()"), None],
    );

    assert_named_declaration_targets(
        "let f; if (condition) function f() { return f; } f;",
        script,
        b"f",
        &[Some("f()"), Some("f; if")],
    );

    assert_named_declaration_targets(
        "if (condition) function f() { return f; } else function f(x) { return f; }",
        script,
        b"f",
        &[Some("f()"), Some("f(x)")],
    );

    assert_named_declaration_targets(
        "if (condition) function f(f) { return f; }",
        script,
        b"f",
        &[Some("f) {")],
    );

    assert_named_declaration_targets(
        "if (condition) function f(a = f) { var f; return f; }",
        script,
        b"f",
        &[Some("f(a"), Some("f; return")],
    );

    assert_named_declaration_targets(
        "if (condition) function arguments() { return arguments; }",
        script,
        b"arguments",
        &[None],
    );

    assert_named_declaration_targets(
        "if (condition) function f(): typeof f { var f; return f; }",
        ts_script,
        b"f",
        &[Some("f():"), Some("f; return")],
    );
}

#[test]
fn expression_name_visibility() {
    let js = Options::default();
    let ts = ts_module_options();

    assert_named_declaration_targets(
        "const f = function n() { return n; }; n;",
        js,
        b"n",
        &[Some("n()"), None],
    );

    assert_named_declaration_targets(
        "const f = function n(n) { return n; };",
        js,
        b"n",
        &[Some("n) {")],
    );

    assert_named_declaration_targets(
        "const f = function n() { var n; return n; };",
        js,
        b"n",
        &[Some("n; return")],
    );

    assert_named_declaration_targets(
        "const f = function n(a = n) { var n; return n; };",
        js,
        b"n",
        &[Some("n(a"), Some("n; return")],
    );

    assert_named_declaration_targets(
        "const f = function n(a = n) { let n; return n; };",
        js,
        b"n",
        &[Some("n(a"), Some("n; return")],
    );

    assert_named_declaration_targets(
        "const f = function n<T extends typeof n>() { var n; };",
        ts,
        b"n",
        &[Some("n<T")],
    );

    assert_named_declaration_targets(
        "const f = function n(): typeof n { var n; return n; };",
        ts,
        b"n",
        &[Some("n():"), Some("n; return")],
    );

    assert_named_declaration_targets(
        "const f = function n<n>(a: n) { return n; };",
        ts,
        b"n",
        &[Some("n>(a"), Some("n<n>")],
    );

    assert_named_declaration_targets(
        "const C = class Inner<T extends Inner> { self!: Inner; };",
        ts,
        b"Inner",
        &[Some("Inner<T"), Some("Inner<T")],
    );

    assert_named_declaration_targets(
        "const C = class Inner<Inner> { value!: Inner; self = Inner; };",
        ts,
        b"Inner",
        &[Some("Inner>"), Some("Inner<")],
    );

    assert_named_declaration_targets(
        "const f = function n<n>(x: n) { return n; }; const g = function n<n>(y: n) { return n; };",
        ts,
        b"n",
        &[Some("n>(x"), Some("n<n>(x"), Some("n>(y"), Some("n<n>(y")],
    );

    assert_named_declaration_targets(
        "const a = 1; const f = function n<a>(x: a) { return a; };",
        ts,
        b"a",
        &[Some("a>(x"), Some("a = 1")],
    );

    assert_named_declaration_targets(
        "namespace N { export const f = function n<n>(x: n) { return n; }; }",
        ts,
        b"n",
        &[Some("n>(x"), Some("n<n>")],
    );

    assert_named_declaration_targets(
        "namespace N { export const C = class Inner<Inner> { value!: Inner; self = Inner; }; }",
        ts,
        b"Inner",
        &[Some("Inner>"), Some("Inner<")],
    );

    assert_named_declaration_targets(
        "interface Type {} function check() { var Type; let value: Type; return Type; }",
        ts,
        b"Type",
        &[Some("Type {}"), Some("Type; let")],
    );
}

#[track_caller]
fn assert_decorator_target(source: &str, name: &[u8], declaration: &str) {
    let source = format!("declare function dec<U>(value?: unknown): any;\n{source}");
    let expected_start = source.find(declaration).unwrap() as u32;
    let allocator = Allocator::new();
    let check = |options| {
        let (ast, semantic) =
            analyze_with_options(&allocator, &source, ts_module_options(), options);
        let reference = semantic
            .iter_references()
            .find(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name)
            .unwrap()
            .reference;
        let target = reference.symbol.unwrap();
        let declaration = semantic.declarations(target).next().unwrap();
        assert_eq!(ast.span(declaration.node).start, expected_start);
        assert_eq!(
            semantic.lookup(reference.scope, name, reference.flags.space),
            Some(target)
        );
    };
    check(AnalyzeOptions::default());
    check(AnalyzeOptions {
        check_syntax: SyntaxChecks::all(),
        ..AnalyzeOptions::default()
    });
}

#[test]
fn decorator_visibility() {
    // TS7 accepts these in the applicable legacy or standard decorator mode.
    // Class declarations and expressions have different decorator environments.
    assert_decorator_target(
        "type T = string;\n@dec<T>() class C<T> {}",
        b"T",
        "T = string",
    );

    assert_decorator_target(
        "type T = string;\nconst C = @dec<T>() class Inner<T> {};",
        b"T",
        "T> {}",
    );

    assert_decorator_target(
        "type T = string;\nclass C<T> { @dec<T>() m<T>() {} }",
        b"T",
        "T> {",
    );

    assert_decorator_target(
        "type T = string;\nclass C<T> { @dec<T>() value = 1; }",
        b"T",
        "T> {",
    );

    assert_decorator_target(
        "type T = string;\nclass C<T> { constructor(@dec<T>() value: T) {} }",
        b"T",
        "T> {",
    );

    assert_decorator_target(
        "type T = string;\nclass C<T> { m<T>(@dec<T>() value: T) {} }",
        b"T",
        "T> {",
    );

    assert_decorator_target(
        "const C = class Inner { @dec<Inner>() m() {} };",
        b"Inner",
        "Inner {",
    );
}

#[test]
fn decorator_lexical_bindings() {
    assert_named_declaration_targets(
        "declare function dec<U>(): any; class C<T> { m<T>(@dec<T>() value: T) {} }",
        ts_module_options(),
        b"T",
        &[Some("T> {"), Some("T>(@")],
    );

    assert_decorator_target(
        "const value = 1;\nclass C { m(@dec(value) value: number) {} }",
        b"value",
        "value = 1",
    );

    assert_decorator_target(
        "class C { @(((local: number) => dec(local))(1)) m() {} }",
        b"local",
        "local: number",
    );
}

#[track_caller]
fn assert_type_name_targets(source: &str, name: &[u8], outer_targets: &[bool]) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let outer = semantic.binding(ScopeId::MODULE, name).unwrap();
        let references = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name)
            .map(|entry| entry.reference)
            .collect::<Vec<_>>();
        assert_eq!(references.len(), outer_targets.len());
        for (reference, &is_outer) in references.iter().zip(outer_targets) {
            let target = reference.symbol.unwrap();
            if is_outer {
                assert_eq!(target, outer);
            } else {
                assert_ne!(target, outer);
                assert!(
                    semantic
                        .symbol(target)
                        .flags
                        .intersects(SymbolFlags::PARAMETER | SymbolFlags::TYPE_PARAMETER)
                );
            }
            assert_eq!(
                semantic.lookup(reference.scope, name, reference.flags.space),
                Some(target)
            );
            assert!(!reference.flags.is_read());
            assert!(!reference.flags.is_write());
        }
    }
}

#[test]
fn parameter_annotation_visibility() {
    assert_type_name_targets(
        "export {}; type T = string; function f(x: T) { type T = number; }",
        b"T",
        &[true],
    );

    assert_type_name_targets(
        "export {}; type T = string; function f(x: T = '') { type T = number; }",
        b"T",
        &[true],
    );

    assert_type_name_targets(
        "export {}; const value = 'outer'; function f(x: typeof value) { var value = 1; }",
        b"value",
        &[true],
    );

    assert_type_name_targets(
        "export {}; const value = 'outer'; function f(x: typeof value = 'outer') { var value = 1; }",
        b"value",
        &[true],
    );

    assert_type_name_targets(
        "export {}; type T = string; function f<T>(x: T) {}",
        b"T",
        &[false],
    );

    assert_type_name_targets(
        "export {}; const value = 'outer'; function f(value: number, x: typeof value) {}",
        b"value",
        &[false],
    );

    assert_type_name_targets(
        "export {}; type T = { x: string }; function f({ x }: T) { type T = number; }",
        b"T",
        &[true],
    );
}

#[test]
fn infer_visibility() {
    assert_type_name_targets(
        "export {}; type U = number; type R<T> = T extends [infer U, U] ? U : U; const ok: R<[string, number]> = 'ok';",
        b"U",
        &[true, false, true],
    );

    assert_type_name_targets(
        "export {}; type U = number; type R<T> = T extends [infer U extends { value: U }, U] ? U : U;",
        b"U",
        &[false, true, false, true],
    );

    assert_type_name_targets(
        "export {}; type U = number; type R<T> = T extends [infer U, T extends [infer V, U] ? U : U] ? U : U;",
        b"U",
        &[true, true, true, false, true],
    );

    assert_type_name_targets(
        "export {}; type U = number; type R<T> = T extends infer U ? U extends [infer V, U] ? U : U : U;",
        b"U",
        &[false, false, false, false, true],
    );
}

#[test]
fn nested_infer_visibility() {
    let ts_module = ts_module_options();

    assert_named_declaration_targets(
        "type U = string; type R<T> = T extends (true extends true ? infer U : never) ? U : never;",
        ts_module,
        b"U",
        &[Some("U : never)")],
    );

    assert_named_declaration_targets(
        "type R<T> = T extends (true extends true ? infer U : never) ? U : never;",
        ts_module,
        b"U",
        &[Some("U : never)")],
    );

    assert_named_declaration_targets(
        "type U = string; type R<T> = T extends (true extends true ? [infer U, U] : never) ? U : never;",
        ts_module,
        b"U",
        &[Some("U = string"), Some("U, U]")],
    );

    assert_named_declaration_targets(
        "type U = string; type R<T> = T extends (infer U extends unknown ? never : U) ? U : never;",
        ts_module,
        b"U",
        &[Some("U = string"), Some("U extends unknown")],
    );

    assert_named_declaration_targets(
        "type U = string; type R<T> = T extends (true extends true ? infer U : U) ? U : never;",
        ts_module,
        b"U",
        &[Some("U = string"), Some("U : U)")],
    );

    assert_named_declaration_targets(
        "type U = string; type R<T> = T extends (false extends true ? never : (true extends true ? infer U : never)) ? U : never;",
        ts_module,
        b"U",
        &[Some("U : never))")],
    );

    assert_named_declaration_targets(
        "type U = string; type R<T> = T extends (<V>() => true extends true ? infer U : never) ? U : never;",
        ts_module,
        b"U",
        &[Some("U : never)")],
    );

    assert_named_declaration_targets(
        "type U = string; type R<T> = T extends { [K in 'x']: true extends true ? infer U : never } ? U : never;",
        ts_module,
        b"U",
        &[Some("U : never }")],
    );
}

#[test]
fn infer_declaration_merging() {
    let source = "type R<T> = T extends (boolean extends true ? infer U : infer U) ? U : never;";
    let expected = source
        .match_indices("infer U")
        .map(|(start, _)| start as u32 + 6)
        .collect::<Vec<_>>();
    let allocator = Allocator::new();
    for preserve_parens in [false, true] {
        let parser_options = Options {
            preserve_parens,
            ..ts_module_options()
        };
        for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
            let options = AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            };
            let (ast, semantic) = analyze_with_options(&allocator, source, parser_options, options);
            let entry = semantic
                .iter_references()
                .find(|entry| ast.get_utf8(entry.reference.name) == "U")
                .unwrap();
            let symbol = entry.reference.symbol.unwrap();
            let declarations = semantic
                .declarations(symbol)
                .map(|declaration| ast.span(declaration.node).start)
                .collect::<Vec<_>>();
            assert_eq!(declarations, expected);
            assert_eq!(
                entry.reference.resolution(),
                NameResolution::Resolved(symbol)
            );
            assert_eq!(
                semantic.reference_candidates(entry.id).collect::<Vec<_>>(),
                [symbol]
            );
            assert_eq!(semantic.uses(symbol), [entry.id]);
            assert!(!entry.reference.flags.is_read());
            assert!(!entry.reference.flags.is_write());
        }
    }
}

#[test]
fn signature_declaration_visibility() {
    let ts_module = ts_module_options();

    assert_named_declaration_targets(
        "const value = 'outer'; function f(x: typeof value) { const value = 123; }",
        ts_module,
        b"value",
        &[Some("value = 'outer'")],
    );

    assert_named_declaration_targets(
        "const value = 'outer'; function f(x: typeof value) { let value = 123; }",
        ts_module,
        b"value",
        &[Some("value = 'outer'")],
    );

    assert_named_declaration_targets(
        "const value = 'outer'; function f(): typeof value { class value {} return 'outer'; }",
        ts_module,
        b"value",
        &[Some("value = 'outer'")],
    );

    assert_named_declaration_targets(
        "const value = 'outer'; function f(x = 0): typeof value { enum value { A } return 'outer'; }",
        ts_module,
        b"value",
        &[Some("value = 'outer'")],
    );

    assert_named_declaration_targets(
        "function f(): typeof value { const value = 123; return value; }",
        ts_module,
        b"value",
        &[Some("value = 123"), Some("value = 123")],
    );

    assert_named_declaration_targets(
        "function f(x = 0): typeof value { const value = 123; return value; }",
        ts_module,
        b"value",
        &[Some("value = 123"), Some("value = 123")],
    );

    assert_named_declaration_targets(
        "function f<T extends typeof value>(x = 0) { let value = 123; }",
        ts_module,
        b"value",
        &[Some("value = 123")],
    );

    assert_named_declaration_targets(
        "const value = 1; function f(x = 0): typeof value { function value() {} return value; }",
        ts_module,
        b"value",
        &[Some("value() {}"), Some("value() {}")],
    );

    assert_named_declaration_targets(
        "function f(x: typeof value = () => {}) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value() {}")],
    );

    assert_named_declaration_targets(
        "function f(): typeof value { function value() {} interface value {} return value; }",
        ts_module,
        b"value",
        &[Some("value() {}"), Some("value() {}")],
    );
}

#[test]
fn parameter_annotations_exclude_body_types() {
    let ts_module = ts_module_options();

    assert_named_declaration_targets(
        "type T = string; function f(...args: T[]) { type T = number; }",
        ts_module,
        b"T",
        &[Some("T = string")],
    );

    assert_named_declaration_targets(
        "type T = string; function f(this: T) { type T = number; }",
        ts_module,
        b"T",
        &[Some("T = string")],
    );

    assert_named_declaration_targets(
        "class T {} const f = (...args: T[]) => { class T {} };",
        ts_module,
        b"T",
        &[Some("T {} const")],
    );

    assert_named_declaration_targets(
        "enum T { A } class C { m(...args: T[]) { enum T { B } } }",
        ts_module,
        b"T",
        &[Some("T { A }")],
    );

    assert_named_declaration_targets(
        "const value = 1; function f(...args: (typeof value)[]) { const value = 2; }",
        ts_module,
        b"value",
        &[Some("value = 1")],
    );

    assert_named_declaration_targets(
        "const value = 1; function f(this: typeof value) { var value = 2; }",
        ts_module,
        b"value",
        &[Some("value = 1")],
    );

    assert_named_declaration_targets(
        "type T = string; function f(...[arg]: T[]) { type T = number; }",
        ts_module,
        b"T",
        &[Some("T = string")],
    );
}

#[test]
fn parameter_type_queries_can_see_body_functions() {
    let ts_module = ts_module_options();

    assert_named_declaration_targets(
        "function f(...args: (typeof value)[]) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value() {}")],
    );

    assert_named_declaration_targets(
        "function f(x = 0, ...args: (typeof value)[]) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value() {}")],
    );

    assert_named_declaration_targets(
        "function f(this: typeof value, x = 0) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value() {}")],
    );

    assert_named_declaration_targets(
        "function f(arg = undefined as unknown as typeof value) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value() {}")],
    );

    assert_named_declaration_targets(
        "function f({ [('arg' as keyof typeof value)]: arg }: any) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value() {}")],
    );
}

#[test]
fn nested_parameter_expressions_keep_their_own_scope() {
    let ts_module = ts_module_options();

    assert_named_declaration_targets(
        "function f(arg = () => undefined as unknown as typeof value) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value() {}")],
    );

    assert_named_declaration_targets(
        "type T = string; function f(arg = function (...args: T[]) { type T = number; }) {}",
        ts_module,
        b"T",
        &[Some("T = string")],
    );

    assert_named_declaration_targets(
        "function f(arg = () => { const value = 1; return undefined as unknown as typeof value; }) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value = 1")],
    );

    assert_named_declaration_targets(
        "const value = 1; function f(arg = () => value) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value = 1")],
    );

    assert_named_declaration_targets(
        "const value = 1; function f({ [value]: arg }: any) { function value() {} }",
        ts_module,
        b"value",
        &[Some("value = 1")],
    );
}

#[test]
fn arguments_lookup_visibility() {
    let script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };

    assert_named_declaration_targets(
        "const f = function arguments() { return arguments; };",
        script,
        b"arguments",
        &[None],
    );

    assert_named_declaration_targets(
        "const f = function arguments(a = arguments) { return arguments; };",
        script,
        b"arguments",
        &[None, None],
    );

    assert_named_declaration_targets(
        "const f = function arguments(arguments) { return arguments; };",
        script,
        b"arguments",
        &[Some("arguments) {")],
    );

    assert_named_declaration_targets(
        "var arguments; function f() { return arguments; }",
        script,
        b"arguments",
        &[None],
    );

    assert_named_declaration_targets(
        "var arguments; const f = () => arguments;",
        script,
        b"arguments",
        &[Some("arguments; const f")],
    );

    assert_named_declaration_targets(
        "var arguments; function f() { return () => arguments; }",
        script,
        b"arguments",
        &[None],
    );

    assert_named_declaration_targets(
        "var arguments; function f(x = arguments) { return arguments; }",
        script,
        b"arguments",
        &[None, None],
    );

    assert_named_declaration_targets(
        "var arguments; function f() { let arguments; return arguments; }",
        script,
        b"arguments",
        &[Some("arguments; return")],
    );
}

#[test]
fn arguments_ownership() {
    let script = Options {
        lang: Lang::Ts,
        source_type: SourceType::Script,
        ..Options::default()
    };

    assert_named_declaration_targets(
        "declare const arguments: { outer: true }; declare function f(): typeof arguments;",
        script,
        b"arguments",
        &[None],
    );

    assert_named_declaration_targets(
        "declare const arguments: { outer: true }; declare function f(x: typeof arguments): void;",
        script,
        b"arguments",
        &[None],
    );

    assert_named_declaration_targets(
        "declare const arguments: { outer: true }; declare abstract class C { abstract f(): typeof arguments; }",
        script,
        b"arguments",
        &[None],
    );

    assert_named_declaration_targets(
        "declare const arguments: { outer: true }; type F = () => typeof arguments;",
        script,
        b"arguments",
        &[Some("arguments:")],
    );

    assert_named_declaration_targets(
        "declare const arguments: { outer: true }; interface I { f(): typeof arguments; }",
        script,
        b"arguments",
        &[Some("arguments:")],
    );
}

#[test]
fn parameter_initializer_environment() {
    let script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };
    assert_named_declaration_targets(
        "const value = 1; function f(x = value) { function value() {} return value; }",
        script,
        b"value",
        &[Some("value = 1"), Some("value() {}")],
    );
}

#[track_caller]
fn assert_signature_references(source: &str, name: &[u8], parameters: &[bool]) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let (ast, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let parameter = semantic
            .iter_symbols()
            .find(|entry| {
                ast.get_wtf8(entry.symbol.name).as_bytes() == name
                    && entry.symbol.flags.contains(SymbolFlags::PARAMETER)
            })
            .unwrap()
            .id;
        let outer = semantic.binding(ScopeId::MODULE, name);
        let references = semantic
            .iter_references()
            .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name)
            .map(|entry| entry.reference)
            .collect::<Vec<_>>();
        assert_eq!(references.len(), parameters.len());
        for (reference, &is_parameter) in references.iter().zip(parameters) {
            let expected = if is_parameter { Some(parameter) } else { outer };
            assert_eq!(reference.symbol, expected);
            assert_eq!(
                semantic.lookup(reference.scope, name, reference.flags.space),
                expected
            );
            assert!(!reference.flags.is_read());
            assert!(!reference.flags.is_write());
        }
    }
}

#[track_caller]
fn assert_header_value_query(source: &str, outer: bool) {
    let allocator = Allocator::new();
    for check_syntax in [SyntaxChecks::empty(), SyntaxChecks::all()] {
        let (_, semantic) = analyze_with_options(
            &allocator,
            source,
            ts_module_options(),
            AnalyzeOptions {
                check_syntax,
                ..AnalyzeOptions::default()
            },
        );
        let query = semantic
            .iter_references()
            .find(|entry| entry.reference.flags.space == ReferenceSpace::Typeof)
            .unwrap()
            .reference;
        let target = query.symbol.unwrap();
        assert_eq!(
            semantic.lookup(query.scope, b"value", ReferenceSpace::Typeof),
            Some(target)
        );
        assert!(!query.flags.is_read());
        assert!(!query.flags.is_write());
        if outer {
            assert_eq!(Some(target), semantic.binding(ScopeId::MODULE, b"value"));
        } else {
            assert!(
                semantic
                    .symbol(target)
                    .flags
                    .contains(SymbolFlags::FUNCTION)
            );
        }
    }
}

#[test]
fn function_header_visibility() {
    assert_header_value_query(
        "export {}; const value = 'outer'; function f(): typeof value { var value = 1; return 'outer'; }",
        true,
    );

    assert_header_value_query(
        "export {}; function f(): typeof value { function value() {} return value; }",
        false,
    );

    assert_header_value_query(
        "export {}; function f<T extends typeof value>() { function value() {} }",
        false,
    );
}

#[test]
fn signature_bindings() {
    assert_signature_references(
        "export {}; const value = 'outer'; type F = (value: number) => typeof value;\n",
        b"value",
        &[true],
    );

    assert_signature_references(
        "export {}; type Guard = (value: unknown) => value is string;\n",
        b"value",
        &[true],
    );

    assert_signature_references(
        "export {}; const value = 'outer'; interface I { (value: number): typeof value; }\n",
        b"value",
        &[true],
    );

    assert_signature_references(
        "export {}; const value = 'outer'; interface I { method(value: number): typeof value; }\n",
        b"value",
        &[true],
    );

    assert_signature_references(
        "export {}; const value = 'outer'; interface I { new (value: number): { value: typeof value }; }\n",
        b"value",
        &[true],
    );

    assert_signature_references(
        "export {}; const value = 'outer'; type I = new (value: number) => { value: typeof value };\n",
        b"value",
        &[true],
    );

    assert_signature_references(
        "export {}; const value = 'outer'; interface I { [value: string]: typeof value; }\n",
        b"value",
        &[true],
    );

    assert_signature_references(
        "export {}; const value = 'outer'; type F = <T extends typeof value>(value: number) => T;\n",
        b"value",
        &[false],
    );

    assert_signature_references(
        "export {}; const value = 'outer'; function f<T extends typeof value>(value: number): T { throw 0; }\n",
        b"value",
        &[false],
    );

    assert_signature_references(
        "export {}; const key = 'method'; interface I { [key](key: number): typeof key; }\n",
        b"key",
        &[false, true],
    );
}
