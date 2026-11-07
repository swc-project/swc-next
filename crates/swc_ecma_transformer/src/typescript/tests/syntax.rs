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
    let semantic = analyze(&ctx, AnalyzeOptions::default()).semantic;
    let program = ctx.root_program();
    Normalizer::normalize(&mut ctx);
    // SAFETY: The pass only uses handles from this AST and its fresh model.
    let ast = unsafe { ctx.ast_mut() };
    transform(program, ast, &semantic, &TypeScriptOptions::default());

    assert!(program.body(ast).is_empty());
    assert!(ast.comments().is_empty());
}
