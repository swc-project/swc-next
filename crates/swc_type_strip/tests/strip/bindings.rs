#[test]
fn optional_parameter_marker() {
    assert_strip!(
        "function f(value?: number) {}"
        "function f(value         ) {}"
    );
}

#[test]
fn optional_parameter_marker_after_comment() {
    assert_strip!(
        "const f = (value/** doc */?: Type) => value;"
        "const f = (value/** doc */       ) => value;"
    );
}

#[test]
fn optional_destructured_parameter_marker() {
    assert_strip!(
        "function f({ value }?: Shape) {}"
        "function f({ value }        ) {}"
    );
}

#[test]
fn optional_array_parameter_marker() {
    assert_strip!(
        "function f([value]?: Items) {}"
        "function f([value]        ) {}"
    );
}

#[test]
fn variable_definite_assignment_marker() {
    assert_strip!(
        "let value!: number;"
        "let value         ;"
    );
}

#[test]
fn explicit_this_parameter_only() {
    assert_strip!(
        "function f(this: void) {}"
        "function f(          ) {}"
    );
}

#[test]
fn explicit_this_parameter_before_value_parameter() {
    assert_strip!(
        "function f(this: void, value: number) {}"
        "function f(            value        ) {}"
    );
}

#[test]
fn explicit_this_parameter_with_trailing_comma() {
    assert_strip!(
        "function f(this: void,) {}"
        "function f(           ) {}"
    );
}

#[test]
fn explicit_this_parameter_before_rest_parameter() {
    assert_strip!(
        "function f(this: void, ...values: Type[]) {}"
        "function f(            ...values        ) {}"
    );
}
