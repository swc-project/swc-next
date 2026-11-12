use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, Lang, SourceType, SymbolId, Visit, VisitWith,
};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SymbolFlags, analyze};

use super::{JsxOptions, JsxRuntime, ReactRefreshOptions};
use crate::{
    Transformer, TransformerOptions, normalize::Normalizer, typescript::TypeScriptOptions,
};

fn transform(source: &str, lang: Lang, jsx: JsxOptions) -> String {
    let (output, diagnostics) = diagnose(source, lang, jsx);
    assert!(diagnostics.is_empty());
    output
}

fn helper_references(output: &str) -> (SymbolId, Vec<Option<SymbolId>>) {
    let allocator = Allocator::new();
    let options = ParserOptions::default();
    let parser = Parser::init(&allocator, output, options, NoTokenParserConfig);
    let mut parsed = parser.parse();
    assert!(parsed.diagnostics.is_empty());
    let analyzed = analyze(&mut parsed.ast, AnalyzeOptions::default());
    let (_, semantic) = analyzed.semantic.into_parts();
    // SAFETY: The semantic handles below come from this AST.
    let ast = unsafe { parsed.ast.ast() };
    let helper = semantic
        .iter_symbols()
        .find(|entry| entry.symbol.flags.contains(SymbolFlags::IMPORT))
        .unwrap();
    let name = ast.get_wtf8(helper.symbol.name).as_bytes();
    let references = semantic
        .iter_references()
        .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name)
        .map(|entry| entry.reference.symbol)
        .collect();
    (helper.id, references)
}

struct ArrowBodies<'visit, 'ast>(&'visit Ast<'ast>);

impl<'ast> Visit<'ast> for ArrowBodies<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.0
    }

    fn visit_arrow_function_expression(&mut self, arrow: ArrowFunctionExpression) {
        assert_eq!(
            arrow.expression(self.0),
            arrow.body(self.0).as_expr(self.0).is_some()
        );
        arrow.visit_children_with(self);
    }
}

fn diagnose(
    source: &str,
    lang: Lang,
    jsx: JsxOptions,
) -> (String, Vec<swc_next_ecma_ast::Diagnostic>) {
    let options = TransformerOptions {
        jsx: Some(jsx),
        typescript: lang.is_typescript().then(TypeScriptOptions::default),
        ..TransformerOptions::default()
    };
    let allocator = Allocator::new();
    let parser_options = ParserOptions {
        source_type: SourceType::Module,
        lang,
        ..ParserOptions::default()
    };
    let parser = Parser::init(&allocator, source, parser_options, NoTokenParserConfig);
    let parsed = parser.parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    if options.typescript.is_some() {
        Normalizer::normalize_runtime(&mut ast);
    } else {
        Normalizer::normalize(&mut ast);
    }
    let semantic = analyze(&mut ast, AnalyzeOptions::default()).semantic;
    let transformed = Transformer::transform(semantic, options);
    assert_eq!(ast.lang(), Lang::Js);
    // SAFETY: The visitor only reads handles from this transformed AST.
    let tree = unsafe { ast.ast() };
    tree.root_program().visit_with(&mut ArrowBodies(tree));
    let output = generate(&ast, GeneratorOptions::default()).code;
    let arena = Allocator::new();
    let options = ParserOptions {
        source_type: ast.source_type(),
        lang: ast.lang(),
        ..ParserOptions::default()
    };
    let parser = Parser::init(&arena, &output, options, NoTokenParserConfig);
    let reparsed = parser.parse();
    assert!(reparsed.diagnostics.is_empty());
    (output, transformed.diagnostics)
}

#[test]
fn diagnostics() {
    let lang = Lang::Jsx;
    let automatic = JsxOptions::default();
    let (_, diagnostics) = diagnose("const node = <div key />;", lang, automatic.clone());
    assert_eq!(diagnostics.len(), 1);

    let (_, diagnostics) = diagnose("const node = <svg:path />;", lang, automatic);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn namespace_attributes() {
    let lang = Lang::Jsx;
    let automatic = JsxOptions::default();
    let classic = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let source = "<svg xlink:href=\"value\" />;";
    let (_, diagnostics) = diagnose(source, lang, automatic.clone());
    assert_eq!(diagnostics.len(), 1);
    let span = diagnostics[0].span;
    assert_eq!(
        &source[span.start as usize..span.end as usize],
        "xlink:href"
    );

    let (_, diagnostics) = diagnose(source, lang, classic);
    assert_eq!(diagnostics.len(), 1);

    let source = "const node: unknown = <svg xlink:href=\"value\" />;";
    let (_, diagnostics) = diagnose(source, Lang::Tsx, automatic);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn spread_children() {
    let lang = Lang::Jsx;
    let automatic = JsxOptions::default();
    let classic = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let source = "<Comp>{...items}</Comp>;";
    let (_, diagnostics) = diagnose(source, lang, automatic.clone());
    assert_eq!(diagnostics.len(), 1);
    let span = diagnostics[0].span;
    assert_eq!(
        &source[span.start as usize..span.end as usize],
        "{...items}"
    );

    let (_, diagnostics) = diagnose(source, lang, classic);
    assert_eq!(diagnostics.len(), 1);

    let source = "<>{...items}</>;";
    let (_, diagnostics) = diagnose(source, lang, automatic.clone());
    assert_eq!(diagnostics.len(), 1);

    let source = "<Comp {...props} key=\"value\">{...items}</Comp>;";
    let (_, diagnostics) = diagnose(source, lang, automatic.clone());
    assert_eq!(diagnostics.len(), 1);

    let source = "const node: unknown = <Comp>{...items}</Comp>;";
    let (_, diagnostics) = diagnose(source, Lang::Tsx, automatic);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn generated_bindings() {
    let options = JsxOptions::default();
    let source = "export namespace _jsx { export const _jsx = <div />; }";
    let output = transform(source, Lang::Tsx, options.clone());
    let (helper, references) = helper_references(&output);
    assert_eq!(references, [Some(helper)]);

    let source = "<div />; export namespace _jsx { export const _jsx = 0; }";
    let output = transform(source, Lang::Tsx, options);
    let (helper, references) = helper_references(&output);
    assert_eq!(references, [Some(helper)]);
}

#[test]
fn arrow_bodies() {
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
        ..JsxOptions::default()
    };
    transform("const App = () => <div />;", Lang::Jsx, options.clone());

    transform("const App = () => useState(0);", Lang::Jsx, options.clone());

    let source = "const App = (() => <div />) satisfies F;";
    transform(source, Lang::Tsx, options);
}
