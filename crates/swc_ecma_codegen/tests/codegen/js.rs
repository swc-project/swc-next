use swc_next_allocator::{
    Allocator,
    wtf8::{Wtf8, Wtf8Buf},
};
use swc_next_ecma_ast::{
    Ast, AstUtf8, BinaryOperator, CallExpression, DUMMY_SP, ExprData, IdentifierReference, Lang,
    ObjectPropertyKind, PropertyKey, PropertyKind, SetSpan, SourceType, Stmt, StmtData,
    StringLiteral, SubRange, UnaryOperator, VisitMut, VisitMutWith,
};
use swc_next_ecma_codegen::{NumberFormat, Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParseOptions, Parser};

use crate::{
    expect_generated, expect_generated_jsx, expect_generated_jsx_minify, expect_generated_minify,
    expect_generated_script, expect_generated_with_options, expect_generated_with_parse_options,
    expect_stable,
};

fn pure_calls(source: &str, options: &GeneratorOptions) -> String {
    struct PureCalls<'visit, 'ast>(&'visit mut Ast<'ast>);

    impl<'ast> VisitMut<'ast> for PureCalls<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.0
        }

        fn visit_mut_call_expression(&mut self, call: CallExpression) -> Option<CallExpression> {
            let pure = match self.0.expr_data(call.callee(self.0)) {
                ExprData::IdentifierReference(callee) => {
                    self.0.get_utf8(callee.name(self.0)) == "pure"
                }
                ExprData::Function(_) | ExprData::Class(_) | ExprData::ObjectExpression(_) => true,
                _ => false,
            };
            if pure {
                call.set_pure(self.0, true);
            }
            call.visit_mut_children_with(self)
        }
    }

    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParseOptions {
            preserve_parens: false,
            ..ParseOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = unsafe { parsed.ast.into_ast() };
    ast.root_program().visit_mut_with(&mut PureCalls(&mut ast));
    generate(&ast.into(), options.clone()).code.to_owned()
}

#[test]
fn test_pure_calls() {
    let pretty = GeneratorOptions::default();
    let minify = GeneratorOptions {
        minify: true,
        ..GeneratorOptions::default()
    };
    let output = pure_calls("pure().props.f();", &pretty);
    assert_eq!(output, "(/* #__PURE__ */ pure()).props.f();\n");

    let output = pure_calls("pure()();", &minify);
    assert_eq!(output, "(/* #__PURE__ */ pure())();");

    let output = pure_calls("pure()``;", &minify);
    assert_eq!(output, "(/* #__PURE__ */ pure())``;");

    let output = pure_calls("pure()?.[key]();", &minify);
    assert_eq!(output, "(/* #__PURE__ */ pure())?.[key]();");

    let output = pure_calls("new (pure())();", &minify);
    assert_eq!(output, "new(/* #__PURE__ */ pure());");

    let output = pure_calls("const value = pure();", &minify);
    assert_eq!(output, "const value=/* #__PURE__ */ pure();");

    let output = pure_calls("pure(pure());", &pretty);
    assert_eq!(output, "/* #__PURE__ */ pure(/* #__PURE__ */ pure());\n");

    let output = pure_calls("(function() {})();", &minify);
    assert_eq!(output, "/* #__PURE__ */ (function(){})();");

    let output = pure_calls("export default (class {})();", &minify);
    assert_eq!(output, "export default/* #__PURE__ */ (class{})();");

    let output = pure_calls("const f = () => ({})();", &pretty);
    assert_eq!(output, "const f = () => /* #__PURE__ */ ({})();\n");

    let no_comments = GeneratorOptions {
        comments: false,
        ..minify
    };
    let output = pure_calls("pure().props.f();", &no_comments);
    assert_eq!(output, "pure().props.f();");

    let output = pure_calls("(function() {})();", &no_comments);
    assert_eq!(output, "(function(){})();");
}

#[track_caller]
fn expect_token_boundary(source: &str, expected: &str) {
    expect_token_boundary_with_options(source, expected, ParseOptions::default());
}

#[track_caller]
fn expect_token_boundary_with_options(source: &str, expected: &str, parse_options: ParseOptions) {
    let generator_options = GeneratorOptions {
        minify: true,
        ..GeneratorOptions::default()
    };
    let allocator = Allocator::new();
    let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    assert!(original.diagnostics.is_empty());
    let compact = generate(&original.ast, generator_options.clone());
    assert_eq!(compact.code, expected);

    let reparsed = Parser::init(
        &allocator,
        &compact.code,
        parse_options,
        NoTokenParserConfig,
    )
    .parse();
    assert!(reparsed.diagnostics.is_empty());
    let original_pretty = generate(&original.ast, GeneratorOptions::default());
    let reparsed_pretty = generate(&reparsed.ast, GeneratorOptions::default());
    assert_eq!(reparsed_pretty.code, original_pretty.code);
    let repeated = generate(&reparsed.ast, generator_options.clone());
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
    let generator_options = GeneratorOptions {
        minify: true,
        source_map_path: Some("input.js".into()),
        ..GeneratorOptions::default()
    };
    let generated = generate(&parsed.ast, generator_options);
    assert_eq!(generated.map.unwrap().mappings(), expected);
}

#[test]
fn test_generate_result_ownership() {
    let code = {
        let allocator = Allocator::new();
        let source = "/*! license */ const value = 1;";
        let parse_options = ParseOptions {
            comments: swc_next_ecma_parser::CommentMode::Flat,
            ..ParseOptions::default()
        };
        let parsed = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
        assert!(parsed.diagnostics.is_empty());
        let generator_options = GeneratorOptions {
            comments: false,
            source_map_path: Some("路径.js".into()),
            ..GeneratorOptions::default()
        };
        let generated = generate(&parsed.ast, generator_options);
        assert_eq!(generated.legal_comments.len(), 1);
        let map = generated.map.unwrap();
        assert_eq!(map.sources().len(), 1);
        assert_eq!(map.sources()[0], "路径.js".as_bytes());
        assert_eq!(map.sources_content().unwrap(), &[Some(source.as_bytes())]);
        assert_eq!(map.names(), &[b"value".as_slice()]);
        assert!(!map.mappings().is_empty());
        generated.code
    };
    assert_eq!(code, "const value = 1;\n");
}

