use std::borrow::Cow;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::SourceType;
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Parser};
use swc_next_type_strip::{Lang, Options, Severity, Span, StripResult, strip_types};

fn options(lang: Lang) -> Options {
    Options {
        source_type: SourceType::Unambiguous,
        lang,
        preserve_parens: true,
        comments: CommentMode::None,
    }
}

#[track_caller]
fn assert_utf16_len_preserved(input: &str, output: &str) {
    assert_eq!(output.encode_utf16().count(), input.encode_utf16().count());
}

#[track_caller]
fn assert_valid_javascript(source: &str, lang: Lang) {
    let lang = if lang == Lang::Tsx {
        Lang::Jsx
    } else {
        Lang::Js
    };
    let allocator = Allocator::default();
    let parsed = Parser::init(&allocator, source, options(lang), NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
}

#[track_caller]
fn assert_strip_text<'a>(input: &'a str, expected: &str, lang: Lang) -> StripResult<'a> {
    let result = strip_types(input, options(lang));
    assert_utf16_len_preserved(input, &result.code);
    assert_eq!(result.code, expected);
    assert!(result.diagnostics.is_empty());
    result
}

#[track_caller]
fn assert_strip(input: &str, expected: &str) {
    let result = assert_strip_text(input, expected, Lang::Ts);
    assert_valid_javascript(&result.code, Lang::Ts);
}

#[track_caller]
fn assert_strip_tsx(input: &str, expected: &str) {
    let result = assert_strip_text(input, expected, Lang::Tsx);
    assert_valid_javascript(&result.code, Lang::Tsx);
}

#[track_caller]
fn assert_strip_error<'a>(input: &'a str, expected: &str) -> StripResult<'a> {
    let result = strip_types(input, options(Lang::Ts));
    assert_utf16_len_preserved(input, &result.code);
    assert_eq!(result.code, expected);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
    assert_valid_javascript(&result.code, Lang::Ts);
    result
}

#[track_caller]
fn assert_strip_error_at<'a>(
    input: &'a str,
    expected: &str,
    diagnostic_text: &str,
) -> StripResult<'a> {
    let result = assert_strip_error(input, expected);
    let Span { start, end } = result.diagnostics[0].span;
    assert_eq!(
        input.get(start as usize..end as usize),
        Some(diagnostic_text)
    );
    result
}

mod annotations;
mod asi;
mod bindings;
mod classes;
mod declarations;
mod diagnostics;
mod expressions;
mod generics;
mod modules;

#[test]
fn unchanged_source_returns_borrowed_code() {
    let source = "const value = 1;";
    let result = strip_types(source, options(Lang::Ts));

    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert!(result.diagnostics.is_empty());
    assert_valid_javascript(&result.code, Lang::Ts);
}

#[test]
fn stripped_source_returns_owned_code() {
    let result = strip_types("let value: number = 1;", options(Lang::Ts));

    assert!(matches!(result.code, Cow::Owned(_)));
    assert_valid_javascript(&result.code, Lang::Ts);
}

#[test]
fn parser_error_returns_borrowed_code() {
    let source = "let value: = 1;";
    let result = strip_types(source, options(Lang::Ts));

    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn parser_error_stops_processing() {
    let source = "let a:T;\nlet b:=1;\nlet c:=2;";
    let result = strip_types(source, options(Lang::Ts));

    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert_eq!(result.diagnostics.len(), 1);
}

#[test]
fn parser_options_select_language() {
    let source = "const element = <Component<Type> />;";
    let result = strip_types(source, options(Lang::Ts));
    assert!(!result.diagnostics.is_empty());

    let result = strip_types(source, options(Lang::Tsx));
    assert!(result.diagnostics.is_empty());
    assert_valid_javascript(&result.code, Lang::Tsx);
}

#[test]
fn preserve_parens_option_does_not_change_output() {
    let source = "(value) as Type;";
    let mut parser_options = options(Lang::Ts);
    parser_options.preserve_parens = false;
    let result = strip_types(source, parser_options);

    assert_utf16_len_preserved(source, &result.code);
    assert_eq!(result.code, "(value)        ;");
    assert!(result.diagnostics.is_empty());
    assert_valid_javascript(&result.code, Lang::Ts);
}

#[test]
fn comments_option_does_not_change_output() {
    let source = "/* lead */ let value: number = 1; // tail";
    let mut parser_options = options(Lang::Ts);
    parser_options.comments = CommentMode::Flat;
    let result = strip_types(source, parser_options);

    assert_utf16_len_preserved(source, &result.code);
    assert_eq!(result.code, "/* lead */ let value         = 1; // tail");
    assert!(result.diagnostics.is_empty());
    assert_valid_javascript(&result.code, Lang::Ts);
}

#[test]
fn unsupported_lang_variants() {
    let source = "const value = 1;";
    let result = strip_types(source, options(Lang::Js));
    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);

    let result = strip_types(source, options(Lang::Jsx));
    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);

    let source = "declare const value: number;";
    let result = strip_types(source, options(Lang::Dts));
    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}
