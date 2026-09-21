use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

use crate::{Transformer, TransformerOptions};

fn transform_options(
    source: &str,
    source_type: SourceType,
    lang: Lang,
    options: TransformerOptions,
) -> (String, Vec<swc_next_ecma_ast::Diagnostic<'static>>) {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            source_type,
            lang,
            comments: CommentMode::Flat,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let transformed = Transformer::transform(&mut ast, &semantic, options);
    assert_eq!(ast.lang(), lang);
    let output = print(&allocator, &ast, CodegenOptions::default())
        .code
        .to_owned();
    (output, transformed.diagnostics)
}

#[test]
fn generated_bindings() {
    let options = TransformerOptions {
        es2015: crate::es2015::ES2015Options {
            arrow_functions: Some(crate::es2015::arrow::ArrowFunctionsOptions::default()),
            ..crate::es2015::ES2015Options::default()
        },
        ..TransformerOptions::default()
    };
    let (output, diagnostics) = transform_options(
        "const _jsxFileName = 'source'; const getThis = () => this;",
        SourceType::Script,
        Lang::Js,
        options,
    );
    assert!(diagnostics.is_empty());
    assert!(output.starts_with("var _this = this;\nconst _jsxFileName = \"source\";"));
}

#[test]
fn development_lexical_environments() {
    let options = TransformerOptions {
        react_jsx_self: true,
        ..TransformerOptions::default()
    };
    let (output, diagnostics) = transform_options(
        "class D extends B { constructor() { class X extends (<div />, B) {} super(); } }",
        SourceType::Module,
        Lang::Jsx,
        options,
    );
    assert!(diagnostics.is_empty());
    assert!(!output.contains("__self"));
}

#[test]
fn development_source_positions() {
    let options = TransformerOptions {
        react_jsx_source: true,
        ..TransformerOptions::default()
    };
    let (output, diagnostics) =
        transform_options("'😀'; <div />;", SourceType::Module, Lang::Jsx, options);
    assert!(diagnostics.is_empty());
    assert!(output.contains("lineNumber: 1, columnNumber: 7"));
}
