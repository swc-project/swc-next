use super::*;

#[test]
fn modifiers() {
    assert_strip("abstract class C {}", "         class C {}");

    assert_strip(
        "@decorators.abstract abstract class C {}",
        "@decorators.abstract          class C {}",
    );

    assert_strip(
        "class C { public value = 1; }",
        "class C {        value = 1; }",
    );

    assert_strip(
        "class C { readonly value = 1; }",
        "class C {          value = 1; }",
    );

    assert_strip(
        "class C { override method() {} }",
        "class C {          method() {} }",
    );

    assert_strip(
        "class C { private value = 1; }",
        "class C {         value = 1; }",
    );

    assert_strip(
        "class C { protected method() {} }",
        "class C {           method() {} }",
    );

    assert_strip(
        "class C { override value = 1; }",
        "class C {          value = 1; }",
    );
}

#[test]
fn heritage() {
    assert_strip(
        "class C extends Base<Type> {}",
        "class C extends Base       {}",
    );

    assert_strip("class C implements A, B {}", "class C                 {}");

    assert_strip(
        "class C<T> implements A<T> {}",
        "class C                    {}",
    );
}

#[test]
fn member_markers() {
    assert_strip("class C { value?: number; }", "class C { value         ; }");

    assert_strip("class C { value!: number; }", "class C { value         ; }");

    assert_strip(
        "class C { get!: Type\n['foo']() {} }",
        "class C { get ;     \n['foo']() {} }",
    );

    assert_strip(
        "class C { set!: Type\n['foo']() {} }",
        "class C { set ;     \n['foo']() {} }",
    );

    assert_strip(
        "class C { static!: Type\n['foo']() {} }",
        "class C { static ;     \n['foo']() {} }",
    );

    assert_strip(
        "class C { method?(): void {} }",
        "class C { method ()       {} }",
    );

    assert_strip(
        "class C { method?<T>(): void {} }",
        "class C { method    ()       {} }",
    );
}

#[test]
fn member_boundaries() {
    assert_strip("class C { get?\nnext() {} }", "class C { get;\nnext() {} }");

    assert_strip(
        "class C { get?\n'foo'() {} }",
        "class C { get;\n'foo'() {} }",
    );

    assert_strip("class C { get?\n*foo() {} }", "class C { get;\n*foo() {} }");

    assert_strip("class C { set?\nnext() {} }", "class C { set;\nnext() {} }");

    assert_strip("class C { set?\n1() {} }", "class C { set;\n1() {} }");

    assert_strip("class C { set?\n*foo() {} }", "class C { set;\n*foo() {} }");

    assert_strip(
        "class C { static?\nnext() {} }",
        "class C { static;\nnext() {} }",
    );

    assert_strip(
        "class C { static?\n['foo']() {} }",
        "class C { static;\n['foo']() {} }",
    );

    assert_strip(
        "class C { static?\n*foo() {} }",
        "class C { static;\n*foo() {} }",
    );

    assert_strip(
        "class C { static?\nconstructor() {} }",
        "class C { static;\nconstructor() {} }",
    );

    assert_strip(
        "class C { static?\n#foo() {} }",
        "class C { static;\n#foo() {} }",
    );

    assert_strip(
        "class C { static?\nstatic {} }",
        "class C { static;\nstatic {} }",
    );

    assert_strip(
        "class C { first = 1\npublic static ['second']() {} }",
        "class C { first = 1\n       static ['second']() {} }",
    );

    assert_strip(
        "class C { first = 1\npublic *next() {} }",
        "class C { first = 1\n;      *next() {} }",
    );

    assert_strip(
        "class C { first = 1\npublic in; }",
        "class C { first = 1\n;      in; }",
    );

    assert_strip(
        "class C { get: Type\nnext() {} }",
        "class C { get;     \nnext() {} }",
    );

    assert_strip(
        "class C { set: Type\nnext() {} }",
        "class C { set;     \nnext() {} }",
    );

    assert_strip(
        "class C { static: Type\nnext() {} }",
        "class C { static;     \nnext() {} }",
    );
}

#[test]
fn parameter_properties() {
    assert_strip_error_at(
        "class C { constructor(public value: number) {} }",
        "class C { constructor(       value        ) {} }",
        "public",
    );

    assert_strip_error_at(
        "class C { constructor(private value: number) {} }",
        "class C { constructor(        value        ) {} }",
        "private",
    );

    assert_strip_error_at(
        "class C { constructor(protected value: number) {} }",
        "class C { constructor(          value        ) {} }",
        "protected",
    );

    assert_strip_error_at(
        "class C { constructor(readonly value: number) {} }",
        "class C { constructor(         value        ) {} }",
        "readonly",
    );
}

#[test]
fn member_declarations() {
    assert_strip(
        "class C { abstract value: number; next = 1; }",
        "class C {                         next = 1; }",
    );

    assert_strip(
        "abstract class C { first = 1\nabstract value: number\n['next']() {} }",
        "         class C { first = 1\n;                     \n['next']() {} }",
    );

    assert_strip(
        "class C { declare value: Type; next = 1; }",
        "class C {                      next = 1; }",
    );

    assert_strip(
        "class C { first = 1\ndeclare value: Type\n['next']() {} }",
        "class C { first = 1\n;                  \n['next']() {} }",
    );

    assert_strip(
        "class C { abstract method(): Type; next() {} }",
        "class C {                          next() {} }",
    );

    assert_strip(
        "abstract class C { first = 1/* missing ; */\nabstract method(): void\n['next']() {} }",
        "         class C { first = 1/* missing ; */\n;                      \n['next']() {} }",
    );

    assert_strip(
        "class C { method(value: string): string; method(value: string) {} }",
        "class C {                                method(value        ) {} }",
    );

    assert_strip(
        "class C { [key: string]: number; next = 1; }",
        "class C {                        next = 1; }",
    );
}

#[test]
fn decorators() {
    assert_strip(
        "class C { first = 1\n@dec public ['second']() {} }",
        "class C { first = 1\n@dec        ['second']() {} }",
    );

    assert_strip(
        "class C { @readonly readonly value = 1; }",
        "class C { @readonly          value = 1; }",
    );

    assert_strip(
        "class C { @override override method() {} }",
        "class C { @override          method() {} }",
    );

    assert_strip(
        "class C { @public public value = 1; }",
        "class C { @public        value = 1; }",
    );
}
