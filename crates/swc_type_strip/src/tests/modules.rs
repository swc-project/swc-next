#[test]
fn whole_type_only_import() {
    assert_strip!(
        "import type { T } from 'm';\nrun();"
        ";                          \nrun();"
    );
}

#[test]
fn default_type_only_import() {
    assert_strip!(
        "import type T from 'm';\nrun();"
        ";                      \nrun();"
    );
}

#[test]
fn namespace_type_only_import() {
    assert_strip!(
        "import type * as T from 'm';\nrun();"
        ";                           \nrun();"
    );
}

#[test]
fn named_type_only_import_item() {
    assert_strip!(
        "import { type A, B } from 'm';"
        "import {         B } from 'm';"
    );
}

#[test]
fn middle_type_only_import_item() {
    assert_strip!(
        "import { A, type B, C } from 'm';"
        "import { A,         C } from 'm';"
    );
}

#[test]
fn last_type_only_import_item() {
    assert_strip!(
        "import { A, type B } from 'm';"
        "import { A,        } from 'm';"
    );
}

#[test]
fn sole_inline_type_only_import_item() {
    assert_strip!(
        "import { type A } from 'm';"
        "import {        } from 'm';"
    );
}

#[test]
fn aliased_inline_type_only_import_item() {
    assert_strip!(
        "import { type A as B, C } from 'm';"
        "import {              C } from 'm';"
    );
}

#[test]
fn consecutive_inline_type_only_import_items() {
    assert_strip!(
        "import { type A, type B, C } from 'm';"
        "import {                 C } from 'm';"
    );
}

#[test]
fn last_inline_type_only_import_item_before_comment() {
    assert_strip!(
        "import { value, type T /* comment */ } from 'm';"
        "import { value,        /* comment */ } from 'm';"
    );
}

#[test]
fn whole_type_only_export() {
    assert_strip!(
        "export type { T } from 'm';\nrun();"
        ";                          \nrun();"
    );
}

#[test]
fn named_type_only_export_item() {
    assert_strip!(
        "export { type A, B };"
        "export {         B };"
    );
}

#[test]
fn middle_type_only_export_item() {
    assert_strip!(
        "export { A, type B, C };"
        "export { A,         C };"
    );
}

#[test]
fn last_type_only_export_item() {
    assert_strip!(
        "export { A, type B };"
        "export { A,        };"
    );
}

#[test]
fn sole_inline_type_only_export_item() {
    assert_strip!(
        "export { type A };"
        "export {        };"
    );
}

#[test]
fn aliased_inline_type_only_export_item() {
    assert_strip!(
        "export { type A as B, C };"
        "export {              C };"
    );
}

#[test]
fn consecutive_inline_type_only_export_items() {
    assert_strip!(
        "export { type A, type B, C };"
        "export {                 C };"
    );
}

#[test]
fn type_only_export_all() {
    assert_strip!(
        "export type * from 'm';\nrun();"
        ";                      \nrun();"
    );
}

#[test]
fn type_only_namespace_export_all() {
    assert_strip!(
        "export type * as ns from 'm';\nrun();"
        ";                            \nrun();"
    );
}

#[test]
fn exported_type_alias_declaration() {
    assert_strip!(
        "export type T = string;\nrun();"
        ";                      \nrun();"
    );
}

#[test]
fn qualified_type_only_import_equals() {
    assert_strip!(
        "import type Alias = Namespace.Member;\nrun();"
        ";                                    \nrun();"
    );
}

#[test]
fn exported_qualified_type_only_import_equals() {
    assert_strip!(
        "export import type Alias = Namespace.Member;"
        ";                                           "
    );
}

#[test]
fn exported_external_type_only_import_equals() {
    assert_strip!(
        "export import type Alias = require('pkg');"
        ";                                         "
    );
}

#[test]
fn type_only_namespace_with_exported_import_equals() {
    assert_strip!(
        "namespace N { export import type Alias = Namespace.Member; }"
        ";                                                           "
    );
}
