#[test]
fn function_type_parameter_declaration() {
    assert_strip!(
        "function id<T>(value: T) {}"
        "function id   (value   ) {}"
    );
}

#[test]
fn function_expression_type_parameter_declaration() {
    assert_strip!(
        "const f = function<T>(value: T) {};"
        "const f = function   (value   ) {};"
    );
}

#[test]
fn class_type_parameter_declaration() {
    assert_strip!(
        "class C<T> {}"
        "class C    {}"
    );
}

#[test]
fn class_expression_type_parameter_declaration() {
    assert_strip!(
        "const C = class<T> {};"
        "const C = class    {};"
    );
}

#[test]
fn constrained_defaulted_type_parameter_declaration() {
    assert_strip!(
        "function f<T extends Base = Default>() {}"
        "function f                          () {}"
    );
}

#[test]
fn class_method_type_parameter_declaration() {
    assert_strip!(
        "class C { method<T>() {} }"
        "class C { method   () {} }"
    );
}

#[test]
fn object_method_type_parameter_declaration() {
    assert_strip!(
        "const object = { method<T>() {} };"
        "const object = { method   () {} };"
    );
}

#[test]
fn single_line_arrow_type_parameter_declaration() {
    assert_strip!(
        "const f = <T>(value) => value;"
        "const f =    (value) => value;"
    );
}

#[test]
fn call_type_argument_instantiation() {
    assert_strip!(
        "id<string>(value);"
        "id        (value);"
    );
}

#[test]
fn optional_call_type_argument_instantiation() {
    assert_strip!(
        "object.method?.<Type>(value);"
        "object.method?.      (value);"
    );
}

#[test]
fn instantiation_expression_type_arguments() {
    assert_strip!(
        "const f = make<string>;"
        "const f = make        ;"
    );
}

#[test]
fn new_expression_type_arguments() {
    assert_strip!(
        "const value = new Box<Type>();"
        "const value = new Box      ();"
    );
}

#[test]
fn new_expression_type_arguments_without_parentheses() {
    assert_strip!(
        "const value = new Box<Type>;"
        "const value = new Box      ;"
    );
}

#[test]
fn super_method_call_type_arguments() {
    assert_strip!(
        "class C extends B { method() { super.method<Type>(); } }"
        "class C extends B { method() { super.method      (); } }"
    );
}

#[test]
fn tagged_template_type_arguments() {
    assert_strip!(
        "tag<Type>`value`;"
        "tag      `value`;"
    );
}

#[test]
fn multiline_async_arrow_type_parameters() {
    assert_strip!(
        "const f = async <\nType\n>(value: Type) => value;"
        "const f = async (\n    \n  value      ) => value;"
    );
}

#[test]
fn multiline_returned_arrow_type_parameters() {
    assert_strip!(
        "function f() { return<Type>\n(value: Type) => value; }"
        "function f() { return(     \n value      ) => value; }"
    );
}

#[test]
fn multiline_thrown_arrow_type_parameters() {
    assert_strip!(
        "function f() { throw<Type>\n(value: Type) => value; }"
        "function f() { throw(     \n value      ) => value; }"
    );
}

#[test]
fn multiline_yielded_arrow_type_parameters() {
    assert_strip!(
        "function* f() { yield<Type>\n(value: Type) => value; }"
        "function* f() { yield(     \n value      ) => value; }"
    );
}

#[test]
fn multiline_plain_arrow_type_parameters_need_no_parenthesis_move() {
    assert_strip!(
        "const f = <Type>\n(value: Type) => value;"
        "const f =       \n(value      ) => value;"
    );
}

#[test]
fn nested_generic_arrow_type_argument() {
    assert_strip!(
        "const value = foo<<T>(x: T) => number>(() => 1);"
        "const value = foo                     (() => 1);"
    );
}
