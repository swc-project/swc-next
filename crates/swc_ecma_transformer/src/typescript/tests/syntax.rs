use super::*;

#[test]
fn declaration_files() {
    let allocator = Allocator::new();
    let source = "/*! license */ declare const value: number;";
    let options = ParserOptions {
        source_type: SourceType::Module,
        lang: Lang::Dts,
        ..ParserOptions::default()
    };
    let parser = Parser::init(&allocator, source, options, NoTokenParserConfig);
    let parsed = parser.parse();
    assert!(parsed.diagnostics.is_empty());

    let mut ctx = parsed.ast;
    Normalizer::normalize(&mut ctx);
    let analyzed = analyze(&mut ctx, AnalyzeOptions::default()).semantic;
    let mut transformer = MultiPassTransformer::new(analyzed);
    transformer.typescript(&TypeScriptOptions::default());
    // SAFETY: The inspected root belongs to the transformed AST.
    let ast = unsafe { transformer.ctx.ast() };

    assert!(ast.root_program().body(ast).is_empty());
    assert!(ast.comments().is_empty());
}
