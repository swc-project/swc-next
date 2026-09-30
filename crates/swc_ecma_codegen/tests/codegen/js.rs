use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, BinaryOperator, DUMMY_SP, ExprData, ObjectPropertyKind, PropertyKey, PropertyKind,
    SubRange,
};
use swc_next_ecma_codegen::{NumberFormat, Options as PrintOptions, print};

use crate::{
    expect_printed, expect_printed_jsx, expect_printed_jsx_minify, expect_printed_minify,
    expect_printed_script, expect_printed_with_options, expect_stable,
};

#[track_caller]
fn expect_token_boundary(source: &str, expected: &str) {
    use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParseOptions, Parser};

    let print_options = PrintOptions {
        minify: true,
        ..PrintOptions::default()
    };
    for preserve_parens in [false, true] {
        let allocator = Allocator::new();
        let parse_options = ParseOptions {
            preserve_parens,
            ..ParseOptions::default()
        };
        let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
        assert!(original.diagnostics.is_empty());
        let compact = print(&allocator, &original.ast, print_options.clone());
        assert_eq!(compact.code, expected);

        let reparsed =
            Parser::init(&allocator, compact.code, parse_options, NoTokenParserConfig).parse();
        assert!(reparsed.diagnostics.is_empty());
        let original_pretty = print(&allocator, &original.ast, PrintOptions::default());
        let reparsed_pretty = print(&allocator, &reparsed.ast, PrintOptions::default());
        assert_eq!(reparsed_pretty.code, original_pretty.code);
        let repeated = print(&allocator, &reparsed.ast, print_options.clone());
        assert_eq!(repeated.code, compact.code);
    }
}

#[track_caller]
fn expect_token_boundary_mapping(source: &str, expected: &[u8]) {
    use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParseOptions, Parser};

    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParseOptions::default(),
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let print_options = PrintOptions {
        minify: true,
        source_map_path: Some("input.js".into()),
        ..PrintOptions::default()
    };
    let printed = print(&allocator, &parsed.ast, print_options);
    assert_eq!(printed.map.unwrap().mappings(), expected);
}

#[test]
fn test_expression_token_boundaries() {
    expect_token_boundary("await true", "await true;");

    expect_token_boundary("await false", "await false;");

    expect_token_boundary("await null", "await null;");

    expect_token_boundary("await this", "await this;");

    expect_token_boundary("await 1_000n", "await 1000n;");

    expect_token_boundary("await await true", "await await true;");

    expect_token_boundary("await new C", "await new C;");

    expect_token_boundary(
        "await import('data:text/javascript,export default 42')",
        "await import(\"data:text/javascript,export default 42\");",
    );

    expect_token_boundary("new this", "new this;");

    expect_token_boundary("new new C", "new new C();");

    expect_token_boundary("throw true", "throw true;");

    expect_token_boundary("throw null", "throw null;");

    expect_token_boundary("throw 1n", "throw 1n;");

    expect_token_boundary("throw await x", "throw await x;");

    expect_token_boundary("throw new C", "throw new C;");

    expect_token_boundary("throw import('x')", "throw import(\"x\");");

    expect_token_boundary(
        "switch (x) { case true: break; }",
        "switch(x){case true:break}",
    );

    expect_token_boundary(
        "switch (x) { case null: break; }",
        "switch(x){case null:break}",
    );

    expect_token_boundary(
        "class C extends B { async f() { await super.x; } }",
        "class C extends B{async f(){await super.x}}",
    );
}

#[test]
fn test_expression_token_boundary_controls() {
    expect_token_boundary("await x", "await x;");

    expect_token_boundary("await 42", "await 42;");

    expect_token_boundary("await 'x'", "await\"x\";");

    expect_token_boundary("await []", "await[];");

    expect_token_boundary("await (x + 1)", "await(x+1);");

    expect_token_boundary("await /x/", "await/x/;");

    expect_printed("await true", "await true;\n");

    expect_printed("await 1_000n", "await 1000n;\n");

    expect_printed("new this", "new this();\n");
}

