use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_formatter::{
    FormatError, FormatOptions, IndentWidth, Semicolons, format, format_ast,
};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};

#[test]
fn indentation_width_bounds() {
    assert_eq!(IndentWidth::try_from(0).unwrap().value(), 0);
    assert_eq!(IndentWidth::try_from(64).unwrap().value(), 64);
    assert!(IndentWidth::try_from(65).is_err());
    assert!(IndentWidth::try_from(255).is_err());
}

#[test]
fn parser_options() {
    let options = FormatOptions::prettier();
    let invalid = format("const = ;", ParserOptions::default(), options);
    assert!(matches!(invalid, Err(FormatError::Parse(_))));

    let module = ParserOptions {
        source_type: SourceType::Module,
        ..ParserOptions::default()
    };
    let reserved = format("await;", module, options);
    assert!(matches!(reserved, Err(FormatError::Parse(_))));

    let script = ParserOptions {
        source_type: SourceType::Script,
        ..ParserOptions::default()
    };
    let identifier = format("await;", script, options);
    assert!(matches!(
        identifier,
        Ok(_) | Err(FormatError::Unsupported { .. })
    ));
}

fn assert_stable(input: &str, options: FormatOptions) {
    let first = format(input, ParserOptions::default(), options).unwrap();
    let second = format(&first, ParserOptions::default(), options).unwrap();
    assert_eq!(first, second);
}

#[test]
fn comment_layout_stability() {
    let options = FormatOptions {
        line_width: 20,
        semicolons: Semicolons::AsNeeded,
        ..FormatOptions::prettier()
    };

    assert_stable(
        "(/** @type {number} */ first || second || third) + fourth;",
        options,
    );

    assert_stable(
        "fourth + (/** @type {number} */ first || second || third);",
        options,
    );

    assert_stable(
        "(/** @satisfies {number} */ [first, second, third]) + fourth;",
        options,
    );

    assert_stable(
        "result = (/** @type {number} */ left ? middle : right)(arg);",
        options,
    );

    assert_stable("call() /* #__PURE__ */;\nnext();", options);

    assert_stable(
        "call() /* #__PURE__ */;\nnext();",
        FormatOptions::prettier(),
    );

    assert_stable("call() /** @type {number} */;\n(value);", options);

    assert_stable(
        "call() /** @type {number} */;\n(value);",
        FormatOptions::prettier(),
    );
}

#[test]
fn ambient_module_attribute_recovery() {
    let allocator = Allocator::new();
    let parser_options = ParserOptions {
        lang: Lang::Dts,
        source_type: SourceType::Unambiguous,
        ..ParserOptions::default()
    };
    let source = "declare module '*.data' /* before */ with /* after */ { readonly type:'json'|'json5'; 'resolution-mode':'import' };\n";
    let parsed = Parser::init(&allocator, source, parser_options, NoTokenParserConfig).parse();
    assert_eq!(parsed.diagnostics.len(), 1);
    let result = format(source, parser_options, FormatOptions::prettier());
    assert!(matches!(result, Err(FormatError::Parse(errors)) if errors.len() == 1));

    for (line_width, semicolons, expected) in [
        (
            20,
            Semicolons::Always,
            "declare module \"*.data\" /* before */ with /* after */ {\n  readonly type:\n    | \"json\"\n    | \"json5\";\n  \"resolution-mode\": \"import\";\n};\n",
        ),
        (
            80,
            Semicolons::Always,
            "declare module \"*.data\" /* before */ with /* after */ {\n  readonly type: \"json\" | \"json5\";\n  \"resolution-mode\": \"import\";\n};\n",
        ),
        (
            20,
            Semicolons::AsNeeded,
            "declare module \"*.data\" /* before */ with /* after */ {\n  readonly type:\n    \"json\" | \"json5\"\n  \"resolution-mode\": \"import\"\n}\n",
        ),
        (
            80,
            Semicolons::AsNeeded,
            "declare module \"*.data\" /* before */ with /* after */ {\n  readonly type: \"json\" | \"json5\"\n  \"resolution-mode\": \"import\"\n}\n",
        ),
    ] {
        let options = FormatOptions {
            line_width,
            semicolons,
            ..FormatOptions::prettier()
        };
        let output = format_ast(&parsed.ast, options).unwrap();
        assert_eq!(output, expected);
        let reparsed =
            Parser::init(&allocator, &output, parser_options, NoTokenParserConfig).parse();
        assert_eq!(reparsed.diagnostics.len(), 1);
        assert_eq!(reparsed.ast.comments().len(), parsed.ast.comments().len());
        assert_eq!(format_ast(&reparsed.ast, options).unwrap(), output);
    }
}
