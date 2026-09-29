use swc_next_ecma_ast::SourceType;
use swc_next_ecma_formatter::{FormatError, FormatOptions, IndentWidth, Semicolons, format};
use swc_next_ecma_parser::Options as ParserOptions;

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
