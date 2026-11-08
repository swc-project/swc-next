use super::*;

#[test]
fn declaration_files() {
    let allocator = Allocator::new();
    let source = "/*! license */ declare const value: number;";
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            source_type: SourceType::Module,
            lang: Lang::Dts,
            comments: CommentMode::Flat,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);

    let mut ctx = parsed.ast;
    Normalizer::normalize(&mut ctx);
    let analyzed = analyze(&mut ctx, AnalyzeOptions::default()).semantic;
    let transformed = transform(analyzed, &TypeScriptOptions::default());
    // SAFETY: The inspected root belongs to the transformed AST.
    let ast = unsafe { transformed.ast() };

    assert!(ast.root_program().body(ast).is_empty());
    assert!(ast.comments().is_empty());
}
