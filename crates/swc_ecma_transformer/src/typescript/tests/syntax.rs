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

    let mut ast = parsed.ast;
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let program = ast.root_program();
    Normalizer::normalize(program, &mut ast);
    transform(program, &mut ast, &semantic, TypeScriptOptions::default());

    assert!(program.body(&ast).is_empty());
    assert!(ast.comments().is_empty());
}
