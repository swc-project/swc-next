use std::{fmt::Write, fs, path::Path};

use serde_json::{Map, Value};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_formatter::{FormatOptions, format, format_ast};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};

use super::prettier_options;

fn option_sets(path: &Path, root: &Path) -> Vec<Map<String, Value>> {
    let mut directory = path.parent().unwrap();
    loop {
        let options = directory.join("options.json");
        if options.exists() {
            let source = fs::read_to_string(&options).unwrap();
            let sets: Vec<Map<String, Value>> = serde_json::from_str(&source)
                .unwrap_or_else(|error| panic!("{}: {error}", options.display()));
            assert!(!sets.is_empty(), "{} has no option sets", options.display());
            return sets;
        }
        if directory == root {
            return vec![Map::new()];
        }
        directory = directory.parent().unwrap();
    }
}

pub fn test_file(relative: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let path = root.join(relative);
    let source = fs::read_to_string(&path).unwrap();
    let parser_options = ParserOptions {
        lang: Lang::from_path(relative),
        source_type: SourceType::from_path(relative),
        comments: CommentMode::Flat,
        ..ParserOptions::default()
    };
    let mut snapshot = format!("---------- Input ----------\n{source}");
    for spec in option_sets(&path, &root) {
        let options = prettier_options::format_options(&spec, &source).unwrap();
        let output = format_pass(relative, parser_options, options, &source);
        writeln!(
            snapshot,
            "---------- Options ----------\n{}",
            serde_json::to_string(&spec).unwrap()
        )
        .unwrap();
        writeln!(snapshot, "---------- Output ----------\n{output}").unwrap();
        let second_output = format_pass(relative, parser_options, options, &output);
        assert_eq!(second_output, output, "{relative} formatting idempotence");
    }
    insta::with_settings!({
        snapshot_path => path.parent().unwrap(),
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(path.file_name().unwrap().to_str().unwrap(), snapshot);
    });
}

fn format_pass(
    id: &str,
    parser_options: ParserOptions,
    options: FormatOptions,
    input: &str,
) -> String {
    let output = format(input, parser_options, options)
        .unwrap_or_else(|error| panic!("{id} text entry point failed: {error}"));

    let allocator = Allocator::new();
    let parsed = Parser::init(&allocator, input, parser_options, NoTokenParserConfig).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "{id} fixture must parse without diagnostics"
    );
    let ast_output = format_ast(&parsed.ast, options)
        .unwrap_or_else(|error| panic!("{id} AST entry point failed: {error}"));
    assert_eq!(ast_output, output, "{id} entry-point parity");

    let output_allocator = Allocator::new();
    let reparsed = Parser::init(
        &output_allocator,
        &output,
        parser_options,
        NoTokenParserConfig,
    )
    .parse();
    assert!(
        reparsed.diagnostics.is_empty(),
        "{id} formatted output must parse without diagnostics"
    );
    assert_eq!(
        parsed.ast.comments().len(),
        reparsed.ast.comments().len(),
        "{id} formatting must preserve comment count"
    );
    output
}
