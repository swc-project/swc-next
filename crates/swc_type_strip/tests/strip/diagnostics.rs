use super::*;

#[test]
fn enums() {
    assert_strip_error("enum E { A }", "enum E { A }");

    assert_strip_error("const enum E { A }", "const enum E { A }");

    assert_strip_error(
        "enum E { A = value as Type }",
        "enum E { A = value         }",
    );
}

#[test]
fn namespaces() {
    assert_strip_error(
        "namespace N { const value: number = 1; }",
        ";                                       ",
    );

    assert_strip_error("namespace N { class C {} }", ";                         ");

    assert_strip_error("namespace N { {} }", ";                 ");

    assert_strip_error(
        "namespace N { declare let value: number; }",
        ";             ;                           ",
    );

    assert_strip_error(
        "namespace N { export function f(): void; }",
        ";             ;                           ",
    );

    assert_strip_error(
        "namespace N { export import x = A.x; }",
        ";                                     ",
    );

    assert_strip_error(
        "namespace A { namespace B { export const value = 1; } }",
        ";             ;                                        ",
    );
}

#[test]
fn module_syntax() {
    assert_strip_error(
        "module M { type T = string; }",
        ";          ;                 ",
    );

    assert_strip_error("declare module M {}", ";                  ");

    assert_strip_error(
        "import lib = require('lib');",
        "import lib = require('lib');",
    );

    assert_strip_error("export = value as Type;", "export = value        ;");

    assert_strip(
        "import type Model = require('model');\nrun();",
        ";                                    \nrun();",
    );
}
