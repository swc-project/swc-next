use std::{
    fs,
    path::{Path, PathBuf},
};

use rstest::rstest;
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_parser::{EsSyntax, Parser, StringSource, Syntax};

#[rstest]
pub fn pass_tests(
    #[base_dir = "tests"]
    #[files("fixtures/*/pass/*.js")]
    #[files("fixtures/*/pass/*.cjs")]
    #[files("fixtures/*/pass/*.mjs")]
    #[files("fixtures/*/pass/*.jsx")]
    // #[files("fixtures/*/pass/*.ts")]
    // #[files("fixtures/*/pass/*.tsx")]
    path: PathBuf,
) {
    assert!(parse_case(&path));
}

#[rstest]
pub fn fail_tests(
    #[base_dir = "tests"]
    #[files("fixtures/*/fail/*.js")]
    #[files("fixtures/*/fail/*.jsx")]
    #[files("fixtures/*/fail/*.cjs")]
    #[files("fixtures/*/fail/*.mjs")]
    // #[files("fixtures/*/fail/*.ts")]
    // #[files("fixtures/*/fail/*.tsx")]
    // SWC passes these cases, which seems like bugs.
    #[exclude("errors-issue-387-4-input.jsx")]
    #[exclude("errors-html-comment-input.jsx")]
    path: PathBuf,
) {
    assert!(!parse_case(&path));
}

fn parse_case(path: &Path) -> bool {
    let code = fs::read_to_string(path).unwrap();
    let ext = path.extension().unwrap().to_string_lossy();

    let allocator = Allocator::new();
    let input = StringSource::new(&code);
    let mut parser = Parser::new(&allocator, syntax(path, &code), input, None);

    let result = match ext.as_ref() {
        "cjs" => parser.parse_script().map(|_| ()),
        "mjs" => parser.parse_module().map(|_| ()),
        "js" | "jsx" => parser.parse_program().map(|_| ()),
        _ => unreachable!(),
    };

    let errors = parser.take_errors();
    if result.is_err() || !errors.is_empty() {
        return false;
    }
    true
}

fn syntax(path: &Path, code: &str) -> Syntax {
    let ext = path.extension().unwrap().to_string_lossy();
    match ext.as_ref() {
        "js" | "cjs" | "mjs" | "jsx" => {
            let mut es = EsSyntax::default();
            if ext == "jsx" {
                es.jsx = true;
            }
            if code.contains("// @decorators") {
                es.decorators = true;
            }
            if code.contains("// @decorators_before_export") {
                es.decorators_before_export = true;
            }
            if code.contains("// @export_default_from") {
                es.export_default_from = true;
            }
            if code.contains("// @allow_super_outside_method") {
                es.allow_super_outside_method = true;
            }
            Syntax::Es(es)
        }
        "ts" | "cts" | "mts" | "tsx" => Syntax::Es(EsSyntax::default()),
        _ => unreachable!(),
    }
}
