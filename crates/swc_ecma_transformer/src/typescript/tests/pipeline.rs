use super::*;

fn with_static_blocks(source: &str, prevent_extensions: bool, check: impl FnOnce(&str)) {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            lang: Lang::Ts,
            source_type: SourceType::Module,
            ..Default::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let scopes = crate::Transformer::transform(
        &mut ast,
        semantic,
        crate::TransformerOptions {
            typescript: Some(TypeScriptOptions::default()),
            es2022: crate::es2022::ES2022Options {
                static_blocks: true,
            },
            proposal: crate::proposal::ProposalOptions { prevent_extensions },
            ..Default::default()
        },
    );
    let fresh = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: true,
            ..Default::default()
        },
    );
    assert!(fresh.diagnostics.is_empty());
    for entry in fresh.semantic.iter_references() {
        let reference = scopes.reference(scopes.reference_of(entry.reference.node).unwrap());
        assert_eq!(
            scopes.scope(reference.scope).node,
            fresh.semantic.scope(entry.reference.scope).node
        );
        assert_eq!(reference.flags, entry.reference.flags);
        assert_eq!(
            reference
                .symbol
                .map(|id| scopes.symbol(id).declaration)
                .map(|id| scopes.declaration(id).node),
            entry
                .reference
                .symbol
                .map(|id| fresh.semantic.symbol(id).declaration)
                .map(|id| fresh.semantic.declaration(id).node)
        );
    }
    let code = swc_next_ecma_codegen::print(&allocator, &ast, Default::default()).code;
    check(code);
}

#[test]
fn namespace_static_blocks() {
    with_static_blocks(
        "namespace N { export class C { static { const value: number = 1; use(value); } } }",
        false,
        |code| {
            assert!(code.contains("static #_"));
            assert!(!code.contains("namespace"));
            assert!(!code.contains(": number"));
        },
    );
}

#[test]
fn class_expression_static_blocks() {
    with_static_blocks(
        "enum E { Value = 1 } export const C = class { static { const value: E = E.Value; use(value); } };",
        true,
        |code| {
            assert!(!code.contains("static {"));
            assert!(!code.contains("enum"));
            assert!(!code.contains(": E"));
            assert!(code.contains("use(value)"));
        },
    );
}
