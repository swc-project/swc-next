use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParseOptions, Parser};

#[track_caller]
pub fn expect_generated_ts_pair(source: &str, pretty: &str, minified: &str) {
    let options = ParseOptions {
        lang: Lang::Ts,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    expect_generated_pair(source, options, pretty, minified);
}

#[track_caller]
pub fn expect_generated_pair(source: &str, options: ParseOptions, pretty: &str, minified: &str) {
    let allocator = Allocator::new();
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let output = generate(&parsed.ast, GeneratorOptions::default());
    assert_eq!(output.code, pretty);

    let output = generate(
        &parsed.ast,
        GeneratorOptions {
            minify: true,
            ..GeneratorOptions::default()
        },
    );
    assert_eq!(output.code, minified);
}

#[track_caller]
pub fn expect_generated(source: &str, expected: &str) {
    expect_generated_common(source, expected, Lang::Js, SourceType::Module, false, false);
}

#[track_caller]
pub fn expect_generated_minify(source: &str, expected: &str) {
    expect_generated_common(source, expected, Lang::Js, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_generated_ts(source: &str, expected: &str) {
    expect_generated_common(source, expected, Lang::Ts, SourceType::Module, false, false);
}

#[track_caller]
pub fn expect_generated_ts_minify(source: &str, expected: &str) {
    expect_generated_common(source, expected, Lang::Ts, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_generated_jsx(source: &str, expected: &str) {
    expect_generated_common(
        source,
        expected,
        Lang::Jsx,
        SourceType::Module,
        false,
        false,
    );
}

#[track_caller]
pub fn expect_generated_jsx_minify(source: &str, expected: &str) {
    expect_generated_common(source, expected, Lang::Jsx, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_generated_ascii(source: &str, expected: &str) {
    expect_generated_common(source, expected, Lang::Js, SourceType::Module, false, true);
}

#[track_caller]
pub fn expect_generated_minify_ascii(source: &str, expected: &str) {
    expect_generated_common(source, expected, Lang::Js, SourceType::Module, true, true);
}

#[track_caller]
pub fn expect_generated_jsx_ascii(source: &str, expected: &str) {
    expect_generated_common(source, expected, Lang::Jsx, SourceType::Module, false, true);
}

#[track_caller]
pub fn expect_generated_script(source: &str, expected: &str) {
    expect_generated_common(source, expected, Lang::Js, SourceType::Script, false, false);
}

#[track_caller]
pub fn expect_generated_with_options(
    source: &str,
    expected: &str,
    generator_options: GeneratorOptions,
) {
    expect_generated_common_with_options(
        source,
        expected,
        Lang::Js,
        SourceType::Module,
        generator_options,
    );
}

#[track_caller]
pub fn expect_stable(source: &str, expected: &str) {
    expect_generated(source, expected);
    expect_generated(expected, expected);
}

#[track_caller]
fn expect_generated_common(
    source: &str,
    expected: &str,
    lang: Lang,
    source_type: SourceType,
    minify: bool,
    ascii_only: bool,
) {
    let generator_options = GeneratorOptions {
        minify,
        ascii_only,
        ..GeneratorOptions::default()
    };
    expect_generated_common_with_options(source, expected, lang, source_type, generator_options);
}

#[track_caller]
fn expect_generated_common_with_options(
    source: &str,
    expected: &str,
    lang: Lang,
    source_type: SourceType,
    generator_options: GeneratorOptions,
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
    let result = generate(&result.ast, generator_options);

    assert_eq!(
        result.code, expected,
        "failed for {}\nactual:\n{}\nexpected:\n{}",
        source, result.code, expected,
    );
}
