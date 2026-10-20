use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{ArrowFunctionExpression, Ast, Lang, SourceType, Visit, VisitWith};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

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
            jsx: Some(jsx),
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
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
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
        jsx: Some(JsxOptions {
            runtime: JsxRuntime::Classic,
            development: true,
            source_self: true,
            ..JsxOptions::default()
        }),
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
    let output = print(&allocator, &ast, CodegenOptions::default())
        .code
        .to_owned();
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
fn spread_attributes() {
    let source_type = SourceType::Module;
    let lang = Lang::Jsx;
    let options = JsxOptions {
        runtime: JsxRuntime::Classic,
        ..JsxOptions::default()
    };
    let source = "<div {...props} />;";
    let output = transform(source, source_type, lang, options.clone(), false);
    assert_eq!(
        output,
        "/* #__PURE__ */ React.createElement(\"div\", props);\n"
    );

    let source = "<div first={before()} {...{ second: middle() }} third={after()} />;";
    let output = transform(source, source_type, lang, options.clone(), false);
    assert_eq!(
        output,
        "/* #__PURE__ */ React.createElement(\"div\", { first: before(), second: middle(), third: after() });\n"
    );

    let source = "<div {...{ __proto__: proto, value: 1 }} />;";
    let output = transform(source, source_type, lang, options.clone(), false);
    assert_eq!(
        output,
        "/* #__PURE__ */ React.createElement(\"div\", { ...{ __proto__: proto, value: 1 } });\n"
    );

    let source = "<Comp {...{ get value() { sideEffect(); return 1; } }} other={2} />;";
    let output = transform(source, source_type, lang, options, false);
    assert!(output.contains("React.createElement(Comp, { ...{ get value()"));

    let options = JsxOptions::default();
    let source = "<Comp {...{ get value() { sideEffect(); return 1; } }} />;";
    let output = transform(source, source_type, lang, options.clone(), false);
    assert!(output.contains("_jsx(Comp, { ...{ get value()"));

    let source = "<Comp {...{ set value(v) { sideEffect(v); } }} />;";
    let output = transform(source, source_type, lang, options, false);
    assert!(output.contains("_jsx(Comp, { ...{ set value(v)"));
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
    let output = transform(source, SourceType::Module, Lang::Tsx, options, true);
    assert!(output.contains("N.h(N.Frag, null, N.h(\"div\", null))"));
}

