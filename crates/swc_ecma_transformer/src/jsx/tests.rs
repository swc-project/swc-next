use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

use super::{JsxOptions, JsxRuntime, ReactRefreshOptions};
use crate::{Transformer, TransformerOptions, typescript::TypeScriptOptions};

fn transform(
    source: &str,
    source_type: SourceType,
    lang: Lang,
    jsx: JsxOptions,
    typescript: bool,
) -> String {
    transform_with_diagnostics(source, source_type, lang, jsx, typescript).0
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
            jsx: Some(jsx),
            typescript: typescript.then(TypeScriptOptions::default),
            ..TransformerOptions::default()
        },
    )
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

fn classic_plugins(source: &str) -> String {
    let options = TransformerOptions {
        source_path: Some("input.jsx".to_owned()),
        react_jsx_self: true,
        react_jsx_source: true,
        jsx: Some(JsxOptions {
            runtime: JsxRuntime::Classic,
            ..JsxOptions::default()
        }),
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
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let expected_lang = if options.jsx.is_some() {
        Lang::Js
    } else {
        lang
    };
    let transformed = Transformer::transform(&mut ast, &semantic, options);
    assert_eq!(ast.lang(), expected_lang);
    let output = print(&allocator, &ast, CodegenOptions::default())
        .code
        .to_owned();
    (output, transformed.diagnostics)
}

#[test]
fn classic() {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let output = transform(
        "const node = <Foo disabled>hello</Foo>;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert_eq!(
        output,
        "const node = /* #__PURE__ */ React.createElement(Foo, { disabled: true }, \"hello\");\n"
    );

    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let output = transform(
        "/** @jsx h */ const node = <><this /><foo.Bar /></>;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("h(React.Fragment"));
    assert!(output.contains("h(this, null)"));
    assert!(output.contains("h(foo.Bar, null)"));
}

#[test]
fn automatic() {
    let output = transform(
        "const node = <><div key=\"a\" /><span /></>;",
        SourceType::Module,
        Lang::Jsx,
        JsxOptions::default(),
        false,
    );
    assert!(output.starts_with(
        "import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from \"react/jsx-runtime\";"
    ));
    assert!(output.contains("_jsx(\"div\", {}, \"a\")"));

    let output = transform(
        "var node = <div />;",
        SourceType::Script,
        Lang::Jsx,
        JsxOptions::default(),
        false,
    );
    assert!(output.starts_with("var _reactJsxRuntime = require(\"react/jsx-runtime\");"));
    assert!(output.contains("_reactJsxRuntime.jsx(\"div\", {})"));
}

#[test]
fn tsx_composition() {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let output = transform(
        "import React, { type FC } from 'react'; const node: JSX.Element = <div />;",
        SourceType::Module,
        Lang::Tsx,
        options,
        true,
    );
    assert!(output.starts_with("import React from \"react\";"));
    assert!(output.contains("const node = /* #__PURE__ */ React.createElement(\"div\", null);"));
    assert!(!output.contains("FC"));
    assert!(!output.contains("JSX.Element"));

    let output = transform(
        "import React from 'react'; const node = <div />;",
        SourceType::Module,
        Lang::Tsx,
        JsxOptions::default(),
        true,
    );
    assert!(!output.contains("import React"));

    let classic = JsxOptions {
        runtime: JsxRuntime::Classic,
        pragma: Some("h".into()),
        pragma_frag: Some("Frag".into()),
        ..JsxOptions::default()
    };
    let output = transform(
        "import { h, Frag } from 'ui'; const node = <></>;",
        SourceType::Module,
        Lang::Tsx,
        classic.clone(),
        true,
    );
    assert!(output.contains("h, Frag"));
    assert!(output.contains("h(Frag, null)"));

    let output = transform(
        "import { h, Frag } from 'ui'; const node = <div />;",
        SourceType::Module,
        Lang::Tsx,
        classic.clone(),
        true,
    );
    assert!(output.contains("import { h }"));
    assert!(!output.contains("Frag"));

    let output = transform(
        "import { h } from 'ui'; function render(h: any) { return <div />; }",
        SourceType::Module,
        Lang::Tsx,
        classic.clone(),
        true,
    );
    assert!(!output.contains("from \"ui\""));

    let output = transform(
        "import h = require('ui'); const node = <div />;",
        SourceType::Module,
        Lang::Tsx,
        classic,
        true,
    );
    assert!(output.contains("import.sync(\"ui\")"));
}

#[test]
fn attribute_comments() {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let output = transform(
        "const node = <Trans /* before */ a=\"1\" /** between */ b=\"2\" /* after */ />;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("Trans,\n  /* before */\n  {"));
    assert!(output.contains("/** between */ b: \"2\" /* after */"));
}

#[test]
fn pragmas() {
    let output = transform(
        "/** @jsxRuntime classic @jsx h @jsxFrag Frag */ const node = <></>;",
        SourceType::Module,
        Lang::Jsx,
        JsxOptions::default(),
        false,
    );
    assert!(output.contains("h(Frag, null)"));

    let output = transform(
        "const before = 1; /** @jsxRuntime classic */ const node = <div />;",
        SourceType::Module,
        Lang::Jsx,
        JsxOptions::default(),
        false,
    );
    assert!(output.starts_with("import { jsx as _jsx } from \"react/jsx-runtime\";"));

    let output = transform(
        "/** @jsxImportSource custom */ const node = <div />;",
        SourceType::Module,
        Lang::Jsx,
        JsxOptions::default(),
        false,
    );
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
    let options = JsxOptions {
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let output = transform(
        "const node = <Foo><span /></Foo>;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("jsxDEV as _jsxDEV"));
    assert!(output.contains("var _jsxFileName = \"\";"));
    assert!(output.contains("_jsxDEV(Foo"));
    assert!(output.contains("fileName: _jsxFileName"));
    assert!(output.contains(", this)"));

    let options = JsxOptions {
        development: true,
        jsx_self: true,
        ..JsxOptions::default()
    };
    let output = transform(
        "const node = <Foo />;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("_jsxDEV(Foo, {}, void 0, false, void 0, this)"));

    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let output = transform(
        "const node = <Foo />;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("{ __self: this, __source: {"));

    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        development: true,
        jsx_self: true,
        ..JsxOptions::default()
    };
    let output = transform(
        "class Base { constructor() { <A />; } } class Derived extends Base { constructor() { super(); <B />; const nested = () => <C />; function allowed() { return <D />; } } }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert_eq!(output.matches("__self: this").count(), 2);
}

#[test]
fn development_plugins() {
    let output = classic_plugins("<div {...props} />;");
    assert!(output.starts_with("var _jsxFileName = \"input.jsx\";"));
    assert!(output.contains("React.createElement(\"div\", { ...props, __self: this, __source:"));

    let output = classic_plugins("class D extends B { constructor() { <div />; } }");
    assert!(!output.contains("__self"));

    let options = JsxOptions {
        jsx_self: true,
        jsx_source: true,
        ..JsxOptions::default()
    };
    let output = transform("<div />;", SourceType::Module, Lang::Jsx, options, false);
    assert!(output.contains("_jsx(\"div\", {})"));
}

#[test]
fn diagnostics() {
    let (_, diagnostics) = transform_with_diagnostics(
        "const node = <div key />;",
        SourceType::Module,
        Lang::Jsx,
        JsxOptions::default(),
        false,
    );
    assert_eq!(diagnostics.len(), 1);

    let (_, diagnostics) = transform_with_diagnostics(
        "const node = <svg:path />;",
        SourceType::Module,
        Lang::Jsx,
        JsxOptions::default(),
        false,
    );
    assert_eq!(diagnostics.len(), 1);

    let options = JsxOptions {
        throw_if_namespace: false,
        ..JsxOptions::default()
    };
    let (output, diagnostics) = transform_with_diagnostics(
        "const node = <svg:path />;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(diagnostics.is_empty());
    assert!(output.contains("_jsx(\"svg:path\""));
}

#[test]
fn refresh_registration() {
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions::default()),
        ..JsxOptions::default()
    };
    let output = transform(
        "function Hello() { return <h1>Hi</h1>; } const Bar = memo(() => <Hello />);",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("_c = Hello;"));
    assert!(output.contains("const Bar = memo(_c2 = () =>"));
    assert!(output.contains("_c3 = Bar;"));
    assert!(output.contains("$RefreshReg$(_c, \"Hello\");"));
    assert!(output.contains("$RefreshReg$(_c2, \"Bar$memo\");"));
    assert!(output.contains("$RefreshReg$(_c3, \"Bar\");"));
}

#[test]
fn refresh_boundaries() {
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions::default()),
        ..JsxOptions::default()
    };
    let output = transform(
        "const A = require('A'); const Header = styled.div``; function App() { return <><A /><Header /></>; }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(!output.contains(", \"A\");"));
    assert!(output.contains(", \"Header\");"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions::default()),
        ..JsxOptions::default()
    };
    let output = transform(
        "namespace Foo { export namespace Bar { export const A = () => null; function B() {} } namespace Hidden { export const C = () => null; } }",
        SourceType::Module,
        Lang::Tsx,
        options,
        true,
    );
    assert!(output.contains(", \"Foo$Bar$A\");"));
    assert!(output.contains(", \"Foo$Bar$B\");"));
    assert!(!output.contains("Foo$Hidden$C"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions::default()),
        ..JsxOptions::default()
    };
    let output = transform(
        "namespace Foo /* comment */ . Bar { export function App() {} }",
        SourceType::Module,
        Lang::Tsx,
        options.clone(),
        true,
    );
    assert!(output.contains(", \"Foo$Bar$App\");"));

    let output = transform(
        "namespace F\\u006fo.Bar { export function App() {} }",
        SourceType::Module,
        Lang::Tsx,
        options,
        true,
    );
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
    let output = transform(
        "function App() { useState(0); }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("_c(_c2, \"App\");"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            refresh_reg: "_s.refreshReg".to_owned(),
        }),
        ..JsxOptions::default()
    };
    let output = transform(
        "function App() { useState(0); }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("_s.refreshReg(_c, \"App\");"));
}

#[test]
fn generated_bindings() {
    let options = TransformerOptions {
        es2015: crate::es2015::ES2015Options {
            arrow_functions: Some(crate::es2015::arrow::ArrowFunctionsOptions::default()),
            ..crate::es2015::ES2015Options::default()
        },
        ..TransformerOptions::default()
    };
    let (output, diagnostics) = transform_options(
        "const _jsxFileName = 'source'; const getThis = () => this;",
        SourceType::Script,
        Lang::Js,
        options,
    );
    assert!(diagnostics.is_empty());
    assert!(output.starts_with("var _this = this;\nconst _jsxFileName = \"source\";"));
}

#[test]
fn custom_element_names() {
    let classic = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let output = transform(
        "<Foo-bar />;",
        SourceType::Module,
        Lang::Jsx,
        classic,
        false,
    );
    assert!(output.contains("React.createElement(\"Foo-bar\", null)"));

    let output = transform(
        "<Foo-bar />;",
        SourceType::Module,
        Lang::Jsx,
        JsxOptions::default(),
        false,
    );
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
    let output = transform(
        "class D extends B { constructor() { class X extends (<div />, B) {} super(); } }",
        SourceType::Module,
        Lang::Jsx,
        options.clone(),
        false,
    );
    assert!(!output.contains("__self"));

    let output = transform(
        "class D extends B { constructor() { class X { [<div />]() {} } super(); } }",
        SourceType::Module,
        Lang::Jsx,
        options.clone(),
        false,
    );
    assert!(!output.contains("__self"));

    let output = transform(
        "class D extends B { constructor() { class X { [<div />] = 1; } super(); } }",
        SourceType::Module,
        Lang::Jsx,
        options.clone(),
        false,
    );
    assert!(!output.contains("__self"));

    let output = transform(
        "class D extends B { constructor() { class X { field = () => <div />; static { <span />; } } super(); } }",
        SourceType::Module,
        Lang::Jsx,
        options.clone(),
        false,
    );
    assert_eq!(output.matches("__self: this").count(), 2);

    let output = transform(
        "class D extends B { constructor() { class X extends (() => <div />)() {} super(); } }",
        SourceType::Module,
        Lang::Jsx,
        options.clone(),
        false,
    );
    assert!(!output.contains("__self"));

    let output = transform(
        "class D extends B { constructor() { class X extends (function() { return <div />; })() {} super(); } }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("__self: this"));

    let options = TransformerOptions {
        react_jsx_self: true,
        ..TransformerOptions::default()
    };
    let (output, diagnostics) = transform_options(
        "class D extends B { constructor() { class X extends (<div />, B) {} super(); } }",
        SourceType::Module,
        Lang::Jsx,
        options,
    );
    assert!(diagnostics.is_empty());
    assert!(!output.contains("__self"));
}

#[test]
fn development_source_positions() {
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        development: true,
        source_self: true,
        ..JsxOptions::default()
    };
    let output = transform(
        "'😀'; <div />;\r\n<span />;\r<p />;\u{2028}<a />;\u{2029}<b />;",
        SourceType::Module,
        Lang::Jsx,
        options.clone(),
        false,
    );
    assert!(output.contains("lineNumber: 1, columnNumber: 7"));
    assert!(output.contains("lineNumber: 2, columnNumber: 1"));
    assert!(output.contains("lineNumber: 3, columnNumber: 1"));
    assert!(output.contains("lineNumber: 4, columnNumber: 1"));
    assert!(output.contains("lineNumber: 5, columnNumber: 1"));

    let output = transform(
        "<div>\n  <span />\n</div>;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("lineNumber: 1, columnNumber: 1"));
    assert!(output.contains("lineNumber: 2, columnNumber: 3"));

    let options = TransformerOptions {
        react_jsx_source: true,
        ..TransformerOptions::default()
    };
    let (output, diagnostics) =
        transform_options("'😀'; <div />;", SourceType::Module, Lang::Jsx, options);
    assert!(diagnostics.is_empty());
    assert!(output.contains("lineNumber: 1, columnNumber: 7"));
}