#[test]
fn test_minify_computed_properties() {
    expect_generated_minify("({get [x](){}});", "({get[x](){}});");

    expect_generated_minify("({set [x](v){}});", "({set[x](v){}});");

    expect_generated_minify("({get \"0\"(){}});", "({get\"0\"(){}});");

    expect_generated_minify("({set \"0\"(v){}});", "({set\"0\"(v){}});");

    expect_generated_minify("class C{static get [x](){}}", "class C{static get[x](){}}");

    expect_generated_minify("class C{get \"x\"(){}}", "class C{get\"x\"(){}}");

    expect_generated_minify("class C{get x(){}}", "class C{get x(){}}");
}

#[test]
fn test_minify_class_modifiers() {
    expect_generated_minify("class C{static [x](){}}", "class C{static[x](){}}");

    expect_generated_minify("class C{static *[x](){}}", "class C{static*[x](){}}");

    expect_generated_minify("class C{static {}}", "class C{static{}}");
}

#[test]
fn test_minify_keyword_expressions() {
    expect_generated_minify("class C extends (a,b){}", "class C extends(a,b){}");

    expect_generated_minify(
        "export default \"x\" in value;",
        "export default\"x\"in value;",
    );
}

#[test]
fn test_minify_empty_bodies() {
    expect_generated_minify("do;while(false);", "do;while(false)");

    expect_generated_minify("if(a)b();else;", "if(a)b();else;");
}

#[test]
fn test_minify_iteration_spacing() {
    expect_generated_minify("for (a in {});", "for(a in{});");

    expect_generated_minify("for (a of []);", "for(a of[]);");

    expect_generated_minify("`${\"x\" in {x:1}}`;", "`${\"x\"in{x:1}}`;");
}

#[test]
fn test_minify_keyword_spacing() {
    expect_generated_minify("function f(){return [1]}", "function f(){return[1]}");

    expect_generated_minify("function* f(){yield [1]}", "function*f(){yield[1]}");

    expect_generated_minify(
        "function*g(){return yield*x}",
        "function*g(){return yield*x}",
    );

    expect_generated_minify("typeof [];", "typeof[];");

    expect_generated_minify("void [];", "void[];");

    expect_generated_minify("delete (x+y).p;", "delete(x+y).p;");

    expect_generated_minify("()=>{return async()=>0}", "()=>{return async()=>0};");

    expect_generated_minify("function f(){return value}", "function f(){return value}");
}

#[test]
#[ignore = "minified numeric member expressions do not choose the shortest literal"]
fn test_minify_numeric_member() {
    expect_generated_minify("4294967295 .x;", "0xffffffff.x;");
}

#[test]
fn test_expression_token_boundaries() {
    let ascii = GeneratorOptions {
        minify: true,
        ascii_only: true,
        ..GeneratorOptions::default()
    };

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

    expect_token_boundary("new new C", "new new C;");

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

    expect_generated_with_options("x = 𠮷 in y;", "x=\\u{20BB7} in y;", ascii.clone());

    expect_generated_with_options(
        "import * as 𠮷 from 'a';",
        "import*as \\u{20BB7} from\"a\";",
        ascii,
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

    expect_generated("await true", "await true;\n");

    expect_generated("await 1_000n", "await 1000n;\n");

    expect_generated("new this", "new this();\n");
}

#[test]
fn test_expression_token_boundary_mappings() {
    expect_token_boundary_mapping("await value", b"AAAA,MAAMA");

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
    let original_pretty = generate(&original.ast, GeneratorOptions::default());

    for minify in [false, true] {
        let generator_options = GeneratorOptions {
            minify,
            ..GeneratorOptions::default()
        };
        let generated = generate(&original.ast, generator_options.clone());
        if minify {
            assert_eq!(generated.code, expected_minified);
        }

        let reparsed = Parser::init(
            &allocator,
            &generated.code,
            parse_options,
            NoTokenParserConfig,
        )
        .parse();
        assert!(reparsed.diagnostics.is_empty());
        let ast = unsafe { reparsed.ast.ast() };
        let statement = ast.get_node_in_sub_range(ast.root_program().body(ast).first().unwrap());
        assert_empty_statement_body(ast, statement);
        let reparsed_pretty = generate(&reparsed.ast, GeneratorOptions::default());
        assert_eq!(reparsed_pretty.code, original_pretty.code);
        let repeated = generate(&reparsed.ast, generator_options);
        assert_eq!(repeated.code, generated.code);
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
        "function f(){do;while(false)}",
    );

    expect_empty_statement("function f(){if(a);else;}", "function f(){if(a);else;}");

    expect_generated_minify("function f(){x();}", "function f(){x()}");

    expect_generated_minify("{x();}", "{x()}");

    expect_generated_minify("switch(a){default:x();}", "switch(a){default:x()}");
}

#[test]
fn test_number_format() {
    expect_generated_with_options(
        "42",
        "0x2a;\n",
        GeneratorOptions {
            number_format: NumberFormat::Hexadecimal { lowercase: true },
            ..GeneratorOptions::default()
        },
    );
    expect_generated_with_options(
        "42",
        "0x2A;\n",
        GeneratorOptions {
            number_format: NumberFormat::Hexadecimal { lowercase: false },
            ..GeneratorOptions::default()
        },
    );
    expect_generated_with_options(
        "0x2a",
        "42;\n",
        GeneratorOptions {
            number_format: NumberFormat::Decimal,
            ..GeneratorOptions::default()
        },
    );
    expect_generated_with_options(
        "42",
        "0b101010;\n",
        GeneratorOptions {
            number_format: NumberFormat::Binary,
            ..GeneratorOptions::default()
        },
    );
    expect_generated_with_options(
        "42",
        "0o52;\n",
        GeneratorOptions {
            number_format: NumberFormat::Octal,
            ..GeneratorOptions::default()
        },
    );
    assert_eq!(
        generate_negative_number_contexts(GeneratorOptions {
            number_format: NumberFormat::Hexadecimal { lowercase: true },
            ..GeneratorOptions::default()
        }),
        "(-0x2).toString;\n(-0x2) ** 0x2;\n"
    );
}

#[test]
fn numeric_literal_value_invalidates_raw() {
    let allocator = Allocator::new();
    let source = "0x1";
    // SAFETY: All handles used below belong to this AST and remain valid.
    let mut ast = unsafe { Ast::new(source, &allocator) };
    // SAFETY: The range covers the entire ASCII source.
    let raw = unsafe { AstUtf8::from_source_range(source, 0, 3) };
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
        generate(&ast.into(), GeneratorOptions::default()).code,
        "2;\n"
    );
}

