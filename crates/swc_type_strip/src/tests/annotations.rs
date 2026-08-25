#[test]
fn variable_type_annotation() {
    assert_strip!(
        "let value: number = 1;"
        "let value         = 1;"
    );
}

#[test]
fn function_parameter_type_annotation() {
    assert_strip!(
        "function f(value: number) {}"
        "function f(value        ) {}"
    );
}

#[test]
fn function_return_type_annotation() {
    assert_strip!(
        "function f(): number {}"
        "function f()         {}"
    );
}

#[test]
fn function_expression_parameter_type_annotation() {
    assert_strip!(
        "const f = function(value: Type) {};"
        "const f = function(value      ) {};"
    );
}

#[test]
fn function_expression_return_type_annotation() {
    assert_strip!(
        "const f = function(): Type {};"
        "const f = function()       {};"
    );
}

#[test]
fn arrow_parameter_type_annotation() {
    assert_strip!(
        "const f = (value: Type) => value;"
        "const f = (value      ) => value;"
    );
}

#[test]
fn arrow_return_type_annotation() {
    assert_strip!(
        "const f = (): Type => value;"
        "const f = ()       => value;"
    );
}

#[test]
fn constructor_parameter_type_annotation() {
    assert_strip!(
        "class C { constructor(value: Type) {} }"
        "class C { constructor(value      ) {} }"
    );
}

#[test]
fn default_parameter_type_annotation() {
    assert_strip!(
        "function f(value: Type = initial) {}"
        "function f(value       = initial) {}"
    );
}

#[test]
fn rest_parameter_type_annotation() {
    assert_strip!(
        "function f(...values: Type[]) {}"
        "function f(...values        ) {}"
    );
}

#[test]
fn object_binding_type_annotation() {
    assert_strip!(
        "const { value }: Shape = input;"
        "const { value }        = input;"
    );
}

#[test]
fn array_binding_type_annotation() {
    assert_strip!(
        "const [value]: Items = input;"
        "const [value]        = input;"
    );
}

#[test]
fn object_parameter_type_annotation() {
    assert_strip!(
        "function f({ value }: Shape) {}"
        "function f({ value }       ) {}"
    );
}

#[test]
fn catch_binding_type_annotation() {
    assert_strip!(
        "try {} catch (error: unknown) {}"
        "try {} catch (error         ) {}"
    );
}

#[test]
fn object_method_parameter_type_annotation() {
    assert_strip!(
        "const object = { method(value: number) {} };"
        "const object = { method(value        ) {} };"
    );
}

#[test]
fn object_method_return_type_annotation() {
    assert_strip!(
        "const object = { method(): number { return 1; } };"
        "const object = { method()         { return 1; } };"
    );
}

#[test]
fn object_getter_return_type_annotation() {
    assert_strip!(
        "const object = { get value(): number { return 1; } };"
        "const object = { get value()         { return 1; } };"
    );
}

#[test]
fn object_setter_parameter_type_annotation() {
    assert_strip!(
        "const object = { set value(input: number) {} };"
        "const object = { set value(input        ) {} };"
    );
}

#[test]
fn class_method_parameter_type_annotation() {
    assert_strip!(
        "class C { method(value: Type) {} }"
        "class C { method(value      ) {} }"
    );
}

#[test]
fn class_method_return_type_annotation() {
    assert_strip!(
        "class C { method(): Type {} }"
        "class C { method()       {} }"
    );
}

#[test]
fn class_getter_return_type_annotation() {
    assert_strip!(
        "class C { get value(): Type { return input; } }"
        "class C { get value()       { return input; } }"
    );
}

#[test]
fn class_setter_parameter_type_annotation() {
    assert_strip!(
        "class C { set value(input: Type) {} }"
        "class C { set value(input      ) {} }"
    );
}

#[test]
fn class_field_type_annotation() {
    assert_strip!(
        "class C { value: number = 1; }"
        "class C { value         = 1; }"
    );
}

#[test]
fn auto_accessor_type_annotation() {
    assert_strip!(
        "class C { accessor value: Type; }"
        "class C { accessor value      ; }"
    );
}

#[test]
fn nested_generic_type_annotation() {
    assert_strip!(
        "let value: Box<Item> = input;"
        "let value            = input;"
    );
}

#[test]
fn multiline_arrow_return_type() {
    assert_strip!(
        "const f = ():\nnumber => 1;"
        "const f = (  \n     ) => 1;"
    );
}

#[test]
fn astral_string_literal_return_type_preserves_utf16_width() {
    assert_strip!(
        "function f(): '💥' {}"
        "function f()       {}"
    );
}
