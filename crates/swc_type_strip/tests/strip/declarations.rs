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

    assert_strip_error(
        "namespace N { import x = A.x; }\nrun();",
        ";                              \nrun();",
    );
}

#[test]
fn ambient_declarations() {
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
        "export declare const value: number;\nrun();",
        ";                                  \nrun();",
    );

    assert_strip(
        "declare module 'pkg' { const value: number; }\nrun();",
        ";                                            \nrun();",
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
