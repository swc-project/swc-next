use std::borrow::Cow;

use swc_next_allocator::Allocator;
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};
use swc_next_type_strip::{Lang, Options, Severity, SourceType, StripResult, strip_types};

fn options(lang: Lang) -> Options {
    Options {
        source_type: SourceType::Unambiguous,
        lang,
    }
}

#[track_caller]
fn assert_valid_javascript(source: &str, lang: Lang) {
    let lang = if lang == Lang::Tsx {
        Lang::Jsx
    } else {
        Lang::Js
    };
    let allocator = Allocator::default();
    let parsed = Parser::init(
        &allocator,
        source,
        options(lang).into(),
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
}

#[track_caller]
fn assert_stripped_output<'a>(input: &'a str, expected: &str, lang: Lang) -> StripResult<'a> {
    let result = strip_types(input, options(lang));
    assert_eq!(
        result.code.encode_utf16().count(),
        input.encode_utf16().count()
    );
    assert_eq!(result.code, expected);
    result
}

#[track_caller]
fn assert_strip_success(input: &str, expected: &str, lang: Lang) {
    let result = assert_stripped_output(input, expected, lang);
    assert!(result.diagnostics.is_empty());
    assert_valid_javascript(&result.code, lang);
}

#[track_caller]
fn assert_strip_error(input: &str, expected: &str, source_fragment: Option<&str>) {
    let result = assert_stripped_output(input, expected, Lang::Ts);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.severity, Severity::Error);
    if let Some(source_fragment) = source_fragment {
        let span = diagnostic.span;
        assert_eq!(
            input.get(span.start as usize..span.end as usize),
            Some(source_fragment),
        );
    }
}

macro_rules! assert_strip {
    (lang: tsx, $input:expr, $expected:expr $(,)?) => {{
        $crate::assert_strip_success($input, $expected, $crate::Lang::Tsx);
    }};
    ($input:expr, $expected:expr $(,)?) => {{
        $crate::assert_strip_success($input, $expected, $crate::Lang::Ts);
    }};
}

macro_rules! assert_strip_with_error {
    ($input:expr, $expected:expr $(,)?) => {{
        $crate::assert_strip_error($input, $expected, None);
    }};
    ($input:expr, $expected:expr, at: $source_fragment:expr $(,)?) => {{
        $crate::assert_strip_error($input, $expected, Some($source_fragment));
    }};
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
fn language() {
    let source = "const element = <Component<Type> />;";
    let result = strip_types(source, options(Lang::Ts));
    assert!(!result.diagnostics.is_empty());

    let result = strip_types(source, options(Lang::Tsx));
    assert!(result.diagnostics.is_empty());
    assert_valid_javascript(&result.code, Lang::Tsx);
}

#[test]
fn parentheses() {
    assert_strip!("(value) as Type;", "(value)        ;");
}

#[test]
fn comments() {
    assert_strip!(
        "/* lead */ let value: number = 1; // tail",
        "/* lead */ let value         = 1; // tail",
    );
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