#[test]
fn pure_calls() {
    let options = JsxOptions::default();
    let output = transform(
        "(<A f={tick} />).props.f();",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("(/* #__PURE__ */ _jsx(A, { f: tick })).props.f()"));
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
        ..JsxOptions::default()
    };
    let output = transform(
        "const node = <Foo />;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("_jsxDEV(Foo, {}, void 0, false)"));
    assert!(!output.contains("_jsxFileName"));

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
        source_self: true,
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

    let options = TransformerOptions {
        jsx: Some(JsxOptions {
            runtime: JsxRuntime::Classic,
            development: true,
            source_self: true,
            ..JsxOptions::default()
        }),
        typescript: Some(TypeScriptOptions::default()),
        ..TransformerOptions::default()
    };
    let source = "namespace N { export const node = <div />; }";
    let (output, diagnostics) = transform_options(source, SourceType::Module, Lang::Tsx, options);
    assert!(diagnostics.is_empty());
    assert!(output.contains("N.node = /* #__PURE__ */ React.createElement(\"div\", {"));
    assert!(output.contains("fileName: _jsxFileName"));
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
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
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
fn refresh_signatures() {
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
        ..JsxOptions::default()
    };
    let output = transform(
        "export default function App() { const [foo, setFoo] = useState(0); React.useEffect(() => {}); return <h1>{foo}</h1>; }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("var _s = $RefreshSig$();"));
    assert!(output.contains("function App() {\n  _s();"));
    assert!(output.contains("_s(App, \"useState{[foo, setFoo](0)}\\\\nuseEffect{}\");"));
    assert!(output.contains("$RefreshReg$(_c, \"App\");"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
        ..JsxOptions::default()
    };
    let output = transform(
        "const useExternal = () => {}; function App(useParameter) { const useLocal = () => {}; useLocal(); useParameter(); useExternal(); }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("return [useExternal];"));
    assert!(!output.contains("return [useLocal"));
    assert!(!output.contains("return [useParameter"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
        ..JsxOptions::default()
    };
    let output = transform(
        "function App() { const state = useState(() => { useCustom(); return 0; }); }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("_s2(App, \"useState{state(() => { useCustom(); return 0; })}\");"));
}

#[test]
fn refresh_control_flow() {
    let source = "switch (1) { case 0: function Foo() { useState(); } break; case 1: Foo(); }";
    let output = refresh(source);
    assert!(output.starts_with("var _s = $RefreshSig$();\nswitch"));
    assert!(output.contains("_s(Foo, \"useState{}\");\n    break;"));

    let source = "function run() { switch (1) { case 0: function Foo() { useState(); } break; case 1: Foo(); } }";
    let output = refresh(source);
    assert!(output.contains("function run() {\n  var _s = $RefreshSig$();\n  switch"));

    let source = "switch (1) { case 0: switch (2) { case 1: function Foo() { useState(); } break; case 2: Foo(); } }";
    let output = refresh(source);
    assert!(output.starts_with("var _s = $RefreshSig$();\nswitch"));

    let output = refresh("const Foo = () => useState(0);");
    assert!(output.contains("const Foo = () => {\n  _s();\n  return useState(0);\n};"));

    let source_type = SourceType::Script;
    let lang = Lang::Jsx;
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
        ..JsxOptions::default()
    };
    let source = "if (flag) function App() { useState(); }";
    let output = transform(source, source_type, lang, options.clone(), false);
    assert!(output.starts_with("var _s = $RefreshSig$();\nif"));
    assert!(output.contains("  _s(App, \"useState{}\");\n}"));

    let source = "if (flag) function App() { useState(); } else function App() { useEffect(); }";
    let output = transform(source, source_type, lang, options.clone(), false);
    assert!(output.starts_with("var _s = $RefreshSig$(), _s2 = $RefreshSig$();\nif"));
    assert!(output.contains("  _s(App, \"useState{}\");\n}"));
    assert!(output.contains("  _s2(App, \"useEffect{}\");\n}"));

    let source = "if (flag) if (other) function App() { useState(); }";
    let output = transform(source, source_type, lang, options.clone(), false);
    assert!(output.starts_with("var _s = $RefreshSig$();\nif"));
    assert!(output.contains("if (flag) if (other) {"));
    assert!(output.contains("  _s(App, \"useState{}\");\n}"));

    let source = "function run(flag) { if (flag) function App() { useState(); } }";
    let output = transform(source, source_type, lang, options, false);
    assert!(output.contains("function run(flag) {\n  var _s = $RefreshSig$();\n  if"));
    assert!(output.contains("    _s(App, \"useState{}\");\n  }"));
}

#[test]
fn refresh_boundaries() {
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
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
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
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

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            refresh_reg: "import.meta.refreshReg".to_owned(),
            refresh_sig: "import.meta.refreshSig".to_owned(),
            emit_full_signatures: true,
        }),
        ..JsxOptions::default()
    };
    let output = transform(
        "export default function App() { useContext(X); return <Foo />; }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("var _s = import.meta.refreshSig();"));
    assert!(output.contains("import.meta.refreshReg(_c, \"App\");"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
        ..JsxOptions::default()
    };
    let output = transform(
        "export function Shallow() { A.B.C.useHook(); } export function Supported() { const value = GlobalHook.property.useNestedThing(); }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("_s(Shallow, \"useHook{}\");"));
    assert!(output.contains("_s2(Supported, \"useNestedThing{value}\", true);"));
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
fn refresh_hook_scope() {
    let output = refresh("function App() { const useLocal = () => {}; useLocal(); }");
    assert!(output.contains("_s(App, \"useLocal{}\", true);"));

    let output = refresh("function App(useThing) { useThing(); }");
    assert!(output.contains("_s(App, \"useThing{}\", true);"));

    let output = refresh("const useThing = () => {}; function App() { useThing(); }");
    assert!(output.contains("_s(App, \"useThing{}\", false, function()"));
    assert!(output.contains("return [useThing];"));
}

#[test]
fn refresh_builtins() {
    let output = refresh("import { use } from 'react'; function App() { use(promise); }");
    assert!(!output.contains("$RefreshSig$"));
    assert!(output.contains("$RefreshReg$(_c, \"App\");"));

    let output = refresh("import React from 'react'; function App() { React.use(promise); }");
    assert!(!output.contains("$RefreshSig$"));

    let output = refresh("function App() { use(promise); useState(0); }");
    assert!(output.contains("_s(App, \"useState{(0)}\");"));
    assert!(!output.contains("return [use]"));
}

#[test]
fn refresh_methods() {
    let output = refresh("const o = { get value() { useThing(); } };");
    assert!(output.contains("get value()"));
    assert!(!output.contains("$RefreshSig$"));

    let output = refresh("const o = { set value(v) { useThing(v); } };");
    assert!(output.contains("set value(v)"));
    assert!(!output.contains("$RefreshSig$"));

    let output = refresh("const o = { method() { useThing(); return super.value; } };");
    assert!(output.contains("method()"));
    assert!(output.contains("return super.value;"));
    assert!(!output.contains("$RefreshSig$"));

    let output = refresh("const o = { value: function() { useState(); } };");
    assert!(output.contains("value: _s(function()"));
    assert!(output.contains("\"useState{}\""));

    let output = refresh("const o = { get value() { return () => useState(0); } };");
    assert!(output.contains("get value()"));
    assert!(output.contains("return _s(() =>"));
    assert!(output.contains("\"useState{(0)}\""));
}

#[test]
fn refresh_signature_flow() {
    let output = refresh("const App = memo(flag ? () => useState(0) : null);");
    assert!(output.contains("memo(flag ? _s("));
    assert!(!output.contains("_s(memo("));

    let output = refresh("const App = memo({ render: () => useState(0) });");
    assert!(output.contains("memo({ render: _s("));
    assert!(!output.contains("_s(memo("));

    let output = refresh("const App = memo(assigned = () => useState(0));");
    assert!(output.contains("_s(memo(assigned = _s("));

    let output = refresh("const App = memo(assigned ||= () => useState(0));");
    assert!(!output.contains("_s(memo("));

    let output = refresh("for (let useThing = () => useState(0); ready; ) useThing();");
    assert!(output.contains("for (let useThing = _s("));
    assert!(!output.contains("_s(useThing,"));
}

#[test]
fn refresh_typescript() {
    let output = refresh_lang("const App = (() => <div />) satisfies F;", Lang::Tsx);
    assert!(output.contains("const App = () =>"));
    assert!(output.contains("$RefreshReg$(_c, \"App\");"));

    let output = refresh_lang("const App = (() => useState(0)) as F;", Lang::Tsx);
    assert!(output.contains("const App = () =>"));
    assert!(output.contains("_s(App, \"useState{(0)}\");"));

    let output = refresh_lang("const App = memo((() => useState(0)) as F);", Lang::Tsx);
    assert!(output.contains("_s(memo(_c = _s("));
    assert!(output.contains("$RefreshReg$(_c, \"App$memo\");"));

    let output = refresh_lang("function App() { (useState as any)(0); }", Lang::Tsx);
    assert!(output.contains("_s(App, \"useState{(0)}\");"));

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
            refresh_sig: "_s".to_owned(),
            ..ReactRefreshOptions::default()
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
    assert!(output.contains("var _s2 = _s();"));
    assert!(output.contains("_c(_c2, \"App\");"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            refresh_reg: "_s.refreshReg".to_owned(),
            refresh_sig: "_c.refreshSig".to_owned(),
            ..ReactRefreshOptions::default()
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
    assert!(output.contains("var _s2 = _c.refreshSig();"));
    assert!(output.contains("_s.refreshReg(_c2, \"App\");"));

    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions {
            refresh_sig: "useSig".to_owned(),
            emit_full_signatures: true,
            ..ReactRefreshOptions::default()
        }),
        ..JsxOptions::default()
    };
    let output = transform(
        "function Outer() { function Inner() { useState(0); } useEffect(() => {}); }",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    assert!(output.contains("_s(Inner, \"useState{(0)}\");"));
    assert!(output.contains("_s2(Outer, \"useEffect{}\");"));
}

#[test]
fn refresh_import_order() {
    let options = JsxOptions {
        refresh: Some(ReactRefreshOptions::default()),
        ..JsxOptions::default()
    };
    let output = transform(
        "const _source = initialize(); import { value } from 'source'; export const View = () => <div />;",
        SourceType::Module,
        Lang::Jsx,
        options,
        false,
    );
    let source_import = output.find("from \"source\"").unwrap();
    let runtime_import = output.find("from \"react/jsx-runtime\"").unwrap();
    assert!(source_import < runtime_import);

    let source = "init(); import x from 'x'; export function App() { useState(); return <div />; }";
    let output = refresh(source);
    let source_import = output.find("from \"x\"").unwrap();
    let runtime_import = output.find("from \"react/jsx-runtime\"").unwrap();
    assert!(source_import < runtime_import);

    let source = "const App=()=>{ useState(); return <div/>; }; import 'first'; import 'last';";
    let output = refresh(source);
    let first_import = output.find("import \"first\"").unwrap();
    let last_import = output.find("import \"last\"").unwrap();
    let runtime_import = output.find("from \"react/jsx-runtime\"").unwrap();
    assert!(first_import < last_import);
    assert!(last_import < runtime_import);
    let signature_init = output.find("var _s = $RefreshSig$();").unwrap();
    let signature_call = output.find("_s(App,").unwrap();
    assert!(signature_init < signature_call);
}

#[test]
fn generated_bindings() {
    let options = TransformerOptions {
        jsx: Some(JsxOptions {
            development: true,
            source_self: true,
            ..JsxOptions::default()
        }),
        ..TransformerOptions::default()
    };
    let (output, diagnostics) = transform_options(
        "const _jsxFileName = 'source'; const render = () => <div />;",
        SourceType::Script,
        Lang::Jsx,
        options,
    );
    assert!(diagnostics.is_empty());
    assert!(output.contains("var _jsxFileName2 = \"\";\n"));
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
}
