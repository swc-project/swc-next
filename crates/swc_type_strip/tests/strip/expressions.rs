use super::*;

#[test]
fn assertions() {
    assert_strip("const x = value as string;", "const x = value          ;");

    assert_strip(
        "const value = [1, 2] as const;",
        "const value = [1, 2]         ;",
    );

    assert_strip(
        "const x = value satisfies Type;",
        "const x = value               ;",
    );

    assert_strip("const x = value!;", "const x = value ;");

    assert_strip("const value = expr!!!;", "const value = expr   ;");

    assert_strip(
        "const value = (expr as A)! satisfies B;",
        "const value = (expr     )             ;",
    );

    assert_strip_error_at(
        "const x = <number>value;",
        "const x =         value;",
        "<number>",
    );

    assert_strip(
        "const value = input as unknown satisfies Type;",
        "const value = input                          ;",
    );

    assert_strip(
        "let value = ['csv', 'json'] as (`csv` | `json`)[];",
        "let value = ['csv', 'json']                      ;",
    );

    assert_strip(
        "const value = 1 * 1 as number + 2;",
        "const value = 1 * 1           + 2;",
    );

    assert_strip(
        "const value = 2 * 3 as number * 2;",
        "const value = 2 * 3           * 2;",
    );

    // [TODO]: Report Invalid syntax for unary assertion left of exponentiation
    assert_strip_text(
        "const value = -input as number ** 2;",
        "const value = -input           ** 2;",
        Lang::Ts,
    );
}

#[test]
fn type_arguments() {
    assert_strip(
        "const value = (fn<A>)! as B;",
        "const value = (fn   )      ;",
    );

    assert_strip(
        "const value = (make!)<Type>;",
        "const value = (make )      ;",
    );

    assert_strip(
        "const value = (fn!)<T>(argument!);",
        "const value = (fn )   (argument );",
    );

    assert_strip("fn!?.<Type>(value);", "fn ?.      (value);");

    assert_strip(
        "const value = new (Map!)<Key, Value>();",
        "const value = new (Map )            ();",
    );

    assert_strip("@decorate<Type>\nclass C {}", "@decorate      \nclass C {}");
}

#[test]
fn assignment_targets() {
    assert_strip("[left!, right!] = values;", "[left , right ] = values;");

    assert_strip("[left as Type] = values;", "[left        ] = values;");

    assert_strip(
        "[left satisfies Type] = values;",
        "[left               ] = values;",
    );
}

#[test]
fn jsx() {
    assert_strip_tsx(
        "const element = <div>{value as string}</div>;",
        "const element = <div>{value          }</div>;",
    );

    assert_strip_tsx(
        "const element = <Component<Type> />;",
        "const element = <Component       />;",
    );

    assert_strip_tsx(
        "const element = <Component render={(value: Type) => value} />;",
        "const element = <Component render={(value      ) => value} />;",
    );
}

#[test]
fn literals() {
    assert_strip(
        r"const value = [/as\s+Type/, input as Type];",
        r"const value = [/as\s+Type/, input        ];",
    );

    assert_strip(
        "const value = `as Type ${input as Type}`;",
        "const value = `as Type ${input        }`;",
    );
}
