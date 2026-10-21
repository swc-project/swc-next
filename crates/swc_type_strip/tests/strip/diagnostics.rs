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
    assert_strip_error_at(
        "namespace N { const enum E {} let x; }",
        ";                                     ",
        "namespace N { const enum E {} let x; }",
    );

    assert_strip_error_at(
        "namespace N { interface I { [<number>1]: string; } const enum E {} }",
        ";                                                                   ",
        "<number>",
    );

    assert_strip_error_at(
        "namespace N { declare namespace M { const enum E {} } const enum F {} }",
        ";                                                                      ",
        "const enum F {}",
    );

    assert_strip_error_at(
        "namespace N { const enum E { A } }",
        ";                                 ",
        "const enum E { A }",
    );

    assert_strip_error_at(
        "export namespace N { export const enum E { A = 1 } }\nconst value = N.E.A;",
        ";                                                   \nconst value = N.E.A;",
        "const enum E { A = 1 }",
    );

    assert_strip_error_at(
        "namespace N.M { const enum E { A } }",
        ";                                   ",
        "const enum E { A }",
    );

    assert_strip_error_at(
        "namespace N { import x = A.x; }\nrun();",
        ";                              \nrun();",
        "import x = A.x;",
    );

    assert_strip_error_at(
        "export namespace N { import type X = require('pkg'); }",
        ";                                                     ",
        "import type X = require('pkg');",
    );

    assert_strip_error_at(
        "declare namespace N { class C { constructor(public x: number); } }",
        ";                                                                 ",
        "public x: number",
    );

    assert_strip_error_at(
        "declare module 'pkg' { class C { constructor(readonly x: number); } }",
        ";                                                                    ",
        "readonly x: number",
    );

    assert_strip_error_at(
        "declare namespace N { enum E { A = <number>1 } }",
        ";                                               ",
        "<number>",
    );

    assert_strip_error_at(
        "namespace N { interface I { [<number>1]: string; } }",
        ";                                                   ",
        "<number>",
    );

    assert_strip_error_at(
        "namespace N { interface T { [<number>1]: string; } export type { T }; }",
        ";                                                                      ",
        "<number>",
    );

    assert_strip_error_at(
        "module 'pkg' { const x = 1; }",
        ";                            ",
        "module 'pkg' { const x = 1; }",
    );

    assert_strip_error_at(
        "declare global { class C { constructor(public x: number); } }",
        ";                                                            ",
        "public x: number",
    );

    assert_strip_error_at(
        "global { const enum E { A } }",
        ";                            ",
        "const enum E { A }",
    );

    assert_strip_error(
        "namespace N { const value: number = 1; }",
        ";                                       ",
    );

    assert_strip_error("namespace N { class C {} }", ";                         ");

    assert_strip_error(
        "namespace N { interface T {} export { type T, value }; }",
        ";                                                       ",
    );

    assert_strip_error(
        "namespace N { interface T {} export { value, type T }; }",
        ";                                                       ",
    );

    assert_strip_error(
        "namespace N { export type { T } from 'pkg'; }",
        ";                                            ",
    );

    assert_strip_error(
        "namespace N { export { type T } from 'pkg'; }",
        ";                                            ",
    );

    assert_strip_error(
        "namespace N { export type * from 'pkg'; }",
        ";                                        ",
    );

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

    assert_strip_error_at(
        "namespace N { \"use strict\"; }",
        ";                            ",
        "namespace N { \"use strict\"; }",
    );

    assert_strip_error_at(
        "namespace N { ; }",
        ";                ",
        "namespace N { ; }",
    );

    assert_strip_error_at(
        "namespace N { declare namespace M { const x: number; } }",
        ";                                                       ",
        "namespace N { declare namespace M { const x: number; } }",
    );

    assert_strip_error_at(
        "namespace N { declare enum E { A } }",
        ";                                   ",
        "namespace N { declare enum E { A } }",
    );
}

#[test]
fn module_syntax() {
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

    assert_strip_error(
        "import type Model = require('model');\nrun();",
        ";                                    \nrun();",
    );
}
