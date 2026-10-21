use std::{fs, path::Path};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_isolated_declarations::transform;

#[test]
fn integration() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for category in fs::read_dir(&root).unwrap() {
        let category = category.unwrap().path();
        if !category.is_dir() {
            continue;
        }
        for input in fs::read_dir(&category).unwrap() {
            let input = input.unwrap().path();
            let file_name = input.file_name().unwrap().to_string_lossy();
            let lang = if file_name.ends_with(".input.ts") {
                Lang::Ts
            } else if file_name.ends_with(".input.tsx") {
                Lang::Tsx
            } else {
                continue;
            };
            run_fixture(&input, lang);
        }
    }
}

fn run_fixture(input: &Path, lang: Lang) {
    let source = fs::read_to_string(input).unwrap();
    let base = input.with_extension("");
    let expected = fs::read_to_string(base.with_extension("output.d.ts"))
        .unwrap()
        .replace("\r\n", "\n");
    let errors = fs::read_to_string(base.with_extension("errors.txt")).unwrap();
    let expected_diagnostics = errors
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.split_once(": ").unwrap().1);

    let allocator = Allocator::default();
    let parse = Parser::init(
        &allocator,
        &source,
        Options {
            lang,
            source_type: SourceType::Unambiguous,
            comments: CommentMode::Flat,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(
        parse.diagnostics.is_empty(),
        "{}: parse diagnostics",
        input.display()
    );

    let mut ast = parse.ast;
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let diagnostics = transform(&mut ast, &semantic)
        .into_iter()
        .map(|diagnostic| diagnostic.message);
    let actual = print(&ast, PrintOptions::default()).code;
    assert!(
        actual == expected,
        "{}: declaration mismatch",
        input.display()
    );
    assert!(
        diagnostics.eq(expected_diagnostics),
        "{}: diagnostic mismatch",
        input.display()
    );
}
