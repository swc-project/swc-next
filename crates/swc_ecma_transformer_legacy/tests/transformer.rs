use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{AstCtx, Severity};
use swc_next_ecma_codegen::generate;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_transformer_legacy::{Transformer, TransformerOptions};

fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> AstCtx<'a> {
    let parsed = Parser::init(allocator, source, Options::default(), NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    parsed.ast
}

#[test]
fn ast_entry() {
    let allocator = Allocator::new();
    let mut ast = parse(&allocator, "const value = 1;");
    let before = generate(&ast, Default::default()).code;
    let result = Transformer::transform(&mut ast, TransformerOptions::default());
    assert!(result.diagnostics.is_empty());
    assert_eq!(generate(&ast, Default::default()).code, before);
}

#[test]
fn pending_transforms() {
    let allocator = Allocator::new();
    let mut ast = parse(&allocator, "2 ** 3;");
    let before = generate(&ast, Default::default()).code;
    let mut options = TransformerOptions::default();
    options.env.es2016.exponentiation = true;
    let result = Transformer::transform(&mut ast, options);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    );
    assert_eq!(generate(&ast, Default::default()).code, before);
}
