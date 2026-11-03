#[test]
fn enums() {
    assert_strip_with_error!("enum E { A }", ";           ");

    assert_strip_with_error!("const enum E { A }", ";                 ");

    assert_strip_with_error!(
        "enum E { A = value as Type }",
        ";                           ",
    );

    assert_strip_with_error!(
        "export enum E { A }",
        ";                  ",
        at: "enum E { A }",
    );

    assert_strip_with_error!(
        "export const enum E { A }",
        ";                        ",
        at: "const enum E { A }",
    );

    assert_strip_with_error!(
        "enum E { A = (() => { type T = number; return 1; })() }",
        ";                                                      ",
    );
}

#[test]
fn namespaces() {
    assert_strip_with_error!(
        "namespace N { const enum E {} let x; }",
        ";                                     ",
        at: "namespace N { const enum E {} let x; }",
    );

    assert_strip_with_error!(
        "namespace N { interface I { [<number>1]: string; } const enum E {} }",
        ";                                                                   ",
        at: "<number>",
    );

    assert_strip_with_error!(
        "namespace N { declare namespace M { const enum E {} } const enum F {} }",
        ";                                                                      ",
        at: "const enum F {}",
    );

    assert_strip_with_error!(
        "namespace N { const enum E { A } }",
        ";                                 ",
        at: "const enum E { A }",
    );

    assert_strip_with_error!(
        "export namespace N { export const enum E { A = 1 } }\nconst value = N.E.A;",
        ";                                                   \nconst value = N.E.A;",
        at: "const enum E { A = 1 }",
    );

    assert_strip_with_error!(
        "namespace N.M { const enum E { A } }",
        ";                                   ",
        at: "const enum E { A }",
    );

    assert_strip_with_error!(
        "namespace N { import x = A.x; }\nrun();",
        ";                              \nrun();",
        at: "import x = A.x;",
    );

    assert_strip_with_error!(
        "export namespace N { import type X = require('pkg'); }",
        ";                                                     ",
        at: "import type X = require('pkg');",
    );

    assert_strip_with_error!(
        "declare namespace N { class C { constructor(public x: number); } }",
        ";                                                                 ",
        at: "public x: number",
    );

    assert_strip_with_error!(
        "declare module 'pkg' { class C { constructor(readonly x: number); } }",
        ";                                                                    ",
        at: "readonly x: number",
    );

    assert_strip_with_error!(
        "declare namespace N { enum E { A = <number>1 } }",
        ";                                               ",
        at: "<number>",
    );

    assert_strip_with_error!(
        "namespace N { interface I { [<number>1]: string; } }",
        ";                                                   ",
        at: "<number>",
    );

    assert_strip_with_error!(
        "namespace N { interface T { [<number>1]: string; } export type { T }; }",
        ";                                                                      ",
        at: "<number>",
    );

    assert_strip_with_error!(
        "module 'pkg' { const x = 1; }",
        ";                            ",
        at: "module 'pkg' { const x = 1; }",
    );

    assert_strip_with_error!(
        "declare global { class C { constructor(public x: number); } }",
        ";                                                            ",
        at: "public x: number",
    );

    assert_strip_with_error!(
        "global { const enum E { A } }",
        ";                            ",
        at: "const enum E { A }",
    );

    assert_strip_with_error!(
        "namespace N { const value: number = 1; }",
        ";                                       ",
    );

    assert_strip_with_error!("namespace N { class C {} }", ";                         ");

    assert_strip_with_error!(
        "namespace N { interface T {} export { type T, value }; }",
        ";                                                       ",
    );

    assert_strip_with_error!(
        "namespace N { interface T {} export { value, type T }; }",
        ";                                                       ",
    );

    assert_strip_with_error!(
        "namespace N { export type { T } from 'pkg'; }",
        ";                                            ",
    );

    assert_strip_with_error!(
        "namespace N { export { type T } from 'pkg'; }",
        ";                                            ",
    );

    assert_strip_with_error!(
        "namespace N { export type * from 'pkg'; }",
        ";                                        ",
    );

    assert_strip_with_error!("namespace N { {} }", ";                 ");

    assert_strip_with_error!(
        "namespace N { declare let value: number; }",
        ";                                         ",
    );

    assert_strip_with_error!(
        "namespace N { export function f(): void; }",
        ";                                         ",
    );

    assert_strip_with_error!(
        "namespace N { export import x = A.x; }",
        ";                                     ",
    );

    assert_strip_with_error!(
        "namespace A { namespace B { export const value = 1; } }",
        ";                                                      ",
    );

    assert_strip_with_error!(
        "export namespace N { const x = 1; }",
        ";                                  ",
        at: "namespace N { const x = 1; }",
    );

    assert_strip_with_error!(
        "export namespace N.M { const x = 1; }",
        ";                                    ",
        at: "namespace N.M { const x = 1; }",
    );

    assert_strip_with_error!(
        "export namespace N { namespace M { const x = 1; } }",
        ";                                                  ",
        at: "namespace N { namespace M { const x = 1; } }",
    );

    assert_strip_with_error!(
        "run();\nexport namespace N { const x = 1; }",
        "run();\n                                   ",
        at: "namespace N { const x = 1; }",
    );

    assert_strip_with_error!(
        "run()\nexport namespace N { const x = 1; }\n(next)();",
        "run()\n;                                  \n(next)();",
        at: "namespace N { const x = 1; }",
    );

    assert_strip_with_error!(
        "namespace N { \"use strict\"; }",
        ";                            ",
        at: "namespace N { \"use strict\"; }",
    );

    assert_strip_with_error!(
        "namespace N { ; }",
        ";                ",
        at: "namespace N { ; }",
    );

    assert_strip_with_error!(
        "namespace N { declare namespace M { const x: number; } }",
        ";                                                       ",
        at: "namespace N { declare namespace M { const x: number; } }",
    );

    assert_strip_with_error!(
        "namespace N { declare enum E { A } }",
        ";                                   ",
        at: "namespace N { declare enum E { A } }",
    );
}

#[test]
fn module_syntax() {
    assert_strip_with_error!(
        "export module M { const x = 1; }",
        ";                               ",
        at: "module M { const x = 1; }",
    );

    assert_strip_with_error!(
        "import lib = require('lib');",
        ";                           ",
    );

    assert_strip_with_error!("export = value as Type;", ";                      ");

    assert_strip_with_error!(
        "export = (() => { type T = number; return 1; })();",
        ";                                                 ",
    );

    assert_strip_with_error!(
        "import type Model = require('model');\nrun();",
        ";                                    \nrun();",
    );
}
