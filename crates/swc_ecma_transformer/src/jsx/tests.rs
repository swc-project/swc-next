use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, Lang, SourceType, SymbolId, Visit, VisitWith,
};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SymbolFlags, analyze};

use super::{JsxOptions, JsxRuntime, ReactRefreshOptions};
use crate::{
    Transformer, TransformerOptions, normalize::Normalizer, typescript::TypeScriptOptions,
};

fn transform(
    source: &str,
    source_type: SourceType,
    lang: Lang,
    jsx: JsxOptions,
    typescript: bool,
) -> String {
    let (output, diagnostics) =
        transform_with_diagnostics(source, source_type, lang, jsx, typescript);
    assert!(diagnostics.is_empty());
    output
}

fn transform_with_diagnostics(
    source: &str,
    source_type: SourceType,
    lang: Lang,
    jsx: JsxOptions,
    typescript: bool,
) -> (String, Vec<swc_next_ecma_ast::Diagnostic<'static>>) {
    transform_options(
        source,
        source_type,
        lang,
        TransformerOptions {
            jsx: Some(Box::new(jsx)),
            typescript: typescript.then(TypeScriptOptions::default),
            ..TransformerOptions::default()
        },
    )
}

fn diagnose(
    source: &str,
    lang: Lang,
    jsx: JsxOptions,
) -> (String, Vec<swc_next_ecma_ast::Diagnostic<'static>>) {
    transform_with_diagnostics(source, SourceType::Module, lang, jsx, lang == Lang::Tsx)
}

fn refresh(source: &str) -> String {
    refresh_lang(source, Lang::Jsx)
}

fn refresh_lang(source: &str, lang: Lang) -> String {
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions::default()),
        ..JsxOptions::default()
    };
    transform(source, SourceType::Module, lang, options, lang == Lang::Tsx)
}

fn pragma(pragma: &str) -> String {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        pragma: Some(pragma.to_owned()),
        ..JsxOptions::default()
    };
    transform("<div />;", SourceType::Module, Lang::Jsx, options, false)
}

fn classic_tsx(source: &str) -> String {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    transform(source, SourceType::Module, Lang::Tsx, options, true)
}

fn lower_jsx(source: &str, options: &JsxOptions) -> String {
    transform(
        source,
        SourceType::Module,
        Lang::Jsx,
        options.clone(),
        false,
    )
}

fn lower_tsx(source: &str, options: &JsxOptions) -> String {
    transform(source, SourceType::Module, Lang::Tsx, options.clone(), true)
}

