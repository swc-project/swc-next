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
use swc_next_ecma_ast::{Ast, Lang, ScopeId, SourceType, SymbolId};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

use swc_next_ecma_semantic::{
    AnalyzeOptions, ExportOrigin, ExportTarget, NameResolution, ReferenceSpace, Semantic,
    SemanticReturn, SymbolFlags, SyntaxChecks, analyze, check_syntax,
};

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
    let ast = parse.ast;
    assert!(parse.diagnostics.is_empty());

    let result = analyze(&ast, analyze_options);
    if !analyze_options.check_syntax.is_empty() {
        assert_eq!(
            check_syntax(&ast, analyze_options.check_syntax),
            result.diagnostics
        );
    }
    assert!(result.diagnostics.is_empty());
    (ast, result.semantic)
}

#[track_caller]
fn analyze_syntax_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Options,
) -> (Ast<'a>, SemanticReturn<'a>) {
    let parse = Parser::init(allocator, source, parser_options, NoTokenParserConfig).parse();
    assert!(parse.diagnostics.is_empty());
    let result = analyze(
        &parse.ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
            ..AnalyzeOptions::default()
        },
    );
    assert_eq!(
        check_syntax(&parse.ast, SyntaxChecks::all()),
        result.diagnostics
    );
    (parse.ast, result)
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
