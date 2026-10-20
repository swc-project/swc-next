use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, BinaryOperator, DUMMY_SP, ExprData, ObjectPropertyKind, PropertyKey, PropertyKind, Stmt,
    StmtData, SubRange, UnaryOperator,
};
use swc_next_ecma_codegen::{NumberFormat, Options as PrintOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParseOptions, Parser};

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
    let allocator = Allocator::new();
    let parse_options = ParseOptions::default();
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

#[track_caller]
fn expect_empty_statement(source: &str, expected_minified: &str) {
    let allocator = Allocator::new();
    let parse_options = ParseOptions::default();
    let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    assert!(original.diagnostics.is_empty());
    let original_pretty = print(&allocator, &original.ast, PrintOptions::default());

    for minify in [false, true] {
        let print_options = PrintOptions {
            minify,
            ..PrintOptions::default()
        };
        let printed = print(&allocator, &original.ast, print_options.clone());
        if minify {
            assert_eq!(printed.code, expected_minified);
        }

        let reparsed =
            Parser::init(&allocator, printed.code, parse_options, NoTokenParserConfig).parse();
        assert!(reparsed.diagnostics.is_empty());
        let ast = &reparsed.ast;
        let statement = ast.get_node_in_sub_range(ast.root_program().body(ast).first().unwrap());
        assert_empty_statement_body(ast, statement);
        let reparsed_pretty = print(&allocator, ast, PrintOptions::default());
        assert_eq!(reparsed_pretty.code, original_pretty.code);
        let repeated = print(&allocator, ast, print_options);
        assert_eq!(repeated.code, printed.code);
    }
}

fn assert_empty_statement_body(ast: &Ast<'_>, statement: Stmt) {
    let body = match ast.stmt_data(statement) {
        StmtData::BlockStatement(block) => {
            ast.get_node_in_sub_range(block.body(ast).first().unwrap())
        }
        StmtData::Declaration(declaration) => {
            let function = declaration.as_function(ast).unwrap();
            let body = function.body(ast);
            ast.get_node_in_sub_range(body.body(ast).first().unwrap())
        }
        StmtData::SwitchStatement(switch) => {
            let case = ast.get_node_in_sub_range(switch.cases(ast).last().unwrap());
            ast.get_node_in_sub_range(case.consequent(ast).first().unwrap())
        }
        StmtData::WhileStatement(statement) => statement.body(ast),
        StmtData::ForStatement(statement) => statement.body(ast),
        StmtData::ForInStatement(statement) => statement.body(ast),
        StmtData::ForOfStatement(statement) => statement.body(ast),
        StmtData::IfStatement(statement) => statement.consequent(ast),
        StmtData::LabeledStatement(statement) => statement.body(ast),
        StmtData::DoWhileStatement(statement) => statement.body(ast),
        _ => {
            assert!(statement.is_empty_statement(ast));
            return;
        }
    };
    assert_empty_statement_body(ast, body);
}

#[test]
fn test_empty_statement_bodies() {
    expect_empty_statement("function f(){while(a);}", "function f(){while(a);}");

    expect_empty_statement("function f(){if(a);}", "function f(){if(a);}");

    expect_empty_statement("function f(){for(;;);}", "function f(){for(;;);}");

    expect_empty_statement("function f(){label:;}", "function f(){label:;}");

    expect_empty_statement("function f(){for(a in b);}", "function f(){for(a in b);}");

    expect_empty_statement("function f(){for(a of b);}", "function f(){for(a of b);}");

    expect_empty_statement("{while(a);}", "{while(a);}");

    expect_empty_statement("{if(a);}", "{if(a);}");

    expect_empty_statement("{for(;;);}", "{for(;;);}");

    expect_empty_statement("{label:;}", "{label:;}");

    expect_empty_statement(
        "switch(a){default:while(b);}",
        "switch(a){default:while(b);}",
    );

    expect_empty_statement("switch(a){default:if(b);}", "switch(a){default:if(b);}");

    expect_empty_statement("switch(a){default:for(;;);}", "switch(a){default:for(;;);}");

    expect_empty_statement("switch(a){default:label:;}", "switch(a){default:label:;}");
}

