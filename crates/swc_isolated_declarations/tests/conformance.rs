use std::{
    env::var,
    fs,
    path::{Path, PathBuf},
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_isolated_declarations::transform;

fn sorted_directory_entries(path: &Path) -> Vec<PathBuf> {
    let mut entries = fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn run_fixture(input: &Path, lang: Lang) {
    let source = fs::read_to_string(input).unwrap();
    let base = input.with_extension("");
    let expected = fs::read_to_string(base.with_extension("output.d.ts")).unwrap();
    let errors = fs::read_to_string(base.with_extension("errors.txt")).unwrap();
    let expected_diagnostics = errors
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.split_once(": ").unwrap().1)
        .collect::<Vec<_>>();

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
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    let actual = print(&allocator, &ast, PrintOptions::default()).code;
    assert!(
        actual == expected,
        "{}: declaration mismatch",
        input.display()
    );
    assert!(
        diagnostics == expected_diagnostics,
        "{}: diagnostic mismatch",
        input.display()
    );
}