#[test]
fn numeric_literal_value_compares_float_bits() {
    let allocator = Allocator::new();
    let source = "0";
    // SAFETY: All handles used below belong to this AST and remain valid.
    let mut ast = unsafe { Ast::new(source, &allocator) };
    // SAFETY: The range covers the entire ASCII source.
    let raw = unsafe { AstUtf8::from_source_range(source, 0, 1) };
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
        generate(&ast.into(), GeneratorOptions::default()).code,
        "-0;\n"
    );
}

#[test]
fn synthesized_number_subtraction() {
    let minified = GeneratorOptions {
        minify: true,
        ..GeneratorOptions::default()
    };

    let code = generate_synthesized_number_subtraction(-1.0, minified.clone());
    assert_eq!(code, "x- -1;");

    let code = generate_synthesized_number_subtraction(-0.0, minified.clone());
    assert_eq!(code, "x- -0;");

    let code = generate_synthesized_number_subtraction(f64::NEG_INFINITY, minified.clone());
    assert_eq!(code, "x- -1/0;");

    let code = generate_synthesized_number_subtraction(1.0, minified);
    assert_eq!(code, "x-1;");

    let pretty = GeneratorOptions::default();

    let code = generate_synthesized_number_subtraction(-1.0, pretty.clone());
    assert_eq!(code, "x - -1;\n");

    let code = generate_synthesized_number_subtraction(-0.0, pretty.clone());
    assert_eq!(code, "x - -0;\n");

    let code = generate_synthesized_number_subtraction(f64::NEG_INFINITY, pretty.clone());
    assert_eq!(code, "x - -1 / 0;\n");

    let code = generate_synthesized_number_subtraction(1.0, pretty);
    assert_eq!(code, "x - 1;\n");
}

#[test]
fn synthesized_number_formats() {
    let generate = |value, number_format| {
        generate_synthesized_number_subtraction(
            value,
            GeneratorOptions {
                minify: true,
                number_format,
                ..GeneratorOptions::default()
            },
        )
    };

    let code = generate(-42.0, NumberFormat::Preserve);
    assert_eq!(code, "x- -42;");

    let code = generate(-42.0, NumberFormat::Decimal);
    assert_eq!(code, "x- -42;");

    let code = generate(-42.0, NumberFormat::Binary);
    assert_eq!(code, "x- -0b101010;");

    let code = generate(-42.0, NumberFormat::Octal);
    assert_eq!(code, "x- -0o52;");

    let code = generate(-42.0, NumberFormat::Hexadecimal { lowercase: true });
    assert_eq!(code, "x- -0x2a;");

    let code = generate(-42.0, NumberFormat::Hexadecimal { lowercase: false });
    assert_eq!(code, "x- -0x2A;");

    let code = generate(-0.5, NumberFormat::Binary);
    assert_eq!(code, "x- -.5;");

    let code = generate(-0.0, NumberFormat::Hexadecimal { lowercase: true });
    assert_eq!(code, "x- -0x0;");

    let code = generate(f64::NEG_INFINITY, NumberFormat::Decimal);
    assert_eq!(code, "x- -1/0;");
}

fn generate_synthesized_number_subtraction(value: f64, options: GeneratorOptions) -> String {
    let allocator = Allocator::new();
    // SAFETY: All handles used below belong to this AST and remain valid.
    let mut ast = unsafe { Ast::new("", &allocator) };
    let name = ast.add_utf8("x");
    let left = ast.expr_identifier_reference(DUMMY_SP, name);
    let right = ast.expr_numeric_literal(DUMMY_SP, value, None);
    let expression = ast.expr_binary_expression(DUMMY_SP, left, right, BinaryOperator::Subtract);
    let statement = ast.stmt_expression_statement(DUMMY_SP, expression);
    let body = ast.add_sub_range([statement]);
    let program = ast.program(DUMMY_SP, SubRange::empty(), body, None);
    ast.set_root_program(program);

    generate(&ast.into(), options).code
}

