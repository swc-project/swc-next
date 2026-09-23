use swc_next_ecma_ast::SourceType;
use swc_next_ecma_formatter::{FormatError, FormatOptions, IndentWidth, format};
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
