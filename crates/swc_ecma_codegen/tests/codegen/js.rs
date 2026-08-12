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
