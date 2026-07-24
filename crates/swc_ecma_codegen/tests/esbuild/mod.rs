use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{CommentMode, Options as ParseOptions, Parser};

mod js_parser_test;
mod js_printer_test;

#[track_caller]
pub fn expect_printed(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, false, false);
}

#[track_caller]
pub fn expect_printed_minify(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_printed_jsx(source: &[u8], expected: &[u8]) {
    expect_printed_common(
        source,
        expected,
        Lang::Jsx,
        SourceType::Module,
        false,
        false,
    );
}

#[track_caller]
pub fn expect_printed_jsx_minify(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Jsx, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_printed_ascii(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, false, true);
}

#[track_caller]
pub fn expect_printed_minify_ascii(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, true, true);
}

#[track_caller]
pub fn expect_printed_jsx_ascii(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Jsx, SourceType::Module, false, true);
}

#[track_caller]
pub fn expect_printed_script(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Script, false, false);
}

#[track_caller]
fn expect_printed_common(
    source: &[u8],
    expected: &[u8],
    lang: Lang,
    source_type: SourceType,
    minify: bool,
    ascii_only: bool,
) {
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let parse_options = ParseOptions {
        source_type,
        lang,
        preserve_parens: false,
        allow_return_outside_function: true,
        semantic_errors: false,
        comments: CommentMode::Attached,
    };
    let mut parser = Parser::init(&mut ast, source, parse_options);
    let Some(program) = parser.parse() else {
        panic!(
            "failed to parse {}\ndiagnostics: {:#?}",
            String::from_utf8_lossy(source),
            parser.diagnostics,
        );
    };
    let has_errors = parser
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == swc_next_ecma_ast::Severity::Error);
    drop(parser);
    if has_errors {
        return;
    }

    let print_options = PrintOptions {
        minify,
        ascii_only,
        ..PrintOptions::default()
    };
    let result = print(&allocator, &ast, program, print_options);

    assert_eq!(
        result.code,
        expected,
        "failed for {}\nactual:\n{}\nexpected:\n{}",
        String::from_utf8_lossy(source),
        String::from_utf8_lossy(result.code),
        String::from_utf8_lossy(expected),
    );
}
