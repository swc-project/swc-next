use super::{TypeScriptOptions, transformed_code_with_language};
use crate::typescript::JsxImportOptions;
use swc_next_ecma_ast::Lang;

fn transform(source: &str, jsx: JsxImportOptions) -> String {
    transformed_code_with_language(
        source,
        TypeScriptOptions {
            jsx,
            ..Default::default()
        },
        false,
        Lang::Tsx,
    )
}

#[test]
fn implicit_imports() {
    let code = transform(
        "import React from 'react'; export const x = <div />;",
        Default::default(),
    );
    assert!(code.contains("import React from"));

    let code = transform(
        "import React from 'react'; export const x = 1;",
        Default::default(),
    );
    assert!(!code.contains("import React"));

    let code = transform(
        "import React from 'react'; function f(React: any) { return <div />; }",
        Default::default(),
    );
    assert!(!code.contains("import React"));

    let code = transform(
        "import type React from 'react'; export const x = <div />;",
        Default::default(),
    );
    assert!(!code.contains("import"));

    let code = transform(
        "import React = require('react'); export const x = <div />;",
        Default::default(),
    );
    assert!(code.contains("React"));
    assert!(code.contains("import.sync"));

    let code = transform(
        "import React from 'react'; export const x = <div />;",
        JsxImportOptions {
            automatic: true,
            ..Default::default()
        },
    );
    assert!(!code.contains("import React"));
}

#[test]
fn custom_factories() {
    let options = || JsxImportOptions {
        pragma: Some("h".into()),
        pragma_frag: Some("Frag".into()),
        ..Default::default()
    };
    let code = transform(
        "import { h, Frag } from 'react'; export const x = <div />;",
        options(),
    );
    assert!(code.contains("{ h }"));
    assert!(!code.contains("Frag"));

    let code = transform(
        "import { h, Frag } from 'react'; export const x = <></>;",
        options(),
    );
    assert!(code.contains("h, Frag"));

    let code = transform(
        "/** @jsx h @jsxFrag Frag */ import { h, Frag } from 'react'; export const x = <></>;",
        Default::default(),
    );
    assert!(code.contains("import { h, Frag }"));

    let code = transform(
        "/** @jsxRuntime automatic */ import React from 'react'; export const x = <div />;",
        Default::default(),
    );
    assert!(!code.contains("import React"));

    let code = transform(
        "namespace N { export function h() {} } import h = N.h; export const x = <div />;",
        options(),
    );
    assert!(code.contains("var h = N.h"));
    assert!(code.contains("var N"));
}

#[test]
fn pragma_imports() {
    let code = transform(
        "/** @jsxFactory h */ import React from 'react'; export const x = <div />;",
        Default::default(),
    );
    assert!(code.contains("import React from"));

    let code = transform(
        "/** @jsxRuntimeautomatic */ import React from 'react'; export const x = <div />;",
        Default::default(),
    );
    assert!(code.contains("import React from"));

    let code = transform(
        "/** @jsx\n * @jsx h */ import React from 'react'; import { h } from 'h'; export const x = <div />;",
        Default::default(),
    );
    assert!(code.contains("import { h }"));
    assert!(!code.contains("import React"));
}
