use std::{env, fs, hint::black_box};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "usage: swc-next <file.ts> <parse|isolated-declaration|type-strip>"
    );
    let operation = args[2].as_str();
    assert!(matches!(
        operation,
        "parse" | "isolated-declaration" | "type-strip"
    ));
    let source = fs::read_to_string(&args[1]).expect("failed to read input");
    let options = Options {
        lang: Lang::from_path(&args[1]),
        source_type: SourceType::Module,
        comments: CommentMode::None,
        ..Options::default()
    };

    if operation == "type-strip" {
        let result = swc_next_type_strip::strip_types(&source, options);
        assert!(result.diagnostics.is_empty(), "type stripping failed");
        black_box(result);
        return;
    }

    let allocator = Allocator::default();
    let parsed = Parser::init(&allocator, &source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty(), "parse failed");
    let mut ast = parsed.ast;
    if operation == "isolated-declaration" {
        let semantic = swc_next_ecma_semantic::analyze(&ast, Default::default()).semantic;
        black_box(swc_next_isolated_declarations::transform(
            &mut ast, &semantic,
        ));
        black_box(swc_next_ecma_codegen::print(
            &allocator,
            &ast,
            Default::default(),
        ));
    }
    black_box(ast);
}
