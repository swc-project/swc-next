use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParseOptions, Parser};

#[track_caller]
pub fn expect_printed(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, false, false);
}

#[track_caller]
pub fn expect_printed_minify(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_printed_ts(source: &[u8], expected: &[u8]) {
    expect_printed_common(source, expected, Lang::Ts, SourceType::Module, false, false);
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
        comments: CommentMode::Attached,
    };
    let result = Parser::init(&mut ast, parse_options, NoTokenParserConfig).parse();
    let has_errors = result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == swc_next_ecma_ast::Severity::Error);
    if has_errors {
        return;
    }
    let program = result.program;

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