fn helper_references(output: &str) -> (SymbolId, Vec<Option<SymbolId>>) {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        output,
        ParserOptions::default(),
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let ast = parsed.ast;
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
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

fn classic_development(source: &str) -> String {
    let options = TransformerOptions {
        source_path: Some("input.jsx".to_owned()),
        jsx: Some(Box::new(JsxOptions {
            runtime: JsxRuntime::Classic,
            development: true,
            source_self: true,
            ..JsxOptions::default()
        })),
        ..TransformerOptions::default()
    };
    let (output, diagnostics) = transform_options(source, SourceType::Module, Lang::Jsx, options);
    assert!(diagnostics.is_empty());
    output
}

fn display_name(source: &str) -> String {
    display_name_at(source, "src/Component.jsx")
}

fn display_name_at(source: &str, path: &str) -> String {
    let options = TransformerOptions {
        source_path: Some(path.to_owned()),
        react_display_name: true,
        jsx: Some(Box::new(JsxOptions {
            runtime: JsxRuntime::Classic,
            ..JsxOptions::default()
        })),
        ..TransformerOptions::default()
    };
    let (output, diagnostics) = transform_options(source, SourceType::Module, Lang::Jsx, options);
    assert!(diagnostics.is_empty());
    output
}

fn transform_options(
    source: &str,
    source_type: SourceType,
    lang: Lang,
    options: TransformerOptions,
) -> (String, Vec<swc_next_ecma_ast::Diagnostic<'static>>) {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            source_type,
            lang,
            comments: CommentMode::Flat,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    if options.typescript.is_some() {
        Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    } else {
        Normalizer::normalize(ast.root_program(), &mut ast);
    }
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let expected_lang = if options.jsx.is_some() {
        Lang::Js
    } else {
        lang
    };
    let transformed = Transformer::transform(&mut ast, semantic, options);
    assert_eq!(ast.lang(), expected_lang);
    ast.root_program().visit_with(&mut ArrowBodies(&ast));
    let output = print(&ast, CodegenOptions::default()).code.to_owned();
    let output_allocator = Allocator::new();
    let reparsed = Parser::init(
        &output_allocator,
        &output,
        ParserOptions {
            source_type: ast.source_type(),
            lang: ast.lang(),
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(reparsed.diagnostics.is_empty());
    (output, transformed.diagnostics)
}

#[test]
fn classic() {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let output = lower_jsx("const node = <Foo disabled>hello</Foo>;", &options);
    assert_eq!(
        output,
        "const node = /* #__PURE__ */ React.createElement(Foo, { disabled: true }, \"hello\");\n"
    );

    let source = "/** @jsx h */ const node = <><this /><foo.Bar /></>;";
    let output = lower_jsx(source, &options);
    assert!(output.contains("h(React.Fragment"));
    assert!(output.contains("h(this, null)"));
    assert!(output.contains("h(foo.Bar, null)"));
}

#[test]
fn spread_attributes() {
    let classic = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let automatic = JsxOptions::default();
    let source = "<div {...props} />;";
    let output = lower_jsx(source, &classic);
    assert_eq!(
        output,
        "/* #__PURE__ */ React.createElement(\"div\", props);\n"
    );

    let source = "<div first={before()} {...{ second: middle() }} third={after()} />;";
    let output = lower_jsx(source, &classic);
    assert_eq!(
        output,
        "/* #__PURE__ */ React.createElement(\"div\", { first: before(), second: middle(), third: after() });\n"
    );

    let source = "<div {...{ __proto__: proto, value: 1 }} />;";
    let output = lower_jsx(source, &classic);
    assert_eq!(
        output,
        "/* #__PURE__ */ React.createElement(\"div\", { ...{ __proto__: proto, value: 1 } });\n"
    );

    let source = "<Comp {...{ get value() { sideEffect(); return 1; } }} other={2} />;";
    let output = lower_jsx(source, &classic);
    assert!(output.contains("React.createElement(Comp, { ...{ get value()"));

    let source = "<Comp {...{ get value() { sideEffect(); return 1; } }} />;";
    let output = lower_jsx(source, &automatic);
    assert!(output.contains("_jsx(Comp, { ...{ get value()"));

    let source = "<Comp {...{ set value(v) { sideEffect(v); } }} />;";
    let output = lower_jsx(source, &automatic);
    assert!(output.contains("_jsx(Comp, { ...{ set value(v)"));
}

#[test]
fn automatic() {
    let options = JsxOptions::default();
    let output = lower_jsx("const node = <><div key=\"a\" /><span /></>;", &options);
    assert!(output.starts_with(
        "import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from \"react/jsx-runtime\";"
    ));
    assert!(output.contains("_jsx(\"div\", {}, \"a\")"));

    let source = "var node = <div />;";
    let output = transform(source, SourceType::Script, Lang::Jsx, options, false);
    assert!(output.starts_with("var _reactJsxRuntime = require(\"react/jsx-runtime\");"));
    assert!(output.contains("_reactJsxRuntime.jsx(\"div\", {})"));
}

#[test]
fn tsx_composition() {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let source = "import React, { type FC } from 'react'; const node: JSX.Element = <div />;";
    let output = lower_tsx(source, &options);
    assert!(output.starts_with("import React from \"react\";"));
    assert!(output.contains("const node = /* #__PURE__ */ React.createElement(\"div\", null);"));
    assert!(!output.contains("FC"));
    assert!(!output.contains("JSX.Element"));

    let automatic = JsxOptions::default();
    let source = "import React from 'react'; const node = <div />;";
    let output = lower_tsx(source, &automatic);
    assert!(!output.contains("import React"));

    let classic = JsxOptions {
        runtime: JsxRuntime::Classic,
        pragma: Some("h".into()),
        pragma_frag: Some("Frag".into()),
        ..JsxOptions::default()
    };
    let source = "import { h, Frag } from 'ui'; const node = <></>;";
    let output = lower_tsx(source, &classic);
    assert!(output.contains("h, Frag"));
    assert!(output.contains("h(Frag, null)"));

    let source = "import { h, Frag } from 'ui'; const node = <div />;";
    let output = lower_tsx(source, &classic);
    assert!(output.contains("import { h }"));
    assert!(!output.contains("Frag"));

    let source = "import { h } from 'ui'; function render(h: any) { return <div />; }";
    let output = lower_tsx(source, &classic);
    assert!(!output.contains("from \"ui\""));

    let output = lower_tsx("import h = require('ui'); const node = <div />;", &classic);
    assert!(output.contains("import.sync(\"ui\")"));
}

#[test]
fn tsx_namespaces() {
    let source = "namespace N { export const React = factory; export const node = <div />; }";
    let output = classic_tsx(source);
    assert!(output.contains("N.React.createElement(\"div\", null)"));

    let source = "namespace N { export const React = factory; function render(React: any) { return <div />; } }";
    let output = classic_tsx(source);
    assert!(output.contains("return /* #__PURE__ */ React.createElement(\"div\", null)"));

    let source = "namespace N { export const React = factory; } namespace N { export const node = <div />; }";
    let output = classic_tsx(source);
    assert!(output.contains("N.React.createElement(\"div\", null)"));

    let source = "namespace N { export const React = factory; export namespace M { export const node = <><div /></>; } }";
    let output = classic_tsx(source);
    assert!(output.contains("N.React.createElement(\"div\", null)"));
    assert!(output.contains("N.React.Fragment,"));

    let source = "namespace N { export const React = factory; class C { [<div /> as any](React: any) { return <span />; } } }";
    let output = classic_tsx(source);
    assert!(output.contains("N.React.createElement(\"div\", null)"));
    assert!(output.contains("return /* #__PURE__ */ React.createElement(\"span\", null)"));

    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        pragma: Some("h".into()),
        pragma_frag: Some("Frag".into()),
        ..JsxOptions::default()
    };
    let source = "namespace N { export const h = factory; export const Frag = fragment; export const node = <><div /></>; }";
    let output = lower_tsx(source, &options);
    assert!(output.contains("N.h(N.Frag, null, N.h(\"div\", null))"));
}

#[test]
fn pure_calls() {
    let options = JsxOptions::default();
    let output = lower_jsx("(<A f={tick} />).props.f();", &options);
    assert!(output.contains("(/* #__PURE__ */ _jsx(A, { f: tick })).props.f()"));
}

#[test]
fn literal_whitespace() {
    let classic = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let automatic = JsxOptions::default();
    for options in [&classic, &automatic] {
        let output = lower_jsx("<div title=\"a\n  b\" />;", options);
        assert!(output.contains("title: \"a b\""));

        let output = lower_jsx("<div title=\"a&#10;&nbsp;b\" />;", options);
        assert!(output.contains("title: \"a b\""));

        let output = lower_jsx("<div>a\tb</div>;", options);
        assert!(output.contains("\"a b\""));

        let output = lower_jsx("<div>a&#10; b</div>;", options);
        assert!(output.contains("\"a b\""));
    }

    let output = lower_jsx("<div>\n&#32;\n</div>;", &classic);
    assert_eq!(
        output,
        "/* #__PURE__ */ React.createElement(\"div\", null);\n"
    );

    let output = lower_jsx("<div>\n&#32;\n</div>;", &automatic);
    assert_eq!(
        output,
        concat!(
            "import { jsx as _jsx } from \"react/jsx-runtime\";\n",
            "/* #__PURE__ */ _jsx(\"div\", {});\n",
        )
    );
}

#[test]
fn attribute_comments() {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let source = "const node = <Trans /* before */ a=\"1\" /** between */ b=\"2\" /* after */ />;";
    let output = lower_jsx(source, &options);
    assert!(output.contains("Trans,\n  /* before */\n  {"));
    assert!(output.contains("/** between */ b: \"2\" /* after */"));
}

#[test]
fn pragmas() {
    let options = JsxOptions::default();
    let source = "/** @jsx h\n * @jsxRuntime classic\n * @jsxFrag Frag */ <></>;";
    let output = lower_jsx(source, &options);
    assert!(output.contains("h(Frag, null)"));

    let source = "const before = 1; /** @jsxRuntime classic */ const node = <div />;";
    let output = lower_jsx(source, &options);
    assert!(output.starts_with("import { jsx as _jsx } from \"react/jsx-runtime\";"));

    let source = "/** @jsxImportSource custom */ const node = <div />;";
    let output = lower_jsx(source, &options);
    assert!(output.starts_with("import { jsx as _jsx } from \"custom/jsx-runtime\";"));
}

#[test]
fn pragma_paths() {
    let output = pragma("bad-name");
    assert!(output.contains("React.createElement(\"div\", null)"));

    let output = pragma("Factory..createElement");
    assert!(output.contains("React.createElement(\"div\", null)"));

    let output = pragma("true.factory");
    assert!(output.contains("React.createElement(\"div\", null)"));

    let output = pragma("factory.default");
    assert!(output.contains("factory.default(\"div\", null)"));

    let output = pragma("import.meta.h");
    assert!(output.contains("import.meta.h(\"div\", null)"));
}

#[test]
fn development() {
    let automatic_metadata = JsxOptions {
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let output = lower_jsx("const node = <Foo><span /></Foo>;", &automatic_metadata);
    assert!(output.contains("jsxDEV as _jsxDEV"));
    assert!(output.contains("var _jsxFileName = \"\";"));
    assert!(output.contains("_jsxDEV(Foo"));
    assert!(output.contains("fileName: _jsxFileName"));
    assert!(output.contains(", this)"));

    let automatic = JsxOptions {
        development: true,
        ..JsxOptions::default()
    };
    let output = lower_jsx("const node = <Foo />;", &automatic);
    assert!(output.contains("_jsxDEV(Foo, {}, void 0, false)"));
    assert!(!output.contains("_jsxFileName"));

    let classic_metadata = JsxOptions {
        runtime: JsxRuntime::Classic,
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let output = lower_jsx("const node = <Foo />;", &classic_metadata);
    assert!(output.contains("{ __self: this, __source: {"));
}

#[test]
fn development_attributes() {
    let output = classic_development("<div {...props} />;");
    assert!(output.starts_with("var _jsxFileName = \"input.jsx\";"));
    assert!(output.contains("React.createElement(\"div\", { ...props, __self: this, __source:"));

    let output = classic_development("class D extends B { constructor() { <div />; } }");
    assert!(!output.contains("__self"));

    let output = classic_development("<Foo __self={owner} __source={location} />;");
    assert_eq!(
        output,
        "/* #__PURE__ */ React.createElement(Foo, { __self: owner, __source: location });\n"
    );

    let output = classic_development("<Foo __self={owner} />;");
    assert!(output.contains("__self: owner"));
    assert_eq!(output.matches("__self:").count(), 1);
    assert!(output.contains("__source: { fileName:"));

    let output = classic_development("<Foo __source={location} />;");
    assert!(output.contains("__source: location"));
    assert_eq!(output.matches("__source:").count(), 1);
    assert!(output.contains("__self: this"));

    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let source = "namespace N { export const node = <div />; }";
    let output = lower_tsx(source, &options);
    assert!(output.contains("N.node = /* #__PURE__ */ React.createElement(\"div\", {"));
    assert!(output.contains("fileName: _jsxFileName"));
}

#[test]
fn diagnostics() {
    let lang = Lang::Jsx;
    let automatic = JsxOptions::default();
    let (_, diagnostics) = diagnose("const node = <div key />;", lang, automatic.clone());
    assert_eq!(diagnostics.len(), 1);

    let (_, diagnostics) = diagnose("const node = <svg:path />;", lang, automatic);
    assert_eq!(diagnostics.len(), 1);

    let options = JsxOptions {
        throw_if_namespace: false,
        ..JsxOptions::default()
    };
    let (output, diagnostics) = diagnose("const node = <svg:path />;", lang, options);
    assert!(diagnostics.is_empty());
    assert!(output.contains("_jsx(\"svg:path\""));
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

    let (_, diagnostics) = diagnose(source, lang, classic.clone());
    assert_eq!(diagnostics.len(), 1);

    let options = JsxOptions {
        throw_if_namespace: false,
        ..automatic.clone()
    };
    let (output, diagnostics) = diagnose(source, lang, options);
    assert!(diagnostics.is_empty());
    assert!(output.contains("_jsx(\"svg\", { \"xlink:href\": \"value\" })"));

    let options = JsxOptions {
        throw_if_namespace: false,
        ..classic
    };
    let (output, diagnostics) = diagnose(source, lang, options);
    assert!(diagnostics.is_empty());
    assert!(output.contains("React.createElement(\"svg\", { \"xlink:href\": \"value\" })"));

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
    let (output, diagnostics) = diagnose(source, lang, automatic.clone());
    assert_eq!(diagnostics.len(), 1);
    assert!(!output.contains("...items"));
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
fn display_names() {
    let output = display_name("const C = createReactClass({ displayName: 'Manual' });");
    assert_eq!(
        output,
        "const C = createReactClass({ displayName: \"Manual\" });\n"
    );

    let output = display_name("export default createReactClass({ render: () => <div /> });");
    assert_eq!(
        output,
        "export default createReactClass({ displayName: \"Component\", render: () => /* #__PURE__ */ React.createElement(\"div\", null) });\n"
    );

    let source = "export default createReactClass({});";
    let output = display_name_at(source, "src/Profile/index.jsx");
    assert_eq!(
        output,
        "export default createReactClass({ displayName: \"Profile\" });\n"
    );

    let output = display_name_at(source, "index.jsx");
    assert_eq!(
        output,
        "export default createReactClass({ displayName: \"index\" });\n"
    );
}

#[test]
fn refresh_registration() {
    let output = refresh("function Hello() { return <h1>Hi</h1>; }");
    assert!(output.contains("_c = Hello;"));
    assert!(output.contains("$RefreshReg$(_c, \"Hello\");"));

    let output = refresh("const Bar = memo(() => <Hello />);");
    assert!(output.contains("const Bar = memo(_c = () =>"));
    assert!(output.contains("_c2 = Bar;"));
    assert!(output.contains("$RefreshReg$(_c, \"Bar$memo\");"));
    assert!(output.contains("$RefreshReg$(_c2, \"Bar\");"));
}

#[test]
fn refresh_boundaries() {
    let output = refresh("const Header = styled.div``; <Header />;");
    assert!(output.contains(", \"Header\");"));

    let source = "namespace Foo { export namespace Bar { export const A = () => null; } }";
    let output = refresh_lang(source, Lang::Tsx);
    assert!(output.contains(", \"Foo$Bar$A\");"));

    let source = "namespace Foo { export namespace Bar { function B() {} } }";
    let output = refresh_lang(source, Lang::Tsx);
    assert!(output.contains(", \"Foo$Bar$B\");"));

    let source = "namespace Foo { namespace Hidden { export const C = () => null; } }";
    let output = refresh_lang(source, Lang::Tsx);
    assert!(!output.contains("Foo$Hidden$C"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions::default()),
        ..JsxOptions::default()
    };
    let source = "namespace Foo /* comment */ . Bar { export function App() {} }";
    let output = lower_tsx(source, &options);
    assert!(output.contains(", \"Foo$Bar$App\");"));

    let source = "namespace F\\u006fo.Bar { export function App() {} }";
    let output = lower_tsx(source, &options);
    assert!(output.contains(", \"Foo$Bar$App\");"));
}

#[test]
fn refresh_component_imports() {
    let output = refresh("const App = () => require('dep');");
    assert!(output.contains("$RefreshReg$(_c, \"App\");"));

    let output = refresh("const App = () => <A onClick={() => import('dep')} />;");
    assert!(output.contains("$RefreshReg$(_c, \"App\");"));

    let output = refresh("const App = memo(() => import('dep'));");
    assert!(output.contains("$RefreshReg$(_c, \"App$memo\");"));
    assert!(output.contains("$RefreshReg$(_c2, \"App\");"));

    let output = refresh("const App = require('dep'); const node = <App />;");
    assert!(!output.contains("$RefreshReg$"));
}

#[test]
fn refresh_typescript() {
    let output = refresh_lang("const App = (() => <div />) satisfies F;", Lang::Tsx);
    assert!(output.contains("const App = () =>"));
    assert!(output.contains("$RefreshReg$(_c, \"App\");"));

    let output = refresh_lang("const App = require('dep') as F; <App />;", Lang::Tsx);
    assert!(!output.contains("$RefreshReg$"));

    let output = refresh_lang("export default memo((() => <div/>) as F) as F;", Lang::Tsx);
    assert!(output.contains("$RefreshReg$(_c, \"%default%$memo\");"));
    assert!(output.contains("$RefreshReg$(_c2, \"%default%\");"));
}

#[test]
fn refresh_callbacks() {
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            refresh_reg: "_c".to_owned(),
        }),
        ..JsxOptions::default()
    };
    let output = lower_jsx("function App() { useState(0); }", &options);
    assert!(output.contains("_c(_c2, \"App\");"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            refresh_reg: "_s.refreshReg".to_owned(),
        }),
        ..JsxOptions::default()
    };
    let output = lower_jsx("function App() { useState(0); }", &options);
    assert!(output.contains("_s.refreshReg(_c, \"App\");"));
}

#[test]
fn pragma_bindings() {
    let factory = JsxOptions {
        runtime: JsxRuntime::Classic,
        pragma: Some("_jsxFileName.h".to_owned()),
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let fragment = JsxOptions {
        pragma: Some("h".to_owned()),
        pragma_frag: Some("_jsxFileName".to_owned()),
        ..factory.clone()
    };
    let registration = JsxOptions {
        runtime: JsxRuntime::Classic,
        pragma: Some("_c".to_owned()),
        refresh: Some(ReactRefreshOptions::default()),
        ..JsxOptions::default()
    };
    for lower in [lower_jsx, lower_tsx] {
        let output = lower("<div />;", &factory);
        assert!(output.contains("var _jsxFileName2 = \"\";"));
        assert!(output.contains("_jsxFileName.h(\"div\","));

        let output = lower("<><div /></>;", &fragment);
        assert!(output.contains("var _jsxFileName2 = \"\";"));
        assert!(output.contains("h(_jsxFileName, null,"));

        let output = lower("function App() { return <div />; }", &registration);
        assert!(output.contains("return _c(\"div\", null);"));
        assert!(output.contains("$RefreshReg$(_c2, \"App\");"));
    }
}

#[test]
fn generated_bindings() {
    let development = JsxOptions {
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let source = "const _jsxFileName = 'source'; const render = () => <div />;";
    let output = transform(source, SourceType::Script, Lang::Jsx, development, false);
    assert!(output.contains("var _jsxFileName2 = \"\";\n"));

    let options = JsxOptions::default();
    let source = "export namespace _jsx { export const _jsx = <div />; }";
    let output = lower_tsx(source, &options);
    let (helper, references) = helper_references(&output);
    assert_eq!(references, [Some(helper)]);

    let source = "<div />; export namespace _jsx { export const _jsx = 0; }";
    let output = lower_tsx(source, &options);
    let (helper, references) = helper_references(&output);
    assert_eq!(references, [Some(helper)]);
}

#[test]
fn custom_element_names() {
    let classic = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let output = lower_jsx("<Foo-bar />;", &classic);
    assert!(output.contains("React.createElement(\"Foo-bar\", null)"));

    let output = lower_jsx("<Foo-bar />;", &JsxOptions::default());
    assert!(output.contains("_jsx(\"Foo-bar\", {})"));
}

#[test]
fn development_lexical_environments() {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let output = lower_jsx("class B { constructor() { <div />; } }", &options);
    assert!(output.contains("__self: this"));

    let source = "class D extends B { constructor() { super(); <div />; } }";
    let output = lower_jsx(source, &options);
    assert!(!output.contains("__self"));

    let source = "class D extends B { constructor() { super(); const nested = () => <div />; } }";
    let output = lower_jsx(source, &options);
    assert!(!output.contains("__self"));

    let source =
        "class D extends B { constructor() { super(); function f() { return <div />; } } }";
    let output = lower_jsx(source, &options);
    assert!(output.contains("__self: this"));

    let source = "class D extends B { constructor() { class X extends (<div />, B) {} super(); } }";
    let output = lower_jsx(source, &options);
    assert!(!output.contains("__self"));

    let source = "class D extends B { constructor() { class X { [<div />]() {} } super(); } }";
    let output = lower_jsx(source, &options);
    assert!(!output.contains("__self"));

    let source = "class D extends B { constructor() { class X { [<div />] = 1; } super(); } }";
    let output = lower_jsx(source, &options);
    assert!(!output.contains("__self"));

    let source = "class D extends B { constructor() { class X { x = () => <div />; } super(); } }";
    let output = lower_jsx(source, &options);
    assert!(output.contains("__self: this"));

    let source = "class D extends B { constructor() { class X { static { <div />; } } super(); } }";
    let output = lower_jsx(source, &options);
    assert!(output.contains("__self: this"));

    let source =
        "class D extends B { constructor() { class X extends (() => <X />)() {} super(); } }";
    let output = lower_jsx(source, &options);
    assert!(!output.contains("__self"));

    let source = "class D extends B { constructor() { class X extends (function() { return <div />; })() {} super(); } }";
    let output = lower_jsx(source, &options);
    assert!(output.contains("__self: this"));
}

#[test]
fn development_source_positions() {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let output = lower_jsx("'😀', <div />;", &options);
    assert!(output.contains("lineNumber: 1, columnNumber: 7"));

    let output = lower_jsx("\r\n<div />;", &options);
    assert!(output.contains("lineNumber: 2, columnNumber: 1"));

    let output = lower_jsx("\r\n\r<div />;", &options);
    assert!(output.contains("lineNumber: 3, columnNumber: 1"));

    let output = lower_jsx("\r\n\r\u{2028}<div />;", &options);
    assert!(output.contains("lineNumber: 4, columnNumber: 1"));

    let output = lower_jsx("\r\n\r\u{2028}\u{2029}<div />;", &options);
    assert!(output.contains("lineNumber: 5, columnNumber: 1"));

    let output = lower_jsx("<div>\n  <span />\n</div>;", &options);
    assert!(output.contains("lineNumber: 1, columnNumber: 1"));
    assert!(output.contains("lineNumber: 2, columnNumber: 3"));
}
