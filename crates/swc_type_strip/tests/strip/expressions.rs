#[test]
fn as_expression() {
    assert_strip!(
        "const x = value as string;"
        "const x = value          ;"
    );
}

#[test]
fn const_assertion() {
    assert_strip!(
        "const value = [1, 2] as const;"
        "const value = [1, 2]         ;"
    );
}

#[test]
fn satisfies_expression() {
    assert_strip!(
        "const x = value satisfies Type;"
        "const x = value               ;"
    );
}

#[test]
fn non_null_expression() {
    assert_strip!(
        "const x = value!;"
        "const x = value ;"
    );
}

#[test]
fn chained_non_null_expressions() {
    assert_strip!(
        "const value = expr!!!;"
        "const value = expr   ;"
    );
}

#[test]
fn non_null_between_as_and_satisfies_expressions() {
    assert_strip!(
        "const value = (expr as A)! satisfies B;"
        "const value = (expr     )             ;"
    );
}

#[test]
fn non_null_after_instantiation_before_as_expression() {
    assert_strip!(
        "const value = (fn<A>)! as B;"
        "const value = (fn   )      ;"
    );
}

#[test]
fn non_null_expressions_in_assignment_target() {
    assert_strip!(
        "[left!, right!] = values;"
        "[left , right ] = values;"
    );
}

#[test]
fn as_expression_in_assignment_target() {
    assert_strip!(
        "[left as Type] = values;"
        "[left        ] = values;"
    );
}

#[test]
fn satisfies_expression_in_assignment_target() {
    assert_strip!(
        "[left satisfies Type] = values;"
        "[left               ] = values;"
    );
}

#[test]
fn non_null_instantiation_expression() {
    assert_strip!(
        "const value = (make!)<Type>;"
        "const value = (make )      ;"
    );
}

#[test]
fn non_null_generic_call_with_non_null_argument() {
    assert_strip!(
        "const value = (fn!)<T>(argument!);"
        "const value = (fn )   (argument );"
    );
}

#[test]
fn non_null_optional_call_with_type_arguments() {
    assert_strip!(
        "fn!?.<Type>(value);"
        "fn ?.      (value);"
    );
}

#[test]
fn non_null_new_expression_with_type_arguments() {
    assert_strip!(
        "const value = new (Map!)<Key, Value>();"
        "const value = new (Map )            ();"
    );
}

#[test]
fn prefix_type_assertion() {
    assert_strip_with_diagnostic!(
        @"<number>"
        "const x = <number>value;"
        "const x =         value;"
    );
}

#[test]
fn as_expression_in_jsx_child() {
    assert_strip!(
        crate::Lang::Tsx;
        "const element = <div>{value as string}</div>;"
        "const element = <div>{value          }</div>;"
    );
}

#[test]
fn type_arguments_in_jsx_opening_element() {
    assert_strip!(
        crate::Lang::Tsx;
        "const element = <Component<Type> />;"
        "const element = <Component       />;"
    );
}

#[test]
fn parameter_annotation_in_jsx_attribute() {
    assert_strip!(
        crate::Lang::Tsx;
        "const element = <Component render={(value: Type) => value} />;"
        "const element = <Component render={(value      ) => value} />;"
    );
}

#[test]
fn chained_suffix_assertions() {
    assert_strip!(
        "const value = input as unknown satisfies Type;"
        "const value = input                          ;"
    );
}

#[test]
fn parenthesized_template_literal_array_assertion() {
    assert_strip!(
        "let value = ['csv', 'json'] as (`csv` | `json`)[];"
        "let value = ['csv', 'json']                      ;"
    );
}

#[test]
fn safe_assertion_before_looser_binary_operator() {
    assert_strip!(
        "const value = 1 * 1 as number + 2;"
        "const value = 1 * 1           + 2;"
    );
}

#[test]
fn safe_assertion_between_left_associative_operators() {
    assert_strip!(
        "const value = 2 * 3 as number * 2;"
        "const value = 2 * 3           * 2;"
    );
}

#[test]
fn unary_assertion_left_of_exponentiation() {
    // [TODO]: Report Invalid syntax for unary assertion left of exponentiation
    assert_strip_text!(
        "const value = -input as number ** 2;"
        "const value = -input           ** 2;"
    );
}

#[test]
fn regular_expression_text_is_not_erased() {
    assert_strip!(
        r"const value = [/as\s+Type/, input as Type];"
        r"const value = [/as\s+Type/, input        ];"
    );
}

#[test]
fn template_quasi_text_is_not_erased() {
    assert_strip!(
        "const value = `as Type ${input as Type}`;"
        "const value = `as Type ${input        }`;"
    );
}

#[test]
fn type_arguments_in_decorator_expression() {
    assert_strip!(
        "@decorate<Type>\nclass C {}"
        "@decorate      \nclass C {}"
    );
}
