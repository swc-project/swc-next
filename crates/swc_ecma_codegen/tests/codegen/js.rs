use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, BinaryOperator, DUMMY_SP, ExprData, ObjectPropertyKind, PropertyKey, PropertyKind,
    TypedSubRange,
};
use swc_next_ecma_codegen::{NumberFormat, Options as PrintOptions, print};

use crate::{
    expect_printed, expect_printed_minify, expect_printed_script, expect_printed_with_options,
    expect_stable,
};

#[test]
fn test_number_format() {
    expect_printed_with_options(
        "42",
        "0x2a;\n",
        PrintOptions {
            number_format: NumberFormat::Hexadecimal { lowercase: true },
            ..PrintOptions::default()
        },
    );
    expect_printed_with_options(
        "42",
        "0x2A;\n",
        PrintOptions {
            number_format: NumberFormat::Hexadecimal { lowercase: false },
            ..PrintOptions::default()
        },
    );
    expect_printed_with_options(
        "0x2a",
        "42;\n",
        PrintOptions {
            number_format: NumberFormat::Decimal,
            ..PrintOptions::default()
        },
    );
    expect_printed_with_options(
        "42",
        "0b101010;\n",
        PrintOptions {
            number_format: NumberFormat::Binary,
            ..PrintOptions::default()
        },
    );
    expect_printed_with_options(
        "42",
        "0o52;\n",
        PrintOptions {
            number_format: NumberFormat::Octal,
            ..PrintOptions::default()
        },
    );
    assert_eq!(
        print_negative_number_contexts(NumberFormat::Hexadecimal { lowercase: true }),
        "(-0x2).toString;\n(-0x2) ** 0x2;\n"
    );
}

#[test]
fn numeric_literal_value_invalidates_raw() {
    let allocator = Allocator::new();
    let mut ast = Ast::new("0x1", &allocator);
    let raw = ast.source_slice(0, 3);
    let expression = ast.expr_numeric_literal(DUMMY_SP, 1.0, Some(raw));
    let ExprData::NumericLiteral(literal) = ast.expr_data(expression) else {
        unreachable!();
    };

    literal.set_value(&mut ast, 1.0);
    assert_eq!(literal.raw(&ast), Some(raw));

    literal.set_value(&mut ast, 2.0);
    assert_eq!(literal.raw(&ast), None);

    let statement = ast.stmt_expression_statement(DUMMY_SP, expression);
    let body = ast.add_typed_sub_range([statement]);
    let program = ast.program(DUMMY_SP, TypedSubRange::empty(), body, None);
    ast.set_root_program(program);

    assert_eq!(
        print(&allocator, &ast, PrintOptions::default()).code,
        "2;\n"
    );
}

#[test]
fn numeric_literal_value_compares_float_bits() {
    let allocator = Allocator::new();
    let mut ast = Ast::new("0", &allocator);
    let raw = ast.source_slice(0, 1);
    let expression = ast.expr_numeric_literal(DUMMY_SP, 0.0, Some(raw));
    let ExprData::NumericLiteral(literal) = ast.expr_data(expression) else {
        unreachable!();
    };

    literal.set_value(&mut ast, -0.0);
    assert_eq!(literal.raw(&ast), None);

    let statement = ast.stmt_expression_statement(DUMMY_SP, expression);
    let body = ast.add_typed_sub_range([statement]);
    let program = ast.program(DUMMY_SP, TypedSubRange::empty(), body, None);
    ast.set_root_program(program);

    assert_eq!(
        print(&allocator, &ast, PrintOptions::default()).code,
        "-0;\n"
    );
}

#[test]
fn synthesized_number_property_keys() {
    assert_eq!(
        print_synthesized_number_property_key(f64::NAN, false, false),
        "({ \"NaN\": 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(f64::INFINITY, false, false),
        "({ \"Infinity\": 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(f64::NEG_INFINITY, false, false),
        "({ \"-Infinity\": 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(-1.0, false, false),
        "({ \"-1\": 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(-0.0, false, false),
        "({ \"0\": 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(f64::NAN, false, true),
        "({\"NaN\":0});"
    );
    assert_eq!(
        print_synthesized_number_property_key(f64::NAN, true, false),
        "({ [0 / 0]: 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(f64::INFINITY, true, false),
        "({ [1 / 0]: 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(f64::NEG_INFINITY, true, false),
        "({ [-1 / 0]: 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(-1.0, true, false),
        "({ [-1]: 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(-0.0, true, false),
        "({ [-0]: 0 });\n"
    );
    assert_eq!(
        print_synthesized_number_property_key(f64::NAN, true, true),
        "({[0/0]:0});"
    );
}

fn print_synthesized_number_property_key(key_value: f64, computed: bool, minify: bool) -> String {
    let allocator = Allocator::new();
    let mut ast = Ast::new("", &allocator);
    let key = ast.property_key_numeric_literal(DUMMY_SP, key_value, None);
    let value = ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
    let property = ast.object_property(
        DUMMY_SP,
        key,
        value,
        PropertyKind::Init,
        false,
        false,
        computed,
    );
    let property = ObjectPropertyKind::ObjectProperty(property);
    let properties = ast.add_typed_sub_range([property]);
    let expression = ast.expr_object_expression(DUMMY_SP, properties);
    let statement = ast.stmt_expression_statement(DUMMY_SP, expression);
    let body = ast.add_typed_sub_range([statement]);
    let program = ast.program(DUMMY_SP, TypedSubRange::empty(), body, None);
    ast.set_root_program(program);

    print(
        &allocator,
        &ast,
        PrintOptions {
            minify,
            ..PrintOptions::default()
        },
    )
    .code
    .to_owned()
}

fn print_negative_number_contexts(number_format: NumberFormat) -> String {
    let allocator = Allocator::new();
    let mut ast = Ast::new("", &allocator);

    let member_object = ast.expr_numeric_literal(DUMMY_SP, -2.0, None);
    let property_name = ast.add_utf8("toString");
    let property = ast.identifier_name(DUMMY_SP, property_name);
    let member = ast.expr_member_expression(
        DUMMY_SP,
        member_object,
        PropertyKey::IdentifierName(property),
        false,
        false,
    );
    let member_statement = ast.stmt_expression_statement(DUMMY_SP, member);

    let exponent_left = ast.expr_numeric_literal(DUMMY_SP, -2.0, None);
    let exponent_right = ast.expr_numeric_literal(DUMMY_SP, 2.0, None);
    let exponent = ast.expr_binary_expression(
        DUMMY_SP,
        exponent_left,
        exponent_right,
        BinaryOperator::Exponent,
    );
    let exponent_statement = ast.stmt_expression_statement(DUMMY_SP, exponent);

    let body = ast.add_typed_sub_range([member_statement, exponent_statement]);
    let program = ast.program(DUMMY_SP, TypedSubRange::empty(), body, None);
    ast.set_root_program(program);

    print(
        &allocator,
        &ast,
        PrintOptions {
            number_format,
            ..PrintOptions::default()
        },
    )
    .code
    .to_owned()
}

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
