use crate::{expect_printed, expect_printed_minify, expect_printed_script, expect_stable};

#[test]
fn test_directive_string_value() {
    expect_printed_script(r#""\u0061""#, "\"\\u0061\";\n");
    // The following statement verifies that the escaped directive stays non-strict.
    expect_printed_script(
        r#"'use\x20strict'; with (x) foo = bar;"#,
        "'use\\x20strict';\nwith (x) foo = bar;\n",
    );
    expect_printed_script(r#"'use\x20asm';"#, "'use\\x20asm';\n");
    expect_printed_minify(r#"'use\x20asm';"#, "\"use asm\";");
    expect_printed_script(r#""</script";"#, "\"<\\/script\";\n");
    expect_printed(
        r#"function f() { "\0" }"#,
        "function f() {\n  \"\\0\";\n}\n",
    );
    expect_printed_minify(r#"'use\x20strict';"#, "'use\\x20strict';");
}

#[test]
fn test_multiline_block_comment_is_stable() {
    // The newline inside the comment triggers ASI before the apparent label.
    expect_printed(
        "while (true) { continue /* Multiline\nComment */there; }",
        "while (true) {\n  continue;\n  /* Multiline\n   Comment */\n  there;\n}\n",
    );
}

#[test]
fn test_for_destructuring_default_initializer() {
    expect_printed(
        "for (const [arrow = () => {}] = []; test;);",
        "for (const [arrow = () => {\n}] = []; test; ) ;\n",
    );
    expect_printed(
        "for (const { arrow = () => {} } = {}; test;);",
        "for (const { arrow = () => {\n} } = {}; test; ) ;\n",
    );
}

#[test]
fn test_for_of_async_identifier() {
    expect_printed("for ((async) of []) ;", "for ((async) of []) ;\n");
    expect_printed("for (\\u0061sync of []) ;", "for ((async) of []) ;\n");
    expect_printed("for (async.x of []) ;", "for (async.x of []) ;\n");
    expect_printed("for await (async of []) ;", "for await (async of []) ;\n");
}

#[test]
fn test_array_pattern_rest_after_elision() {
    expect_printed(
        "function foo([,,...rest]) {}",
        "function foo([, , ...rest]) {\n}\n",
    );
    expect_printed("([,,...rest] = value)", "[, , ...rest] = value;\n");
}

#[test]
fn test_for_initializer_nested_in_context() {
    expect_stable(
        r#"for (obj = { get ["x" in empty]() {}, value: "x" in empty }; ;);"#,
        "for (obj = { get [\"x\" in empty]() {\n}, value: \"x\" in empty }; ; ) ;\n",
    );
    expect_printed_minify(
        r#"for (obj = { get ["x" in empty]() {}, value: "x" in empty }; ;);"#,
        "for(obj={get [\"x\"in empty](){},value:\"x\"in empty};;);",
    );
    expect_printed_minify(
        "for(obj={get [\"x\"in empty](){},value:\"x\"in empty};;);",
        "for(obj={get [\"x\"in empty](){},value:\"x\"in empty};;);",
    );
    expect_stable(
        r#"for (C = class { get ["x" in empty]() {} static set ["x" in empty](value) {} }; ;);"#,
        "for (C = class {\n  get [\"x\" in empty]() {\n  }\n  static set [\"x\" in empty](value) {\n  }\n}; ; ) ;\n",
    );
    expect_printed_minify(
        r#"for (C = class { get ["x" in empty]() {} static set ["x" in empty](value) {} }; ;);"#,
        "for(C=class{get [\"x\"in empty](){}static set [\"x\"in empty](value){}};;);",
    );
    expect_printed_minify(
        "for(C=class{get [\"x\"in empty](){}static set [\"x\"in empty](value){}};;);",
        "for(C=class{get [\"x\"in empty](){}static set [\"x\"in empty](value){}};;);",
    );
}

#[test]
fn test_function_source_comments_are_stable() {
    expect_stable(
        "let f = () => /* arrow */ 0;",
        "let f = () => /* arrow */ 0;\n",
    );
    expect_stable(
        "let f = async () => /* async */ 0;",
        "let f = async () => /* async */ 0;\n",
    );
    expect_stable(
        "let f = async () => (/* object */ {});",
        "let f = async () => /* object */ ({});\n",
    );
    expect_stable(
        "let f = () => (/* sequence */ a, b);",
        "let f = () => /* sequence */ (a, b);\n",
    );
    expect_stable(
        "let f = () => (/* array */ [] = []);",
        "let f = () => /* array */ ([] = []);\n",
    );
    expect_stable(
        "let f = () => (/* object assignment */ {} = {});",
        "let f = () => /* object assignment */ ({} = {});\n",
    );
    expect_stable(
        "let f = () => (/* member */ {}).x;",
        "let f = () => /* member */ ({}).x;\n",
    );
    expect_stable(
        "let f = () => (/* binary */ {}) + 1;",
        "let f = () => /* binary */ ({}) + 1;\n",
    );
    expect_stable(
        "let f = () => (/* conditional */ {}) ? 1 : 2;",
        "let f = () => /* conditional */ ({}) ? 1 : 2;\n",
    );
    expect_stable(
        "foo({ get x() {} /* getter */, next() {} }, 0);",
        "foo({ get x() {\n} /* getter */, next() {\n} }, 0);\n",
    );
    expect_stable(
        "foo({ set [x](value) {} /* setter */ }, 0);",
        "foo({ set [x](value) {\n} /* setter */ }, 0);\n",
    );
    expect_stable(
        "function f() { foo({ get x() {} // getter\n, next() {} }, 0); }",
        "function f() {\n  foo({ get x() {\n  } // getter\n  , next() {\n  } }, 0);\n}\n",
    );
}
