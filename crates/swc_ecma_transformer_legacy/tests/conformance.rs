use std::{
    collections::BTreeMap,
    fmt::Write,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{AstCtx, Diagnostic, Severity};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_transformer_legacy::Transformer;

#[path = "conformance/options.rs"]
mod options;

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn fixtures() -> PathBuf {
    crate_root().join("../../tasks/testsuite/fixtures/swc/crates/swc/tests/compile")
}

fn inputs(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    assert!(
        root.is_dir(),
        "Initialize SWC fixtures with scripts/clone_fixtures.sh"
    );
    let mut inputs = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry.unwrap();
        if !entry.file_type().is_dir() || entry.file_name() != "input" {
            continue;
        }
        let input = entry.path();
        for entry in WalkDir::new(&input).skip_hidden(false) {
            let entry = entry.unwrap();
            let path = entry.path();
            if !entry.file_type().is_file()
                || !matches!(
                    path.extension().and_then(|v| v.to_str()),
                    Some("js" | "jsx" | "ts" | "tsx")
                )
            {
                continue;
            }
            let output = input
                .parent()
                .unwrap()
                .join("output")
                .join(path.strip_prefix(&input).unwrap());
            inputs.push((path, output));
        }
    }
    inputs.sort_unstable();
    assert!(!inputs.is_empty(), "SWC compile fixture set is empty");
    inputs
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut path = path.as_os_str().to_os_string();
    path.push(suffix);
    path.into()
}

enum Outcome {
    Passed,
    Failed(String),
    Skipped(&'static str),
}

fn diagnostic(diagnostics: &[Diagnostic]) -> Option<String> {
    diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
        .map(|diagnostic| {
            let mut message = diagnostic.message.to_string();
            if let Some(help) = &diagnostic.help {
                write!(message, ": {help}").unwrap();
            }
            message
        })
}

fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
) -> Result<AstCtx<'a>, String> {
    let parsed = Parser::init(allocator, source, options, NoTokenParserConfig).parse();
    if let Some(error) = diagnostic(&parsed.diagnostics) {
        return Err(format!("parser: {error}"));
    }
    Ok(parsed.ast)
}

fn normalize(
    source: &str,
    mut parser: ParserOptions,
    mut generator: GeneratorOptions,
) -> Result<String, String> {
    let allocator = Allocator::new();
    parser.preserve_parens = false;
    let ast = parse(&allocator, source, parser)?;
    generator.minify = true;
    Ok(generate(&ast, generator).code)
}

fn run(input: &Path, output: &Path, root: &Path) -> Outcome {
    let config = match options::read(input, root) {
        Ok(config) => config,
        Err(reason) => return Outcome::Skipped(reason),
    };
    for suffix in [".stderr", ".map", ".d.ts", ".extra.json"] {
        let path = with_suffix(output, suffix);
        if path.is_file() && !fs::read_to_string(path).unwrap().is_empty() {
            return Outcome::Skipped("non-code snapshot");
        }
    }
    let raw = with_suffix(output, ".raw");
    let output = if raw.is_file() { raw.as_path() } else { output };
    if !output.is_file() {
        return Outcome::Skipped("missing code snapshot");
    }
    let source = fs::read_to_string(input).unwrap();
    let expected = fs::read_to_string(output).unwrap();
    let allocator = Allocator::new();
    let mut ast = match parse(&allocator, &source, config.parser) {
        Ok(ast) => ast,
        Err(error) => return Outcome::Failed(error),
    };
    let result = Transformer::transform(&mut ast, config.transformer);
    if let Some(error) = diagnostic(&result.diagnostics) {
        return Outcome::Failed(format!("transform: {error}"));
    }
    let actual = generate(&ast, config.generator.clone()).code;
    // Normalize both sides with the same parser/printer, keeping comments and
    // token distinctions while ignoring formatting and redundant parentheses.
    let actual = match normalize(&actual, config.parser, config.generator.clone()) {
        Ok(actual) => actual,
        Err(error) => return Outcome::Failed(format!("output {error}")),
    };
    let expected = match normalize(&expected, config.parser, config.generator) {
        Ok(expected) => expected,
        Err(_) => return Outcome::Skipped("unparseable reference output"),
    };
    if actual == expected {
        Outcome::Passed
    } else {
        Outcome::Failed("transformed output differs from SWC snapshot".into())
    }
}

#[test]
#[ignore = "requires pinned SWC fixtures; run with --ignored"]
fn swc() {
    // Large upstream inputs require the same stack budget as testsuite runners.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run_suite)
        .unwrap()
        .join()
        .unwrap();
}

fn run_suite() {
    let root = fixtures();
    let cases = inputs(&root);
    let mut passed = 0;
    let mut failed = Vec::new();
    let mut panics = Vec::new();
    let mut skipped = BTreeMap::<&str, usize>::new();
    for (input, output) in &cases {
        let path = input
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        match catch_unwind(AssertUnwindSafe(|| run(input, output, &root))) {
            Ok(Outcome::Passed) => passed += 1,
            Ok(Outcome::Failed(error)) => failed.push(format!("Failed: {path} - {error}")),
            Ok(Outcome::Skipped(reason)) => *skipped.entry(reason).or_default() += 1,
            Err(_) => panics.push(format!("Panic: {path}")),
        }
    }
    let skipped_count = skipped.values().sum::<usize>();
    let mut snapshot = format!(
        "transform_legacy * swc\n========================\nTotal: {}\nPassed: {passed}\nFailed: {}\nPanicked: {}\nSkipped: {skipped_count}\n",
        cases.len(),
        failed.len(),
        panics.len(),
    );
    for (reason, count) in skipped {
        writeln!(snapshot, "Skipped ({reason}): {count}").unwrap();
    }
    for line in failed.iter().chain(&panics) {
        writeln!(snapshot, "{line}").unwrap();
    }
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(crate_root().join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.set_omit_expression(true);
    settings.bind(|| insta::assert_snapshot!("conformance_swc", snapshot));
    assert!(panics.is_empty(), "SWC conformance contained panics");
}
