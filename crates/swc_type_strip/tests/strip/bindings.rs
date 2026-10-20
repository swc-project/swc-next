use super::*;

#[test]
fn optional_bindings() {
    assert_strip(
        "function f(value?: number) {}",
        "function f(value         ) {}",
    );

    assert_strip(
        "const f = (value/** doc */?: Type) => value;",
        "const f = (value/** doc */       ) => value;",
    );

    assert_strip(
        "function f({ value }?: Shape) {}",
        "function f({ value }        ) {}",
    );

    assert_strip(
        "function f([value]?: Items) {}",
        "function f([value]        ) {}",
    );
}

#[test]
fn definite_assignments() {
    assert_strip("let value!: number;", "let value         ;");
}

#[test]
fn this_parameters() {
    assert_strip("function f(this: void) {}", "function f(          ) {}");

    assert_strip(
        "function f(this: void, value: number) {}",
        "function f(            value        ) {}",
    );

    assert_strip("function f(this: void,) {}", "function f(           ) {}");

    assert_strip(
        "function f(this: void, ...values: Type[]) {}",
        "function f(            ...values        ) {}",
    );
}
