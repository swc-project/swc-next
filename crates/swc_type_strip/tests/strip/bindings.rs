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

    assert_strip(
        "function f(value? = cond ? 1 : 2) {}",
        "function f(value  = cond ? 1 : 2) {}",
    );

    assert_strip(
        "function f(value?: number = cond ? 1 : 2) {}",
        "function f(value          = cond ? 1 : 2) {}",
    );

    assert_strip(
        "function f(value?: A extends B ? C : D) {}",
        "function f(value                      ) {}",
    );

    assert_strip(
        "function f(value?: A extends B ? C : D = 1) {}",
        "function f(value                       = 1) {}",
    );

    assert_strip(
        "function f(value? = a ? b ? 1 : 2 : 3) {}",
        "function f(value  = a ? b ? 1 : 2 : 3) {}",
    );

    assert_strip(
        "function f(value? = () => cond ? 1 : 2) {}",
        "function f(value  = () => cond ? 1 : 2) {}",
    );

    assert_strip(
        "function f(value? = (inner? = cond ? 1 : 2) => inner) {}",
        "function f(value  = (inner  = cond ? 1 : 2) => inner) {}",
    );

    assert_strip(
        "class C { constructor(value? = cond ? 1 : 2) {} }",
        "class C { constructor(value  = cond ? 1 : 2) {} }",
    );

    assert_strip(
        "function f([value]? = cond ? 1 : 2) {}",
        "function f([value]  = cond ? 1 : 2) {}",
    );

    assert_strip(
        "function f({value}? = cond ? 1 : 2) {}",
        "function f({value}  = cond ? 1 : 2) {}",
    );

    assert_strip(
        "function f([value = cond ? 1 : 2]?) {}",
        "function f([value = cond ? 1 : 2] ) {}",
    );

    assert_strip(
        "function f(value? = obj?.prop) {}",
        "function f(value  = obj?.prop) {}",
    );

    assert_strip(
        "function f(value? = a ?? b) {}",
        "function f(value  = a ?? b) {}",
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
