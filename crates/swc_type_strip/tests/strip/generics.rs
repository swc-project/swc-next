use super::*;

#[test]
fn type_parameters() {
    assert_strip("function id<T>(value: T) {}", "function id   (value   ) {}");

    assert_strip(
        "const f = function<T>(value: T) {};",
        "const f = function   (value   ) {};",
    );

    assert_strip("class C<T> {}", "class C    {}");

    assert_strip("const C = class<T> {};", "const C = class    {};");

    assert_strip(
        "function f<T extends Base = Default>() {}",
        "function f                          () {}",
    );

    assert_strip("class C { method<T>() {} }", "class C { method   () {} }");

    assert_strip(
        "const object = { method<T>() {} };",
        "const object = { method   () {} };",
    );

    assert_strip(
        "const f = <T>(value) => value;",
        "const f =    (value) => value;",
    );

    assert_strip(
        "const value = foo<<T>(x: T) => number>(() => 1);",
        "const value = foo                     (() => 1);",
    );
}

#[test]
fn type_arguments() {
    assert_strip("id<string>(value);", "id        (value);");

    assert_strip(
        "object.method?.<Type>(value);",
        "object.method?.      (value);",
    );

    assert_strip("const f = make<string>;", "const f = make        ;");

    assert_strip(
        "const value = new Box<Type>();",
        "const value = new Box      ();",
    );

    assert_strip(
        "const value = new Box<Type>;",
        "const value = new Box      ;",
    );

    assert_strip(
        "class C extends B { method() { super.method<Type>(); } }",
        "class C extends B { method() { super.method      (); } }",
    );

    assert_strip("tag<Type>`value`;", "tag      `value`;");
}

#[test]
fn arrow_boundaries() {
    assert_strip(
        "const f = async <\nType\n>(value: Type) => value;",
        "const f = async (\n    \n  value      ) => value;",
    );

    assert_strip(
        "function f() { return<Type>\n(value: Type) => value; }",
        "function f() { return(     \n value      ) => value; }",
    );

    assert_strip(
        "function f() { throw<Type>\n(value: Type) => value; }",
        "function f() { throw(     \n value      ) => value; }",
    );

    assert_strip(
        "function* f() { yield<Type>\n(value: Type) => value; }",
        "function* f() { yield(     \n value      ) => value; }",
    );

    assert_strip(
        "const f = <Type>\n(value: Type) => value;",
        "const f =       \n(value      ) => value;",
    );
}
