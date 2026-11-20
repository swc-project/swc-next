use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{AstCtx, Diagnostic, Lang, Severity, SourceType};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, ScopeModel, SyntaxChecks, analyze};

use swc_next_ecma_transformer::{
    HelperMode, Transformer, TransformerOptions, es2022::ES2022Options, normalize::Normalizer,
    proposal::ProposalOptions,
};

fn inline(source: &str, source_type: SourceType) -> String {
    let (code, diagnostics) =
        transform_with_helpers(source, source_type, Lang::Js, HelperMode::Inline);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    code
}

fn transform_with_helpers(
    source: &str,
    source_type: SourceType,
    lang: Lang,
    helpers: HelperMode,
) -> (String, Vec<Diagnostic>) {
    transform_program(source, source_type, lang, helpers, true)
}

fn transform_without_normalizer(
    source: &str,
    source_type: SourceType,
    lang: Lang,
    helpers: HelperMode,
) -> (String, Vec<Diagnostic>) {
    transform_program(source, source_type, lang, helpers, false)
}

fn transform_program(
    source: &str,
    source_type: SourceType,
    lang: Lang,
    helpers: HelperMode,
    normalize: bool,
) -> (String, Vec<Diagnostic>) {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            source_type,
            lang,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    if normalize {
        Normalizer::normalize(&mut ast);
    }
    let analyzed = analyze(&mut ast, AnalyzeOptions::default());
    assert!(analyzed.diagnostics.is_empty());
    let original_references: Vec<_> = analyzed
        .semantic
        .semantic()
        .iter_references()
        .map(|entry| (entry.id, entry.reference.node, entry.reference.symbol))
        .collect();
    let transformed = Transformer::transform(
        analyzed.semantic,
        TransformerOptions {
            helpers,
            es2022: ES2022Options {
                static_blocks: true,
            },
            proposal: ProposalOptions {
                prevent_extensions: true,
            },
            ..TransformerOptions::default()
        },
    );
    for (id, node, symbol) in original_references {
        assert_eq!(transformed.semantic.reference_of(node), Some(id));
        assert_eq!(transformed.semantic.reference(id).symbol, symbol);
    }
    assert_semantic(&ast, &transformed.semantic);
    (
        swc_next_ecma_codegen::generate(&ast, Default::default()).code,
        transformed.diagnostics,
    )
}

fn assert_semantic(ast: &AstCtx<'_>, semantic: &ScopeModel<'_>) {
    let analyzed = analyze(
        ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
        },
    );
    assert!(
        analyzed.diagnostics.is_empty(),
        "{:?}",
        analyzed.diagnostics
    );
    let expected_semantic = analyzed.semantic.semantic();
    // SAFETY: Both semantic models refer to this AST.
    let ast = unsafe { ast.ast() };
    for entry in expected_semantic.iter_declarations() {
        let expected = entry.declaration;
        // Annex B companion declarations share the lexical binding node.
        if expected_semantic.symbol_of(expected.node) != Some(expected.symbol) {
            continue;
        }
        let symbol = semantic
            .symbol_of(expected.node)
            .expect("registered binding");
        let actual = semantic
            .declarations(symbol)
            .find(|declaration| declaration.node == expected.node)
            .expect("registered declaration");
        assert_eq!(actual.flags, expected.flags);
        assert_eq!(
            semantic.scope(actual.scope).node,
            expected_semantic.scope(expected.scope).node,
        );
        assert_eq!(
            semantic.symbol(symbol).flags,
            expected_semantic.symbol(expected.symbol).flags,
        );
    }
    for entry in expected_semantic.iter_references() {
        let expected = entry.reference;
        let id = semantic
            .reference_of(expected.node)
            .expect("registered reference");
        let actual = semantic.reference(id);
        assert_eq!(ast.get_utf8(actual.name), ast.get_utf8(expected.name));
        assert_eq!(
            actual
                .symbol
                .map(|id| semantic.symbol(id).declaration)
                .map(|id| semantic.declaration(id).node),
            expected
                .symbol
                .map(|id| expected_semantic.symbol(id).declaration)
                .map(|id| expected_semantic.declaration(id).node)
        );
        assert_eq!(
            semantic.scope(actual.scope).node,
            expected_semantic.scope(expected.scope).node
        );
        assert_eq!(actual.flags.is_read(), expected.flags.is_read());
        assert_eq!(actual.flags.is_write(), expected.flags.is_write());
        if let Some(symbol) = actual.symbol {
            assert!(semantic.uses(symbol).contains(&id));
        }
    }
    for entry in expected_semantic.iter_scopes() {
        if entry.scope.parent.is_none() {
            continue;
        }
        let id = semantic
            .scope_of(entry.scope.node)
            .expect("registered scope");
        let actual = semantic.scope(id);
        assert_eq!(actual.kind, entry.scope.kind);
        assert_eq!(actual.flags.strict, entry.scope.flags.strict);
        assert_eq!(
            actual.flags.arguments_barrier,
            entry.scope.flags.arguments_barrier
        );
        assert_eq!(
            actual.parent.map(|id| semantic.scope(id).node),
            entry
                .scope
                .parent
                .map(|id| expected_semantic.scope(id).node)
        );
    }
}

