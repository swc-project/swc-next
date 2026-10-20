use super::*;

#[test]
fn imports() {
    assert_strip(
        "import type { T } from 'm';\nrun();",
        ";                          \nrun();",
    );

    assert_strip(
        "import type T from 'm';\nrun();",
        ";                      \nrun();",
    );

    assert_strip(
        "import type * as T from 'm';\nrun();",
        ";                           \nrun();",
    );

    assert_strip(
        "import { type A, B } from 'm';",
        "import {         B } from 'm';",
    );

    assert_strip(
        "import { A, type B, C } from 'm';",
        "import { A,         C } from 'm';",
    );

    assert_strip(
        "import { A, type B } from 'm';",
        "import { A,        } from 'm';",
    );

    assert_strip("import { type A } from 'm';", "import {        } from 'm';");

    assert_strip(
        "import { type A as B, C } from 'm';",
        "import {              C } from 'm';",
    );

    assert_strip(
        "import { type A, type B, C } from 'm';",
        "import {                 C } from 'm';",
    );

    assert_strip(
        "import { value, type T /* comment */ } from 'm';",
        "import { value,        /* comment */ } from 'm';",
    );

    assert_strip(
        "import type Alias = Namespace.Member;\nrun();",
        ";                                    \nrun();",
    );
}

#[test]
fn exports() {
    assert_strip(
        "export type { T } from 'm';\nrun();",
        ";                          \nrun();",
    );

    assert_strip("export { type A, B };", "export {         B };");

    assert_strip("export { A, type B, C };", "export { A,         C };");

    assert_strip("export { A, type B };", "export { A,        };");

    assert_strip("export { type A };", "export {        };");

    assert_strip("export { type A as B, C };", "export {              C };");

    assert_strip(
        "export { type A, type B, C };",
        "export {                 C };",
    );

    assert_strip(
        "export type * from 'm';\nrun();",
        ";                      \nrun();",
    );

    assert_strip(
        "export type * as ns from 'm';\nrun();",
        ";                            \nrun();",
    );

    assert_strip(
        "export type T = string;\nrun();",
        ";                      \nrun();",
    );

    assert_strip(
        "export import type Alias = Namespace.Member;",
        ";                                           ",
    );

    assert_strip(
        "export import type Alias = require('pkg');",
        ";                                         ",
    );

    assert_strip(
        "namespace N { export import type Alias = Namespace.Member; }",
        ";                                                           ",
    );
}
