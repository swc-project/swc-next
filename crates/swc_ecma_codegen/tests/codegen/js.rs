use crate::{expect_printed, expect_printed_minify, expect_printed_script};

#[test]
fn test_directive_string_value() {
    expect_printed_script(r#""\u0061""#, "\"\\u0061\";\n");
    expect_printed_script(
        r#"'use\x20strict'; with (x) foo = bar;"#,
        "'use\\x20strict';\nwith (x) foo = bar;\n",
    );
    expect_printed_script(r#"'use\x20asm';"#, "'use\\x20asm';\n");
    expect_printed_minify(r#"'use\x20asm'; foo;"#, "\"use asm\";foo;");
    expect_printed_script(r#""</script";"#, "\"<\\/script\";\n");
    expect_printed(
        r#"function f() { "\0" }"#,
        "function f() {\n  \"\\0\";\n}\n",
    );
    expect_printed_minify(r#"'use\x20strict'; foo;"#, "'use\\x20strict';foo;");
}

#[test]
fn test_multiline_block_comment_is_stable() {
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
fn test_function_source_comments_are_stable() {
    expect_stable(
        "let f = () => /* arrow */ 0; let g = async () => /* async */ 0;",
        "let f = () => /* arrow */ 0;\nlet g = async () => /* async */ 0;\n",
    );
    expect_stable(
        "let f = async () => (/* object */ {}); let g = () => (/* sequence */ a, b); let h = () => (/* array */ [] = []); let i = () => (/* object assignment */ {} = {});",
        "let f = async () => /* object */ ({});\nlet g = () => /* sequence */ (a, b);\nlet h = () => /* array */ ([] = []);\nlet i = () => /* object assignment */ ({} = {});\n",
    );
    expect_stable(
        "let f = () => (/* member */ {}).x; let g = () => (/* binary */ {}) + 1; let h = () => (/* conditional */ {}) ? 1 : 2;",
        "let f = () => /* member */ ({}).x;\nlet g = () => /* binary */ ({}) + 1;\nlet h = () => /* conditional */ ({}) ? 1 : 2;\n",
    );
    expect_stable(
        "foo({ get x() {} /* getter */, next() {} }, 0); foo({ set [x](value) {} /* setter */ }, 0);",
        "foo({ get x() {\n} /* getter */, next() {\n} }, 0);\nfoo({ set [x](value) {\n} /* setter */ }, 0);\n",
    );
    expect_stable(
        "function f() { foo({ get x() {} // getter\n, next() {} }, 0); }",
        "function f() {\n  foo({ get x() {\n  } // getter\n  , next() {\n  } }, 0);\n}\n",
    );
}

fn expect_stable(source: &str, expected: &str) {
    expect_printed(source, expected);
    expect_printed(expected, expected);
}
