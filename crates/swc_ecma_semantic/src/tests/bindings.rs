use super::*;

#[test]
fn canonical_names() {
    let allocator = Allocator::new();
    let script = Options {
        source_type: SourceType::Script,
        ..Options::default()
    };

    let (ast, semantic) = analyze_source(&allocator, "let value; v\\u0061lue;", script);
    let symbol = semantic.symbol(symbol_named(&ast, &semantic, b"value"));
    let reference = semantic.iter_references().next().unwrap().reference;
    assert_eq!(symbol.name.as_utf8(), Some(reference.name));

    let (ast, semantic) = analyze_source(&allocator, r#"enum E { "\ud800" }"#, ts_module_options());
    let member = semantic
        .iter_symbols()
        .find(|entry| entry.symbol.flags.contains(SymbolFlags::ENUM_MEMBER))
        .unwrap();
    assert!(matches!(member.symbol.name, SymbolName::Wtf8(_)));
    assert_eq!(ast.get_wtf8(member.symbol.name).as_bytes(), b"\xed\xa0\x80");
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
    let (ast, semantic) = analyze_with_options(&allocator, source, script, checked);
    let name = symbol_named(&ast, &semantic, b"f");
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
    assert!(semantic.binding(ScopeId::ROOT, b"f").is_none());

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
