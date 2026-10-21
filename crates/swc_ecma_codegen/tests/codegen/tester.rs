use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParseOptions, Parser};

#[track_caller]
pub fn expect_printed_ts_pair(source: &str, pretty: &str, minified: &str) {
    let options = ParseOptions {
        lang: Lang::Ts,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    expect_printed_pair(source, options, pretty, minified);
}

#[track_caller]
pub fn expect_printed_pair(source: &str, options: ParseOptions, pretty: &str, minified: &str) {
    let allocator = Allocator::new();
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let output = print(&parsed.ast, PrintOptions::default());
    assert_eq!(output.code, pretty);

    let output = print(
        &parsed.ast,
        PrintOptions {
            minify: true,
            ..PrintOptions::default()
        },
    );
    assert_eq!(output.code, minified);
}

#[track_caller]
pub fn expect_printed(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, false, false);
}

#[track_caller]
pub fn expect_printed_minify(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_printed_ts(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Ts, SourceType::Module, false, false);
}

#[track_caller]
pub fn expect_printed_ts_minify(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Ts, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_printed_jsx(source: &str, expected: &str) {
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
pub fn expect_printed_jsx_minify(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Jsx, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_printed_ascii(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, false, true);
}

#[track_caller]
pub fn expect_printed_minify_ascii(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, true, true);
}

#[track_caller]
pub fn expect_printed_jsx_ascii(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Jsx, SourceType::Module, false, true);
}

#[track_caller]
pub fn expect_printed_script(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Script, false, false);
}

#[track_caller]
pub fn expect_printed_with_options(source: &str, expected: &str, print_options: PrintOptions) {
    expect_printed_common_with_options(
        source,
        expected,
        Lang::Js,
        SourceType::Module,
        print_options,
    );
}

#[track_caller]
pub fn expect_stable(source: &str, expected: &str) {
    expect_printed(source, expected);
    expect_printed(expected, expected);
}

#[track_caller]
fn expect_printed_common(
    source: &str,
    expected: &str,
    lang: Lang,
    source_type: SourceType,
    minify: bool,
    ascii_only: bool,
) {
    let print_options = PrintOptions {
        minify,
        ascii_only,
        ..PrintOptions::default()
    };
    expect_printed_common_with_options(source, expected, lang, source_type, print_options);
}

#[track_caller]
fn expect_printed_common_with_options(
    source: &str,
    expected: &str,
    lang: Lang,
    source_type: SourceType,
    print_options: PrintOptions,
) {
    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        source_type,
        lang,
        preserve_parens: false,
        comments: CommentMode::Flat,
    };
    let result = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    let has_errors = result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == swc_next_ecma_ast::Severity::Error);
    if has_errors {
        return;
    }
    let result = print(&result.ast, print_options);

    assert_eq!(
        result.code, expected,
        "failed for {}\nactual:\n{}\nexpected:\n{}",
        source, result.code, expected,
    );
}
