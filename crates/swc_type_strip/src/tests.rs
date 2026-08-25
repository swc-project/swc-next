use std::borrow::Cow;

use swc_next_ecma_ast::SourceType;
use swc_next_ecma_parser::CommentMode;

use crate::strip_types;

fn options(lang: crate::Lang) -> crate::Options {
    crate::Options {
        source_type: SourceType::Unambiguous,
        lang,
        preserve_parens: true,
        comments: CommentMode::None,
    }
}

fn assert_utf16_len_preserved(input: &str, output: &str) {
    let input_len = input.encode_utf16().count();
    let output_len = output.encode_utf16().count();

    assert_eq!(output_len, input_len);
}

fn assert_diagnostic_span_text(input: &str, span: crate::Span, expected: &str) {
    let start = span.start as usize;
    let end = span.end as usize;
    let actual = input
        .get(start..end)
        .expect("diagnostic span must select valid source text");

    assert_eq!(actual, expected);
}

macro_rules! assert_strip {
    ($input:literal $expected:literal $(,)?) => {{
        assert_strip!(crate::Lang::Ts; $input $expected)
    }};
    ($lang:expr; $input:literal $expected:literal $(,)?) => {{
        let result = crate::strip_types($input, crate::tests::options($lang));

        crate::tests::assert_utf16_len_preserved($input, &result.code);
        assert_eq!(result.code, $expected);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );
    }};
}

macro_rules! assert_strip_with_diagnostic {
    (@$diagnostic_text:literal $input:literal $expected:literal $(,)?) => {{
        let result = assert_strip_with_diagnostic!($input $expected);

        crate::tests::assert_diagnostic_span_text(
            $input,
            result.diagnostics[0].span,
            $diagnostic_text,
        );
    }};
    ($input:literal $expected:literal $(,)?) => {{
        let result = crate::strip_types($input, crate::tests::options(crate::Lang::Ts));

        crate::tests::assert_utf16_len_preserved($input, &result.code);
        assert_eq!(result.code, $expected);
        assert_eq!(result.diagnostics.len(), 1, "{:#?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].severity, crate::Severity::Error);
        assert_eq!(
            result.diagnostics[0].message,
            crate::ERASABLE_SYNTAX_ONLY_MESSAGE
        );

        result
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
    let result = strip_types(source, options(crate::Lang::Ts));

    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert!(result.diagnostics.is_empty());
}

#[test]
fn stripped_source_returns_owned_code() {
    let result = strip_types("let value: number = 1;", options(crate::Lang::Ts));

    assert!(matches!(result.code, Cow::Owned(_)));
}

#[test]
fn parser_error_returns_borrowed_code() {
    let source = "let value: = 1;";
    let result = strip_types(source, options(crate::Lang::Ts));

    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn parser_error_does_not_block_recovery_edits() {
    let result = strip_types("let a:T;\nlet b:=1;\nlet c:=2;", options(crate::Lang::Ts));

    assert_eq!(result.code, "let a  ;\nlet b:=1;\nlet c:=2;");
    assert!(result.diagnostics.len() > 1);
}

#[test]
fn parser_options_select_language() {
    let source = "const element = <Component<Type> />;";
    let result = strip_types(source, options(crate::Lang::Ts));
    assert!(!result.diagnostics.is_empty());

    let result = strip_types(source, options(crate::Lang::Tsx));
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
}

#[test]
fn preserve_parens_option_does_not_change_output() {
    let source = "(value) as Type;";
    let mut parser_options = options(crate::Lang::Ts);
    parser_options.preserve_parens = false;
    let result = strip_types(source, parser_options);

    assert_utf16_len_preserved(source, &result.code);
    assert_eq!(result.code, "(value)        ;");
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
}

#[test]
fn comments_option_does_not_change_output() {
    let source = "/* lead */ let value: number = 1; // tail";
    let mut parser_options = options(crate::Lang::Ts);
    parser_options.comments = CommentMode::Flat;
    let result = strip_types(source, parser_options);

    assert_utf16_len_preserved(source, &result.code);
    assert_eq!(result.code, "/* lead */ let value         = 1; // tail");
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
}

#[test]
fn unsupported_lang_variants() {
    let source = "let value: = 1;";
    let result = strip_types(source, options(crate::Lang::Js));
    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, crate::Severity::Error);
    assert_eq!(
        result.diagnostics[0].message,
        crate::UNSUPPORTED_LANG_MESSAGE
    );

    let result = strip_types(source, options(crate::Lang::Jsx));
    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, crate::Severity::Error);
    assert_eq!(
        result.diagnostics[0].message,
        crate::UNSUPPORTED_LANG_MESSAGE
    );

    let result = strip_types(source, options(crate::Lang::Dts));
    assert!(matches!(result.code, Cow::Borrowed(value) if value == source));
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, crate::Severity::Error);
    assert_eq!(
        result.diagnostics[0].message,
        crate::UNSUPPORTED_LANG_MESSAGE
    );
}