#[test]
fn inline_helpers() {
    for source_type in [SourceType::Module, SourceType::Script, SourceType::CommonJs] {
        let code = inline(
            "const obj = { [key()]: class { static { init(); } } };",
            source_type,
        );
        assert!(!code.contains("@swc-next/helpers"));
        assert!(code.contains("function _set_function_name("));
        assert!(code.contains("function _to_property_key("));
        assert!(code.contains("function _to_primitive("));
        assert!(code.contains("function _type_of("));
        assert!(code.contains("_to_property_key(key())"));

        let code = inline("const C = class { static { init(); } };", source_type);
        assert!(code.contains("function _set_function_name("));
        assert!(!code.contains("function _to_property_key("));

        let code = inline("class C {}", source_type);
        assert!(!code.contains("function _"));

        let code = inline(
            "const obj = { [first()]: class { static { init(); } }, [second()]: class { static { init(); } } };",
            source_type,
        );
        assert_eq!(code.matches("function _type_of(").count(), 1);
        assert_eq!(code.matches("function _set_function_name(").count(), 1);
    }
}

#[test]
fn inline_helper_bindings() {
    let code = inline(
        "import Object from './shadow'; const C = class { static { use(Object); } };",
        SourceType::Module,
    );
    assert!(code.contains("import _Object from"));
    assert!(code.contains("Object.defineProperty"));
    assert!(code.contains("use(_Object)"));

    let code = inline(
        "function run(_to_primitive, _type_of, _to_property_key, _set_function_name) { return { [key()]: class { static { init(); } } }; }",
        SourceType::Module,
    );
    assert!(code.contains("function _to_primitive2("));
    assert!(code.contains("function _type_of2("));
    assert!(code.contains("_to_primitive2(arg, \"string\")"));

    let code = inline(
        "const Object = other; const C = class { static { use(Object); } };",
        SourceType::Module,
    );
    assert!(code.contains("const _Object = other;"));
    assert!(code.contains("Object.defineProperty"));
    assert!(code.contains("use(_Object)"));

    let code = inline(
        "const Symbol = other; const obj = { [key()]: class { static { use(Symbol); } } };",
        SourceType::Module,
    );
    assert!(code.contains("const _Symbol = other;"));
    assert!(code.contains("input[Symbol.toPrimitive]"));

    let code = inline(
        "export const Object = other, answer = 42; export const C = class { static { use({ Object }); } };",
        SourceType::Module,
    );
    assert!(code.contains("_Object as Object"));
    assert!(code.contains(", answer"));
    assert!(code.contains("Object: _Object"));

    let code = inline(
        "let Object; ({ Object } = input); const C = class { static { use(Object); } };",
        SourceType::Module,
    );
    assert!(code.contains("({ Object: _Object } = input)"));

    let code = inline(
        "let Object; ({ Object = 7 } = input); const C = class { static { use(Object); } };",
        SourceType::Module,
    );
    assert!(code.contains("({ Object: _Object = 7 } = input)"));

    let code = inline(
        "const { Object = other } = source; export { Object }; const C = class { static { use(Object); } };",
        SourceType::Module,
    );
    assert!(code.contains("Object: _Object = other"));
    assert!(code.contains("_Object as Object"));

    let code = inline(
        "const Object = () => 1; const C = class { static { use(Object.name); } };",
        SourceType::Module,
    );
    assert!(code.contains("_set_function_name(() => 1, \"Object\")"));

    let code = inline(
        "function Object() {} const C = class { static { use(Object.name); } };",
        SourceType::Module,
    );
    assert!(code.contains("_set_function_name(_Object, \"Object\");"));

    let code = inline(
        "class Object { static value = this.name; } const C = class { static { use(Object.value); } };",
        SourceType::Module,
    );
    assert!(code.contains("_set_class_name(this, \"Object\")"));

    let code = inline(
        "const Object = class { static value = this.name; }; const C = class { static { use(Object.value); } };",
        SourceType::Module,
    );
    assert!(code.contains("_set_class_name(this, \"Object\")"));
    assert!(!code.contains("_set_function_name(class"));
}

#[test]
fn inline_helper_jsx_bindings() {
    let (code, diagnostics) = transform_with_helpers(
        "const Object = () => null; const C = class { static { use(this.name); } }; <Object.Part></Object.Part>;",
        SourceType::Module,
        Lang::Jsx,
        HelperMode::Inline,
    );
    assert!(diagnostics.is_empty());
    assert!(code.contains("<_Object.Part></_Object.Part>"));
}