#[test]
fn test_empty_statement_delimiters() {
    expect_empty_statement("function f(){while(a);x();}", "function f(){while(a);x()}");

    expect_empty_statement("function f(){if(a);x();}", "function f(){if(a);x()}");

    expect_empty_statement("function f(){for(;;);x();}", "function f(){for(;;);x()}");

    expect_empty_statement("function f(){label:;x();}", "function f(){label:;x()}");

    expect_empty_statement(";", ";");

    expect_empty_statement(";;", ";;");

    expect_empty_statement("{;}", "{;}");

    expect_empty_statement("function f(){;x();}", "function f(){;x()}");

    expect_empty_statement(
        "function f(){do ; while(false);}",
        "function f(){do ;while(false);}",
    );

    expect_empty_statement("function f(){if(a);else;}", "function f(){if(a);else ;}");

    expect_printed_minify("function f(){x();}", "function f(){x()}");

    expect_printed_minify("{x();}", "{x()}");

    expect_printed_minify("switch(a){default:x();}", "switch(a){default:x()}");
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
        print_negative_number_contexts(PrintOptions {
            number_format: NumberFormat::Hexadecimal { lowercase: true },
            ..PrintOptions::default()
        }),
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
fn synthesized_number_subtraction() {
    let minified = PrintOptions {
        minify: true,
        ..PrintOptions::default()
    };

    let code = print_synthesized_number_subtraction(-1.0, minified.clone());
    assert_eq!(code, "x- -1;");

    let code = print_synthesized_number_subtraction(-0.0, minified.clone());
    assert_eq!(code, "x- -0;");

    let code = print_synthesized_number_subtraction(f64::NEG_INFINITY, minified.clone());
    assert_eq!(code, "x- -1/0;");

    let code = print_synthesized_number_subtraction(1.0, minified);
    assert_eq!(code, "x-1;");

    let pretty = PrintOptions::default();

    let code = print_synthesized_number_subtraction(-1.0, pretty.clone());
    assert_eq!(code, "x - -1;\n");

    let code = print_synthesized_number_subtraction(-0.0, pretty.clone());
    assert_eq!(code, "x - -0;\n");

    let code = print_synthesized_number_subtraction(f64::NEG_INFINITY, pretty.clone());
    assert_eq!(code, "x - -1 / 0;\n");

    let code = print_synthesized_number_subtraction(1.0, pretty);
    assert_eq!(code, "x - 1;\n");
}

#[test]
fn synthesized_number_formats() {
    let print = |value, number_format| {
        print_synthesized_number_subtraction(
            value,
            PrintOptions {
                minify: true,
                number_format,
                ..PrintOptions::default()
            },
        )
    };

    let code = print(-42.0, NumberFormat::Preserve);
    assert_eq!(code, "x- -42;");

    let code = print(-42.0, NumberFormat::Decimal);
    assert_eq!(code, "x- -42;");

    let code = print(-42.0, NumberFormat::Binary);
    assert_eq!(code, "x- -0b101010;");

    let code = print(-42.0, NumberFormat::Octal);
    assert_eq!(code, "x- -0o52;");

    let code = print(-42.0, NumberFormat::Hexadecimal { lowercase: true });
    assert_eq!(code, "x- -0x2a;");

    let code = print(-42.0, NumberFormat::Hexadecimal { lowercase: false });
    assert_eq!(code, "x- -0x2A;");

    let code = print(-0.5, NumberFormat::Binary);
    assert_eq!(code, "x- -.5;");

    let code = print(-0.0, NumberFormat::Hexadecimal { lowercase: true });
    assert_eq!(code, "x- -0x0;");

    let code = print(f64::NEG_INFINITY, NumberFormat::Decimal);
    assert_eq!(code, "x- -1/0;");
}

fn print_synthesized_number_subtraction(value: f64, options: PrintOptions) -> String {
    let allocator = Allocator::new();
    let mut ast = Ast::new("", &allocator);
    let name = ast.add_utf8("x");
    let left = ast.expr_identifier_reference(DUMMY_SP, name);
    let right = ast.expr_numeric_literal(DUMMY_SP, value, None);
    let expression = ast.expr_binary_expression(DUMMY_SP, left, right, BinaryOperator::Subtract);
    let statement = ast.stmt_expression_statement(DUMMY_SP, expression);
    let body = ast.add_sub_range([statement]);
    let program = ast.program(DUMMY_SP, SubRange::empty(), body, None);
    ast.set_root_program(program);

    print(&allocator, &ast, options).code.to_owned()
}

#[test]
fn test_string_property_keys() {
    expect_printed_minify(r#"({ "'\"": 1 });"#, r#"({"'\"":1});"#);

    expect_printed_minify(r#"class C { "'\"" = 1; }"#, r#"class C{"'\""=1}"#);

    expect_printed_minify(r#"const { "'\"": x } = o;"#, r#"const{"'\"":x}=o;"#);

    expect_printed_minify(r#"({ ["'\""]: 1 });"#, r#"({[`'"`]:1});"#);
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

#[test]
fn synthesized_number_contexts() {
    fn print_unary(value: f64, options: PrintOptions) -> String {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let argument = ast.expr_numeric_literal(DUMMY_SP, value, None);
        let unary = ast.expr_unary_expression(DUMMY_SP, argument, UnaryOperator::Negate);
        let statement = ast.stmt_expression_statement(DUMMY_SP, unary);
        let body = ast.add_sub_range([statement]);
        let program = ast.program(DUMMY_SP, SubRange::empty(), body, None);
        ast.set_root_program(program);
        print(&allocator, &ast, options).code.to_owned()
    }

    let pretty = PrintOptions::default();
    let minified = PrintOptions {
        minify: true,
        ..PrintOptions::default()
    };

    let code = print_negative_number_contexts(pretty);
    assert_eq!(code, "(-2).toString;\n(-2) ** 2;\n");

    let code = print_negative_number_contexts(minified.clone());
    assert_eq!(code, "(-2).toString;(-2)**2;");

    let code = print_unary(-1.0, minified.clone());
    assert_eq!(code, "-(-1);");

    let code = print_unary(f64::NEG_INFINITY, minified);
    assert_eq!(code, "-(-1/0);");
}

fn print_negative_number_contexts(options: PrintOptions) -> String {
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

    print(&allocator, &ast, options).code.to_owned()
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
fn test_jsx_spread_attributes() {
    expect_printed_jsx("<C {...(a, b)} />;", "<C {...(a, b)} />;\n");

    expect_printed_jsx_minify("<C {...(a, b)} />;", "<C{...(a,b)}/>;");

    expect_printed_jsx("<C {...(a = b)} />;", "<C {...a = b} />;\n");

    expect_printed_jsx_minify("<C {...a} />;", "<C{...a}/>;");
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
fn test_update_expression_parentheses() {
    expect_printed("(a++)`x`;", "(a++)`x`;\n");

    expect_printed_minify("(a++)`x`;", "(a++)`x`;");

    expect_printed("(++a)`x`;", "(++a)`x`;\n");

    expect_printed_minify("(++a)`x`;", "(++a)`x`;");

    expect_printed("(a--)`x`;", "(a--)`x`;\n");

    expect_printed_minify("(a--)`x`;", "(a--)`x`;");

    expect_printed("(--a)`x`;", "(--a)`x`;\n");

    expect_printed_minify("(--a)`x`;", "(--a)`x`;");

    expect_printed_minify("a++;", "a++;");

    expect_printed_minify("++a;", "++a;");
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

#[track_caller]
fn expect_stable_with_parens(source: &str, expected: &str, print_options: &PrintOptions) {
    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        preserve_parens: true,
        ..ParseOptions::default()
    };
    let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    assert!(original.diagnostics.is_empty());
    let printed = print(&allocator, &original.ast, print_options.clone());
    assert_eq!(printed.code, expected);

    let reparsed =
        Parser::init(&allocator, printed.code, parse_options, NoTokenParserConfig).parse();
    assert!(reparsed.diagnostics.is_empty());
    let repeated = print(&allocator, &reparsed.ast, print_options.clone());
    assert_eq!(repeated.code, expected);
}

#[test]
fn test_yield_comments() {
    let pretty = PrintOptions::default();
    let minified = PrintOptions {
        minify: true,
        ..PrintOptions::default()
    };

    expect_stable_with_parens(
        "function* f() { yield* /*a\nb*/ xs; }",
        "function* f() {\n  yield* ( /*a\n   b*/\nxs);\n}\n",
        &pretty,
    );

    expect_stable_with_parens(
        "function* f() { yield* // a\nxs; }",
        "function* f() {\n  yield* ( // a\n  xs);\n}\n",
        &pretty,
    );

    expect_stable_with_parens(
        "function* f() { yield* /*a\nb*/ xs; }",
        "function*f(){yield* (/*a\nb*/xs)}",
        &minified,
    );

    expect_stable_with_parens(
        "function* f() { yield* // a\nxs; }",
        "function*f(){yield* (// a\nxs)}",
        &minified,
    );
}

#[track_caller]
fn expect_export_default_expression(
    source: &str,
    expected: &str,
    print_options: PrintOptions,
    preserve_parens: bool,
) {
    use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParseOptions, Parser};

    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        preserve_parens,
        ..ParseOptions::default()
    };
    let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    assert!(original.diagnostics.is_empty());
    let printed = print(&allocator, &original.ast, print_options.clone());
    assert_eq!(printed.code, expected);

    let reparsed =
        Parser::init(&allocator, printed.code, parse_options, NoTokenParserConfig).parse();
    assert!(reparsed.diagnostics.is_empty());
    let body = reparsed.ast.root_program().body(&reparsed.ast);
    assert_eq!(body.len(), 1);
    let export = body
        .get_node(&reparsed.ast, 0)
        .unwrap()
        .as_export_default_declaration(&reparsed.ast)
        .unwrap();
    assert!(
        export
            .declaration(&reparsed.ast)
            .as_expr(&reparsed.ast)
            .is_some()
    );
    let repeated = print(&allocator, &reparsed.ast, print_options);
    assert_eq!(repeated.code, printed.code);
}

#[test]
fn test_export_default_comments() {
    let pretty = PrintOptions::default();
    let minified = PrintOptions {
        minify: true,
        ..PrintOptions::default()
    };

    expect_export_default_expression(
        "export default (/* a */ function(){} + 1) + 2;",
        "export default /* a */ (function() {\n}) + 1 + 2;\n",
        pretty.clone(),
        false,
    );

    expect_export_default_expression(
        "export default (/* a */ class {} + 1) + 2;",
        "export default /* a */ (class {\n}) + 1 + 2;\n",
        pretty.clone(),
        false,
    );

    expect_export_default_expression(
        "export default (/* a */ function(){} && 1) && 2;",
        "export default /* a */ (function() {\n}) && 1 && 2;\n",
        pretty.clone(),
        false,
    );

    expect_export_default_expression(
        "export default (/* a */ function(){} + 1) + 2;",
        "export default /* a */(function(){})+1+2;",
        minified.clone(),
        false,
    );

    expect_export_default_expression(
        "export default (/* a */ class {} + 1) + 2;",
        "export default /* a */(class{})+1+2;",
        minified.clone(),
        false,
    );

    expect_export_default_expression(
        "export default (/* a */ function(){} && 1) && 2;",
        "export default /* a */(function(){})&&1&&2;",
        minified.clone(),
        false,
    );

    expect_export_default_expression(
        "export default (/* a */ function(){} + 1) + 2;",
        "export default ( /* a */ function() {\n} + 1) + 2;\n",
        pretty,
        true,
    );

    expect_export_default_expression(
        "export default (/* a */ function(){} + 1) + 2;",
        "export default (/* a */function(){}+1)+2;",
        minified,
        true,
    );

    expect_printed_minify(
        "export default (function(){} + 1) + 2;",
        "export default (function(){})+1+2;",
    );

    expect_printed_minify(
        "export default /* a */ (function(){} + 1) + 2;",
        "export default /* a */(function(){})+1+2;",
    );

    expect_printed_minify(
        "export default (/* a */ function(){} + 1) * 2;",
        "export default /* a */(function(){}+1)*2;",
    );

    expect_stable(
        "(/* a */ function(){} + 1) + 2;",
        "/* a */\n(function() {\n}) + 1 + 2;\n",
    );

    expect_printed_minify(
        "(/* a */ function(){} + 1) + 2;",
        "/* a */(function(){})+1+2;",
    );

    expect_stable(
        "/* a */ export default function(){}",
        "/* a */\nexport default function() {\n}\n",
    );

    expect_stable(
        "/* a */ export default class {}",
        "/* a */\nexport default class {\n}\n",
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

#[track_caller]
fn expect_statement_body(source: &str, expected: &str) {
    use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParseOptions, Parser};

    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        preserve_parens: false,
        ..ParseOptions::default()
    };
    let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    assert!(original.diagnostics.is_empty());
    let print_options = PrintOptions {
        minify: true,
        ..PrintOptions::default()
    };
    let compact = print(&allocator, &original.ast, print_options.clone());
    assert_eq!(compact.code, expected);

    let reparsed =
        Parser::init(&allocator, compact.code, parse_options, NoTokenParserConfig).parse();
    assert!(reparsed.diagnostics.is_empty());
    let original_pretty = print(&allocator, &original.ast, PrintOptions::default());
    let reparsed_pretty = print(&allocator, &reparsed.ast, PrintOptions::default());
    assert_eq!(reparsed_pretty.code, original_pretty.code);
    let repeated = print(&allocator, &reparsed.ast, print_options);
    assert_eq!(repeated.code, compact.code);
}

#[test]
fn test_do_while_statement_bodies() {
    expect_statement_body("do debugger; while(false);", "do debugger;while(false);");

    expect_statement_body("do if(a) break; while(b);", "do if(a)break;while(b);");

    expect_statement_body(
        "function f(){ do return 1; while(false); }",
        "function f(){do return 1;while(false);}",
    );

    expect_statement_body("do for(;a;) break; while(b);", "do for(;a;)break;while(b);");

    expect_statement_body("do while(a) break; while(b);", "do while(a)break;while(b);");

    expect_statement_body("do switch(a){} while(b);", "do switch(a){}while(b);");

    expect_statement_body("do try{} finally{} while(b);", "do try{}finally{}while(b);");

    expect_statement_body("do break; while(false);", "do break;while(false);");

    expect_statement_body("do continue; while(false);", "do continue;while(false);");
}

#[test]
fn test_else_statement_bodies() {
    expect_statement_body("if(a) x(); else debugger;", "if(a)x();else debugger;");

    expect_statement_body(
        "if(a) x(); else while(b) y();",
        "if(a)x();else while(b)y();",
    );

    expect_statement_body(
        "if(a) x(); else do y(); while(b);",
        "if(a)x();else do y();while(b);",
    );

    expect_statement_body(
        "function f(){ if(a) x(); else return 1; }",
        "function f(){if(a)x();else return 1}",
    );

    expect_statement_body(
        "function f(){ if(a) x(); else throw 1; }",
        "function f(){if(a)x();else throw 1}",
    );

    expect_statement_body(
        "if(a) x(); else for(;b;) y();",
        "if(a)x();else for(;b;)y();",
    );

    expect_statement_body("if(a) x(); else switch(b){}", "if(a)x();else switch(b){}");

    expect_statement_body(
        "if(a) x(); else try{} finally{}",
        "if(a)x();else try{}finally{}",
    );
}

#[test]
fn test_statement_body_formatting() {
    expect_statement_body("do x(); while(false);", "do x();while(false);");

    expect_statement_body("if(a) x(); else y();", "if(a)x();else y();");

    expect_statement_body("do { x(); } while(false);", "do{x()}while(false);");

    expect_statement_body("if(a) { x(); } else { y(); }", "if(a){x()}else{y()}");

    expect_statement_body(
        "if(a) x(); else if(b) y(); else z();",
        "if(a)x();else if(b)y();else z();",
    );

    expect_stable(
        "do debugger; while(false);",
        "do\n  debugger;\nwhile (false);\n",
    );

    expect_stable(
        "if(a) x(); else while(b) y();",
        "if (a) x();\nelse while (b) y();\n",
    );

    expect_stable(
        "do { x(); } while(false);",
        "do {\n  x();\n} while (false);\n",
    );

    expect_stable(
        "if(a) { x(); } else { y(); }",
        "if (a) {\n  x();\n} else {\n  y();\n}\n",
    );

    expect_stable(
        "if(a) x(); else if(b) y(); else z();",
        "if (a) x();\nelse if (b) y();\nelse z();\n",
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

#[test]
fn test_comment_line_terminators() {
    let pretty = PrintOptions::default();

    expect_stable_with_parens(
        "function f() { return (/* lead\rtrail */ value); }",
        "function f() {\n  return ( /* lead\n   trail */\nvalue);\n}\n",
        &pretty,
    );

    expect_stable_with_parens(
        "throw (/* lead\u{2028}trail */ value);",
        "throw ( /* lead\n trail */\nvalue);\n",
        &pretty,
    );

    expect_stable_with_parens(
        "function* f() { yield (/* lead\u{2029}trail */ value); }",
        "function* f() {\n  yield ( /* lead\n   trail */\nvalue);\n}\n",
        &pretty,
    );
}
