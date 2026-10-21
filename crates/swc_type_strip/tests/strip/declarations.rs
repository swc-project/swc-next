use super::*;

#[test]
fn type_declarations() {
    assert_strip("type T = string;\nrun();", ";               \nrun();");

    assert_strip("interface I {}\nrun();", ";             \nrun();");

    assert_strip(
        "export interface I {}\nrun();",
        ";                    \nrun();",
    );

    assert_strip(
        "export default interface I {}\nrun();",
        ";                            \nrun();",
    );
}

#[test]
fn namespaces() {
    assert_strip(
        "export as namespace Lib;\nrun();",
        ";                       \nrun();",
    );

    assert_strip(
        "namespace N { export type T = string; }\nrun();",
        ";                                      \nrun();",
    );

    assert_strip(
        "namespace A.B { type T = string; }\nrun();",
        ";                                 \nrun();",
    );

    assert_strip("namespace N {}\nrun();", ";             \nrun();");

    assert_strip("export namespace N {}", ";                    ");

    assert_strip(
        "namespace N { interface T {} export type { T }; }",
        ";                                                ",
    );

    assert_strip(
        "namespace N { interface T {} export { type T }; }",
        ";                                                ",
    );

    assert_strip(
        "namespace N { interface T {} export { type T, type T as U }; }",
        ";                                                             ",
    );

    assert_strip(
        "export namespace N { interface T {} export type { T }; }",
        ";                                                       ",
    );

    assert_strip(
        "namespace N { declare namespace M { interface T {} export type { T }; } }",
        ";                                                                        ",
    );

    assert_strip("namespace N { export {}; }", ";                         ");

    assert_strip(
        "namespace N { interface I { [(() => { return 1; })()]: string; } }",
        ";                                                                 ",
    );

    assert_strip(
        "export namespace N { export type T = string; }",
        ";                                             ",
    );

    assert_strip(
        "export namespace N.M { interface I {} }",
        ";                                      ",
    );

    assert_strip(
        "module M { type T = string; }",
        ";                            ",
    );

    assert_strip("export module M {}", ";                 ");

    assert_strip(
        "export module M { type T = string; }",
        ";                                   ",
    );

    assert_strip("declare module M {}", ";                  ");

    assert_strip(
        "export declare module M { const x: number; }",
        ";                                           ",
    );

    assert_strip(
        "export declare module A.B {}",
        ";                           ",
    );

    assert_strip(
        "export namespace N { declare const enum E { A } }",
        ";                                                ",
    );

    assert_strip("namespace N { module M {} }", ";                          ");

    assert_strip(
        "declare namespace N { namespace M { const x: number; } }",
        ";                                                       ",
    );
}

#[test]
fn ambient_declarations() {
    assert_strip(
        "declare namespace N { const enum E { A } }",
        ";                                         ",
    );

    assert_strip(
        "declare namespace N { namespace M { import X = N.X; } }",
        ";                                                      ",
    );

    assert_strip(
        "declare module 'pkg' { enum E { A } }",
        ";                                    ",
    );

    assert_strip(
        "namespace N { declare namespace M { const enum E { A } } }",
        ";                                                         ",
    );

    assert_strip(
        "declare const value: number;\nrun();",
        ";                           \nrun();",
    );

    assert_strip("declare class C {}\nrun();", ";                 \nrun();");

    assert_strip(
        "export declare class C {}\nrun();",
        ";                        \nrun();",
    );

    assert_strip(
        "declare function f(): void;\nrun();",
        ";                          \nrun();",
    );

    assert_strip(
        "export declare function f(): void;\nrun();",
        ";                                 \nrun();",
    );

    assert_strip("declare enum E {}\nrun();", ";                \nrun();");

    assert_strip(
        "export declare enum E {}\nrun();",
        ";                       \nrun();",
    );

    assert_strip(
        "declare namespace N { const value: number; }\nrun();",
        ";                                           \nrun();",
    );

    assert_strip(
        "export declare namespace N {}\nrun();",
        ";                            \nrun();",
    );

    assert_strip(
        "export declare namespace N { const x: number; }",
        ";                                              ",
    );

    assert_strip(
        "export declare const value: number;\nrun();",
        ";                                  \nrun();",
    );

    assert_strip(
        "declare module 'pkg' { const value: number; }\nrun();",
        ";                                            \nrun();",
    );

    assert_strip(
        "export declare module 'pkg' { const x: number; }",
        ";                                               ",
    );

    assert_strip(
        "declare global { interface Window {} }\nrun();",
        ";                                     \nrun();",
    );
}

#[test]
fn overloads() {
    assert_strip(
        "export function f(): void;\nexport function f() {}",
        ";                         \nexport function f() {}",
    );

    assert_strip(
        "function f(): void;\nfunction f() {}",
        ";                  \nfunction f() {}",
    );
}