#[test]
fn inline_helper_name_restoration() {
    let code = inline(
        "if (true) function Object() {} const C = class { static { use(Object.name); } };",
        SourceType::CommonJs,
    );
    assert!(code.contains("function _Object() {\n  }\n  _set_function_name(_Object, \"Object\");"));

    let code = inline(
        "let Object; class Outer { static #_name = 42; static init() { Object = class { static value = Outer.#_name; }; } } Outer.init(); const C = class { static { use(Object.value); } };",
        SourceType::Module,
    );
    assert!(code.contains("static #_name2 = _set_class_name"));
    assert!(code.contains("static value = Outer.#_name"));

    let code = inline(
        "switch (0) { case 0: function Object() {} } const C = class { static { use(Object.name); } };",
        SourceType::CommonJs,
    );
    assert!(code.contains("case _set_function_name(_Object, \"Object\"), 0:"));

    let code = inline(
        "switch (1) { case 1: use(Object.name); break; case 2: function Object() {} } const C = class { static { init(); } };",
        SourceType::CommonJs,
    );
    assert!(code.contains("_set_function_name(_Object, \"Object\"), 1"));

    let code = inline(
        "switch (0) { default: function Object() {} } const C = class { static { use(Object.name); } };",
        SourceType::CommonJs,
    );
    assert!(code.contains("default:\n    _set_function_name(_Object, \"Object\");"));

    let (code, diagnostics) = transform_without_normalizer(
        "const Object = (() => 1); const C = class { static { use(Object.name); } };",
        SourceType::Module,
        Lang::Js,
        HelperMode::Inline,
    );
    assert!(diagnostics.is_empty());
    assert!(code.contains("_set_function_name((() => 1), \"Object\")"));
}

#[test]
fn inline_helper_observable_bindings() {
    let (code, diagnostics) = transform_with_helpers(
        "const Object = 1; const C = class { static { use(eval('Object')); } };",
        SourceType::Module,
        Lang::Js,
        HelperMode::Inline,
    );
    assert!(diagnostics.iter().any(|d| d.severity == Severity::Error));
    assert!(code.contains("const Object = 1;"));

    let (code, diagnostics) = transform_with_helpers(
        "let Object = 1; with (other) use(Object); const C = class { static { use(this.name); } };",
        SourceType::CommonJs,
        Lang::Js,
        HelperMode::Inline,
    );
    assert!(diagnostics.iter().any(|d| d.severity == Severity::Error));
    assert!(code.contains("use(Object)"));

    let (code, diagnostics) = transform_with_helpers(
        "var Object = 1; const C = class { static { use(globalThis.Object); } };",
        SourceType::Script,
        Lang::Js,
        HelperMode::Inline,
    );
    assert!(diagnostics.iter().any(|d| d.severity == Severity::Error));
    assert!(code.contains("var Object = 1;"));

    let code = inline(
        "const Object = 1; function f(Object) { return eval('Object'); } const C = class { static { use(Object); } };",
        SourceType::Module,
    );
    assert!(code.contains("const _Object = 1;"));
    assert!(code.contains("function f(Object)"));

    let (code, diagnostics) = transform_with_helpers(
        "let Object = 1; with (other) delete Object; const C = class { static { init(); } };",
        SourceType::CommonJs,
        Lang::Js,
        HelperMode::Inline,
    );
    assert!(diagnostics.iter().any(|d| d.severity == Severity::Error));
    assert!(code.contains("delete Object"));

    let (code, diagnostics) = transform_with_helpers(
        "const Object = 1; class Symbol { static value = this.name; } function f(Symbol) { return eval('Object'); } const obj = { [key()]: class { static { use(Object, Symbol.value); } } };",
        SourceType::Module,
        Lang::Js,
        HelperMode::Inline,
    );
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Severity::Error);
    assert!(code.contains("const Object = 1;"));
    assert!(code.contains("class _Symbol"));
}

#[test]
fn inline_helper_script_globals() {
    let (code, diagnostics) = transform_with_helpers(
        "const Object = 7; const C = class { static { init(); } }; use((0, eval)('Object'));",
        SourceType::Script,
        Lang::Js,
        HelperMode::Inline,
    );
    assert!(diagnostics.iter().any(|d| d.severity == Severity::Error));
    assert!(code.contains("const Object = 7;"));

    let (code, diagnostics) = transform_with_helpers(
        "let Object = 7; const C = class { static { init(); } };",
        SourceType::Script,
        Lang::Js,
        HelperMode::Inline,
    );
    assert!(diagnostics.iter().any(|d| d.severity == Severity::Error));
    assert!(code.contains("let Object = 7;"));

    let (code, diagnostics) = transform_with_helpers(
        "class Object {} const C = class { static { init(); } };",
        SourceType::Script,
        Lang::Js,
        HelperMode::Inline,
    );
    assert!(diagnostics.iter().any(|d| d.severity == Severity::Error));
    assert!(code.contains("class Object"));
}
