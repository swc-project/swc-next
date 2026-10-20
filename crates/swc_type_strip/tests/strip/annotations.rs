use super::*;

#[test]
fn binding_annotations() {
    assert_strip("let value: number = 1;", "let value         = 1;");

    assert_strip(
        "const { value }: Shape = input;",
        "const { value }        = input;",
    );

    assert_strip(
        "const [value]: Items = input;",
        "const [value]        = input;",
    );

    assert_strip(
        "function f({ value }: Shape) {}",
        "function f({ value }       ) {}",
    );

    assert_strip(
        "try {} catch (error: unknown) {}",
        "try {} catch (error         ) {}",
    );

    assert_strip(
        "let value: Box<Item> = input;",
        "let value            = input;",
    );
}

#[test]
fn function_annotations() {
    assert_strip(
        "function f(value: number) {}",
        "function f(value        ) {}",
    );

    assert_strip("function f(): number {}", "function f()         {}");

    assert_strip(
        "const f = function(value: Type) {};",
        "const f = function(value      ) {};",
    );

    assert_strip(
        "const f = function(): Type {};",
        "const f = function()       {};",
    );

    assert_strip(
        "function f(value: Type = initial) {}",
        "function f(value       = initial) {}",
    );

    assert_strip(
        "function f(...values: Type[]) {}",
        "function f(...values        ) {}",
    );

    assert_strip("function f(): '💥' {}", "function f()       {}");
}

#[test]
fn arrow_annotations() {
    assert_strip(
        "const f = (value: Type) => value;",
        "const f = (value      ) => value;",
    );

    assert_strip(
        "const f = (): Type => value;",
        "const f = ()       => value;",
    );

    assert_strip("const f = ():\nnumber => 1;", "const f = (  \n     ) => 1;");
}

#[test]
fn member_annotations() {
    assert_strip(
        "class C { constructor(value: Type) {} }",
        "class C { constructor(value      ) {} }",
    );

    assert_strip(
        "const object = { method(value: number) {} };",
        "const object = { method(value        ) {} };",
    );

    assert_strip(
        "const object = { method(): number { return 1; } };",
        "const object = { method()         { return 1; } };",
    );

    assert_strip(
        "const object = { get value(): number { return 1; } };",
        "const object = { get value()         { return 1; } };",
    );

    assert_strip(
        "const object = { set value(input: number) {} };",
        "const object = { set value(input        ) {} };",
    );

    assert_strip(
        "class C { method(value: Type) {} }",
        "class C { method(value      ) {} }",
    );

    assert_strip(
        "class C { method(): Type {} }",
        "class C { method()       {} }",
    );

    assert_strip(
        "class C { get value(): Type { return input; } }",
        "class C { get value()       { return input; } }",
    );

    assert_strip(
        "class C { set value(input: Type) {} }",
        "class C { set value(input      ) {} }",
    );

    assert_strip(
        "class C { value: number = 1; }",
        "class C { value         = 1; }",
    );

    assert_strip(
        "class C { accessor value: Type; }",
        "class C { accessor value      ; }",
    );
}
