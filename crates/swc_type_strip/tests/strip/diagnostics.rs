use super::*;

#[test]
fn enums() {
    assert_strip_error("enum E { A }", ";           ");

    assert_strip_error("const enum E { A }", ";                 ");

    assert_strip_error(
        "enum E { A = value as Type }",
        ";                           ",
    );

    assert_strip_error_at("export enum E { A }", ";                  ", "enum E { A }");

    assert_strip_error_at(
        "export const enum E { A }",
        ";                        ",
        "const enum E { A }",
    );

    assert_strip_error(
        "enum E { A = (() => { type T = number; return 1; })() }",
        ";                                                      ",
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
        ";                                         ",
    );

    assert_strip_error(
        "namespace N { export function f(): void; }",
        ";                                         ",
    );

    assert_strip_error(
        "namespace N { export import x = A.x; }",
        ";                                     ",
    );

    assert_strip_error(
        "namespace A { namespace B { export const value = 1; } }",
        ";                                                      ",
    );

    assert_strip_error_at(
        "export namespace N { const x = 1; }",
        ";                                  ",
        "namespace N { const x = 1; }",
    );

    assert_strip_error_at(
        "export namespace N.M { const x = 1; }",
        ";                                    ",
        "namespace N.M { const x = 1; }",
    );

    assert_strip_error_at(
        "export namespace N { namespace M { const x = 1; } }",
        ";                                                  ",
        "namespace N { namespace M { const x = 1; } }",
    );

    assert_strip_error_at(
        "run();\nexport namespace N { const x = 1; }",
        "run();\n                                   ",
        "namespace N { const x = 1; }",
    );

    assert_strip_error_at(
        "run()\nexport namespace N { const x = 1; }\n(next)();",
        "run()\n;                                  \n(next)();",
        "namespace N { const x = 1; }",
    );
}

#[test]
fn module_syntax() {
    assert_strip_error(
        "module M { type T = string; }",
        ";                            ",
    );

    assert_strip_error("declare module M {}", ";                  ");

    assert_strip_error_at(
        "export module M { const x = 1; }",
        ";                               ",
        "module M { const x = 1; }",
    );

    assert_strip_error(
        "import lib = require('lib');",
        ";                           ",
    );

    assert_strip_error("export = value as Type;", ";                      ");

    assert_strip_error(
        "export = (() => { type T = number; return 1; })();",
        ";                                                 ",
    );

    assert_strip(
        "import type Model = require('model');\nrun();",
        ";                                    \nrun();",
    );
}
