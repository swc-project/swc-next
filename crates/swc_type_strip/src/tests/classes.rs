#[test]
fn abstract_class_modifier() {
    assert_strip!(
        "abstract class C {}"
        "         class C {}"
    );
}

#[test]
fn decorated_abstract_class_modifier() {
    assert_strip!(
        "@decorators.abstract abstract class C {}"
        "@decorators.abstract          class C {}"
    );
}

#[test]
fn public_field_modifier() {
    assert_strip!(
        "class C { public value = 1; }"
        "class C {        value = 1; }"
    );
}

#[test]
fn readonly_field_modifier() {
    assert_strip!(
        "class C { readonly value = 1; }"
        "class C {          value = 1; }"
    );
}

#[test]
fn override_method_modifier() {
    assert_strip!(
        "class C { override method() {} }"
        "class C {          method() {} }"
    );
}

#[test]
fn private_field_modifier() {
    assert_strip!(
        "class C { private value = 1; }"
        "class C {         value = 1; }"
    );
}

#[test]
fn protected_method_modifier() {
    assert_strip!(
        "class C { protected method() {} }"
        "class C {           method() {} }"
    );
}

#[test]
fn override_field_modifier() {
    assert_strip!(
        "class C { override value = 1; }"
        "class C {          value = 1; }"
    );
}

#[test]
fn class_extends_type_arguments() {
    assert_strip!(
        "class C extends Base<Type> {}"
        "class C extends Base       {}"
    );
}

#[test]
fn class_implements_clause() {
    assert_strip!(
        "class C implements A, B {}"
        "class C                 {}"
    );
}

#[test]
fn generic_class_implements_clause() {
    assert_strip!(
        "class C<T> implements A<T> {}"
        "class C                    {}"
    );
}

#[test]
fn optional_field_marker() {
    assert_strip!(
        "class C { value?: number; }"
        "class C { value         ; }"
    );
}

#[test]
fn optional_get_named_field_keeps_field_boundary() {
    assert_strip!(
        "class C { get?\nnext() {} }"
        "class C { get;\nnext() {} }"
    );
}

#[test]
fn optional_set_named_field_keeps_field_boundary() {
    assert_strip!(
        "class C { set?\nnext() {} }"
        "class C { set;\nnext() {} }"
    );
}

#[test]
fn optional_static_named_field_keeps_field_boundary() {
    assert_strip!(
        "class C { static?\nnext() {} }"
        "class C { static;\nnext() {} }"
    );
}

#[test]
fn definite_field_marker() {
    assert_strip!(
        "class C { value!: number; }"
        "class C { value         ; }"
    );
}

#[test]
fn optional_method_marker() {
    assert_strip!(
        "class C { method?(): void {} }"
        "class C { method ()       {} }"
    );
}

#[test]
fn optional_generic_method() {
    assert_strip!(
        "class C { method?<T>(): void {} }"
        "class C { method    ()       {} }"
    );
}

#[test]
fn parameter_property_modifier_and_annotation() {
    assert_strip_with_diagnostic!(
        @"public"
        "class C { constructor(public value: number) {} }"
        "class C { constructor(       value        ) {} }"
    );
}

#[test]
fn private_parameter_property() {
    assert_strip_with_diagnostic!(
        @"private"
        "class C { constructor(private value: number) {} }"
        "class C { constructor(        value        ) {} }"
    );
}

#[test]
fn protected_parameter_property() {
    assert_strip_with_diagnostic!(
        @"protected"
        "class C { constructor(protected value: number) {} }"
        "class C { constructor(          value        ) {} }"
    );
}

#[test]
fn readonly_parameter_property() {
    assert_strip_with_diagnostic!(
        @"readonly"
        "class C { constructor(readonly value: number) {} }"
        "class C { constructor(         value        ) {} }"
    );
}

#[test]
fn abstract_property_declaration() {
    assert_strip!(
        "class C { abstract value: number; next = 1; }"
        "class C {                         next = 1; }"
    );
}

#[test]
fn declared_property_declaration() {
    assert_strip!(
        "class C { declare value: Type; next = 1; }"
        "class C {                      next = 1; }"
    );
}

#[test]
fn abstract_method_declaration() {
    assert_strip!(
        "class C { abstract method(): Type; next() {} }"
        "class C {                          next() {} }"
    );
}

#[test]
fn method_overload_signature() {
    assert_strip!(
        "class C { method(value: string): string; method(value: string) {} }"
        "class C {                                method(value        ) {} }"
    );
}

#[test]
fn class_index_signature() {
    assert_strip!(
        "class C { [key: string]: number; next = 1; }"
        "class C {                        next = 1; }"
    );
}

#[test]
fn erased_modifier_before_static_computed_method_needs_no_separator() {
    assert_strip!(
        "class C { first = 1\npublic static ['second']() {} }"
        "class C { first = 1\n       static ['second']() {} }"
    );
}

#[test]
fn erased_modifier_separates_generator_method() {
    assert_strip!(
        "class C { first = 1\npublic *next() {} }"
        "class C { first = 1\n;      *next() {} }"
    );
}

#[test]
fn erased_modifier_separates_in_field() {
    assert_strip!(
        "class C { first = 1\npublic in; }"
        "class C { first = 1\n;      in; }"
    );
}

#[test]
fn get_named_field_keeps_field_boundary() {
    assert_strip!(
        "class C { get: Type\nnext() {} }"
        "class C { get;     \nnext() {} }"
    );
}

#[test]
fn set_named_field_keeps_field_boundary() {
    assert_strip!(
        "class C { set: Type\nnext() {} }"
        "class C { set;     \nnext() {} }"
    );
}

#[test]
fn static_named_field_keeps_field_boundary() {
    assert_strip!(
        "class C { static: Type\nnext() {} }"
        "class C { static;     \nnext() {} }"
    );
}

#[test]
fn decorator_separates_computed_method_after_modifier_erasure() {
    assert_strip!(
        "class C { first = 1\n@dec public ['second']() {} }"
        "class C { first = 1\n@dec        ['second']() {} }"
    );
}

#[test]
fn decorator_named_readonly_does_not_hide_field_modifier() {
    assert_strip!(
        "class C { @readonly readonly value = 1; }"
        "class C { @readonly          value = 1; }"
    );
}

#[test]
fn decorator_named_override_does_not_hide_method_modifier() {
    assert_strip!(
        "class C { @override override method() {} }"
        "class C { @override          method() {} }"
    );
}

#[test]
fn decorator_named_public_does_not_hide_accessibility_modifier() {
    assert_strip!(
        "class C { @public public value = 1; }"
        "class C { @public        value = 1; }"
    );
}
