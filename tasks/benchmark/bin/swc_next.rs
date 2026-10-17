use std::{fs, hint::black_box};

use benchmark::{Args, Operation};
use clap::Parser as _;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};

fn main() {
    let args = Args::parse();
    let source = fs::read_to_string(&args.file).expect("failed to read input");
    let options = Options {
        lang: Lang::from_path(&args.file),
        source_type: SourceType::Module,
        comments: CommentMode::None,
        ..Options::default()
    };

    for _ in 0..args.iterations.get() {
        if args.operation == Operation::TypeStrip {
            let result = swc_next_type_strip::strip_types(&source, options);
            assert!(result.diagnostics.is_empty(), "type stripping failed");
            black_box(result);
            continue;
        }

        let allocator = Allocator::default();
        let parsed = Parser::init(&allocator, &source, options, NoTokenParserConfig).parse();
        assert!(parsed.diagnostics.is_empty(), "parse failed");
        let mut ast = parsed.ast;
        if args.operation == Operation::IsolatedDeclaration {
            let semantic = swc_next_ecma_semantic::analyze(&ast, Default::default()).semantic;
            let diagnostics = swc_next_isolated_declarations::transform(&mut ast, &semantic);
            assert!(diagnostics.is_empty(), "isolated declaration diagnostics");
            black_box(swc_next_ecma_codegen::print(
                &allocator,
                &ast,
                Default::default(),
            ));
        }
        black_box(ast);
    }
}
