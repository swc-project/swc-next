#[test]
fn unterminated_expression_before_type_alias() {
    assert_strip!(
        "foo\ntype T = string;\n(1);"
        "foo\n;               \n(1);"
    );
}

#[test]
fn terminated_expression_before_type_alias() {
    assert_strip!(
        "foo;\ntype T = string;\n(1);"
        "foo;\n                \n(1);"
    );
}

#[test]
fn type_alias_as_if_statement_body() {
    assert_strip!(
        "if (ok)\n    type T = string;\nrun();"
        "if (ok)\n    ;               \nrun();"
    );
}

#[test]
fn type_alias_as_else_statement_body() {
    assert_strip!(
        "if (ok) run();\nelse type T = string;\nafter();"
        "if (ok) run();\nelse ;               \nafter();"
    );
}

#[test]
fn interface_as_while_statement_body() {
    assert_strip!(
        "while (ok)\n    interface I {}\nafter();"
        "while (ok)\n    ;             \nafter();"
    );
}

#[test]
fn type_alias_as_for_statement_body() {
    assert_strip!(
        "for (;;)\n    type T = string;\nafter();"
        "for (;;)\n    ;               \nafter();"
    );
}

#[test]
fn interface_as_labeled_statement_body() {
    assert_strip!(
        "label: interface I {}\nafter();"
        "label: ;             \nafter();"
    );
}

#[test]
fn interface_as_do_while_statement_body() {
    assert_strip!(
        "do\n    interface I {}\nwhile (ok);"
        "do\n    ;             \nwhile (ok);"
    );
}

#[test]
fn suffix_assertion_before_parenthesized_statement() {
    assert_strip!(
        "value as string\n(next)();"
        "value;         \n(next)();"
    );
}

#[test]
fn suffix_assertion_with_trailing_comment_before_parenthesized_statement() {
    assert_strip!(
        "value as Type/* comment */\n(next)();"
        "value;       /* comment */\n(next)();"
    );
}

#[test]
fn satisfies_assertion_before_computed_statement() {
    assert_strip!(
        "value satisfies Type\n[next]();"
        "value;              \n[next]();"
    );
}

#[test]
fn consecutive_type_declarations_terminate_directive_prologue() {
    assert_strip!(
        "interface A {}\ntype B = string;\n'use strict';"
        ";             \n                \n'use strict';"
    );
}

#[test]
fn first_erased_script_statement_terminates_directive_prologue() {
    assert_strip!(
        "type T = string;\n'use strict';"
        ";               \n'use strict';"
    );
}

#[test]
fn first_erased_module_statement_terminates_directive_prologue() {
    assert_strip!(
        "type T = string;\n'use strict';\nexport {};"
        ";               \n'use strict';\nexport {};"
    );
}

#[test]
fn first_erased_function_statement_terminates_directive_prologue() {
    assert_strip!(
        "function f() {\ntype T = string;\n'use strict';\n}"
        "function f() {\n;               \n'use strict';\n}"
    );
}

#[test]
fn erased_class_modifier_separates_computed_field() {
    assert_strip!(
        "class C { first = 1\npublic ['second'] = 2; }"
        "class C { first = 1\n;      ['second'] = 2; }"
    );
}

#[test]
fn suffix_assertion_separates_computed_class_member() {
    assert_strip!(
        "class C { value = input as Type\n['next']() {} }"
        "class C { value = input;       \n['next']() {} }"
    );
}

#[test]
fn suffix_assertion_before_binary_continuation_stays_connected() {
    assert_strip!(
        "value as Type\n+ next;"
        "value        \n+ next;"
    );
}

#[test]
fn suffix_assertion_before_member_continuation_stays_connected() {
    assert_strip!(
        "const value = input!\n[0];"
        "const value = input \n[0];"
    );
}

#[test]
fn parenthesized_suffix_assertion_before_call_continuation_stays_connected() {
    assert_strip!(
        "const value = (input as Fn)\n(0);"
        "const value = (input      )\n(0);"
    );
}
