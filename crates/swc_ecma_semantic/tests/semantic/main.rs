mod aliases;
mod bindings;
mod exports;
mod members;
mod properties;
mod references;
mod support;
mod visibility;

use support::*;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, AstCtx, Diagnostic, Lang, ScopeId, SourceType, SymbolId};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

use swc_next_ecma_semantic::{
    AnalyzeOptions, NameResolution, ReferenceSpace, Semantic, SymbolFlags, SyntaxChecks,
    check_syntax,
};

struct TestAnalysis<'a> {
    semantic: Semantic<'a>,
    diagnostics: Vec<Diagnostic<'a>>,
}

fn analyze<'a>(ast: &AstCtx<'a>, options: AnalyzeOptions) -> TestAnalysis<'a> {
    let result = swc_next_ecma_semantic::analyze(ast, options);
    let semantic = result.semantic.into_parts().1;
    TestAnalysis {
        semantic,
        diagnostics: result.diagnostics,
    }
}

#[track_caller]
fn analyze_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Options,
) -> (Ast<'a>, Semantic<'a>) {
    analyze_with_options(allocator, source, parser_options, AnalyzeOptions::default())
}

#[track_caller]
fn analyze_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Options,
    analyze_options: AnalyzeOptions,
) -> (Ast<'a>, Semantic<'a>) {
    let parse = Parser::init(allocator, source, parser_options, NoTokenParserConfig).parse();
    assert!(parse.diagnostics.is_empty());

    let result = analyze(&parse.ast, analyze_options);
    if !analyze_options.check_syntax.is_empty() {
        assert_eq!(
            check_syntax(&parse.ast, analyze_options.check_syntax),
            result.diagnostics
        );
    }
    assert!(result.diagnostics.is_empty());
    // SAFETY: The test helpers only use handles with their originating AST.
    (unsafe { parse.ast.into_ast() }, result.semantic)
}

#[track_caller]
fn analyze_syntax_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Options,
) -> (Ast<'a>, TestAnalysis<'a>) {
    let parse = Parser::init(allocator, source, parser_options, NoTokenParserConfig).parse();
    assert!(parse.diagnostics.is_empty());
    let result = analyze(
        &parse.ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
        },
    );
    assert_eq!(
        check_syntax(&parse.ast, SyntaxChecks::all()),
        result.diagnostics
    );
    (unsafe { parse.ast.into_ast() }, result)
}

#[track_caller]
fn has_syntax_diagnostics(allocator: &Allocator, source: &str, parser_options: Options) -> bool {
    let (_, result) = analyze_syntax_source(allocator, source, parser_options);
    !result.diagnostics.is_empty()
}

#[track_caller]
fn reference_symbol(ast: &Ast<'_>, semantic: &Semantic<'_>, name: &[u8]) -> Option<SymbolId> {
    let mut references = semantic
        .iter_references()
        .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name);
    let symbol = references.next().unwrap().reference.symbol;
    assert!(references.next().is_none());
    symbol
}

fn reference_symbols(ast: &Ast<'_>, semantic: &Semantic<'_>, name: &[u8]) -> Vec<Option<SymbolId>> {
    semantic
        .iter_references()
        .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name)
        .map(|entry| entry.reference.symbol)
        .collect()
}

#[track_caller]
fn symbol_named(ast: &Ast<'_>, semantic: &Semantic<'_>, name: &[u8]) -> SymbolId {
    let mut symbols = semantic
        .iter_symbols()
        .filter(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == name);
    let symbol = symbols.next().unwrap().id;
    assert!(symbols.next().is_none());
    symbol
}

fn ts_module_options() -> Options {
    Options {
        lang: Lang::Ts,
        ..Options::default()
    }
}

fn dts_module_options() -> Options {
    Options {
        lang: Lang::Dts,
        ..Options::default()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TsModuleState {
    NonInstantiated,
    Instantiated,
    ConstEnumOnly,
}

#[track_caller]
fn assert_ts_module_state(semantic: &Semantic<'_>, symbol: SymbolId, expected: TsModuleState) {
    let flags = semantic.symbol(symbol).flags;
    let actual = match (
        flags.contains(SymbolFlags::VALUE_MODULE),
        flags.contains(SymbolFlags::CONST_ENUM_ONLY_MODULE),
    ) {
        (false, false) => TsModuleState::NonInstantiated,
        (true, false) => TsModuleState::Instantiated,
        (true, true) => TsModuleState::ConstEnumOnly,
        (false, true) => panic!("const-enum-only modules must exist in value space"),
    };
    assert_eq!(actual, expected);
}

#[test]
fn regular_expression_diagnostics() {
    use swc_next_ecma_ast::{Severity, Span};

    let allocator = Allocator::new();
    let parser_options = Options::default();
    let unchecked_options = AnalyzeOptions::default();

    let source = " /a/gg;";
    let (ast, result) = analyze_syntax_source(&allocator, source, parser_options);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
    assert_eq!(
        result.diagnostics[0].span,
        Span::new(1, source.len() as u32 - 1)
    );
    assert!(result.diagnostics[0].help.is_none());
    let ctx = ast.into();
    assert!(analyze(&ctx, unchecked_options).diagnostics.is_empty());
    assert!(check_syntax(&ctx, SyntaxChecks::all() - SyntaxChecks::REGEXP_LITERALS).is_empty());

    let source = " /🥹[z-a]/u;";
    let (_, result) = analyze_syntax_source(&allocator, source, parser_options);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].span,
        Span::new(1, source.len() as u32 - 1)
    );

    let source = " /(?<x>a)(?<x>b)/;";
    let (_, result) = analyze_syntax_source(&allocator, source, parser_options);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].span,
        Span::new(1, source.len() as u32 - 1)
    );
    assert!(result.diagnostics[0].labels.is_empty());
}
