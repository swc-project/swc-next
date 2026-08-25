#[test]
fn type_alias_declaration() {
    assert_strip!(
        "type T = string;\nrun();"
        ";               \nrun();"
    );
}

#[test]
fn interface_declaration() {
    assert_strip!(
        "interface I {}\nrun();"
        ";             \nrun();"
    );
}

#[test]
fn exported_interface_declaration() {
    assert_strip!(
        "export interface I {}\nrun();"
        ";                    \nrun();"
    );
}

#[test]
fn namespace_export_declaration() {
    assert_strip!(
        "export as namespace Lib;\nrun();"
        ";                       \nrun();"
    );
}

#[test]
fn ambient_variable_declaration() {
    assert_strip!(
        "declare const value: number;\nrun();"
        ";                           \nrun();"
    );
}

#[test]
fn ambient_class_declaration() {
    assert_strip!(
        "declare class C {}\nrun();"
        ";                 \nrun();"
    );
}

#[test]
fn exported_ambient_class_declaration() {
    assert_strip!(
        "export declare class C {}\nrun();"
        ";                        \nrun();"
    );
}

#[test]
fn ambient_function_declaration() {
    assert_strip!(
        "declare function f(): void;\nrun();"
        ";                          \nrun();"
    );
}

#[test]
fn exported_ambient_function_declaration() {
    assert_strip!(
        "export declare function f(): void;\nrun();"
        ";                                 \nrun();"
    );
}

#[test]
fn exported_function_overload_signature() {
    assert_strip!(
        "export function f(): void;\nexport function f() {}"
        ";                         \nexport function f() {}"
    );
}

#[test]
fn function_overload_signature() {
    assert_strip!(
        "function f(): void;\nfunction f() {}"
        ";                  \nfunction f() {}"
    );
}

#[test]
fn ambient_enum_declaration() {
    assert_strip!(
        "declare enum E {}\nrun();"
        ";                \nrun();"
    );
}

#[test]
fn exported_ambient_enum_declaration() {
    assert_strip!(
        "export declare enum E {}\nrun();"
        ";                       \nrun();"
    );
}

#[test]
fn ambient_namespace_declaration() {
    assert_strip!(
        "declare namespace N { const value: number; }\nrun();"
        ";                                           \nrun();"
    );
}

#[test]
fn exported_ambient_namespace_declaration() {
    assert_strip!(
        "export declare namespace N {}\nrun();"
        ";                            \nrun();"
    );
}

#[test]
fn exported_ambient_variable_declaration() {
    assert_strip!(
        "export declare const value: number;\nrun();"
        ";                                  \nrun();"
    );
}

#[test]
fn type_only_namespace_declaration() {
    assert_strip!(
        "namespace N { export type T = string; }\nrun();"
        ";                                      \nrun();"
    );
}

#[test]
fn qualified_type_only_namespace_declaration() {
    assert_strip!(
        "namespace A.B { type T = string; }\nrun();"
        ";                                 \nrun();"
    );
}

#[test]
fn empty_namespace_declaration() {
    assert_strip!(
        "namespace N {}\nrun();"
        ";             \nrun();"
    );
}

#[test]
fn namespace_with_local_import_alias() {
    assert_strip_with_diagnostic!(
        "namespace N { import x = A.x; }\nrun();"
        ";                              \nrun();"
    );
}

#[test]
fn ambient_external_module_declaration() {
    assert_strip!(
        "declare module 'pkg' { const value: number; }\nrun();"
        ";                                            \nrun();"
    );
}

#[test]
fn global_augmentation_declaration() {
    assert_strip!(
        "declare global { interface Window {} }\nrun();"
        ";                                     \nrun();"
    );
}

#[test]
fn default_exported_interface_declaration() {
    assert_strip!(
        "export default interface I {}\nrun();"
        ";                            \nrun();"
    );
}
