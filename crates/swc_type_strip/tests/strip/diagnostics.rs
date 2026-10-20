#[test]
fn runtime_enum() {
    assert_strip_with_diagnostic!(
        "enum E { A }"
        "enum E { A }"
    );
}

#[test]
fn runtime_const_enum() {
    assert_strip_with_diagnostic!(
        "const enum E { A }"
        "const enum E { A }"
    );
}

#[test]
fn runtime_enum_diagnostic_does_not_block_initializer_erasure() {
    assert_strip_with_diagnostic!(
        "enum E { A = value as Type }"
        "enum E { A = value         }"
    );
}

#[test]
fn runtime_namespace_with_type_annotation() {
    assert_strip_with_diagnostic!(
        "namespace N { const value: number = 1; }"
        ";                                       "
    );
}

#[test]
fn namespace_with_private_class() {
    assert_strip_with_diagnostic!(
        "namespace N { class C {} }"
        ";                         "
    );
}

#[test]
fn namespace_with_block_statement() {
    assert_strip_with_diagnostic!(
        "namespace N { {} }"
        ";                 "
    );
}

#[test]
fn namespace_with_ambient_value_declaration() {
    assert_strip_with_diagnostic!(
        "namespace N { declare let value: number; }"
        ";             ;                           "
    );
}

#[test]
fn namespace_with_function_signature() {
    assert_strip_with_diagnostic!(
        "namespace N { export function f(): void; }"
        ";             ;                           "
    );
}

#[test]
fn namespace_with_exported_import_alias() {
    assert_strip_with_diagnostic!(
        "namespace N { export import x = A.x; }"
        ";                                     "
    );
}

#[test]
fn nested_runtime_namespaces_report_outer_namespace() {
    assert_strip_with_diagnostic!(
        "namespace A { namespace B { export const value = 1; } }"
        ";             ;                                        "
    );
}

#[test]
fn legacy_module_declaration() {
    assert_strip_with_diagnostic!(
        "module M { type T = string; }"
        ";          ;                 "
    );
}

#[test]
fn declared_legacy_module_declaration() {
    assert_strip_with_diagnostic!(
        "declare module M {}"
        ";                  "
    );
}

#[test]
fn value_import_equals() {
    assert_strip_with_diagnostic!(
        "import lib = require('lib');"
        "import lib = require('lib');"
    );
}

#[test]
fn export_assignment_with_as_expression() {
    assert_strip_with_diagnostic!(
        "export = value as Type;"
        "export = value        ;"
    );
}

#[test]
fn type_only_import_equals() {
    assert_strip!(
        "import type Model = require('model');\nrun();"
        ";                                    \nrun();"
    );
}