#[test]
fn test_expression_token_boundary_mappings() {
    // VLQ segments map each token after its inserted separator.
    expect_token_boundary_mapping("await true", b"AAAA,MAAM");

    expect_token_boundary_mapping("throw await null", b"AAAA,MAAM,MAAM");

    expect_token_boundary_mapping("await await 1n", b"AAAA,MAAM,MAAM");

    expect_token_boundary_mapping("await new this", b"AAAA,MAAM,IAAI");

    expect_token_boundary_mapping("await import('x')", b"AAAA,MAAM,OAAO");
}

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
    let body = ast.add_sub_range([statement]);
    let program = ast.program(DUMMY_SP, SubRange::empty(), body, None);
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
    let body = ast.add_sub_range([statement]);
    let program = ast.program(DUMMY_SP, SubRange::empty(), body, None);
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
    let key = ast.property_key_expr_numeric_literal(DUMMY_SP, key_value, None);
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
    let properties = ast.add_sub_range([property]);
    let expression = ast.expr_object_expression(DUMMY_SP, properties);
    let statement = ast.stmt_expression_statement(DUMMY_SP, expression);
    let body = ast.add_sub_range([statement]);
    let program = ast.program(DUMMY_SP, SubRange::empty(), body, None);
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

    let body = ast.add_sub_range([member_statement, exponent_statement]);
    let program = ast.program(DUMMY_SP, SubRange::empty(), body, None);
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
    expect_printed_script(r#""</script";"#, "\"</script\";\n");
    expect_printed(
        r#"function f() { "\0" }"#,
        "function f() {\n  \"\\0\";\n}\n",
    );
    expect_printed_minify(r#"'use\x20strict';"#, "'use\\x20strict';");
}

#[test]
fn test_computed_key_comments() {
    expect_printed(
        "({ [/* @__PURE__ */ 'x']: make() })",
        "({ [ /* @__PURE__ */ \"x\"]: make() });\n",
    );

    expect_printed_minify(
        "({ [/* @__PURE__ */ 'x']: make() })",
        "({[ /* @__PURE__ */ \"x\"]:make()});",
    );

    expect_printed_minify(
        "({ [/* @__PURE__ */ 1]: make() })",
        "({[ /* @__PURE__ */ 1]:make()});",
    );

    expect_printed_minify(
        "({ [/* @__PURE__ */ 1n]: make() })",
        "({[ /* @__PURE__ */ 1n]:make()});",
    );
}

#[test]
fn test_jsx_container_comments() {
    expect_printed_jsx(
        "<C value={x /* keep */} />;",
        "<C value={x /* keep */ } />;\n",
    );

    expect_printed_jsx("<C>{x /* keep */}</C>;", "<C>{x /* keep */ }</C>;\n");

    expect_printed_jsx("<C {...x /* keep */} />;", "<C {...x /* keep */ } />;\n");

    expect_printed_jsx("<C>{...x /* keep */}</C>;", "<C>{...x /* keep */ }</C>;\n");

    expect_printed_jsx(
        "<C /* tag */ value={x // keep\n} />;",
        "<C value={x // keep\n} />;\n",
    );

    expect_printed_jsx_minify("<C value={x /* keep */} />;", "<C value={x/* keep */}/>;");

    expect_printed_jsx_minify("<C>{/* keep */}</C>;", "<C>{/* keep */}</C>;");
}

#[test]
fn test_container_comments() {
    expect_stable("let x /*! license */;", "let x /*! license */ ;\n");

    expect_stable("const x = [1 /* keep */];", "const x = [1 /* keep */ ];\n");

    expect_stable("[1 /* keep */]; next();", "[1 /* keep */ ];\nnext();\n");

    expect_stable("[/* keep */];", "[ /* keep */ ];\n");

    expect_stable("[1 // keep\n];", "[1 // keep\n];\n");

    expect_stable("({ /* keep */ });", "({ /* keep */ });\n");

    expect_stable("f(/* keep */);", "f( /* keep */ );\n");

    expect_stable("/* only */", "/* only */\n");

    expect_printed_minify("[1 /* keep */];", "[1/* keep */];");
}

#[test]
fn test_parenthesized_comments() {
    expect_stable(
        "function f() { (/* first */ /* second */ value); }",
        "function f() {\n  /* first */\n  /* second */\n  value;\n}\n",
    );

    expect_stable(
        "(/* before */ value /* after */); /* end */",
        "/* before */\nvalue /* after */ ;\n/* end */\n",
    );

    expect_printed_minify(
        "function f() { (/* before */ value /* after */); }",
        "function f(){/* before */value/* after */}",
    );
    expect_printed_minify(
        "function f(){/* before */value/* after */}",
        "function f(){/* before */value/* after */}",
    );
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
        "let f = () => /* array */ [] = [];\n",
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

#[test]
fn test_inline_script() {
    let inline = PrintOptions {
        inline_script: true,
        ..PrintOptions::default()
    };
    let ascii = PrintOptions {
        ascii_only: true,
        ..PrintOptions::default()
    };
    let minified = PrintOptions {
        minify: true,
        ..PrintOptions::default()
    };

    expect_printed("x = `</script>`", "x = `</script>`;\n");
    expect_printed_with_options("x = `</script>`", "x = `</script>`;\n", ascii.clone());
    expect_printed_with_options("x = `</script>`", "x=`</script>`;", minified.clone());
    expect_printed_with_options("x = `</script>`", "x = `<\\/script>`;\n", inline.clone());
    expect_printed_with_options(
        "x = `</script>`",
        "x = `<\\/script>`;\n",
        PrintOptions {
            inline_script: true,
            ..ascii
        },
    );
    expect_printed_with_options(
        "x = `</script>`",
        "x=`<\\/script>`;",
        PrintOptions {
            inline_script: true,
            ..minified
        },
    );
    expect_printed_with_options(
        "String.raw`</script>`",
        "String.raw`</script>`;\n",
        inline.clone(),
    );
    expect_printed_with_options(r#""</SCRIPT>";"#, "\"<\\/SCRIPT>\";\n", inline.clone());
    expect_printed_with_options(r#""use\x20script";"#, "\"use\\x20script\";\n", inline);
    expect_printed("//! </script>", "//! </script>\n");
    expect_printed_minify("x < /script/.test(y)", "x</script/.test(y);");
    expect_printed_minify("x / /script/.test(y)", "x/ /script/.test(y);");
}