#[test]
fn test_string_property_keys() {
    expect_generated_minify(r#"({ "'\"": 1 });"#, r#"({"'\"":1});"#);

    expect_generated_minify(r#"class C { "'\"" = 1; }"#, r#"class C{"'\""=1}"#);

    expect_generated_minify(r#"const { "'\"": x } = o;"#, r#"const{"'\"":x}=o;"#);

    expect_generated_minify(r#"({ ["'\""]: 1 });"#, r#"({[`'"`]:1});"#);
}

#[test]
fn synthesized_number_property_keys() {
    assert_eq!(
        generate_synthesized_number_property_key(f64::NAN, false, false),
        "({ \"NaN\": 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(f64::INFINITY, false, false),
        "({ \"Infinity\": 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(f64::NEG_INFINITY, false, false),
        "({ \"-Infinity\": 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(-1.0, false, false),
        "({ \"-1\": 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(-0.0, false, false),
        "({ \"0\": 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(f64::NAN, false, true),
        "({\"NaN\":0});"
    );
    assert_eq!(
        generate_synthesized_number_property_key(f64::NAN, true, false),
        "({ [0 / 0]: 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(f64::INFINITY, true, false),
        "({ [1 / 0]: 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(f64::NEG_INFINITY, true, false),
        "({ [-1 / 0]: 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(-1.0, true, false),
        "({ [-1]: 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(-0.0, true, false),
        "({ [-0]: 0 });\n"
    );
    assert_eq!(
        generate_synthesized_number_property_key(f64::NAN, true, true),
        "({[0/0]:0});"
    );
}

fn generate_synthesized_number_property_key(
    key_value: f64,
    computed: bool,
    minify: bool,
) -> String {
    let allocator = Allocator::new();
    // SAFETY: All handles used below belong to this AST and remain valid.
    let mut ast = unsafe { Ast::new("", &allocator) };
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

    generate(
        &ast.into(),
        GeneratorOptions {
            minify,
            ..GeneratorOptions::default()
        },
    )
    .code
}

#[test]
fn synthesized_number_contexts() {
    fn generate_unary(value: f64, options: GeneratorOptions) -> String {
        let allocator = Allocator::new();
        // SAFETY: All handles used below belong to this AST and remain valid.
        let mut ast = unsafe { Ast::new("", &allocator) };
        let argument = ast.expr_numeric_literal(DUMMY_SP, value, None);
        let unary = ast.expr_unary_expression(DUMMY_SP, argument, UnaryOperator::Negate);
        let statement = ast.stmt_expression_statement(DUMMY_SP, unary);
        let body = ast.add_sub_range([statement]);
        let program = ast.program(DUMMY_SP, SubRange::empty(), body, None);
        ast.set_root_program(program);
        generate(&ast.into(), options).code
    }

    let pretty = GeneratorOptions::default();
    let minified = GeneratorOptions {
        minify: true,
        ..GeneratorOptions::default()
    };

    let code = generate_negative_number_contexts(pretty);
    assert_eq!(code, "(-2).toString;\n(-2) ** 2;\n");

    let code = generate_negative_number_contexts(minified.clone());
    assert_eq!(code, "(-2).toString;(-2)**2;");

    let code = generate_unary(-1.0, minified.clone());
    assert_eq!(code, "-(-1);");

    let code = generate_unary(f64::NEG_INFINITY, minified);
    assert_eq!(code, "-(-1/0);");
}

fn generate_negative_number_contexts(options: GeneratorOptions) -> String {
    let allocator = Allocator::new();
    // SAFETY: All handles used below belong to this AST and remain valid.
    let mut ast = unsafe { Ast::new("", &allocator) };

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

    generate(&ast.into(), options).code
}

#[test]
fn identifier_names() {
    let minified = GeneratorOptions {
        minify: true,
        ..GeneratorOptions::default()
    };
    let ascii = GeneratorOptions {
        ascii_only: true,
        ..minified.clone()
    };

    expect_generated_with_options("value.𐐀;", "value.𐐀;", minified.clone());

    expect_generated_with_options("({ 𐐀: 0 });", "({𐐀:0});", minified.clone());

    expect_generated_with_options("({ 𐐀 });", "({𐐀});", minified);

    expect_generated_with_options("value.𐐀;", "value.\\u{10400};", ascii.clone());

    expect_generated_with_options("({ 𐐀: 0 });", "({\\u{10400}:0});", ascii.clone());

    expect_generated_with_options("({ 𐐀 });", "({\\u{10400}});", ascii);
}

#[track_caller]
fn expect_directive_position_string(source: &str, expected: &str) {
    for lang in [Lang::Js, Lang::Ts, Lang::Tsx] {
        let allocator = Allocator::new();
        let parse_options = ParseOptions {
            lang,
            preserve_parens: false,
            ..ParseOptions::default()
        };
        let options = GeneratorOptions {
            minify: true,
            ..GeneratorOptions::default()
        };
        let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
        assert!(original.diagnostics.is_empty());
        let generated = generate(&original.ast, options.clone());
        assert_eq!(generated.code, expected);

        let reparsed = Parser::init(
            &allocator,
            &generated.code,
            parse_options,
            NoTokenParserConfig,
        )
        .parse();
        assert!(reparsed.diagnostics.is_empty());
        // SAFETY: All handles below are obtained from their own AST.
        let (original_ast, output_ast) = unsafe { (original.ast.ast(), reparsed.ast.ast()) };
        assert_eq!(
            original_ast.root_program().directives(original_ast).len(),
            output_ast.root_program().directives(output_ast).len()
        );
        assert_eq!(generate(&reparsed.ast, options).code, generated.code);
    }
}

#[test]
fn test_directive_position_strings() {
    expect_directive_position_string("(\"a\\\"'\");", "`a\"'`;");

    expect_directive_position_string("function f() { (\"a\\\"'\"); }", "function f(){`a\"'`}");

    expect_directive_position_string("(\"use strict\");", "(\"use strict\");");

    expect_directive_position_string(
        "function f() { (\"use strict\"); }",
        "function f(){(\"use strict\")}",
    );

    expect_directive_position_string("(\"a\\\"'\"); \"use strict\";", "`a\"'`;\"use strict\";");
}

#[test]
fn test_directive_string_value() {
    let script = ParseOptions {
        source_type: SourceType::Script,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    let ascii = GeneratorOptions {
        ascii_only: true,
        ..GeneratorOptions::default()
    };
    let minified = GeneratorOptions {
        minify: true,
        ..ascii.clone()
    };

    expect_generated_script(r#""\u0061""#, "\"\\u0061\";\n");

    // The following statement verifies that the escaped directive stays non-strict.
    expect_generated_script(
        r#"'use\x20strict'; with (x) foo = bar;"#,
        "'use\\x20strict';\nwith (x) foo = bar;\n",
    );

    expect_generated_script(r#"'use\x20asm';"#, "'use\\x20asm';\n");

    expect_generated_minify(r#"'use\x20asm';"#, "'use\\x20asm';");

    expect_generated_script(r#""</script";"#, "\"</script\";\n");

    expect_generated(
        r#"function f() { "\0" }"#,
        "function f() {\n  \"\\0\";\n}\n",
    );

    expect_generated_minify(r#"'use\x20strict';"#, "'use\\x20strict';");

    expect_generated_script("'use\\x20client';", "'use\\x20client';\n");

    expect_generated_minify("'use\\x20client';", "'use\\x20client';");

    expect_generated(
        "function f() { 'use\\x20asm'; }",
        "function f() {\n  'use\\x20asm';\n}\n",
    );

    expect_generated_minify(
        "function f() { 'use\\x20asm'; }",
        "function f(){'use\\x20asm'}",
    );

    let source = "\"use\\\u{2028} strict\";";
    expect_generated_with_parse_options(source, "\"use\\x20strict\";\n", script, &ascii);

    expect_generated_with_parse_options(source, "\"use\\x20strict\";", script, &minified);

    let source = "\"use\\\u{2029} strict\";";
    expect_generated_with_parse_options(source, "\"use\\x20strict\";\n", script, &ascii);

    expect_generated_with_parse_options(source, "\"use\\x20strict\";", script, &minified);

    let source = "\"use strict\";";
    expect_generated_with_parse_options(source, "\"use strict\";\n", script, &ascii);

    expect_generated_with_parse_options(source, "\"use strict\";", script, &minified);
}

#[test]
fn test_computed_key_comments() {
    expect_generated(
        "({ [/* @__PURE__ */ 'x']: make() })",
        "({ [ /* @__PURE__ */ \"x\"]: make() });\n",
    );

    expect_generated_minify(
        "({ [/* @__PURE__ */ 'x']: make() })",
        "({[ /* @__PURE__ */ \"x\"]:make()});",
    );

    expect_generated_minify(
        "({ [/* @__PURE__ */ 1]: make() })",
        "({[ /* @__PURE__ */ 1]:make()});",
    );

    expect_generated_minify(
        "({ [/* @__PURE__ */ 1n]: make() })",
        "({[ /* @__PURE__ */ 1n]:make()});",
    );
}

fn first_expression(ast: &Ast<'_>) -> swc_next_ecma_ast::Expr {
    let statement = ast.root_program().body(ast).get_node(ast, 0).unwrap();
    statement
        .as_expression_statement(ast)
        .unwrap()
        .expression(ast)
}

fn jsx_attribute_literal(ast: &Ast<'_>) -> StringLiteral {
    assert_eq!(ast.root_program().body(ast).len(), 1);
    let element = first_expression(ast).as_jsx_element(ast).unwrap();
    let attributes = element.opening_element(ast).attributes(ast);
    assert_eq!(attributes.len(), 1);
    let attribute = attributes
        .get_node(ast, 0)
        .unwrap()
        .as_jsx_attribute(ast)
        .unwrap();
    let value = attribute.value(ast).unwrap();
    if let Some(literal) = value.as_string_literal(ast) {
        literal
    } else {
        value
            .as_jsx_expression_container(ast)
            .unwrap()
            .expression(ast)
            .as_expr(ast)
            .unwrap()
            .as_string_literal(ast)
            .unwrap()
    }
}

fn roundtrip_jsx_attribute(value: &Wtf8, options: &GeneratorOptions) -> Wtf8Buf {
    let parse_options = ParseOptions {
        lang: Lang::Jsx,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    let output = crate::generate_edited("<C title=\"old\" />;", parse_options, options, |ast| {
        let literal = jsx_attribute_literal(ast);
        let replacement = ast.add_wtf8(value);
        literal.set_value(ast, replacement);
        literal.set_raw(ast, swc_next_ecma_ast::AstUtf8::EMPTY);
    });

    let allocator = Allocator::new();
    let parsed = Parser::init(&allocator, &output, parse_options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    // SAFETY: All inspected handles belong to this parsed AST.
    let ast = unsafe { parsed.ast.ast() };
    let literal = jsx_attribute_literal(ast);
    ast.get_wtf8(literal.value(ast)).to_owned()
}

#[test]
fn test_jsx_tokens() {
    expect_generated_jsx_minify("a < <A/>;", "a< <A/>;");

    expect_generated_jsx_minify("a < <>x</>;", "a< <>x</>;");

    expect_generated_jsx_minify("<><A/></>;", "<><A/></>;");

    expect_generated_jsx_minify("a < /x/;", "a< /x/;");
}

#[test]
fn test_jsx_attributes() {
    expect_generated_jsx("<C {...(a, b)} />;", "<C {...(a, b)} />;\n");

    expect_generated_jsx_minify("<C {...(a, b)} />;", "<C{...(a,b)}/>;");

    expect_generated_jsx("<C {...(a = b)} />;", "<C {...a = b} />;\n");

    expect_generated_jsx_minify("<C {...a} />;", "<C{...a}/>;");

    expect_generated_jsx("<a b- c />;", "<a b- c />;\n");

    expect_generated_jsx_minify("<a b- c />;", "<a b- c/>;");

    expect_generated_jsx("<a- b />;", "<a- b />;\n");

    expect_generated_jsx_minify("<a- b />;", "<a- b/>;");

    for minify in [false, true] {
        for ascii_only in [false, true] {
            let options = GeneratorOptions {
                minify,
                ascii_only,
                ..GeneratorOptions::default()
            };
            let input = Wtf8::from_str("\"'");
            let output = roundtrip_jsx_attribute(input, &options);
            assert_eq!(&*output, input);

            let input = Wtf8::from_str("\" ' onClick={pwn()} x='");
            let output = roundtrip_jsx_attribute(input, &options);
            assert_eq!(&*output, input);

            let input = Wtf8::from_str("&quot;");
            let output = roundtrip_jsx_attribute(input, &options);
            assert_eq!(&*output, input);

            let input = Wtf8::from_str("a\nb\tc\rd");
            let output = roundtrip_jsx_attribute(input, &options);
            assert_eq!(&*output, input);

            let input = Wtf8Buf::from_ill_formed_utf16(&[0xD800]);
            let output = roundtrip_jsx_attribute(&input, &options);
            assert_eq!(output, input);

            let input = Wtf8::from_str("new");
            let output = roundtrip_jsx_attribute(input, &options);
            assert_eq!(&*output, input);
        }
    }
}

#[test]
fn test_jsx_container_comments() {
    expect_generated_jsx(
        "<C value={x /* keep */} />;",
        "<C value={x /* keep */ } />;\n",
    );

    expect_generated_jsx("<C>{x /* keep */}</C>;", "<C>{x /* keep */ }</C>;\n");

    expect_generated_jsx("<C {...x /* keep */} />;", "<C {...x /* keep */ } />;\n");

    expect_generated_jsx("<C>{...x /* keep */}</C>;", "<C>{...x /* keep */ }</C>;\n");

    expect_generated_jsx(
        "<C /* tag */ value={x // keep\n} />;",
        "<C value={x // keep\n} />;\n",
    );

    expect_generated_jsx_minify("<C value={x /* keep */} />;", "<C value={x/* keep */}/>;");

    expect_generated_jsx_minify("<C>{/* keep */}</C>;", "<C>{/* keep */}</C>;");
}

#[test]
fn test_container_comments() {
    let without_comments = GeneratorOptions {
        comments: false,
        ..GeneratorOptions::default()
    };
    expect_generated_with_options("f(a, b /* keep */, c);", "f(a, b, c);\n", without_comments);

    expect_stable("let x /*! license */;", "let x /*! license */ ;\n");

    expect_stable("const x = [1 /* keep */];", "const x = [1 /* keep */ ];\n");

    expect_stable("[1 /* keep */]; next();", "[1 /* keep */ ];\nnext();\n");

    expect_stable("[/* keep */];", "[ /* keep */ ];\n");

    expect_stable("[1 // keep\n];", "[1 // keep\n];\n");

    expect_stable("({ /* keep */ });", "({ /* keep */ });\n");

    expect_stable("f(/* keep */);", "f( /* keep */ );\n");

    expect_stable("/* only */", "/* only */\n");

    expect_generated_minify("[1 /* keep */];", "[1/* keep */];");
}

#[test]
fn test_update_expression_parentheses() {
    expect_generated("(a++)`x`;", "(a++)`x`;\n");

    expect_generated_minify("(a++)`x`;", "(a++)`x`;");

    expect_generated("(++a)`x`;", "(++a)`x`;\n");

    expect_generated_minify("(++a)`x`;", "(++a)`x`;");

    expect_generated("(a--)`x`;", "(a--)`x`;\n");

    expect_generated_minify("(a--)`x`;", "(a--)`x`;");

    expect_generated("(--a)`x`;", "(--a)`x`;\n");

    expect_generated_minify("(--a)`x`;", "(--a)`x`;");

    expect_generated_minify("a++;", "a++;");

    expect_generated_minify("++a;", "++a;");
}

#[test]
#[ignore = "minified BigInt literals do not yet choose the shortest representation"]
fn test_minify_bigint() {
    expect_generated_minify("0x33333333n;", "858993459n;");

    expect_generated_minify("0b1010n;", "10n;");

    expect_generated_minify("0x0001n;", "1n;");

    expect_generated_minify("123456789012345678901n;", "0x6b14e9f812f366c35n;");
}

#[test]
fn test_minify_do_while() {
    expect_generated_minify("do {} while (false);", "do{}while(false)");

    expect_generated_minify(
        "if (a) do {} while (b); else x();",
        "if(a)do{}while(b)else x();",
    );

    expect_generated_minify(
        "do {} while (false); /x/.test('x');",
        "do{}while(false)/x/.test(\"x\");",
    );

    expect_generated_minify(
        "do do; while (false); while (false);",
        "do do;while(false)while(false)",
    );
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

    expect_generated_minify(
        "function f() { (/* before */ value /* after */); }",
        "function f(){/* before */value/* after */}",
    );
    expect_generated_minify(
        "function f(){/* before */value/* after */}",
        "function f(){/* before */value/* after */}",
    );
}

#[track_caller]
fn expect_stable_with_parens(source: &str, expected: &str, generator_options: &GeneratorOptions) {
    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        preserve_parens: true,
        ..ParseOptions::default()
    };
    let original = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    assert!(original.diagnostics.is_empty());
    let generated = generate(&original.ast, generator_options.clone());
    assert_eq!(generated.code, expected);

    let reparsed = Parser::init(
        &allocator,
        &generated.code,
        parse_options,
        NoTokenParserConfig,
    )
    .parse();
    assert!(reparsed.diagnostics.is_empty());
    let repeated = generate(&reparsed.ast, generator_options.clone());
    assert_eq!(repeated.code, expected);
}

#[test]
fn test_yield_comments() {
    let pretty = GeneratorOptions::default();
    let minified = GeneratorOptions {
        minify: true,
        ..GeneratorOptions::default()
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
        "function*f(){yield*(/*a\nb*/xs)}",
        &minified,
    );

    expect_stable_with_parens(
        "function* f() { yield* // a\nxs; }",
        "function*f(){yield*(// a\nxs)}",
        &minified,
    );
}

#[track_caller]
fn expect_export_default_expression(
    source: &str,
    expected: &str,
    generator_options: GeneratorOptions,
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
    let generated = generate(&original.ast, generator_options.clone());
    assert_eq!(generated.code, expected);

    let reparsed = Parser::init(
        &allocator,
        &generated.code,
        parse_options,
        NoTokenParserConfig,
    )
    .parse();
    assert!(reparsed.diagnostics.is_empty());
    // SAFETY: The export and all inspected children belong to the reparsed AST.
    let ast = unsafe { reparsed.ast.ast() };
    let body = ast.root_program().body(ast);
    assert_eq!(body.len(), 1);
    let export = body
        .get_node(ast, 0)
        .unwrap()
        .as_export_default_declaration(ast)
        .unwrap();
    assert!(export.declaration(ast).as_expr(ast).is_some());
    let repeated = generate(&reparsed.ast, generator_options);
    assert_eq!(repeated.code, generated.code);
}

#[test]
fn test_export_default_comments() {
    let pretty = GeneratorOptions::default();
    let minified = GeneratorOptions {
        minify: true,
        ..GeneratorOptions::default()
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
        "export default/* a */(function(){})+1+2;",
        minified.clone(),
        false,
    );

    expect_export_default_expression(
        "export default (/* a */ class {} + 1) + 2;",
        "export default/* a */(class{})+1+2;",
        minified.clone(),
        false,
    );

    expect_export_default_expression(
        "export default (/* a */ function(){} && 1) && 2;",
        "export default/* a */(function(){})&&1&&2;",
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
        "export default(/* a */function(){}+1)+2;",
        minified,
        true,
    );

    expect_generated_minify(
        "export default (function(){} + 1) + 2;",
        "export default(function(){})+1+2;",
    );

    expect_generated_minify(
        "export default /* a */ (function(){} + 1) + 2;",
        "export default/* a */(function(){})+1+2;",
    );

    expect_generated_minify(
        "export default (/* a */ function(){} + 1) * 2;",
        "export default/* a */(function(){}+1)*2;",
    );

    expect_stable(
        "(/* a */ function(){} + 1) + 2;",
        "/* a */\n(function() {\n}) + 1 + 2;\n",
    );

    expect_generated_minify(
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
    expect_generated(
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
    let generator_options = GeneratorOptions {
        minify: true,
        ..GeneratorOptions::default()
    };
    let compact = generate(&original.ast, generator_options.clone());
    assert_eq!(compact.code, expected);

    let reparsed = Parser::init(
        &allocator,
        &compact.code,
        parse_options,
        NoTokenParserConfig,
    )
    .parse();
    assert!(reparsed.diagnostics.is_empty());
    let original_pretty = generate(&original.ast, GeneratorOptions::default());
    let reparsed_pretty = generate(&reparsed.ast, GeneratorOptions::default());
    assert_eq!(reparsed_pretty.code, original_pretty.code);
    let repeated = generate(&reparsed.ast, generator_options);
    assert_eq!(repeated.code, compact.code);
}

#[test]
fn test_do_while_statement_bodies() {
    expect_statement_body("do debugger; while(false);", "do debugger;while(false)");

    expect_statement_body("do if(a) break; while(b);", "do if(a)break;while(b)");

    expect_statement_body(
        "function f(){ do return 1; while(false); }",
        "function f(){do return 1;while(false)}",
    );

    expect_statement_body("do for(;a;) break; while(b);", "do for(;a;)break;while(b)");

    expect_statement_body("do while(a) break; while(b);", "do while(a)break;while(b)");

    expect_statement_body("do switch(a){} while(b);", "do switch(a){}while(b)");

    expect_statement_body("do try{} finally{} while(b);", "do try{}finally{}while(b)");

    expect_statement_body("do break; while(false);", "do break;while(false)");

    expect_statement_body("do continue; while(false);", "do continue;while(false)");
}

#[track_caller]
fn unwrapped_blocks(source: &str, options: &GeneratorOptions) -> String {
    struct UnwrapBlocks<'visit, 'ast>(&'visit mut Ast<'ast>);

    impl<'ast> VisitMut<'ast> for UnwrapBlocks<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.0
        }

        fn visit_mut_stmt(&mut self, node: Stmt) -> Option<Stmt> {
            node.visit_mut_children_with(self);
            if let Some(block) = node.as_block_statement(self.0) {
                let body = block.body(self.0);
                assert_eq!(body.len(), 1);
                return body.get_node(self.0, 0);
            }
            Some(node)
        }
    }

    let parse_options = ParseOptions {
        preserve_parens: false,
        ..ParseOptions::default()
    };
    crate::generate_edited(source, parse_options, options, |ast| {
        ast.root_program().visit_mut_with(&mut UnwrapBlocks(ast));
    })
}

#[test]
fn test_else_statement_bodies() {
    let pretty = GeneratorOptions::default();
    let minified = GeneratorOptions {
        minify: true,
        ..pretty.clone()
    };

    expect_statement_body("if(a) x(); else debugger;", "if(a)x();else debugger;");

    expect_statement_body(
        "if(a) x(); else while(b) y();",
        "if(a)x();else while(b)y();",
    );

    expect_statement_body(
        "if(a) x(); else do y(); while(b);",
        "if(a)x();else do y();while(b)",
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

    let source = "if (a) { if (b) x(); } else y();";
    let output = unwrapped_blocks(source, &pretty);
    assert_eq!(output, "if (a) {\n  if (b) x();\n} else y();\n");

    let source = "if (a) { while (b) { if (c) x(); } } else y();";
    let output = unwrapped_blocks(source, &minified);
    assert_eq!(output, "if(a){while(b)if(c)x()}else y();");
}

#[test]
fn test_statement_body_formatting() {
    expect_statement_body("do x(); while(false);", "do x();while(false)");

    expect_statement_body("if(a) x(); else y();", "if(a)x();else y();");

    expect_statement_body("do { x(); } while(false);", "do{x()}while(false)");

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
fn test_object_property_comments() {
    expect_generated(
        "const value = { /* before */ first: 1, // between\nsecond: 2 };",
        "const value = { /* before */ first: 1, // between\nsecond: 2 };\n",
    );
}

#[test]
fn test_for_destructuring_default_initializer() {
    expect_generated(
        "for (const [arrow = () => {}] = []; test;);",
        "for (const [arrow = () => {\n}] = []; test; ) ;\n",
    );
    expect_generated(
        "for (const { arrow = () => {} } = {}; test;);",
        "for (const { arrow = () => {\n} } = {}; test; ) ;\n",
    );
}

#[test]
fn test_for_of_async_identifier() {
    expect_generated("for ((async) of []) ;", "for ((async) of []) ;\n");
    expect_generated("for (\\u0061sync of []) ;", "for ((async) of []) ;\n");
    expect_generated("for (async.x of []) ;", "for (async.x of []) ;\n");
    expect_generated("for await (async of []) ;", "for await (async of []) ;\n");
}

#[test]
fn test_array_pattern_rest_after_elision() {
    expect_generated(
        "function foo([,,...rest]) {}",
        "function foo([, , ...rest]) {\n}\n",
    );
    expect_generated("([,,...rest] = value)", "[, , ...rest] = value;\n");
}

#[test]
fn test_for_initializer_comments() {
    let script_options = ParseOptions {
        source_type: SourceType::Script,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    expect_token_boundary_with_options(
        "for (/*keep*/(let)[0];false;);",
        "for(/*keep*/(let)[0];false;);",
        script_options,
    );

    expect_token_boundary_with_options(
        "for (// keep\n(let)[0];false;);",
        "for(// keep\n(let)[0];false;);",
        script_options,
    );
}

#[test]
fn test_for_initializers() {
    expect_stable(
        r#"for (obj = { get ["x" in empty]() {}, value: "x" in empty }; ;);"#,
        "for (obj = { get [\"x\" in empty]() {\n}, value: \"x\" in empty }; ; ) ;\n",
    );
    expect_generated_minify(
        r#"for (obj = { get ["x" in empty]() {}, value: "x" in empty }; ;);"#,
        "for(obj={get[\"x\"in empty](){},value:\"x\"in empty};;);",
    );
    expect_generated_minify(
        "for(obj={get[\"x\"in empty](){},value:\"x\"in empty};;);",
        "for(obj={get[\"x\"in empty](){},value:\"x\"in empty};;);",
    );
    expect_stable(
        r#"for (C = class { get ["x" in empty]() {} static set ["x" in empty](value) {} }; ;);"#,
        "for (C = class {\n  get [\"x\" in empty]() {\n  }\n  static set [\"x\" in empty](value) {\n  }\n}; ; ) ;\n",
    );
    expect_generated_minify(
        r#"for (C = class { get ["x" in empty]() {} static set ["x" in empty](value) {} }; ;);"#,
        "for(C=class{get[\"x\"in empty](){}static set[\"x\"in empty](value){}};;);",
    );
    expect_generated_minify(
        "for(C=class{get[\"x\"in empty](){}static set[\"x\"in empty](value){}};;);",
        "for(C=class{get[\"x\"in empty](){}static set[\"x\"in empty](value){}};;);",
    );

    let script = ParseOptions {
        source_type: SourceType::Script,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    crate::expect_generated_pair(
        "for (var i = (a in b) in c);",
        script,
        "for (var i = (a in b) in c) ;\n",
        "for(var i=(a in b)in c);",
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

fn inline_script_jsx_text(options: &GeneratorOptions) -> String {
    let parse_options = ParseOptions {
        lang: Lang::Jsx,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    crate::generate_edited("<>old</>;", parse_options, options, |ast| {
        let fragment = first_expression(ast).as_jsx_fragment(ast).unwrap();
        let text = fragment
            .children(ast)
            .get_node(ast, 0)
            .unwrap()
            .as_jsx_text(ast)
            .unwrap();
        let value = ast.add_utf8("</script");
        text.set_value(ast, value);
    })
}

#[test]
fn test_inline_script() {
    let inline = GeneratorOptions {
        inline_script: true,
        ..GeneratorOptions::default()
    };
    let ascii = GeneratorOptions {
        ascii_only: true,
        ..GeneratorOptions::default()
    };
    let minified = GeneratorOptions {
        minify: true,
        ..GeneratorOptions::default()
    };
    let inline_minified = GeneratorOptions {
        minify: true,
        ..inline.clone()
    };
    let inline_ascii = GeneratorOptions {
        inline_script: true,
        ..ascii.clone()
    };
    let jsx = ParseOptions {
        lang: Lang::Jsx,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    let typescript = ParseOptions {
        lang: Lang::Ts,
        ..jsx
    };

    expect_generated("x = `</script>`", "x = `</script>`;\n");

    expect_generated_with_options("x = `</script>`", "x = `</script>`;\n", ascii);

    expect_generated_with_options("x = `</script>`", "x=`</script>`;", minified);

    expect_generated_with_options("x = `</script>`", "x = `<\\/script>`;\n", inline.clone());

    expect_generated_with_options("x = `</script>`", "x = `<\\/script>`;\n", inline_ascii);

    expect_generated_with_options(
        "x = `</script>`",
        "x=`<\\/script>`;",
        inline_minified.clone(),
    );

    expect_generated_with_options(
        "String.raw`</script>`",
        "String.raw`</script>`;\n",
        inline.clone(),
    );

    expect_generated_with_options(r#""</SCRIPT>";"#, "\"<\\/SCRIPT>\";\n", inline.clone());

    expect_generated_with_options(
        r#""use\x20script";"#,
        "\"use\\x20script\";\n",
        inline.clone(),
    );

    expect_generated("//! </script>", "//! </script>\n");

    expect_generated_minify("x < /script/.test(y)", "x</script/.test(y);");

    expect_generated_minify("x / /script/.test(y)", "x/ /script/.test(y);");

    let source = "a = <a b=\"</script>\" />;";
    expect_generated_with_parse_options(source, "a = <a b=\"&lt;/script>\" />;\n", jsx, &inline);

    expect_generated_with_parse_options(
        source,
        "a=<a b=\"&lt;/script>\"/>;",
        jsx,
        &inline_minified,
    );

    let source = "a = <script>{x}</script>;";
    expect_generated_with_parse_options(source, "a = <script>{x}</ script>;\n", jsx, &inline);

    expect_generated_with_parse_options(source, "a=<script>{x}</ script>;", jsx, &inline_minified);

    let source = "type A = `</script>${string}`;";
    expect_generated_with_parse_options(
        source,
        "type A = `<\\/script>${string}`;\n",
        typescript,
        &inline,
    );

    expect_generated_with_parse_options(
        source,
        "type A=`<\\/script>${string}`;",
        typescript,
        &inline_minified,
    );

    let source = "#!/bin/node </script>\n";
    expect_generated_with_options(source, "#!/bin/node <\\/script>\n", inline.clone());

    expect_generated_with_options(source, "#!/bin/node <\\/script>\n", inline_minified.clone());

    let output = inline_script_jsx_text(&inline);
    assert_eq!(output, "<>&lt;/script</>;\n");

    let output = inline_script_jsx_text(&inline_minified);
    assert_eq!(output, "<>&lt;/script</>;");
}

#[track_caller]
fn synthetic_callee(source: &str, options: &GeneratorOptions) -> String {
    struct SyntheticCallee<'visit, 'ast>(&'visit mut Ast<'ast>);

    impl<'ast> VisitMut<'ast> for SyntheticCallee<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.0
        }

        fn visit_mut_identifier_reference(
            &mut self,
            node: IdentifierReference,
        ) -> Option<IdentifierReference> {
            if self.0.get_utf8(node.name(self.0)) == "value" {
                node.set_span(self.0, DUMMY_SP);
            }
            Some(node)
        }
    }

    let parse_options = ParseOptions {
        preserve_parens: false,
        ..ParseOptions::default()
    };
    crate::generate_edited(source, parse_options, options, |ast| {
        ast.root_program().visit_mut_with(&mut SyntheticCallee(ast));
    })
}

#[test]
fn test_comment_line_terminators() {
    let pretty = GeneratorOptions::default();
    let minified = GeneratorOptions {
        minify: true,
        ..pretty.clone()
    };

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

    let source = "function f() { return (\n// line\nvalue.has(name)); }";
    let output = synthetic_callee(source, &pretty);
    assert_eq!(
        output,
        "function f() {\n  return ( // line\n  value.has(name));\n}\n"
    );

    let output = synthetic_callee(source, &minified);
    assert_eq!(output, "function f(){return(// line\nvalue.has(name))}");

    let source = "throw (\n// line\nvalue.has(name));";
    let output = synthetic_callee(source, &pretty);
    assert_eq!(output, "throw ( // line\nvalue.has(name));\n");

    let output = synthetic_callee(source, &minified);
    assert_eq!(output, "throw(// line\nvalue.has(name));");

    let source = "function* f() { yield (\n// line\nvalue.has(name)); }";
    let output = synthetic_callee(source, &pretty);
    assert_eq!(
        output,
        "function* f() {\n  yield ( // line\n  value.has(name));\n}\n"
    );

    let output = synthetic_callee(source, &minified);
    assert_eq!(output, "function*f(){yield(// line\nvalue.has(name))}");
}

fn jsx_source_map(source: &str, options: &GeneratorOptions) -> Vec<u8> {
    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        lang: Lang::Jsx,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    let parsed = Parser::init(&allocator, source, parse_options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let output = generate(&parsed.ast, options.clone());
    output.map.unwrap().mappings().to_vec()
}

#[test]
fn test_jsx_source_maps() {
    let pretty = GeneratorOptions {
        source_map_path: Some("input.jsx".into()),
        ..GeneratorOptions::default()
    };
    let minified = GeneratorOptions {
        minify: true,
        ..pretty.clone()
    };
    let mappings = jsx_source_map("<C /\u{2028}>;", &pretty);
    assert!(!mappings.is_empty());

    let mappings = jsx_source_map("<></\u{2029}>;", &pretty);
    assert!(!mappings.is_empty());

    let mappings = jsx_source_map("<C\u{2028}></C>;", &minified);
    assert!(!mappings.is_empty());
}
